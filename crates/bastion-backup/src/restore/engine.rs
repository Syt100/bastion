use std::io::Read;
use std::path::Path;

use super::path;
use super::selection;
use super::sinks::RestoreSink;
use super::{PayloadDecryption, RestoreSelection};

pub(super) struct RestoreEngine<'a, S: RestoreSink> {
    sink: &'a mut S,
    decryption: PayloadDecryption,
    selection: Option<selection::NormalizedRestoreSelection>,
}

impl<'a, S: RestoreSink> RestoreEngine<'a, S> {
    pub(super) fn new(
        sink: &'a mut S,
        decryption: PayloadDecryption,
        selection: Option<&RestoreSelection>,
    ) -> Result<Self, anyhow::Error> {
        Ok(Self {
            sink,
            decryption,
            selection: selection
                .map(selection::normalize_restore_selection)
                .transpose()?,
        })
    }

    pub(super) fn restore(&mut self, payload: Box<dyn Read + Send>) -> Result<(), anyhow::Error> {
        self.sink.prepare()?;

        let payload: Box<dyn Read> = payload;
        let reader: Box<dyn Read> = match self.decryption.clone() {
            PayloadDecryption::None => payload,
            PayloadDecryption::AgeX25519 { identity } => {
                use std::str::FromStr as _;

                let identity = age::x25519::Identity::from_str(identity.trim())
                    .map_err(|e| anyhow::anyhow!(e))?;
                let decryptor = age::Decryptor::new(payload)?;
                let reader = decryptor.decrypt(std::iter::once(&identity as &dyn age::Identity))?;
                Box::new(reader)
            }
        };

        let decoder = zstd::Decoder::new(reader)?;
        let mut archive = tar::Archive::new(decoder);
        archive.set_unpack_xattrs(false);
        archive.set_preserve_mtime(true);

        for entry in archive.entries()? {
            let mut entry = entry?;
            let rel_raw = entry.path()?.to_path_buf();

            let rel_match = path::archive_path_for_match(&rel_raw)
                .ok_or_else(|| anyhow::anyhow!("invalid entry path: {}", rel_raw.display()))?;
            if let Some(selection) = self.selection.as_ref()
                && !selection.matches(&rel_match)
            {
                continue;
            }

            let rel = path::safe_join(Path::new(""), &rel_raw)
                .ok_or_else(|| anyhow::anyhow!("invalid entry path: {}", rel_raw.display()))?;

            self.sink.apply_entry(&mut entry, &rel)?;
        }

        Ok(())
    }
}
