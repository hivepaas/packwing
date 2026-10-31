//! Making a ZIP archive of files and folders, through hivepack-zip. It is
//! written beside what is there, never over it, and taken away again if it
//! fails or is stopped. Links are kept as links, on Unix; what is not a
//! file, a folder or a link (a pipe, a socket, a device) is left out, as
//! reading it could wait for ever.

use std::fmt;
use std::fs::{self, File, Metadata, OpenOptions};
use std::io::{self, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::UNIX_EPOCH;

use hivepack_extract::Progress;
use hivepack_zip::{
    Archive, BufferSource, Closed, DataSource, ErrorCode, FileAttributes, Flags, OpenFlags, Stat,
    cm, em, file_attributes, opsys, stat,
};

use crate::{Control, Error};

/// How a new archive is made.
#[derive(Clone, PartialEq, Eq)]
pub struct NewArchive {
    /// How hard Deflate squeezes, 1 to 9.
    pub level: u32,
    /// Encrypts its files with WinZip AES-256.
    pub password: Option<Vec<u8>>,
    /// Cuts it into `name.zip.001`, `name.zip.002`, … of this many bytes, as
    /// 7-Zip does.
    pub part_size: Option<u64>,
}

impl Default for NewArchive {
    fn default() -> NewArchive {
        NewArchive {
            level: 6,
            password: None,
            part_size: None,
        }
    }
}

impl fmt::Debug for NewArchive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NewArchive")
            .field("level", &self.level)
            .field("password", &self.password.is_some())
            .field("part_size", &self.part_size)
            .finish()
    }
}

/// What a selection of files and folders holds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Measured {
    pub files: u64,
    pub bytes: u64,
}

/// What goes in the archive, with the name it has there.
struct Item {
    name: String,
    path: PathBuf,
    metadata: Metadata,
}

/// `path` and what is under it, its names under `name`, in order of name;
/// links are not followed, and what is neither a file, a folder nor a link
/// is left out.
fn walk(path: &Path, name: String, items: &mut Vec<Item>) -> Result<(), Error> {
    let read = |e: io::Error| Error::Failed(format!("Could not read {}: {e}", path.display()));
    let metadata = fs::symlink_metadata(path).map_err(read)?;
    let kind = metadata.file_type();
    if kind.is_dir() {
        let mut children = fs::read_dir(path)
            .map_err(read)?
            .collect::<io::Result<Vec<_>>>()
            .map_err(read)?;
        children.sort_by_key(|c| c.file_name());
        items.push(Item {
            name: format!("{name}/"),
            path: path.to_path_buf(),
            metadata,
        });
        for child in children {
            let under = format!("{name}/{}", unicode_name(&child.path())?);
            walk(&child.path(), under, items)?;
        }
    } else if kind.is_file() || (cfg!(unix) && kind.is_symlink()) {
        items.push(Item {
            name,
            path: path.to_path_buf(),
            metadata,
        });
    }
    Ok(())
}

/// The last part of `path`, which an archive names it by. A name that is not
/// Unicode is refused: changed to fit, it would come out as another file.
fn unicode_name(path: &Path) -> Result<String, Error> {
    let name = path
        .file_name()
        .ok_or_else(|| Error::Failed(format!("{} has no name.", path.display())))?;
    name.to_str().map(str::to_owned).ok_or_else(|| {
        Error::Failed(format!(
            "The name of {} is not valid Unicode, so it cannot go in a ZIP archive.",
            path.display()
        ))
    })
}

fn items_of(paths: &[PathBuf]) -> Result<Vec<Item>, Error> {
    let mut items = Vec::new();
    for path in paths {
        walk(path, unicode_name(path)?, &mut items)?;
    }
    Ok(items)
}

/// Counts the files in `paths`, and the bytes in them.
pub fn measure(paths: &[PathBuf]) -> Result<Measured, Error> {
    let mut measured = Measured::default();
    for item in items_of(paths)? {
        if !item.metadata.is_dir() {
            measured.files += 1;
            measured.bytes += item.metadata.len();
        }
    }
    Ok(measured)
}

