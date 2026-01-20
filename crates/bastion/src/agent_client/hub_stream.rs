use std::collections::HashMap;
use std::io::Read;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{Mutex, mpsc, oneshot};
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

use bastion_core::agent_protocol::{
    AgentToHubMessageV1, ArtifactStreamOpenResultV1, ArtifactStreamOpenV1, ArtifactStreamPullV1,
    PROTOCOL_VERSION,
};

#[derive(Debug, Clone)]
pub(crate) struct HubStreamChunk {
    pub(crate) eof: bool,
    pub(crate) bytes: Vec<u8>,
}

type StreamKey = Uuid;
type PendingOpen = HashMap<StreamKey, oneshot::Sender<ArtifactStreamOpenResultV1>>;
type PendingChunk = HashMap<StreamKey, oneshot::Sender<Result<HubStreamChunk, String>>>;

#[derive(Clone)]
pub(crate) struct HubStreamManager {
    outbox: mpsc::UnboundedSender<Message>,
    pending_open: Arc<Mutex<PendingOpen>>,
    pending_chunk: Arc<Mutex<PendingChunk>>,
}

impl HubStreamManager {
    pub(crate) fn new(outbox: mpsc::UnboundedSender<Message>) -> Self {
        Self {
            outbox,
            pending_open: Arc::new(Mutex::new(HashMap::new())),
            pending_chunk: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub(crate) async fn open(
        &self,
        req: ArtifactStreamOpenV1,
        timeout: Duration,
    ) -> Result<ArtifactStreamOpenResultV1, anyhow::Error> {
        let stream_id = Uuid::parse_str(req.stream_id.trim())?;
        let (tx, rx) = oneshot::channel::<ArtifactStreamOpenResultV1>();
        self.pending_open.lock().await.insert(stream_id, tx);

        let msg = AgentToHubMessageV1::ArtifactStreamOpen {
            v: PROTOCOL_VERSION,
            req,
        };
        let text = serde_json::to_string(&msg)?;
        if self.outbox.send(Message::Text(text.into())).is_err() {
            let _ = self.pending_open.lock().await.remove(&stream_id);
            anyhow::bail!("hub stream open send failed");
        }

        let res = tokio::time::timeout(timeout, rx)
            .await
            .map_err(|_| anyhow::anyhow!("hub stream open timeout"))?
            .map_err(|_| anyhow::anyhow!("hub stream open channel closed"))?;

        let _ = self.pending_open.lock().await.remove(&stream_id);
        Ok(res)
    }

    pub(crate) async fn complete_open(&self, res: ArtifactStreamOpenResultV1) {
        let Ok(stream_id) = Uuid::parse_str(res.stream_id.trim()) else {
            return;
        };
        if let Some(tx) = self.pending_open.lock().await.remove(&stream_id) {
            let _ = tx.send(res);
        }
    }

    pub(crate) async fn pull(
        &self,
        stream_id: Uuid,
        max_bytes: u32,
        timeout: Duration,
    ) -> Result<HubStreamChunk, anyhow::Error> {
        let (tx, rx) = oneshot::channel::<Result<HubStreamChunk, String>>();
        self.pending_chunk.lock().await.insert(stream_id, tx);

        let msg = AgentToHubMessageV1::ArtifactStreamPull {
            v: PROTOCOL_VERSION,
            req: ArtifactStreamPullV1 {
                stream_id: stream_id.to_string(),
                max_bytes,
            },
        };
        let text = serde_json::to_string(&msg)?;
        if self.outbox.send(Message::Text(text.into())).is_err() {
            let _ = self.pending_chunk.lock().await.remove(&stream_id);
            anyhow::bail!("hub stream pull send failed");
        }

        let res = tokio::time::timeout(timeout, rx)
            .await
            .map_err(|_| anyhow::anyhow!("hub stream pull timeout"))?
            .map_err(|_| anyhow::anyhow!("hub stream pull channel closed"))?;

        let _ = self.pending_chunk.lock().await.remove(&stream_id);
        res.map_err(anyhow::Error::msg)
    }

    pub(crate) async fn complete_chunk(&self, chunk: HubStreamChunk, stream_id: Uuid) {
        if let Some(tx) = self.pending_chunk.lock().await.remove(&stream_id) {
            let _ = tx.send(Ok(chunk));
        }
    }

    pub(crate) async fn read_bytes(
        &self,
        op_id: &str,
        run_id: &str,
        artifact: &str,
        open_timeout: Duration,
        pull_timeout: Duration,
        max_bytes: u32,
    ) -> Result<Vec<u8>, anyhow::Error> {
        let stream_id = Uuid::new_v4();
        let open = ArtifactStreamOpenV1 {
            stream_id: stream_id.to_string(),
            op_id: op_id.to_string(),
            run_id: run_id.to_string(),
            artifact: artifact.to_string(),
            path: None,
        };
        let res = self.open(open, open_timeout).await?;
        if let Some(error) = res.error.as_deref()
            && !error.trim().is_empty()
        {
            anyhow::bail!("hub stream open failed: {error}");
        }

        let mut out = Vec::new();
        loop {
            let chunk = self.pull(stream_id, max_bytes, pull_timeout).await?;
            out.extend_from_slice(&chunk.bytes);
            if chunk.eof {
                break;
            }
        }

        Ok(out)
    }
}

pub(crate) struct HubStreamReader {
    handle: tokio::runtime::Handle,
    streams: HubStreamManager,
    stream_id: Uuid,
    max_bytes: u32,
    pull_timeout: Duration,
    eof: bool,
    buf: Vec<u8>,
    pos: usize,
}

impl HubStreamReader {
    pub(crate) fn new(
        handle: tokio::runtime::Handle,
        streams: HubStreamManager,
        stream_id: Uuid,
        max_bytes: u32,
        pull_timeout: Duration,
    ) -> Self {
        Self {
            handle,
            streams,
            stream_id,
            max_bytes,
            pull_timeout,
            eof: false,
            buf: Vec::new(),
            pos: 0,
        }
    }
}

impl Read for HubStreamReader {
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

            let want = out.len().min(self.max_bytes as usize).max(1) as u32;
            let chunk = self
                .handle
                .block_on(self.streams.pull(self.stream_id, want, self.pull_timeout))
                .map_err(|e| std::io::Error::other(e.to_string()))?;

            self.eof = chunk.eof;
            self.buf = chunk.bytes;
            self.pos = 0;
            if self.eof && self.buf.is_empty() {
                return Ok(0);
            }
        }
    }
}
