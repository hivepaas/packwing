//! What an archive holds: its entries, as it names them, and its folders
//! as they show, those it only implies by its paths too.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use hivepack::extract::Cancel;
use hivepack::image::Disk;
use hivepack::open::{Container, Layer, Opener};
use hivepack::tar::{Reader, TYPE_DIR, TYPE_SYMLINK, TYPE_X_GLOBAL_HEADER};
use hivepack::udf::{Udf, VType};
use hivepack::zip::{Archive, Flags, OpenFlags, Volumes, em, opsys, volume_paths};
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

use crate::image::{self, Holds, Image, blocks};
use crate::rar;
use crate::sfx;
use crate::source::Source;
use crate::spool::spool;
use crate::unsupported::named;
use crate::volume::{squashfs, walk, walk_squashfs};
use crate::{Error, Keys, SevenZip, opener_of, seven_zip, without_layers};

/// The names of a macOS package's payloads, which hivepack::extract expands
/// into folders of their names, as `pkgutil --expand-full` does: a
/// component package's files and scripts, and a `.xip` file's content.
const PAYLOADS: [&str; 3] = ["Payload", "Scripts", "Content"];

/// GNU's volume label and dump directory, in a tar archive.
const TYPE_GNU_VOLUME: u8 = b'V';
const TYPE_GNU_DUMPDIR: u8 = b'D';

/// A symbolic link's mode, in Unix's `st_mode`, as ZIP and 7z keep it.
const S_IFMT: u32 = 0o170_000;
const S_IFLNK: u32 = 0o120_000;

/// The longest link target read from a ZIP archive, as Linux's `PATH_MAX`.
const MOST_ZIP_TARGET: u64 = 4096;

/// 7-Zip's mark that the high half of an entry's attributes is a Unix mode.
const UNIX_EXTENSION: u32 = 0x8000;

/// A symbolic link in an archive.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Link {
    /// Where it leads, if the archive says without the link's data being
    /// decompressed: tar, RAR 5, ISO 9660 (Rock Ridge), XAR, cpio, UDF and
    /// SquashFS do, and ZIP when the link is not encrypted; 7z and RAR 4
    /// keep it in their compressed data, read when the link is extracted.
    pub target: Option<String>,
}

impl Link {
    /// A link to `target`, as the archive has it; none if it is empty.
    fn to(target: &[u8]) -> Link {
        Link {
            target: (!target.is_empty()).then(|| String::from_utf8_lossy(target).into_owned()),
        }
    }
}

/// A file or folder in an archive.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// Its path, a component at a time: `["docs", "notes.txt"]`.
    pub path: Vec<String>,
    pub dir: bool,
    /// Unpacked, in bytes.
    pub size: u64,
    /// Seconds since 1970, when the archive says.
    pub modified: Option<i64>,
    /// If it is a symbolic link.
    pub link: Option<Link>,
}

/// What a folder of an archive shows: a file, or a folder with what is in
/// it counted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Child {
    pub name: String,
    pub dir: bool,
    /// A file's size, or all a folder holds.
    pub size: u64,
    /// The files a folder holds, at every depth; 1 for a file.
    pub files: u64,
    /// The latest time among what it holds.
    pub modified: Option<i64>,
    /// If it is a symbolic link.
    pub link: Option<Link>,
}

/// What a search of an archive found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found {
    /// Where it is, a component at a time, its own name last.
    pub path: Vec<String>,
    pub child: Child,
}

/// `text` as a search compares it: lower case, without accents, however they
/// are written. Vietnamese's `đ`, which no accent makes, goes as `d`.
pub(crate) fn folded(text: &str) -> String {
    text.nfd()
        .filter(|c| !is_combining_mark(*c))
        .flat_map(char::to_lowercase)
        .map(|c| if c == 'đ' { 'd' } else { c })
        .collect()
}

/// An archive, read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Listing {
    /// Its file's name and folder.
    pub name: String,
    pub folder: PathBuf,
    /// Its own size, and what it holds unpacked, in bytes.
    pub packed: u64,
    pub unpacked: u64,
    /// The files in it, folders not counted.
    pub files: u64,
    /// The layers around it, the outermost first.
    pub layers: Vec<Layer>,
    pub container: Container,
    /// Where the archive begins in its file: past the program of a
    /// self-extracting archive, and 0 for every other, which begins at its
    /// file's own beginning. Nothing is added to or deleted from an archive
    /// after a program ([`can_add`](crate::can_add)): what holds it would have
    /// to be written again, and it is a program.
    pub at: u64,
    /// How it is encrypted, if it is: `age`, `AES-256`, `ZipCrypto`.
    pub encryption: Option<String>,
    pub entries: Vec<Entry>,
}

