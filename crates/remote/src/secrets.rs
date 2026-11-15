//! A storage's secrets, in the system's keychain, under the storage's id.

use std::io;

use packwing_core::Values;

const SERVICE: &str = "packwing storages";

/// keyring's own `Entry` stands for a store it sets itself, which it has none of
/// on iOS: there the system keychain's is set, and its entries are keyring-core's.
#[cfg(not(target_os = "ios"))]
pub(crate) type Entry = keyring::Entry;
#[cfg(target_os = "ios")]
pub(crate) type Entry = keyring_core::Entry;

/// The entry `account` of `service`, in the system's keychain.
pub(crate) fn keychain(service: &str, account: &str) -> io::Result<Entry> {
    #[cfg(target_os = "ios")]
    phone_store()?;
    Entry::new(service, account).map_err(io::Error::other)
}

fn entry(id: &str, key: &str) -> io::Result<Entry> {
    keychain(SERVICE, &format!("{id}/{key}"))
}

/// On iOS the keyring crate picks no store: the system's own, its keychain of
/// protected data (the one iOS has), is set the first time one is needed.
#[cfg(target_os = "ios")]
pub(crate) fn phone_store() -> io::Result<()> {
    use std::sync::OnceLock;
    static SET: OnceLock<Result<(), String>> = OnceLock::new();
    SET.get_or_init(|| {
        let store =
            apple_native_keyring_store::protected::Store::new().map_err(|e| e.to_string())?;
        keyring_core::set_default_store(store);
        Ok(())
    })
    .clone()
    .map_err(io::Error::other)
}

pub fn keep(id: &str, secrets: &Values) -> io::Result<()> {
    for (key, value) in secrets {
        entry(id, key)?
            .set_password(value)
            .map_err(io::Error::other)?;
    }
    Ok(())
}

/// The secret kept as `key`; empty if there is none.
pub fn get(id: &str, key: &str) -> io::Result<String> {
    match entry(id, key)?.get_password() {
        Ok(secret) => Ok(secret),
        Err(keyring_core::Error::NoEntry) => Ok(String::new()),
        Err(e) => Err(io::Error::other(e)),
    }
}

/// The secrets kept as `keys`, by key.
pub fn kept(id: &str, keys: &[&str]) -> io::Result<Values> {
    keys.iter()
        .map(|key| Ok(((*key).to_owned(), get(id, key)?)))
        .collect()
}

pub fn forget(id: &str, keys: &[&str]) {
    for key in keys {
        if let Ok(entry) = entry(id, key) {
            let _ = entry.delete_credential();
        }
    }
}
