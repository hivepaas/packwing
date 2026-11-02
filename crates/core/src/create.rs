//! Making an archive of files and folders: ZIP through hivepack-zip, 7z
//! through hivepack-7z, and tar through hivepack-tar, under gzip, xz, zstd, bzip2
//! or LZ4 or none; or one file compressed by itself. Those two can be encrypted
//! with age, to a passphrase or to keys.
//! It is written beside what is there, never over it, and taken away again
//! if it fails or is stopped. Links are kept as links, on Unix; what is not a
//! file, a folder or a link (a pipe, a socket, a device) is left out, as
//! reading it could wait for ever. A tar archive's owners are left out too:
//! its files may go to someone with other users.
//!
//! Files are added to a ZIP archive by writing it anew beside itself, read
//! back, and only then put in its place: if anything goes wrong, it is as it
//! was.

use packwing_i18n::tr;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs::{self, File, Metadata, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::UNIX_EPOCH;

use hivepack_age::{
    BoxedRecipient, Recipient, ScryptRecipient, parse_recipient, parse_recipients_file,
};
use hivepack_deflate::{DEF_MEM_LEVEL, Deflate, Flush, Z_DEFAULT_STRATEGY};
use hivepack_extract::Progress;
use hivepack_open::Container;
use hivepack_xz::{MtOptions, Stream, stream_encoder_mt_memusage};
use hivepack_zip::{
    Archive, BufferSource, Closed, DataSource, ErrorCode, FileAttributes, Flags, OpenFlags, ReadAt,
    Stat, cm, em, file_attributes, opsys, stat, volume_paths,
};

use crate::job::opened_subfolder;
use crate::storage::{Item as StoredItem, Storage};
use crate::transfer::Watched;
use crate::{Control, Error, Keys, Listing, opener};

/// A kind of archive packwing makes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Kind {
    #[default]
    Zip,
    SevenZip,
    Tar,
    TarGz,
    TarXz,
    TarZst,
    TarBz2,
    TarLz4,
    /// One file compressed by itself, with no archive around it.
    Gz,
    Xz,
    Zst,
    Bz2,
    Lz4,
}

impl Kind {
    /// Whether it holds one file, with no archive around it.
    pub fn one_file(self) -> bool {
        matches!(
            self,
            Kind::Gz | Kind::Xz | Kind::Zst | Kind::Bz2 | Kind::Lz4
        )
    }
}

/// How hard the files are squeezed: higher makes a smaller archive, which
/// takes longer to make, not to open.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Level {
    Lowest,
    Low,
    #[default]
    Medium,
    High,
    Highest,
}

impl Level {
    /// The level Deflate is asked for: zlib's own default in the middle.
    fn deflate(self) -> u32 {
        match self {
            Level::Lowest => 1,
            Level::Low => 3,
            Level::Medium => 6,
            Level::High => 8,
            Level::Highest => 9,
        }
    }

    /// The preset of LZMA and LZMA2: xz's own default in the middle.
    fn xz(self) -> u32 {
        match self {
            Level::Lowest => 1,
            Level::Low => 3,
            Level::Medium => 6,
            Level::High => 7,
            Level::Highest => 9,
        }
    }

    /// The level zstd is asked for: 19 is as far as the `zstd` tool goes
    /// without `--ultra`, whose archives take far more memory to open.
    /// bzip2's block size, in 100 KB: most of what it can find is within one.
    fn bzip2(self) -> i32 {
        match self {
            Level::Lowest => 1,
            Level::Low => 5,
            Level::Medium | Level::High | Level::Highest => 9,
        }
    }

    /// LZ4's level: its fast way up to the middle, as speed is why it is chosen.
    fn lz4(self) -> i32 {
        match self {
            Level::Lowest => -1,
            Level::Low | Level::Medium => 1,
            Level::High => 9,
            Level::Highest => 12,
        }
    }

    fn zstd(self) -> i32 {
        match self {
            Level::Lowest => 1,
            Level::Low => 3,
            Level::Medium => 9,
            Level::High => 15,
            Level::Highest => 19,
        }
    }
}

/// How a new archive is made.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct NewArchive {
    pub kind: Kind,
    pub level: Level,
    /// Encrypts its files with AES-256: WinZip's in a ZIP archive, 7-Zip's in
    /// a 7z one. Any other kind is encrypted whole with age, to this passphrase.
    pub password: Option<Vec<u8>>,
    /// Encrypts it whole with age, to these keys: `age1…` or `ssh-ed25519 …`. Not for
    /// a ZIP or 7z archive, and not with a password, as age has a passphrase alone.
    pub age_keys: Vec<String>,
    /// Encrypts a 7z archive's names too, with its password.
    pub hide_names: bool,
    /// Leaves out what only a Mac uses, under the paths chosen: `.DS_Store`,
    /// `__MACOSX` and the `._` files of resource forks.
    pub skip_mac_files: bool,
    /// Cuts it into `name.zip.001`, `name.zip.002`, … of this many bytes, as
    /// 7-Zip does.
    pub part_size: Option<u64>,
    /// Bound for a storage across a network, it goes straight there if its files
    /// come to more than this many bytes and the storage can take it that way. It
    /// goes through a temporary file otherwise, and if this is none.
    pub stream_above: Option<u64>,
}

/// How a new archive reaches a storage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Route {
    /// Made where it goes, in a folder on this computer.
    Here,
    /// Made in a temporary file, then uploaded.
    Temporary,
    /// Written to the storage as it is made.
    Straight,
}

/// How a new archive of `bytes` of files reaches `folder` of `to`.
pub fn route(to: &dyn Storage, folder: &[String], new: &NewArchive, bytes: u64) -> Route {
    if to.local_path(folder).is_some() {
        return Route::Here;
    }
    // A tar or ZIP archive can be written going only forwards. A 7z one goes back to
    // its start, and parts are files of their own, which are not worth the trouble yet.
    let forwards = new.kind != Kind::SevenZip;
    let can = new.part_size.is_none() && (to.writes_anywhere() || (forwards && to.sends_as_read()));
    match new.stream_above {
        Some(above) if can && bytes > above => Route::Straight,
        _ => Route::Temporary,
    }
}

