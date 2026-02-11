use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::Context;
use sqlx::SqlitePool;
use uuid::Uuid;

use bastion_backup::restore::sources::{ArtifactSource, DriverSource, RunArtifactSource};
use bastion_core::HUB_NODE_ID;
use bastion_core::backup_format::{COMPLETE_NAME, ENTRIES_INDEX_NAME, MANIFEST_NAME};
use bastion_core::job_spec;
use bastion_core::manifest::{HashAlgorithm, ManifestV1};
use bastion_driver_api::{OpenReaderRequest, TargetRunReader};
use bastion_driver_registry::builtins;
use bastion_driver_registry::target_runtime::{self, WebdavRuntimeAuth};
use bastion_engine::agent_manager::AgentManager;
use bastion_storage::jobs_repo;
use bastion_storage::runs_repo;
use bastion_storage::secrets::SecretsCrypto;
use bastion_storage::secrets_repo;

use super::{
    ARTIFACT_STREAM_MAX_BYTES, ARTIFACT_STREAM_OPEN_TIMEOUT, ARTIFACT_STREAM_PULL_TIMEOUT,
};

pub(super) struct HubArtifactStream {
    pub(super) reader: Arc<Mutex<Box<dyn Read + Send>>>,
    pub(super) cleanup_dir: Option<PathBuf>,
}

#[derive(Clone)]
struct RunArtifactsLocation {
    node_id: String,
    reader: Arc<dyn TargetRunReader>,
}

fn target_ref(spec: &job_spec::JobSpecV1) -> &job_spec::TargetV1 {
    match spec {
        job_spec::JobSpecV1::Filesystem { target, .. } => target,
        job_spec::JobSpecV1::Sqlite { target, .. } => target,
        job_spec::JobSpecV1::Vaultwarden { target, .. } => target,
    }
}

async fn resolve_target_config_for_reader(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    node_id: &str,
    target: &job_spec::TargetV1,
) -> Result<(bastion_driver_api::DriverId, serde_json::Value), anyhow::Error> {
    let webdav_auth = match target {
        job_spec::TargetV1::Webdav { secret_name, .. } => {
            let secret_name = secret_name.trim();
            if secret_name.is_empty() {
                anyhow::bail!("webdav.secret_name is required");
            }

            let cred_bytes = secrets_repo::get_secret(db, secrets, node_id, "webdav", secret_name)
                .await?
                .ok_or_else(|| anyhow::anyhow!("missing webdav secret: {secret_name}"))?;
            let credentials = bastion_targets::WebdavCredentials::from_json(&cred_bytes)?;
            Some(WebdavRuntimeAuth {
                username: credentials.username,
                password: credentials.password,
                secret_name: Some(secret_name.to_string()),
            })
        }
        job_spec::TargetV1::LocalDir { .. } => None,
    };

    target_runtime::runtime_input_for_job_target(target, webdav_auth.as_ref())
        .map_err(anyhow::Error::new)
}

async fn resolve_run_artifacts_location(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    run_id: &str,
) -> Result<RunArtifactsLocation, anyhow::Error> {
    let run = runs_repo::get_run(db, run_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("run not found"))?;
    if run.status != runs_repo::RunStatus::Success {
        anyhow::bail!("run is not successful");
    }

    let job = jobs_repo::get_job(db, &run.job_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("job not found"))?;
    let node_id = job
        .agent_id
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .unwrap_or(HUB_NODE_ID)
        .to_string();

    let spec = job_spec::parse_value(&job.spec)?;
    job_spec::validate(&spec)?;

    let (driver_id, target_config) =
        resolve_target_config_for_reader(db, secrets, &node_id, target_ref(&spec)).await?;
    let reader = builtins::target_registry().open_reader(
        &driver_id,
        OpenReaderRequest {
            job_id: run.job_id.clone(),
            run_id: run_id.to_string(),
            target_config,
        },
    )?;

    Ok(RunArtifactsLocation { node_id, reader })
}