fn seconds(metadata: &Metadata) -> i64 {
    match metadata.modified().map(|t| t.duration_since(UNIX_EPOCH)) {
        Ok(Ok(d)) => d.as_secs() as i64,
        Ok(Err(e)) => -(e.duration().as_secs() as i64),
        Err(_) => 0,
    }
}

/// The external attributes of `metadata`'s file, as a Unix zip writes them
/// (Windows' read-only bit in the low byte too), and the system they are of.
fn attributes(metadata: &Metadata) -> (u8, u32) {
    #[cfg(unix)]
    let mode = std::os::unix::fs::MetadataExt::mode(metadata);
    #[cfg(not(unix))]
    let mode: u32 = match (metadata.is_dir(), metadata.permissions().readonly()) {
        (true, _) => 0o040755,
        (false, true) => 0o100444,
        (false, false) => 0o100644,
    };
    (opsys::UNIX, (mode << 16) | u32::from(mode & 0o200 == 0))
}

/// How far the whole archive is, and what a file that fails says.
struct Tally {
    control: Control,
    progress: Mutex<Progress>,
    failed: Mutex<Option<String>>,
}

impl Tally {
    fn advance(&self, entries: u64, bytes: u64) {
        let mut progress = self.progress.lock().unwrap_or_else(PoisonError::into_inner);
        progress.entries += entries;
        progress.bytes += bytes;
        self.control.report(&progress);
    }
}

/// A file's data, for hivepack-zip: opened when it is first read and closed
/// at its end, as an archive may hold more files than can be open at once.
/// Reading it counts towards the progress, and is where a job stops.
struct DiskFile {
    path: PathBuf,
    size: u64,
    mtime: i64,
    attributes: (u8, u32),
    file: Mutex<Option<File>>,
    /// As far as it has been read: it may be read twice.
    reached: AtomicU64,
    tally: Arc<Tally>,
}

impl DiskFile {
    fn failed(&self, e: &io::Error) -> hivepack_zip::Error {
        *self
            .tally
            .failed
            .lock()
            .unwrap_or_else(PoisonError::into_inner) =
            Some(format!("Could not read {}: {e}", self.path.display()));
        hivepack_zip::Error::new(ErrorCode::Read, e.raw_os_error().unwrap_or(0))
    }
}

impl DataSource for DiskFile {
    fn stat(&self) -> hivepack_zip::Result<Stat<'static>> {
        Ok(Stat {
            valid: stat::SIZE | stat::MTIME,
            size: self.size,
            mtime: self.mtime,
            ..Stat::default()
        })
    }

    fn file_attributes(&self) -> hivepack_zip::Result<FileAttributes> {
        Ok(FileAttributes {
            valid: file_attributes::HOST_SYSTEM | file_attributes::EXTERNAL_FILE_ATTRIBUTES,
            host_system: self.attributes.0,
            external_file_attributes: self.attributes.1,
            ..FileAttributes::default()
        })
    }

    fn read_at(&self, offset: u64, buf: &mut [u8]) -> hivepack_zip::Result<usize> {
        if self.tally.control.check().is_err() {
            return Err(hivepack_zip::Error::new(ErrorCode::Cancelled, 0));
        }
        let left = self.size.saturating_sub(offset);
        let wanted = (buf.len() as u64).min(left) as usize;
        if wanted == 0 {
            return Ok(0);
        }
        let mut file = self.file.lock().unwrap_or_else(PoisonError::into_inner);
        if file.is_none() {
            *file = Some(File::open(&self.path).map_err(|e| self.failed(&e))?);
        }
        let Some(open) = file.as_ref() else {
            return Ok(0);
        };
        let n = hivepack_zip::ReadAt::read_at(&open, offset, &mut buf[..wanted])
            .map_err(|e| self.failed(&e))?;
        let end = offset + n as u64;
        let before = self.reached.fetch_max(end, Ordering::Relaxed);
        let finished = end >= self.size;
        if finished {
            *file = None;
        }
        drop(file);
        self.tally.advance(
            u64::from(finished && before < self.size),
            end.saturating_sub(before),
        );
        Ok(n)
    }
}

/// The archive's output, one file or several of `size` bytes each, numbered
/// as 7-Zip numbers them. One file at a time is open: there may be many.
struct Parts {
    base: PathBuf,
    size: u64,
    open: Option<(u64, File)>,
    /// How many have been made.
    made: u64,
    position: u64,
    length: u64,
}

