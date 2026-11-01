//! A storage's secrets, in the system's keychain, under the storage's id.

use std::io;

use packwing_core::Values;

const SERVICE: &str = "packwing storages";

fn entry(id: &str, key: &str) -> io::Result<keyring::Entry> {
    keyring::Entry::new(SERVICE, &format!("{id}/{key}")).map_err(io::Error::other)
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
        Err(keyring::Error::NoEntry) => Ok(String::new()),
        Err(e) => Err(io::Error::other(e)),
    }
}

pub fn forget(id: &str, keys: &[&str]) {
    for key in keys {
        if let Ok(entry) = entry(id, key) {
            let _ = entry.delete_credential();
        }
    }
}
