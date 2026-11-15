//! SSH keys packwing keeps, for where there are no key files to reach (the Mac App
//! Store's sandbox, a phone) or for a user who would rather not use them.
//!
//! Each key is kept in the keychain, with no passphrase of its own, as the
//! keychain is locked already; the list of them, with nothing secret, is in
//! packwing's settings folder (see [`crate::keep_in`]). A storage over SFTP
//! names its key by id, its `key` value; with none, every key kept is tried after
//! the key files and the agent's. The passphrases of key files the user asked to
//! keep are kept in the keychain too.

use std::fmt;
use std::io;
use std::path::PathBuf;

use packwing_i18n::tr;
use shellwing_ssh::keys::{self, ReadError, StoredKey};

use crate::trust;

const SERVICE: &str = "packwing ssh keys";

/// A key kept: what is shown of it, with no secret.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeptKey {
    /// What a storage names it by: its fingerprint.
    pub id: String,
    pub name: String,
    /// `ED25519`, `ECDSA P-256`, `RSA 3072`…
    pub kind: String,
    /// `SHA256:…`, as `ssh-keygen -l` writes it.
    pub fingerprint: String,
    /// Its public half, one line, as `authorized_keys` takes it.
    pub public: String,
}

/// Why a key was not taken.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyRefused {
    /// It has a passphrase, and none, or not the right one, was given.
    Passphrase,
    /// It is not a private key in OpenSSH's format.
    NotAKey,
}

impl fmt::Display for KeyRefused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&match self {
            KeyRefused::Passphrase => tr!("The key has a passphrase: type it to open the key."),
            KeyRefused::NotAKey => tr!(
                "That is not a private key. Paste the whole of the key file, from its BEGIN OPENSSH PRIVATE KEY line to its END line."
            ),
        })
    }
}

impl std::error::Error for KeyRefused {}

fn entry(account: &str) -> io::Result<crate::secrets::Entry> {
    // As secrets.rs has it: on iOS, the system keychain's store and entries.
    crate::secrets::keychain(SERVICE, account)
}

fn get(account: &str) -> io::Result<Option<String>> {
    match entry(account)?.get_password() {
        Ok(secret) => Ok(Some(secret)),
        Err(keyring_core::Error::NoEntry) => Ok(None),
        Err(e) => Err(io::Error::other(e)),
    }
}

/// The list of the keys kept.
fn list_file() -> io::Result<PathBuf> {
    trust::folder().map(|f| f.join("keys")).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "packwing has no settings folder to keep keys in",
        )
    })
}

/// A line of the list: tab-separated, so no field may hold a tab or a line end.
fn line(key: &KeptKey) -> String {
    let clean = |s: &str| s.replace(['\t', '\n', '\r'], " ");
    [&key.id, &key.kind, &key.fingerprint, &key.public, &key.name]
        .map(|s| clean(s))
        .join("\t")
}

fn parse_line(line: &str) -> Option<KeptKey> {
    let mut fields = line.splitn(5, '\t');
    Some(KeptKey {
        id: fields.next()?.to_owned(),
        kind: fields.next()?.to_owned(),
        fingerprint: fields.next()?.to_owned(),
        public: fields.next()?.to_owned(),
        name: fields.next()?.to_owned(),
    })
}

fn write_list(keys: &[KeptKey]) -> io::Result<()> {
    let file = list_file()?;
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let text: String = keys.iter().map(|k| line(k) + "\n").collect();
    let part = file.with_extension("part");
    std::fs::write(&part, text)?;
    std::fs::rename(part, file)
}

/// The keys kept, in the order they were added.
pub fn kept_keys() -> io::Result<Vec<KeptKey>> {
    let file = list_file()?;
    match std::fs::read_to_string(file) {
        Ok(text) => Ok(text.lines().filter_map(parse_line).collect()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(e),
    }
}

/// Keeps the private key `text` (OpenSSH's format, as a key file has it) as
/// `name`, opening it with `passphrase` if it has one. The same key again is
/// kept once, under the newer name. A key that is not taken is a [`KeyRefused`]
/// in the error.
pub fn import_key(name: &str, text: &str, passphrase: Option<&str>) -> io::Result<KeptKey> {
    let key = keys::parse(text, passphrase).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            match e {
                ReadError::Passphrase => KeyRefused::Passphrase,
                ReadError::NotAKey(_) => KeyRefused::NotAKey,
            },
        )
    })?;
    let info = keys::info(key.public_key());
    let kept = KeptKey {
        id: info.fingerprint.clone(),
        name: Some(name.trim())
            .filter(|n| !n.is_empty())
            .map_or_else(|| info.comment.clone(), str::to_owned),
        kind: info.kind,
        fingerprint: info.fingerprint,
        public: info.public,
    };
    let openssh = keys::to_vault(&key).map_err(|e| io::Error::other(e.to_string()))?;
    let mut list = kept_keys()?;
    entry(&kept.id)?
        .set_password(&openssh)
        .map_err(io::Error::other)?;
    list.retain(|k| k.id != kept.id);
    list.push(kept.clone());
    write_list(&list)?;
    Ok(kept)
}

/// Forgets the key kept as `id`.
pub fn remove_key(id: &str) -> io::Result<()> {
    let mut list = kept_keys()?;
    list.retain(|k| k.id != id);
    write_list(&list)?;
    match entry(id)?.delete_credential() {
        Ok(()) | Err(keyring_core::Error::NoEntry) => Ok(()),
        Err(e) => Err(io::Error::other(e)),
    }
}

/// The keys kept, as signing in takes them; one that cannot be read is left out.
pub(crate) fn stored() -> Vec<StoredKey> {
    kept_keys()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|k| {
            Some(StoredKey {
                openssh: keys::Zeroizing::new(get(&k.id).ok()??),
                id: k.id,
                name: k.name,
            })
        })
        .collect()
}

/// The passphrase kept for the key file whose fingerprint is `fingerprint`.
pub(crate) fn passphrase(fingerprint: &str) -> Option<String> {
    get(&format!("passphrase/{fingerprint}")).ok().flatten()
}

pub(crate) fn keep_passphrase(fingerprint: &str, passphrase: &str) {
    if let Ok(entry) = entry(&format!("passphrase/{fingerprint}")) {
        let _ = entry.set_password(passphrase);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_line_of_the_list_reads_back() {
        let key = KeptKey {
            id: "SHA256:abc".into(),
            name: "work\tlaptop".into(),
            kind: "ED25519".into(),
            fingerprint: "SHA256:abc".into(),
            public: "ssh-ed25519 AAAA me@here".into(),
        };
        let back = parse_line(&line(&key)).expect("read back");
        assert_eq!(back.name, "work laptop");
        assert_eq!(back.public, key.public);
    }

    #[test]
    fn what_is_not_a_key_is_refused_as_such() {
        let e = import_key("x", "hello", None).expect_err("not a key");
        let why = e.get_ref().and_then(|e| e.downcast_ref::<KeyRefused>());
        assert_eq!(why, Some(&KeyRefused::NotAKey));
    }
}
