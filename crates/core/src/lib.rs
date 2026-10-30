//! What packwing does with archives, through hivepack: it reads what a file
//! holds, extracts it, and checks that all of it reads. Nothing here draws;
//! the window shows what comes back.
//!
//! A file is opened through its layers (age, zstd, xz, gzip, bzip2 and the
//! others hivepack-open finds), and what is under them read: a tar archive
//! as it streams, a ZIP archive from a file (taken out to a temporary one
//! first when it is under layers), or one file, compressed or encrypted on
//! its own (`notes.txt.zst`).

mod job;
mod listing;
mod spool;

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

/// What opens an encrypted archive.
#[derive(Clone, Default)]
pub struct Keys {
    /// Asked for a passphrase when an age layer turns out to need one.
    pub passphrase: Option<PassphrasePrompt>,
    /// The password of a ZIP archive's encrypted files.
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
    /// `ssh-ed25519` a key.
    Locked { needs: Vec<String> },
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
            _ => Error::Failed(e.to_string()),
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