impl fmt::Debug for NewArchive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NewArchive")
            .field("kind", &self.kind)
            .field("level", &self.level)
            .field("password", &self.password.is_some())
            .field("hide_names", &self.hide_names)
            .field("stream_above", &self.stream_above)
            .field("skip_mac_files", &self.skip_mac_files)
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
fn walk(path: &Path, name: String, items: &mut Vec<Item>, control: &Control) -> Result<(), Error> {
    control.check()?;
    let read = |e: io::Error| {
        Error::Failed(tr!(
            "Could not read {path}: {e}",
            path = path.display(),
            e = e
        ))
    };
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
            walk(&child.path(), under, items, control)?;
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
        .ok_or_else(|| Error::Failed(tr!("{path} has no name.", path = path.display())))?;
    name.to_str().map(str::to_owned).ok_or_else(|| {
        Error::Failed(tr!(
            "The name of {path} is not valid Unicode, so it cannot go in an archive.",
            path = path.display()
        ))
    })
}

fn items_of(paths: &[PathBuf], control: &Control) -> Result<Vec<Item>, Error> {
    let mut items = Vec::new();
    for path in paths {
        walk(path, unicode_name(path)?, &mut items, control)?;
    }
    Ok(items)
}

/// Whether `name`, in an archive, is what only a Mac uses. A path chosen by
/// name is that path's own business: only what is under it is looked at.
fn is_mac_file(name: &str) -> bool {
    let mut parts = name.trim_end_matches('/').split('/').skip(1);
    parts.any(|part| part == ".DS_Store" || part == "__MACOSX" || part.starts_with("._"))
}

/// Counts the files in `paths`, and the bytes in them.
pub fn measure(paths: &[PathBuf]) -> Result<Measured, Error> {
    Ok(measured(&items_of(paths, &Control::default())?))
}

/// The files in `items`, links too, and the bytes in the files: a link's
/// target is not read as a file is.
fn measured(items: &[Item]) -> Measured {
    let mut measured = Measured::default();
    for item in items {
        if !item.metadata.is_dir() {
            measured.files += 1;
        }
        if item.metadata.is_file() {
            measured.bytes += item.metadata.len();
        }
    }
    measured
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
    let directory = if metadata.is_dir() { 0x10 } else { 0 };
    (
        opsys::UNIX,
        (mode << 16) | directory | u32::from(mode & 0o200 == 0),
    )
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
            .unwrap_or_else(PoisonError::into_inner) = Some(tr!(
            "Could not read {path}: {e}",
            path = self.path.display(),
            e = e
        ));
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

/// A file's name, split before what names its kind: `a.tar` and `.gz` would
/// make `a.tar (2).gz`.
fn name_and_extension(name: &str) -> (&str, &str) {
    // `.age` follows what it encrypts: `a (2).tar.zst.age`, `b.sql (2).zst.age`.
    let plain = name
        .strip_suffix(".age")
        .filter(|p| !p.is_empty())
        .unwrap_or(name);
    let dot = plain.rfind(".tar.").or_else(|| plain.rfind('.'));
    name.split_at(dot.filter(|d| *d > 0).unwrap_or(plain.len()))
}

/// `wanted`, or if that is taken `name (2).zip`, `name (3).zip`, …, the
/// first for which `take` makes something new.
fn free_name<T>(
    wanted: &Path,
    mut take: impl FnMut(&Path) -> io::Result<T>,
) -> Result<(PathBuf, T), Error> {
    let folder = wanted.parent().unwrap_or(Path::new(""));
    let name = wanted.file_name().unwrap_or_default().to_string_lossy();
    let (stem, extension) = name_and_extension(&name);
    for n in 1..10_000 {
        let path = match n {
            1 => wanted.to_path_buf(),
            _ => folder.join(format!("{stem} ({n}){extension}")),
        };
        match take(&path) {
            Ok(taken) => return Ok((path, taken)),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
            Err(e) => {
                return Err(Error::Failed(tr!(
                    "Could not write {path}: {e}",
                    path = path.display(),
                    e = e
                )));
            }
        }
    }
    Err(Error::Failed(tr!(
        "{wanted} and the names after it are taken.",
        wanted = wanted.display()
    )))
}

/// Writes `items` in the archive to `out`.
fn write_archive<W: Write + Seek>(
    items: Vec<Item>,
    new: &NewArchive,
    tally: &Arc<Tally>,
    out: W,
) -> Result<(), Error> {
    match new.kind {
        Kind::Zip => write_zip(items, new, tally, out),
        Kind::SevenZip => write_7z(items, new, tally, out),
        _ => write_squeezed(items, new, tally, out),
    }
}

fn write_zip<W: Write + Seek>(
    items: Vec<Item>,
    new: &NewArchive,
    tally: &Arc<Tally>,
    out: W,
) -> Result<(), Error> {
    zip_closed(new_zip(items, new, tally)?.close_to(out)?)
}

/// What closing a new ZIP archive came to.
fn zip_closed(closed: Closed) -> Result<(), Error> {
    match closed {
        Closed::Written => Ok(()),
        Closed::Unchanged | Closed::Removed => Err(Error::Failed(tr!("Nothing to compress."))),
    }
}

/// A new ZIP archive of `items`, to close.
fn new_zip(
    items: Vec<Item>,
    new: &NewArchive,
    tally: &Arc<Tally>,
) -> Result<Archive<&'static [u8]>, Error> {
    let mut za = Archive::open(&[][..], OpenFlags(0))?;
    za.set_random_source(Some(Arc::new(|buffer: &mut [u8]| {
        getrandom::fill(buffer).map_err(io::Error::other)
    })));
    if new.password.is_some() {
        za.set_default_password(new.password.as_deref());
    }
    add_items(&mut za, items, new.level, new.password.is_some(), tally)?;
    Ok(za)
}

/// Writes a tar archive of `items`, or the one file that is, to `out`, compressed as
/// `new` says.
fn write_squeezed<W: Write>(
    items: Vec<Item>,
    new: &NewArchive,
    tally: &Tally,
    out: W,
) -> Result<(), Error> {
    let Some(recipients) = age_recipients(new)? else {
        squeeze(items, new, tally, out)?;
        return Ok(());
    };
    let recipients: Vec<&dyn Recipient> = recipients
        .iter()
        .map(|r| r.as_ref() as &dyn Recipient)
        .collect();
    let encrypted = hivepack_age::encrypt(out, &recipients, None)
        .map_err(|e| Error::Failed(tr!("Could not encrypt: {e}", e = e.to_string())))?;
    squeeze(items, new, tally, encrypted)?.finish()?;
    Ok(())
}

