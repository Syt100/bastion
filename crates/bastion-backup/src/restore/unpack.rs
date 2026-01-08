use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use super::{ConflictPolicy, RestoreSelection};

#[derive(Debug, Clone)]
pub(super) enum PayloadDecryption {
    None,
    AgeX25519 { identity: String },
}

pub(super) fn restore_from_parts(
    part_paths: &[PathBuf],
    destination_dir: &Path,
    conflict: ConflictPolicy,
    decryption: PayloadDecryption,
    selection: Option<&RestoreSelection>,
) -> Result<(), anyhow::Error> {
    std::fs::create_dir_all(destination_dir)?;

    let selection = selection.map(normalize_restore_selection).transpose()?;

    let files = part_paths
        .iter()
        .map(File::open)
        .collect::<Result<Vec<_>, _>>()?;
    let reader = ConcatReader { files, index: 0 };
    let reader: Box<dyn Read> = match decryption {
        PayloadDecryption::None => Box::new(reader),
        PayloadDecryption::AgeX25519 { identity } => {
            use std::str::FromStr as _;

            let identity =
                age::x25519::Identity::from_str(identity.trim()).map_err(|e| anyhow::anyhow!(e))?;
            let decryptor = age::Decryptor::new(reader)?;
            let reader = decryptor.decrypt(std::iter::once(&identity as &dyn age::Identity))?;
            Box::new(reader)
        }
    };
    let decoder = zstd::Decoder::new(reader)?;
    let mut archive = tar::Archive::new(decoder);
    archive.set_unpack_xattrs(false);
    archive.set_preserve_mtime(true);

    for entry in archive.entries()? {
        let mut entry = entry?;
        let rel = entry.path()?.to_path_buf();

        let rel_match = archive_path_for_match(&rel)
            .ok_or_else(|| anyhow::anyhow!("invalid entry path: {}", rel.display()))?;
        if let Some(selection) = selection.as_ref()
            && !selection.matches(&rel_match)
        {
            continue;
        }

        let dest_path = safe_join(destination_dir, &rel)
            .ok_or_else(|| anyhow::anyhow!("invalid entry path: {}", rel.display()))?;

        match conflict {
            ConflictPolicy::Overwrite => {
                if dest_path.exists() {
                    remove_existing_path(&dest_path)?;
                }
            }
            ConflictPolicy::Skip => {
                if dest_path.exists() {
                    continue;
                }
            }
            ConflictPolicy::Fail => {
                if dest_path.exists() {
                    anyhow::bail!("restore conflict: {} exists", dest_path.display());
                }
            }
        }

        entry.unpack_in(destination_dir)?;
    }

    Ok(())
}

#[derive(Debug)]
struct NormalizedRestoreSelection {
    files: std::collections::HashSet<String>,
    dirs: Vec<String>,
}

impl NormalizedRestoreSelection {
    fn matches(&self, archive_path: &str) -> bool {
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

fn normalize_restore_selection(
    selection: &RestoreSelection,
) -> Result<NormalizedRestoreSelection, anyhow::Error> {
    let mut files = std::collections::HashSet::<String>::new();
    let mut dirs = std::collections::HashSet::<String>::new();

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

fn archive_path_for_match(rel: &Path) -> Option<String> {
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

fn remove_existing_path(path: &Path) -> Result<(), anyhow::Error> {
    let meta = std::fs::symlink_metadata(path)?;
    if meta.is_dir() {
        std::fs::remove_dir_all(path)?;
    } else {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

struct ConcatReader {
    files: Vec<File>,
    index: usize,
}

impl Read for ConcatReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        loop {
            if self.index >= self.files.len() {
                return Ok(0);
            }
            let n = self.files[self.index].read(buf)?;
            if n == 0 {
                self.index += 1;
                continue;
            }
            return Ok(n);
        }
    }
}
