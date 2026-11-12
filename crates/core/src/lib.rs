//! What packwing does with archives, through hivepack: it reads what a file
//! holds, extracts it, checks that all of it reads, and makes ZIP, 7z and compressed tar
//! archives. Nothing here draws; the window shows what comes back.
//!
//! A file is opened through its layers (age, zstd, xz, gzip, bzip2, an RPM
//! package's lead and headers, a Debian package's members before its
//! `data.tar`, and the others hivepack::open finds), and what is under them
//! read: a tar, cpio or XAR archive (a macOS package) or an ISO 9660 image (a
//! type 1 AppImage too) as it streams, a ZIP, RAR or 7z archive, a UDF volume
//! or a SquashFS image (a snap, a type 2 AppImage's) from a file (taken out to
//! a temporary one first when it is under layers), or one file, compressed or
//! encrypted on its own (`notes.txt.zst`).

pub mod audit;
mod checksum;
mod convert;
mod create;
mod filter;
mod job;
mod joined;
mod journal;
mod listing;
mod mark;
pub mod memory;
mod passphrase;
mod rar;
mod sign;
mod source;
mod split;
mod spool;
mod storage;
mod transfer;
mod unsupported;
mod volume;

use packwing_i18n::{tr, tr_n};
use std::fmt;
use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

pub use hivepack::Threads;
use hivepack::age::{
    BoxedIdentity, LazyScryptIdentity, PassphraseFor, PassphrasePrompt, parse_identities_file,
};
use hivepack::open::Opener;

use crate::source::SourceReader;

pub use checksum::{
    Algorithm, algorithms_of, checksum_file_name, checksum_in, checksum_line, checksums,
    checksums_of, save_checksum,
};
pub use convert::{Converted, convert};
pub use create::{
    AddOptions, Blocks, COMPRESSED_EXTENSIONS, FRAME_SIZE, Frames, Keep, Kind, Level, Measured,
    NewArchive, OnExisting, Route, SEVERAL_FRAMES_FROM, Times, add, add_with,
    age_key_is_post_quantum, age_key_problem, age_keys_in, can_add, can_rename, create, create_on,
    delete, free_archive_path, measure, memory_needed, memory_warning, refused_by, rename, route,
    within_memory,
};
pub use filter::Filter;
pub use hivepack::extract::{
    Cancel, OnDamaged, OnLinksOutside, OnUnnamable, Owners, Progress, Summary,
};
pub use hivepack::open::{Container, Layer};
pub use job::{
    BadName, Control, Damage, DamagedFn, ExtractOptions, GoneFn, LinksOutsideFn, OutsideLink,
    Renaming, UnnamableFn, clean_opened_files, damages, extract, extract_selected, extract_to_open,
    extract_with, image_fits, image_size, left_out, preview_entry, read_entry, renamings, test,
};
pub use journal::{clean_left_extractions, keep_journal_in};
pub use listing::{Child, Entry, Found, Link, Listen, Listing, list, list_as_read};
pub use mark::came_from_internet;
pub use passphrase::suggest_passphrase;
pub use sign::{
    KeyRefused, PublicKey, Signed, SigningKey, check_signature, has_signature, is_part_name,
    is_signature_name, look_at_signature, make_signing_key, read_public_key, read_signing_key,
    sign_file, signature_name,
};
pub use source::{Source, Stored};
pub use split::{join_parts, parts_of, split_file};
pub use storage::{
    Counted, Field, Item, Lister, LocalFolder, Storage, StorageKind, StoredFile, Values, bad_name,
    by_name, count, folded_query, name_holds,
};
pub use transfer::{Transfer, Transferred, transfer};
pub use unsupported::Unsupported;

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
        } else if needs
            .iter()
            .any(|n| matches!(n.as_str(), "X25519" | "mlkem768x25519" | "ssh-ed25519"))
        {
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
    /// A kind of archive packwing knows of, but does not read yet.
    Unsupported(Unsupported),
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
            Error::NotAnArchive => {
                f.write_str(&tr!("This file is not an archive packwing can open."))
            }
            Error::Unsupported(what) => f.write_str(&match what {
                Unsupported::Ar => tr!(
                    "This is an ar archive, as static libraries are. packwing can’t open those yet."
                ),
                Unsupported::Dmg => {
                    tr!("This is a macOS disk image (.dmg). packwing can’t open those yet.")
                }
            }),
            Error::Locked { needs } if needs.iter().any(|n| n == "scrypt") => {
                f.write_str(&tr!("This file is encrypted with a passphrase."))
            }
            Error::Locked { needs } if needs.iter().any(|n| n == PASSWORD) => {
                f.write_str(&tr!("This archive needs its password."))
            }
            Error::Locked { needs } if needs.iter().any(|n| n == KEY_PASSPHRASE) => {
                f.write_str(&tr!("This key file needs its passphrase."))
            }
            Error::WrongPassword => {
                f.write_str(&tr!("The password is wrong, or the archive is damaged."))
            }
            Error::WrongPassphrase => f.write_str(&tr!("The passphrase is wrong.")),
            Error::WrongKeyPassphrase => f.write_str(&tr!("The key file’s passphrase is wrong.")),
            Error::Locked { .. } => f.write_str(&tr!(
                "This file is encrypted to a key packwing does not have."
            )),
            Error::Cancelled => f.write_str(&tr!("Cancelled.")),
            Error::Failed(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        // hivepack::open's errors travel inside io::Error on their way out of
        // a reader such as hivepack::extract's.
        if e.get_ref()
            .is_some_and(|inner| inner.is::<hivepack::open::Error>())
        {
            if let Some(inner) = e.into_inner()
                && let Ok(open) = inner.downcast::<hivepack::open::Error>()
            {
                return Error::from(*open);
            }
            return Error::Failed(tr!("the archive could not be read"));
        }
        // A 7z or ZIP file's data, read with a password that is not its own.
        if let Some(&inner) = e
            .get_ref()
            .and_then(|inner| inner.downcast_ref::<hivepack::sevenz::Error>())
        {
            return Error::from(inner);
        }
        if let Some(&inner) = e
            .get_ref()
            .and_then(|inner| inner.downcast_ref::<hivepack::zip::Error>())
            && is_password_error(&inner)
        {
            return Error::from(inner);
        }
        Error::Failed(e.to_string())
    }
}

