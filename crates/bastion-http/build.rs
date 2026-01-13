use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    println!("cargo:rerun-if-env-changed=BASTION_VERSION");
    println!("cargo:rerun-if-env-changed=BASTION_BUILD_TIME_UNIX");
    println!("cargo:rerun-if-env-changed=SOURCE_DATE_EPOCH");

    let version = std::env::var("BASTION_VERSION")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".into()));
    println!("cargo:rustc-env=BASTION_VERSION={version}");

    let build_time_unix = std::env::var("BASTION_BUILD_TIME_UNIX")
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .or_else(|| {
            std::env::var("SOURCE_DATE_EPOCH")
                .ok()
                .and_then(|v| v.parse::<i64>().ok())
        })
        .unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64
        });
    println!("cargo:rustc-env=BASTION_BUILD_TIME_UNIX={build_time_unix}");
}
