//! age with keys, as the desktop has it: an age file opened with the key file
//! it was encrypted to (age's keys, or an SSH key, maybe protected itself),
//! and a new one encrypted to public keys, typed or read from a file of them.

use packwing_i18n::tr;

use crate::MobileError;

/// Whether `contents` is a key file packwing can open an age file with; if
/// not, why, as the app says it. One protected by a passphrase is one, its
/// passphrase asked for when it is used.
#[uniffi::export]
pub fn check_key_file(contents: Vec<u8>) -> Result<(), MobileError> {
    packwing_core::check_key_file(&contents).map_err(|why| MobileError::Failed {
        reason: tr!("This is not a key file packwing can use: {why}.", why = why),
    })
}

/// Why `key` is not a public key age encrypts to, if it is not.
#[uniffi::export]
pub fn age_key_problem(key: String) -> Option<String> {
    packwing_core::age_key_problem(key.trim())
}

/// Whether `key` is post-quantum: only age 1.3 and newer opens what is
/// encrypted to it.
#[uniffi::export]
pub fn age_key_is_post_quantum(key: String) -> bool {
    packwing_core::age_key_is_post_quantum(key.trim())
}

/// The public keys in a file of them, as `age -R` reads it.
#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct AgeKeysFile {
    pub keys: Vec<String>,
    /// Of a kind age does not encrypt to, so left out.
    pub left_out: u64,
}

/// The public keys in `contents`, a file of them; or why there are none to
/// encrypt to.
#[uniffi::export]
pub fn age_keys_in(contents: Vec<u8>) -> Result<AgeKeysFile, MobileError> {
    let (keys, left_out) =
        packwing_core::age_keys_in(&contents).map_err(|reason| MobileError::Failed { reason })?;
    Ok(AgeKeysFile {
        keys,
        left_out: left_out as u64,
    })
}
