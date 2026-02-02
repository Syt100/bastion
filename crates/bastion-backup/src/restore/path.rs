use std::path::{Path, PathBuf};

pub(super) fn safe_join(base: &Path, rel: &Path) -> Option<PathBuf> {
    let mut out = PathBuf::from(base);
    for c in rel.components() {
        match c {
            std::path::Component::Normal(p) => out.push(p),
            std::path::Component::CurDir => {}
            _ => return None,
        }
    }
    Some(out)
}

pub(super) fn archive_path_for_match(rel: &Path) -> Option<String> {
    let mut parts = Vec::<String>::new();
    for c in rel.components() {
        match c {
            std::path::Component::Normal(p) => parts.push(p.to_string_lossy().to_string()),
            std::path::Component::CurDir => {}
            _ => return None,
        }
    }
    Some(parts.join("/"))
}

#[cfg(test)]
mod tests {
    use super::archive_path_for_match;
    use super::safe_join;

    use std::path::Path;

    #[test]
    fn safe_join_allows_normal_and_curdir_and_rejects_parent() {
        let base = Path::new("/base");
        assert_eq!(
            safe_join(base, Path::new("./a/./b")).as_deref(),
            Some(Path::new("/base/a/b"))
        );
        assert!(safe_join(base, Path::new("../etc")).is_none());
        assert!(safe_join(base, Path::new("a/../b")).is_none());
    }

    #[test]
    fn archive_path_for_match_normalizes_curdir_and_rejects_traversal() {
        assert_eq!(
            archive_path_for_match(Path::new("./a/./b")),
            Some("a/b".to_string())
        );
        assert!(archive_path_for_match(Path::new("../etc")).is_none());
        assert!(archive_path_for_match(Path::new("a/../b")).is_none());
    }

    #[cfg(unix)]
    #[test]
    fn archive_path_for_match_rejects_absolute_paths() {
        assert!(archive_path_for_match(Path::new("/etc/passwd")).is_none());
    }
}