/// What a new archive is encrypted to with age, if it is: none for ZIP and 7z, which
/// have their own encryption.
fn age_recipients(new: &NewArchive) -> Result<Option<Vec<BoxedRecipient>>, Error> {
    let bad = |e: String| Error::Failed(tr!("Could not encrypt: {e}", e = e));
    if matches!(new.kind, Kind::Zip | Kind::SevenZip) {
        if !new.age_keys.is_empty() {
            return Err(Error::Failed(tr!(
                "Only tar and single files take age keys."
            )));
        }
        return Ok(None);
    }
    match (&new.password, new.age_keys.as_slice()) {
        (None, []) => Ok(None),
        (Some(_), [_, ..]) => Err(Error::Failed(tr!(
            "age encrypts to a passphrase or to keys, not to both."
        ))),
        (Some(password), []) => {
            let recipient = ScryptRecipient::new(password).map_err(bad)?;
            Ok(Some(vec![Box::new(recipient)]))
        }
        (None, keys) => keys
            .iter()
            .map(|key| parse_recipient(key).map_err(bad))
            .collect::<Result<Vec<_>, _>>()
            .map(Some),
    }
}

/// Why `key` is not one to encrypt to with age, if it is not.
pub fn age_key_problem(key: &str) -> Option<String> {
    parse_recipient(key).err()
}

/// The keys in a file of them, as `age -R` reads it, and how many it left out, of a
/// kind age does not encrypt to.
pub fn age_keys_in(file: &[u8]) -> Result<(Vec<String>, usize), String> {
    let read = parse_recipients_file(file)?;
    let left_out: HashSet<usize> = read.skipped.iter().map(|k| k.line).collect();
    let text = String::from_utf8_lossy(file);
    let keys: Vec<String> = text
        .lines()
        .enumerate()
        .filter(|(i, line)| {
            !line.is_empty() && !line.starts_with('#') && !left_out.contains(&(i + 1))
        })
        .map(|(_, line)| line.to_owned())
        .collect();
    if keys.is_empty() {
        return Err(tr!("There are no keys in this file to encrypt to."));
    }
    Ok((keys, left_out.len()))
}

/// Writes a tar archive of `items`, or the one file that is, to `out`, compressed as
/// `new` says, and gives `out` back.
fn squeeze<W: Write>(
    items: Vec<Item>,
    new: &NewArchive,
    tally: &Tally,
    out: W,
) -> Result<W, Error> {
    let mut out = Squeezed::new(new, out)?;
    if !new.kind.one_file() {
        return write_tar(items, tally, out);
    }
    let [item] = items.as_slice() else {
        return Err(Error::Failed(tr!(
            "This format holds one file, and no more."
        )));
    };
    if !item.metadata.file_type().is_file() {
        return Err(Error::Failed(tr!(
            "This format holds one file, and no more."
        )));
    }
    pump(item, &mut out, tally, &mut vec![0; PIECE])?;
    Ok(out.finish()?)
}

/// `items` in the ZIP archive `za`, to be read when it closes.
fn add_items<R: ReadAt>(
    za: &mut Archive<R>,
    items: Vec<Item>,
    level: Level,
    encrypt: bool,
    tally: &Arc<Tally>,
) -> Result<(), Error> {
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
            if encrypt {
                za.file_set_encryption(index, em::AES_256, None)?;
            }
            tally.advance(1, 0);
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
            za.set_file_compression(index, cm::DEFAULT, level.deflate())?;
            // Nothing to read, so nothing would count it.
            if item.metadata.len() == 0 {
                tally.advance(1, 0);
            }
            if encrypt {
                za.file_set_encryption(index, em::AES_256, None)?;
            }
            index
        };
        za.file_set_mtime(index, mtime, Flags::NONE)?;
        za.file_set_external_attributes(index, Flags::NONE, system, external)?;
    }
    Ok(())
}

/// The data of `item`, a file, into `out`, a piece at a time: the size it had
/// when it was counted, as a tar archive has said already by then. Reading it
/// counts towards the progress, and is where a job stops.
fn pump(item: &Item, out: &mut impl Write, tally: &Tally, buf: &mut [u8]) -> Result<(), Error> {
    let read = |e: io::Error| {
        Error::Failed(tr!(
            "Could not read {path}: {e}",
            path = item.path.display(),
            e = e
        ))
    };
    let mut file = File::open(&item.path).map_err(read)?;
    let mut left = item.metadata.len();
    while left > 0 {
        tally.control.check()?;
        let wanted = left.min(buf.len() as u64) as usize;
        match file.read(&mut buf[..wanted]) {
            Ok(0) => {
                return Err(Error::Failed(tr!(
                    "{path} got shorter while it was being read.",
                    path = item.path.display()
                )));
            }
            Ok(n) => {
                out.write_all(&buf[..n])?;
                left -= n as u64;
                tally.advance(0, n as u64);
            }
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(read(e)),
        }
    }
    tally.advance(1, 0);
    Ok(())
}

/// A time as 7z has it: in 100 ns since 1601, which an earlier one cannot be.
fn ntfs_time(metadata: &Metadata) -> Option<u64> {
    let since_1601 = seconds(metadata).checked_add(11_644_473_600)?;
    u64::try_from(since_1601).ok()?.checked_mul(10_000_000)
}

fn write_7z<W: Write + Seek>(
    items: Vec<Item>,
    new: &NewArchive,
    tally: &Tally,
    out: W,
) -> Result<(), Error> {
    let password = match &new.password {
        None => None,
        Some(p) => Some(
            String::from_utf8(p.clone())
                .map_err(|_| Error::Failed(tr!("A password has to be valid Unicode.")))?,
        ),
    };
    let options = hivepack_7z::WriteOptions {
        method: hivepack_7z::Method::Lzma2 {
            preset: new.level.xz(),
        },
        encrypt_headers: new.hide_names && password.is_some(),
        password,
        random_source: Some(Arc::new(|buffer: &mut [u8]| {
            getrandom::fill(buffer).map_err(io::Error::other)
        })),
        ..hivepack_7z::WriteOptions::default()
    };
    let mut archive = hivepack_7z::Writer::with_options(out, options)?;
    let mut buf = vec![0; PIECE];
    for item in &items {
        // 7-Zip keeps a Unix mode above its own attributes, marked by 0x8000.
        let attributes = attributes(&item.metadata).1 | 0x8000;
        let entry = hivepack_7z::NewEntry {
            name: item.name.trim_end_matches('/').to_owned(),
            attributes: Some(attributes),
            mtime: ntfs_time(&item.metadata),
            ctime: None,
        };
        if item.metadata.is_dir() {
            archive.add_directory(entry)?;
        } else if item.metadata.file_type().is_symlink() {
            archive.start_file(entry)?;
            archive.write_all(&link_target(&item.path)?)?;
            tally.advance(1, 0);
        } else {
            archive.start_file(entry)?;
            pump(item, &mut archive, tally, &mut buf)?;
        }
    }
    archive.finish()?;
    Ok(())
}