impl From<hivepack::open::Error> for Error {
    fn from(e: hivepack::open::Error) -> Error {
        match e {
            hivepack::open::Error::Age(hivepack::age::Error::NoMatch(m)) => Error::Locked {
                needs: m.stanza_types,
            },
            hivepack::open::Error::Age(hivepack::age::Error::Header(message)) => {
                age_header_error(message)
            }
            hivepack::open::Error::Io(e) => Error::Failed(e.to_string()),
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

impl From<hivepack::rar::Error> for Error {
    fn from(e: hivepack::rar::Error) -> Error {
        match e {
            hivepack::rar::Error::MissingPassword { .. } => Error::Locked {
                needs: vec![PASSWORD.to_owned()],
            },
            hivepack::rar::Error::Io(io) => Error::from(io),
            e => Error::Failed(e.to_string()),
        }
    }
}

impl From<hivepack::extract::Error> for Error {
    fn from(e: hivepack::extract::Error) -> Error {
        match e.kind {
            hivepack::extract::ErrorKind::Cancelled => Error::Cancelled,
            hivepack::extract::ErrorKind::Name(why) => {
                let name = e
                    .name
                    .as_deref()
                    .map_or_else(String::new, |raw| String::from_utf8_lossy(raw).into_owned());
                Error::Failed(tr!(
                    "This can’t be extracted, as a file’s name can’t be used here: “{name}” ({why}).",
                    name = name,
                    why = why
                ))
            }
            hivepack::extract::ErrorKind::Rar(inner) => Error::from(inner),
            hivepack::extract::ErrorKind::RarEntry(
                hivepack::extract::RarProblem::MissingPassword,
            ) => Error::Locked {
                needs: vec![PASSWORD.to_owned()],
            },
            hivepack::extract::ErrorKind::RarEntry(hivepack::extract::RarProblem::BadPassword) => {
                Error::WrongPassword
            }
            hivepack::extract::ErrorKind::Io(io) => Error::from(io),
            hivepack::extract::ErrorKind::Archive(zip) if is_password_error(&zip) => {
                Error::from(zip)
            }
            hivepack::extract::ErrorKind::SevenZip(
                inner @ (hivepack::sevenz::Error::PasswordRequired
                | hivepack::sevenz::Error::WrongPassword),
            ) => Error::from(inner),
            _ => Error::Failed(e.to_string()),
        }
    }
}

impl From<hivepack::sevenz::Error> for Error {
    fn from(e: hivepack::sevenz::Error) -> Error {
        match e {
            hivepack::sevenz::Error::PasswordRequired => Error::Locked {
                needs: vec![PASSWORD.to_owned()],
            },
            hivepack::sevenz::Error::WrongPassword => Error::WrongPassword,
            hivepack::sevenz::Error::NoArchive => Error::NotAnArchive,
            e => Error::Failed(e.to_string()),
        }
    }
}

impl From<hivepack::cpio::Error> for Error {
    fn from(e: hivepack::cpio::Error) -> Error {
        match e {
            // What hivepack::open said, under the archive's reader.
            hivepack::cpio::Error::Io(io) => Error::from(io),
            e => Error::Failed(e.to_string()),
        }
    }
}

impl From<hivepack::xar::Error> for Error {
    fn from(e: hivepack::xar::Error) -> Error {
        match e {
            hivepack::xar::Error::Io(io) => Error::from(io),
            e => Error::Failed(e.to_string()),
        }
    }
}

/// Whether a ZIP archive's error is about its password.
fn is_password_error(e: &hivepack::zip::Error) -> bool {
    matches!(
        e.code(),
        hivepack::zip::ErrorCode::NoPasswd | hivepack::zip::ErrorCode::WrongPasswd
    )
}

impl From<hivepack::zip::Error> for Error {
    fn from(e: hivepack::zip::Error) -> Error {
        match e.code() {
            hivepack::zip::ErrorCode::NoPasswd => Error::Locked {
                needs: vec![PASSWORD.to_owned()],
            },
            hivepack::zip::ErrorCode::WrongPasswd => Error::WrongPassword,
            _ => Error::Failed(e.to_string()),
        }
    }
}

impl From<hivepack::tar::Error> for Error {
    fn from(e: hivepack::tar::Error) -> Error {
        Error::Failed(e.to_string())
    }
}

impl From<hivepack::iso::Error> for Error {
    fn from(e: hivepack::iso::Error) -> Error {
        match e {
            // What hivepack::open said, under the image's reader.
            hivepack::iso::Error::Io(io) => Error::from(io),
            e => Error::Failed(e.to_string()),
        }
    }
}

impl From<hivepack::squashfs::Error> for Error {
    fn from(e: hivepack::squashfs::Error) -> Error {
        match e {
            hivepack::squashfs::Error::Io(io) => Error::from(io),
            e => Error::Failed(e.to_string()),
        }
    }
}

impl From<hivepack::appimage::Error> for Error {
    fn from(e: hivepack::appimage::Error) -> Error {
        match e {
            hivepack::appimage::Error::Io(io) => Error::from(io),
            hivepack::appimage::Error::Squashfs(hivepack::squashfs::Error::Io(io)) => {
                Error::from(io)
            }
            e => Error::Failed(e.to_string()),
        }
    }
}

impl From<hivepack::udf::Error> for Error {
    fn from(e: hivepack::udf::Error) -> Error {
        match e {
            hivepack::udf::Error::Io(io) => Error::from(io),
            e => Error::Failed(e.to_string()),
        }
    }
}

/// How many threads compressing and decompressing may take, as
/// [`use_threads`] was last told: 0 for as many as the machine has.
static THREADS: AtomicU32 = AtomicU32::new(0);

/// Lets compressing and decompressing take up to `count` threads from now on:
/// 0 for as many as the machine has, which is how it starts. Where a format
/// can be read or written in pieces (xz's blocks, bzip2's streams, 7z's LZMA2
/// blocks, zstd's as it is written, a ZIP archive's files as they are
/// extracted), each piece goes to a thread; what is made is the same however
/// many there are.
pub fn use_threads(count: u32) {
    THREADS.store(count, Ordering::Relaxed);
}

/// The threads [`use_threads`] allows.
pub fn threads() -> Threads {
    Threads::of(THREADS.load(Ordering::Relaxed))
}

/// The file at `path`, to be opened through its layers as often as asked.
/// An age layer encrypted with a passphrase takes the one of `keys`; with
/// none, it says so, and opening fails with [`Error::Locked`], so that the
/// window can ask for one. The passphrase is asked for, and scrypt run,
/// only for a layer that needs it. So is a key file's own, the key file taken
/// first to [`check_key_file`].
fn opener<'a>(
    source: &'a Source,
    keys: &Keys,
) -> Opener<impl FnMut() -> io::Result<SourceReader> + use<'a>> {
    opener_of(&source.name(), keys, move || source.reader())
}

/// [`opener`], of what `open` reads: the file named `name`, from its start.
pub(crate) fn opener_of<F, R>(name: &str, keys: &Keys, open: F) -> Opener<F>
where
    F: FnMut() -> io::Result<R>,
    R: io::Read + Send,
{
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
    Opener::new(open)
        .name(name)
        .identities(identities)
        .threads(threads())
        // bzip2's threads, about 20 MiB each, as many as the memory there
        // is takes; zstd's frames in flight, as many as it takes too, within
        // hivepack's 1 GiB; xz's Blocks in threads, each holding its
        // dictionary and its Block in and out, within the budget, and never
        // less than hivepack's 1 GiB, which a file of `xz -9` takes well
        // within on one thread. liblzma's limit for the threads is never
        // more than its limit to stop at, so the two are raised together:
        // 2.3 GB of `.tar.xz` from `xz -T0` decode on 32 threads in 0.82 s
        // where 1 GiB took 0.99 s, in 1.9 GB as `xz` takes.
        .limits({
            let mut limits = hivepack::open::Limits::default();
            let budget = memory::budget();
            limits.bzip2_memlimit = budget;
            limits.zstd_memlimit = limits.zstd_memlimit.min(budget);
            limits.xz_memlimit = limits.xz_memlimit.max(budget);
            limits
        })
}

/// A 7z archive, opened, and whether its names are encrypted.
pub(crate) struct SevenZip<R> {
    pub(crate) archive: hivepack::sevenz::Archive<R>,
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
    let mut options = hivepack::sevenz::Options {
        memory_limit: SEVEN_ZIP_MEMORY,
        decoder_memory_limit: SEVEN_ZIP_MEMORY,
        password: None,
        max_key_cycles: SEVEN_ZIP_KEY_CYCLES,
    };
    match hivepack::sevenz::Archive::open_with(open()?, &options) {
        Ok(mut archive) => {
            if let Some(password) = &password {
                archive.set_password(password);
            }
            archive.set_threads(threads());
            Ok(SevenZip {
                archive,
                headers_encrypted: false,
            })
        }
        Err(hivepack::sevenz::Error::PasswordRequired) => {
            let Some(password) = password else {
                return Err(Error::Locked {
                    needs: vec![PASSWORD.to_owned()],
                });
            };
            options.password = Some(password);
            match hivepack::sevenz::Archive::open_with(open()?, &options) {
                Ok(archive) => Ok(SevenZip {
                    archive,
                    headers_encrypted: true,
                }),
                // Headers that do not decode with the password: it is not
                // theirs, or they are damaged, which 7z cannot tell apart.
                Err(
                    hivepack::sevenz::Error::WrongPassword
                    | hivepack::sevenz::Error::Crc
                    | hivepack::sevenz::Error::Data
                    | hivepack::sevenz::Error::Archive,
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

// hivepack's `unsafe-fast` is on in all that has it, as the workspace asks.
const _: () = assert!(hivepack::deflate::UNSAFE_FAST);
const _: () = assert!(hivepack::lz4::UNSAFE_FAST);
const _: () = assert!(hivepack::zstd::UNSAFE_FAST);

/// `name` without its archive extensions and a part number: what it unpacks
/// to (`Báo cáo.part1.rar` → `Báo cáo`, `backup.tar.zst.age` → `backup`).
pub fn stem(name: &str) -> &str {
    const EXTENSIONS: [&str; 41] = [
        ".age",
        ".zst",
        ".gz",
        ".xz",
        ".bz2",
        ".lz4",
        ".lz",
        ".lzma",
        ".tgz",
        ".tbz2",
        ".tbz",
        ".txz",
        ".tzst",
        ".tlz",
        ".tlz4",
        ".tar",
        ".7z",
        ".zip",
        ".rar",
        ".iso",
        ".cpio",
        ".rpm",
        ".xar",
        ".pkg",
        ".jar",
        ".war",
        ".ear",
        ".apk",
        ".xpi",
        ".epub",
        ".cbz",
        ".cbr",
        ".cb7",
        ".cbt",
        ".deb",
        ".udeb",
        ".snap",
        ".sqfs",
        ".squashfs",
        ".AppImage",
        ".appimage",
    ];
    let mut s = name;
    while let Some(rest) = EXTENSIONS
        .iter()
        .find_map(|ext| s.strip_suffix(ext).filter(|r| !r.is_empty()))
    {
        s = rest;
    }
    if let Some((rest, part)) = s.rsplit_once(".part")
        && !part.is_empty()
        && part.bytes().all(|b| b.is_ascii_digit())
    {
        s = rest;
    }
    s
}

/// A size as people read it: three figures at most, no trailing zeros, in SI
/// units as macOS shows them (`880 KB`, `6.4 MB`, `412 MB`, `1.24 GB`).
pub fn size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["bytes", "KB", "MB", "GB", "TB"];
    if bytes < 1000 {
        return tr_n!(bytes, "{n} byte", "{n} bytes");
    }
    let mut v = bytes as f64;
    let mut unit = 0;
    while v >= 999.5 && unit < UNITS.len() - 1 {
        v /= 1000.;
        unit += 1;
    }
    let text = if unit == 1 || v >= 99.95 {
        format!("{v:.0}")
    } else if unit == 2 || v >= 9.995 {
        format!("{v:.1}")
    } else {
        format!("{v:.2}")
    };
    let text = match text.split_once('.') {
        Some(_) => text.trim_end_matches('0').trim_end_matches('.'),
        None => &text,
    };
    format!("{text} {}", UNITS[unit])
}