fn artifact_stream_staging_dir(data_dir: &Path, op_id: &str, stream_id: Uuid) -> PathBuf {
    data_dir
        .join("hub")
        .join("artifact_streams")
        .join(op_id)
        .join(stream_id.to_string())
}

async fn open_local_file_reader(
    path: PathBuf,
) -> Result<(Box<dyn Read + Send>, u64), anyhow::Error> {
    tokio::task::spawn_blocking(
        move || -> Result<(Box<dyn Read + Send>, u64), anyhow::Error> {
            let size = std::fs::metadata(&path)?.len();
            let file = std::fs::File::open(&path)?;
            Ok((Box::new(file) as Box<dyn Read + Send>, size))
        },
    )
    .await
    .context("join error while opening local file")?
}

pub(super) async fn open_hub_artifact_stream(
    data_dir: &Path,
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    agent_manager: &AgentManager,
    req: &bastion_core::agent_protocol::ArtifactStreamOpenV1,
    stream_id: Uuid,
) -> Result<(HubArtifactStream, Option<u64>), anyhow::Error> {
    let op_id = req.op_id.trim();
    if op_id.is_empty() {
        anyhow::bail!("op_id is required");
    }
    let run_id = req.run_id.trim();
    if run_id.is_empty() {
        anyhow::bail!("run_id is required");
    }
    let artifact = req.artifact.trim();
    if artifact.is_empty() {
        anyhow::bail!("artifact is required");
    }

    let location = resolve_run_artifacts_location(db, secrets, run_id).await?;

    match artifact {
        "payload" => {
            if let Some(run_dir) = location.reader.local_run_dir()
                && location.node_id != HUB_NODE_ID
            {
                let manifest =
                    read_agent_manifest(agent_manager, &location.node_id, op_id, run_id, &run_dir)
                        .await?;
                let size = Some(manifest.artifacts.iter().map(|p| p.size).sum::<u64>());

                let reader = RemoteAgentPartsReader::new(
                    tokio::runtime::Handle::current(),
                    agent_manager.clone(),
                    location.node_id,
                    op_id.to_string(),
                    run_id.to_string(),
                    run_dir,
                    manifest,
                );

                return Ok((
                    HubArtifactStream {
                        reader: Arc::new(Mutex::new(Box::new(reader))),
                        cleanup_dir: None,
                    },
                    size,
                ));
            }

            let has_local_dir = location.reader.local_run_dir().is_some();
            let handle = tokio::runtime::Handle::current();
            let source = RunArtifactSource::Driver(DriverSource::new(handle, location.reader));
            let manifest = source.read_manifest().await?;
            let size = Some(manifest.artifacts.iter().map(|p| p.size).sum::<u64>());

            let (reader, cleanup_dir) = if has_local_dir {
                (source.open_payload_reader(&manifest, data_dir)?, None)
            } else {
                let staging_dir = artifact_stream_staging_dir(data_dir, op_id, stream_id);
                tokio::fs::create_dir_all(&staging_dir).await?;
                (
                    source.open_payload_reader(&manifest, &staging_dir)?,
                    Some(staging_dir),
                )
            };

            Ok((
                HubArtifactStream {
                    reader: Arc::new(Mutex::new(reader)),
                    cleanup_dir,
                },
                size,
            ))
        }
        MANIFEST_NAME | COMPLETE_NAME => {
            if let Some(run_dir) = location.reader.local_run_dir() {
                let path = run_dir.join(artifact);
                if location.node_id == HUB_NODE_ID {
                    let (reader, size) = open_local_file_reader(path).await?;
                    return Ok((
                        HubArtifactStream {
                            reader: Arc::new(Mutex::new(reader)),
                            cleanup_dir: None,
                        },
                        Some(size),
                    ));
                }

                let (reader, size) = open_agent_file_reader(
                    agent_manager,
                    &location.node_id,
                    op_id,
                    run_id,
                    artifact,
                    &path,
                )
                .await?;
                return Ok((
                    HubArtifactStream {
                        reader: Arc::new(Mutex::new(reader)),
                        cleanup_dir: None,
                    },
                    size,
                ));
            }

            let bytes = location.reader.read_bytes(artifact.to_string()).await?;
            let size = Some(bytes.len() as u64);
            let reader = std::io::Cursor::new(bytes);
            Ok((
                HubArtifactStream {
                    reader: Arc::new(Mutex::new(Box::new(reader))),
                    cleanup_dir: None,
                },
                size,
            ))
        }
        ENTRIES_INDEX_NAME => {
            if let Some(run_dir) = location.reader.local_run_dir() {
                let path = run_dir.join(ENTRIES_INDEX_NAME);
                if location.node_id == HUB_NODE_ID {
                    let (reader, size) = open_local_file_reader(path).await?;
                    return Ok((
                        HubArtifactStream {
                            reader: Arc::new(Mutex::new(reader)),
                            cleanup_dir: None,
                        },
                        Some(size),
                    ));
                }

                let (reader, size) = open_agent_file_reader(
                    agent_manager,
                    &location.node_id,
                    op_id,
                    run_id,
                    ENTRIES_INDEX_NAME,
                    &path,
                )
                .await?;
                return Ok((
                    HubArtifactStream {
                        reader: Arc::new(Mutex::new(reader)),
                        cleanup_dir: None,
                    },
                    size,
                ));
            }

            let staging_dir = artifact_stream_staging_dir(data_dir, op_id, stream_id);
            tokio::fs::create_dir_all(&staging_dir).await?;
            let dest = staging_dir.join(ENTRIES_INDEX_NAME);
            let expected = location
                .reader
                .head_size(ENTRIES_INDEX_NAME.to_string())
                .await?;
            let size = if let Some(expected_size) = expected {
                if let Ok(meta) = tokio::fs::metadata(&dest).await {
                    if meta.len() == expected_size {
                        expected_size
                    } else {
                        location
                            .reader
                            .get_to_file(
                                ENTRIES_INDEX_NAME.to_string(),
                                dest.clone(),
                                Some(expected_size),
                                3,
                            )
                            .await?
                    }
                } else {
                    location
                        .reader
                        .get_to_file(
                            ENTRIES_INDEX_NAME.to_string(),
                            dest.clone(),
                            Some(expected_size),
                            3,
                        )
                        .await?
                }
            } else {
                location
                    .reader
                    .get_to_file(ENTRIES_INDEX_NAME.to_string(), dest.clone(), None, 3)
                    .await?
            };
            let (reader, _) = open_local_file_reader(dest).await?;
            Ok((
                HubArtifactStream {
                    reader: Arc::new(Mutex::new(reader)),
                    cleanup_dir: Some(staging_dir),
                },
                Some(size),
            ))
        }
        other => anyhow::bail!("unsupported artifact: {}", other),
    }
}