fn write_tar<W: Write>(items: Vec<Item>, tally: &Tally, out: Squeezed<W>) -> Result<W, Error> {
    let mut tar = hivepack_tar::Writer::new(out);
    let mut buf = vec![0; PIECE];
    for item in &items {
        let kind = item.metadata.file_type();
        let link = kind.is_symlink();
        let mut header = hivepack_tar::Header {
            name: item.name.as_bytes().to_vec(),
            mode: i64::from((attributes(&item.metadata).1 >> 16) & 0o7777),
            mod_time: Some(hivepack_tar::Time {
                secs: seconds(&item.metadata),
                nanos: 0,
            }),
            ..hivepack_tar::Header::default()
        };
        if kind.is_dir() {
            header.typeflag = hivepack_tar::TYPE_DIR;
        } else if link {
            header.typeflag = hivepack_tar::TYPE_SYMLINK;
            header.linkname = link_target(&item.path)?;
        } else {
            header.typeflag = hivepack_tar::TYPE_REG;
            header.size = i64::try_from(item.metadata.len()).map_err(|_| {
                Error::Failed(tr!("{path} is too big.", path = item.path.display()))
            })?;
        }
        tar.write_header(&header)?;
        if link {
            tally.advance(1, 0);
        } else if !kind.is_dir() {
            pump(item, &mut tar, tally, &mut buf)?;
        }
    }
    tar.close()?;
    Ok(tar.into_inner().finish()?)
}

/// How much is read of a file at a time, and made of what compresses.
const PIECE: usize = 1 << 16;

/// The most xz's threads may take between them, in bytes. Each takes about
/// three dictionaries, and the highest level's are 64 MiB.
const XZ_MEMORY: u64 = 4 << 30;

/// What compresses a tar archive.
enum Coder {
    /// Written as it is.
    Plain,
    Gzip(Box<Deflate>),
    Xz(Box<Stream>),
    Zstd(Box<hivepack_zstd::CCtx>),
    Bzip2(Box<hivepack_bzip2::Compress>),
    /// What it makes waits in its `Vec`. None once it has ended.
    Lz4(Option<Box<hivepack_lz4::Encoder<Vec<u8>>>>),
}

/// A stream compressed into `out`, whose end [`Squeezed::finish`] writes.
struct Squeezed<W> {
    out: W,
    coder: Coder,
    buf: Vec<u8>,
}

/// As many threads as the machine has.
fn threads() -> u32 {
    std::thread::available_parallelism().map_or(1, |n| n.get() as u32)
}

impl<W: Write> Squeezed<W> {
    fn new(new: &NewArchive, out: W) -> Result<Squeezed<W>, Error> {
        let failed = |e: String| Error::Failed(tr!("Could not start compressing: {e}", e = e));
        let coder = match new.kind {
            Kind::Tar => Coder::Plain,
            Kind::TarBz2 | Kind::Bz2 => Coder::Bzip2(Box::new(
                hivepack_bzip2::Compress::new(new.level.bzip2(), 0)
                    .map_err(|e| failed(e.to_string()))?,
            )),
            Kind::TarLz4 | Kind::Lz4 => {
                let prefs = hivepack_lz4::frame::Preferences {
                    compression_level: new.level.lz4(),
                    ..Default::default()
                };
                Coder::Lz4(Some(Box::new(
                    hivepack_lz4::Encoder::new(Vec::new(), Some(&prefs))
                        .map_err(|e| failed(e.to_string()))?,
                )))
            }
            Kind::TarXz | Kind::Xz => {
                let mut options = MtOptions {
                    threads: threads(),
                    preset: new.level.xz(),
                    check: hivepack_xz::Check::CRC64,
                    ..MtOptions::default()
                };
                while options.threads > 1 && stream_encoder_mt_memusage(&options) > XZ_MEMORY {
                    options.threads -= 1;
                }
                Coder::Xz(Box::new(
                    Stream::stream_encoder_mt(&options).map_err(|e| failed(e.to_string()))?,
                ))
            }
            Kind::TarZst | Kind::Zst => {
                let mut cctx = hivepack_zstd::CCtx::new();
                for parameter in [
                    hivepack_zstd::CParameter::CompressionLevel(new.level.zstd()),
                    hivepack_zstd::CParameter::ChecksumFlag(true),
                    hivepack_zstd::CParameter::NbWorkers(threads()),
                ] {
                    cctx.set_parameter(parameter)
                        .map_err(|e| failed(e.to_string()))?;
                }
                Coder::Zstd(Box::new(cctx))
            }
            // 16 more than the window's 15 asks for gzip's header and end.
            _ => Coder::Gzip(Box::new(
                Deflate::new(
                    new.level.deflate() as i32,
                    15 + 16,
                    DEF_MEM_LEVEL,
                    Z_DEFAULT_STRATEGY,
                )
                .map_err(|e| failed(e.to_string()))?,
            )),
        };
        Ok(Squeezed {
            out,
            coder,
            buf: vec![0; PIECE],
        })
    }

