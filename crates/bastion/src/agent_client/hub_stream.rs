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

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::Duration;

    use bastion_core::agent_protocol::AgentToHubMessageV1;
    use bastion_core::agent_protocol::ArtifactStreamOpenV1;

    async fn recv_text_message(
        rx: &mut mpsc::UnboundedReceiver<Message>,
    ) -> Result<String, anyhow::Error> {
        let msg = tokio::time::timeout(Duration::from_secs(2), rx.recv())
            .await
            .map_err(|_| anyhow::anyhow!("timeout waiting for outbox message"))?
            .ok_or_else(|| anyhow::anyhow!("outbox closed"))?;
        let Message::Text(text) = msg else {
            anyhow::bail!("expected text message");
        };
        Ok(text.to_string())
    }

    #[tokio::test]
    async fn open_sends_message_and_returns_result() -> Result<(), anyhow::Error> {
        let (out_tx, mut out_rx) = mpsc::unbounded_channel::<Message>();
        let mgr = HubStreamManager::new(out_tx);

        let stream_id = Uuid::new_v4();
        let req = ArtifactStreamOpenV1 {
            stream_id: stream_id.to_string(),
            op_id: "op".to_string(),
            run_id: "run".to_string(),
            artifact: "entries".to_string(),
            path: None,
        };

        let mgr_for_open = mgr.clone();
        let open_task =
            tokio::spawn(async move { mgr_for_open.open(req, Duration::from_secs(2)).await });

        let text = recv_text_message(&mut out_rx).await?;
        let msg: AgentToHubMessageV1 = serde_json::from_str(&text)?;
        match msg {
            AgentToHubMessageV1::ArtifactStreamOpen { v, req } => {
                assert_eq!(v, PROTOCOL_VERSION);
                assert_eq!(req.stream_id, stream_id.to_string());
                assert_eq!(req.op_id, "op");
                assert_eq!(req.run_id, "run");
                assert_eq!(req.artifact, "entries");
                assert_eq!(req.path, None);
            }
            other => anyhow::bail!("unexpected message: {other:?}"),
        }

        mgr.complete_open(ArtifactStreamOpenResultV1 {
            stream_id: stream_id.to_string(),
            size: Some(123),
            error: None,
        })
        .await;

        let res = open_task.await??;
        assert_eq!(res.stream_id, stream_id.to_string());
        assert_eq!(res.size, Some(123));
        assert_eq!(res.error, None);
        Ok(())
    }

    #[tokio::test]
    async fn open_fails_fast_when_outbox_is_closed() {
        let (out_tx, out_rx) = mpsc::unbounded_channel::<Message>();
        drop(out_rx);
        let mgr = HubStreamManager::new(out_tx);

        let req = ArtifactStreamOpenV1 {
            stream_id: Uuid::new_v4().to_string(),
            op_id: "op".to_string(),
            run_id: "run".to_string(),
            artifact: "entries".to_string(),
            path: None,
        };

        let err = mgr
            .open(req, Duration::from_secs(1))
            .await
            .expect_err("expected error");
        assert!(err.to_string().contains("open send failed"));
    }

    #[tokio::test]
    async fn pull_sends_message_and_returns_chunk() -> Result<(), anyhow::Error> {
        let (out_tx, mut out_rx) = mpsc::unbounded_channel::<Message>();
        let mgr = HubStreamManager::new(out_tx);

        let stream_id = Uuid::new_v4();
        let mgr_for_pull = mgr.clone();
        let pull_task = tokio::spawn(async move {
            mgr_for_pull
                .pull(stream_id, 64, Duration::from_secs(2))
                .await
        });

        let text = recv_text_message(&mut out_rx).await?;
        let msg: AgentToHubMessageV1 = serde_json::from_str(&text)?;
        match msg {
            AgentToHubMessageV1::ArtifactStreamPull { v, req } => {
                assert_eq!(v, PROTOCOL_VERSION);
                assert_eq!(req.stream_id, stream_id.to_string());
                assert_eq!(req.max_bytes, 64);
            }
            other => anyhow::bail!("unexpected message: {other:?}"),
        }

        mgr.complete_chunk(
            HubStreamChunk {
                eof: true,
                bytes: vec![1, 2, 3],
            },
            stream_id,
        )
        .await;

        let chunk = pull_task.await??;
        assert!(chunk.eof);
        assert_eq!(chunk.bytes, vec![1, 2, 3]);
        Ok(())
    }

    #[tokio::test]
    async fn hub_stream_reader_reads_to_eof_across_chunks() -> Result<(), anyhow::Error> {
        let (out_tx, mut out_rx) = mpsc::unbounded_channel::<Message>();
        let mgr = HubStreamManager::new(out_tx);

        let stream_id = Uuid::new_v4();
        let handle = tokio::runtime::Handle::current();
        let mut reader =
            HubStreamReader::new(handle, mgr.clone(), stream_id, 3, Duration::from_secs(2));

        let read_task = tokio::task::spawn_blocking(move || {
            let mut out = Vec::new();
            reader.read_to_end(&mut out)?;
            Ok::<_, std::io::Error>(out)
        });

        // First pull -> "hel"
        let text = recv_text_message(&mut out_rx).await?;
        let msg: AgentToHubMessageV1 = serde_json::from_str(&text)?;
        match msg {
            AgentToHubMessageV1::ArtifactStreamPull { v, req } => {
                assert_eq!(v, PROTOCOL_VERSION);
                assert_eq!(req.stream_id, stream_id.to_string());
                assert_eq!(req.max_bytes, 3);
            }
            other => anyhow::bail!("unexpected message: {other:?}"),
        }
        mgr.complete_chunk(
            HubStreamChunk {
                eof: false,
                bytes: b"hel".to_vec(),
            },
            stream_id,
        )
        .await;

        // Second pull -> "lo" + eof.
        let text = recv_text_message(&mut out_rx).await?;
        let msg: AgentToHubMessageV1 = serde_json::from_str(&text)?;
        match msg {
            AgentToHubMessageV1::ArtifactStreamPull { v, req } => {
                assert_eq!(v, PROTOCOL_VERSION);
                assert_eq!(req.stream_id, stream_id.to_string());
                assert_eq!(req.max_bytes, 3);
            }
            other => anyhow::bail!("unexpected message: {other:?}"),
        }
        mgr.complete_chunk(
            HubStreamChunk {
                eof: true,
                bytes: b"lo".to_vec(),
            },
            stream_id,
        )
        .await;

        let bytes = tokio::time::timeout(Duration::from_secs(2), read_task)
            .await
            .map_err(|_| anyhow::anyhow!("timeout waiting for reader"))??;
        let bytes = bytes?;
        assert_eq!(bytes, b"hello".to_vec());
        Ok(())
    }

    #[tokio::test]
    async fn read_bytes_opens_and_pulls_until_eof() -> Result<(), anyhow::Error> {
        let (out_tx, mut out_rx) = mpsc::unbounded_channel::<Message>();
        let mgr = HubStreamManager::new(out_tx);

        let mgr_for_read = mgr.clone();
        let read_task = tokio::spawn(async move {
            mgr_for_read
                .read_bytes(
                    "op",
                    "run",
                    "entries",
                    Duration::from_secs(2),
                    Duration::from_secs(2),
                    64,
                )
                .await
        });

        // Open request
        let text = recv_text_message(&mut out_rx).await?;
        let msg: AgentToHubMessageV1 = serde_json::from_str(&text)?;
        let stream_id = match msg {
            AgentToHubMessageV1::ArtifactStreamOpen { v, req } => {
                assert_eq!(v, PROTOCOL_VERSION);
                assert_eq!(req.op_id, "op");
                assert_eq!(req.run_id, "run");
                assert_eq!(req.artifact, "entries");
                Uuid::parse_str(req.stream_id.as_str())?
            }
            other => anyhow::bail!("unexpected message: {other:?}"),
        };

        mgr.complete_open(ArtifactStreamOpenResultV1 {
            stream_id: stream_id.to_string(),
            size: None,
            error: None,
        })
        .await;

        // Pull 1
        let text = recv_text_message(&mut out_rx).await?;
        let msg: AgentToHubMessageV1 = serde_json::from_str(&text)?;
        match msg {
            AgentToHubMessageV1::ArtifactStreamPull { v, req } => {
                assert_eq!(v, PROTOCOL_VERSION);
                assert_eq!(req.stream_id, stream_id.to_string());
                assert_eq!(req.max_bytes, 64);
            }
            other => anyhow::bail!("unexpected message: {other:?}"),
        }
        mgr.complete_chunk(
            HubStreamChunk {
                eof: false,
                bytes: b"hi ".to_vec(),
            },
            stream_id,
        )
        .await;

        // Pull 2 -> eof.
        let text = recv_text_message(&mut out_rx).await?;
        let msg: AgentToHubMessageV1 = serde_json::from_str(&text)?;
        match msg {
            AgentToHubMessageV1::ArtifactStreamPull { v, req } => {
                assert_eq!(v, PROTOCOL_VERSION);
                assert_eq!(req.stream_id, stream_id.to_string());
                assert_eq!(req.max_bytes, 64);
            }
            other => anyhow::bail!("unexpected message: {other:?}"),
        }
        mgr.complete_chunk(
            HubStreamChunk {
                eof: true,
                bytes: b"there".to_vec(),
            },
            stream_id,
        )
        .await;

        let bytes = read_task.await??;
        assert_eq!(bytes, b"hi there".to_vec());
        Ok(())
    }
}
