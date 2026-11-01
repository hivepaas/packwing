//! Storages across the network, as `packwing_core::StorageKind`s: the app knows
//! them only through that trait.

mod dal;
mod s3;
mod secrets;
mod sftp;
mod webdav;

use std::sync::Arc;

use packwing_core::StorageKind;

pub use dal::Dal;
pub use s3::{S3Kind, S3Settings, s3};
pub use secrets::get as secret;
pub use sftp::{SftpKind, SftpSettings, SftpStorage, sftp};
pub use webdav::{WebDavKind, WebDavSettings, webdav};

/// The kinds there are.
pub fn kinds() -> Vec<Arc<dyn StorageKind>> {
    vec![Arc::new(S3Kind), Arc::new(SftpKind), Arc::new(WebDavKind)]
}
