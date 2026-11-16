//! What the iOS and Android apps call: an archive opened and what it holds,
//! folder by folder; a file of it read, to preview it; its files extracted;
//! new ZIP and 7z archives made, and changed; an archive's own checksums and
//! signature; and storages across the network, browsed, their archives opened
//! where they are. The work is packwing-core's, as on the
//! desktop.
//!
//! Every call that reads or writes a file blocks until it is done, so the app
//! makes them off its main thread; a [`Job`] tells how far one is, and stops
//! it. Swift and Kotlin are both generated from this crate by UniFFI
//! (`scripts/build_ios.sh`, `scripts/build_android.sh`), so one API serves
//! both; names here are what each language sees, in its own case.
//!
//! What only one system has goes behind its own `#[cfg(target_os)]`, and the
//! other's bindings never see it: the bindings are read from the library
//! built for that system. [`Document`] is the one such thing so far, for
//! Android's document picker, which hands out no paths.

uniffi::setup_scaffolding!();

mod age;
mod checksum;
mod edit;
mod i18n;
mod signature;
mod signing;
mod ssh;
mod storage;

/// Android's document picker hands out no paths: see [`Document`].
#[cfg(target_os = "android")]
mod document;

#[cfg(target_os = "android")]
pub use document::Document;

/// Android has no keychain the keyring crate can reach: see [`Vault`].
#[cfg(target_os = "android")]
mod vault;

#[cfg(target_os = "android")]
pub use vault::{Vault, keep_secrets_in};

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use packwing_core::{
    Asked, COMPRESSED_EXTENSIONS, Cancel, Child, Control, Error, ExtractOptions, Keys, Kind, Level,
    Listen, Listing, NewArchive, Progress, Source, Summary,
};

pub use age::{AgeKeysFile, age_key_is_post_quantum, age_key_problem, age_keys_in, check_key_file};
pub use checksum::{
    Checksum, ChecksumKind, ChecksumKindRow, Compared, EntryChecksum, EntryChecksums, FileVerdict,
    FoundBySum, PastedCompared, checksum_kinds, checksum_list, checksum_list_name,
    compare_checksum, compare_pasted,
};
pub use edit::OnExisting;
pub use i18n::LanguageRow;
pub use signature::{KeyRead, SignatureState, read_key};
pub use signing::{
    SigningKeyRow, export_signing_key, import_signing_key, make_signing_key, may_sign,
    remove_signing_key, signing_keys,
};
pub use ssh::{
    SshKey, SshQuestions, TypedSecret, ask_ssh_with, import_ssh_key, remove_ssh_key, ssh_keys,
};
pub use storage::{
    RemoteStorage, StorageField, StorageKindRow, StoredItem, Transferred, describe_storage,
    forget_storage_secrets, keep_storage_secrets, kept_storage_secrets, storage_kinds,
    transfer_items,
};

/// Why a call did not do what it was asked, as the app says it: `reason` is
/// in the user's language already. It is not called `message`, which Kotlin's
/// own `Throwable` has: an exception of its own would not compile beside it.
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum MobileError {
    /// Not an archive packwing opens, or a kind it does not read yet.
    #[error("{reason}")]
    NotAnArchive { reason: String },
    /// Encrypted, and nothing given opens it: `asks` says what does, a
    /// password (ZIP, 7z, RAR), an age file's passphrase, the key file it was
    /// encrypted to, or that key file's passphrase. `passphrase` is whether it
    /// is the age file's passphrase. A key file asked again, one given, was not
    /// the key; `reason` says so.
    #[error("{reason}")]
    Locked {
        reason: String,
        passphrase: bool,
        asks: Asks,
    },
    /// The password or passphrase given is not its own, or its data is damaged.
    #[error("{reason}")]
    WrongPassword { reason: String },
    /// Stopped by [`Job::cancel`].
    #[error("{reason}")]
    Cancelled { reason: String },
    #[error("{reason}")]
    Failed { reason: String },
}