impl Listing {
    /// What in it, at any depth, has `query` in its name, whatever the case and
    /// accents: folders it only implies by its paths too, each with what it
    /// holds counted. Its folders first, then its files, each by path.
    pub fn search(&self, query: &str) -> Vec<Found> {
        let query = folded(query.trim());
        if query.is_empty() {
            return Vec::new();
        }
        let mut found: BTreeMap<(bool, Vec<String>), Found> = BTreeMap::new();
        for e in &self.entries {
            for depth in 1..=e.path.len() {
                let name = &e.path[depth - 1];
                let dir = e.dir || depth < e.path.len();
                if !folded(name).contains(&query) {
                    continue;
                }
                let path = &e.path[..depth];
                let key = path.iter().map(|c| c.to_lowercase()).collect();
                let hit = found.entry((!dir, key)).or_insert_with(|| Found {
                    path: path.to_vec(),
                    child: Child {
                        name: name.clone(),
                        dir,
                        size: 0,
                        files: 0,
                        modified: None,
                        link: None,
                    },
                });
                if !e.dir {
                    hit.child.size += e.size;
                    hit.child.files += 1;
                }
                if depth == e.path.len() && !dir {
                    hit.child.link = e.link.clone();
                }
                hit.child.modified = hit.child.modified.max(e.modified);
            }
        }
        found.into_values().collect()
    }

    /// What `folder` holds: its folders, then its files, each by name.
    pub fn children(&self, folder: &[String]) -> Vec<Child> {
        let mut found: BTreeMap<(bool, String, String), Child> = BTreeMap::new();
        for e in &self.entries {
            let Some(rest) = e.path.strip_prefix(folder) else {
                continue;
            };
            let Some(name) = rest.first() else {
                continue;
            };
            let dir = e.dir || rest.len() > 1;
            let child = found
                .entry((!dir, name.to_lowercase(), name.clone()))
                .or_insert_with(|| Child {
                    name: name.clone(),
                    dir,
                    size: 0,
                    files: 0,
                    modified: None,
                    link: None,
                });
            if !e.dir {
                child.size += e.size;
                child.files += 1;
            }
            if !dir {
                child.link = e.link.clone();
            }
            child.modified = child.modified.max(e.modified);
        }
        found.into_values().collect()
    }
}

/// The path of a name in an archive, a component at a time: `/` between
/// them, a leading `./` and empty components dropped.
pub(crate) fn components(name: &str) -> Vec<String> {
    name.split('/')
        .filter(|c| !c.is_empty())
        .enumerate()
        .filter(|&(i, c)| !(i == 0 && c == "."))
        .map(|(_, c)| c.to_owned())
        .collect()
}

/// How many bytes the volumes of the ZIP archive at `path` hold together.
fn zip_size(path: &Path) -> io::Result<u64> {
    let mut size = 0;
    for volume in volume_paths(path) {
        size += std::fs::metadata(volume)?.len();
    }
    Ok(size)
}

/// What the archive at `source` holds, opened with `keys`.
pub fn list(source: impl Into<Source>, keys: &Keys) -> Result<Listing, Error> {
    list_as_read(source, keys, Listen::default())
}

/// What [`list_as_read`] tells of an archive as it is read, and what stops it.
#[derive(Default)]
pub struct Listen<'a> {
    /// Given what is found, some at a time, as it is: an archive read in
    /// order (tar, cpio, ZIP, RAR, ISO 9660) shows before it is read through.
    /// What is given is in the listing too.
    pub found: Option<&'a mut (dyn FnMut(Vec<Entry>) + Send)>,
    /// Kept at how much of the archive's file has been read, in thousandths:
    /// of one read in order; one read from its end goes there at once.
    pub read: Option<Arc<AtomicU32>>,
    /// Stops it, with [`Error::Cancelled`].
    pub cancel: Option<Cancel>,
}

/// How often what is found is given, at most.
const TELL_EVERY: Duration = Duration::from_millis(100);

/// The entries of an archive, gathered as it is read: those found given, now
/// and then, to whom listens.
pub(crate) struct Gather<'a> {
    pub(crate) entries: Vec<Entry>,
    /// How many were given.
    told: usize,
    at: Instant,
    found: Option<&'a mut (dyn FnMut(Vec<Entry>) + Send)>,
    cancel: Option<Cancel>,
}

