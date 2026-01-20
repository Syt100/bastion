use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HashAlgorithm {
    Blake3,
    Sha256,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactFormatV1 {
    #[default]
    ArchiveV1,
    RawTreeV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactPart {
    pub name: String,
    pub size: u64,
    pub hash_alg: HashAlgorithm,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EntryIndexRef {
    pub name: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PipelineSettings {
    #[serde(default)]
    pub format: ArtifactFormatV1,
    pub tar: String,
    pub compression: String,
    pub encryption: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encryption_key: Option<String>,
    pub split_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManifestV1 {
    pub format_version: u32,
    pub job_id: Uuid,
    pub run_id: Uuid,
    pub started_at: String,
    pub ended_at: String,
    pub pipeline: PipelineSettings,
    pub artifacts: Vec<ArtifactPart>,
    pub entry_index: EntryIndexRef,
}

impl ManifestV1 {
    pub const FORMAT_VERSION: u32 = 1;
}

#[cfg(test)]
mod tests {
    use assert_json_diff::assert_json_eq;
    use uuid::Uuid;

    use super::{
        ArtifactFormatV1, ArtifactPart, EntryIndexRef, HashAlgorithm, ManifestV1, PipelineSettings,
    };

    #[test]
    fn manifest_round_trip() {
        let manifest = ManifestV1 {
            format_version: ManifestV1::FORMAT_VERSION,
            job_id: Uuid::nil(),
            run_id: Uuid::nil(),
            started_at: "2025-12-30T12:00:00Z".to_string(),
            ended_at: "2025-12-30T12:00:01Z".to_string(),
            pipeline: PipelineSettings {
                format: ArtifactFormatV1::ArchiveV1,
                tar: "pax".to_string(),
                compression: "zstd".to_string(),
                encryption: "none".to_string(),
                encryption_key: None,
                split_bytes: 268_435_456,
            },
            artifacts: vec![ArtifactPart {
                name: "payload.part000001".to_string(),
                size: 123,
                hash_alg: HashAlgorithm::Blake3,
                hash: "b3:deadbeef".to_string(),
            }],
            entry_index: EntryIndexRef {
                name: "entries.jsonl.zst".to_string(),
                count: 42,
            },
        };

        let json = serde_json::to_value(&manifest).expect("serialize");
        let de: ManifestV1 = serde_json::from_value(json.clone()).expect("deserialize");
        assert_eq!(manifest, de);

        assert_json_eq!(
            json,
            serde_json::json!({
              "format_version": 1,
              "job_id": "00000000-0000-0000-0000-000000000000",
              "run_id": "00000000-0000-0000-0000-000000000000",
              "started_at": "2025-12-30T12:00:00Z",
              "ended_at": "2025-12-30T12:00:01Z",
              "pipeline": {
                "format": "archive_v1",
                "tar": "pax",
                "compression": "zstd",
                "encryption": "none",
                "split_bytes": 268435456
              },
              "artifacts": [
                {
                  "name": "payload.part000001",
                  "size": 123,
                  "hash_alg": "blake3",
                  "hash": "b3:deadbeef"
                }
              ],
              "entry_index": {
                "name": "entries.jsonl.zst",
                "count": 42
              }
            })
        );
    }

    #[test]
    fn manifest_defaults_format_to_archive_v1_when_missing() {
        let json = serde_json::json!({
          "format_version": 1,
          "job_id": "00000000-0000-0000-0000-000000000000",
          "run_id": "00000000-0000-0000-0000-000000000000",
          "started_at": "2025-12-30T12:00:00Z",
          "ended_at": "2025-12-30T12:00:01Z",
          "pipeline": {
            "tar": "pax",
            "compression": "zstd",
            "encryption": "none",
            "split_bytes": 268435456
          },
          "artifacts": [],
          "entry_index": { "name": "entries.jsonl.zst", "count": 0 }
        });

        let de: ManifestV1 = serde_json::from_value(json).expect("deserialize");
        assert_eq!(de.pipeline.format, ArtifactFormatV1::ArchiveV1);
    }
}