impl From<Error> for MobileError {
    fn from(e: Error) -> MobileError {
        let reason = e.to_string();
        match e {
            Error::NotAnArchive | Error::Unsupported(_) => MobileError::NotAnArchive { reason },
            Error::Locked { needs } => match Keys::asks_for(&needs) {
                Some(asked) => MobileError::Locked {
                    reason,
                    passphrase: asked == Asked::Passphrase,
                    asks: asked.into(),
                },
                // A kind of key packwing does not read.
                None => MobileError::Failed { reason },
            },
            Error::WrongPassword | Error::WrongPassphrase | Error::WrongKeyPassphrase => {
                MobileError::WrongPassword { reason }
            }
            Error::Cancelled => MobileError::Cancelled { reason },
            Error::Failed(_) => MobileError::Failed { reason },
        }
    }
}

/// Gets packwing ready, once, as the app starts: speaks `language` (a tag;
/// none for the system's), and keeps in `support_folder` (the app's
/// Application Support) the journal of extractions, so that what one left
/// when the app was ended is cleaned up the next time.
#[uniffi::export]
pub fn start(support_folder: String, language: Option<String>) {
    packwing_i18n::set_language(language.as_deref());
    packwing_core::clean_opened_files();
    packwing_core::keep_journal_in(PathBuf::from(&support_folder));
    // SFTP's own known_hosts and kept keys: a phone has no `~/.ssh` of the user's.
    packwing_remote::keep_in(PathBuf::from(support_folder));
    std::thread::spawn(packwing_core::clean_left_extractions);
}

/// What opens an encrypted archive: its password (ZIP, 7z, RAR), and the
/// passphrase of an age file, or what is in the key file it was encrypted to
/// (age's keys, or an SSH key) and that file's own passphrase.
#[derive(Clone, PartialEq, Eq, uniffi::Record, Default)]
pub struct Secrets {
    #[uniffi(default = None)]
    pub password: Option<String>,
    #[uniffi(default = None)]
    pub passphrase: Option<String>,
    #[uniffi(default = None)]
    pub key_file: Option<Vec<u8>>,
    #[uniffi(default = None)]
    pub key_passphrase: Option<String>,
}

impl std::fmt::Debug for Secrets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Secrets")
            .field("password", &self.password.is_some())
            .field("passphrase", &self.passphrase.is_some())
            .field("key_file", &self.key_file.is_some())
            .field("key_passphrase", &self.key_passphrase.is_some())
            .finish()
    }
}

impl Secrets {
    fn keys(&self) -> Keys {
        Keys {
            password: self.password.as_ref().map(|p| p.as_bytes().to_vec()),
            passphrase: self.passphrase.as_ref().map(|p| p.as_bytes().to_vec()),
            key_file: self.key_file.clone(),
            key_passphrase: self.key_passphrase.as_ref().map(|p| p.as_bytes().to_vec()),
        }
    }
}

/// What a locked archive asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum Asks {
    Password,
    Passphrase,
    /// The key file an age file was encrypted to.
    KeyFile,
    /// That key file's own passphrase.
    KeyPassphrase,
}

impl From<Asked> for Asks {
    fn from(asked: Asked) -> Asks {
        match asked {
            Asked::Password => Asks::Password,
            Asked::Passphrase => Asks::Passphrase,
            Asked::KeyFile => Asks::KeyFile,
            Asked::KeyPassphrase => Asks::KeyPassphrase,
        }
    }
}

/// How far a job is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, uniffi::Record)]
pub struct JobProgress {
    /// Files, folders and links written.
    pub entries: u64,
    pub total_entries: u64,
    pub bytes: u64,
    pub total_bytes: u64,
}

impl From<&Progress> for JobProgress {
    fn from(p: &Progress) -> JobProgress {
        JobProgress {
            entries: p.entries,
            total_entries: p.total_entries,
            bytes: p.bytes,
            total_bytes: p.total_bytes,
        }
    }
}

/// Told how far a job is, ten times a second at most, and when it is through.
/// Called off the main thread.
#[uniffi::export(with_foreign)]
pub trait JobListener: Send + Sync {
    fn progress(&self, progress: JobProgress);
}

