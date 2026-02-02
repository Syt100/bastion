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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_round_trip_and_ignore_unknown_bits() {
        assert_eq!(
            ArtifactChunkFrameV1Flags::from_byte(0b0000_0000),
            ArtifactChunkFrameV1Flags { eof: false }
        );
        assert_eq!(
            ArtifactChunkFrameV1Flags::from_byte(0b0000_0001),
            ArtifactChunkFrameV1Flags { eof: true }
        );

        // Unknown bits are ignored.
        assert_eq!(
            ArtifactChunkFrameV1Flags::from_byte(0b1111_1111),
            ArtifactChunkFrameV1Flags { eof: true }
        );

        let flags = ArtifactChunkFrameV1Flags { eof: true };
        assert_eq!(ArtifactChunkFrameV1Flags::from_byte(flags.to_byte()), flags);
    }

    #[test]
    fn encode_decode_round_trip() -> Result<(), anyhow::Error> {
        let stream_id = Uuid::new_v4();
        let flags = ArtifactChunkFrameV1Flags { eof: true };
        let payload = b"hello";

        let frame = encode_artifact_chunk_frame_v1(&stream_id, flags, payload);
        assert_eq!(
            frame.len(),
            ARTIFACT_CHUNK_FRAME_V1_HEADER_LEN + payload.len()
        );

        let decoded = decode_artifact_chunk_frame_v1(&frame)?;
        assert_eq!(decoded.stream_id, stream_id);
        assert_eq!(decoded.flags, flags);
        assert_eq!(decoded.payload, payload);
        Ok(())
    }

    #[test]
    fn decode_rejects_too_short_frames() {
        let err = decode_artifact_chunk_frame_v1(&[0_u8; ARTIFACT_CHUNK_FRAME_V1_HEADER_LEN - 1])
            .err()
            .expect("expected error");
        assert!(err.to_string().contains("too short"));
    }
}
