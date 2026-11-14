//! Android's way in: a file the system has opened for the app, read where the
//! app asks. Only built for Android, so neither the iOS app's Swift nor the
//! desktop ever sees it.
//!
//! Android's document picker, and its gallery, hand out a `content://` URI,
//! not a path. The descriptor the system opens from one can be read, but
//! `/proc/self/fd/N` is not a path the app can open again: opening it is a
//! fresh open of the file underneath, checked against the app's own rights
//! rather than against the grant that handed over the descriptor. So the app
//! used to copy every archive into its own folder before packwing-core could
//! read it.
//!
//! It no longer has to. packwing-core already reads an archive that is not on
//! this computer by asking for the ranges it wants, as it does for a file on
//! a storage ([`Stored`]); a document is the same thing with a shorter way to
//! go. ZIP and 7z archives, which is what a phone opens and makes, are read
//! this way whole: listed, previewed and extracted straight from the
//! document. A RAR archive is the exception, as the library that reads one
//! takes a filename: packwing-core writes it to a temporary file first.

use std::io::{self, Read};
use std::path::PathBuf;
use std::sync::Arc;

use packwing_core::{Destination, ExtractOptions, Item, Lister, Source, Storage, Stored};

use crate::{Archive, ExtractHow, Extracted, Job, MobileError, Secrets};

/// A file the system opened for the app: Kotlin reads it through the
/// descriptor the picker gave, with `pread`, which asks no rights of its own.
///
/// Called from whatever thread the work is on, and from several at once.
#[uniffi::export(with_foreign)]
pub trait Document: Send + Sync {
    /// Its name, which is what the archive is then called.
    fn name(&self) -> String;

    /// How long it is, in bytes.
    fn size(&self) -> u64;

    /// `length` bytes from `offset`; fewer only at its end, and none past it.
    fn read_at(&self, offset: u64, length: u32) -> Result<Vec<u8>, MobileError>;
}

/// The one document, as the storage packwing-core reads an archive from.
struct OneDocument(Arc<dyn Document>);

/// A document is one file: everything but reading it is for a real storage.
fn one_file_only() -> io::Error {
    io::Error::new(
        io::ErrorKind::Unsupported,
        "a document is one file, read where it is asked for",
    )
}

impl Storage for OneDocument {
    /// As many as were asked for, a short read being only the file's end: the
    /// app may hand back less than a whole block at a time.
    fn read_at(&self, _path: &[String], offset: u64, len: usize) -> io::Result<Vec<u8>> {
        let mut got: Vec<u8> = Vec::new();
        while got.len() < len {
            let want = u32::try_from(len - got.len()).unwrap_or(u32::MAX);
            let more = self
                .0
                .read_at(offset + got.len() as u64, want)
                .map_err(io::Error::other)?;
            if more.is_empty() {
                break;
            }
            got.extend_from_slice(&more);
        }
        Ok(got)
    }

    fn read(&self, path: &[String], most: u64) -> io::Result<Vec<u8>> {
        self.read_at(path, 0, usize::try_from(most).unwrap_or(usize::MAX))
    }

    fn local_path(&self, _path: &[String]) -> Option<PathBuf> {
        None
    }

    fn show_path(&self, path: &[String]) -> String {
        path.join("/")
    }

    fn has_trash(&self) -> bool {
        false
    }

    fn list(&self, _folder: &[String]) -> io::Result<Box<dyn Lister>> {
        Err(one_file_only())
    }

    fn stat(&self, _folder: &[String], _name: &str) -> io::Result<Option<Item>> {
        Err(one_file_only())
    }

    fn open(&self, _path: &[String]) -> io::Result<Box<dyn Read + Send>> {
        Err(one_file_only())
    }

    fn put(
        &self,
        _path: &[String],
        _from: &mut dyn Read,
        _like: &Item,
        _replace: bool,
    ) -> io::Result<u64> {
        Err(one_file_only())
    }

    fn remove(&self, _path: &[String]) -> io::Result<()> {
        Err(one_file_only())
    }