async fn read_agent_manifest(
    agent_manager: &AgentManager,
    agent_id: &str,
    op_id: &str,
    run_id: &str,
    run_dir: &Path,
) -> Result<ManifestV1, anyhow::Error> {
    let path = run_dir.join(MANIFEST_NAME);
    let stream_id = Uuid::new_v4();
    let open = bastion_core::agent_protocol::ArtifactStreamOpenV1 {
        stream_id: stream_id.to_string(),
        op_id: op_id.to_string(),
        run_id: run_id.to_string(),
        artifact: MANIFEST_NAME.to_string(),
        path: Some(path.to_string_lossy().to_string()),
    };

    let res = agent_manager
        .artifact_stream_open(agent_id, open, ARTIFACT_STREAM_OPEN_TIMEOUT)
        .await?;
    if let Some(error) = res.error.as_deref()
        && !error.trim().is_empty()
    {
        anyhow::bail!("agent open manifest failed: {error}");
    }

    let mut bytes = Vec::new();
    loop {
        let chunk = agent_manager
            .artifact_stream_pull(
                agent_id,
                bastion_core::agent_protocol::ArtifactStreamPullV1 {
                    stream_id: stream_id.to_string(),
                    max_bytes: ARTIFACT_STREAM_MAX_BYTES as u32,
                },
                ARTIFACT_STREAM_PULL_TIMEOUT,
            )
            .await?;
        bytes.extend_from_slice(&chunk.bytes);
        if chunk.eof {
            break;
        }
    }

    let manifest = serde_json::from_slice::<ManifestV1>(&bytes)?;
    Ok(manifest)
}