impl<'a> Gather<'a> {
    fn new(listen: Listen<'a>) -> Gather<'a> {
        Gather {
            entries: Vec::new(),
            told: 0,
            at: Instant::now(),
            found: listen.found,
            cancel: listen.cancel,
        }
    }

    /// One that tells nobody, and stops when it is told to.
    pub(crate) fn quiet() -> Gather<'static> {
        Gather::new(Listen::default())
    }

    pub(crate) fn push(&mut self, entry: Entry) -> Result<(), Error> {
        if self.cancel.as_ref().is_some_and(Cancel::is_cancelled) {
            return Err(Error::Cancelled);
        }
        self.entries.push(entry);
        if let Some(found) = &mut self.found
            && self.at.elapsed() >= TELL_EVERY
        {
            found(self.entries[self.told..].to_vec());
            self.told = self.entries.len();
            self.at = Instant::now();
        }
        Ok(())
    }
}

/// What an archive's file is read through, counting: how far in it is, in
/// thousandths, and stopped when it is to be.
struct Counted<R> {
    inner: R,
    at: u64,
    whole: u64,
    read: Option<Arc<AtomicU32>>,
    cancel: Option<Cancel>,
}

impl<R: Read> Read for Counted<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.cancel.as_ref().is_some_and(Cancel::is_cancelled) {
            return Err(io::Error::other(Error::Cancelled.to_string()));
        }
        let n = self.inner.read(buf)?;
        self.at += n as u64;
        if let Some(read) = &self.read {
            let thousandths = (self.at.saturating_mul(1000) / self.whole.max(1)).min(1000);
            read.store(thousandths as u32, Ordering::Relaxed);
        }
        Ok(n)
    }
}

/// [`list`], telling `listen` of it as it is read.
pub fn list_as_read(
    source: impl Into<Source>,
    keys: &Keys,
    listen: Listen<'_>,
) -> Result<Listing, Error> {
    let cancel = listen.cancel.clone();
    let source = source.into();
    let listed = named(&source, listed(source.clone(), keys, listen));
    // What stopped reading it, it was told to.
    match listed {
        Err(_) if cancel.as_ref().is_some_and(Cancel::is_cancelled) => Err(Error::Cancelled),
        listed => listed,
    }
}