    fn make_folder(&self, _folder: &[String], _name: &str) -> io::Result<()> {
        Err(one_file_only())
    }

    fn rename(&self, _folder: &[String], _from: &str, _to: &str) -> io::Result<()> {
        Err(one_file_only())
    }

    fn delete(&self, _folder: &[String], _names: &[String]) -> io::Result<()> {
        Err(one_file_only())
    }
}

#[uniffi::export]
impl Archive {
    /// Opens what `document` holds, with no copy of it: for an archive the
    /// document picker, the gallery or another app handed over, which has no
    /// path. Fails as [`Archive::open`] does, and is opened again with the
    /// secrets a locked one asks for.
    #[uniffi::constructor]
    pub fn open_document(
        document: Arc<dyn Document>,
        secrets: Secrets,
        job: Arc<Job>,
    ) -> Result<Arc<Archive>, MobileError> {
        let (name, size) = (document.name(), document.size());
        let storage: Arc<dyn Storage> = Arc::new(OneDocument(document));
        Archive::of(
            Source::Stored(Stored::new(storage, vec![name], size)),
            secrets,
            job,
        )
    }
}

/// Where extracting writes, in place of a folder: Android's document tree,
/// which is reached through the system and has no path to write to.
///
/// The entries come in the order a folder's extraction writes them. A folder
/// on the way to a file comes only if the archive holds an entry for it, so
/// the app makes the ones between itself. Called from the thread the work is
/// on, one entry at a time.
#[uniffi::export(with_foreign)]
pub trait Tree: Send + Sync {
    /// A folder, and the ones above it that are not there yet.
    fn directory(&self, path: Vec<String>) -> Result<(), MobileError>;

    /// A file, whose data follows; `size` is what the archive says it is,
    /// where it says before the data.
    fn start_file(&self, path: Vec<String>, size: Option<u64>) -> Result<(), MobileError>;

    /// A piece of the file's data.
    fn write(&self, data: Vec<u8>) -> Result<(), MobileError>;

    /// The end of the file, all of its data given.
    fn finish_file(&self) -> Result<(), MobileError>;

    /// The end of a file whose data failed: what was given of it goes.
    fn discard_file(&self);
}

/// A [`Tree`] as what packwing-core writes an extraction into.
struct Writing(Arc<dyn Tree>);

/// What the app said went wrong, as the core takes it: its words are kept, so
/// that what the user is shown is the app's own and not a number.
fn as_io(e: MobileError) -> io::Error {
    io::Error::other(e.to_string())
}

impl Destination for Writing {
    fn directory(&mut self, path: &[String]) -> io::Result<()> {
        self.0.directory(path.to_vec()).map_err(as_io)
    }

    fn start_file(&mut self, path: &[String], size: Option<u64>) -> io::Result<()> {
        self.0.start_file(path.to_vec(), size).map_err(as_io)
    }

    fn write(&mut self, data: &[u8]) -> io::Result<()> {
        self.0.write(data.to_vec()).map_err(as_io)
    }

    fn finish_file(&mut self) -> io::Result<()> {
        self.0.finish_file().map_err(as_io)
    }

    fn discard_file(&mut self) {
        self.0.discard_file();
    }
}

#[uniffi::export]
impl Archive {
    /// Extracts it into `tree`, writing each file where it is made rather
    /// than into a folder and across: all of it, or only `picked` (paths in
    /// it), which keep the folders they are in.
    pub fn extract_into(
        &self,
        tree: Arc<dyn Tree>,
        picked: Vec<Vec<String>>,
        how: ExtractHow,
        job: Arc<Job>,
    ) -> Result<Extracted, MobileError> {
        let options = ExtractOptions {
            overwrite: how.overwrite,
            // No quarantine to keep on a phone.
            keep_mark: false,
            ..ExtractOptions::default()
        };
        let mut writing = Writing(tree);
        let summary = packwing_core::extract_to(
            self.source.clone(),
            &mut writing,
            &picked,
            &self.secrets.keys(),
            &options,
            &job.control(),
        )?;
        Ok(Extracted::from(summary))
    }
}
