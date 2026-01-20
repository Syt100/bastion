pub(super) use super::engine::PayloadDecryption;
pub(super) use super::path::safe_join;

#[cfg(test)]
use std::fs::File;
#[cfg(test)]
use std::io::Read;

#[cfg(test)]
pub(super) fn restore_from_parts(
    part_paths: &[std::path::PathBuf],
    destination_dir: &std::path::Path,
    conflict: super::ConflictPolicy,
    decryption: PayloadDecryption,
    selection: Option<&super::RestoreSelection>,
) -> Result<(), anyhow::Error> {
    use super::engine::RestoreEngine;
    use super::sinks::LocalFsSink;

    let files = part_paths
        .iter()
        .map(File::open)
        .collect::<Result<Vec<_>, _>>()?;
    let reader: Box<dyn Read> = Box::new(ConcatReader { files, index: 0 });

    let mut sink = LocalFsSink::new(destination_dir.to_path_buf(), conflict);
    let mut engine = RestoreEngine::new(&mut sink, decryption, selection)?;
    engine.restore(reader)?;
    Ok(())
}

#[cfg(test)]
struct ConcatReader {
    files: Vec<File>,
    index: usize,
}

#[cfg(test)]
impl Read for ConcatReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        loop {
            if self.index >= self.files.len() {
                return Ok(0);
            }
            let n = self.files[self.index].read(buf)?;
            if n == 0 {
                self.index += 1;
                continue;
            }
            return Ok(n);
        }
    }
}
