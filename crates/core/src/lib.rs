//! What packwing does with archives, through hivepack: it reads what a file
//! holds, extracts it, checks that all of it reads, and makes ZIP, 7z and compressed tar
//! archives. Nothing here draws; the window shows what comes back.
//!
//! A file is opened through its layers (age, zstd, xz, gzip, bzip2, an RPM
//! package's lead and headers and the others hivepack-open finds), and what
//! is under them read: a tar, cpio or XAR archive (a macOS package) or an
//! ISO 9660 image as it streams, a ZIP or 7z archive or a UDF volume from a
//! file (taken out to a temporary one first when it is under layers), or
//! one file, compressed or encrypted on its own (`notes.txt.zst`).

mod create;
mod job;
mod joined;
mod listing;
mod spool;
mod volume;

use std::fmt;
use std::io;
use std::path::Path;
use std::sync::Arc;

use hivepack_age::{
    BoxedIdentity, LazyScryptIdentity, PassphraseFor, PassphrasePrompt, parse_identities_file,
};
use hivepack_open::Opener;

use crate::joined::Joined;

pub use create::{Kind, Level, Measured, NewArchive, add, can_add, create, measure};
pub use hivepack_extract::{Cancel, Progress, Summary};
pub use hivepack_open::{Container, Layer};
pub use job::{Control, extract, test};
pub use listing::{Child, Entry, Listing, list};

/// The largest scrypt work factor accepted: 2^20, 1 GiB and about a second.
/// age encrypts with 2^18; hivepack's default, 2^22, takes 4 GiB, which a
/// file from a stranger should not get to ask for.
pub const MAX_WORK_FACTOR: u8 = 20;

/// What [`Error::Locked`] needs for an archive's own password.
pub const PASSWORD: &str = "password";

/// What [`Error::Locked`] needs for an age file's passphrase: the type of
/// its recipient stanza.
pub const PASSPHRASE: &str = "scrypt";

/// What [`Error::Locked`] needs for a key file's own passphrase.
pub const KEY_PASSPHRASE: &str = "key passphrase";

/// What a passphrase prompt says when a key file's passphrase is not there,
/// told apart from other errors by it as [`NO_PASSPHRASE`] is.
const NO_KEY_PASSPHRASE: &str = "no key passphrase given";

/// What a passphrase prompt says when there is none to give: age's
/// "could not read passphrase" then names it, and it is told apart from
/// other errors of the header by it.
const NO_PASSPHRASE: &str = "no passphrase given";

/// What opens an encrypted archive. A file is opened with what is here; one
/// that needs more says so ([`Error::Locked`]), and the window asks for it
/// and opens it again.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Keys {
    /// The passphrase of an age layer encrypted with one.
    pub passphrase: Option<Vec<u8>>,
    /// The password of a ZIP or 7z archive's encrypted files, and of a 7z
    /// archive's encrypted names.
    pub password: Option<Vec<u8>>,
    /// What is in the key file an age layer was encrypted to: age's keys or
    /// an SSH key, maybe encrypted itself.
    pub key_file: Option<Vec<u8>>,
    /// The passphrase of that key file, if it has one.
    pub key_passphrase: Option<Vec<u8>>,
}

impl Keys {
    /// What `needs` (an [`Error::Locked`]'s) asks for that a person types: a
    /// passphrase, a password, or a key file; `None` for a kind of key
    /// hivepack does not read.
    pub fn asks_for(needs: &[String]) -> Option<Asked> {
        if needs.iter().any(|n| n == PASSWORD) {
            Some(Asked::Password)
        } else if needs.iter().any(|n| n == PASSPHRASE) {
            Some(Asked::Passphrase)
        } else if needs.iter().any(|n| n == KEY_PASSPHRASE) {
            Some(Asked::KeyPassphrase)
        } else if needs.iter().any(|n| n == "X25519" || n == "ssh-ed25519") {
            Some(Asked::KeyFile)
        } else {
            None
        }
    }

    /// These keys with `secret`, typed or read from a file for what was asked.
    pub fn with(&self, asked: Asked, secret: Vec<u8>) -> Keys {
        let mut keys = self.clone();
        match asked {
            Asked::Password => keys.password = Some(secret),
            Asked::Passphrase => keys.passphrase = Some(secret),
            Asked::KeyFile => {
                keys.key_file = Some(secret);
                keys.key_passphrase = None;
            }
            Asked::KeyPassphrase => keys.key_passphrase = Some(secret),
        }
        keys
    }
}