    /// Compresses `input`, and with `finish`, ends the stream.
    fn code(&mut self, input: &[u8], finish: bool) -> io::Result<()> {
        let failed = |e: String| io::Error::other(tr!("Could not compress: {e}", e = e));
        match &mut self.coder {
            Coder::Plain => return self.out.write_all(input),
            Coder::Lz4(encoder) => {
                let waiting = match encoder {
                    Some(going) => {
                        going.write_all(input)?;
                        if finish {
                            match encoder.take() {
                                Some(ending) => ending.finish()?,
                                None => Vec::new(),
                            }
                        } else {
                            std::mem::take(going.get_mut())
                        }
                    }
                    None => Vec::new(),
                };
                return self.out.write_all(&waiting);
            }
            _ => {}
        }
        let mut at = 0;
        loop {
            let (made, done) = match &mut self.coder {
                Coder::Plain | Coder::Lz4(_) => return Ok(()),
                Coder::Bzip2(stream) => {
                    use hivepack_bzip2::{Action, InBuffer, OutBuffer, Status};
                    if !finish && input.is_empty() {
                        return Ok(());
                    }
                    let mut output = OutBuffer::new(&mut self.buf);
                    let mut from = InBuffer::new(&input[at..]);
                    let action = if finish { Action::Finish } else { Action::Run };
                    let status = stream
                        .compress(&mut output, &mut from, action)
                        .map_err(|e| failed(e.to_string()))?;
                    at += from.pos;
                    let done = status == Status::StreamEnd || (!finish && at == input.len());
                    (output.pos, done)
                }
                Coder::Gzip(stream) => {
                    use hivepack_deflate::{InBuffer, OutBuffer, Status};
                    let mut output = OutBuffer::new(&mut self.buf);
                    let mut from = InBuffer::new(&input[at..]);
                    let flush = if finish {
                        Flush::Finish
                    } else {
                        Flush::NoFlush
                    };
                    let status = stream.deflate(&mut output, &mut from, flush);
                    at += from.pos;
                    let (made, full) = (output.pos, output.pos == output.dst.len());
                    let status = match status {
                        Ok(status) => status,
                        // Nothing to do: no more input, and nothing held back.
                        Err(e) if !finish && e.code() == hivepack_deflate::ErrorCode::Buf => {
                            Status::Ok
                        }
                        Err(e) => return Err(failed(e.to_string())),
                    };
                    let done =
                        status == Status::StreamEnd || (!finish && at == input.len() && !full);
                    (made, done)
                }
                Coder::Xz(stream) => {
                    use hivepack_xz::{Action, InBuffer, OutBuffer, Status};
                    let mut output = OutBuffer::new(&mut self.buf);
                    let mut from = InBuffer::new(&input[at..]);
                    let action = if finish { Action::Finish } else { Action::Run };
                    let status = stream
                        .code(&mut output, &mut from, action)
                        .map_err(|e| failed(e.to_string()))?;
                    at += from.pos;
                    let (made, full) = (output.pos, output.pos == output.dst.len());
                    let done =
                        status == Status::StreamEnd || (!finish && at == input.len() && !full);
                    (made, done)
                }
                Coder::Zstd(cctx) => {
                    use hivepack_zstd::{EndDirective, InBuffer, OutBuffer};
                    let mut output = OutBuffer::new(&mut self.buf);
                    let mut from = InBuffer::new(&input[at..]);
                    let directive = if finish {
                        EndDirective::End
                    } else {
                        EndDirective::Continue
                    };
                    let left = cctx
                        .compress_stream2(&mut output, &mut from, directive)
                        .map_err(|e| failed(e.to_string()))?;
                    at += from.pos;
                    let done = if finish { left == 0 } else { at == input.len() };
                    (output.pos, done)
                }
            };
            self.out.write_all(&self.buf[..made])?;
            if done {
                return Ok(());
            }
        }
    }

    fn finish(mut self) -> io::Result<W> {
        self.code(&[], true)?;
        self.out.flush()?;
        Ok(self.out)
    }
}

impl<W: Write> Write for Squeezed<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.code(buf, false)?;
        Ok(buf.len())
    }

    /// The stream is not flushed, which would make it bigger.
    fn flush(&mut self) -> io::Result<()> {
        self.out.flush()
    }
}

#[cfg(unix)]
fn link_target(path: &Path) -> Result<Vec<u8>, Error> {
    use std::os::unix::ffi::OsStringExt;
    fs::read_link(path)
        .map(|target| target.into_os_string().into_vec())
        .map_err(|e| {
            Error::Failed(tr!(
                "Could not read {path}: {e}",
                path = path.display(),
                e = e
            ))
        })
}

#[cfg(not(unix))]
fn link_target(path: &Path) -> Result<Vec<u8>, Error> {
    Err(Error::Failed(tr!(
        "{path} is a link, which is kept only on Unix.",
        path = path.display()
    )))
}

/// What stopped a job, said as well as can be: that it was stopped, what could
/// not be read, or else the error it came to.
fn why(result: Result<(), Error>, tally: &Tally) -> Result<(), Error> {
    match result {
        Err(_) if tally.control.check().is_err() => Err(Error::Cancelled),
        Err(e) => {
            let failed = tally.failed.lock().unwrap_or_else(PoisonError::into_inner);
            Err(failed.clone().map_or(e, Error::Failed))
        }
        ok => ok,
    }
}

/// Whether files can be added to the archive listed: a ZIP archive that is one
/// file, under no layers. A 7z or tar archive would be written whole again, and
/// a compressed one is a stream that has no place to add to.
pub fn can_add(listing: &Listing) -> bool {
    listing.container == Container::Zip
        && listing.layers.is_empty()
        && volume_paths(&listing.folder.join(&listing.name)).len() == 1
}

/// What to do with a file whose name an archive holds already.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OnExisting {
    /// Stop, with nothing changed.
    #[default]
    Stop,
    Replace,
    /// Leave the one in the archive.
    Skip,
}

/// [`add_with`], stopping at a name the archive holds.
pub fn add(
    paths: &[PathBuf],
    archive: &Path,
    folder: &[String],
    keys: &Keys,
    control: &Control,
) -> Result<(), Error> {
    add_with(paths, archive, folder, OnExisting::Stop, keys, control)
}

/// Whether the archive at `path` is one that can be changed: a ZIP archive of
/// one file, under no layers.
fn can_change(archive: &Path, keys: &Keys) -> Result<bool, Error> {
    let source = crate::Source::from(archive);
    let opened = opener(&source, keys).open()?;
    Ok(opened.container() == Container::Zip
        && opened.layers().is_empty()
        && volume_paths(archive).len() == 1)
}

/// Writes `za`, with its changes, beside the archive at `archive`, reads it back
/// to see it holds `expected` entries, and puts it in the archive's place. If
/// anything fails, the archive is as it was.
fn replace_with<R: ReadAt>(
    mut za: Archive<R>,
    archive: &Path,
    expected: u64,
    control_result: impl FnOnce(Result<(), Error>) -> Result<(), Error>,
) -> Result<(), Error> {
    let name = archive.file_name().unwrap_or_default().to_string_lossy();
    let beside = archive.with_file_name(format!(".{name}.adding"));
    let (written_to, mut file) = free_name(&beside, |path| {
        OpenOptions::new().write(true).create_new(true).open(path)
    })?;
    let written = match za.close_to(&mut file) {
        Ok(Closed::Written) => Ok(()),
        Ok(Closed::Unchanged | Closed::Removed) => Err(Error::Failed(tr!("Nothing changed."))),
        Err(e) => Err(Error::from(e)),
    }
    .and_then(|()| {
        file.sync_all()?;
        drop(file);
        let back = Archive::open(File::open(&written_to)?, OpenFlags::CHECKCONS)?;
        if back.get_num_entries(Flags::NONE) != expected {
            return Err(Error::Failed(tr!("The archive did not come out whole.")));
        }
        drop(back);
        // Its own file goes before the new one takes its place, as Windows wants.
        drop(za);
        fs::set_permissions(&written_to, fs::metadata(archive)?.permissions())?;
        fs::rename(&written_to, archive)?;
        Ok(())
    });
    if let Err(e) = control_result(written) {
        let _ = fs::remove_file(&written_to);
        return Err(e);
    }
    Ok(())
}

