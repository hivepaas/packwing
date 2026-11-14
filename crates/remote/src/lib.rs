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

/// The kinds there are; in the Mac App Store's build, all but SFTP.
pub fn kinds() -> Vec<Arc<dyn StorageKind>> {
    let mut kinds: Vec<Arc<dyn StorageKind>> = vec![Arc::new(S3Kind)];
    #[cfg(not(feature = "app-store"))]
    kinds.push(Arc::new(SftpKind));
    kinds.push(Arc::new(WebDavKind));
    kinds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_is_there_but_sftp_in_the_store_build() {
        let ids: Vec<&str> = kinds().iter().map(|k| k.id()).collect();
        let mut wanted = vec![S3Kind.id(), SftpKind.id(), WebDavKind.id()];
        if cfg!(feature = "app-store") {
            wanted.retain(|id| *id != SftpKind.id());
        }
        assert_eq!(ids, wanted);
    }
}