/// How often a job's listener is told, at most.
const TELL_EVERY: Duration = Duration::from_millis(100);

/// One call's work: what it tells of how far it is, and what stops it. A job
/// is for one call.
#[derive(uniffi::Object)]
pub struct Job {
    cancel: Cancel,
    listener: Option<Arc<dyn JobListener>>,
}

#[uniffi::export]
impl Job {
    /// A job telling `listener` how far it is.
    #[uniffi::constructor]
    pub fn new(listener: Arc<dyn JobListener>) -> Arc<Job> {
        Arc::new(Job {
            cancel: Cancel::new(),
            listener: Some(listener),
        })
    }

    /// A job that tells nothing, only stops.
    #[uniffi::constructor]
    pub fn quiet() -> Arc<Job> {
        Arc::new(Job {
            cancel: Cancel::new(),
            listener: None,
        })
    }

    /// Stops the call it was given to, which then fails with
    /// [`MobileError::Cancelled`] and leaves nothing behind.
    pub fn cancel(&self) {
        self.cancel.cancel();
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel.is_cancelled()
    }
}

impl Job {
    fn control(&self) -> Control {
        let progress = self.listener.clone().map(|listener| {
            let last = Mutex::new(None::<Instant>);
            let tell: Arc<dyn Fn(&Progress) + Send + Sync> = Arc::new(move |p: &Progress| {
                let done =
                    p.total_bytes > 0 && p.bytes >= p.total_bytes && p.entries >= p.total_entries;
                let now = Instant::now();
                let Ok(mut last) = last.lock() else { return };
                if done || last.is_none_or(|at| now.duration_since(at) >= TELL_EVERY) {
                    *last = Some(now);
                    listener.progress(JobProgress::from(p));
                }
            });
            tell
        });
        Control {
            progress,
            cancel: Some(self.cancel.clone()),
            ..Control::default()
        }
    }
}

/// What an archive is, as its header shows it.
#[derive(Clone, Debug, PartialEq, uniffi::Record)]
pub struct ArchiveInfo {
    /// Its file's name.
    pub name: String,
    /// Its file's size.
    pub packed: u64,
    /// What its files come to, once extracted.
    pub unpacked: u64,
    pub files: u64,
    /// How it is encrypted, as packwing-core names it; none if it is not.
    pub encryption: Option<String>,
}

/// A file or folder in an archive's folder.
#[derive(Clone, Debug, PartialEq, uniffi::Record)]
pub struct EntryRow {
    pub name: String,
    pub folder: bool,
    /// Its size; a folder's, what is under it.
    pub size: u64,
    /// The files under a folder.
    pub files: u64,
    /// When it was last changed, in seconds since 1970, if the archive says.
    pub modified: Option<i64>,
    /// A symbolic link: not opened, nor previewed.
    pub link: bool,
}

impl From<Child> for EntryRow {
    fn from(c: Child) -> EntryRow {
        EntryRow {
            name: c.name,
            folder: c.dir,
            size: c.size,
            files: c.files,
            modified: c.modified,
            link: c.link.is_some(),
        }
    }
}

/// What a search found: its path in the archive, the folders and its name.
#[derive(Clone, Debug, PartialEq, uniffi::Record)]
pub struct FoundRow {
    pub path: Vec<String>,
    pub entry: EntryRow,
}

/// How an archive is extracted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Record)]
pub struct ExtractHow {
    /// Replaces a file that is there; otherwise one stops the job, before
    /// anything is written.
    #[uniffi(default = false)]
    pub overwrite: bool,
    /// The most it may write, in bytes: what stops an archive bomb. 0 for no
    /// limit, but the ratio of written to compressed stays.
    #[uniffi(default = 100000000000)]
    pub limit: u64,
}

impl Default for ExtractHow {
    fn default() -> ExtractHow {
        ExtractHow {
            overwrite: false,
            limit: 100_000_000_000,
        }
    }
}

