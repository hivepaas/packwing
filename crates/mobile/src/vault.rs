//! Android's keychain: where a storage's secrets and the keys kept are, as
//! the system's keychain holds them on the desktop and on iOS. Only built for
//! Android.
//!
//! The keyring crate has no store for Android it can set by itself: the one
//! there is reaches the Android Keystore through JNI, and the app loads this
//! library through JNA, with no JNI of its own. So the app is the store: it
//! keeps each secret encrypted with a key of the Android Keystore, which never
//! leaves it ([`Vault`]), and packwing-remote's entries are keyring-core's,
//! through [`VaultStore`], set as the default store once the app hands its
//! vault over ([`keep_secrets_in`]).

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use keyring_core::api::{Credential, CredentialApi, CredentialStoreApi};
use keyring_core::{Entry, Error};

use crate::MobileError;

/// Where the app keeps secrets: encrypted with a key of the Android Keystore.
/// Each is told by its service and its account, as a keychain tells it.
///
/// Called from whatever thread the work is on, and from several at once.
#[uniffi::export(with_foreign)]
pub trait Vault: Send + Sync {
    /// The secret kept, or none.
    fn get(&self, service: String, account: String) -> Result<Option<Vec<u8>>, MobileError>;

    /// Keeps `secret`, in place of one kept already.
    fn set(&self, service: String, account: String, secret: Vec<u8>) -> Result<(), MobileError>;

    /// Forgets the secret; whether there was one.
    fn delete(&self, service: String, account: String) -> Result<bool, MobileError>;
}

/// Keeps the secrets of storages, and the keys kept, in `vault` from now on.
/// The app hands its vault over as it starts, before anything asks for one.
#[uniffi::export]
pub fn keep_secrets_in(vault: Arc<dyn Vault>) {
    keyring_core::set_default_store(Arc::new(VaultStore(vault)));
}

/// The app's vault, as keyring-core's store.
struct VaultStore(Arc<dyn Vault>);

impl CredentialStoreApi for VaultStore {
    fn vendor(&self) -> String {
        "packwing, in the Android Keystore".to_owned()
    }

    fn id(&self) -> String {
        "packwing-vault".to_owned()
    }

    fn build(
        &self,
        service: &str,
        user: &str,
        _modifiers: Option<&HashMap<&str, &str>>,
    ) -> keyring_core::Result<Entry> {
        Ok(Entry::new_with_credential(Arc::new(Secret {
            vault: Arc::clone(&self.0),
            service: service.to_owned(),
            account: user.to_owned(),
        })))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// One secret of the vault.
struct Secret {
    vault: Arc<dyn Vault>,
    service: String,
    account: String,
}

/// What the vault said went wrong, as keyring-core says it.
fn failed(e: MobileError) -> Error {
    Error::PlatformFailure(Box::new(e))
}

impl CredentialApi for Secret {
    fn set_secret(&self, secret: &[u8]) -> keyring_core::Result<()> {
        self.vault
            .set(self.service.clone(), self.account.clone(), secret.to_vec())
            .map_err(failed)
    }

    fn get_secret(&self) -> keyring_core::Result<Vec<u8>> {
        self.vault
            .get(self.service.clone(), self.account.clone())
            .map_err(failed)?
            .ok_or(Error::NoEntry)
    }

    fn delete_credential(&self) -> keyring_core::Result<()> {
        match self.vault.delete(self.service.clone(), self.account.clone()) {
            Ok(true) => Ok(()),
            Ok(false) => Err(Error::NoEntry),
            Err(e) => Err(failed(e)),
        }
    }

    /// Itself, if it is kept: it wraps no other.
    fn get_credential(&self) -> keyring_core::Result<Option<Arc<Credential>>> {
        self.get_secret().map(|_| None)
    }

    fn get_specifiers(&self) -> Option<(String, String)> {
        Some((self.service.clone(), self.account.clone()))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