/// The entries of `za` by their paths, a component at a time, with their indexes.
fn entries_of<R: ReadAt>(za: &Archive<R>) -> Result<Vec<(u64, Vec<String>, bool)>, Error> {
    let mut entries = Vec::new();
    for i in 0..za.get_num_entries(Flags::NONE) {
        let stat = za.stat_index(i, Flags::NONE)?;
        let name = String::from_utf8_lossy(stat.name.unwrap_or_default()).into_owned();
        let path = crate::listing::components(&name);
        entries.push((i, path, name.ends_with('/')));
    }
    Ok(entries)
}

/// Deletes `picked` from the ZIP archive at `archive`, files or folders by their
/// paths in it, each with what is under it. All of it cannot go: an archive of
/// nothing is not one.
pub fn delete(archive: &Path, picked: &[Vec<String>], keys: &Keys) -> Result<(), Error> {
    if !can_change(archive, keys)? {
        return Err(Error::Failed(tr!(
            "packwing can change only a ZIP archive that is one file."
        )));
    }
    let mut za = Archive::open(File::open(archive)?, OpenFlags::CHECKCONS)?;
    let entries = entries_of(&za)?;
    let gone: Vec<u64> = entries
        .iter()
        .filter(|(_, path, _)| picked.iter().any(|p| path.starts_with(p)))
        .map(|(i, _, _)| *i)
        .collect();
    if gone.is_empty() {
        return Err(Error::Failed(tr!("That is not in the archive.")));
    }
    if gone.len() == entries.len() {
        return Err(Error::Failed(tr!(
            "That is all the archive holds. Delete the archive itself instead."
        )));
    }
    for i in &gone {
        za.delete(*i)?;
    }
    replace_with(za, archive, (entries.len() - gone.len()) as u64, |r| r)
}

/// Renames the file or folder at `from` in the ZIP archive at `archive` to `to`,
/// in the same folder. A name there already stops it.
pub fn rename(archive: &Path, from: &[String], to: &str, keys: &Keys) -> Result<(), Error> {
    if !can_change(archive, keys)? {
        return Err(Error::Failed(tr!(
            "packwing can change only a ZIP archive that is one file."
        )));
    }
    if to.is_empty() || to.contains(['/', '\\']) || to == "." || to == ".." {
        return Err(Error::Failed(tr!("“{to}” cannot be a name.", to = to)));
    }
    let Some((_, parent)) = from.split_last() else {
        return Err(Error::Failed(tr!("That is not in the archive.")));
    };
    let mut target = parent.to_vec();
    target.push(to.to_owned());
    if target == from {
        return Ok(());
    }
    let mut za = Archive::open(File::open(archive)?, OpenFlags::CHECKCONS)?;
    let entries = entries_of(&za)?;
    if entries.iter().any(|(_, path, _)| path.starts_with(&target)) {
        return Err(Error::Failed(tr!(
            "“{to}” is already in that folder. Nothing was changed.",
            to = to
        )));
    }
    let mut renamed = 0;
    for (i, path, dir) in &entries {
        if let Some(rest) = path.strip_prefix(from) {
            let mut new_name = target.join("/");
            for part in rest {
                new_name.push('/');
                new_name.push_str(part);
            }
            if *dir {
                new_name.push('/');
            }
            za.file_rename(*i, new_name.as_bytes(), Flags::ENC_UTF_8)?;
            renamed += 1;
        }
    }
    if renamed == 0 {
        return Err(Error::Failed(tr!("That is not in the archive.")));
    }
    replace_with(za, archive, entries.len() as u64, |r| r)
}

/// Adds `paths`, each with what is under it, to the ZIP archive at `archive`,
/// in its folder `folder`. A file whose name it holds goes as `on_existing` says;
/// a file where it holds a folder, or the other way, always stops it. In an
/// archive with encrypted files, the new ones are encrypted too, with its
/// password. If this fails or is stopped, the archive is as it was.
pub fn add_with(
    paths: &[PathBuf],
    archive: &Path,
    folder: &[String],
    on_existing: OnExisting,
    keys: &Keys,
    control: &Control,
) -> Result<(), Error> {
    if !can_change(archive, keys)? {
        return Err(Error::Failed(tr!(
            "packwing can add files only to a ZIP archive that is one file."
        )));
    }
    let mut items = items_of(paths, control)?;
    let inside = archive.canonicalize()?;
    let mut prefix = folder.join("/");
    if !prefix.is_empty() {
        prefix.push('/');
    }
    for item in &mut items {
        if item.path.canonicalize().is_ok_and(|p| p == inside) {
            return Err(Error::Failed(tr!("An archive can’t go inside itself.")));
        }
        item.name.insert_str(0, &prefix);
    }

    let mut za = Archive::open(File::open(archive)?, OpenFlags::CHECKCONS)?;
    let (mut files, mut dirs) = (HashMap::new(), HashSet::new());
    let mut encrypted = None;
    for i in 0..za.get_num_entries(Flags::NONE) {
        let stat = za.stat_index(i, Flags::NONE)?;
        let name = String::from_utf8_lossy(stat.name.unwrap_or_default()).into_owned();
        if !name.ends_with('/') {
            if stat.encryption_method != em::NONE {
                encrypted = encrypted.or(Some(i));
            }
            files.insert(name.clone(), i);
        }
        // Its folders, which it may hold only by implication.
        let mut end = 0;
        while let Some(slash) = name[end..].find('/') {
            end += slash + 1;
            dirs.insert(name[..end - 1].to_owned());
        }
    }
    let mut new = Vec::new();
    let mut replaced = Vec::new();
    for item in items {
        let bare = item.name.trim_end_matches('/');
        let folder = item.metadata.is_dir();
        let taken = || {
            Err(Error::Failed(tr!(
                "“{bare}” is already in the archive. Nothing was changed.",
                bare = bare
            )))
        };
        // A file where there is a folder, or the other way, is never replaced.
        if (folder && files.contains_key(bare)) || (!folder && dirs.contains(bare)) {
            return taken();
        }
        if let Some(index) = files.get(bare) {
            match on_existing {
                OnExisting::Stop => return taken(),
                OnExisting::Skip => continue,
                OnExisting::Replace => replaced.push(*index),
            }
        }
        if !(folder && dirs.contains(bare)) {
            new.push(item);
        }
    }
    if new.is_empty() {
        return Ok(());
    }
    // The password that opens what is there, or new files would have another.
    if let Some(i) = encrypted {
        if keys.password.is_none() {
            return Err(Error::Locked {
                needs: vec![crate::PASSWORD.to_owned()],
            });
        }
        za.set_default_password(keys.password.as_deref());
        za.fopen_index(i, Flags::NONE)?;
    }
    za.set_random_source(Some(Arc::new(|buffer: &mut [u8]| {
        getrandom::fill(buffer).map_err(io::Error::other)
    })));

    let mut progress = Progress::default();
    let counted = measured(&new);
    progress.total_entries = counted.files;
    progress.total_bytes = counted.bytes;
    let tally = Arc::new(Tally {
        control: control.clone(),
        progress: Mutex::new(progress),
        failed: Mutex::new(None),
    });
    control.report(&progress);

    let expected = za.get_num_entries(Flags::NONE) + new.len() as u64 - replaced.len() as u64;
    for index in replaced {
        za.delete(index)?;
    }
    if let Err(e) = add_items(&mut za, new, Level::default(), encrypted.is_some(), &tally) {
        return why(Err(e), &tally);
    }
    replace_with(za, archive, expected, |written| why(written, &tally))
}