/// What a person is asked for, to open an encrypted archive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Asked {
    /// An age file's passphrase.
    Passphrase,
    /// A ZIP or 7z archive's password.
    Password,
    /// The file of the key an age file was encrypted to.
    KeyFile,
    /// That key file's own passphrase.
    KeyPassphrase,
}

/// Why `contents` is no key file that packwing can use, as the window says it.
pub fn check_key_file(contents: &[u8]) -> Result<(), String> {
    let prompt: PassphrasePrompt = Arc::new(|_| Err(NO_KEY_PASSPHRASE.to_owned()));
    parse_identities_file(contents, prompt).map(|_| ())
}

impl fmt::Debug for Keys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Keys")
            .field("passphrase", &self.passphrase.is_some())
            .field("password", &self.password.is_some())
            .field("key_file", &self.key_file.is_some())
            .field("key_passphrase", &self.key_passphrase.is_some())
            .finish()
    }
}

/// Why an archive did not open, extract or check, as the window says it.
#[derive(Debug)]
pub enum Error {
    /// Nothing packwing reads: no layer it knows, and no archive.
    NotAnArchive,
    /// Encrypted, and none of the keys given opens it. The types of its
    /// recipients say what it needs: `scrypt` is a passphrase, `X25519` or
    /// `ssh-ed25519` a key; `password` is an archive's own password (a 7z
    /// archive's names, or its files').
    Locked { needs: Vec<String> },
    /// The password given is not the archive's, or its data is damaged:
    /// 7z has nothing to tell the two apart with.
    WrongPassword,
    /// The passphrase given is not the age file's.
    WrongPassphrase,
    /// The passphrase given is not the key file's.
    WrongKeyPassphrase,
    /// Stopped by [`Cancel`].
    Cancelled,
    /// What hivepack said: the archive is damaged, refused, or could not be
    /// read or written.
    Failed(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotAnArchive => f.write_str("This file is not an archive packwing can open."),
            Error::Locked { needs } if needs.iter().any(|n| n == "scrypt") => {
                f.write_str("This file is encrypted with a passphrase.")
            }
            Error::Locked { needs } if needs.iter().any(|n| n == PASSWORD) => {
                f.write_str("This archive needs its password.")
            }
            Error::Locked { needs } if needs.iter().any(|n| n == KEY_PASSPHRASE) => {
                f.write_str("This key file needs its passphrase.")
            }
            Error::WrongPassword => {
                f.write_str("The password is wrong, or the archive is damaged.")
            }
            Error::WrongPassphrase => f.write_str("The passphrase is wrong."),
            Error::WrongKeyPassphrase => f.write_str("The key file’s passphrase is wrong."),
            Error::Locked { .. } => {
                f.write_str("This file is encrypted to a key packwing does not have.")
            }
            Error::Cancelled => f.write_str("Cancelled."),
            Error::Failed(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        // hivepack-open's errors travel inside io::Error on their way out of
        // a reader such as hivepack-extract's.
        if e.get_ref()
            .is_some_and(|inner| inner.is::<hivepack_open::Error>())
        {
            if let Some(inner) = e.into_inner()
                && let Ok(open) = inner.downcast::<hivepack_open::Error>()
            {
                return Error::from(*open);
            }
            return Error::Failed("the archive could not be read".into());
        }
        // A 7z or ZIP file's data, read with a password that is not its own.
        if let Some(&inner) = e
            .get_ref()
            .and_then(|inner| inner.downcast_ref::<hivepack_7z::Error>())
        {
            return Error::from(inner);
        }
        if let Some(&inner) = e
            .get_ref()
            .and_then(|inner| inner.downcast_ref::<hivepack_zip::Error>())
            && is_password_error(&inner)
        {
            return Error::from(inner);
        }
        Error::Failed(e.to_string())
    }
}

impl From<hivepack_open::Error> for Error {
    fn from(e: hivepack_open::Error) -> Error {
        match e {
            hivepack_open::Error::Age(hivepack_age::Error::NoMatch(m)) => Error::Locked {
                needs: m.stanza_types,
            },
            hivepack_open::Error::Age(hivepack_age::Error::Header(message)) => {
                age_header_error(message)
            }
            hivepack_open::Error::Io(e) => Error::Failed(e.to_string()),
            e => Error::Failed(e.to_string()),
        }
    }
}

/// An age header's error: the passphrase the prompt had none to give
/// ([`Error::Locked`]), or a wrong one, as age says them; anything else as
/// it is.
fn age_header_error(message: String) -> Error {
    // LazyScryptIdentity's words, as age's `cmd/age` has them.
    if message.starts_with("could not read passphrase") && message.ends_with(NO_PASSPHRASE) {
        return Error::Locked {
            needs: vec![PASSPHRASE.to_owned()],
        };
    }
    if message == "incorrect passphrase" {
        return Error::WrongPassphrase;
    }
    // A key file's own passphrase comes wrapped in what it was for.
    if message.contains(NO_KEY_PASSPHRASE) {
        return Error::Locked {
            needs: vec![KEY_PASSPHRASE.to_owned()],
        };
    }
    if message.contains("failed to decrypt SSH key file")
        || (message.contains("failed to decrypt identity file")
            && message.contains("incorrect passphrase"))
    {
        return Error::WrongKeyPassphrase;
    }
    Error::Failed(message)
}

impl From<hivepack_extract::Error> for Error {
    fn from(e: hivepack_extract::Error) -> Error {
        match e.kind {
            hivepack_extract::ErrorKind::Cancelled => Error::Cancelled,
            hivepack_extract::ErrorKind::Io(io) => Error::from(io),
            hivepack_extract::ErrorKind::Archive(zip) if is_password_error(&zip) => {
                Error::from(zip)
            }
            hivepack_extract::ErrorKind::SevenZip(
                inner @ (hivepack_7z::Error::PasswordRequired | hivepack_7z::Error::WrongPassword),
            ) => Error::from(inner),
            _ => Error::Failed(e.to_string()),
        }
    }
}

impl From<hivepack_7z::Error> for Error {
    fn from(e: hivepack_7z::Error) -> Error {
        match e {
            hivepack_7z::Error::PasswordRequired => Error::Locked {
                needs: vec![PASSWORD.to_owned()],
            },
            hivepack_7z::Error::WrongPassword => Error::WrongPassword,
            hivepack_7z::Error::NoArchive => Error::NotAnArchive,
            e => Error::Failed(e.to_string()),
        }
    }
}

impl From<hivepack_cpio::Error> for Error {
    fn from(e: hivepack_cpio::Error) -> Error {
        match e {
            // What hivepack-open said, under the archive's reader.
            hivepack_cpio::Error::Io(io) => Error::from(io),
            e => Error::Failed(e.to_string()),
        }
    }
}

impl From<hivepack_xar::Error> for Error {
    fn from(e: hivepack_xar::Error) -> Error {
        match e {
            hivepack_xar::Error::Io(io) => Error::from(io),
            e => Error::Failed(e.to_string()),
        }
    }
}

/// Whether a ZIP archive's error is about its password.
fn is_password_error(e: &hivepack_zip::Error) -> bool {
    matches!(
        e.code(),
        hivepack_zip::ErrorCode::NoPasswd | hivepack_zip::ErrorCode::WrongPasswd
    )
}

impl From<hivepack_zip::Error> for Error {
    fn from(e: hivepack_zip::Error) -> Error {
        match e.code() {
            hivepack_zip::ErrorCode::NoPasswd => Error::Locked {
                needs: vec![PASSWORD.to_owned()],
            },
            hivepack_zip::ErrorCode::WrongPasswd => Error::WrongPassword,
            _ => Error::Failed(e.to_string()),
        }
    }
}

impl From<hivepack_tar::Error> for Error {
    fn from(e: hivepack_tar::Error) -> Error {
        Error::Failed(e.to_string())
    }
}

impl From<hivepack_iso::Error> for Error {
    fn from(e: hivepack_iso::Error) -> Error {
        match e {
            // What hivepack-open said, under the image's reader.
            hivepack_iso::Error::Io(io) => Error::from(io),
            e => Error::Failed(e.to_string()),
        }
    }
}

impl From<hivepack_udf::Error> for Error {
    fn from(e: hivepack_udf::Error) -> Error {
        match e {
            hivepack_udf::Error::Io(io) => Error::from(io),
            e => Error::Failed(e.to_string()),
        }
    }
}

/// Whether the file at `path` came from the Internet, by the mark its
/// system put on it (the Mark of the Web, macOS's quarantine): what is
/// extracted from it keeps that mark.
pub fn came_from_internet(path: &Path) -> bool {
    hivepack_extract::Origin::of(path).is_ok_and(|o| o.is_some())
}

/// The file at `path`, to be opened through its layers as often as asked.
/// An age layer encrypted with a passphrase takes the one of `keys`; with
/// none, it says so, and opening fails with [`Error::Locked`], so that the
/// window can ask for one. The passphrase is asked for, and scrypt run,
/// only for a layer that needs it. So is a key file's own, the key file taken
/// first to [`check_key_file`].
fn opener<'a>(path: &'a Path, keys: &Keys) -> Opener<impl FnMut() -> io::Result<Joined> + use<'a>> {
    let (passphrase, key_passphrase) = (keys.passphrase.clone(), keys.key_passphrase.clone());
    let prompt: PassphrasePrompt = Arc::new(move |what| {
        let (secret, none) = match what {
            PassphraseFor::File => (&passphrase, NO_PASSPHRASE),
            _ => (&key_passphrase, NO_KEY_PASSPHRASE),
        };
        match secret {
            Some(p) if !p.is_empty() => Ok(p.clone()),
            _ => Err(none.to_owned()),
        }
    });
    let mut lazy = LazyScryptIdentity::new(prompt.clone());
    lazy.set_max_work_factor(MAX_WORK_FACTOR);
    let mut identities: Vec<BoxedIdentity> = vec![Box::new(lazy)];
    if let Some(found) = keys
        .key_file
        .as_deref()
        .and_then(|key| parse_identities_file(key, prompt).ok())
    {
        identities.extend(found);
    }
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    Opener::new(move || Joined::open(path))
        .name(&name)
        .identities(identities)
}

