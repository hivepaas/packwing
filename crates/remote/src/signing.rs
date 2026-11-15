//! Keys packwing signs files with, kept in the system's keychain: for a phone,
//! where there are no key files to reach.
//!
//! Each key's file is kept as it was given (an SSH Ed25519 key's, or packwing's
//! hybrid key's), with its passphrase if it has one; the list of them, with
//! nothing secret, is in packwing's settings folder, beside the SSH keys'.

use std::io;
use std::path::PathBuf;

use packwing_core::{SigningKey, read_signing_key};

use crate::trust;

const SERVICE: &str = "packwing signing keys";

/// A signing key kept: what is shown of it, with no secret.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeptSigningKey {
    /// Its fingerprint, which names it in the keychain.
    pub id: String,
    pub name: String,
    /// `SHA256:…`.
    pub fingerprint: String,
    /// Its public half, one line, as a `.pub` file has it.
    pub public: String,
    /// Made by packwing here, rather than given: there is no other copy of it.
    pub made_here: bool,
}

fn entry(account: &str) -> io::Result<crate::secrets::Entry> {
    crate::secrets::keychain(SERVICE, account)
}

fn passphrase_account(id: &str) -> String {
    format!("{id}/passphrase")
}

fn get(account: &str) -> io::Result<Option<Vec<u8>>> {
    match entry(account)?.get_secret() {
        Ok(secret) => Ok(Some(secret)),
        Err(keyring_core::Error::NoEntry) => Ok(None),
        Err(e) => Err(io::Error::other(e)),
    }
}

fn forget(account: &str) -> io::Result<()> {
    match entry(account)?.delete_credential() {
        Ok(()) | Err(keyring_core::Error::NoEntry) => Ok(()),
        Err(e) => Err(io::Error::other(e)),
    }
}

fn list_file() -> io::Result<PathBuf> {
    trust::folder()
        .map(|f| f.join("signing-keys"))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "packwing has no settings folder to keep keys in",
            )
        })
}

/// A line of the list: tab-separated, so no field may hold a tab or a line end.
fn line(key: &KeptSigningKey) -> String {
    let clean = |s: &str| s.replace(['\t', '\n', '\r'], " ");
    let made = if key.made_here { "made" } else { "given" };
    [
        clean(&key.id),
        clean(&key.fingerprint),
        clean(&key.public),
        made.to_owned(),
        clean(&key.name),
    ]
    .join("\t")
}

fn parse_line(line: &str) -> Option<KeptSigningKey> {
    let mut fields = line.splitn(5, '\t');
    Some(KeptSigningKey {
        id: fields.next()?.to_owned(),
        fingerprint: fields.next()?.to_owned(),
        public: fields.next()?.to_owned(),
        made_here: fields.next()? == "made",
        name: fields.next()?.to_owned(),
    })
}

fn write_list(keys: &[KeptSigningKey]) -> io::Result<()> {
    let file = list_file()?;
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let text: String = keys.iter().map(|k| line(k) + "\n").collect();
    let part = file.with_extension("part");
    std::fs::write(&part, text)?;
    std::fs::rename(part, file)
}

/// The signing keys kept, in the order they were added.
pub fn kept_signing_keys() -> io::Result<Vec<KeptSigningKey>> {
    let file = list_file()?;
    match std::fs::read_to_string(file) {
        Ok(text) => Ok(text.lines().filter_map(parse_line).collect()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(e),
    }
}

/// Keeps `key`, read from `file` with `passphrase`, as `name`. The same key again
/// is kept once, under the newer name.
pub fn keep_signing_key(
    name: &str,
    key: &SigningKey,
    file: &[u8],
    passphrase: Option<&[u8]>,
    made_here: bool,
) -> io::Result<KeptSigningKey> {
    let public = key.public();
    let kept = KeptSigningKey {
        id: public.fingerprint(),
        name: name.trim().to_owned(),
        fingerprint: public.fingerprint(),
        public: public.to_string(),
        made_here,
    };
    let mut list = kept_signing_keys()?;
    entry(&kept.id)?
        .set_secret(file)
        .map_err(io::Error::other)?;
    match passphrase {
        Some(passphrase) => entry(&passphrase_account(&kept.id))?
            .set_secret(passphrase)
            .map_err(io::Error::other)?,
        None => forget(&passphrase_account(&kept.id))?,
    }
    // Made here once, it stays so: given again, it is no less the only copy.
    let made_before = list.iter().any(|k| k.id == kept.id && k.made_here);
    list.retain(|k| k.id != kept.id);
    let kept = KeptSigningKey {
        made_here: made_here || made_before,
        ..kept
    };
    list.push(kept.clone());
    write_list(&list)?;
    Ok(kept)
}

/// The key kept as `id`, opened to sign with.
pub fn signing_key(id: &str) -> io::Result<SigningKey> {
    let file = get(id)?
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "this key is no longer kept"))?;
    let passphrase = get(&passphrase_account(id))?;
    read_signing_key(&file, passphrase.as_deref()).map_err(|e| io::Error::other(format!("{e:?}")))
}

/// Forgets the key kept as `id`.
pub fn remove_signing_key(id: &str) -> io::Result<()> {
    let mut list = kept_signing_keys()?;
    list.retain(|k| k.id != id);
    write_list(&list)?;
    forget(id)?;
    forget(&passphrase_account(id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use packwing_core::new_signing_key;

    /// keyring's own store, the Mac's keychain, is set on its first entry: it is
    /// set first, then the mock's in its place, so that nothing reaches the
    /// keychain of the computer running the tests.
    fn mock_keychain() {
        #[cfg(not(target_os = "ios"))]
        let _ = keyring::Entry::store_status();
        keyring_core::set_default_store(keyring_core::mock::Store::new().expect("mock"));
    }

    #[test]
    fn signing_keys_are_kept_opened_and_forgotten() {
        mock_keychain();
        let dir = std::env::temp_dir().join(format!("packwing-signing-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("dir");
        crate::keep_in(dir.clone());

        // Made here: kept with no passphrase.
        let made = new_signing_key().expect("made");
        let file = made.to_file(None).expect("file");
        let kept = keep_signing_key("mine", &made, &file, None, true).expect("kept");
        assert_eq!(kept.public, made.public().to_string());
        assert!(kept.made_here);

        // Given, an SSH key with a passphrase of its own.
        let data = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../core/tests/data");
        let ssh = std::fs::read(data.join("test-ssh-locked.key")).expect("ssh key");
        let given = read_signing_key(&ssh, Some("khoá ssh".as_bytes())).expect("opened");
        let other = keep_signing_key("work", &given, &ssh, Some("khoá ssh".as_bytes()), false)
            .expect("kept");
        assert!(!other.made_here);

        let names: Vec<String> = kept_signing_keys()
            .expect("list")
            .into_iter()
            .map(|k| k.name)
            .collect();
        assert_eq!(names, ["mine", "work"]);
        // Each opens again, as it was kept.
        assert_eq!(signing_key(&kept.id).expect("mine").public(), made.public());
        assert_eq!(
            signing_key(&other.id).expect("work").public(),
            given.public()
        );

        remove_signing_key(&kept.id).expect("removed");
        assert!(signing_key(&kept.id).is_err());
        assert_eq!(kept_signing_keys().expect("list").len(), 1);
        let _ = std::fs::remove_dir_all(dir);
    }
}