/// What was extracted.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, uniffi::Record)]
pub struct Extracted {
    pub files: u64,
    pub folders: u64,
    pub bytes: u64,
    /// Left out: links leading outside the folder, and the like.
    pub skipped: u64,
}

impl From<Summary> for Extracted {
    fn from(s: Summary) -> Extracted {
        Extracted {
            files: s.files,
            folders: s.directories,
            bytes: s.bytes,
            skipped: s.skipped.len() as u64,
        }
    }
}

/// An archive, open: what it holds is read once, as it opens, and kept.
/// Reading a file of it, or extracting, reads it again, from wherever it came
/// from: a file (iOS, whose picker gives one, and whose security-scoped URL
/// the app keeps for as long as it keeps this) or a document (Android, whose
/// picker does not).
#[derive(uniffi::Object)]
pub struct Archive {
    pub(crate) source: Source,
    pub(crate) secrets: Secrets,
    pub(crate) listing: Listing,
    /// Read from a file the system opened for the app (Android's document):
    /// it is changed as a copy, which the app writes over it.
    pub(crate) document: bool,
}

#[uniffi::export]
impl Archive {
    /// Opens the archive at `path` with `secrets`; one that needs more fails
    /// with [`MobileError::Locked`], and is opened again with them.
    #[uniffi::constructor]
    pub fn open(
        path: String,
        secrets: Secrets,
        job: Arc<Job>,
    ) -> Result<Arc<Archive>, MobileError> {
        Archive::of(Source::File(PathBuf::from(path)), secrets, job)
    }

    pub fn info(&self) -> ArchiveInfo {
        let l = &self.listing;
        ArchiveInfo {
            name: l.name.clone(),
            packed: l.packed,
            unpacked: l.unpacked,
            files: l.files,
            encryption: l.encryption.clone(),
        }
    }

    /// What is in `folder` (its path in the archive; empty for the top), as
    /// the desktop lists it: folders first, then files, by name.
    pub fn children(&self, folder: Vec<String>) -> Vec<EntryRow> {
        self.listing
            .children(&folder)
            .into_iter()
            .map(EntryRow::from)
            .collect()
    }

    /// Every file and folder whose name holds `query`, whatever the case
    /// and accents (`bao cao` finds `Báo cáo`).
    pub fn search(&self, query: String) -> Vec<FoundRow> {
        self.listing
            .search(&query)
            .into_iter()
            .map(|f| FoundRow {
                path: f.path,
                entry: EntryRow::from(f.child),
            })
            .collect()
    }

    /// What the file at `entry` holds, if that is `most` bytes or less: to
    /// show it.
    pub fn preview(
        &self,
        entry: Vec<String>,
        most: u64,
        job: Arc<Job>,
    ) -> Result<Vec<u8>, MobileError> {
        Ok(packwing_core::preview_entry(
            self.source.clone(),
            &entry,
            &self.secrets.keys(),
            most,
            &job.control(),
        )?)
    }

    /// A copy of the file at `entry`, to open (Quick Look shows it): in a new
    /// folder of the temporary one, readable by the app only, gone a day later.
    pub fn open_copy(&self, entry: Vec<String>, job: Arc<Job>) -> Result<String, MobileError> {
        let copy = packwing_core::extract_to_open(
            self.source.clone(),
            &entry,
            &self.secrets.keys(),
            &job.control(),
        )?;
        Ok(copy.display().to_string())
    }

    /// Extracts it into `destination`, made if it is not there: all of it, as
    /// its folders have it, or only `picked` (paths in it), each straight
    /// into `destination` with what is under it. Nothing is written outside
    /// it or through a link, and a job that fails or is stopped leaves
    /// nothing.
    pub fn extract(
        &self,
        destination: String,
        picked: Vec<Vec<String>>,
        how: ExtractHow,
        job: Arc<Job>,
    ) -> Result<Extracted, MobileError> {
        let destination = PathBuf::from(destination);
        let options = ExtractOptions {
            overwrite: how.overwrite,
            limit: (how.limit > 0).then_some(how.limit),
            // No quarantine to keep on a phone.
            keep_mark: false,
            ..ExtractOptions::default()
        };
        let keys = self.secrets.keys();
        let control = job.control();
        let summary = if picked.is_empty() {
            packwing_core::extract_with(
                self.source.clone(),
                &destination,
                &keys,
                &options,
                &control,
            )?
        } else {
            packwing_core::extract_selected(
                self.source.clone(),
                &destination,
                &picked,
                &keys,
                &options,
                &control,
            )?
        };
        Ok(Extracted::from(summary))
    }
}

