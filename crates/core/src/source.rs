//! What an archive is read from: a file on this computer, in its parts if it has
//! any, or a file on a storage, read by ranges as it is asked for. A block once
//! read is kept, so that listing an archive and then taking a file from it reads
//! its directory once.

use std::collections::HashMap;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use hivepack::zip::ReadAt;

use crate::joined::{Joined, joined_size};
use crate::storage::Storage;

/// How much is fetched at the least: measured on a ZIP archive of 200,000 small
/// files, 50 of them scattered took 3 MiB this way and 200 MiB by 4 MiB blocks.
const BLOCK: u64 = 64 << 10;
/// The most fetched at once, reads that go in order fetching more and more.
const MOST_AHEAD: u64 = 16 << 20;
/// The most kept, in bytes.
const MOST_KEPT: u64 = 64 << 20;

#[derive(Default)]
struct Blocks {
    kept: HashMap<u64, (Arc<Vec<u8>>, u64)>,
    tick: u64,
    /// The block after the last read, and how many to fetch from there.
    next: u64,
    window: u64,
}

/// A file on a storage, to read an archive from.
#[derive(Clone)]
pub struct Stored {
    storage: Arc<dyn Storage>,
    path: Vec<String>,
    size: u64,
    blocks: Arc<Mutex<Blocks>>,
    fetched: Arc<AtomicU64>,
}

impl Stored {
    pub fn new(storage: Arc<dyn Storage>, path: Vec<String>, size: u64) -> Stored {
        Stored {
            storage,
            path,
            size,
            blocks: Arc::default(),
            fetched: Arc::default(),
        }
    }

    /// How many bytes of it have come across so far.
    pub fn fetched(&self) -> u64 {
        self.fetched.load(Ordering::Relaxed)
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    /// Its storage, and its path there.
    pub(crate) fn place(&self) -> (Arc<dyn Storage>, Vec<String>) {
        (self.storage.clone(), self.path.clone())
    }

    /// `first..=last`, those not kept fetched in one read, with what reads in
    /// order may want next.
    fn blocks(&self, first: u64, last: u64) -> io::Result<Vec<Arc<Vec<u8>>>> {
        let mut b = self.blocks.lock().unwrap_or_else(PoisonError::into_inner);
        let in_order = first == b.next && first != 0;
        b.window = if in_order {
            (b.window * 2).clamp(1, MOST_AHEAD / BLOCK)
        } else {
            1
        };
        b.next = last + 1;
        let end_block = self.size.saturating_sub(1) / BLOCK;
        let upto = last.max(first + b.window - 1).min(end_block);
        let missing: Vec<u64> = (first..=upto)
            .filter(|at| !b.kept.contains_key(at))
            .collect();
        if let (Some(&low), Some(&high)) = (missing.first(), missing.last()) {
            let end = ((high + 1) * BLOCK).min(self.size);
            let data =
                self.storage
                    .read_at(&self.path, low * BLOCK, (end - low * BLOCK) as usize)?;
            self.fetched.fetch_add(data.len() as u64, Ordering::Relaxed);
            for at in low..=high {
                let from = (((at - low) * BLOCK) as usize).min(data.len());
                let to = (from + BLOCK as usize).min(data.len());
                b.tick += 1;
                let tick = b.tick;
                b.kept.insert(at, (Arc::new(data[from..to].to_vec()), tick));
            }
            while b.kept.len() as u64 * BLOCK > MOST_KEPT {
                let oldest = b.kept.iter().min_by_key(|(_, v)| v.1).map(|(at, _)| *at);
                let Some(oldest) = oldest else { break };
                // What this read needs stays.
                if (first..=last).contains(&oldest) {
                    break;
                }
                b.kept.remove(&oldest);
            }
        }
        b.tick += 1;
        let tick = b.tick;
        let mut found = Vec::with_capacity((last - first + 1) as usize);
        for at in first..=last {
            let kept = b.kept.get_mut(&at).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "the file is shorter than it was",
                )
            })?;
            kept.1 = tick;
            found.push(kept.0.clone());
        }
        Ok(found)
    }

    fn read_range(&self, offset: u64, buf: &mut [u8]) -> io::Result<usize> {
        if offset >= self.size || buf.is_empty() {
            return Ok(0);
        }
        let end = (offset + buf.len() as u64).min(self.size);
        let (first, last) = (offset / BLOCK, (end - 1) / BLOCK);
        let mut written = 0;
        for (i, data) in self.blocks(first, last)?.iter().enumerate() {
            let at = (first + i as u64) * BLOCK;
            let from = (offset.max(at) - at) as usize;
            let to = ((end.min(at + BLOCK) - at) as usize).min(data.len());
            if from >= to {
                break;
            }
            buf[written..written + to - from].copy_from_slice(&data[from..to]);
            written += to - from;
        }
        Ok(written)
    }
}

