//! What the iOS app calls: an archive opened and what it holds, folder by
//! folder; a file of it read, to preview it; its files extracted; and new ZIP
//! and 7z archives made. The work is packwing-core's, as on the desktop.
//!
//! Every call that reads or writes a file blocks until it is done, so the app
//! makes them off its main thread; a [`Job`] tells how far one is, and stops
//! it. The Swift side is generated from this crate by UniFFI
//! (`scripts/build_ios.sh`); names here are what Swift sees, in its case.

uniffi::setup_scaffolding!();

mod i18n;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use packwing_core::{
    Asked, COMPRESSED_EXTENSIONS, Cancel, Child, Control, Error, ExtractOptions, Keys, Kind, Level,
    Listen, Listing, NewArchive, Progress, Summary,
};

pub use i18n::LanguageRow;

/// Why a call did not do what it was asked, as the app says it: `message` is
/// in the user's language already.
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum MobileError {
    /// Not an archive packwing opens, or a kind it does not read yet.
    #[error("{message}")]
    NotAnArchive { message: String },
    /// Encrypted, and nothing given opens it: it needs a password (ZIP, 7z,
    /// RAR), or a passphrase (an age file) with `passphrase`.
    #[error("{message}")]
    Locked { message: String, passphrase: bool },
    /// The password or passphrase given is not its own, or its data is damaged.
    #[error("{message}")]
    WrongPassword { message: String },
    /// Stopped by [`Job::cancel`].
    #[error("{message}")]
    Cancelled { message: String },
    #[error("{message}")]
    Failed { message: String },
}

impl From<Error> for MobileError {
    fn from(e: Error) -> MobileError {
        let message = e.to_string();
        match e {
            Error::NotAnArchive | Error::Unsupported(_) => MobileError::NotAnArchive { message },
            Error::Locked { needs } => match Keys::asks_for(&needs) {
                Some(Asked::Password) => MobileError::Locked {
                    message,
                    passphrase: false,
                },
                Some(Asked::Passphrase) => MobileError::Locked {
                    message,
                    passphrase: true,
                },
                // Age keys, and their files' passphrases: not on the phone yet.
                _ => MobileError::Failed { message },
            },
            Error::WrongPassword | Error::WrongPassphrase | Error::WrongKeyPassphrase => {
                MobileError::WrongPassword { message }
            }
            Error::Cancelled => MobileError::Cancelled { message },
            Error::Failed(_) => MobileError::Failed { message },
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
    packwing_core::keep_journal_in(PathBuf::from(support_folder));
    std::thread::spawn(packwing_core::clean_left_extractions);
}

/// What opens an encrypted archive: its password (ZIP, 7z, RAR), and the
/// passphrase of an age file.
#[derive(Clone, Debug, Default, PartialEq, Eq, uniffi::Record)]
pub struct Secrets {
    #[uniffi(default = None)]
    pub password: Option<String>,
    #[uniffi(default = None)]
    pub passphrase: Option<String>,
}

impl Secrets {
    fn keys(&self) -> Keys {
        Keys {
            password: self.password.as_ref().map(|p| p.as_bytes().to_vec()),
            passphrase: self.passphrase.as_ref().map(|p| p.as_bytes().to_vec()),
            ..Keys::default()
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, uniffi::Record)]
pub struct ExtractHow {
    /// Replaces a file that is there; otherwise one stops the job, before
    /// anything is written.
    #[uniffi(default = false)]
    pub overwrite: bool,
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
/// Reading a file of it, or extracting, reads its file again: the app keeps
/// access to it (a security-scoped URL) for as long as it keeps this.
#[derive(uniffi::Object)]
pub struct Archive {
    path: PathBuf,
    secrets: Secrets,
    listing: Listing,
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
        let path = PathBuf::from(path);
        let listen = Listen {
            cancel: Some(job.cancel.clone()),
            ..Listen::default()
        };
        let listing = packwing_core::list_as_read(path.clone(), &secrets.keys(), listen)?;
        Ok(Arc::new(Archive {
            path,
            secrets,
            listing,
        }))
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
            self.path.clone(),
            &entry,
            &self.secrets.keys(),
            most,
            &job.control(),
        )?)
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
            // No quarantine to keep on a phone.
            keep_mark: false,
            ..ExtractOptions::default()
        };
        let keys = self.secrets.keys();
        let control = job.control();
        let summary = if picked.is_empty() {
            packwing_core::extract_with(self.path.clone(), &destination, &keys, &options, &control)?
        } else {
            packwing_core::extract_selected(
                self.path.clone(),
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

/// The kinds of archive the phone makes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, uniffi::Enum)]
pub enum ArchiveFormat {
    #[default]
    Zip,
    SevenZip,
}

impl ArchiveFormat {
    fn kind(self) -> Kind {
        match self {
            ArchiveFormat::Zip => Kind::Zip,
            ArchiveFormat::SevenZip => Kind::SevenZip,
        }
    }
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
#[derive(Clone, Debug, Default, PartialEq, Eq, uniffi::Record)]
pub struct NewArchiveHow {
    pub format: ArchiveFormat,
    pub compression: CompressionLevel,
    /// Encrypts its files with AES-256.
    #[uniffi(default = None)]
    pub password: Option<String>,
    /// Encrypts a 7z archive's names too, with its password.
    #[uniffi(default = false)]
    pub hide_names: bool,
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
    let new = NewArchive {
        kind: how.format.kind(),
        level: how.compression.level(),
        password: how.password.map(String::into_bytes),
        hide_names: how.hide_names && how.format == ArchiveFormat::SevenZip,
        stored_extensions: COMPRESSED_EXTENSIONS
            .iter()
            .map(|e| e.to_string())
            .collect(),
        skip_mac_files: true,
        ..NewArchive::default()
    };
    let made = packwing_core::create(&paths, &wanted, &new, &job.control())?;
    Ok(made.display().to_string())
}

/// Whether a picture's `bytes` are of a size that is safe to decode and show:
/// a few hundred million pixels at most.
#[uniffi::export]
pub fn picture_fits(bytes: Vec<u8>) -> bool {
    packwing_core::image_fits(&bytes)
}