impl Parts {
    fn path(&self, part: u64) -> PathBuf {
        part_path(&self.base, part)
    }

    /// The part at `index` open, made if it is the next.
    fn part(&mut self, index: u64) -> io::Result<&mut File> {
        if self.open.as_ref().is_none_or(|(i, _)| *i != index) {
            let mut options = OpenOptions::new();
            options.write(true);
            if index >= self.made {
                options.create_new(true);
            }
            let file = options.open(self.path(index))?;
            self.made = self.made.max(index + 1);
            self.open = Some((index, file));
        }
        match &mut self.open {
            Some((_, file)) => Ok(file),
            None => Err(io::Error::other("no part is open")),
        }
    }

    fn remove(&mut self) {
        self.open = None;
        for part in 0..self.made {
            let _ = fs::remove_file(self.path(part));
        }
    }
}

impl Write for Parts {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        let (index, within) = (self.position / self.size, self.position % self.size);
        let room = (self.size - within).min(buf.len() as u64) as usize;
        let file = self.part(index)?;
        file.seek(SeekFrom::Start(within))?;
        let n = file.write(&buf[..room])?;
        self.position += n as u64;
        self.length = self.length.max(self.position);
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        match &mut self.open {
            Some((_, file)) => file.flush(),
            None => Ok(()),
        }
    }
}

impl Seek for Parts {
    fn seek(&mut self, to: SeekFrom) -> io::Result<u64> {
        let position = match to {
            SeekFrom::Start(p) => Some(p),
            SeekFrom::Current(d) => self.position.checked_add_signed(d),
            SeekFrom::End(d) => self.length.checked_add_signed(d),
        };
        self.position =
            position.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "seek before 0"))?;
        Ok(self.position)
    }
}

/// `name.zip.001` for the part numbered 0.
fn part_path(base: &Path, part: u64) -> PathBuf {
    let mut name = base.as_os_str().to_owned();
    name.push(format!(".{:03}", part + 1));
    PathBuf::from(name)
}

/// `wanted`, or if that is taken `name (2).zip`, `name (3).zip`, …, the
/// first for which `take` makes something new.
fn free_name<T>(
    wanted: &Path,
    mut take: impl FnMut(&Path) -> io::Result<T>,
) -> Result<(PathBuf, T), Error> {
    let folder = wanted.parent().unwrap_or(Path::new(""));
    let stem = wanted.file_stem().unwrap_or_default().to_string_lossy();
    let extension = wanted.extension().map(|e| e.to_string_lossy());
    for n in 1..10_000 {
        let path = match (n, &extension) {
            (1, _) => wanted.to_path_buf(),
            (_, Some(e)) => folder.join(format!("{stem} ({n}).{e}")),
            (_, None) => folder.join(format!("{stem} ({n})")),
        };
        match take(&path) {
            Ok(taken) => return Ok((path, taken)),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
            Err(e) => {
                return Err(Error::Failed(format!(
                    "Could not write {}: {e}",
                    path.display()
                )));
            }
        }
    }
    Err(Error::Failed(format!(
        "{} and the names after it are taken.",
        wanted.display()
    )))
}

/// Writes `items` in the archive to `out`.
fn write_archive<W: Write + Seek>(
    items: Vec<Item>,
    new: &NewArchive,
    tally: &Arc<Tally>,
    out: W,
) -> Result<(), Error> {
    let mut za = Archive::open(&[][..], OpenFlags(0))?;
    za.set_random_source(Some(Arc::new(|buffer: &mut [u8]| {
        getrandom::fill(buffer).map_err(io::Error::other)
    })));
    if new.password.is_some() {
        za.set_default_password(new.password.as_deref());
    }
    for item in items {
        let mtime = seconds(&item.metadata);
        let (system, external) = attributes(&item.metadata);
        let name = item.name.as_bytes();
        let index = if item.metadata.is_dir() {
            za.dir_add(name, Flags::ENC_UTF_8)?
        } else if item.metadata.file_type().is_symlink() {
            let index = za.file_add(
                name,
                BufferSource::with_mtime(link_target(&item.path)?, mtime),
                Flags::ENC_UTF_8,
            )?;
            za.set_file_compression(index, i32::from(cm::STORE), 0)?;
            if new.password.is_some() {
                za.file_set_encryption(index, em::AES_256, None)?;
            }
            index
        } else {
            let index = za.file_add(
                name,
                DiskFile {
                    path: item.path,
                    size: item.metadata.len(),
                    mtime,
                    attributes: (system, external),
                    file: Mutex::new(None),
                    reached: AtomicU64::new(0),
                    tally: tally.clone(),
                },
                Flags::ENC_UTF_8,
            )?;
            za.set_file_compression(index, cm::DEFAULT, new.level)?;
            if new.password.is_some() {
                za.file_set_encryption(index, em::AES_256, None)?;
            }
            index
        };
        za.file_set_mtime(index, mtime, Flags::NONE)?;
        za.file_set_external_attributes(index, Flags::NONE, system, external)?;
    }
    match za.close_to(out)? {
        Closed::Written => Ok(()),
        Closed::Unchanged | Closed::Removed => Err(Error::Failed("Nothing to compress.".into())),
    }
}

