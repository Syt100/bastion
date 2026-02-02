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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn normalize_restore_path_strips_and_normalizes_separators() {
        assert_eq!(
            normalize_restore_path("  ./a\\b/c/  ", false),
            Some("a/b/c".to_string())
        );
        assert_eq!(
            normalize_restore_path("/a/b/c///", false),
            Some("a/b/c".to_string())
        );
        assert_eq!(
            normalize_restore_path("/a/b/c///", true),
            Some("a/b/c".to_string())
        );
    }

    #[test]
    fn normalize_restore_path_rejects_empty_and_traversal() {
        assert_eq!(normalize_restore_path("", false), None);
        assert_eq!(normalize_restore_path("   ", false), None);
        assert_eq!(normalize_restore_path("/", false), None);
        assert_eq!(normalize_restore_path("../etc", false), None);
        assert_eq!(normalize_restore_path("a/../b", false), None);
        assert_eq!(normalize_restore_path("a/..", false), None);
        assert_eq!(normalize_restore_path("..", false), None);
    }

    #[test]
    fn normalize_restore_selection_errors_on_empty_after_normalization() {
        let sel = RestoreSelection {
            files: vec!["".to_string(), "../a".to_string()],
            dirs: vec!["/".to_string(), "a/..".to_string()],
        };
        assert!(normalize_restore_selection(&sel).is_err());
    }

    #[test]
    fn normalize_restore_selection_dedupes_and_sorts_dirs_longest_first()
    -> Result<(), anyhow::Error> {
        let sel = RestoreSelection {
            files: vec![
                "a/b.txt".to_string(),
                "./a/b.txt".to_string(),
                "a\\b.txt".to_string(),
            ],
            dirs: vec!["a/".to_string(), "a/b/".to_string(), "/a/b/".to_string()],
        };
        let out = normalize_restore_selection(&sel)?;

        assert_eq!(out.files.len(), 1);
        assert!(out.files.contains("a/b.txt"));

        // Longest first to make prefix checks cheap and deterministic.
        assert_eq!(out.dirs, vec!["a/b".to_string(), "a".to_string()]);
        Ok(())
    }

    #[test]
    fn matches_respects_file_exact_and_dir_prefix_boundaries() {
        let mut files = HashSet::new();
        files.insert("a/file.txt".to_string());

        let sel = NormalizedRestoreSelection {
            files,
            dirs: vec!["dir".to_string(), "a".to_string()],
        };

        assert!(sel.matches("a/file.txt"));
        assert!(sel.matches("dir"));
        assert!(sel.matches("dir/sub.txt"));

        // Prefix must be a directory boundary.
        assert!(!sel.matches("directory/sub.txt"));
        assert!(!sel.matches("dir2/sub.txt"));
        assert!(!sel.matches("ab/c.txt"));
    }
}