impl Archive {
    /// Reads what `source` holds, as every way of opening one does.
    pub(crate) fn of(
        source: Source,
        secrets: Secrets,
        job: Arc<Job>,
    ) -> Result<Arc<Archive>, MobileError> {
        Archive::read(source, secrets, job).map(Arc::new)
    }

    /// The archive `source` holds, read with `secrets`.
    pub(crate) fn read(
        source: Source,
        secrets: Secrets,
        job: Arc<Job>,
    ) -> Result<Archive, MobileError> {
        let listen = Listen {
            cancel: Some(job.cancel.clone()),
            ..Listen::default()
        };
        let listing = packwing_core::list_as_read(source.clone(), &secrets.keys(), listen)?;
        Ok(Archive {
            source,
            secrets,
            listing,
            document: false,
        })
    }
}

/// The kinds of archive the phone makes: the desktop's.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, uniffi::Enum)]
pub enum ArchiveFormat {
    #[default]
    Zip,
    SevenZip,
    TarZst,
    TarGz,
    TarXz,
    TarBz2,
    TarLz4,
    Tar,
    /// One file compressed by itself, with no archive around it.
    Gz,
    Zst,
    Xz,
    Bz2,
    Lz4,
}

impl ArchiveFormat {
    /// Those for any files, most used first, then those for one file by
    /// itself: as the desktop lists them.
    const ALL: [ArchiveFormat; 13] = [
        ArchiveFormat::Zip,
        ArchiveFormat::SevenZip,
        ArchiveFormat::TarZst,
        ArchiveFormat::TarGz,
        ArchiveFormat::TarXz,
        ArchiveFormat::TarBz2,
        ArchiveFormat::TarLz4,
        ArchiveFormat::Tar,
        ArchiveFormat::Gz,
        ArchiveFormat::Zst,
        ArchiveFormat::Xz,
        ArchiveFormat::Bz2,
        ArchiveFormat::Lz4,
    ];

    fn kind(self) -> Kind {
        match self {
            ArchiveFormat::Zip => Kind::Zip,
            ArchiveFormat::SevenZip => Kind::SevenZip,
            ArchiveFormat::TarZst => Kind::TarZst,
            ArchiveFormat::TarGz => Kind::TarGz,
            ArchiveFormat::TarXz => Kind::TarXz,
            ArchiveFormat::TarBz2 => Kind::TarBz2,
            ArchiveFormat::TarLz4 => Kind::TarLz4,
            ArchiveFormat::Tar => Kind::Tar,
            ArchiveFormat::Gz => Kind::Gz,
            ArchiveFormat::Zst => Kind::Zst,
            ArchiveFormat::Xz => Kind::Xz,
            ArchiveFormat::Bz2 => Kind::Bz2,
            ArchiveFormat::Lz4 => Kind::Lz4,
        }
    }

    fn ext(self) -> &'static str {
        match self {
            ArchiveFormat::Zip => ".zip",
            ArchiveFormat::SevenZip => ".7z",
            ArchiveFormat::TarZst => ".tar.zst",
            ArchiveFormat::TarGz => ".tar.gz",
            ArchiveFormat::TarXz => ".tar.xz",
            ArchiveFormat::TarBz2 => ".tar.bz2",
            ArchiveFormat::TarLz4 => ".tar.lz4",
            ArchiveFormat::Tar => ".tar",
            ArchiveFormat::Gz => ".gz",
            ArchiveFormat::Zst => ".zst",
            ArchiveFormat::Xz => ".xz",
            ArchiveFormat::Bz2 => ".bz2",
            ArchiveFormat::Lz4 => ".lz4",
        }
    }
}