fn listed(source: Source, keys: &Keys, listen: Listen<'_>) -> Result<Listing, Error> {
    let source = &source;
    let mut packed = source.size()?;
    let name = source.name();
    let folder = source.folder();
    let whole = packed;
    let (read, cancel) = (listen.read.clone(), listen.cancel.clone());
    let mut opener = opener_of(&name, keys, move || {
        source.reader().map(|inner| Counted {
            inner,
            at: 0,
            whole,
            read: read.clone(),
            cancel: cancel.clone(),
        })
    });
    let mut gather = Gather::new(listen);
    let mut opened = opener.open()?;
    let layers = opened.layers().to_vec();
    // A self-extracting archive arrives as a program's bytes, which are not an
    // archive's: what it carries after it is looked for before giving up on
    // it, and `at` is where that begins (0 for RAR, which hivepack finds
    // after the program itself).
    let sfx::Carried { container, at } = sfx::of(source, opened.container(), &layers)?;
    let mut encryption = layers.contains(&Layer::Age).then(|| "age".to_owned());
    let entries = match container {
        Container::Tar => tar_entries(&mut opened, &mut gather)?,
        // An archive after a program: read from where it begins, and never
        // as one volume of a split archive, which it cannot be.
        Container::Zip if at > 0 => {
            drop(opened);
            let (entries, zip_encryption) = match source {
                Source::File(path) => {
                    zip_entries(sfx::At::new(File::open(path)?, at, whole), &mut gather)?
                }
                Source::Stored(stored) => {
                    zip_entries(sfx::At::new(stored.stored_reader(), at, whole), &mut gather)?
                }
            };
            encryption = encryption.or(zip_encryption);
            entries
        }
        Container::SevenZip if at > 0 => {
            drop(opened);
            let open = || source.reader().map(|r| sfx::At::new(r, at, whole));
            let (entries, seven_zip_encryption) = seven_zip_entries(seven_zip(open, keys)?);
            encryption = encryption.or(seven_zip_encryption);
            entries
        }
        Container::Zip => {
            let (entries, zip_encryption) = if layers.is_empty() {
                drop(opened);
                match source {
                    Source::File(path) => {
                        packed = zip_size(path)?;
                        zip_entries(Volumes::open(path)?, &mut gather)?
                    }
                    Source::Stored(stored) => zip_entries(stored.stored_reader(), &mut gather)?,
                }
            } else {
                let spooled = spool(&mut opened)?;
                zip_entries(&spooled.file, &mut gather)?
            };
            encryption = encryption.or(zip_encryption);
            entries
        }
        // Read in order; from the file itself, a file whose data is before
        // its directory is read by going back. A type 1 AppImage is an ISO
        // 9660 image, its program in the image's system area.
        Container::Iso | Container::AppImageType1 if layers.is_empty() => {
            drop(opened);
            iso_entries(
                hivepack::iso::Reader::seekable(source.reader()?),
                &mut gather,
            )?
        }
        Container::Iso | Container::AppImageType1 => {
            iso_entries(hivepack::iso::Reader::new(&mut opened), &mut gather)?
        }
        Container::Udf if layers.is_empty() => {
            drop(opened);
            udf_entries(source.reader()?)?
        }
        Container::Udf => {
            let spooled = spool(&mut opened)?;
            udf_entries(&spooled.file)?
        }
        // Read by random access, as a UDF volume is: a snap, and a type 2
        // AppImage's image, after its program.
        Container::Squashfs | Container::AppImageType2 if layers.is_empty() => {
            drop(opened);
            squashfs_entries(squashfs(container, source.reader()?)?, &mut gather)?
        }
        Container::Squashfs | Container::AppImageType2 => {
            let spooled = spool(&mut opened)?;
            squashfs_entries(squashfs(container, &spooled.file)?, &mut gather)?
        }
        Container::SevenZip => {
            let (entries, seven_zip_encryption) = if layers.is_empty() {
                drop(opened);
                seven_zip_entries(seven_zip(|| source.reader(), keys)?)
            } else {
                let spooled = spool(&mut opened)?;
                seven_zip_entries(seven_zip(|| spooled.rewound(), keys)?)
            };
            encryption = encryption.or(seven_zip_encryption);
            entries
        }
        Container::Rar => {
            let password = rar::password(keys);
            let (found, encrypted) = if let (true, Some(path)) = (layers.is_empty(), source.path())
            {
                drop(opened);
                let more = Arc::new(AtomicU64::new(0));
                let found =
                    rar::entries(rar::open(path, &password, more.clone())?, keys, &mut gather)?;
                packed += more.load(Ordering::Relaxed);
                found
            } else {
                let spooled = spool(&mut opened)?;
                rar::entries(
                    hivepack::rar::Archive::open_with_password(
                        spooled.rewound()?,
                        false,
                        &password,
                    )?,
                    keys,
                    &mut gather,
                )?
            };
            if encrypted {
                encryption = encryption.or(Some("Encrypted".to_owned()));
            }
            found
        }
        // An RPM package is its payload: the package's lead and headers
        // are a layer. A Debian package, its data.tar, is a tar archive.
        Container::Cpio => cpio_entries(&mut opened, &mut gather)?,
        Container::Xar => xar_entries(&mut opened)?,
        Container::Data if layers.is_empty() => return Err(Error::NotAnArchive),
        Container::Data => {
            let size = io::copy(&mut opened, &mut io::sink())?;
            vec![Entry {
                path: vec![without_layers(&name, &layers)],
                dir: false,
                size,
                modified: None,
                link: None,
            }]
        }
        // A disk image: read by random access, as a UDF volume is, so under
        // layers it is written to a file first. Its partitions are folders
        // and the file systems in them hold the files.
        Container::PartitionedDisk
        | Container::Vhd
        | Container::Qcow2
        | Container::Vhdx
        | Container::Vmdk => {
            let blocks = blocks(source, &mut opened, &layers)?;
            let image = Image::open(container, blocks.file.try_clone()?, blocks.path.as_deref())?;
            disk_entries(&image, &mut gather)?
        }
        // One file system with no table in front of it: an EFI system
        // partition taken out on its own, a camera's card, a floppy image.
        Container::Fat | Container::Ext | Container::Ntfs => {
            let blocks = blocks(source, &mut opened, &layers)?;
            let disk: &dyn Disk = &blocks.file;
            let holds = image::holds_at(disk, 0, disk.size()?);
            if matches!(holds, Holds::Bytes) {
                return Err(Error::NotAnArchive);
            }
            file_system_entries(&holds, &[], &mut gather)?;
            std::mem::take(&mut gather.entries)
        }
        // What a later hivepack tells and packwing does not read yet.
        _ => return Err(Error::NotAnArchive),
    };
    let files = entries.iter().filter(|e| !e.dir).count() as u64;
    let unpacked = entries.iter().filter(|e| !e.dir).map(|e| e.size).sum();
    Ok(Listing {
        name,
        folder,
        packed,
        unpacked,
        files,
        layers,
        container,
        at,
        encryption,
        entries,
    })
}

