//! An archive's signature, checked as the desktop checks one: it says who
//! signed the archive only by a key the user trusts, and a signature by any
//! other key proves nothing, whatever it matches. Checking takes no license;
//! only signing does, and the phones do not sign.
//!
//! The signature is a file of its own, `name.sig` beside the archive. A phone
//! cannot always reach the file beside one (iOS gives an app the file chosen,
//! not its folder), so the app gives the signature it found, or the one the
//! user chose.

use std::fs;
use std::sync::Arc;

use packwing_core::{PublicKey, Signed, check_signature_given, read_public_key, signature_key};
use packwing_i18n::tr;

use crate::{Archive, Job, MobileError};

/// The most of a signature file read: far more than a signature is, as a file
/// of that name may be anything.
const MOST_SIGNATURE: u64 = 64 << 10;

/// What a signature says of the archive.
#[derive(Clone, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum SignatureState {
    /// It is no signature packwing reads, and why.
    Unreadable { reason: String },
    /// By a key the user does not trust: the archive was not read, as that
    /// would prove nothing either way. `key` is the key, to trust it.
    Stranger { fingerprint: String, key: String },
    /// All of the archive is what the key the user trusts signed.
    Good { fingerprint: String },
    /// The signature names a key the user trusts, and the archive is not what
    /// it signed: changed since, or not the file it was made for.
    Mismatch { fingerprint: String },
    /// The archive or its signature could not be read, and why.
    Failed { reason: String },
}

/// A public key, read from what was pasted or from a file of them.
#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct KeyRead {
    /// The key alone, as it is kept: its kind and its base64.
    pub line: String,
    /// How it is told from others, as `ssh-keygen -l` says it.
    pub fingerprint: String,
}

impl From<&PublicKey> for KeyRead {
    fn from(key: &PublicKey) -> KeyRead {
        KeyRead {
            line: key.to_string(),
            fingerprint: key.fingerprint(),
        }
    }
}

/// The first public key in `text` (a line as `ssh-keygen` writes one, or a
/// file of them), or why there is none.
#[uniffi::export]
pub fn read_key(text: String) -> Result<KeyRead, MobileError> {
    read_public_key(&text)
        .map(|key| KeyRead::from(&key))
        .map_err(|why| MobileError::Failed {
            reason: tr!(
                "This is not a public key packwing can check with: {why}.",
                why = why
            ),
        })
}

#[uniffi::export]
impl Archive {
    /// Checks the archive against the signature in the file at `signature`,
    /// with the keys trusted (`trusted`, as [`read_key`] gives them): one by a
    /// key trusted against all of the archive, which `job` is told of.
    pub fn check_signature(
        &self,
        signature: String,
        trusted: Vec<String>,
        job: Arc<Job>,
    ) -> Result<SignatureState, MobileError> {
        let bytes = match read_most(&signature) {
            Ok(bytes) => bytes,
            Err(reason) => return Ok(SignatureState::Failed { reason }),
        };
        let trusted: Vec<PublicKey> = trusted
            .iter()
            .filter_map(|line| read_public_key(line).ok())
            .collect();
        let Some((storage, path)) = self.source.place() else {
            return Ok(SignatureState::Failed {
                reason: tr!("Its signature could not be checked"),
            });
        };
        let signed =
            check_signature_given(storage.as_ref(), &path, &bytes, &trusted, &job.control())?;
        Ok(match signed {
            Signed::Good { fingerprint } => SignatureState::Good { fingerprint },
            Signed::Mismatch { fingerprint } => SignatureState::Mismatch { fingerprint },
            Signed::Stranger { fingerprint } => SignatureState::Stranger {
                fingerprint,
                key: signature_key(&bytes)
                    .map(|k| k.to_string())
                    .unwrap_or_default(),
            },
            Signed::Unreadable(reason) => SignatureState::Unreadable { reason },
            Signed::Failed(reason) => SignatureState::Failed { reason },
            // Given, it is there; and given, it is checked.
            Signed::No | Signed::NotChecked { .. } => SignatureState::Failed {
                reason: tr!("Its signature could not be checked"),
            },
        })
    }
}

/// The file at `path`, if it is no bigger than a signature can be.
fn read_most(path: &str) -> Result<Vec<u8>, String> {
    let size = fs::metadata(path).map_err(|e| e.to_string())?.len();
    if size > MOST_SIGNATURE {
        return Err(tr!("Its signature cannot be read"));
    }
    fs::read(path).map_err(|e| e.to_string())
}
