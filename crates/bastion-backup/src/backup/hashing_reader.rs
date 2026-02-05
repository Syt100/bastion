use std::io::Read;

#[derive(Debug)]
pub(crate) struct HashingReader<R> {
    inner: R,
    hasher: blake3::Hasher,
    bytes_read: u64,
}

impl<R> HashingReader<R> {
    pub(crate) fn new(inner: R) -> Self {
        Self {
            inner,
            hasher: blake3::Hasher::new(),
            bytes_read: 0,
        }
    }

    pub(crate) fn bytes_read(&self) -> u64 {
        self.bytes_read
    }

    pub(crate) fn finalize_hex(&mut self) -> String {
        let hasher = std::mem::replace(&mut self.hasher, blake3::Hasher::new());
        hasher.finalize().to_hex().to_string()
    }
}

impl<R: Read> Read for HashingReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.inner.read(buf)?;
        if n == 0 {
            return Ok(0);
        }
        self.hasher.update(&buf[..n]);
        self.bytes_read = self.bytes_read.saturating_add(n as u64);
        Ok(n)
    }
}