/// [`Stored`], with a position.
pub(crate) struct StoredReader {
    stored: Stored,
    position: u64,
}

impl Read for StoredReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.stored.read_range(self.position, buf)?;
        self.position += n as u64;
        Ok(n)
    }
}

impl Seek for StoredReader {
    fn seek(&mut self, to: SeekFrom) -> io::Result<u64> {
        let position = match to {
            SeekFrom::Start(p) => Some(p),
            SeekFrom::Current(d) => self.position.checked_add_signed(d),
            SeekFrom::End(d) => self.stored.size.checked_add_signed(d),
        };
        self.position =
            position.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "seek before 0"))?;
        Ok(self.position)
    }
}

impl ReadAt for StoredReader {
    fn size(&self) -> io::Result<u64> {
        Ok(self.stored.size)
    }

    /// As many as there are: a read may stop at a block's end.
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> io::Result<usize> {
        let mut done = 0;
        while done < buf.len() {
            match self
                .stored
                .read_range(offset + done as u64, &mut buf[done..])?
            {
                0 => break,
                n => done += n,
            }
        }
        Ok(done)
    }
}

/// Where an archive is.
#[derive(Clone)]
pub enum Source {
    File(PathBuf),
    Stored(Stored),
}

impl PartialEq for Source {
    fn eq(&self, other: &Source) -> bool {
        match (self, other) {
            (Source::File(a), Source::File(b)) => a == b,
            (Source::Stored(a), Source::Stored(b)) => {
                Arc::ptr_eq(&a.storage, &b.storage) && a.path == b.path
            }
            _ => false,
        }
    }
}

impl std::fmt::Debug for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Source::File(path) => write!(f, "File({path:?})"),
            Source::Stored(s) => write!(f, "Stored({:?})", s.path),
        }
    }
}

impl From<&Path> for Source {
    fn from(path: &Path) -> Source {
        Source::File(path.to_path_buf())
    }
}

impl From<PathBuf> for Source {
    fn from(path: PathBuf) -> Source {
        Source::File(path)
    }
}

impl From<&PathBuf> for Source {
    fn from(path: &PathBuf) -> Source {
        Source::File(path.clone())
    }
}

impl From<&Source> for Source {
    fn from(source: &Source) -> Source {
        source.clone()
    }
}

impl Source {
    /// Its file on this computer, if it is on this computer.
    pub fn path(&self) -> Option<&Path> {
        match self {
            Source::File(path) => Some(path),
            Source::Stored(_) => None,
        }
    }

    /// How much has come across a network to read it, if it is on a storage, and
    /// its size.
    pub fn fetched(&self) -> Option<(u64, u64)> {
        match self {
            Source::File(_) => None,
            Source::Stored(s) => Some((s.fetched(), s.size)),
        }
    }

    /// Its file's name.
    pub fn name(&self) -> String {
        match self {
            Source::File(path) => path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default(),
            Source::Stored(s) => s.path.last().cloned().unwrap_or_default(),
        }
    }