/// Makes an archive of `paths`, each with what is under it, named after
/// `wanted` or the first name after it that is free, and gives where it
/// went: the archive, or its first part if it took more than one. The names
/// in it start from each path's own.
pub fn create(
    paths: &[PathBuf],
    wanted: &Path,
    new: &NewArchive,
    control: &Control,
) -> Result<PathBuf, Error> {
    // Refused before anything is read.
    age_recipients(new)?;
    // Before the archive is made, so that it is not among them.
    create_of(chosen_items(paths, new, control)?, wanted, new, control)
}

/// What goes in a new archive of `paths`, walked once: what it holds, how much that
/// is and whether there is room for it all come from here. Stops when it is told to.
fn chosen_items(
    paths: &[PathBuf],
    new: &NewArchive,
    control: &Control,
) -> Result<Vec<Item>, Error> {
    let mut items = items_of(paths, control)?;
    if new.skip_mac_files {
        items.retain(|item| !is_mac_file(&item.name));
    }
    Ok(items)
}

/// What counts how far making an archive of `items` has come, told where it starts.
fn tally_of(items: &[Item], control: &Control) -> Arc<Tally> {
    let mut progress = Progress::default();
    let counted = measured(items);
    progress.total_entries = counted.files;
    progress.total_bytes = counted.bytes;
    control.report(&progress);
    Arc::new(Tally {
        control: control.clone(),
        progress: Mutex::new(progress),
        failed: Mutex::new(None),
    })
}

