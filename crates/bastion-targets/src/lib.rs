pub mod local_dir;
pub mod webdav;
pub mod webdav_client;

#[derive(Debug, Clone, Copy)]
pub struct StoreRunProgress {
    pub bytes_done: u64,
    pub bytes_total: Option<u64>,
}

pub use webdav_client::{
    WebdavClient, WebdavCredentials, WebdavHttpError, WebdavNotDirectoryError, WebdavPropfindEntry,
};
