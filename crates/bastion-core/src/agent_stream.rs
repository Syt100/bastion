use uuid::Uuid;

pub const ARTIFACT_CHUNK_FRAME_V1_HEADER_LEN: usize = 17; // 16 bytes UUID + 1 byte flags

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArtifactChunkFrameV1Flags {
    pub eof: bool,
}

impl ArtifactChunkFrameV1Flags {
    pub fn to_byte(self) -> u8 {
        if self.eof { 0b0000_0001 } else { 0 }
    }

    pub fn from_byte(b: u8) -> Self {
        Self {
            eof: (b & 0b0000_0001) != 0,
        }
    }
}

pub fn encode_artifact_chunk_frame_v1(
    stream_id: &Uuid,
    flags: ArtifactChunkFrameV1Flags,
    payload: &[u8],
) -> Vec<u8> {
    let mut out = Vec::with_capacity(ARTIFACT_CHUNK_FRAME_V1_HEADER_LEN + payload.len());
    out.extend_from_slice(stream_id.as_bytes());
    out.push(flags.to_byte());
    out.extend_from_slice(payload);
    out
}

pub struct DecodedArtifactChunkFrameV1<'a> {
    pub stream_id: Uuid,
    pub flags: ArtifactChunkFrameV1Flags,
    pub payload: &'a [u8],
}

pub fn decode_artifact_chunk_frame_v1(
    bytes: &[u8],
) -> Result<DecodedArtifactChunkFrameV1<'_>, anyhow::Error> {
    if bytes.len() < ARTIFACT_CHUNK_FRAME_V1_HEADER_LEN {
        anyhow::bail!("invalid artifact chunk frame: too short");
    }

    let id_bytes: [u8; 16] = bytes[0..16].try_into().expect("slice length verified");
    let stream_id = Uuid::from_bytes(id_bytes);
    let flags = ArtifactChunkFrameV1Flags::from_byte(bytes[16]);
    Ok(DecodedArtifactChunkFrameV1 {
        stream_id,
        flags,
        payload: &bytes[17..],
    })
}