/// A 7z archive, opened, and whether its names are encrypted.
pub(crate) struct SevenZip<R> {
    pub(crate) archive: hivepack_7z::Archive<R>,
    pub(crate) headers_encrypted: bool,
}

/// The most a 7z archive's headers and a folder's decoders may take, in
/// bytes: a file from a stranger should not get to ask for more. The
/// headers of an archive of a million files take about 200 MiB; LZMA's
/// largest dictionary 7-Zip writes by default, at -mx9, is 64 MiB.
const SEVEN_ZIP_MEMORY: u64 = 1 << 30;

/// How many times 7z's AES key may be hashed, as a power of 2: 7-Zip writes
/// 19, and its own limit, 24, takes seconds for each try.
const SEVEN_ZIP_KEY_CYCLES: u32 = 22;

/// The 7z archive that `open` gives, opened with the password of `keys`:
/// its names read without it if they can be, as most archives keep them,
/// the password kept for its files; else, its names being encrypted too,
/// opened again with it ([`Error::Locked`] without one). `open` gives the
/// archive from its start each time it is called.
pub(crate) fn seven_zip<R: io::Read + io::Seek>(
    mut open: impl FnMut() -> io::Result<R>,
    keys: &Keys,
) -> Result<SevenZip<R>, Error> {
    let password = keys
        .password
        .as_deref()
        .map(|p| String::from_utf8_lossy(p).into_owned());
    let mut options = hivepack_7z::Options {
        memory_limit: SEVEN_ZIP_MEMORY,
        decoder_memory_limit: SEVEN_ZIP_MEMORY,
        password: None,
        max_key_cycles: SEVEN_ZIP_KEY_CYCLES,
    };
    match hivepack_7z::Archive::open_with(open()?, &options) {
        Ok(mut archive) => {
            if let Some(password) = &password {
                archive.set_password(password);
            }
            Ok(SevenZip {
                archive,
                headers_encrypted: false,
            })
        }
        Err(hivepack_7z::Error::PasswordRequired) => {
            let Some(password) = password else {
                return Err(Error::Locked {
                    needs: vec![PASSWORD.to_owned()],
                });
            };
            options.password = Some(password);
            match hivepack_7z::Archive::open_with(open()?, &options) {
                Ok(archive) => Ok(SevenZip {
                    archive,
                    headers_encrypted: true,
                }),
                // Headers that do not decode with the password: it is not
                // theirs, or they are damaged, which 7z cannot tell apart.
                Err(
                    hivepack_7z::Error::WrongPassword
                    | hivepack_7z::Error::Crc
                    | hivepack_7z::Error::Data
                    | hivepack_7z::Error::Archive,
                ) => Err(Error::WrongPassword),
                Err(e) => Err(e.into()),
            }
        }
        Err(e) => Err(e.into()),
    }
}

/// `name` without the extensions of `layers`: `notes.txt` for
/// `notes.txt.zst.age`. A name with too few extensions is kept whole.
fn without_layers(name: &str, layers: &[Layer]) -> String {
    let mut stem = name;
    for _ in layers {
        match stem.rsplit_once('.') {
            Some((s, _)) if !s.is_empty() => stem = s,
            _ => return name.to_owned(),
        }
    }
    stem.to_owned()
}