/// [`create`], its `items` found.
fn create_of(
    items: Vec<Item>,
    wanted: &Path,
    new: &NewArchive,
    control: &Control,
) -> Result<PathBuf, Error> {
    if items.is_empty() {
        return Err(Error::Failed(tr!("Nothing to compress.")));
    }
    let tally = tally_of(&items, control);
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

/// A folder that goes when this does, with what is in it.
struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// How much room making an archive of `bytes` may take: data that does not
/// compress is stored, with a little around it.
fn room_needed(bytes: u64) -> u64 {
    bytes.saturating_add(bytes / 64).saturating_add(1 << 20)
}

/// Why an archive that may take `need` bytes should not start in `folder`, with
/// `free` bytes free there.
fn no_room(need: u64, free: u64, folder: &Path) -> Option<Error> {
    (free < need).then(|| {
        Error::Failed(tr!(
            "Before it is uploaded, this archive is made in {folder}, where it may take up to {need}. Only {free} is free there.",
            folder = folder.display(),
            need = crate::size(need),
            free = crate::size(free)
        ))
    })
}

/// `name`, or the first name after it that is free in `folder` of `to`: for an
/// archive split into parts, free for the archive and for its first part.
fn free_name_on(
    to: &dyn Storage,
    folder: &[String],
    name: &str,
    parts: bool,
) -> Result<String, Error> {
    let taken = |name: &str| to.stat(folder, name).map(|found| found.is_some());
    let (free, ()) = free_name(Path::new(name), |path| {
        let tried = path.to_string_lossy();
        if taken(&tried)? || (parts && taken(&part_path(path, 0).to_string_lossy())?) {
            return Err(io::ErrorKind::AlreadyExists.into());
        }
        Ok(())
    })?;
    Ok(free.to_string_lossy().into_owned())
}

/// Most of what is written to [`Piped`] waits for the upload in this many pieces, of
/// [`PIPED_PIECE`] each: how far compressing runs ahead of it.
const PIPED_PIECES: usize = 8;
const PIPED_PIECE: usize = 1 << 20;

/// What an archive is written to, on its way to [`PipeReader`].
struct Piped {
    to: SyncSender<io::Result<Vec<u8>>>,
    piece: Vec<u8>,
    /// The upload has stopped reading.
    broken: Arc<AtomicBool>,
}

impl Piped {
    fn send(&mut self) -> io::Result<()> {
        let piece = std::mem::replace(&mut self.piece, Vec::with_capacity(PIPED_PIECE));
        if self.to.send(Ok(piece)).is_err() {
            self.broken.store(true, Ordering::Relaxed);
            return Err(io::ErrorKind::BrokenPipe.into());
        }
        Ok(())
    }
}

impl Write for Piped {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = buf.len().min(PIPED_PIECE - self.piece.len());
        self.piece.extend_from_slice(&buf[..n]);
        if self.piece.len() == PIPED_PIECE {
            self.send()?;
        }
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for Piped {
    /// The rest. An upload that has stopped says why itself.
    fn drop(&mut self) {
        if !self.piece.is_empty() {
            let _ = self.send();
        }
    }
}

/// What the upload reads, as it is written to [`Piped`].
struct PipeReader {
    from: Receiver<io::Result<Vec<u8>>>,
    piece: Vec<u8>,
    taken: usize,
}

impl Read for PipeReader {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if self.taken == self.piece.len() {
            match self.from.recv() {
                Ok(piece) => self.piece = piece?,
                // All of it was written.
                Err(_) => return Ok(0),
            }
            self.taken = 0;
        }
        let n = out.len().min(self.piece.len() - self.taken);
        out[..n].copy_from_slice(&self.piece[self.taken..self.taken + n]);
        self.taken += n;
        Ok(n)
    }
}

/// Writes an archive of `items` straight to `path` of `to`, as [`route`] says it can.
fn straight(
    items: Vec<Item>,
    bytes: u64,
    to: &dyn Storage,
    path: &[String],
    name: &str,
    new: &NewArchive,
    control: &Control,
) -> Result<(), Error> {
    let upload_failed = |e: io::Error| Error::Failed(e.to_string());
    let like = StoredItem {
        name: name.to_owned(),
        dir: false,
        link: false,
        hidden: false,
        // At most, about: what it is told to expect, with each entry's headers.
        size: room_needed(bytes).saturating_add(items.len() as u64 * 1536),
        modified: std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .map(|d| d.as_secs() as i64),
        mode: None,
    };
    let tally = tally_of(&items, control);
    if to.writes_anywhere() {
        let mut file = to.new_file(path, &like).map_err(upload_failed)?;
        let written = write_archive(items, new, &tally, &mut file).and_then(|()| {
            file.flush()?;
            Ok(())
        });
        why(written, &tally)?;
        file.finish().map_err(upload_failed)?;
        return Ok(());
    }
    let (sender, from) = sync_channel(PIPED_PIECES);
    let broken = Arc::new(AtomicBool::new(false));
    let tell = sender.clone();
    std::thread::scope(|scope| {
        let sending = scope.spawn(|| {
            let mut reader = PipeReader {
                from,
                piece: Vec::new(),
                taken: 0,
            };
            to.put(path, &mut reader, &like, false)
        });
        let out = Piped {
            to: sender,
            piece: Vec::with_capacity(PIPED_PIECE),
            broken: broken.clone(),
        };
        let written = match new.kind {
            Kind::Zip => new_zip(items, new, &tally)
                .and_then(|mut za| Ok(za.close_to_stream(out)?))
                .and_then(zip_closed),
            _ => write_squeezed(items, new, &tally, out),
        };
        let written = why(written, &tally);
        if let Err(e) = &written {
            // Stops the upload, so that nothing appears.
            let _ = tell.send(Err(io::Error::other(e.to_string())));
        }
        drop(tell);
        let sent = sending
            .join()
            .unwrap_or_else(|_| Err(io::Error::other(tr!("The upload stopped."))));
        match (written, sent) {
            // The upload stopped first: the writing only ran out of road.
            (Err(_), Err(e)) if broken.load(Ordering::Relaxed) => Err(upload_failed(e)),
            (Err(e), _) => Err(e),
            (Ok(()), Err(e)) => Err(upload_failed(e)),
            (Ok(()), Ok(_)) => Ok(()),
        }
    })
}

/// Makes an archive of `paths` in `folder` of `to`, as [`create`] does in a folder
/// here, named `name` or the first name after it that is free. In a folder on this
/// computer it is made where it goes, and that place is given. Anywhere else, as
/// [`route`] says: written straight there, or made in a temporary folder first, if
/// there is room for it, then uploaded and taken away. `uploading` is called as that
/// starts, and the progress starts again with what is to go up. Failed or stopped,
/// nothing is left in either place.
pub fn create_on(
    paths: &[PathBuf],
    to: &dyn Storage,
    folder: &[String],
    name: &str,
    new: &NewArchive,
    control: &Control,
    uploading: &dyn Fn(),
) -> Result<Option<PathBuf>, Error> {
    if let Some(here) = to.local_path(folder) {
        return create(paths, &here.join(name), new, control).map(Some);
    }
    // Refused before anything is read.
    age_recipients(new)?;
    let failed = |e: io::Error| Error::Failed(e.to_string());
    // Asked before anything is compressed: what stops it soon stops it first.
    let name = free_name_on(to, folder, name, new.part_size.is_some())?;
    let items = chosen_items(paths, new, control)?;
    let bytes = measured(&items).bytes;
    if route(to, folder, new, bytes) == Route::Straight {
        if items.is_empty() {
            return Err(Error::Failed(tr!("Nothing to compress.")));
        }
        let mut path = folder.to_vec();
        path.push(name.clone());
        straight(items, bytes, to, &path, &name, new, control)?;
        return Ok(None);
    }
    let scratch = Scratch(opened_subfolder()?);
    // Not told, it is found out by trying.
    if let Ok(free) = fs4::available_space(&scratch.0)
        && let Some(e) = no_room(room_needed(bytes), free, &scratch.0)
    {
        return Err(e);
    }
    create_of(items, &scratch.0.join(&name), new, control)?;
    let mut files = Vec::new();
    for entry in fs::read_dir(&scratch.0)? {
        let entry = entry?;
        files.push((entry.file_name(), entry.metadata()?));
    }
    files.sort_by(|a, b| a.0.cmp(&b.0));
    let mut progress = Progress::default();
    progress.total_entries = files.len() as u64;
    progress.total_bytes = files.iter().map(|(_, m)| m.len()).sum();
    control.report(&progress);
    uploading();
    let mut sent: Vec<Vec<String>> = Vec::new();
    for (file, metadata) in files {
        let name = file.to_string_lossy().into_owned();
        let mut path = folder.to_vec();
        path.push(name.clone());
        let like = StoredItem {
            name,
            dir: false,
            link: false,
            hidden: false,
            size: metadata.len(),
            modified: Some(seconds(&metadata)),
            mode: None,
        };
        let local = scratch.0.join(&file);
        let put = File::open(&local).and_then(|from| {
            let mut reader = Watched {
                inner: Box::new(from),
                crc: 0,
                control,
                progress: &mut progress,
            };
            to.put(&path, &mut reader, &like, false)
        });
        if let Err(e) = put {
            for path in &sent {
                let _ = to.remove(path);
            }
            return Err(match control.check() {
                Err(stopped) => stopped,
                Ok(()) => failed(e),
            });
        }
        sent.push(path);
        // What has gone up need not stay.
        let _ = fs::remove_file(&local);
        progress.entries += 1;
        control.report(&progress);
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn room_is_asked_for_what_does_not_compress() {
        assert!(room_needed(0) > 0);
        assert!(room_needed(6_400) > 6_400);
        assert_eq!(room_needed(u64::MAX), u64::MAX);
    }

    #[test]
    fn too_little_room_says_how_much_and_where() {
        let folder = Path::new("/tmp/here");
        assert!(no_room(100, 100, folder).is_none());
        let said = no_room(5_000_000, 1_000, folder)
            .expect("no room")
            .to_string();
        assert!(said.contains("/tmp/here"), "{said}");
        assert!(said.contains("5 MB"), "{said}");
        assert!(said.contains("1 KB"), "{said}");
    }
}
