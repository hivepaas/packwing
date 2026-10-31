//! Making an archive of files and folders: ZIP through hivepack-zip, 7z
//! through hivepack-7z, and tar through hivepack-tar under gzip, xz or zstd.
//! It is written beside what is there, never over it, and taken away again
//! if it fails or is stopped. Links are kept as links, on Unix; what is not a
//! file, a folder or a link (a pipe, a socket, a device) is left out, as
//! reading it could wait for ever. A tar archive's owners are left out too:
//! its files may go to someone with other users.

use std::fmt;
use std::fs::{self, File, Metadata, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::UNIX_EPOCH;

use hivepack_deflate::{DEF_MEM_LEVEL, Deflate, Flush, Z_DEFAULT_STRATEGY};
use hivepack_extract::Progress;
use hivepack_xz::{MtOptions, Stream, stream_encoder_mt_memusage};
use hivepack_zip::{
    Archive, BufferSource, Closed, DataSource, ErrorCode, FileAttributes, Flags, OpenFlags, Stat,
    cm, em, file_attributes, opsys, stat,
};

use crate::{Control, Error};

/// A kind of archive packwing makes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Kind {
    #[default]
    Zip,
    SevenZip,
    TarGz,
    TarXz,
    TarZst,
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
    /// a 7z one. A tar archive cannot have one.
    pub password: Option<Vec<u8>>,
    /// Encrypts a 7z archive's names too, with its password.
    pub hide_names: bool,
    /// Cuts it into `name.zip.001`, `name.zip.002`, … of this many bytes, as
    /// 7-Zip does.
    pub part_size: Option<u64>,
}

impl fmt::Debug for NewArchive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NewArchive")
            .field("kind", &self.kind)
            .field("level", &self.level)
            .field("password", &self.password.is_some())
            .field("hide_names", &self.hide_names)
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
            "The name of {} is not valid Unicode, so it cannot go in an archive.",
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
    Ok(measured(&items_of(paths)?))
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

/// A file's name, split before what names its kind: `a.tar` and `.gz` would
/// make `a.tar (2).gz`.
fn name_and_extension(name: &str) -> (&str, &str) {
    let dot = name.rfind(".tar.").or_else(|| name.rfind('.'));
    name.split_at(dot.filter(|d| *d > 0).unwrap_or(name.len()))
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
    match new.kind {
        Kind::Zip => write_zip(items, new, tally, out),
        Kind::SevenZip => write_7z(items, new, tally, out),
        Kind::TarGz | Kind::TarXz | Kind::TarZst => {
            write_tar(items, tally, Squeezed::new(new, out)?)
        }
    }
}

fn write_zip<W: Write + Seek>(
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
            za.set_file_compression(index, cm::DEFAULT, new.level.deflate())?;
            // Nothing to read, so nothing would count it.
            if item.metadata.len() == 0 {
                tally.advance(1, 0);
            }
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

/// The data of `item`, a file, into `out`, a piece at a time: the size it had
/// when it was counted, as a tar archive has said already by then. Reading it
/// counts towards the progress, and is where a job stops.
fn pump(item: &Item, out: &mut impl Write, tally: &Tally, buf: &mut [u8]) -> Result<(), Error> {
    let read = |e: io::Error| Error::Failed(format!("Could not read {}: {e}", item.path.display()));
    let mut file = File::open(&item.path).map_err(read)?;
    let mut left = item.metadata.len();
    while left > 0 {
        tally.control.check()?;
        let wanted = left.min(buf.len() as u64) as usize;
        match file.read(&mut buf[..wanted]) {
            Ok(0) => {
                return Err(Error::Failed(format!(
                    "{} got shorter while it was being read.",
                    item.path.display()
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
                .map_err(|_| Error::Failed("A password has to be valid Unicode.".into()))?,
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

fn write_tar<W: Write>(items: Vec<Item>, tally: &Tally, out: Squeezed<W>) -> Result<(), Error> {
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
            header.size = i64::try_from(item.metadata.len())
                .map_err(|_| Error::Failed(format!("{} is too big.", item.path.display())))?;
        }
        tar.write_header(&header)?;
        if link {
            tally.advance(1, 0);
        } else if !kind.is_dir() {
            pump(item, &mut tar, tally, &mut buf)?;
        }
    }
    tar.close()?;
    tar.into_inner().finish()?;
    Ok(())
}

/// How much is read of a file at a time, and made of what compresses.
const PIECE: usize = 1 << 16;

/// The most xz's threads may take between them, in bytes. Each takes about
/// three dictionaries, and the highest level's are 64 MiB.
const XZ_MEMORY: u64 = 4 << 30;

/// What compresses a tar archive.
enum Coder {
    Gzip(Box<Deflate>),
    Xz(Box<Stream>),
    Zstd(Box<hivepack_zstd::CCtx>),
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
        let failed = |e: String| Error::Failed(format!("Could not start compressing: {e}"));
        let coder = match new.kind {
            Kind::TarXz => {
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
            Kind::TarZst => {
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
        let failed = |e: String| io::Error::other(format!("Could not compress: {e}"));
        let mut at = 0;
        loop {
            let (made, done) = match &mut self.coder {
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
        .map_err(|e| Error::Failed(format!("Could not read {}: {e}", path.display())))
}

#[cfg(not(unix))]
fn link_target(path: &Path) -> Result<Vec<u8>, Error> {
    Err(Error::Failed(format!(
        "{} is a link, which is kept only on Unix.",
        path.display()
    )))
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
    if new.password.is_some() && !matches!(new.kind, Kind::Zip | Kind::SevenZip) {
        return Err(Error::Failed("A tar archive can’t have a password.".into()));
    }
    let items = items_of(paths)?;
    if items.is_empty() {
        return Err(Error::Failed("Nothing to compress.".into()));
    }
    // Before the archive is made, so that it is not among them.
    let mut progress = Progress::default();
    let counted = measured(&items);
    progress.total_entries = counted.files;
    progress.total_bytes = counted.bytes;
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
