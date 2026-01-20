use std::future::Future;
use std::io;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::pin::Pin;

use bastion_core::backup_format::{ENTRIES_INDEX_NAME, MANIFEST_NAME};
use bastion_core::manifest::{HashAlgorithm, ManifestV1};
use bastion_targets::WebdavClient;
use tokio::runtime::Handle;
use url::Url;

pub trait ArtifactSource: Send {
    fn read_manifest(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<ManifestV1, anyhow::Error>> + Send + '_>>;

    fn fetch_entries_index(
        &self,
        staging_dir: &Path,
    ) -> Pin<Box<dyn Future<Output = Result<PathBuf, anyhow::Error>> + Send + '_>>;

    fn open_payload_reader(
        &self,
        manifest: &ManifestV1,
        staging_dir: &Path,
    ) -> Result<Box<dyn Read + Send>, anyhow::Error>;
}

pub enum RunArtifactSource {
    Local(LocalDirSource),
    Webdav(Box<WebdavSource>),
}

impl ArtifactSource for RunArtifactSource {
    fn read_manifest(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<ManifestV1, anyhow::Error>> + Send + '_>> {
        match self {
            Self::Local(s) => s.read_manifest(),
            Self::Webdav(s) => s.read_manifest(),
        }
    }

    fn fetch_entries_index(
        &self,
        staging_dir: &Path,
    ) -> Pin<Box<dyn Future<Output = Result<PathBuf, anyhow::Error>> + Send + '_>> {
        match self {
            Self::Local(s) => s.fetch_entries_index(staging_dir),
            Self::Webdav(s) => s.fetch_entries_index(staging_dir),
        }
    }

    fn open_payload_reader(
        &self,
        manifest: &ManifestV1,
        staging_dir: &Path,
    ) -> Result<Box<dyn Read + Send>, anyhow::Error> {
        match self {
            Self::Local(s) => s.open_payload_reader(manifest, staging_dir),
            Self::Webdav(s) => s.open_payload_reader(manifest, staging_dir),
        }
    }
}

pub struct LocalDirSource {
    run_dir: PathBuf,
}

impl LocalDirSource {
    pub fn new(run_dir: PathBuf) -> Self {
        Self { run_dir }
    }
}

impl ArtifactSource for LocalDirSource {
    fn read_manifest(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<ManifestV1, anyhow::Error>> + Send + '_>> {
        Box::pin(async move {
            let bytes = tokio::fs::read(self.run_dir.join(MANIFEST_NAME)).await?;
            Ok(serde_json::from_slice::<ManifestV1>(&bytes)?)
        })
    }

    fn fetch_entries_index(
        &self,
        _staging_dir: &Path,
    ) -> Pin<Box<dyn Future<Output = Result<PathBuf, anyhow::Error>> + Send + '_>> {
        Box::pin(async move { Ok(self.run_dir.join(ENTRIES_INDEX_NAME)) })
    }

    fn open_payload_reader(
        &self,
        manifest: &ManifestV1,
        _staging_dir: &Path,
    ) -> Result<Box<dyn Read + Send>, anyhow::Error> {
        Ok(Box::new(VerifiedPartsReader::new_local(
            manifest
                .artifacts
                .iter()
                .map(|p| PartSpec {
                    name: p.name.clone(),
                    expected_size: p.size,
                    expected_hash_alg: p.hash_alg.clone(),
                    expected_hash: p.hash.clone(),
                    source: PartSource::Local {
                        path: self.run_dir.join(&p.name),
                    },
                })
                .collect(),
        )))
    }
}

pub struct WebdavSource {
    handle: Handle,
    client: WebdavClient,
    run_url: Url,
}

impl WebdavSource {
    pub fn new(handle: Handle, client: WebdavClient, run_url: Url) -> Self {
        Self {
            handle,
            client,
            run_url,
        }
    }
}

impl ArtifactSource for WebdavSource {
    fn read_manifest(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<ManifestV1, anyhow::Error>> + Send + '_>> {
        Box::pin(async move {
            let url = self.run_url.join(MANIFEST_NAME)?;
            let bytes = self.client.get_bytes(&url).await?;
            Ok(serde_json::from_slice::<ManifestV1>(&bytes)?)
        })
    }