fn tar_entries(r: impl Read, gather: &mut Gather) -> Result<Vec<Entry>, Error> {
    let mut tr = Reader::new(r);
    while let Some(h) = tr.next()? {
        if matches!(h.typeflag, TYPE_X_GLOBAL_HEADER | TYPE_GNU_VOLUME) {
            continue;
        }
        let path = components(&String::from_utf8_lossy(&h.name));
        if path.is_empty() {
            continue;
        }
        let dir = matches!(h.typeflag, TYPE_DIR | TYPE_GNU_DUMPDIR) || h.name.ends_with(b"/");
        let link = (h.typeflag == TYPE_SYMLINK && !dir).then(|| Link::to(&h.linkname));
        gather.push(Entry {
            path,
            dir,
            size: if dir { 0 } else { h.size.max(0) as u64 },
            modified: h.mod_time.map(|t| t.secs),
            link,
        })?;
    }
    Ok(std::mem::take(&mut gather.entries))
}

fn zip_entries<R: hivepack::zip::ReadAt>(
    source: R,
    gather: &mut Gather,
) -> Result<(Vec<Entry>, Option<String>), Error> {
    let archive = Archive::open(source, OpenFlags::CHECKCONS | OpenFlags::RDONLY)?;
    let mut encryption = None;
    for i in 0..archive.get_num_entries(Flags::NONE) {
        let stat = archive.stat_index(i, Flags::NONE)?;
        let name = String::from_utf8_lossy(stat.name.unwrap_or_default()).into_owned();
        let path = components(&name);
        if path.is_empty() {
            continue;
        }
        if encryption.is_none() {
            encryption = match stat.encryption_method {
                em::NONE => None,
                em::TRAD_PKWARE => Some("ZipCrypto".to_owned()),
                em::AES_128 => Some("AES-128".to_owned()),
                em::AES_192 => Some("AES-192".to_owned()),
                em::AES_256 => Some("AES-256".to_owned()),
                _ => Some("Encrypted".to_owned()),
            };
        }
        let dir = name.ends_with('/');
        // Unix keeps the mode in the high half of the external attributes;
        // the link's target is its data.
        let (system, attributes) = archive.file_get_external_attributes(i, Flags::NONE)?;
        let link =
            (system == opsys::UNIX && (attributes >> 16) & S_IFMT == S_IFLNK && !dir).then(|| {
                Link {
                    target: (stat.encryption_method == em::NONE && stat.size <= MOST_ZIP_TARGET)
                        .then(|| zip_target(&archive, i))
                        .flatten(),
                }
            });
        gather.push(Entry {
            path,
            dir,
            size: if dir { 0 } else { stat.size },
            modified: Some(stat.mtime),
            link,
        })?;
    }
    Ok((std::mem::take(&mut gather.entries), encryption))
}

/// The target of the ZIP archive's link at `index`, its data; none if it
/// cannot be read.
fn zip_target<R: hivepack::zip::ReadAt>(archive: &Archive<R>, index: u64) -> Option<String> {
    let mut data = Vec::new();
    let file = archive.fopen_index(index, Flags::NONE).ok()?;
    file.take(MOST_ZIP_TARGET + 1).read_to_end(&mut data).ok()?;
    (data.len() as u64 <= MOST_ZIP_TARGET)
        .then(|| Link::to(&data).target)
        .flatten()
}

/// An ISO 9660 image's entries: Rock Ridge's names, else Joliet's, else
/// ISO 9660's. The root directory, `.`, is not one, nor is an entry
/// whose name could not be made.
fn iso_entries<R: Read>(
    mut iso: hivepack::iso::Reader<R>,
    gather: &mut Gather,
) -> Result<Vec<Entry>, Error> {
    while let Some(h) = iso.next()? {
        let Some(name) = &h.path else {
            continue;
        };
        let path = components(&String::from_utf8_lossy(name));
        if path.is_empty() {
            continue;
        }
        let dir = h.is_dir();
        let link = (h.mode & hivepack::iso::AE_IFMT == hivepack::iso::AE_IFLNK)
            .then(|| Link::to(h.symlink.as_deref().unwrap_or_default()));
        gather.push(Entry {
            path,
            dir,
            // A hard link has no size: its data, the entry's before it, is
            // counted there, once.
            size: if dir { 0 } else { h.size.unwrap_or(0) },
            modified: h.mod_time,
            link,
        })?;
    }
    Ok(std::mem::take(&mut gather.entries))
}

