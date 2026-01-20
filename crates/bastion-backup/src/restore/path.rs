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