    /// The folder it is in, as the user knows it.
    pub fn folder(&self) -> PathBuf {
        match self {
            Source::File(path) => path.parent().map(Path::to_path_buf).unwrap_or_default(),
            Source::Stored(s) => {
                let folder = &s.path[..s.path.len().saturating_sub(1)];
                PathBuf::from(s.storage.show_path(folder))
            }
        }
    }

    /// Its size, all its parts together.
    pub(crate) fn size(&self) -> io::Result<u64> {
        match self {
            Source::File(path) => joined_size(path),
            Source::Stored(s) => Ok(s.size),
        }
    }

    /// It, from its start.
    pub(crate) fn reader(&self) -> io::Result<SourceReader> {
        Ok(match self {
            Source::File(path) => SourceReader::File(Joined::open(path)?),
            Source::Stored(s) => SourceReader::Stored(s.stored_reader()),
        })
    }
}

impl Stored {
    pub(crate) fn stored_reader(&self) -> StoredReader {
        StoredReader {
            stored: self.clone(),
            position: 0,
        }
    }
}

/// [`Source::reader`]'s reader.
pub(crate) enum SourceReader {
    File(Joined),
    Stored(StoredReader),
}

impl Read for SourceReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            SourceReader::File(r) => r.read(buf),
            SourceReader::Stored(r) => r.read(buf),
        }
    }
}