/// A 7z archive's entries, and `AES-256` if its files or names are
/// encrypted (7z has no other encryption). One whose names are encrypted
/// needs the password to be opened at all.
fn seven_zip_entries<R: Read + io::Seek>(archive: SevenZip<R>) -> (Vec<Entry>, Option<String>) {
    let mut entries = Vec::new();
    let mut encrypted = archive.headers_encrypted;
    for e in archive.archive.entries() {
        encrypted |= e.encrypted;
        let path = components(&e.name);
        if path.is_empty() {
            continue;
        }
        // Its target is its data, compressed.
        let link = e
            .attributes
            .filter(|a| a & UNIX_EXTENSION != 0 && (a >> 16) & S_IFMT == S_IFLNK && !e.is_dir)
            .map(|_| Link::default());
        entries.push(Entry {
            path,
            dir: e.is_dir,
            size: if e.is_dir { 0 } else { e.size },
            modified: e.mtime.map(ntfs_to_unix),
            link,
        });
    }
    (entries, encrypted.then(|| "AES-256".to_owned()))
}

/// An NTFS time, in 100 ns since 1601, in seconds since 1970.
fn ntfs_to_unix(ntfs: u64) -> i64 {
    (ntfs / 10_000_000) as i64 - 11_644_473_600
}

/// A cpio archive's entries: Linux's initramfs, an RPM package's payload.
/// The root, `.`, is not one.
fn cpio_entries(r: impl Read, gather: &mut Gather) -> Result<Vec<Entry>, Error> {
    let mut cpio = hivepack::cpio::Reader::new(r);
    while let Some(h) = cpio.next()? {
        let path = components(&String::from_utf8_lossy(&h.path));
        if path.is_empty() {
            continue;
        }
        let kind = h.file_type();
        let dir = kind == hivepack::cpio::AE_IFDIR;
        gather.push(Entry {
            path,
            dir,
            size: if kind == hivepack::cpio::AE_IFREG {
                h.size
            } else {
                0
            },
            modified: Some(h.mod_time),
            link: (kind == hivepack::cpio::AE_IFLNK)
                .then(|| Link::to(h.symlink.as_deref().unwrap_or_default())),
        })?;
    }
    Ok(std::mem::take(&mut gather.entries))
}

/// A XAR archive's entries. A macOS package's payloads (`Payload`,
/// `Scripts`, a `.xip` file's `Content`), cpio archives under gzip or
/// pbzx, show as folders of what they hold, as they extract.
fn xar_entries(r: impl Read + Send) -> Result<Vec<Entry>, Error> {
    let mut xar = hivepack::xar::Reader::new(r);
    let mut entries = Vec::new();
    while let Some(h) = xar.next()? {
        let Some(name) = &h.path else {
            continue;
        };
        let path = components(&String::from_utf8_lossy(name));
        if path.is_empty() {
            continue;
        }
        let kind = h.mode & hivepack::xar::AE_IFMT;
        let is_payload = kind == hivepack::xar::AE_IFREG
            && h.hardlink.is_none()
            && path
                .last()
                .is_some_and(|last| PAYLOADS.contains(&last.as_str()));
        if is_payload && let Some(inside) = payload_entries(&mut xar)? {
            entries.push(Entry {
                path: path.clone(),
                dir: true,
                size: 0,
                modified: h.mtime,
                link: None,
            });
            entries.extend(inside.into_iter().map(|mut e| {
                e.path.splice(0..0, path.iter().cloned());
                e
            }));
            continue;
        }
        let dir = kind == hivepack::xar::AE_IFDIR;
        entries.push(Entry {
            path,
            dir,
            size: if kind == hivepack::xar::AE_IFREG && h.hardlink.is_none() {
                h.size
            } else {
                0
            },
            modified: h.mtime,
            link: (kind == hivepack::xar::AE_IFLNK)
                .then(|| Link::to(h.symlink.as_deref().unwrap_or_default())),
        });
    }
    Ok(entries)
}

