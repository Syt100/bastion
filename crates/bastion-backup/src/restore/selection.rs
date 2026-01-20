use std::collections::HashSet;

use super::RestoreSelection;

#[derive(Debug)]
pub(super) struct NormalizedRestoreSelection {
    pub(super) files: HashSet<String>,
    pub(super) dirs: Vec<String>,
}

impl NormalizedRestoreSelection {
    pub(super) fn matches(&self, archive_path: &str) -> bool {
        if self.files.contains(archive_path) {
            return true;
        }
        for dir in &self.dirs {
            if archive_path == dir {
                return true;
            }
            if archive_path.starts_with(dir)
                && archive_path.as_bytes().get(dir.len()) == Some(&b'/')
            {
                return true;
            }
        }
        false
    }
}

pub(super) fn normalize_restore_selection(
    selection: &RestoreSelection,
) -> Result<NormalizedRestoreSelection, anyhow::Error> {
    let mut files = HashSet::<String>::new();
    let mut dirs = HashSet::<String>::new();

    for f in &selection.files {
        if let Some(v) = normalize_restore_path(f, false) {
            files.insert(v);
        }
    }
    for d in &selection.dirs {
        if let Some(v) = normalize_restore_path(d, true) {
            dirs.insert(v.trim_end_matches('/').to_string());
        }
    }

    if files.is_empty() && dirs.is_empty() {
        anyhow::bail!("restore selection is empty");
    }

    let mut dirs = dirs.into_iter().collect::<Vec<_>>();
    dirs.sort_by_key(|v| std::cmp::Reverse(v.len())); // longest first for prefix checks
    Ok(NormalizedRestoreSelection { files, dirs })
}

fn normalize_restore_path(path: &str, allow_trailing_slash: bool) -> Option<String> {
    let mut s = path.trim().replace('\\', "/");
    if s.is_empty() {
        return None;
    }
    while s.starts_with("./") {
        s = s.trim_start_matches("./").to_string();
    }
    while s.starts_with('/') {
        s = s.trim_start_matches('/').to_string();
    }
    if !allow_trailing_slash {
        while s.ends_with('/') {
            s = s.trim_end_matches('/').to_string();
        }
    }
    let s = s.trim_matches('/').to_string();
    if s.is_empty() {
        return None;
    }
    if s.split('/').any(|seg| seg == "..") {
        return None;
    }
    Some(s)
}
