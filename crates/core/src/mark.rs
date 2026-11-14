//! The mark of where a file came from, which the system checks before running it. A
//! file that arrives through packwing gets one, or copying would be a way around the
//! check: what was marked stays marked, and what comes from a storage across a network
//! counts as from the Internet, as a browser's download would.

use std::path::{Path, PathBuf};

use hivepack::extract::Origin;

use crate::{Error, Source, Storage};

/// The mark of a file from a storage across a network, on this system.
#[cfg(windows)]
fn of_a_storage() -> Option<Origin> {
    Some(Origin::ZoneIdentifier(
        b"[ZoneTransfer]\r\nZoneId=3\r\n".to_vec(),
    ))
}

/// The mark of a file from a storage across a network, on this system: quarantined,
/// by packwing, now. Gatekeeper reads the flags; 0081 is what a download gets.
#[cfg(target_os = "macos")]
fn of_a_storage() -> Option<Origin> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    Some(Origin::Quarantine(
        format!("0081;{now:08x};packwing;").into_bytes(),
    ))
}

/// Other systems have no mark.
#[cfg(not(any(windows, target_os = "macos")))]
fn of_a_storage() -> Option<Origin> {
    None
}

/// The mark for what comes from the file at `path`, without the addresses a download
/// left in it; or, with none, from a storage across a network.
fn origin(path: Option<&Path>) -> Result<Option<Origin>, Error> {
    match path {
        Some(path) => Ok(Origin::of(path)?.map(Origin::without_addresses)),
        None if packwing_policy::current().marks_storage_files() => Ok(of_a_storage()),
        None => Ok(None),
    }
}

/// In the App Sandbox (the Mac App Store's packwing), macOS marks every file packwing
/// makes as quarantined, by packwing, and lets no one write that mark but itself: there
/// packwing writes none, and what it makes is marked all the same, whether the user
/// would keep the mark or not.
fn the_system_marks() -> bool {
    cfg!(target_os = "macos") && std::env::var_os("APP_SANDBOX_CONTAINER_ID").is_some()
}

/// The mark to put on what is extracted from `source`, as the user chose and the
/// organization allows.
pub(crate) fn for_extracting(source: &Source, keep: bool) -> Result<Option<Origin>, Error> {
    if the_system_marks() || !packwing_policy::current().keeps_mark(keep) {
        return Ok(None);
    }
    origin(source.path())
}

/// Whether what is extracted from `source` will carry a mark that it came from the
/// Internet, had the user not turned that off.
pub fn came_from_internet(source: &Source) -> bool {
    origin(source.path()).is_ok_and(|o| o.is_some())
}

/// Puts on each of `files`, a new archive made of what `source` holds, the mark
/// `source` has, as the user chose and the organization allows: what is in it came
/// from where `source` did.
pub(crate) fn on_new_archive(source: &Source, files: &[PathBuf], keep: bool) -> Result<(), Error> {
    let Some(origin) = for_extracting(source, keep)? else {
        return Ok(());
    };
    for file in files {
        write(&origin, file)?;
    }
    Ok(())
}

/// Puts on the file at `to`, just copied here from `path` of `from`, the mark it should
/// have. A copy that cannot carry it does not stay, as extracting does not.
pub(crate) fn on_a_copy(
    from: &dyn Storage,
    path: &[String],
    to: &Path,
    keep: bool,
) -> Result<(), Error> {
    if the_system_marks() || !packwing_policy::current().keeps_mark(keep) {
        return Ok(());
    }
    let Some(origin) = origin(from.local_path(path).as_deref())? else {
        return Ok(());
    };
    write(&origin, to).map_err(|e| {
        let _ = std::fs::remove_file(to);
        Error::from(e)
    })
}

#[cfg(windows)]
fn write(origin: &Origin, to: &Path) -> std::io::Result<()> {
    let Origin::ZoneIdentifier(value) = origin else {
        return Ok(());
    };
    let mut stream = std::path::absolute(to)?.into_os_string();
    stream.push(":Zone.Identifier");
    std::fs::write(stream, value)
}

#[cfg(target_os = "macos")]
fn write(origin: &Origin, to: &Path) -> std::io::Result<()> {
    let Origin::Quarantine(value) = origin else {
        return Ok(());
    };
    xattr::set(to, "com.apple.quarantine", value)
}

#[cfg(not(any(windows, target_os = "macos")))]
fn write(_: &Origin, _: &Path) -> std::io::Result<()> {
    Ok(())
}