async fn open_agent_file_reader(
    agent_manager: &AgentManager,
    agent_id: &str,
    op_id: &str,
    run_id: &str,
    artifact: &str,
    path: &Path,
) -> Result<(Box<dyn Read + Send>, Option<u64>), anyhow::Error> {
    let stream_id = Uuid::new_v4();
    let open = bastion_core::agent_protocol::ArtifactStreamOpenV1 {
        stream_id: stream_id.to_string(),
        op_id: op_id.to_string(),
        run_id: run_id.to_string(),
        artifact: artifact.to_string(),
        path: Some(path.to_string_lossy().to_string()),
    };

    let res = agent_manager
        .artifact_stream_open(agent_id, open, ARTIFACT_STREAM_OPEN_TIMEOUT)
        .await?;
    if let Some(error) = res.error.as_deref()
        && !error.trim().is_empty()
    {
        anyhow::bail!("agent open {artifact} failed: {error}");
    }

    let reader = RemoteAgentFileReader {
        handle: tokio::runtime::Handle::current(),
        agent_manager: agent_manager.clone(),
        agent_id: agent_id.to_string(),
        stream_id,
        eof: false,
        buf: Vec::new(),
        pos: 0,
    };
    Ok((Box::new(reader), res.size))
}

struct RemoteAgentFileReader {
    handle: tokio::runtime::Handle,
    agent_manager: AgentManager,
    agent_id: String,
    stream_id: Uuid,
    eof: bool,
    buf: Vec<u8>,
    pos: usize,
}

impl Read for RemoteAgentFileReader {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        if out.is_empty() {
            return Ok(0);
        }

        loop {
            if self.pos < self.buf.len() {
                let n = std::cmp::min(out.len(), self.buf.len() - self.pos);
                out[..n].copy_from_slice(&self.buf[self.pos..self.pos + n]);
                self.pos += n;
                if self.pos >= self.buf.len() {
                    self.buf.clear();
                    self.pos = 0;
                }
                return Ok(n);
            }

            if self.eof {
                return Ok(0);
            }

            let want = out.len().clamp(1, ARTIFACT_STREAM_MAX_BYTES);
            let chunk = self
                .handle
                .block_on(self.agent_manager.artifact_stream_pull(
                    &self.agent_id,
                    bastion_core::agent_protocol::ArtifactStreamPullV1 {
                        stream_id: self.stream_id.to_string(),
                        max_bytes: want as u32,
                    },
                    ARTIFACT_STREAM_PULL_TIMEOUT,
                ))
                .map_err(|e| std::io::Error::other(e.to_string()))?;

            self.eof = chunk.eof;
            self.buf = chunk.bytes;
            self.pos = 0;
        }
    }
}

struct RemoteAgentPartsReader {
    handle: tokio::runtime::Handle,
    agent_manager: AgentManager,
    agent_id: String,
    op_id: String,
    run_id: String,
    run_dir: PathBuf,
    parts: Vec<bastion_core::manifest::ArtifactPart>,
    next_index: usize,
    current: Option<RemoteActivePart>,
}