/// What the payload the XAR reader is at holds, if it is a cpio archive
/// under its layers; `None` if it is something else, which shows as a
/// file.
fn payload_entries<R: Read + Send>(
    xar: &mut hivepack::xar::Reader<R>,
) -> Result<Option<Vec<Entry>>, Error> {
    let mut once = Some(xar);
    let mut opener = Opener::new(move || {
        once.take()
            .ok_or_else(|| io::Error::other("a payload opened twice"))
    });
    let opened = opener.open()?;
    if opened.container() != Container::Cpio {
        return Ok(None);
    }
    cpio_entries(opened, &mut Gather::quiet()).map(Some)
}

/// A UDF volume's entries, as its directories list them.
fn udf_entries<R: Read + io::Seek>(image: R) -> Result<Vec<Entry>, Error> {
    let mut udf = Udf::open(image)?;
    let mut entries = Vec::new();
    walk(&mut udf, |udf, path, node, stat| {
        let dir = stat.kind == VType::VDIR;
        // Its target is its data, most often in its node.
        let link = (stat.kind == VType::VLNK).then(|| Link {
            target: udf.read_link(node).ok().and_then(|t| Link::to(&t).target),
        });
        entries.push(Entry {
            path: path.to_vec(),
            dir,
            size: if stat.kind == VType::VREG {
                stat.size
            } else {
                0
            },
            modified: Some(stat.mtime.sec),
            link,
        });
        Ok(())
    })?;
    Ok(entries)
}

/// A SquashFS image's entries, as its directories list them: a snap's, an
/// AppImage's. A second name of a file, a hard link, has no size: its data
/// is counted at its first, once.
fn squashfs_entries<R: Read + io::Seek>(
    mut fs: hivepack::squashfs::Squashfs<R>,
    gather: &mut Gather,
) -> Result<Vec<Entry>, Error> {
    walk_squashfs(&mut fs, |fs, path, inode, first| {
        let link = inode.is_symlink().then(|| Link {
            target: fs.read_link(inode).ok().and_then(|t| Link::to(&t).target),
        });
        gather.push(Entry {
            path: path.to_vec(),
            dir: inode.is_dir(),
            size: if inode.is_file() && first {
                inode.file_size()
            } else {
                0
            },
            modified: Some(i64::from(inode.base.mtime)),
            link,
        })
    })?;
    Ok(std::mem::take(&mut gather.entries))
}

/// A disk image's entries: **a folder for each partition**, named by its
/// number and what it holds (`1-EFI`, `2-ext4`), with the files of its file
/// system under it. A partition whose file system packwing does not read is
/// one file of its bytes (`3-swap.img`), so that it can still be taken off
/// the disk.
///
/// A disk no table names a partition on is one file system over its whole
/// length, as WSL2's `ext4.vhdx` holds; one that is neither is not an
/// archive.
fn disk_entries(image: &Image, gather: &mut Gather) -> Result<Vec<Entry>, Error> {
    let disk = image.disk();
    let partitions = image::table_of(disk)?;
    if partitions.is_empty() {
        let size = disk.size()?;
        let holds = image::holds_at(disk, 0, size);
        if matches!(holds, Holds::Bytes) {
            return Err(Error::NotAnArchive);
        }
        file_system_entries(&holds, &[], gather)?;
        return Ok(std::mem::take(&mut gather.entries));
    }
    for partition in &partitions {
        let holds = image::holds(disk, partition);
        let name = image::name_of(partition, &holds);
        if matches!(holds, Holds::Bytes) {
            gather.push(Entry {
                path: vec![format!("{name}.img")],
                dir: false,
                size: partition.size,
                modified: None,
                link: None,
            })?;
            continue;
        }
        gather.push(Entry {
            path: vec![name.clone()],
            dir: true,
            size: 0,
            modified: None,
            link: None,
        })?;
        file_system_entries(&holds, std::slice::from_ref(&name), gather)?;
    }
    Ok(std::mem::take(&mut gather.entries))
}

