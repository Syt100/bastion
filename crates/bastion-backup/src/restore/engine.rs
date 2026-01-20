use std::io::Read;
use std::path::Path;
use std::time::{Duration, Instant};

use bastion_core::progress::ProgressUnitsV1;

use super::path;
use super::selection;
use super::sinks::RestoreSink;
use super::{PayloadDecryption, RestoreSelection};

const RESTORE_PROGRESS_MIN_INTERVAL: Duration = Duration::from_secs(1);

struct RestoreProgressCtx<'a> {
    done: ProgressUnitsV1,
    last_emit: Instant,
    on_progress: &'a dyn Fn(ProgressUnitsV1),
}

impl<'a> RestoreProgressCtx<'a> {
    fn new(on_progress: &'a dyn Fn(ProgressUnitsV1)) -> Self {
        Self {
            done: ProgressUnitsV1::default(),
            last_emit: Instant::now(),
            on_progress,
        }
    }

    fn maybe_emit(&mut self, force: bool) {
        if !force && self.last_emit.elapsed() < RESTORE_PROGRESS_MIN_INTERVAL {
            return;
        }
        self.last_emit = Instant::now();
        (self.on_progress)(self.done);
    }

    fn record(&mut self, is_dir: bool, size: u64) {
        if is_dir {
            self.done.dirs = self.done.dirs.saturating_add(1);
            self.maybe_emit(false);
            return;
        }
        self.done.files = self.done.files.saturating_add(1);
        self.done.bytes = self.done.bytes.saturating_add(size);
        self.maybe_emit(false);
    }
}

pub(super) struct RestoreEngine<'a, S: RestoreSink> {
    sink: &'a mut S,
    decryption: PayloadDecryption,
    selection: Option<selection::NormalizedRestoreSelection>,
    progress: Option<RestoreProgressCtx<'a>>,
}

impl<'a, S: RestoreSink> RestoreEngine<'a, S> {
    pub(super) fn new(
        sink: &'a mut S,
        decryption: PayloadDecryption,
        selection: Option<&RestoreSelection>,
        on_progress: Option<&'a dyn Fn(ProgressUnitsV1)>,
    ) -> Result<Self, anyhow::Error> {
        Ok(Self {
            sink,
            decryption,
            selection: selection
                .map(selection::normalize_restore_selection)
                .transpose()?,
            progress: on_progress.map(RestoreProgressCtx::new),
        })
    }

    pub(super) fn restore(&mut self, payload: Box<dyn Read + Send>) -> Result<(), anyhow::Error> {
        self.sink.prepare()?;
        if let Some(ctx) = self.progress.as_mut() {
            ctx.maybe_emit(true);
        }

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

            if let Some(ctx) = self.progress.as_mut() {
                let ty = entry.header().entry_type();
                let is_dir = matches!(ty, tar::EntryType::Directory);
                let size = entry.header().size().unwrap_or(0);
                ctx.record(is_dir, size);
            }
        }

        if let Some(ctx) = self.progress.as_mut() {
            ctx.maybe_emit(true);
        }
        Ok(())
    }
}
