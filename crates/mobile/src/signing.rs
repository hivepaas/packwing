//! Signing files, as the desktop signs them, with keys kept in the system's
//! keychain (packwing-remote): one given as a file (an SSH Ed25519 key's, or a
//! hybrid key's packwing made), or made here. The signature is `name.sig`, in
//! the format of `ssh-keygen -Y sign`.

use std::sync::Arc;

use packwing_core::{
    KeyRefused, Source, Storage, is_part_name, new_signing_key, read_signing_key, sign_file,
    signature_of,
};
use packwing_i18n::tr;
use packwing_remote::KeptSigningKey;

use crate::{Archive, Asks, Job, MobileError};

/// A signing key kept, with no secret.
#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct SigningKeyRow {
    pub id: String,
    pub name: String,
    /// `SHA256:…`.
    pub fingerprint: String,
    /// Its public half, one line, as a `.pub` file has it: what others trust.
    pub public: String,
    /// Made here: there is no other copy of it until one is saved.
    pub made_here: bool,
}

impl From<KeptSigningKey> for SigningKeyRow {
    fn from(k: KeptSigningKey) -> SigningKeyRow {
        SigningKeyRow {
            id: k.id,
            name: k.name,
            fingerprint: k.fingerprint,
            public: k.public,
            made_here: k.made_here,
        }
    }
}

fn failed(e: std::io::Error) -> MobileError {
    MobileError::Failed {
        reason: e.to_string(),
    }
}

/// The signing keys kept, in the order they were added.
#[uniffi::export]
pub fn signing_keys() -> Result<Vec<SigningKeyRow>, MobileError> {
    Ok(packwing_remote::kept_signing_keys()
        .map_err(failed)?
        .into_iter()
        .map(Into::into)
        .collect())
}

/// Keeps the key in `contents`, a key file, as `name`, opening it with
/// `passphrase` if it has one: its passphrase is asked for (Locked), or said
/// to be wrong (WrongPassword), and a file that is no key says why.
#[uniffi::export]
pub fn import_signing_key(
    name: String,
    contents: Vec<u8>,
    passphrase: Option<String>,
) -> Result<SigningKeyRow, MobileError> {
    let passphrase = passphrase.filter(|p| !p.is_empty());
    let key = read_signing_key(&contents, passphrase.as_deref().map(str::as_bytes)).map_err(
        |refused| match refused {
            KeyRefused::NeedsPassphrase => MobileError::Locked {
                reason: tr!("The key file’s passphrase"),
                passphrase: false,
                asks: Asks::KeyPassphrase,
            },
            KeyRefused::WrongPassphrase => MobileError::WrongPassword {
                reason: tr!("That passphrase does not open this key."),
            },
            KeyRefused::Not(why) => MobileError::Failed {
                reason: tr!(
                    "This is not a key packwing can sign with: {why}.",
                    why = why
                ),
            },
        },
    )?;
    packwing_remote::keep_signing_key(
        &name,
        &key,
        &contents,
        passphrase.as_deref().map(str::as_bytes),
        false,
    )
    .map(Into::into)
    .map_err(failed)
}

/// Makes a key of Ed25519 and ML-DSA-65, and keeps it as `name`.
#[uniffi::export]
pub fn make_signing_key(name: String) -> Result<SigningKeyRow, MobileError> {
    let key = new_signing_key()?;
    let mut file = key.to_file(None)?;
    let kept = packwing_remote::keep_signing_key(&name, &key, &file, None, true);
    file.fill(0);
    kept.map(Into::into).map_err(failed)
}

/// The key made here kept as `id`, as a file encrypted with `passphrase`: a
/// copy to keep elsewhere, which the desktop and `pkw` sign with too.
#[uniffi::export]
pub fn export_signing_key(id: String, passphrase: String) -> Result<Vec<u8>, MobileError> {
    let key = packwing_remote::signing_key(&id).map_err(failed)?;
    Ok(key.to_file(Some(passphrase.as_bytes()))?)
}

/// Forgets the key kept as `id`.
#[uniffi::export]
pub fn remove_signing_key(id: String) -> Result<(), MobileError> {
    packwing_remote::remove_signing_key(&id).map_err(failed)
}

/// Whether a file of this name may be signed: not one part of a file cut in
/// several, which a signature would seem to cover whole.
#[uniffi::export]
pub fn may_sign(name: String) -> bool {
    !is_part_name(&name)
}

#[uniffi::export]
impl Archive {
    /// Whether its file is on a storage across the network, rather than on
    /// this phone.
    pub fn stored(&self) -> bool {
        matches!(self.source, Source::Stored(_))
    }

    /// Signs its file with the key kept as `key_id`: the signature goes beside
    /// it, as `name.sig`, in place of one that is there.
    pub fn sign_beside(&self, key_id: String, job: Arc<Job>) -> Result<(), MobileError> {
        let key = packwing_remote::signing_key(&key_id).map_err(failed)?;
        let (storage, path) = self.place()?;
        Ok(sign_file(storage.as_ref(), &path, &key, &job.control())?)
    }

    /// Its file's signature by the key kept as `key_id`, armored as `name.sig`
    /// holds it, written nowhere: for where its folder cannot be written to.
    pub fn signature(&self, key_id: String, job: Arc<Job>) -> Result<String, MobileError> {
        let key = packwing_remote::signing_key(&key_id).map_err(failed)?;
        let (storage, path) = self.place()?;
        Ok(signature_of(storage.as_ref(), &path, &key, &job.control())?)
    }
}

impl Archive {
    fn place(&self) -> Result<(Arc<dyn Storage>, Vec<String>), MobileError> {
        self.source.place().ok_or_else(|| MobileError::Failed {
            reason: tr!("There is no file to sign."),
        })
    }
}
