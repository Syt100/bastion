use std::collections::HashSet;
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};

use super::ConflictPolicy;
use super::entries_index::EntryRecord;
use bastion_targets::WebdavClient;
use serde::Serialize;
use tokio::runtime::Handle;
use url::Url;
use uuid::Uuid;

pub(super) trait RestoreSink {
    fn prepare(&mut self) -> Result<(), anyhow::Error>;

    fn apply_entry<R: Read>(
        &mut self,
        entry: &mut tar::Entry<R>,
        rel_path: &Path,
    ) -> Result<(), anyhow::Error>;
}

pub(super) struct LocalFsSink {
    base_dir: PathBuf,
    conflict: ConflictPolicy,
}

impl LocalFsSink {
    pub(super) fn new(base_dir: PathBuf, conflict: ConflictPolicy) -> Self {
        Self { base_dir, conflict }
    }
}

impl RestoreSink for LocalFsSink {
    fn prepare(&mut self) -> Result<(), anyhow::Error> {
        std::fs::create_dir_all(&self.base_dir)?;
        Ok(())
    }

    fn apply_entry<R: Read>(
        &mut self,
        entry: &mut tar::Entry<R>,
        rel_path: &Path,
    ) -> Result<(), anyhow::Error> {
        let dest_path = self.base_dir.join(rel_path);

        match self.conflict {
            ConflictPolicy::Overwrite => {
                if dest_path.exists() {
                    remove_existing_path(&dest_path)?;
                }
            }
            ConflictPolicy::Skip => {
                if dest_path.exists() {
                    return Ok(());
                }
            }
            ConflictPolicy::Fail => {
                if dest_path.exists() {
                    anyhow::bail!("restore conflict: {} exists", dest_path.display());
                }
            }
        }

        // `unpack_in` also provides its own path traversal checks. We validate paths separately
        // (engine stage) and treat "skipped" as an error to preserve existing behavior.
        let unpacked = entry.unpack_in(&self.base_dir)?;
        if !unpacked {
            anyhow::bail!("invalid tar entry path: {}", rel_path.display());
        }
        Ok(())
    }
}