/// One file system's entries, under `at` (a partition's folder, or nothing
/// for a file system with no table in front of it).
///
/// A file with more than one name is counted once: its data is at the first
/// name the walk gives, as a SquashFS image's hard links are.
fn file_system_entries(holds: &Holds<'_>, at: &[String], gather: &mut Gather) -> Result<(), Error> {
    let under = |path: &str| {
        let mut components = at.to_vec();
        components.extend(path.split('/').map(str::to_owned));
        components
    };
    match holds {
        Holds::Fat(fs, _) => {
            let mut walk = fs.walk().map_err(failed)?;
            while let Some((path, entry)) = walk.next(fs).map_err(failed)? {
                gather.push(Entry {
                    path: under(&path),
                    dir: entry.is_dir(),
                    size: if entry.is_dir() { 0 } else { entry.size },
                    modified: entry.modified.map(|t| t.unix()),
                    link: None,
                })?;
            }
        }
        Holds::Ext(fs, _) => {
            let mut walk = fs.walk().map_err(failed)?;
            let mut met: BTreeSet<u32> = BTreeSet::new();
            while let Some(step) = walk.next(fs).map_err(failed)? {
                let node = &step.node;
                let first = met.insert(node.ino);
                let link = node.is_symlink().then(|| Link {
                    target: fs.read_link(node).ok().and_then(|t| Link::to(&t).target),
                });
                gather.push(Entry {
                    path: under(&String::from_utf8_lossy(&step.path)),
                    dir: node.is_dir(),
                    size: if node.is_file() && first {
                        node.size
                    } else {
                        0
                    },
                    modified: node.modified.map(|t| t.seconds),
                    link,
                })?;
            }
        }
        // Every call into the library that reads NTFS is caught: a damaged
        // directory index panics it, and the fix is in no release yet.
        Holds::Ntfs(fs) => image::caught(|| {
            let mut walk = fs.walk().map_err(failed)?;
            let mut met: BTreeSet<u64> = BTreeSet::new();
            while let Some((path, entry)) = walk.next(fs).map_err(failed)? {
                let first = met.insert(entry.record());
                gather.push(Entry {
                    path: under(&path),
                    dir: entry.is_dir(),
                    size: if entry.is_file() && first {
                        entry.size
                    } else {
                        0
                    },
                    modified: entry.modified.map(|t| t.unix().0),
                    link: None,
                })?;
            }
            Ok(())
        })?,
        // Its bytes are the file, and `disk_entries` wrote that entry.
        Holds::Bytes => {}
    }
    Ok(())
}

/// What a reader refused, as packwing carries it.
fn failed<E: std::fmt::Display>(e: E) -> Error {
    Error::Failed(e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str) -> Entry {
        Entry {
            path: vec![name.to_owned()],
            dir: false,
            size: 1,
            modified: None,
            link: None,
        }
    }

    fn data(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data")
            .join(name)
    }

    #[test]
    fn what_is_found_is_told_now_and_then() {
        let mut told: Vec<Entry> = Vec::new();
        let mut tell = |more: Vec<Entry>| told.extend(more);
        let mut gather = Gather::new(Listen {
            found: Some(&mut tell),
            ..Listen::default()
        });
        gather.push(entry("a")).expect("not stopped");
        std::thread::sleep(TELL_EVERY);
        // What was not told yet goes with it.
        gather.push(entry("b")).expect("not stopped");
        gather.push(entry("c")).expect("not stopped");
        let entries = std::mem::take(&mut gather.entries);
        drop(gather);
        assert_eq!(told, [entry("a"), entry("b")]);
        assert_eq!(entries, [entry("a"), entry("b"), entry("c")]);
    }

    #[test]
    fn a_reading_stops_when_told_to() {
        for name in ["notes.tlz", "sample.7z", "rar5_multiple_files.rar"] {
            let cancel = Cancel::new();
            cancel.cancel();
            let listen = Listen {
                cancel: Some(cancel),
                ..Listen::default()
            };
            let listed = list_as_read(data(name), &Keys::default(), listen);
            assert!(
                matches!(listed, Err(Error::Cancelled)),
                "{name}: {listed:?}"
            );
        }
    }

    #[test]
    fn a_reading_says_how_far_in_it_is() {
        let read = Arc::new(AtomicU32::new(0));
        let listen = Listen {
            read: Some(read.clone()),
            ..Listen::default()
        };
        let listing = list_as_read(data("notes.tlz"), &Keys::default(), listen).expect("read");
        assert_eq!(
            listing,
            list(data("notes.tlz"), &Keys::default()).expect("read")
        );
        let read = read.load(Ordering::Relaxed);
        assert!(read > 0 && read <= 1000, "{read}");
    }

    #[test]
    fn components_of_names() {
        assert_eq!(components("./a/b.txt"), ["a", "b.txt"]);
        assert_eq!(components("a//b/"), ["a", "b"]);
        assert_eq!(components("/abs/x"), ["abs", "x"]);
        assert_eq!(components("a/./b"), ["a", ".", "b"]);
        assert!(components("./").is_empty());
    }
}