#[cfg(unix)]
fn link_target(path: &Path) -> Result<Vec<u8>, Error> {
    use std::os::unix::ffi::OsStringExt;
    fs::read_link(path)
        .map(|target| target.into_os_string().into_vec())
        .map_err(|e| Error::Failed(format!("Could not read {}: {e}", path.display())))
}

#[cfg(not(unix))]
fn link_target(path: &Path) -> Result<Vec<u8>, Error> {
    Err(Error::Failed(format!(
        "{} is a link, which is kept only on Unix.",
        path.display()
    )))
}

/// Makes a ZIP archive of `paths`, each with what is under it, named after
/// `wanted` or the first name after it that is free, and gives where it
/// went: the archive, or its first part if it took more than one. The names
/// in it start from each path's own.
pub fn create_zip(
    paths: &[PathBuf],
    wanted: &Path,
    new: &NewArchive,
    control: &Control,
) -> Result<PathBuf, Error> {
    let items = items_of(paths)?;
    if items.is_empty() {
        return Err(Error::Failed("Nothing to compress.".into()));
    }
    // Before the archive is made, so that it is not among them.
    let mut progress = Progress::default();
    for item in &items {
        if !item.metadata.is_dir() {
            progress.total_entries += 1;
            progress.total_bytes += item.metadata.len();
        }
    }
    let tally = Arc::new(Tally {
        control: control.clone(),
        progress: Mutex::new(progress),
        failed: Mutex::new(None),
    });
    control.report(&progress);
    let why = |result: Result<(), Error>, tally: &Tally| match result {
        Err(_) if control.check().is_err() => Err(Error::Cancelled),
        Err(e) => {
            let failed = tally.failed.lock().unwrap_or_else(PoisonError::into_inner);
            Err(failed.clone().map_or(e, Error::Failed))
        }
        ok => ok,
    };
    match new.part_size {
        Some(size) if size > 0 => {
            let (base, mut parts) = free_name(wanted, |base| {
                let mut parts = Parts {
                    base: base.to_path_buf(),
                    size,
                    open: None,
                    made: 0,
                    position: 0,
                    length: 0,
                };
                if base.exists() {
                    return Err(io::ErrorKind::AlreadyExists.into());
                }
                parts.part(0)?;
                Ok(parts)
            })?;
            let written = write_archive(items, new, &tally, &mut parts).and_then(|()| {
                parts.flush()?;
                Ok(())
            });
            if let Err(e) = why(written, &tally) {
                parts.remove();
                return Err(e);
            }
            parts.open = None;
            // It fitted in one: an archive like any other.
            if parts.made == 1 && !base.exists() {
                fs::rename(parts.path(0), &base)?;
                return Ok(base);
            }
            Ok(parts.path(0))
        }
        _ => {
            let (path, mut file) = free_name(wanted, |path| {
                OpenOptions::new().write(true).create_new(true).open(path)
            })?;
            let written = write_archive(items, new, &tally, &mut file).and_then(|()| {
                file.sync_all()?;
                Ok(())
            });
            if let Err(e) = why(written, &tally) {
                drop(file);
                let _ = fs::remove_file(&path);
                return Err(e);
            }
            Ok(path)
        }
    }
}
