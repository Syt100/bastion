use std::io::Read;
use std::path::Path;

use super::RestoreSelection;
use super::path;
use super::sinks::RestoreSink;

#[derive(Debug, Clone)]
pub(super) enum PayloadDecryption {
    None,
    AgeX25519 { identity: String },
}

pub(super) struct RestoreEngine<'a, S: RestoreSink> {
    sink: &'a mut S,
    decryption: PayloadDecryption,
    selection: Option<NormalizedRestoreSelection>,
}

impl<'a, S: RestoreSink> RestoreEngine<'a, S> {
    pub(super) fn new(
        sink: &'a mut S,
        decryption: PayloadDecryption,
        selection: Option<&RestoreSelection>,
    ) -> Result<Self, anyhow::Error> {
        Ok(Self {
            sink,
            decryption,
            selection: selection.map(normalize_restore_selection).transpose()?,
        })
    }

    pub(super) fn restore(&mut self, payload: Box<dyn Read>) -> Result<(), anyhow::Error> {
        self.sink.prepare()?;

        let reader: Box<dyn Read> = match self.decryption.clone() {
            PayloadDecryption::None => payload,
            PayloadDecryption::AgeX25519 { identity } => {
                use std::str::FromStr as _;

                let identity = age::x25519::Identity::from_str(identity.trim())
                    .map_err(|e| anyhow::anyhow!(e))?;
                let decryptor = age::Decryptor::new(payload)?;
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
            let rel_raw = entry.path()?.to_path_buf();

            let rel_match = path::archive_path_for_match(&rel_raw)
                .ok_or_else(|| anyhow::anyhow!("invalid entry path: {}", rel_raw.display()))?;
            if let Some(selection) = self.selection.as_ref()
                && !selection.matches(&rel_match)
            {
                continue;
            }

            let rel = path::safe_join(Path::new(""), &rel_raw)
                .ok_or_else(|| anyhow::anyhow!("invalid entry path: {}", rel_raw.display()))?;

            self.sink.apply_entry(&mut entry, &rel)?;
        }

        Ok(())
    }
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
