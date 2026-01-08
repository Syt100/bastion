use tempfile::TempDir;

use super::{EncryptedSecret, SecretsCrypto, export_keypack, import_keypack, rotate_master_key};

#[test]
fn keypack_round_trip() {
    let temp = TempDir::new().unwrap();
    let data_dir = temp.path();

    let crypto1 = SecretsCrypto::load_or_create(data_dir).unwrap();
    let encrypted: EncryptedSecret = crypto1
        .encrypt("hub", "webdav", "primary", b"secret")
        .unwrap();

    let pack_path = data_dir.join("keypack.json");
    export_keypack(data_dir, &pack_path, "pw1").unwrap();

    let temp2 = TempDir::new().unwrap();
    import_keypack(temp2.path(), &pack_path, "pw1", false).unwrap();

    let crypto2 = SecretsCrypto::load_or_create(temp2.path()).unwrap();
    let plain = crypto2
        .decrypt("hub", "webdav", "primary", &encrypted)
        .unwrap();
    assert_eq!(plain, b"secret");

    assert!(import_keypack(temp2.path(), &pack_path, "pw1", false).is_err());
    import_keypack(temp2.path(), &pack_path, "pw1", true).unwrap();
}

#[test]
fn keypack_wrong_password_fails() {
    let temp = TempDir::new().unwrap();
    let data_dir = temp.path();

    let pack_path = data_dir.join("keypack.json");
    export_keypack(data_dir, &pack_path, "pw1").unwrap();

    let temp2 = TempDir::new().unwrap();
    assert!(import_keypack(temp2.path(), &pack_path, "pw2", false).is_err());
}

#[test]
fn rotate_preserves_old_keys() {
    let temp = TempDir::new().unwrap();
    let data_dir = temp.path();

    let crypto1 = SecretsCrypto::load_or_create(data_dir).unwrap();
    let encrypted1: EncryptedSecret = crypto1
        .encrypt("hub", "webdav", "primary", b"secret")
        .unwrap();
    assert_eq!(encrypted1.kid, crypto1.active_kid());

    let rotated = rotate_master_key(data_dir).unwrap();
    assert_ne!(rotated.previous_kid, rotated.active_kid);

    let crypto2 = SecretsCrypto::load_or_create(data_dir).unwrap();
    let encrypted2: EncryptedSecret = crypto2
        .encrypt("hub", "webdav", "primary", b"secret2")
        .unwrap();
    assert_eq!(encrypted2.kid, crypto2.active_kid());
    assert_ne!(encrypted1.kid, encrypted2.kid);

    let plain = crypto2
        .decrypt("hub", "webdav", "primary", &encrypted1)
        .unwrap();
    assert_eq!(plain, b"secret");
}
