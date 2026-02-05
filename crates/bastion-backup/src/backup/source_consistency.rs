use std::path::Path;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use bastion_core::job_spec::FsSymlinkPolicy;

const REPORT_VERSION: u32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "platform")]
pub enum FileIdV2 {
    Unix { dev: u64, ino: u64 },
    Windows { volume_serial: u32, file_index: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileFingerprintV2 {
    pub size_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtime_unix_nanos: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<FileIdV2>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceConsistencySampleV2 {
    /// Archive path (not the OS path).
    pub path: String,
    /// Machine-readable reason (e.g. "mtime_changed", "size_changed", "file_id_changed",
    /// "stat_after_missing", "read_error").
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<FileFingerprintV2>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_handle: Option<FileFingerprintV2>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_path: Option<FileFingerprintV2>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceConsistencyReportV2 {
    pub v: u32,
    pub changed_total: u64,
    pub replaced_total: u64,
    pub deleted_total: u64,
    pub read_error_total: u64,
    pub sample_truncated: bool,
    pub sample: Vec<SourceConsistencySampleV2>,
}

impl Default for SourceConsistencyReportV2 {
    fn default() -> Self {
        Self {
            v: REPORT_VERSION,
            changed_total: 0,
            replaced_total: 0,
            deleted_total: 0,
            read_error_total: 0,
            sample_truncated: false,
            sample: Vec::new(),
        }
    }
}

impl SourceConsistencyReportV2 {
    pub fn total(&self) -> u64 {
        self.changed_total
            .saturating_add(self.replaced_total)
            .saturating_add(self.deleted_total)
            .saturating_add(self.read_error_total)
    }

    pub fn is_empty(&self) -> bool {
        self.total() == 0
    }
}

#[derive(Debug)]
pub struct SourceConsistencyTracker {
    max_samples: usize,
    report: SourceConsistencyReportV2,
}

impl SourceConsistencyTracker {
    pub fn new(max_samples: usize) -> Self {
        Self {
            max_samples,
            report: SourceConsistencyReportV2::default(),
        }
    }

    pub fn finish(self) -> SourceConsistencyReportV2 {
        self.report
    }

    pub fn record_changed(
        &mut self,
        archive_path: &str,
        reason: &'static str,
        before: Option<FileFingerprintV2>,
        after_handle: Option<FileFingerprintV2>,
        after_path: Option<FileFingerprintV2>,
    ) {
        self.report.changed_total = self.report.changed_total.saturating_add(1);
        self.push_sample(SourceConsistencySampleV2 {
            path: archive_path.to_string(),
            reason: reason.to_string(),
            before,
            after_handle,
            after_path,
            error: None,
        });
    }

    pub fn record_replaced(
        &mut self,
        archive_path: &str,
        before: Option<FileFingerprintV2>,
        after_handle: Option<FileFingerprintV2>,
        after_path: Option<FileFingerprintV2>,
    ) {
        self.report.replaced_total = self.report.replaced_total.saturating_add(1);
        self.push_sample(SourceConsistencySampleV2 {
            path: archive_path.to_string(),
            reason: "file_id_changed".to_string(),
            before,
            after_handle,
            after_path,
            error: None,
        });
    }

    pub fn record_deleted(
        &mut self,
        archive_path: &str,
        before: Option<FileFingerprintV2>,
        after_handle: Option<FileFingerprintV2>,
    ) {
        self.report.deleted_total = self.report.deleted_total.saturating_add(1);
        self.push_sample(SourceConsistencySampleV2 {
            path: archive_path.to_string(),
            reason: "stat_after_missing".to_string(),
            before,
            after_handle,
            after_path: None,
            error: None,
        });
    }

    pub fn record_read_error(
        &mut self,
        archive_path: &str,
        error: impl Into<String>,
        before: Option<FileFingerprintV2>,
        after_handle: Option<FileFingerprintV2>,
        after_path: Option<FileFingerprintV2>,
    ) {
        self.report.read_error_total = self.report.read_error_total.saturating_add(1);
        self.push_sample(SourceConsistencySampleV2 {
            path: archive_path.to_string(),
            reason: "read_error".to_string(),
            before,
            after_handle,
            after_path,
            error: Some(error.into()),
        });
    }

    fn push_sample(&mut self, sample: SourceConsistencySampleV2) {
        if self.report.sample.len() < self.max_samples {
            self.report.sample.push(sample);
        } else {
            self.report.sample_truncated = true;
        }
    }
}

pub fn fingerprint_for_meta(meta: &std::fs::Metadata) -> FileFingerprintV2 {
    FileFingerprintV2 {
        size_bytes: meta.len(),
        mtime_unix_nanos: meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .and_then(|d| u64::try_from(d.as_nanos()).ok()),
        file_id: file_id_for_meta(meta),
    }
}

pub fn source_meta_for_policy(
    path: &Path,
    policy: FsSymlinkPolicy,
) -> Result<std::fs::Metadata, std::io::Error> {
    if policy == FsSymlinkPolicy::Follow {
        std::fs::metadata(path)
    } else {
        std::fs::symlink_metadata(path)
    }
}

#[cfg(unix)]
fn file_id_for_meta(meta: &std::fs::Metadata) -> Option<FileIdV2> {
    use std::os::unix::fs::MetadataExt as _;
    Some(FileIdV2::Unix {
        dev: meta.dev(),
        ino: meta.ino(),
    })
}

#[cfg(windows)]
fn file_id_for_meta(meta: &std::fs::Metadata) -> Option<FileIdV2> {
    use std::os::windows::fs::MetadataExt as _;
    Some(FileIdV2::Windows {
        volume_serial: meta.volume_serial_number(),
        file_index: meta.file_index(),
    })
}

#[cfg(not(any(unix, windows)))]
fn file_id_for_meta(_meta: &std::fs::Metadata) -> Option<FileIdV2> {
    None
}

pub fn detect_change_reason(
    before: &FileFingerprintV2,
    after: &FileFingerprintV2,
) -> Option<&'static str> {
    if before.file_id.is_some() && after.file_id.is_some() && before.file_id != after.file_id {
        return Some("file_id_changed");
    }

    if before.size_bytes != after.size_bytes {
        return Some("size_changed");
    }

    if before.mtime_unix_nanos.is_some()
        && after.mtime_unix_nanos.is_some()
        && before.mtime_unix_nanos != after.mtime_unix_nanos
    {
        return Some("mtime_changed");
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_change_reason_detects_mtime_nanos() {
        let before = FileFingerprintV2 {
            size_bytes: 1,
            mtime_unix_nanos: Some(1000),
            file_id: None,
        };
        let after = FileFingerprintV2 {
            size_bytes: 1,
            mtime_unix_nanos: Some(1001),
            file_id: None,
        };
        assert_eq!(detect_change_reason(&before, &after), Some("mtime_changed"));
    }

    #[test]
    fn detect_change_reason_detects_size_change() {
        let before = FileFingerprintV2 {
            size_bytes: 1,
            mtime_unix_nanos: None,
            file_id: None,
        };
        let after = FileFingerprintV2 {
            size_bytes: 2,
            mtime_unix_nanos: None,
            file_id: None,
        };
        assert_eq!(detect_change_reason(&before, &after), Some("size_changed"));
    }

    #[cfg(unix)]
    #[test]
    fn fingerprint_for_meta_includes_unix_file_id() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("a.txt");
        std::fs::write(&path, b"hi").expect("write file");
        let meta = std::fs::metadata(&path).expect("metadata");

        let fp = fingerprint_for_meta(&meta);
        assert_eq!(fp.size_bytes, 2);
        assert!(fp.mtime_unix_nanos.is_some());
        assert!(matches!(fp.file_id, Some(FileIdV2::Unix { .. })));
    }

    #[cfg(windows)]
    #[test]
    fn fingerprint_for_meta_includes_windows_file_id() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("a.txt");
        std::fs::write(&path, b"hi").expect("write file");
        let meta = std::fs::metadata(&path).expect("metadata");

        let fp = fingerprint_for_meta(&meta);
        assert_eq!(fp.size_bytes, 2);
        assert!(fp.mtime_unix_nanos.is_some());
        assert!(matches!(fp.file_id, Some(FileIdV2::Windows { .. })));
    }
}
