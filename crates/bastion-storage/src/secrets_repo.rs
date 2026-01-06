use sqlx::{Row, SqlitePool};
use time::OffsetDateTime;
use uuid::Uuid;

use bastion_core::HUB_NODE_ID;

use crate::secrets::{EncryptedSecret, SecretsCrypto};

#[derive(Debug, Clone)]
pub struct SecretListItem {
    pub name: String,
    pub updated_at: i64,
}

pub async fn upsert_secret(
    db: &SqlitePool,
    crypto: &SecretsCrypto,
    node_id: &str,
    kind: &str,
    name: &str,
    plaintext: &[u8],
) -> Result<(), anyhow::Error> {
    let node_id = node_id.trim();
    if node_id.is_empty() {
        anyhow::bail!("node_id is required");
    }

    let encrypted = crypto.encrypt(node_id, kind, name, plaintext)?;
    let now = OffsetDateTime::now_utc().unix_timestamp();

    let id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO secrets (id, node_id, kind, name, kid, nonce, ciphertext, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(kind, node_id, name) DO UPDATE SET
          kid = excluded.kid,
          nonce = excluded.nonce,
          ciphertext = excluded.ciphertext,
          updated_at = excluded.updated_at
        "#,
    )
    .bind(id)
    .bind(node_id)
    .bind(kind)
    .bind(name)
    .bind(encrypted.kid as i64)
    .bind(encrypted.nonce.to_vec())
    .bind(encrypted.ciphertext)
    .bind(now)
    .bind(now)
    .execute(db)
    .await?;

    Ok(())
}

pub async fn get_secret(
    db: &SqlitePool,
    crypto: &SecretsCrypto,
    node_id: &str,
    kind: &str,
    name: &str,
) -> Result<Option<Vec<u8>>, anyhow::Error> {
    let row = sqlx::query(
        "SELECT kid, nonce, ciphertext FROM secrets WHERE node_id = ? AND kind = ? AND name = ? LIMIT 1",
    )
    .bind(node_id)
    .bind(kind)
    .bind(name)
    .fetch_optional(db)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let kid = row.get::<i64, _>("kid");
    let nonce = row.get::<Vec<u8>, _>("nonce");
    let ciphertext = row.get::<Vec<u8>, _>("ciphertext");

    let nonce: [u8; 24] = nonce.try_into().map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid nonce length")
    })?;

    let secret = EncryptedSecret {
        kid: kid as u32,
        nonce,
        ciphertext,
    };

    let plaintext = crypto.decrypt(node_id, kind, name, &secret)?;
    Ok(Some(plaintext))
}

pub async fn list_secrets(
    db: &SqlitePool,
    node_id: &str,
    kind: &str,
) -> Result<Vec<SecretListItem>, anyhow::Error> {
    let rows = sqlx::query(
        "SELECT name, updated_at FROM secrets WHERE node_id = ? AND kind = ? ORDER BY name ASC",
    )
    .bind(node_id)
    .bind(kind)
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| SecretListItem {
            name: r.get::<String, _>("name"),
            updated_at: r.get::<i64, _>("updated_at"),
        })
        .collect())
}

pub async fn delete_secret(
    db: &SqlitePool,
    node_id: &str,
    kind: &str,
    name: &str,
) -> Result<bool, anyhow::Error> {
    let result = sqlx::query("DELETE FROM secrets WHERE node_id = ? AND kind = ? AND name = ?")
        .bind(node_id)
        .bind(kind)
        .bind(name)
        .execute(db)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn secret_exists(
    db: &SqlitePool,
    node_id: &str,
    kind: &str,
    name: &str,
) -> Result<bool, anyhow::Error> {
    let row =
        sqlx::query("SELECT 1 FROM secrets WHERE node_id = ? AND kind = ? AND name = ? LIMIT 1")
            .bind(node_id)
            .bind(kind)
            .bind(name)
            .fetch_optional(db)
            .await?;
    Ok(row.is_some())
}

pub async fn list_secrets_hub(
    db: &SqlitePool,
    kind: &str,
) -> Result<Vec<SecretListItem>, anyhow::Error> {
    list_secrets(db, HUB_NODE_ID, kind).await
}

pub async fn get_secret_hub(
    db: &SqlitePool,
    crypto: &SecretsCrypto,
    kind: &str,
    name: &str,
) -> Result<Option<Vec<u8>>, anyhow::Error> {
    get_secret(db, crypto, HUB_NODE_ID, kind, name).await
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use crate::db;
    use crate::secrets::SecretsCrypto;

    use super::{delete_secret, get_secret, list_secrets, upsert_secret};

    #[tokio::test]
    async fn secrets_round_trip() {
        let temp = TempDir::new().expect("tempdir");
        let pool = db::init(temp.path()).await.expect("db init");
        let crypto = SecretsCrypto::load_or_create(temp.path()).expect("crypto");

        upsert_secret(&pool, &crypto, "hub", "webdav", "primary", b"secret1")
            .await
            .expect("upsert");
        let v = get_secret(&pool, &crypto, "hub", "webdav", "primary")
            .await
            .expect("get")
            .expect("present");
        assert_eq!(v, b"secret1");

        upsert_secret(&pool, &crypto, "hub", "webdav", "primary", b"secret2")
            .await
            .expect("upsert2");
        let v = get_secret(&pool, &crypto, "hub", "webdav", "primary")
            .await
            .expect("get2")
            .expect("present2");
        assert_eq!(v, b"secret2");

        let listed = list_secrets(&pool, "hub", "webdav").await.expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].name, "primary");

        let deleted = delete_secret(&pool, "hub", "webdav", "primary")
            .await
            .expect("delete");
        assert!(deleted);

        let missing = get_secret(&pool, &crypto, "hub", "webdav", "primary")
            .await
            .expect("get missing");
        assert!(missing.is_none());
    }
}