    fn fetch_entries_index(
        &self,
        staging_dir: &Path,
    ) -> Pin<Box<dyn Future<Output = Result<PathBuf, anyhow::Error>> + Send + '_>> {
        let staging_dir = staging_dir.to_path_buf();
        Box::pin(async move {
            let dst = staging_dir.join(ENTRIES_INDEX_NAME);

            let url = self.run_url.join(ENTRIES_INDEX_NAME)?;
            let expected = self.client.head_size(&url).await?;

            if let Some(size) = expected
                && let Ok(meta) = tokio::fs::metadata(&dst).await
                && meta.len() == size
            {
                return Ok(dst);
            }

            self.client.get_to_file(&url, &dst, expected, 3).await?;
            Ok(dst)
        })
    }

    fn open_payload_reader(
        &self,
        manifest: &ManifestV1,
        staging_dir: &Path,
    ) -> Result<Box<dyn Read + Send>, anyhow::Error> {
        std::fs::create_dir_all(staging_dir)?;

        Ok(Box::new(VerifiedPartsReader::new_webdav(
            self.handle.clone(),
            self.client.clone(),
            manifest
                .artifacts
                .iter()
                .map(|p| {
                    let url = self.run_url.join(&p.name)?;
                    Ok::<_, anyhow::Error>(PartSpec {
                        name: p.name.clone(),
                        expected_size: p.size,
                        expected_hash_alg: p.hash_alg.clone(),
                        expected_hash: p.hash.clone(),
                        source: PartSource::Webdav {
                            url,
                            dest: staging_dir.join(&p.name),
                        },
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
        )))
    }
}

#[derive(Debug, Clone)]
enum PartSource {
    Local { path: PathBuf },
    Webdav { url: Url, dest: PathBuf },
}

#[derive(Debug, Clone)]
struct PartSpec {
    name: String,
    expected_size: u64,
    expected_hash_alg: HashAlgorithm,
    expected_hash: String,
    source: PartSource,
}

#[derive(Debug)]
struct ActivePart {
    index: usize,
    file: std::fs::File,
    hasher: blake3::Hasher,
    read_bytes: u64,
    expected_size: u64,
    expected_hash_alg: HashAlgorithm,
    expected_hash: String,
    cleanup_path: Option<PathBuf>,
}

struct VerifiedPartsReader {
    handle: Option<Handle>,
    client: Option<WebdavClient>,
    parts: Vec<PartSpec>,
    next_index: usize,
    current: Option<ActivePart>,
}

impl VerifiedPartsReader {
    fn new_local(parts: Vec<PartSpec>) -> Self {
        Self {
            handle: None,
            client: None,
            parts,
            next_index: 0,
            current: None,
        }
    }

    fn new_webdav(handle: Handle, client: WebdavClient, parts: Vec<PartSpec>) -> Self {
        Self {
            handle: Some(handle),
            client: Some(client),
            parts,
            next_index: 0,
            current: None,
        }
    }

    fn open_next(&mut self) -> io::Result<()> {
        let idx = self.next_index;
        let spec = self
            .parts
            .get(idx)
            .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "no more parts"))?
            .clone();

        let (file, cleanup_path) = match spec.source {
            PartSource::Local { path } => {
                let meta = std::fs::metadata(&path)?;
                if meta.len() != spec.expected_size {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "part size mismatch for {}: expected {}, got {}",
                            path.display(),
                            spec.expected_size,
                            meta.len()
                        ),
                    ));
                }
                (std::fs::File::open(&path)?, None)
            }
            PartSource::Webdav { url, dest } => {
                let expected_size = spec.expected_size;
                let handle = self
                    .handle
                    .as_ref()
                    .ok_or_else(|| io::Error::other("missing tokio handle"))?
                    .clone();
                let client = self
                    .client
                    .as_ref()
                    .ok_or_else(|| io::Error::other("missing webdav client"))?
                    .clone();

                if let Ok(meta) = std::fs::metadata(&dest)
                    && meta.len() == expected_size
                {
                    (std::fs::File::open(&dest)?, Some(dest))
                } else {
                    handle
                        .block_on(client.get_to_file(&url, &dest, Some(expected_size), 3))
                        .map_err(|e| io::Error::other(e.to_string()))?;
                    (std::fs::File::open(&dest)?, Some(dest))
                }
            }
        };

        self.current = Some(ActivePart {
            index: idx,
            file,
            hasher: blake3::Hasher::new(),
            read_bytes: 0,
            expected_size: spec.expected_size,
            expected_hash_alg: spec.expected_hash_alg,
            expected_hash: spec.expected_hash,
            cleanup_path,
        });
        self.next_index += 1;
        Ok(())
    }

    fn finish_current(&mut self) -> io::Result<()> {
        let Some(active) = self.current.take() else {
            return Ok(());
        };

        if active.read_bytes != active.expected_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "part read size mismatch for {}: expected {}, got {}",
                    self.parts
                        .get(active.index)
                        .map(|p| p.name.as_str())
                        .unwrap_or("<unknown>"),
                    active.expected_size,
                    active.read_bytes
                ),
            ));
        }

        match active.expected_hash_alg {
            HashAlgorithm::Blake3 => {
                let computed = active.hasher.finalize().to_hex().to_string();
                if computed != active.expected_hash {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "part hash mismatch for {}: expected {}, got {}",
                            self.parts
                                .get(active.index)
                                .map(|p| p.name.as_str())
                                .unwrap_or("<unknown>"),
                            active.expected_hash,
                            computed
                        ),
                    ));
                }
            }
            other => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("unsupported part hash algorithm: {other:?}"),
                ));
            }
        }

        if let Some(path) = active.cleanup_path {
            let _ = std::fs::remove_file(path);
        }

        Ok(())
    }
}

impl Read for VerifiedPartsReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            if self.current.is_none() {
                if self.next_index >= self.parts.len() {
                    return Ok(0);
                }
                self.open_next()?;
            }

            let active = self.current.as_mut().expect("current part exists");
            let n = active.file.read(buf)?;
            if n == 0 {
                self.finish_current()?;
                continue;
            }
            active.hasher.update(&buf[..n]);
            active.read_bytes = active.read_bytes.saturating_add(n as u64);
            if active.read_bytes > active.expected_size {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "part size overflow for {}: expected {}, got >{}",
                        self.parts
                            .get(active.index)
                            .map(|p| p.name.as_str())
                            .unwrap_or("<unknown>"),
                        active.expected_size,
                        active.expected_size
                    ),
                ));
            }
            return Ok(n);
        }
    }
}