impl Seek for SourceReader {
    fn seek(&mut self, to: SeekFrom) -> io::Result<u64> {
        match self {
            SourceReader::File(r) => r.seek(to),
            SourceReader::Stored(r) => r.seek(to),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::atomic::AtomicUsize;

    use super::*;
    use crate::storage::{Item, Lister, LocalFolder};
    use crate::{
        Control, Keys, Kind, Level, NewArchive, create, extract_selected, list, read_entry, test,
    };

    /// A folder that does not say where it is, as one across a network, counting
    /// its reads.
    struct Far(LocalFolder, AtomicUsize);

    impl Storage for Far {
        fn list(&self, f: &[String]) -> io::Result<Box<dyn Lister>> {
            self.0.list(f)
        }
        fn read(&self, p: &[String], most: u64) -> io::Result<Vec<u8>> {
            self.0.read(p, most)
        }
        fn read_at(&self, p: &[String], offset: u64, len: usize) -> io::Result<Vec<u8>> {
            self.1.fetch_add(1, Ordering::Relaxed);
            self.0.read_at(p, offset, len)
        }
        fn stat(&self, f: &[String], n: &str) -> io::Result<Option<Item>> {
            self.0.stat(f, n)
        }
        fn open(&self, p: &[String]) -> io::Result<Box<dyn Read + Send>> {
            self.0.open(p)
        }
        fn put(
            &self,
            p: &[String],
            from: &mut dyn Read,
            like: &Item,
            replace: bool,
        ) -> io::Result<u64> {
            self.0.put(p, from, like, replace)
        }
        fn remove(&self, p: &[String]) -> io::Result<()> {
            self.0.remove(p)
        }
        fn make_folder(&self, f: &[String], n: &str) -> io::Result<()> {
            self.0.make_folder(f, n)
        }
        fn rename(&self, f: &[String], a: &str, b: &str) -> io::Result<()> {
            self.0.rename(f, a, b)
        }
        fn delete(&self, f: &[String], n: &[String]) -> io::Result<()> {
            self.0.delete(f, n)
        }
        fn has_trash(&self) -> bool {
            false
        }
        fn local_path(&self, _: &[String]) -> Option<PathBuf> {
            None
        }
        fn show_path(&self, p: &[String]) -> String {
            format!("far://{}", p.join("/"))
        }
    }

    fn none() -> Control {
        Control {
            progress: None,
            cancel: None,
            on_damaged: None,
            on_unnamable: None,
            on_links_outside: None,
        }
    }

    /// An archive of `kind` under a new folder: `small.txt` and 8 MiB that do not
    /// compress, and its path.
    fn archive(test: &str, kind: Kind, name: &str) -> (PathBuf, PathBuf) {
        let dir =
            std::env::temp_dir().join(format!("packwing-source-{}-{test}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("in/sub")).expect("mkdir");
        fs::write(dir.join("in/sub/small.txt"), "small").expect("write");
        let mut noise = vec![0u8; 8 << 20];
        let mut x = 88172645463325252u64;
        for b in noise.iter_mut() {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            *b = x as u8;
        }
        fs::write(dir.join("in/big.bin"), &noise).expect("write");
        let at = dir.join(name);
        let how = NewArchive {
            kind,
            level: Level::default(),
            ..NewArchive::default()
        };
        create(&[dir.join("in")], &at, &how, &none()).expect("an archive");
        (dir, at)
    }

    fn stored(dir: &Path, name: &str) -> (Source, Arc<Far>) {
        let far = Arc::new(Far(LocalFolder::new(dir), AtomicUsize::new(0)));
        let size = fs::metadata(dir.join(name)).expect("meta").len();
        let source = Source::Stored(Stored::new(far.clone(), vec![name.to_owned()], size));
        (source, far)
    }

    fn agrees(kind: Kind, name: &str) -> (u64, u64) {
        let (dir, at) = archive(name, kind, name);
        let (source, _) = stored(&dir, name);
        let keys = Keys::default();
        let (here, there) = (
            list(&at, &keys).expect("list"),
            list(&source, &keys).expect("list"),
        );
        assert_eq!(here.entries, there.entries);
        assert_eq!(here.container, there.container);
        assert_eq!(here.packed, there.packed);
        let small = ["in", "sub", "small.txt"].map(String::from).to_vec();
        let bytes = read_entry(&source, &small, &keys, 1000, &none()).expect("read");
        assert_eq!(bytes, b"small");
        let (Source::Stored(s), whole) = (&source, fs::metadata(&at).expect("meta").len()) else {
            unreachable!("made above");
        };
        let fetched = s.fetched();
        // All of it comes out the same as from the file, and is checked.
        let to = dir.join("out");
        let big = ["in", "big.bin"].map(String::from).to_vec();
        extract_selected(&source, &to, &[big], &keys, &Default::default(), &none())
            .expect("extract");
        assert_eq!(
            fs::read(to.join("big.bin")).expect("read"),
            fs::read(dir.join("in/big.bin")).expect("read")
        );
        test(&source, &keys, &none()).expect("test");
        let _ = fs::remove_dir_all(&dir);
        (fetched, whole)
    }

    #[test]
    fn a_zip_on_a_storage_is_read_without_reading_it_all() {
        let (fetched, whole) = agrees(Kind::Zip, "a.zip");
        assert!(fetched < whole / 20, "{fetched} of {whole}");
    }

    #[test]
    fn a_7z_on_a_storage_reads_as_a_file_does() {
        agrees(Kind::SevenZip, "a.7z");
    }

    #[test]
    fn a_compressed_tar_on_a_storage_reads_as_a_file_does() {
        agrees(Kind::TarZst, "a.tar.zst");
    }

    #[test]
    fn what_was_read_is_not_read_again() {
        let (dir, _) = archive("again", Kind::Zip, "a.zip");
        let (source, far) = stored(&dir, "a.zip");
        let keys = Keys::default();
        list(&source, &keys).expect("list");
        let once = far.1.load(Ordering::Relaxed);
        list(&source, &keys).expect("list");
        assert_eq!(far.1.load(Ordering::Relaxed), once);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_file_gone_short_is_an_error() {
        let (dir, _) = archive("short", Kind::Zip, "a.zip");
        let far = Arc::new(Far(LocalFolder::new(&dir), AtomicUsize::new(0)));
        // It says it is longer than it is.
        let size = fs::metadata(dir.join("a.zip")).expect("meta").len();
        let source = Source::Stored(Stored::new(far, vec!["a.zip".into()], size + (1 << 20)));
        assert!(list(&source, &Keys::default()).is_err());
        let _ = fs::remove_dir_all(&dir);
    }
}
