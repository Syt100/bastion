use std::fs::File;
use std::io::BufRead;
use std::path::Path;

use sqlx::SqlitePool;

use bastion_storage::secrets::SecretsCrypto;

use super::super::access;
use super::ListChildrenFromEntriesIndexOptions;
use super::fetch_entries_index;
use super::types::EntryRecord;
use super::{ListRunEntriesChildrenOptions, RunEntriesChild, RunEntriesChildrenResponse};

#[allow(clippy::too_many_arguments)]
pub async fn list_run_entries_children(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    data_dir: &Path,
    run_id: &str,
    prefix: Option<&str>,
    cursor: u64,
    limit: u64,
    q: Option<&str>,
    kind: Option<&str>,
    hide_dotfiles: bool,
    min_size_bytes: Option<u64>,
    max_size_bytes: Option<u64>,
    type_sort_file_first: bool,
) -> Result<RunEntriesChildrenResponse, anyhow::Error> {
    list_run_entries_children_with_options(
        db,
        secrets,
        data_dir,
        run_id,
        ListRunEntriesChildrenOptions {
            prefix: prefix.map(str::to_string),
            cursor,
            limit,
            q: q.map(str::to_string),
            kind: kind.map(str::to_string),
            hide_dotfiles,
            min_size_bytes,
            max_size_bytes,
            type_sort_file_first,
        },
    )
    .await
}

pub async fn list_run_entries_children_with_options(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    data_dir: &Path,
    run_id: &str,
    options: ListRunEntriesChildrenOptions,
) -> Result<RunEntriesChildrenResponse, anyhow::Error> {
    let ListRunEntriesChildrenOptions {
        prefix,
        cursor,
        limit,
        q,
        kind,
        hide_dotfiles,
        min_size_bytes,
        max_size_bytes,
        type_sort_file_first,
    } = options;

    let access::ResolvedRunAccess { access, .. } =
        access::resolve_success_run_access(db, secrets, run_id).await?;

    let cache_dir = data_dir.join("cache").join("entries").join(run_id);
    tokio::fs::create_dir_all(&cache_dir).await?;
    let entries_path = fetch_entries_index(&access, &cache_dir).await?;

    let prefix = prefix.unwrap_or_default();
    let prefix = prefix
        .trim()
        .trim_start_matches('/')
        .trim_end_matches('/')
        .to_string();
    let cursor = cursor as usize;
    let limit = limit.clamp(1, 1000) as usize;

    let list_options = ListChildrenFromEntriesIndexOptions {
        prefix,
        cursor,
        limit,
        q,
        kind,
        hide_dotfiles,
        min_size_bytes,
        max_size_bytes,
        type_sort_file_first,
    };
    tokio::task::spawn_blocking(move || {
        super::list_children_from_entries_index(&entries_path, list_options)
    })
    .await?
}

pub(in crate::restore) fn list_children_from_entries_index(
    entries_path: &Path,
    options: ListChildrenFromEntriesIndexOptions,
) -> Result<RunEntriesChildrenResponse, anyhow::Error> {
    use std::collections::HashMap;

    #[derive(Debug)]
    struct ChildAgg {
        kind: String,
        size: u64,
    }

    let ListChildrenFromEntriesIndexOptions {
        prefix,
        cursor,
        limit,
        q,
        kind,
        hide_dotfiles,
        min_size_bytes,
        max_size_bytes,
        type_sort_file_first,
    } = options;

    let file = File::open(entries_path)?;
    let decoder = zstd::Decoder::new(file)?;
    let reader = std::io::BufReader::new(decoder);

    let prefix = prefix.trim().trim_start_matches('/').trim_end_matches('/');
    let prefix_slash = if prefix.is_empty() {
        String::new()
    } else {
        format!("{prefix}/")
    };

    let mut children = HashMap::<String, ChildAgg>::new();
    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let rec: EntryRecord = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let path = rec.path;
        let remainder: &str = if prefix.is_empty() {
            path.as_str()
        } else if path == prefix {
            continue;
        } else if let Some(rest) = path.strip_prefix(&prefix_slash) {
            rest
        } else {
            continue;
        };

        if remainder.is_empty() {
            continue;
        }

        let (child_name, has_more) = match remainder.split_once('/') {
            Some((first, _rest)) => (first, true),
            None => (remainder, false),
        };
        if child_name.is_empty() {
            continue;
        }
        if hide_dotfiles && child_name.starts_with('.') {
            continue;
        }

        let child_path = if prefix.is_empty() {
            child_name.to_string()
        } else {
            format!("{prefix}/{child_name}")
        };

        let inferred_dir = has_more;
        let kind = if inferred_dir {
            "dir".to_string()
        } else {
            rec.kind
        };
        let kind = if matches!(kind.as_str(), "file" | "dir" | "symlink") {
            kind
        } else if inferred_dir {
            "dir".to_string()
        } else {
            "file".to_string()
        };
        let size = if matches!(kind.as_str(), "file" | "symlink") {
            rec.size
        } else {
            0
        };

        children
            .entry(child_path)
            .and_modify(|existing| {
                if existing.kind != "dir" && kind == "dir" {
                    existing.kind = "dir".to_string();
                    existing.size = 0;
                    return;
                }
                if existing.kind == kind && existing.kind != "dir" {
                    existing.size = existing.size.max(size);
                }
            })
            .or_insert(ChildAgg { kind, size });
    }

    let mut entries = children
        .into_iter()
        .map(|(path, agg)| RunEntriesChild {
            path,
            kind: agg.kind,
            size: agg.size,
        })
        .collect::<Vec<_>>();

    if let Some(kind) = kind.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
        entries.retain(|e| e.kind == kind);
    }

    let q = q.as_deref().map(str::trim).filter(|v| !v.is_empty());
    if let Some(q) = q {
        let needle = q.to_lowercase();
        entries.retain(|e| {
            let name = e.path.rsplit('/').next().unwrap_or(e.path.as_str());
            name.to_lowercase().contains(&needle)
        });
    }

    if min_size_bytes.is_some() || max_size_bytes.is_some() {
        let min = min_size_bytes.unwrap_or(0);
        let max = max_size_bytes.unwrap_or(u64::MAX);
        entries.retain(|e| {
            if e.kind == "dir" {
                true
            } else {
                e.size >= min && e.size <= max
            }
        });
    }

    fn kind_rank(kind: &str, file_first: bool) -> u8 {
        match (file_first, kind) {
            (false, "dir") => 0,
            (false, "file") => 1,
            (false, "symlink") => 2,
            (true, "file") => 0,
            (true, "symlink") => 0,
            (true, "dir") => 1,
            _ => 3,
        }
    }

    entries.sort_by(|a, b| {
        kind_rank(&a.kind, type_sort_file_first)
            .cmp(&kind_rank(&b.kind, type_sort_file_first))
            .then_with(|| a.path.cmp(&b.path))
    });

    let total = entries.len();
    let start = cursor.min(total);
    let end = start.saturating_add(limit).min(total);
    let next_cursor = if end < total { Some(end as u64) } else { None };
    let page = entries
        .into_iter()
        .skip(start)
        .take(limit)
        .collect::<Vec<_>>();

    Ok(RunEntriesChildrenResponse {
        prefix: prefix.to_string(),
        cursor: start as u64,
        next_cursor,
        entries: page,
    })
}