struct RemoteActivePart {
    name: String,
    reader: RemoteAgentFileReader,
    hasher: blake3::Hasher,
    read_bytes: u64,
    expected_size: u64,
    expected_hash_alg: HashAlgorithm,
    expected_hash: String,
}

impl RemoteAgentPartsReader {
    fn new(
        handle: tokio::runtime::Handle,
        agent_manager: AgentManager,
        agent_id: String,
        op_id: String,
        run_id: String,
        run_dir: PathBuf,
        manifest: ManifestV1,
    ) -> Self {
        Self {
            handle,
            agent_manager,
            agent_id,
            op_id,
            run_id,
            run_dir,
            parts: manifest.artifacts,
            next_index: 0,
            current: None,
        }
    }

    fn open_next(&mut self) -> std::io::Result<()> {
        let idx = self.next_index;
        let spec = self
            .parts
            .get(idx)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "no more parts"))?
            .clone();

        let path = self.run_dir.join(&spec.name);
        let stream_id = Uuid::new_v4();
        let open = bastion_core::agent_protocol::ArtifactStreamOpenV1 {
            stream_id: stream_id.to_string(),
            op_id: self.op_id.clone(),
            run_id: self.run_id.clone(),
            artifact: spec.name.clone(),
            path: Some(path.to_string_lossy().to_string()),
        };

        let res = self
            .handle
            .block_on(self.agent_manager.artifact_stream_open(
                &self.agent_id,
                open,
                ARTIFACT_STREAM_OPEN_TIMEOUT,
            ))
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        if let Some(error) = res.error.as_deref()
            && !error.trim().is_empty()
        {
            return Err(std::io::Error::other(format!(
                "agent open part {} failed: {error}",
                spec.name
            )));
        }
        if let Some(size) = res.size
            && size != spec.size
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "part size mismatch for {}: expected {}, got {}",
                    spec.name, spec.size, size
                ),
            ));
        }

        self.current = Some(RemoteActivePart {
            name: spec.name.clone(),
            reader: RemoteAgentFileReader {
                handle: self.handle.clone(),
                agent_manager: self.agent_manager.clone(),
                agent_id: self.agent_id.clone(),
                stream_id,
                eof: false,
                buf: Vec::new(),
                pos: 0,
            },
            hasher: blake3::Hasher::new(),
            read_bytes: 0,
            expected_size: spec.size,
            expected_hash_alg: spec.hash_alg,
            expected_hash: spec.hash,
        });
        self.next_index += 1;
        Ok(())
    }

    fn finish_current(&mut self) -> std::io::Result<()> {
        let Some(active) = self.current.take() else {
            return Ok(());
        };

        if active.read_bytes != active.expected_size {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "part read size mismatch for {}: expected {}, got {}",
                    active.name, active.expected_size, active.read_bytes
                ),
            ));
        }

        match active.expected_hash_alg {
            HashAlgorithm::Blake3 => {
                let computed = active.hasher.finalize().to_hex().to_string();
                if computed != active.expected_hash {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                            "part hash mismatch for {}: expected {}, got {}",
                            active.name, active.expected_hash, computed
                        ),
                    ));
                }
            }
            other => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("unsupported part hash algorithm: {other:?}"),
                ));
            }
        }

        Ok(())
    }
}

impl Read for RemoteAgentPartsReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        loop {
            if self.current.is_none() {
                if self.next_index >= self.parts.len() {
                    return Ok(0);
                }
                self.open_next()?;
            }

            let active = self.current.as_mut().expect("current part exists");
            let n = active.reader.read(buf)?;
            if n == 0 {
                self.finish_current()?;
                continue;
            }

            active.hasher.update(&buf[..n]);
            active.read_bytes = active.read_bytes.saturating_add(n as u64);
            if active.read_bytes > active.expected_size {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "part size overflow for {}: expected {}, got >{}",
                        active.name, active.expected_size, active.expected_size
                    ),
                ));
            }
            return Ok(n);
        }
    }
}
