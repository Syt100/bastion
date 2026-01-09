use serde::Serialize;

mod node_validation;
mod smtp;
mod webdav;
mod wecom_bot;

pub(super) use smtp::{delete_smtp_secret, get_smtp_secret, list_smtp_secrets, upsert_smtp_secret};
pub(super) use webdav::{
    delete_webdav_secret, delete_webdav_secret_node, get_webdav_secret, get_webdav_secret_node,
    list_webdav_secrets, list_webdav_secrets_node, upsert_webdav_secret, upsert_webdav_secret_node,
};
pub(super) use wecom_bot::{
    delete_wecom_bot_secret, get_wecom_bot_secret, list_wecom_bot_secrets, upsert_wecom_bot_secret,
};

#[derive(Debug, Serialize)]
pub(super) struct SecretListItem {
    name: String,
    updated_at: i64,
}