/// A kind of archive, as the app offers it.
#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct ArchiveFormatRow {
    pub format: ArchiveFormat,
    /// What follows its name: `.tar.zst`.
    pub ext: String,
    /// It holds one file by itself, with no archive around it.
    pub one_file: bool,
    /// Its files can go in as they are ([`CompressionLevel::Store`]): ZIP's and 7z's.
    pub stores: bool,
    /// A password encrypts it whole, with age, to that passphrase: all but ZIP
    /// and 7z, which have their own AES-256.
    pub uses_age: bool,
    /// The names in it can be hidden with its password: 7z's.
    pub hides_names: bool,
}

/// The kinds of archive there are to make, in the desktop's order: those for
/// any files, then, with `one_file`, those for one file by itself.
#[uniffi::export]
pub fn archive_formats(one_file: bool) -> Vec<ArchiveFormatRow> {
    ArchiveFormat::ALL
        .into_iter()
        .filter(|f| one_file || !f.kind().one_file())
        .map(|f| ArchiveFormatRow {
            format: f,
            ext: f.ext().to_owned(),
            one_file: f.kind().one_file(),
            stores: matches!(f, ArchiveFormat::Zip | ArchiveFormat::SevenZip),
            uses_age: !matches!(f, ArchiveFormat::Zip | ArchiveFormat::SevenZip),
            hides_names: f == ArchiveFormat::SevenZip,
        })
        .collect()
}

/// How hard a new archive is compressed, as the desktop's five levels and
/// Store.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, uniffi::Enum)]
pub enum CompressionLevel {
    Store,
    Lowest,
    Low,
    #[default]
    Medium,
    High,
    Highest,
}

impl CompressionLevel {
    fn level(self) -> Level {
        match self {
            CompressionLevel::Store => Level::Store,
            CompressionLevel::Lowest => Level::Lowest,
            CompressionLevel::Low => Level::Low,
            CompressionLevel::Medium => Level::Medium,
            CompressionLevel::High => Level::High,
            CompressionLevel::Highest => Level::Highest,
        }
    }
}

/// How a new archive is made.
#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct NewArchiveHow {
    pub format: ArchiveFormat,
    pub compression: CompressionLevel,
    /// Encrypts its files with AES-256 (ZIP, 7z), or all of it with age to
    /// this passphrase (the others).
    #[uniffi(default = None)]
    pub password: Option<String>,
    /// Encrypts a 7z archive's names too, with its password.
    #[uniffi(default = false)]
    pub hide_names: bool,
    /// Cuts it into `name.zip.001`, `name.zip.002`, … of this many bytes, as
    /// 7-Zip does.
    #[uniffi(default = None)]
    pub part_size: Option<u64>,
    /// Encrypts all of it with age to these public keys (age's, or SSH
    /// keys), in place of a passphrase: formats other than ZIP and 7z.
    #[uniffi(default = [])]
    pub age_keys: Vec<String>,
    /// What is compressed already (pictures, video, sound, archives) goes in
    /// as it is, in ZIP and 7z.
    #[uniffi(default = true)]
    pub store_compressed: bool,
    /// What only a Mac uses (`.DS_Store`, `__MACOSX`, resource forks) is left out.
    #[uniffi(default = true)]
    pub skip_mac_files: bool,
    /// Encrypts a ZIP or 7z archive whole with age, as the other formats are,
    /// in place of its own AES-256: to `password` as a passphrase, or to
    /// `age_keys`; its names are hidden too. Saved as `name.zip.age`.
    #[uniffi(default = false)]
    pub whole_with_age: bool,
    /// What goes in as it is, with `store_compressed`: the files whose names
    /// end in these, whatever their case; none for the desktop's list
    /// ([`default_stored_extensions`]).
    #[uniffi(default = None)]
    pub stored_extensions: Option<Vec<String>>,
    /// Bound for a storage across the network, it is written straight there
    /// if its files come to more than this many bytes and the storage takes
    /// it so; made here first and sent otherwise, and if this is none.
    #[uniffi(default = None)]
    pub stream_above: Option<u64>,
}