pub(super) fn remove_existing_path(path: &Path) -> Result<(), anyhow::Error> {
    let meta = std::fs::symlink_metadata(path)?;
    if meta.is_dir() {
        std::fs::remove_dir_all(path)?;
    } else {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::remove_existing_path;

    #[test]
    fn remove_existing_path_removes_file() -> Result<(), anyhow::Error> {
        let tmp = tempfile::TempDir::new()?;
        let path = tmp.path().join("file.txt");
        std::fs::write(&path, b"hi")?;

        remove_existing_path(&path)?;
        assert!(!path.exists());
        Ok(())
    }

    #[test]
    fn remove_existing_path_removes_directory_recursively() -> Result<(), anyhow::Error> {
        let tmp = tempfile::TempDir::new()?;
        let dir = tmp.path().join("dir");
        std::fs::create_dir_all(dir.join("nested"))?;
        std::fs::write(dir.join("nested").join("file.txt"), b"hi")?;

        remove_existing_path(&dir)?;
        assert!(!dir.exists());
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn remove_existing_path_removes_symlink_without_touching_target() -> Result<(), anyhow::Error> {
        use std::os::unix::fs as unix_fs;

        let tmp = tempfile::TempDir::new()?;
        let target = tmp.path().join("target.txt");
        std::fs::write(&target, b"hi")?;

        let link = tmp.path().join("link.txt");
        unix_fs::symlink(&target, &link)?;

        assert!(std::path::Path::new(&link).exists());
        remove_existing_path(&link)?;
        assert!(!std::path::Path::new(&link).exists());
        assert!(std::path::Path::new(&target).exists());
        Ok(())
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
struct WebdavMetaEntry {
    path: String,
    kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    link_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    uid: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    gid: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mtime: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    xattrs: Option<std::collections::BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hardlink_group: Option<String>,
    action: String,
}

pub(super) struct WebdavSink {
    handle: Handle,
    client: WebdavClient,
    prefix_url: Url,
    conflict: ConflictPolicy,
    staging_dir: PathBuf,
    meta_rel_path: PathBuf,
    meta_entries_url: Url,
    ensured_collections: HashSet<String>,
}

impl WebdavSink {
    pub(super) fn new(
        handle: Handle,
        client: WebdavClient,
        prefix_url: Url,
        conflict: ConflictPolicy,
        op_id: String,
        staging_dir: PathBuf,
    ) -> Result<Self, anyhow::Error> {
        let meta_rel_path = PathBuf::from(".bastion-meta")
            .join("restore")
            .join(op_id.trim())
            .join("entries");

        let mut meta_entries_url = prefix_url.clone();
        {
            let mut segs = meta_entries_url
                .path_segments_mut()
                .map_err(|_| anyhow::anyhow!("webdav prefix_url cannot be a base"))?;
            segs.push(".bastion-meta");
            segs.push("restore");
            segs.push(op_id.trim());
            segs.push("entries");
        }
        if !meta_entries_url.path().ends_with('/') {
            meta_entries_url.set_path(&format!("{}/", meta_entries_url.path()));
        }

        Ok(Self {
            handle,
            client,
            prefix_url,
            conflict,
            staging_dir,
            meta_rel_path,
            meta_entries_url,
            ensured_collections: HashSet::new(),
        })
    }

    fn url_for_rel_path(&self, rel_path: &Path, is_dir: bool) -> Result<Url, anyhow::Error> {
        let mut url = self.prefix_url.clone();
        {
            let mut segs = url
                .path_segments_mut()
                .map_err(|_| anyhow::anyhow!("webdav prefix_url cannot be a base"))?;
            for c in rel_path.components() {
                match c {
                    std::path::Component::Normal(p) => {
                        segs.push(&p.to_string_lossy());
                    }
                    std::path::Component::CurDir => {}
                    _ => anyhow::bail!("invalid relative path: {}", rel_path.display()),
                }
            }
        }
        if is_dir && !url.path().ends_with('/') {
            url.set_path(&format!("{}/", url.path()));
        }
        Ok(url)
    }

    fn ensure_parent_collections(&mut self, rel_path: &Path) -> Result<(), anyhow::Error> {
        let mut dir_url = self.prefix_url.clone();
        let mut segs = dir_url
            .path_segments_mut()
            .map_err(|_| anyhow::anyhow!("webdav prefix_url cannot be a base"))?;

        let mut comps = rel_path.components().peekable();
        while let Some(c) = comps.next() {
            let is_last = comps.peek().is_none();
            match c {
                std::path::Component::Normal(p) => {
                    if is_last {
                        break;
                    }
                    segs.push(&p.to_string_lossy());
                    drop(segs);

                    if !dir_url.path().ends_with('/') {
                        dir_url.set_path(&format!("{}/", dir_url.path()));
                    }

                    let key = dir_url.path().to_string();
                    if self.ensured_collections.insert(key) {
                        let url = dir_url.clone();
                        let client = self.client.clone();
                        self.handle
                            .block_on(async move { client.ensure_collection(&url).await })
                            .map_err(|e| anyhow::anyhow!("{e:#}"))?;
                    }

                    segs = dir_url
                        .path_segments_mut()
                        .map_err(|_| anyhow::anyhow!("webdav prefix_url cannot be a base"))?;
                }
                std::path::Component::CurDir => {}
                _ => anyhow::bail!("invalid relative path: {}", rel_path.display()),
            }
        }

        Ok(())
    }

    fn write_meta_entry(
        &self,
        rel_path: &Path,
        entry: &tar::Header,
        action: &str,
    ) -> Result<(), anyhow::Error> {
        let path = super::path::archive_path_for_match(rel_path)
            .ok_or_else(|| anyhow::anyhow!("invalid relative path: {}", rel_path.display()))?;
        let kind = match entry.entry_type() {
            t if t.is_dir() => "dir",
            t if t.is_file() => "file",
            t if t.is_symlink() => "symlink",
            t if t.is_hard_link() => "hardlink",
            other => {
                // Fallback for unusual tar types.
                if other.is_fifo() {
                    "fifo"
                } else if other.is_block_special() {
                    "block_special"
                } else {
                    "other"
                }
            }
        }
        .to_string();

        let link_name = entry
            .link_name()
            .ok()
            .flatten()
            .map(|p| p.to_string_lossy().to_string())
            .filter(|v| !v.trim().is_empty());

        let meta = WebdavMetaEntry {
            path,
            kind,
            link_name,
            mode: entry.mode().ok(),
            uid: entry.uid().ok(),
            gid: entry.gid().ok(),
            mtime: entry.mtime().ok(),
            size: entry.size().ok(),
            xattrs: None,
            hardlink_group: None,
            action: action.to_string(),
        };

        let bytes = serde_json::to_vec(&meta)?;
        let digest = blake3::hash(bytes.as_slice()).to_hex().to_string();
        let url = self.meta_entries_url.join(&format!("{digest}.json"))?;
        let client = self.client.clone();
        self.handle
            .block_on(async move { client.put_bytes(&url, bytes, "application/json").await })
            .map_err(|e| anyhow::anyhow!("{e:#}"))?;
        Ok(())
    }

    fn write_meta_entry_from_record(
        &self,
        rel_path: &Path,
        record: &EntryRecord,
        action: &str,
    ) -> Result<(), anyhow::Error> {
        let path = super::path::archive_path_for_match(rel_path)
            .ok_or_else(|| anyhow::anyhow!("invalid relative path: {}", rel_path.display()))?;

        let link_name = record
            .symlink_target
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(|v| v.to_string());

        let meta = WebdavMetaEntry {
            path,
            kind: record.kind.clone(),
            link_name,
            mode: record.mode,
            uid: record.uid,
            gid: record.gid,
            mtime: record.mtime,
            size: Some(record.size),
            xattrs: record.xattrs.clone(),
            hardlink_group: record.hardlink_group.clone(),
            action: action.to_string(),
        };

        let bytes = serde_json::to_vec(&meta)?;
        let digest = blake3::hash(bytes.as_slice()).to_hex().to_string();
        let url = self.meta_entries_url.join(&format!("{digest}.json"))?;
        let client = self.client.clone();
        self.handle
            .block_on(async move { client.put_bytes(&url, bytes, "application/json").await })
            .map_err(|e| anyhow::anyhow!("{e:#}"))?;
        Ok(())
    }

    pub(super) fn apply_raw_tree_dir(
        &mut self,
        rel_path: &Path,
        record: &EntryRecord,
    ) -> Result<(), anyhow::Error> {
        self.ensure_parent_collections(rel_path)?;
        let url = self.url_for_rel_path(rel_path, true)?;
        let client = self.client.clone();
        self.handle
            .block_on(async move { client.ensure_collection(&url).await })
            .map_err(|e| anyhow::anyhow!("{e:#}"))?;
        self.write_meta_entry_from_record(rel_path, record, "written")?;
        Ok(())
    }

    pub(super) fn apply_raw_tree_file<R: Read>(
        &mut self,
        rel_path: &Path,
        record: &EntryRecord,
        mut reader: R,
    ) -> Result<(), anyhow::Error> {
        self.ensure_parent_collections(rel_path)?;

        let url = self.url_for_rel_path(rel_path, false)?;
        let exists = self
            .handle
            .block_on(self.client.head_size(&url))
            .map_err(|e| anyhow::anyhow!("{e:#}"))?
            .is_some();

        match self.conflict {
            ConflictPolicy::Overwrite => {
                if exists {
                    let client = self.client.clone();
                    let url = url.clone();
                    self.handle
                        .block_on(async move { client.delete(&url).await })
                        .map_err(|e| anyhow::anyhow!("{e:#}"))?;
                }
            }
            ConflictPolicy::Skip => {
                if exists {
                    self.write_meta_entry_from_record(rel_path, record, "skipped_existing")?;
                    return Ok(());
                }
            }
            ConflictPolicy::Fail => {
                if exists {
                    anyhow::bail!("restore conflict: {} exists", rel_path.display());
                }
            }
        }

        let size = record.size;
        let file_id = Uuid::new_v4().to_string();
        let tmp_path = self.staging_dir.join(format!("raw-tree-{file_id}.bin"));
        let mut tmp = std::fs::File::create(&tmp_path)?;
        let written = std::io::copy(&mut reader, &mut tmp)?;
        tmp.flush()?;
        if written != size {
            anyhow::bail!("restore entry size mismatch: expected {size}, got {written}");
        }

        let client = self.client.clone();
        let tmp_path_for_put = tmp_path.clone();
        self.handle
            .block_on(async move {
                client
                    .put_file_with_retries(&url, &tmp_path_for_put, size, 3)
                    .await
            })
            .map_err(|e| anyhow::anyhow!("{e:#}"))?;
        let _ = std::fs::remove_file(&tmp_path);

        self.write_meta_entry_from_record(rel_path, record, "written")?;
        Ok(())
    }

    pub(super) fn apply_raw_tree_symlink(
        &mut self,
        rel_path: &Path,
        record: &EntryRecord,
        _target: &str,
    ) -> Result<(), anyhow::Error> {
        self.write_meta_entry_from_record(rel_path, record, "skipped_unsupported")?;
        Ok(())
    }
}

impl RestoreSink for WebdavSink {
    fn prepare(&mut self) -> Result<(), anyhow::Error> {
        // Ensure the destination prefix exists.
        let prefix = self.prefix_url.clone();
        let client = self.client.clone();
        self.handle
            .block_on(async move { client.ensure_collection(&prefix).await })
            .map_err(|e| anyhow::anyhow!("{e:#}"))?;

        // Ensure `.bastion-meta/restore/<op_id>/entries/` exists.
        //
        // WebDAV `MKCOL` doesn't create intermediate collections; some servers return HTTP 409
        // Conflict if parent collections are missing. Ensure parents first to be robust.
        let meta_rel_path = self.meta_rel_path.clone();
        self.ensure_parent_collections(&meta_rel_path)?;
        let meta_dir = self.meta_entries_url.clone();
        let client = self.client.clone();
        self.handle
            .block_on(async move { client.ensure_collection(&meta_dir).await })
            .map_err(|e| anyhow::anyhow!("{e:#}"))?;

        std::fs::create_dir_all(&self.staging_dir)?;
        Ok(())
    }

    fn apply_entry<R: Read>(
        &mut self,
        entry: &mut tar::Entry<R>,
        rel_path: &Path,
    ) -> Result<(), anyhow::Error> {
        let header = entry.header().clone();
        let entry_type = header.entry_type();

        // Directories: best-effort creation; do not enforce conflict semantics (WebDAV lacks a
        // consistent "exists" primitive for collections across servers).
        if entry_type.is_dir() {
            self.ensure_parent_collections(rel_path)?;
            let url = self.url_for_rel_path(rel_path, true)?;
            let client = self.client.clone();
            self.handle
                .block_on(async move { client.ensure_collection(&url).await })
                .map_err(|e| anyhow::anyhow!("{e:#}"))?;
            self.write_meta_entry(rel_path, &header, "written")?;
            return Ok(());
        }

        // Unsupported types: record metadata and continue.
        if !entry_type.is_file() {
            self.write_meta_entry(rel_path, &header, "skipped_unsupported")?;
            return Ok(());
        }

        self.ensure_parent_collections(rel_path)?;

        let url = self.url_for_rel_path(rel_path, false)?;
        let exists = self
            .handle
            .block_on(self.client.head_size(&url))
            .map_err(|e| anyhow::anyhow!("{e:#}"))?
            .is_some();

        match self.conflict {
            ConflictPolicy::Overwrite => {
                if exists {
                    let client = self.client.clone();
                    let url = url.clone();
                    self.handle
                        .block_on(async move { client.delete(&url).await })
                        .map_err(|e| anyhow::anyhow!("{e:#}"))?;
                }
            }
            ConflictPolicy::Skip => {
                if exists {
                    self.write_meta_entry(rel_path, &header, "skipped_existing")?;
                    return Ok(());
                }
            }
            ConflictPolicy::Fail => {
                if exists {
                    anyhow::bail!("restore conflict: {} exists", rel_path.display());
                }
            }
        }

        // Materialize the tar entry to a temp file first; WebDAV PUT requires a Content-Length.
        let size = header.size().unwrap_or(0);
        let file_id = Uuid::new_v4().to_string();
        let tmp_path = self.staging_dir.join(format!("entry-{file_id}.bin"));
        let mut tmp = std::fs::File::create(&tmp_path)?;
        let written = std::io::copy(entry, &mut tmp)?;
        tmp.flush()?;
        if written != size {
            anyhow::bail!("restore entry size mismatch: expected {size}, got {written}");
        }

        let client = self.client.clone();
        let tmp_path_for_put = tmp_path.clone();
        self.handle
            .block_on(async move {
                client
                    .put_file_with_retries(&url, &tmp_path_for_put, size, 3)
                    .await
            })
            .map_err(|e| anyhow::anyhow!("{e:#}"))?;
        let _ = std::fs::remove_file(&tmp_path);

        self.write_meta_entry(rel_path, &header, "written")?;
        Ok(())
    }
}
