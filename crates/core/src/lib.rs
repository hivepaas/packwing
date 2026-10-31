//! What packwing does with archives, through hivepack: it reads what a file
//! holds, extracts it, and checks that all of it reads. Nothing here draws;
//! the window shows what comes back.
//!
//! A file is opened through its layers (age, zstd, xz, gzip, bzip2, an RPM
//! package's lead and headers and the others hivepack-open finds), and what
//! is under them read: a tar, cpio or XAR archive (a macOS package) or an
//! ISO 9660 image as it streams, a ZIP or 7z archive or a UDF volume from a
//! file (taken out to a temporary one first when it is under layers), or
//! one file, compressed or encrypted on its own (`notes.txt.zst`).

mod job;
mod listing;
mod spool;
mod volume;

use std::fmt;
use std::fs::File;
use std::io;
use std::path::Path;

use hivepack_age::{BoxedIdentity, LazyScryptIdentity, PassphrasePrompt};
use hivepack_open::Opener;

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

/// What opens an encrypted archive.
#[derive(Clone, Default)]
pub struct Keys {
    /// Asked for a passphrase when an age layer turns out to need one.
    pub passphrase: Option<PassphrasePrompt>,
    /// The password of a ZIP or 7z archive's encrypted files, and of a 7z
    /// archive's encrypted names.
    pub password: Option<Vec<u8>>,
}

impl fmt::Debug for Keys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Keys")
            .field("passphrase", &self.passphrase.is_some())
            .field("password", &self.password.is_some())
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
            Error::WrongPassword => {
                f.write_str("The password is wrong, or the archive is damaged.")
            }
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
        // A 7z file's data, read with a password that is not its own.
        if let Some(&inner) = e
            .get_ref()
            .and_then(|inner| inner.downcast_ref::<hivepack_7z::Error>())
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
            hivepack_open::Error::Io(e) => Error::Failed(e.to_string()),
            e => Error::Failed(e.to_string()),
        }
    }
}

impl From<hivepack_extract::Error> for Error {
    fn from(e: hivepack_extract::Error) -> Error {
        match e.kind {
            hivepack_extract::ErrorKind::Cancelled => Error::Cancelled,
            hivepack_extract::ErrorKind::Io(io) => Error::from(io),
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

impl From<hivepack_zip::Error> for Error {
    fn from(e: hivepack_zip::Error) -> Error {
        Error::Failed(e.to_string())
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
fn opener<'a>(path: &'a Path, keys: &Keys) -> Opener<impl FnMut() -> io::Result<File> + use<'a>> {
    let mut identities: Vec<BoxedIdentity> = Vec::new();
    if let Some(prompt) = &keys.passphrase {
        let mut lazy = LazyScryptIdentity::new(prompt.clone());
        lazy.set_max_work_factor(MAX_WORK_FACTOR);
        identities.push(Box::new(lazy));
    }
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    Opener::new(move || File::open(path))
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
