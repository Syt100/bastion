#[cfg(windows)]
use std::path::Path;
use std::path::PathBuf;

fn main() {
    let icon_path = workspace_icon_path();
    println!("cargo:rerun-if-changed={}", icon_path.display());

    #[cfg(windows)]
    compile_windows_resources(&icon_path)
        .unwrap_or_else(|err| panic!("failed to embed Windows icon resource: {err}"));
}

fn workspace_icon_path() -> PathBuf {
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set"));
    manifest_dir.join("../../assets/branding/bastion.ico")
}

#[cfg(windows)]
fn compile_windows_resources(icon_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if !icon_path.exists() {
        return Err(format!("icon file not found: {}", icon_path.display()).into());
    }

    let mut res = winres::WindowsResource::new();
    res.set_icon(icon_path.to_string_lossy().as_ref());
    res.compile()?;
    Ok(())
}
