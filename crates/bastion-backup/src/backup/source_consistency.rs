use std::path::Path;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use bastion_core::job_spec::FsSymlinkPolicy;

const REPORT_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileIdV1 {
    pub dev: u64,
    pub ino: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileFingerprintV1 {
    pub size_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtime_unix_seconds: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<FileIdV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceConsistencySampleV1 {
    /// Archive path (not the OS path).
    pub path: String,
    /// Machine-readable reason (e.g. "mtime_changed", "size_changed", "file_id_changed",
    /// "stat_after_missing", "read_error").
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<FileFingerprintV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<FileFingerprintV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceConsistencyReportV1 {
    pub v: u32,
    pub changed_total: u64,
    pub replaced_total: u64,
    pub deleted_total: u64,
    pub read_error_total: u64,
    pub sample_truncated: bool,
    pub sample: Vec<SourceConsistencySampleV1>,
}

impl Default for SourceConsistencyReportV1 {
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

impl SourceConsistencyReportV1 {
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
    report: SourceConsistencyReportV1,
}

impl SourceConsistencyTracker {
    pub fn new(max_samples: usize) -> Self {
        Self {
            max_samples,
            report: SourceConsistencyReportV1::default(),
        }
    }

    pub fn finish(self) -> SourceConsistencyReportV1 {
        self.report
    }

    pub fn record_changed(
        &mut self,
        archive_path: &str,
        reason: &'static str,
        before: Option<FileFingerprintV1>,
        after: Option<FileFingerprintV1>,
    ) {
        self.report.changed_total = self.report.changed_total.saturating_add(1);
        self.push_sample(SourceConsistencySampleV1 {
            path: archive_path.to_string(),
            reason: reason.to_string(),
            before,
            after,
            error: None,
        });
    }

    pub fn record_replaced(
        &mut self,
        archive_path: &str,
        before: Option<FileFingerprintV1>,
        after: Option<FileFingerprintV1>,
    ) {
        self.report.replaced_total = self.report.replaced_total.saturating_add(1);
        self.push_sample(SourceConsistencySampleV1 {
            path: archive_path.to_string(),
            reason: "file_id_changed".to_string(),
            before,
            after,
            error: None,
        });
    }

    pub fn record_deleted(&mut self, archive_path: &str, before: Option<FileFingerprintV1>) {
        self.report.deleted_total = self.report.deleted_total.saturating_add(1);
        self.push_sample(SourceConsistencySampleV1 {
            path: archive_path.to_string(),
            reason: "stat_after_missing".to_string(),
            before,
            after: None,
            error: None,
        });
    }

    pub fn record_read_error(
        &mut self,
        archive_path: &str,
        error: impl Into<String>,
        before: Option<FileFingerprintV1>,
    ) {
        self.report.read_error_total = self.report.read_error_total.saturating_add(1);
        self.push_sample(SourceConsistencySampleV1 {
            path: archive_path.to_string(),
            reason: "read_error".to_string(),
            before,
            after: None,
            error: Some(error.into()),
        });
    }

    fn push_sample(&mut self, sample: SourceConsistencySampleV1) {
        if self.report.sample.len() < self.max_samples {
            self.report.sample.push(sample);
        } else {
            self.report.sample_truncated = true;
        }
    }
}

pub fn fingerprint_for_meta(meta: &std::fs::Metadata) -> FileFingerprintV1 {
    FileFingerprintV1 {
        size_bytes: meta.len(),
        mtime_unix_seconds: meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs()),
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
fn file_id_for_meta(meta: &std::fs::Metadata) -> Option<FileIdV1> {
    use std::os::unix::fs::MetadataExt as _;
    Some(FileIdV1 {
        dev: meta.dev(),
        ino: meta.ino(),
    })
}

#[cfg(not(unix))]
fn file_id_for_meta(_meta: &std::fs::Metadata) -> Option<FileIdV1> {
    None
}

pub fn detect_change_reason(
    before: &FileFingerprintV1,
    after: &FileFingerprintV1,
) -> Option<&'static str> {
    if before.file_id.is_some()
        && after.file_id.is_some()
        && before.file_id != after.file_id
    {
        return Some("file_id_changed");
    }

    if before.size_bytes != after.size_bytes {
        return Some("size_changed");
    }

    if before.mtime_unix_seconds.is_some()
        && after.mtime_unix_seconds.is_some()
        && before.mtime_unix_seconds != after.mtime_unix_seconds
    {
        return Some("mtime_changed");
    }

    None
}