impl Default for NewArchiveHow {
    fn default() -> NewArchiveHow {
        NewArchiveHow {
            format: ArchiveFormat::default(),
            compression: CompressionLevel::default(),
            password: None,
            hide_names: false,
            part_size: None,
            age_keys: Vec::new(),
            store_compressed: true,
            skip_mac_files: true,
            whole_with_age: false,
            stored_extensions: None,
            stream_above: None,
        }
    }
}

/// Makes an archive of `paths` (files and folders) at `wanted`, or beside it
/// under a free name if that is taken (`name 2.zip`), and says where. What is
/// compressed already (pictures, video, sound) goes in as it is, and what only
/// a Mac uses (`.DS_Store`, `__MACOSX`) is left out, as on the desktop.
#[uniffi::export]
pub fn create_archive(
    paths: Vec<String>,
    wanted: String,
    how: NewArchiveHow,
    job: Arc<Job>,
) -> Result<String, MobileError> {
    let paths: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    let wanted = packwing_core::free_archive_path(Path::new(&wanted), false);
    let made = packwing_core::create(&paths, &wanted, &how.new_archive(), &job.control())?;
    Ok(made.display().to_string())
}

impl NewArchiveHow {
    /// What the core makes of it.
    pub(crate) fn new_archive(&self) -> NewArchive {
        let how = self.clone();
        NewArchive {
            kind: how.format.kind(),
            // Store is ZIP's and 7z's: a tar format's is `.tar`.
            level: match how.compression {
                CompressionLevel::Store
                    if !matches!(how.format, ArchiveFormat::Zip | ArchiveFormat::SevenZip) =>
                {
                    Level::Lowest
                }
                level => level.level(),
            },
            password: how.password.map(String::into_bytes),
            hide_names: how.hide_names
                && how.format == ArchiveFormat::SevenZip
                && !how.whole_with_age,
            whole_with_age: how.whole_with_age
                && matches!(how.format, ArchiveFormat::Zip | ArchiveFormat::SevenZip),
            part_size: how.part_size,
            age_keys: how.age_keys,
            stored_extensions: match (how.store_compressed, how.stored_extensions) {
                (false, _) => Vec::new(),
                (true, Some(chosen)) => chosen,
                (true, None) => default_stored_extensions(),
            },
            stream_above: how.stream_above,
            skip_mac_files: how.skip_mac_files,
            ..NewArchive::default()
        }
    }
}

/// The extensions of what is compressed already, which goes in as it is: the
/// desktop's list, as Settings starts with it.
#[uniffi::export]
pub fn default_stored_extensions() -> Vec<String> {
    COMPRESSED_EXTENSIONS
        .iter()
        .map(|e| e.to_string())
        .collect()
}

/// The extensions in `text`, as typed, as the desktop reads them: apart by
/// spaces or commas, with or without their dot (or `*.`), each once, in
/// lower case.
#[uniffi::export]
pub fn extensions_in(text: String) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for word in text.split(|c: char| c.is_whitespace() || c == ',' || c == ';') {
        let ext = word
            .trim_start_matches('*')
            .trim_start_matches('.')
            .to_lowercase();
        if !ext.is_empty() && !found.contains(&ext) {
            found.push(ext);
        }
    }
    found
}

/// Whether a picture's `bytes` are of a size that is safe to decode and show:
/// a few hundred million pixels at most.
#[uniffi::export]
pub fn picture_fits(bytes: Vec<u8>) -> bool {
    packwing_core::image_fits(&bytes)
}

/// `name` without its archive extensions: what a folder
/// extracted from it is called (`Báo cáo.tar.gz` → `Báo cáo`).
#[uniffi::export]
pub fn stem(name: String) -> String {
    packwing_core::stem(&name).to_owned()
}
