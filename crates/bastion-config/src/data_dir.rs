use std::{
    env,
    fs::{self, OpenOptions},
    io,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;
use rand::RngCore;

pub fn resolve_data_dir(cli_override: Option<PathBuf>) -> Result<PathBuf, anyhow::Error> {
    if let Some(path) = cli_override {
        ensure_writable(&path)?;
        return Ok(path);
    }

    if let Ok(path) = env::var("BASTION_DATA_DIR") {
        let path = PathBuf::from(path);
        ensure_writable(&path)?;
        return Ok(path);
    }

    let default = default_exe_data_dir()?;
    if ensure_writable(&default).is_ok() {
        return Ok(default);
    }

    let fallback = fallback_data_dir()?;
    ensure_writable(&fallback)?;
    Ok(fallback)
}

fn default_exe_data_dir() -> Result<PathBuf, anyhow::Error> {
    let exe = env::current_exe()?;
    let exe_dir = exe
        .parent()
        .ok_or_else(|| io::Error::other("executable has no parent dir"))?;
    Ok(exe_dir.join("data"))
}

fn fallback_data_dir() -> Result<PathBuf, anyhow::Error> {
    #[cfg(windows)]
    {
        if let Ok(program_data) = env::var("PROGRAMDATA") {
            return Ok(PathBuf::from(program_data).join("bastion").join("data"));
        }
    }

    let project_dirs = ProjectDirs::from("io", "bastion", "bastion")
        .ok_or_else(|| io::Error::other("unable to determine data dir"))?;
    Ok(project_dirs.data_local_dir().join("data"))
}

fn ensure_writable(dir: &Path) -> Result<(), anyhow::Error> {
    fs::create_dir_all(dir)?;

    let mut name = [0_u8; 16];
    rand::rng().fill_bytes(&mut name);
    let file_name = format!(".bastion_write_test_{}", hex::encode(name));
    let test_path = dir.join(file_name);

    OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&test_path)?;
    fs::remove_file(&test_path)?;
    Ok(())
}
