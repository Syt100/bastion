use sqlx::{Row, SqlitePool};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::secrets::{EncryptedSecret, SecretsCrypto};

#[derive(Debug, Clone)]
pub struct SecretListItem {
    pub name: String,
    pub updated_at: i64,
}

pub async fn upsert_secret(
    db: &SqlitePool,
    crypto: &SecretsCrypto,
    kind: &str,
    name: &str,
    plaintext: &[u8],
) -> Result<(), anyhow::Error> {
    let encrypted = crypto.encrypt(kind, name, plaintext)?;
    let now = OffsetDateTime::now_utc().unix_timestamp();

    let id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO secrets (id, kind, name, kid, nonce, ciphertext, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(kind, name) DO UPDATE SET
          kid = excluded.kid,
          nonce = excluded.nonce,
          ciphertext = excluded.ciphertext,
          updated_at = excluded.updated_at
        "#,
    )
    .bind(id)
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
    kind: &str,
    name: &str,
) -> Result<Option<Vec<u8>>, anyhow::Error> {
    let row = sqlx::query(
        "SELECT kid, nonce, ciphertext FROM secrets WHERE kind = ? AND name = ? LIMIT 1",
    )
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

    let plaintext = crypto.decrypt(kind, name, &secret)?;
    Ok(Some(plaintext))
}

pub async fn list_secrets(
    db: &SqlitePool,
    kind: &str,
) -> Result<Vec<SecretListItem>, anyhow::Error> {
    let rows = sqlx::query("SELECT name, updated_at FROM secrets WHERE kind = ? ORDER BY name ASC")
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

pub async fn delete_secret(db: &SqlitePool, kind: &str, name: &str) -> Result<bool, anyhow::Error> {
    let result = sqlx::query("DELETE FROM secrets WHERE kind = ? AND name = ?")
        .bind(kind)
        .bind(name)
        .execute(db)
        .await?;
    Ok(result.rows_affected() > 0)
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

        upsert_secret(&pool, &crypto, "webdav", "primary", b"secret1")
            .await
            .expect("upsert");
        let v = get_secret(&pool, &crypto, "webdav", "primary")
            .await
            .expect("get")
            .expect("present");
        assert_eq!(v, b"secret1");

        upsert_secret(&pool, &crypto, "webdav", "primary", b"secret2")
            .await
            .expect("upsert2");
        let v = get_secret(&pool, &crypto, "webdav", "primary")
            .await
            .expect("get2")
            .expect("present2");
        assert_eq!(v, b"secret2");

        let listed = list_secrets(&pool, "webdav").await.expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].name, "primary");

        let deleted = delete_secret(&pool, "webdav", "primary")
            .await
            .expect("delete");
        assert!(deleted);

        let missing = get_secret(&pool, &crypto, "webdav", "primary")
            .await
            .expect("get missing");
        assert!(missing.is_none());
    }
}
