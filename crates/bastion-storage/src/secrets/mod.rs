const KEYRING_VERSION: u32 = 1;
const ACTIVE_KID_START: u32 = 1;
const MASTER_KEY_FILE: &str = "master.key";

const KEYPACK_VERSION: u32 = 1;
const KEYPACK_AAD: &[u8] = b"bastion-keypack-v1";

mod crypto;
mod io;
mod keypack;
mod keyring;

pub use crypto::{EncryptedSecret, SecretsCrypto};
pub use keypack::{export_keypack, import_keypack};
pub use keyring::{KeyRotationResult, rotate_master_key};

#[cfg(test)]
mod tests;
