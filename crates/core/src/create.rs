//! Making an archive of files and folders: ZIP through hivepack::zip, 7z
//! through hivepack::sevenz, and tar through hivepack::tar, under gzip, xz, zstd, bzip2
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
use packwing_policy::{Method, Policy, Refused};
use serde_json::{Map, Value, json};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs::{self, File, Metadata, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant, UNIX_EPOCH};

use hivepack::age::{
    BoxedRecipient, RandomSource, Recipient, ScryptRecipient, parse_recipient,
    parse_recipients_file,
};
use hivepack::deflate::{DeflateMt, Flush};
use hivepack::extract::{NameRules, Progress, check_name_for};
use hivepack::open::Container;
use hivepack::xz::{LzmaOptions, MtOptions, Stream, stream_encoder_mt_memusage};
use hivepack::zip::{
    Archive, BufferSource, Closed, DataSource, ErrorCode, FileAttributes, Flags, OpenFlags, ReadAt,
    Stat, cm, em, file_attributes, opsys, stat, volume_paths,
};

use crate::audit;
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
    /// Whether it is a tar archive, compressed or not.
    pub fn is_tar(self) -> bool {
        matches!(
            self,
            Kind::Tar | Kind::TarGz | Kind::TarXz | Kind::TarZst | Kind::TarBz2 | Kind::TarLz4
        )
    }

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
    /// Not at all: a ZIP or 7z archive's files go in as they are. What is
    /// compressed whole, as a tar archive is, takes its lowest level, as plain
    /// `.tar` is the one with none.
    Store,
    Lowest,
    Low,
    #[default]
    Medium,
    High,
    Highest,
}

impl Level {
    /// The level Deflate is asked for: zlib's own default in the middle.
    /// What the user calls it.
    pub fn name(self) -> String {
        match self {
            Level::Store => tr!("Store"),
            Level::Lowest => tr!("Lowest"),
            Level::Low => tr!("Low"),
            Level::Medium => tr!("Medium"),
            Level::High => tr!("High"),
            Level::Highest => tr!("Highest"),
        }
    }

    /// The level below, where there is one that compresses: Lowest is the
    /// last, as Store is not taken for it.
    fn lower(self) -> Option<Level> {
        match self {
            Level::Highest => Some(Level::High),
            Level::High => Some(Level::Medium),
            Level::Medium => Some(Level::Low),
            Level::Low => Some(Level::Lowest),
            Level::Lowest | Level::Store => None,
        }
    }

    pub(crate) fn deflate(self) -> u32 {
        match self {
            Level::Store | Level::Lowest => 1,
            Level::Low => 3,
            Level::Medium => 6,
            Level::High => 8,
            Level::Highest => 9,
        }
    }

    /// The preset of LZMA and LZMA2: xz's own default in the middle.
    pub(crate) fn xz(self) -> u32 {
        match self {
            Level::Store | Level::Lowest => 1,
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
            Level::Store | Level::Lowest => 1,
            Level::Low => 5,
            Level::Medium | Level::High | Level::Highest => 9,
        }
    }

    /// LZ4's level: its fast way up to the middle, as speed is why it is chosen.
    fn lz4(self) -> i32 {
        match self {
            Level::Store | Level::Lowest => -1,
            Level::Low | Level::Medium => 1,
            Level::High => 9,
            Level::Highest => 12,
        }
    }

    fn zstd(self) -> i32 {
        match self {
            Level::Store | Level::Lowest => 1,
            Level::Low => 3,
            Level::Medium => 9,
            Level::High => 15,
            Level::Highest => 19,
        }
    }
}

/// How a zstd stream is cut: one frame, which compresses best, or frames of
/// [`hivepack::zstd::FRAME_SIZE`] (32 MiB) each, which a reader decodes on every
/// core, a frame in each thread, and whose seek table says where each starts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Frames {
    /// Several where what goes in comes to [`SEVERAL_FRAMES_FROM`] or more, as
    /// far as it is known; one otherwise.
    #[default]
    Auto,
    One,
    Several,
}

/// What each of several zstd frames holds: 32 MiB.
pub const FRAME_SIZE: u64 = hivepack::zstd::FRAME_SIZE as u64;

/// What goes in from which [`Frames::Auto`] makes several frames: 256 MiB, eight
/// frames, where decoding on several cores starts to tell.
pub const SEVERAL_FRAMES_FROM: u64 = 256 << 20;

impl Frames {
    /// Whether a stream of `bytes`, if they are known, is cut into several
    /// frames.
    pub fn several(self, bytes: Option<u64>) -> bool {
        match self {
            Frames::Auto => bytes.is_some_and(|b| b >= SEVERAL_FRAMES_FROM),
            Frames::One => false,
            Frames::Several => true,
        }
    }
}

/// How LZMA2 is cut into blocks, each compressed in a thread and decoded in
/// one: a `.tar.xz` or `.xz`'s, and a 7z archive's. The same archive whatever
/// the threads, as the blocks depend on the input's size alone.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Blocks {
    /// The format's own: xz's of three dictionaries, 7-Zip's of four, up to
    /// 256 MiB. The smallest archive, but fewer blocks than cores leave
    /// cores idle: 2.3 GB at the highest level is 13 blocks of xz, 10 of 7z.
    #[default]
    Large,
    /// Enough for [`SMALL_BLOCKS_FOR`] cores where the format's own are
    /// fewer, down to one dictionary each, as what LZMA finds is within one.
    /// At the highest level on 32 cores: 2.3 GB of a computer's own files as
    /// xz, 79 s in blocks of 192 MiB, 38 s in 72 MiB, and 4% larger; as 7z,
    /// 97 s in 256 MiB, 35–44 s in 64–96 MiB, 3–7% larger; 3 GB of programs
    /// as 7z, 113 s, and 63–73 s, 0.4–0.8% larger.
    Small,
}

/// The cores [`Blocks::Small`] cuts enough blocks for, whatever the threads.
pub const SMALL_BLOCKS_FOR: u64 = 32;

impl Blocks {
    /// The input of each block of a `.tar.xz` or `.xz` of `bytes` at
    /// `level`, or 0 for xz's own.
    fn xz(self, level: Level, bytes: u64) -> u64 {
        match self {
            Blocks::Large => 0,
            Blocks::Small => small_blocks(bytes, xz_blocks_of(level)),
        }
    }

    /// A 7z archive's blocks, of `bytes` at `level`.
    fn seven_zip(self, level: Level, bytes: u64) -> hivepack::sevenz::Lzma2Blocks {
        match self {
            Blocks::Large => hivepack::sevenz::Lzma2Blocks::Auto,
            Blocks::Small => {
                hivepack::sevenz::Lzma2Blocks::Size(small_blocks(bytes, seven_zip_blocks_of(level)))
            }
        }
    }

    /// Whether small blocks differ from large ones for `bytes` of `kind` at
    /// `level`: for xz and 7z, when there are fewer than
    /// [`SMALL_BLOCKS_FOR`] of the format's own, and more than one
    /// dictionary.
    pub fn differ(kind: Kind, level: Level, bytes: u64) -> bool {
        let (dictionary, own) = match kind {
            _ if level == Level::Store => return false,
            Kind::TarXz | Kind::Xz => xz_blocks_of(level),
            Kind::SevenZip => seven_zip_blocks_of(level),
            _ => return false,
        };
        bytes > dictionary && small_blocks(bytes, (dictionary, own)) < own
    }
}

/// Small blocks for `bytes`, of a dictionary and the format's own.
fn small_blocks(bytes: u64, (dictionary, own): (u64, u64)) -> u64 {
    bytes.div_ceil(SMALL_BLOCKS_FOR).clamp(dictionary, own)
}

/// LZMA2's dictionary at `level`.
fn dictionary_of(level: Level) -> u64 {
    LzmaOptions::preset(level.xz()).map_or(8 << 20, |o| u64::from(o.dict_size))
}

/// xz's dictionary at `level`, and its own blocks, three dictionaries.
fn xz_blocks_of(level: Level) -> (u64, u64) {
    let dictionary = dictionary_of(level);
    let own = (3 * dictionary).max(1 << 20);
    (dictionary.min(own), own)
}

/// 7z's dictionary at `level`, and its own blocks
/// (`Lzma2Blocks::Auto`): four dictionaries, from 1 MiB to 256 MiB.
fn seven_zip_blocks_of(level: Level) -> (u64, u64) {
    let dictionary = dictionary_of(level);
    let own = (4 * dictionary).clamp(1 << 20, 256 << 20);
    (dictionary.min(own), own)
}

/// What a tar archive keeps of each file for a backup, to be restored on the computer
/// it came from: each is left out by default, as an archive to share has no use for
/// it and would tell others the names of the users.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Keep {
    /// Its owner and group, by number and by name (PAX's `uid`, `uname`, `gid`,
    /// `gname`), where they are 0 and none otherwise. Unix only.
    pub owners: bool,
    /// Its extended attributes, all of them, the ACLs Linux keeps among them and
    /// the Finder's on macOS (PAX's `SCHILY.xattr.` records, as GNU tar and bsdtar
    /// write them). Linux and macOS only.
    pub xattrs: bool,
}

impl Keep {
    /// Whether it keeps anything.
    pub fn any(self) -> bool {
        self.owners || self.xattrs
    }
}

/// The times the files of a new archive are given.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Times {
    /// Those they were last changed at.
    #[default]
    AsTheyAre,
    /// Those, but one later than this, in seconds since 1970, is this: a
    /// reproducible build's `SOURCE_DATE_EPOCH`, as GNU tar's `--clamp-mtime`.
    AtMost(i64),
    /// This one for every file, as GNU tar's `--mtime`.
    All(i64),
}

impl Times {
    /// The time a file last changed at `secs` is given.
    pub fn of(self, secs: i64) -> i64 {
        match self {
            Times::AsTheyAre => secs,
            Times::AtMost(latest) => secs.min(latest),
            Times::All(time) => time,
        }
    }
}

/// How a new archive is made.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct NewArchive {
    pub kind: Kind,
    pub level: Level,
    /// Encrypts its files with AES-256: WinZip's in a ZIP archive, 7-Zip's in
    /// a 7z one. Any other kind is encrypted whole with age, to this passphrase,
    /// and a ZIP or 7z one too with `whole_with_age`.
    pub password: Option<Vec<u8>>,
    /// Encrypts it whole with age, to these keys: `age1…`, `age1pq1…` or
    /// `ssh-ed25519 …`, post-quantum ones or others but not both. Not for a ZIP or 7z
    /// archive unless `whole_with_age`, and not with a password, as age has a
    /// passphrase alone.
    pub age_keys: Vec<String>,
    /// Encrypts a ZIP or 7z archive whole with age, as any other kind is, in place
    /// of its own AES-256: its names are hidden too, and it can go to keys.
    pub whole_with_age: bool,
    /// Encrypts a 7z archive's names too, with its password.
    pub hide_names: bool,
    /// The extensions of files held as they are, whatever their case: those
    /// compressed already, as [`COMPRESSED_EXTENSIONS`], which squeezed again
    /// take time and come out no smaller. A ZIP archive stores each; a 7z one
    /// puts them after the rest, in a folder of their own, stored.
    pub stored_extensions: Vec<String>,
    /// Leaves out what only a Mac uses, under the paths chosen: `.DS_Store`,
    /// `__MACOSX` and the `._` files of resource forks.
    pub skip_mac_files: bool,
    /// Cuts it into `name.zip.001`, `name.zip.002`, … of this many bytes, as
    /// 7-Zip does.
    pub part_size: Option<u64>,
    /// What a tar archive keeps for a backup besides names, data, permissions, times
    /// and links. ZIP and 7z have no place for it, nor does one file alone.
    pub keep: Keep,
    /// The times the files are given in it, for an archive that comes out the same
    /// from the same files whenever they were last changed.
    pub times: Times,
    /// Bound for a storage across a network, it goes straight there if its files
    /// come to more than this many bytes and the storage can take it that way. It
    /// goes through a temporary file otherwise, and if this is none.
    pub stream_above: Option<u64>,
    /// How a `.tar.zst` or `.zst` is cut into frames.
    pub frames: Frames,
    /// How LZMA2 is cut into blocks, in a `.tar.xz`, a `.xz` or a 7z archive.
    pub blocks: Blocks,
}

/// How `new` is encrypted, if it is.
fn method_of(new: &NewArchive) -> Option<Method> {
    match (&new.password, new.kind) {
        (Some(_), _) if with_age(new) => Some(Method::AgePassphrase),
        (Some(_), Kind::Zip) => Some(Method::ZipAes256),
        (Some(_), Kind::SevenZip) => Some(Method::SevenZipAes256),
        (Some(_), _) => Some(Method::AgePassphrase),
        (None, _) if !new.age_keys.is_empty() => Some(Method::AgeKey),
        (None, _) => None,
    }
}

/// What a record of making `new` of `paths` says: its kind, how it is
/// encrypted (never the password), and what goes in.
pub(crate) fn making(paths: &[PathBuf], new: &NewArchive) -> Map<String, Value> {
    audit::fields([
        ("kind", json!(format!("{:?}", new.kind).to_lowercase())),
        ("level", json!(format!("{:?}", new.level).to_lowercase())),
        ("encryption", json!(method_of(new).map(Method::name))),
        ("hide_names", json!(new.hide_names)),
        (
            "recipients",
            if new.age_keys.is_empty() {
                Value::Null
            } else {
                json!(new.age_keys)
            },
        ),
        ("part_size", json!(new.part_size)),
        ("keep_owners", json!(new.keep.owners)),
        ("keep_xattrs", json!(new.keep.xattrs)),
        ("paths", json!(paths.len())),
        (
            "path_names",
            audit::list(paths.iter().map(|p| audit::path_text(p))),
        ),
    ])
}

/// Why `policy` stops `new` being made, if it does: what the window asks before
/// it starts, and [`create`] and [`create_on`] whoever asks them.
pub fn refused_by(policy: &Policy, new: &NewArchive) -> Result<(), Refused> {
    policy.check_method(method_of(new))?;
    match &new.password {
        Some(password) => policy.check_password(password),
        None => Ok(()),
    }
}

/// [`refused_by`] this computer's policy, as an error.
pub(crate) fn obey_policy(new: &NewArchive) -> Result<(), Error> {
    refused_by(packwing_policy::current(), new).map_err(|why| Error::Failed(why.to_string()))
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
    // Encrypted whole with age, it is made in a temporary file and goes forwards.
    let forwards = new.kind != Kind::SevenZip || new.whole_with_age;
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
            .field("whole_with_age", &self.whole_with_age)
            .field("hide_names", &self.hide_names)
            .field("stored_extensions", &self.stored_extensions)
            .field("stream_above", &self.stream_above)
            .field("skip_mac_files", &self.skip_mac_files)
            .field("part_size", &self.part_size)
            .field("frames", &self.frames)
            .field("keep", &self.keep)
            .field("times", &self.times)
            .finish()
    }
}

/// What a selection of files and folders holds.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Measured {
    pub files: u64,
    pub bytes: u64,
    /// The names in it that some system can't use, as they go in the archive:
    /// Windows' characters and device names, and a name that differs from
    /// another only in case.
    pub not_portable: Vec<String>,
}

/// What goes in the archive, with the name it has there.
#[derive(Clone)]
struct Item {
    name: String,
    path: PathBuf,
    metadata: Metadata,
    /// The time it is given, in seconds since 1970: when it was last changed, or
    /// as [`NewArchive::times`] says.
    mtime: i64,
}

/// `path` and what is under it, its names under `name`, in order of name;
/// links are not followed, and what is neither a file, a folder nor a link
/// is left out. So is what was listed in a folder and is gone by the time it
/// is looked at, as another program's files may go while it runs: not a
/// path that was chosen (`chosen`), which is an error.
fn walk(
    path: &Path,
    name: String,
    items: &mut Vec<Item>,
    control: &Control,
    chosen: bool,
) -> Result<(), Error> {
    control.check()?;
    let read = |e: io::Error| {
        Error::Failed(tr!(
            "Could not read {path}: {e}",
            path = path.display(),
            e = e
        ))
    };
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(e) if !chosen && is_gone(&e) => {
            control.gone(path);
            return Ok(());
        }
        Err(e) => return Err(read(e)),
    };
    let kind = metadata.file_type();
    if kind.is_dir() {
        let listed = match fs::read_dir(path) {
            Ok(listed) => listed,
            Err(e) if !chosen && is_gone(&e) => {
                control.gone(path);
                return Ok(());
            }
            Err(e) => return Err(read(e)),
        };
        let mut children = listed.collect::<io::Result<Vec<_>>>().map_err(read)?;
        children.sort_by_key(|c| c.file_name());
        items.push(Item {
            name: format!("{name}/"),
            path: path.to_path_buf(),
            mtime: seconds(&metadata),
            metadata,
        });
        for child in children {
            let under = format!("{name}/{}", unicode_name(&child.path())?);
            walk(&child.path(), under, items, control, false)?;
        }
    } else if kind.is_file() || (cfg!(unix) && kind.is_symlink()) {
        items.push(Item {
            name,
            path: path.to_path_buf(),
            mtime: seconds(&metadata),
            metadata,
        });
    }
    Ok(())
}

/// Whether `e` says that what was to be read is gone: removed, or a folder
/// above it removed, since it was listed.
fn is_gone(e: &io::Error) -> bool {
    e.kind() == io::ErrorKind::NotFound
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
        walk(path, unicode_name(path)?, &mut items, control, true)?;
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
    let mut seen = std::collections::HashSet::new();
    for item in items {
        let unnamable = check_name_for(item.name.as_bytes(), NameRules::Portable)
            .is_err_and(|e| e.is_unnamable());
        if unnamable || !seen.insert(item.name.trim_end_matches('/').to_lowercase()) {
            measured.not_portable.push(item.name.clone());
        }
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
    /// What was gone by the time it was to be read, and left out.
    gone: Mutex<Vec<PathBuf>>,
    /// A ZIP archive's file was gone when it was read, which the archive
    /// cannot leave out once it is being written: it is made again without.
    zip_gone: AtomicBool,
    /// What a tar's compressor has taken and not yet compressed, as
    /// [`Squeezed::watch`] measures it: read, but not done.
    behind: Arc<AtomicU64>,
    /// The bytes last told, which the next never goes below.
    told: AtomicU64,
}

impl Tally {
    /// `item` left out, gone by the time it was to be read: told, and
    /// counted as done, as the total counted it.
    fn gone(&self, item: &Item) {
        let bytes = if item.metadata.is_file() {
            item.metadata.len()
        } else {
            0
        };
        self.gone_at(&item.path, bytes);
    }

    /// The file at `path`, of `bytes`, gone by the time it was to be read.
    fn gone_at(&self, path: &Path, bytes: u64) {
        self.gone
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(path.to_path_buf());
        self.control.gone(path);
        self.advance(1, bytes);
    }

    /// Tells how far it is while a compressor ends, its threads compressing
    /// what they hold: an error once the job is stopped, which [`why`] says.
    fn going(&self) -> io::Result<()> {
        self.advance(0, 0);
        self.control
            .check()
            .map_err(|_| io::Error::other("stopped"))
    }

    /// What was gone so far.
    fn gone_paths(&self) -> HashSet<PathBuf> {
        self.gone
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .iter()
            .cloned()
            .collect()
    }

    fn advance(&self, entries: u64, bytes: u64) {
        let mut progress = self.progress.lock().unwrap_or_else(PoisonError::into_inner);
        progress.entries += entries;
        progress.bytes += bytes;
        // What is read is done but for what a compressor in threads holds:
        // up to a block for each, as much as the whole input at xz's highest
        // level, which would show the end long before it is.
        let done = progress
            .bytes
            .saturating_sub(self.behind.load(Ordering::Relaxed));
        let told = self.told.fetch_max(done, Ordering::Relaxed).max(done);
        let mut shown = *progress;
        shown.bytes = told;
        self.control.report(&shown);
    }
}

/// A file's data, for hivepack::zip: opened when it is first read and closed
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
    fn failed(&self, e: &io::Error) -> hivepack::zip::Error {
        *self
            .tally
            .failed
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = Some(tr!(
            "Could not read {path}: {e}",
            path = self.path.display(),
            e = e
        ));
        hivepack::zip::Error::new(ErrorCode::Read, e.raw_os_error().unwrap_or(0))
    }
}

impl DataSource for DiskFile {
    fn stat(&self) -> hivepack::zip::Result<Stat<'static>> {
        Ok(Stat {
            valid: stat::SIZE | stat::MTIME,
            size: self.size,
            mtime: self.mtime,
            ..Stat::default()
        })
    }

    fn file_attributes(&self) -> hivepack::zip::Result<FileAttributes> {
        Ok(FileAttributes {
            valid: file_attributes::HOST_SYSTEM | file_attributes::EXTERNAL_FILE_ATTRIBUTES,
            host_system: self.attributes.0,
            external_file_attributes: self.attributes.1,
            ..FileAttributes::default()
        })
    }

    fn read_at(&self, offset: u64, buf: &mut [u8]) -> hivepack::zip::Result<usize> {
        if self.tally.control.check().is_err() {
            return Err(hivepack::zip::Error::new(ErrorCode::Cancelled, 0));
        }
        let left = self.size.saturating_sub(offset);
        let wanted = (buf.len() as u64).min(left) as usize;
        if wanted == 0 {
            return Ok(0);
        }
        let mut file = self.file.lock().unwrap_or_else(PoisonError::into_inner);
        if file.is_none() {
            match File::open(&self.path) {
                Ok(opened) => *file = Some(opened),
                Err(e) if is_gone(&e) && offset == 0 => {
                    // Its entry is written already: the archive is made again.
                    self.tally.zip_gone.store(true, Ordering::Relaxed);
                    self.tally.gone_at(&self.path, self.size);
                    return Err(self.failed(&e));
                }
                Err(e) => return Err(self.failed(&e)),
            }
        }
        let Some(open) = file.as_ref() else {
            return Ok(0);
        };
        let n = hivepack::zip::ReadAt::read_at(&open, offset, &mut buf[..wanted])
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
pub(crate) fn free_name<T>(
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

/// What a new archive asked for as `wanted` will be called on this computer:
/// `wanted`, or the first `name (2).zip` after it that is free, as [`create`] takes
/// it; for one in `parts`, free for its first part too. Said ahead: what is there
/// when it is made decides.
pub fn free_archive_path(wanted: &Path, parts: bool) -> PathBuf {
    free_name(wanted, |path| {
        if path.exists() || (parts && part_path(path, 0).exists()) {
            return Err(io::ErrorKind::AlreadyExists.into());
        }
        Ok(())
    })
    .map_or_else(|_| wanted.to_path_buf(), |(free, ())| free)
}

/// Writes `items` in the archive to `out`.
fn write_archive<W: Write + Seek>(
    items: Vec<Item>,
    new: &NewArchive,
    tally: &Arc<Tally>,
    out: W,
) -> Result<(), Error> {
    match new.kind {
        Kind::Zip | Kind::SevenZip if new.whole_with_age => write_sealed(items, new, tally, out),
        Kind::Zip => write_zip(items, new, tally, out),
        Kind::SevenZip => write_7z(items, new, tally, out),
        _ => write_squeezed(items, new, tally, out),
    }
}

/// A ZIP or 7z archive, encrypted whole with age: a ZIP one as it is made, a 7z
/// one, which goes back to its start, in a temporary file first that only its
/// owner can read, as one that is opened is.
fn write_sealed<W: Write>(
    items: Vec<Item>,
    new: &NewArchive,
    tally: &Arc<Tally>,
    mut out: W,
) -> Result<(), Error> {
    // With nothing to encrypt to, it is not encrypted.
    match age_recipients(new)? {
        Some(recipients) => {
            let mut encrypted = age_writer(out, &recipients)?;
            write_unsealed(items, new, tally, &mut encrypted)?;
            encrypted.finish()?;
        }
        None => write_unsealed(items, new, tally, &mut out)?,
    }
    Ok(())
}

/// The ZIP or 7z archive that [`write_sealed`] encrypts, with no password.
fn write_unsealed<W: Write>(
    items: Vec<Item>,
    new: &NewArchive,
    tally: &Arc<Tally>,
    out: &mut W,
) -> Result<(), Error> {
    let plain = NewArchive {
        password: None,
        age_keys: Vec::new(),
        whole_with_age: false,
        hide_names: false,
        ..new.clone()
    };
    if new.kind == Kind::Zip {
        let (mut za, _held) = new_zip(items, &plain, tally)?;
        return zip_closed(za.close_to_stream(out)?);
    }
    let scratch = crate::spool::scratch()?;
    write_7z(items, &plain, tally, &scratch.file)?;
    io::copy(&mut scratch.rewound()?, out)?;
    Ok(())
}

/// What encrypts what is written to `out` with age, to `recipients`.
pub(crate) fn age_writer<W: Write>(
    out: W,
    recipients: &[BoxedRecipient],
) -> Result<hivepack::age::Encryptor<W>, Error> {
    let recipients: Vec<&dyn Recipient> = recipients
        .iter()
        .map(|r| r.as_ref() as &dyn Recipient)
        .collect();
    // Random bytes given, as hivepack::age has its own source on Unix alone.
    let random: RandomSource =
        Arc::new(|buffer: &mut [u8]| getrandom::fill(buffer).map_err(io::Error::other));
    hivepack::age::encrypt(out, &recipients, Some(&random))
        .map_err(|e| Error::Failed(tr!("Could not encrypt: {e}", e = e.to_string())))
}

fn write_zip<W: Write + Seek>(
    items: Vec<Item>,
    new: &NewArchive,
    tally: &Arc<Tally>,
    out: W,
) -> Result<(), Error> {
    let (mut za, _held) = new_zip(items, new, tally)?;
    zip_closed(za.close_to(out)?)
}

/// What closing a new ZIP archive came to.
fn zip_closed(closed: Closed) -> Result<(), Error> {
    match closed {
        Closed::Written => Ok(()),
        Closed::Unchanged | Closed::Removed => Err(Error::Failed(tr!("Nothing to compress."))),
    }
}

/// What each thread compressing a ZIP archive may take: hivepack's 32 MiB of
/// what waits to be written, the compressor, and what encrypting adds.
const ZIP_THREAD_MEMORY: u64 = 34 << 20;

/// The memory making `new` takes with one thread, in bytes: what it takes
/// however little memory there is, as fewer threads cannot take less. Close
/// enough for what takes much: the levels of xz, 7z and zstd that take
/// hundreds of MiB, and more.
pub fn memory_needed(new: &NewArchive) -> u64 {
    match new.kind {
        Kind::SevenZip => seven_zip_options(
            &NewArchive {
                password: None,
                ..new.clone()
            },
            None,
        )
        .ok()
        .and_then(|(options, _)| options.memory_usage(1).ok())
        .unwrap_or(0),
        Kind::TarXz | Kind::Xz => stream_encoder_mt_memusage(&MtOptions {
            threads: 1,
            preset: new.level.xz(),
            check: hivepack::xz::Check::CRC64,
            ..MtOptions::default()
        }),
        Kind::TarZst | Kind::Zst if new.frames == Frames::Several => {
            frames_options(new).memory_usage(1).unwrap_or(0)
        }
        Kind::TarZst | Kind::Zst => {
            let mut cctx = hivepack::zstd::CCtx::new();
            let set = cctx
                .set_parameter(hivepack::zstd::CParameter::CompressionLevel(
                    new.level.zstd(),
                ))
                .is_ok();
            if set {
                cctx.estimate_memory(1).unwrap_or(0)
            } else {
                0
            }
        }
        Kind::TarBz2 | Kind::Bz2 => hivepack::bzip2::MtOptions {
            block_size_100k: new.level.bzip2(),
            ..hivepack::bzip2::MtOptions::default()
        }
        .memory_usage(1)
        .unwrap_or(0),
        Kind::Zip => ZIP_THREAD_MEMORY,
        Kind::TarGz | Kind::Gz => gzip_options(new).memory_usage(1).unwrap_or(0),
        _ => 0,
    }
}

/// `new`'s gzip, at its level, in one thread.
fn gzip_options(new: &NewArchive) -> hivepack::deflate::MtOptions {
    hivepack::deflate::MtOptions {
        level: new.level.deflate() as i32,
        ..hivepack::deflate::MtOptions::default()
    }
}

/// `new` at the highest level, at most its own, whose one thread takes no
/// more than `available`: its own where that fits, and the lowest where none
/// does. The archive is made so, so that it is made at all where a level
/// would take more memory than there is: the system would end packwing.
fn within(new: &NewArchive, available: u64) -> NewArchive {
    let mut fitted = new.clone();
    while memory_needed(&fitted) > available
        && let Some(lower) = fitted.level.lower()
    {
        fitted.level = lower;
    }
    fitted
}

/// [`within`] the memory available now.
pub fn within_memory(new: &NewArchive) -> NewArchive {
    within(new, crate::memory::available())
}

/// What to say before making `new` where even one thread of its level would
/// take more memory than is available now: the lower level it is made at,
/// or that none fits. Nothing where it fits.
pub fn memory_warning(new: &NewArchive) -> Option<String> {
    let available = crate::memory::available();
    let needed = memory_needed(new);
    if needed <= available {
        return None;
    }
    let fitted = within(new, available);
    Some(if fitted.level != new.level {
        tr!(
            "{level} takes about {needed} of memory, and {available} is free: it is made at {lower}, which takes about {less}.",
            level = new.level.name(),
            needed = crate::size(needed),
            available = crate::size(available),
            lower = fitted.level.name(),
            less = crate::size(memory_needed(&fitted))
        )
    } else {
        tr!(
            "This level takes about {needed} of memory, and {available} is free: the computer may slow down, or packwing stop.",
            needed = crate::size(needed),
            available = crate::size(available)
        )
    })
}

/// A new ZIP archive of `items`, to close, and the memory its threads hold
/// until then.
fn new_zip(
    items: Vec<Item>,
    new: &NewArchive,
    tally: &Arc<Tally>,
) -> Result<(Archive<&'static [u8]>, crate::memory::Held), Error> {
    let mut za = Archive::open(&[][..], OpenFlags(0))?;
    za.set_random_source(Some(Arc::new(|buffer: &mut [u8]| {
        getrandom::fill(buffer).map_err(io::Error::other)
    })));
    if new.password.is_some() {
        za.set_default_password(new.password.as_deref());
    }
    // Files compressed in threads ahead of the one written, the archive the
    // same whatever the threads: as many as the memory there is takes.
    let (threads, held) = crate::memory::hold_threads(crate::threads().count(), |n| {
        u64::from(n) * ZIP_THREAD_MEMORY
    });
    za.set_threads(hivepack::Threads::of(threads));
    // Deflate in blocks, as pigz does: one large file on every core, where
    // it took one; 0.84 MiB a thread more, within what each is held.
    za.set_deflate_in_blocks(true);
    let how = Squeeze {
        level: new.level,
        stored: &new.stored_extensions,
    };
    add_items(&mut za, items, how, new.password.is_some(), tally)?;
    Ok((za, held))
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
    squeeze(items, new, tally, age_writer(out, &recipients)?)?.finish()?;
    Ok(())
}

/// Whether what `new` is encrypted with, if anything, is age.
pub(crate) fn with_age(new: &NewArchive) -> bool {
    !matches!(new.kind, Kind::Zip | Kind::SevenZip) || new.whole_with_age
}

/// What a new archive is encrypted to with age, if it is: none for ZIP and 7z, which
/// have their own encryption, unless they are encrypted whole with age.
pub(crate) fn age_recipients(new: &NewArchive) -> Result<Option<Vec<BoxedRecipient>>, Error> {
    let bad = |e: String| Error::Failed(tr!("Could not encrypt: {e}", e = e));
    if !with_age(new) {
        if !new.age_keys.is_empty() {
            return Err(Error::Failed(tr!(
                "A ZIP or 7z archive takes age keys only when age encrypts it whole."
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
        (None, keys) if age_keys_mixed(keys) => Err(bad(age_keys_mixed_problem())),
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

/// Whether `key` is post-quantum: only its private key opens the file even with a
/// quantum computer, and only age 1.3 and newer read it.
pub fn age_key_is_post_quantum(key: &str) -> bool {
    key.starts_with("age1pq1")
}

/// Whether these keys are post-quantum and others together, which age refuses: the
/// others would leave the file open to a quantum computer.
fn age_keys_mixed(keys: &[String]) -> bool {
    keys.iter().any(|key| age_key_is_post_quantum(key))
        && !keys.iter().all(|key| age_key_is_post_quantum(key))
}

fn age_keys_mixed_problem() -> String {
    tr!(
        "Post-quantum keys can’t be used with other keys: the others would leave the file open to a quantum computer."
    )
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
    if age_keys_mixed(&keys) {
        return Err(age_keys_mixed_problem());
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
    let bytes = items
        .iter()
        .filter(|item| item.metadata.is_file())
        .map(|item| item.metadata.len())
        .sum();
    let mut out = Squeezed::new(new, Some(bytes), out)?;
    out.watch(Arc::clone(&tally.behind));
    if !new.kind.one_file() {
        return write_tar(items, new.keep, tally, out);
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
    let file = File::open(&item.path).map_err(|e| read_failed(&item.path, &e))?;
    pump(item, file, &mut out, tally, &mut vec![0; PIECE])?;
    Ok(out.finish_telling(&mut || tally.going())?)
}

/// How a ZIP archive's files are compressed.
#[derive(Clone, Copy)]
struct Squeeze<'a> {
    level: Level,
    /// See [`NewArchive::stored_extensions`].
    stored: &'a [String],
}

/// The extensions of files that are compressed already: what squeezing again
/// leaves no smaller. Not PDF or office documents, which often are.
#[rustfmt::skip]
pub const COMPRESSED_EXTENSIONS: &[&str] = &[
    // Pictures.
    "jpg", "jpeg", "png", "gif", "webp", "heic", "heif", "avif", "jxl",
    // Video. Not `ts`, which is TypeScript as often.
    "mp4", "m4v", "mov", "mkv", "webm", "avi", "wmv", "3gp", "mpg", "mpeg", "m2ts", "ogv", "flv",
    // Sound.
    "mp3", "m4a", "aac", "ogg", "oga", "opus", "flac", "wma", "amr",
    // Archives, and what is encrypted.
    "zip", "7z", "rar", "gz", "tgz", "bz2", "tbz2", "xz", "txz", "zst", "tzst", "lz4", "lzma",
    "lz", "br", "cab", "dmg", "age",
    // Packages, compressed inside.
    "pkg", "xar", "deb", "rpm", "jar", "apk", "ipa", "whl", "nupkg", "appx", "msix", "snap",
];

/// Whether the file named `name` ends in one of `extensions`, whatever its case.
pub(crate) fn has_extension_in(name: &str, extensions: &[impl AsRef<str>]) -> bool {
    name.rsplit_once('.').is_some_and(|(_, ext)| {
        extensions
            .iter()
            .any(|known| known.as_ref().eq_ignore_ascii_case(ext))
    })
}

/// `items` in the ZIP archive `za`, to be read when it closes.
fn add_items<R: ReadAt>(
    za: &mut Archive<R>,
    items: Vec<Item>,
    how: Squeeze,
    encrypt: bool,
    tally: &Arc<Tally>,
) -> Result<(), Error> {
    for item in items {
        let mtime = item.mtime;
        let (system, external) = attributes(&item.metadata);
        let name = item.name.as_bytes();
        let index = if item.metadata.is_dir() {
            za.dir_add(name, Flags::ENC_UTF_8)?
        } else if item.metadata.file_type().is_symlink() {
            let Some(target) = target_of(&item, tally)? else {
                continue;
            };
            let index = za.file_add(
                name,
                BufferSource::with_mtime(target, mtime),
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
            if how.level == Level::Store || has_extension_in(&item.name, how.stored) {
                za.set_file_compression(index, i32::from(cm::STORE), 0)?;
            } else {
                za.set_file_compression(index, cm::DEFAULT, how.level.deflate())?;
            }
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

/// `item`, a file, opened to be read: none if it is gone, which `tally` is
/// told, so that it is left out before anything of it is written.
fn open_item(item: &Item, tally: &Tally) -> Result<Option<File>, Error> {
    match File::open(&item.path) {
        Ok(file) => Ok(Some(file)),
        Err(e) if is_gone(&e) => {
            tally.gone(item);
            Ok(None)
        }
        Err(e) => Err(read_failed(&item.path, &e)),
    }
}

/// `item`, a symbolic link, its target: none if it is gone, as
/// [`open_item`] says.
fn target_of(item: &Item, tally: &Tally) -> Result<Option<Vec<u8>>, Error> {
    match link_target(&item.path) {
        Ok(target) => Ok(Some(target)),
        Err(_) if fs::symlink_metadata(&item.path).is_err_and(|e| is_gone(&e)) => {
            tally.gone(item);
            Ok(None)
        }
        Err(e) => Err(e),
    }
}

fn read_failed(path: &Path, e: &io::Error) -> Error {
    Error::Failed(tr!(
        "Could not read {path}: {e}",
        path = path.display(),
        e = e
    ))
}

/// The data of `item`, a file, from `file` into `out`, a piece at a time: the
/// size it had when it was counted, as a tar archive has said already by then.
/// Reading it counts towards the progress, and is where a job stops.
fn pump(
    item: &Item,
    mut file: File,
    out: &mut impl Write,
    tally: &Tally,
    buf: &mut [u8],
) -> Result<(), Error> {
    let read = |e: io::Error| read_failed(&item.path, &e);
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

/// A time in seconds since 1970 as 7z has it: in 100 ns since 1601, which an
/// earlier one cannot be.
fn ntfs_time(secs: i64) -> Option<u64> {
    let since_1601 = secs.checked_add(11_644_473_600)?;
    u64::try_from(since_1601).ok()?.checked_mul(10_000_000)
}

/// How hivepack::sevenz writes the 7z archive `new` asks for.
/// The options of a 7z archive, of `bytes` if they are known: as
/// [`Blocks`] says of them, in small blocks.
pub(crate) fn seven_zip_options(
    new: &NewArchive,
    bytes: Option<u64>,
) -> Result<(hivepack::sevenz::WriteOptions, crate::memory::Held), Error> {
    let password = match &new.password {
        None => None,
        Some(p) => Some(
            String::from_utf8(p.clone())
                .map_err(|_| Error::Failed(tr!("A password has to be valid Unicode.")))?,
        ),
    };
    let method = match new.level {
        Level::Store => hivepack::sevenz::Method::Copy,
        level => hivepack::sevenz::Method::Lzma2 { preset: level.xz() },
    };
    let mut options = hivepack::sevenz::WriteOptions {
        method,
        encrypt_headers: new.hide_names && password.is_some(),
        password,
        random_source: Some(Arc::new(|buffer: &mut [u8]| {
            getrandom::fill(buffer).map_err(io::Error::other)
        })),
        // Blocks of 4 times the dictionary, each compressed in a thread and
        // decoded so, by packwing and 7-Zip alike: about 2% larger, and as
        // many times sooner as there are blocks and threads; smaller, if
        // asked, where those are fewer than the cores. The same archive
        // whatever the threads.
        lzma2_blocks: match bytes {
            Some(bytes) => new.blocks.seven_zip(new.level, bytes),
            None => hivepack::sevenz::Lzma2Blocks::Auto,
        },
        ..hivepack::sevenz::WriteOptions::default()
    };
    // As many threads as the memory there is takes: at the highest level,
    // each takes more than a GiB.
    // Options it refuses are refused when it is made: nothing is held for them.
    let (threads, held) = crate::memory::hold_threads(crate::threads().count(), |n| {
        options.memory_usage(n).unwrap_or(0)
    });
    options.threads = hivepack::Threads::of(threads);
    options.memory_limit = held.bytes();
    Ok((options, held))
}

fn write_7z<W: Write + Seek>(
    items: Vec<Item>,
    new: &NewArchive,
    tally: &Tally,
    out: W,
) -> Result<(), Error> {
    // What goes into LZMA2: what is compressed already is stored apart.
    let squeezed_bytes = items
        .iter()
        .filter(|item| {
            item.metadata.is_file()
                && (new.level == Level::Store
                    || !has_extension_in(&item.name, &new.stored_extensions))
        })
        .map(|item| item.metadata.len())
        .sum();
    let (mut options, _held) = seven_zip_options(new, Some(squeezed_bytes))?;
    // LZMA2's blocks are compressed in threads, up to a block for each taken
    // ahead of what is compressed: what they hold is not done, and the
    // writer, which may wait long for a block, is stopped from here.
    let stop = Arc::new(AtomicBool::new(false));
    options.stop = Some(Arc::clone(&stop));
    let archive = hivepack::sevenz::Writer::with_options(out, options)?;
    let progress = archive.progress();
    let mut archive = Holding {
        archive,
        progress: progress.clone(),
        behind: &tally.behind,
    };
    let ending = AtomicBool::new(false);
    std::thread::scope(|scope| {
        scope.spawn(|| {
            while !ending.load(Ordering::Relaxed) {
                std::thread::sleep(TICK_EVERY);
                let held = progress.taken().saturating_sub(progress.compressed());
                tally.behind.store(held, Ordering::Relaxed);
                if tally.going().is_err() {
                    stop.store(true, Ordering::Relaxed);
                }
            }
        });
        let written = fill_7z(&items, new, tally, &mut archive).and_then(|()| {
            archive.archive.finish()?;
            Ok(())
        });
        ending.store(true, Ordering::Relaxed);
        written
    })
}

/// A 7z writer that keeps in `behind`, as it is given data, what its threads
/// hold and have not compressed.
struct Holding<'a, W: Write + Seek> {
    archive: hivepack::sevenz::Writer<W>,
    progress: hivepack::sevenz::WriteProgress,
    behind: &'a AtomicU64,
}

impl<W: Write + Seek> Write for Holding<'_, W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.archive.write(buf)?;
        let held = self
            .progress
            .taken()
            .saturating_sub(self.progress.compressed());
        self.behind.store(held, Ordering::Relaxed);
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.archive.flush()
    }
}

/// Writes `items` into `archive`, but for its end.
fn fill_7z<W: Write + Seek>(
    items: &[Item],
    new: &NewArchive,
    tally: &Tally,
    holding: &mut Holding<'_, W>,
) -> Result<(), Error> {
    let mut buf = vec![0; PIECE];
    // What is compressed already comes last, stored in a folder of its own, and
    // the rest stays together in one, which compresses it best.
    let (stored, squeezed): (Vec<&Item>, Vec<&Item>) = items.iter().partition(|item| {
        new.level != Level::Store
            && item.metadata.is_file()
            && has_extension_in(&item.name, &new.stored_extensions)
    });
    for (i, item) in squeezed.iter().chain(&stored).enumerate() {
        let archive = &mut holding.archive;
        if i == squeezed.len() {
            archive.set_method(hivepack::sevenz::Method::Copy)?;
        }
        // 7-Zip keeps a Unix mode above its own attributes, marked by 0x8000.
        let attributes = attributes(&item.metadata).1 | 0x8000;
        let entry = hivepack::sevenz::NewEntry {
            name: item.name.trim_end_matches('/').to_owned(),
            attributes: Some(attributes),
            mtime: ntfs_time(item.mtime),
            ctime: None,
        };
        if item.metadata.is_dir() {
            archive.add_directory(entry)?;
        } else if item.metadata.file_type().is_symlink() {
            let Some(target) = target_of(item, tally)? else {
                continue;
            };
            archive.start_file(entry)?;
            holding.write_all(&target)?;
            tally.advance(1, 0);
        } else {
            let Some(file) = open_item(item, tally)? else {
                continue;
            };
            archive.start_file(entry)?;
            pump(item, file, holding, tally, &mut buf)?;
        }
    }
    Ok(())
}

/// Writes a tar archive of `items` to `out`, each file that has other names (a
/// hard link) once, its other names linked to the first, and what `keep` says
/// besides.
fn write_tar<W: Write>(
    items: Vec<Item>,
    keep: Keep,
    tally: &Tally,
    out: Squeezed<W>,
) -> Result<W, Error> {
    let mut tar = hivepack::tar::Writer::new(out);
    let mut buf = vec![0; PIECE];
    let mut kept = Kept::default();
    // The first name of each file with more than one, by its device and inode.
    let mut first_names: HashMap<(u64, u64), &str> = HashMap::new();
    for item in &items {
        let kind = item.metadata.file_type();
        let link = kind.is_symlink();
        let mut header = hivepack::tar::Header {
            name: item.name.as_bytes().to_vec(),
            mode: i64::from((attributes(&item.metadata).1 >> 16) & 0o7777),
            mod_time: Some(hivepack::tar::Time {
                secs: item.mtime,
                nanos: 0,
            }),
            ..hivepack::tar::Header::default()
        };
        let key = hard_link_key(&item.metadata);
        let hard_link = key.and_then(|key| first_names.get(&key).copied());
        // What is read of it, before anything of it is written: nothing if it
        // is gone, as another program's files may go while it runs.
        let mut file = None;
        if link {
            let Some(target) = target_of(item, tally)? else {
                continue;
            };
            header.linkname = target;
        } else if kind.is_file() && hard_link.is_none() {
            let Some(opened) = open_item(item, tally)? else {
                continue;
            };
            file = Some(opened);
        }
        kept.add(item, keep, &mut header)?;
        if let (Some(key), None) = (key, hard_link) {
            first_names.insert(key, &item.name);
        }
        if kind.is_dir() {
            header.typeflag = hivepack::tar::TYPE_DIR;
        } else if link {
            header.typeflag = hivepack::tar::TYPE_SYMLINK;
        } else if let Some(first) = hard_link {
            header.typeflag = hivepack::tar::TYPE_LINK;
            header.linkname = first.as_bytes().to_vec();
        } else {
            header.typeflag = hivepack::tar::TYPE_REG;
            header.size = i64::try_from(item.metadata.len()).map_err(|_| {
                Error::Failed(tr!("{path} is too big.", path = item.path.display()))
            })?;
        }
        tar.write_header(&header)?;
        if link {
            tally.advance(1, 0);
        } else if hard_link.is_some() {
            // Its bytes were counted with the files', and are in the first name's.
            tally.advance(1, item.metadata.len());
        } else if let Some(file) = file {
            pump(item, file, &mut tar, tally, &mut buf)?;
        }
    }
    tar.close()?;
    Ok(tar.into_inner().finish_telling(&mut || tally.going())?)
}

/// What tells the names of one file apart from those of another: its device and
/// inode, for a file with more than one name. None for one with a single name, a
/// folder, a link, and on Windows.
fn hard_link_key(metadata: &Metadata) -> Option<(u64, u64)> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        (metadata.is_file() && metadata.nlink() > 1).then(|| (metadata.dev(), metadata.ino()))
    }
    #[cfg(not(unix))]
    {
        let _ = metadata;
        None
    }
}

/// The names of the owners and groups found so far, which the system is asked
/// for once each. Asked on Unix only: elsewhere a tar archive keeps none.
#[derive(Default)]
#[cfg_attr(not(unix), allow(dead_code))]
struct Kept {
    users: HashMap<u32, Vec<u8>>,
    groups: HashMap<u32, Vec<u8>>,
}

impl Kept {
    /// Puts in `header` what `keep` says of `item`: its owner and group, and its
    /// extended attributes.
    #[cfg(unix)]
    fn add(
        &mut self,
        item: &Item,
        keep: Keep,
        header: &mut hivepack::tar::Header,
    ) -> Result<(), Error> {
        use nix::unistd::{Gid, Group, Uid, User};
        use std::os::unix::fs::MetadataExt;
        if keep.owners {
            let (uid, gid) = (item.metadata.uid(), item.metadata.gid());
            header.uid = i64::from(uid);
            header.gid = i64::from(gid);
            header.uname = self
                .users
                .entry(uid)
                .or_insert_with(|| {
                    User::from_uid(Uid::from_raw(uid))
                        .ok()
                        .flatten()
                        .map(|user| user.name.into_bytes())
                        .unwrap_or_default()
                })
                .clone();
            header.gname = self
                .groups
                .entry(gid)
                .or_insert_with(|| {
                    Group::from_gid(Gid::from_raw(gid))
                        .ok()
                        .flatten()
                        .map(|group| group.name.into_bytes())
                        .unwrap_or_default()
                })
                .clone();
        }
        if keep.xattrs {
            header.xattrs = xattrs_of(&item.path)?;
        }
        Ok(())
    }

    #[cfg(not(unix))]
    fn add(&mut self, _: &Item, _: Keep, _: &mut hivepack::tar::Header) -> Result<(), Error> {
        Ok(())
    }
}

/// The extended attributes of the file at `path`, a link's own and not its
/// target's: none where its file system has none.
#[cfg(unix)]
fn xattrs_of(path: &Path) -> Result<hivepack::tar::Records, Error> {
    use std::os::unix::ffi::OsStrExt;
    let read = |e: io::Error| {
        Error::Failed(tr!(
            "Could not read the extended attributes of {path}: {e}",
            path = path.display(),
            e = e
        ))
    };
    let mut records = hivepack::tar::Records::new();
    let names = match xattr::list(path) {
        Ok(names) => names,
        Err(e) if e.kind() == io::ErrorKind::Unsupported => return Ok(records),
        Err(e) => return Err(read(e)),
    };
    for name in names {
        match xattr::get(path, &name) {
            Ok(Some(value)) => {
                records.insert(name.as_bytes().to_vec(), value);
            }
            // Gone since it was listed.
            Ok(None) => {}
            Err(e) => return Err(read(e)),
        }
    }
    Ok(records)
}

/// How much is read of a file at a time, and made of what compresses.
pub(crate) const PIECE: usize = 1 << 16;

/// What compresses a tar archive.
enum Coder {
    /// Written as it is.
    Plain,
    /// In blocks compressed in threads, as pigz writes them: one stream,
    /// the same whatever the threads.
    Gzip(Box<DeflateMt>),
    Xz(Box<Stream>),
    Zstd(Box<hivepack::zstd::CCtx>),
    /// In frames of 32 MiB each, compressed in threads, and their seek table.
    ZstdFrames(Box<hivepack::zstd::CompressFrames>),
    /// In streams of a block each, one after another, which `bzip2 -d`
    /// reads as one and packwing reads a stream in each thread.
    Bzip2(Box<hivepack::bzip2::CompressMt>),
    /// In independent blocks of 4 MB compressed in threads, as `lz4 -T`
    /// writes them, with a checksum of the content. What it makes waits in
    /// its `Vec`. None once it has ended.
    Lz4(Option<Box<hivepack::lz4::EncoderMt<Vec<u8>>>>),
}

/// A stream compressed into `out`, whose end [`Squeezed::finish`] writes.
pub(crate) struct Squeezed<W> {
    out: W,
    coder: Coder,
    buf: Vec<u8>,
    /// The memory its threads hold, until it is dropped.
    _held: Option<crate::memory::Held>,
    /// Where what it has taken and not yet compressed is kept, if anywhere.
    behind: Option<Arc<AtomicU64>>,
    /// What it has taken.
    taken: u64,
    /// When `tick` was last called, while it ends.
    ticked: Option<Instant>,
}

/// How often a [`Squeezed`] that is watched tells how far it is while it
/// ends, its threads compressing what they hold.
const TICK_EVERY: Duration = Duration::from_millis(50);

/// What a [`Squeezed`] calls to tell how far it is: an error stops it.
type Tick<'a> = dyn FnMut() -> io::Result<()> + 'a;

/// As many threads as [`crate::use_threads`] allows.
fn threads() -> u32 {
    crate::threads().count()
}

/// The frames of `new`'s zstd, at its level, with a checksum each.
fn frames_options(new: &NewArchive) -> hivepack::zstd::FramesOptions {
    hivepack::zstd::FramesOptions {
        level: new.level.zstd(),
        checksum: true,
        ..hivepack::zstd::FramesOptions::default()
    }
}

impl<W: Write> Squeezed<W> {
    /// A stream of `new`'s kind into `out`, of `bytes` if they are known: as
    /// [`Frames::several`] says of them, in frames.
    pub(crate) fn new(new: &NewArchive, bytes: Option<u64>, out: W) -> Result<Squeezed<W>, Error> {
        let failed = |e: String| Error::Failed(tr!("Could not start compressing: {e}", e = e));
        let mut held = None;
        let coder = match new.kind {
            Kind::Tar => Coder::Plain,
            Kind::TarBz2 | Kind::Bz2 => {
                let mut options = hivepack::bzip2::MtOptions {
                    block_size_100k: new.level.bzip2(),
                    ..hivepack::bzip2::MtOptions::default()
                };
                let (threads, holds) = crate::memory::hold_threads(threads(), |n| {
                    options.memory_usage(n).unwrap_or(0)
                });
                options.threads = hivepack::Threads::of(threads);
                options.memory_limit = holds.bytes();
                held = Some(holds);
                Coder::Bzip2(Box::new(
                    hivepack::bzip2::CompressMt::with_options(&options)
                        .map_err(|e| failed(e.to_string()))?,
                ))
            }
            Kind::TarLz4 | Kind::Lz4 => {
                let mut options = hivepack::lz4::MtOptions {
                    level: new.level.lz4(),
                    ..hivepack::lz4::MtOptions::default()
                };
                // As many threads as the memory there is takes: each holds
                // two blocks of 4 MB and what they compress to.
                let (threads, holds) =
                    crate::memory::hold_threads(threads(), |n| options.memory_usage(n));
                options.threads = hivepack::Threads::of(threads);
                options.memory_limit = holds.bytes();
                held = Some(holds);
                Coder::Lz4(Some(Box::new(
                    hivepack::lz4::EncoderMt::new(Vec::new(), &options)
                        .map_err(|e| failed(e.to_string()))?,
                )))
            }
            Kind::TarXz | Kind::Xz => {
                let mut options = MtOptions {
                    threads: threads(),
                    preset: new.level.xz(),
                    check: hivepack::xz::Check::CRC64,
                    // A block's output is read once it is done, which at the
                    // highest level may be many seconds: back every so often
                    // meanwhile, to tell how far it is and to hear a stop.
                    timeout: TICK_EVERY.as_millis() as u32,
                    ..MtOptions::default()
                };
                if let Some(bytes) = bytes {
                    options.block_size = new.blocks.xz(new.level, bytes);
                }
                // As many threads as the memory there is takes: each takes
                // about three dictionaries, the highest level's 64 MiB.
                let (threads, holds) = crate::memory::hold_threads(options.threads, |n| {
                    stream_encoder_mt_memusage(&MtOptions {
                        threads: n,
                        ..options.clone()
                    })
                });
                options.threads = threads;
                held = Some(holds);
                Coder::Xz(Box::new(
                    Stream::stream_encoder_mt(&options).map_err(|e| failed(e.to_string()))?,
                ))
            }
            Kind::TarZst | Kind::Zst if new.frames.several(bytes) => {
                let mut options = frames_options(new);
                // As many threads as the memory there is takes: each holds a
                // frame and what it compresses to, and a context.
                let (threads, holds) = crate::memory::hold_threads(threads(), |n| {
                    options.memory_usage(n).unwrap_or(0)
                });
                options.threads = hivepack::Threads::of(threads);
                options.memory_limit = holds.bytes();
                held = Some(holds);
                Coder::ZstdFrames(Box::new(
                    hivepack::zstd::CompressFrames::new(&options)
                        .map_err(|e| failed(e.to_string()))?,
                ))
            }
            Kind::TarZst | Kind::Zst => {
                let mut cctx = hivepack::zstd::CCtx::new();
                for parameter in [
                    hivepack::zstd::CParameter::CompressionLevel(new.level.zstd()),
                    hivepack::zstd::CParameter::ChecksumFlag(true),
                    hivepack::zstd::CParameter::NbWorkers(threads()),
                ] {
                    cctx.set_parameter(parameter)
                        .map_err(|e| failed(e.to_string()))?;
                }
                // As many workers as the memory there is takes: at the
                // highest level, each takes about 220 MiB.
                let (workers, holds) = crate::memory::hold_threads(threads(), |n| {
                    cctx.estimate_memory(n).unwrap_or(0)
                });
                cctx.set_parameter(hivepack::zstd::CParameter::NbWorkers(workers))
                    .map_err(|e| failed(e.to_string()))?;
                cctx.set_workers_memory_limit(holds.bytes());
                held = Some(holds);
                Coder::Zstd(Box::new(cctx))
            }
            _ => {
                let mut options = gzip_options(new);
                let (threads, holds) = crate::memory::hold_threads(threads(), |n| {
                    options.memory_usage(n).unwrap_or(0)
                });
                options.threads = hivepack::Threads::of(threads);
                options.memory_limit = holds.bytes();
                held = Some(holds);
                Coder::Gzip(Box::new(
                    DeflateMt::new(&options).map_err(|e| failed(e.to_string()))?,
                ))
            }
        };
        Ok(Squeezed {
            out,
            coder,
            buf: vec![0; PIECE],
            _held: held,
            behind: None,
            taken: 0,
            ticked: None,
        })
    }

    /// Keeps in `behind` what it has taken and not yet compressed, measured
    /// as it goes, for a progress to leave out.
    pub(crate) fn watch(&mut self, behind: Arc<AtomicU64>) {
        self.behind = Some(behind);
    }

    /// What it has compressed of what it has taken: what its threads hold
    /// is not. The others compress what they take as they take it, or near
    /// enough.
    fn compressed_now(&self) -> u64 {
        match &self.coder {
            Coder::Xz(stream) => stream.get_progress().0,
            Coder::Zstd(cctx) => cctx.get_frame_progression().consumed,
            Coder::ZstdFrames(frames) => frames.consumed(),
            Coder::Plain | Coder::Gzip(_) | Coder::Bzip2(_) | Coder::Lz4(_) => self.taken,
        }
    }

    /// If it is watched, keeps how far behind it is, and when it is time,
    /// calls `tick`, whose error it returns. Measured at each call: a few
    /// locks, the threads' own.
    fn measure(&mut self, tick: &mut Tick<'_>) -> io::Result<()> {
        let Some(behind) = &self.behind else {
            return Ok(());
        };
        let compressed = self.compressed_now().min(self.taken);
        behind.store(self.taken - compressed, Ordering::Relaxed);
        if self.ticked.is_none_or(|at| at.elapsed() >= TICK_EVERY) {
            self.ticked = Some(Instant::now());
            tick()?;
        }
        Ok(())
    }

    /// Compresses `input`, and with `finish`, ends the stream: `tick` called
    /// as [`Squeezed::measure`] says.
    fn code(&mut self, input: &[u8], finish: bool, tick: &mut Tick<'_>) -> io::Result<()> {
        self.taken += input.len() as u64;
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
                    use hivepack::bzip2::{Action, InBuffer, OutBuffer, Status};
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
                    use hivepack::deflate::{InBuffer, OutBuffer, Status};
                    let mut output = OutBuffer::new(&mut self.buf);
                    let mut from = InBuffer::new(&input[at..]);
                    let flush = if finish {
                        Flush::Finish
                    } else {
                        Flush::NoFlush
                    };
                    // It reads all of the input, or fills the output.
                    let status = stream
                        .deflate(&mut output, &mut from, flush)
                        .map_err(|e| failed(e.to_string()))?;
                    at += from.pos;
                    let (made, full) = (output.pos, output.pos == output.dst.len());
                    let done =
                        status == Status::StreamEnd || (!finish && at == input.len() && !full);
                    (made, done)
                }
                Coder::Xz(stream) => {
                    use hivepack::xz::{Action, InBuffer, OutBuffer, Status};
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
                Coder::ZstdFrames(frames) => {
                    use hivepack::zstd::{EndDirective, InBuffer, OutBuffer};
                    let mut output = OutBuffer::new(&mut self.buf);
                    let mut from = InBuffer::new(&input[at..]);
                    let directive = if finish {
                        EndDirective::End
                    } else {
                        EndDirective::Continue
                    };
                    let left = frames
                        .compress_stream(&mut output, &mut from, directive)
                        .map_err(|e| failed(e.to_string()))?;
                    at += from.pos;
                    let done = if finish { left == 0 } else { at == input.len() };
                    (output.pos, done)
                }
                Coder::Zstd(cctx) => {
                    use hivepack::zstd::{EndDirective, InBuffer, OutBuffer};
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
            self.measure(tick)?;
            if done {
                return Ok(());
            }
        }
    }

    pub(crate) fn finish(self) -> io::Result<W> {
        self.finish_telling(&mut || Ok(()))
    }

    /// Ends the stream, calling `tick` now and then while the threads
    /// compress what they hold, and once more at the end, when nothing is
    /// behind: its error is returned, and the stream left unfinished.
    pub(crate) fn finish_telling(mut self, tick: &mut Tick<'_>) -> io::Result<W> {
        self.ticked = None;
        self.code(&[], true, tick)?;
        if let Some(behind) = &self.behind {
            behind.store(0, Ordering::Relaxed);
        }
        tick()?;
        self.out.flush()?;
        Ok(self.out)
    }
}

impl<W: Write> Write for Squeezed<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // What is read is told as it is read.
        self.code(buf, false, &mut || Ok(()))?;
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
    add_with(paths, archive, folder, OnExisting::Stop, &[], keys, control)
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
    let job = audit::begin(
        "delete",
        audit::fields([
            ("archive", json!(audit::path_text(archive))),
            ("picked", json!(picked.len())),
            (
                "picked_names",
                audit::list(picked.iter().map(|p| p.join("/"))),
            ),
        ]),
    );
    let result = delete_in(archive, picked, keys);
    audit::end(job, &result, |_| Map::new());
    result
}

fn delete_in(archive: &Path, picked: &[Vec<String>], keys: &Keys) -> Result<(), Error> {
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
    let job = audit::begin(
        "rename",
        audit::fields([
            ("archive", json!(audit::path_text(archive))),
            ("from_name", json!(from.join("/"))),
            ("to_name", json!(to)),
        ]),
    );
    let result = rename_in(archive, from, to, keys);
    audit::end(job, &result, |_| Map::new());
    result
}

fn rename_in(archive: &Path, from: &[String], to: &str, keys: &Keys) -> Result<(), Error> {
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
/// password. Files with `stored_extensions` go in as they are, as
/// [`NewArchive::stored_extensions`] says.
/// If this fails or is stopped, the archive is as it was.
pub fn add_with(
    paths: &[PathBuf],
    archive: &Path,
    folder: &[String],
    on_existing: OnExisting,
    stored_extensions: &[String],
    keys: &Keys,
    control: &Control,
) -> Result<(), Error> {
    let job = audit::begin(
        "add",
        audit::fields([
            ("archive", json!(audit::path_text(archive))),
            ("into", json!(folder.join("/"))),
            (
                "on_existing",
                json!(format!("{on_existing:?}").to_lowercase()),
            ),
            ("paths", json!(paths.len())),
            (
                "path_names",
                audit::list(paths.iter().map(|p| audit::path_text(p))),
            ),
        ]),
    );
    let result = add_into(
        paths,
        archive,
        folder,
        on_existing,
        stored_extensions,
        keys,
        control,
    );
    audit::end(job, &result, |_| Map::new());
    result
}

fn add_into(
    paths: &[PathBuf],
    archive: &Path,
    folder: &[String],
    on_existing: OnExisting,
    stored_extensions: &[String],
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
    let policy = packwing_policy::current();
    let obeying = policy.check_adding(encrypted.is_some()).and_then(|()| {
        // What is added to encrypted files is encrypted as they are.
        match encrypted {
            Some(_) => policy.check_method(Some(Method::ZipAes256)),
            None => Ok(()),
        }
    });
    obeying.map_err(|why| Error::Failed(why.to_string()))?;
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
        gone: Mutex::new(Vec::new()),
        zip_gone: AtomicBool::new(false),
        behind: Arc::new(AtomicU64::new(0)),
        told: AtomicU64::new(0),
    });
    control.report(&progress);

    let expected = za.get_num_entries(Flags::NONE) + new.len() as u64 - replaced.len() as u64;
    for index in replaced {
        za.delete(index)?;
    }
    let how = Squeeze {
        level: Level::default(),
        stored: stored_extensions,
    };
    // As a new archive's: the files compressed in threads ahead of the one
    // written, a large one deflated in blocks on all of them, as many as the
    // memory there is takes, held until the archive is written.
    let (threads, _held) = crate::memory::hold_threads(crate::threads().count(), |n| {
        u64::from(n) * ZIP_THREAD_MEMORY
    });
    za.set_threads(hivepack::Threads::of(threads));
    za.set_deflate_in_blocks(true);
    if let Err(e) = add_items(&mut za, new, how, encrypted.is_some(), &tally) {
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
    let mut fields = making(paths, new);
    fields.insert("destination".into(), json!(audit::path_text(wanted)));
    let job = audit::begin("create", fields);
    let result = (|| {
        // Refused before anything is read.
        obey_policy(new)?;
        age_recipients(new)?;
        // At a level whose one thread the memory there is takes.
        let new = &within_memory(new);
        // Before the archive is made, so that it is not among them.
        create_of(chosen_items(paths, new, control)?, wanted, new, control)
    })();
    audit::end(job, &result, |made| {
        audit::fields([("made", json!(audit::path_text(made)))])
    });
    result
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
    for item in &mut items {
        item.mtime = new.times.of(item.mtime);
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
        gone: Mutex::new(Vec::new()),
        zip_gone: AtomicBool::new(false),
        behind: Arc::new(AtomicU64::new(0)),
        told: AtomicU64::new(0),
    })
}

/// [`create`], its `items` found.
fn create_of(
    items: Vec<Item>,
    wanted: &Path,
    new: &NewArchive,
    control: &Control,
) -> Result<PathBuf, Error> {
    let mut items = items;
    let mut attempt = 0;
    loop {
        attempt += 1;
        if items.is_empty() {
            return Err(Error::Failed(tr!("Nothing to compress.")));
        }
        let tally = tally_of(&items, control);
        // A ZIP archive may be made again, without what was gone.
        let again = new.kind == Kind::Zip && attempt < ZIP_ATTEMPTS;
        let these = if again {
            items.clone()
        } else {
            std::mem::take(&mut items)
        };
        let made = write_new(wanted, new, |out| {
            why(write_archive(these, new, &tally, out), &tally)
        });
        match made {
            Err(_) if again && tally.zip_gone.load(Ordering::Relaxed) => {
                let gone = tally.gone_paths();
                items.retain(|item| !gone.contains(&item.path));
            }
            made => return made.map(|(made, _)| made),
        }
    }
}

/// How many times a ZIP archive is made at most, each time without the files
/// that were gone when it was read: another program's files may keep going.
const ZIP_ATTEMPTS: u32 = 3;

/// What can be written and gone back over: a new archive's file, or its parts.
pub(crate) trait WriteSeek: Write + Seek {}

impl<T: Write + Seek> WriteSeek for T {}

/// Writes a new archive with `write`, named after `wanted` or the first name after it
/// that is free, in parts as `new` says: where it went, the archive or its first part,
/// and each of its files. What `write` fails to make is taken away.
pub(crate) fn write_new(
    wanted: &Path,
    new: &NewArchive,
    write: impl FnOnce(&mut dyn WriteSeek) -> Result<(), Error>,
) -> Result<(PathBuf, Vec<PathBuf>), Error> {
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
            let written = write(&mut parts).and_then(|()| {
                parts.flush()?;
                Ok(())
            });
            if let Err(e) = written {
                parts.remove();
                return Err(e);
            }
            parts.open = None;
            // It fitted in one: an archive like any other.
            if parts.made == 1 && !base.exists() {
                fs::rename(parts.path(0), &base)?;
                return Ok((base.clone(), vec![base]));
            }
            let all = (0..parts.made).map(|part| parts.path(part)).collect();
            Ok((parts.path(0), all))
        }
        _ => {
            let (path, mut file) = free_name(wanted, |path| {
                OpenOptions::new().write(true).create_new(true).open(path)
            })?;
            let written = write(&mut file).and_then(|()| {
                file.sync_all()?;
                Ok(())
            });
            if let Err(e) = written {
                drop(file);
                let _ = fs::remove_file(&path);
                return Err(e);
            }
            Ok((path.clone(), vec![path]))
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
            Kind::Zip | Kind::SevenZip if new.whole_with_age => {
                write_sealed(items, new, &tally, out)
            }
            Kind::Zip => new_zip(items, new, &tally)
                .and_then(|(mut za, _held)| Ok(za.close_to_stream(out)?))
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
    let mut fields = making(paths, new);
    let mut path = folder.to_vec();
    path.push(name.to_owned());
    fields.insert("destination".into(), json!(audit::place(to, &path)));
    let job = audit::begin("create", fields);
    let result = create_on_storage(paths, to, folder, name, new, control, uploading);
    audit::end(job, &result, |made| {
        audit::fields([("made", json!(made.as_deref().map(audit::path_text)))])
    });
    result
}

fn create_on_storage(
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
    obey_policy(new)?;
    age_recipients(new)?;
    // At a level whose one thread the memory there is takes.
    let new = &within_memory(new);
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
    fn a_level_too_large_for_the_memory_is_lowered_until_it_fits() {
        const MB: u64 = 1_000_000;
        let new = |kind, level| NewArchive {
            kind,
            level,
            ..NewArchive::default()
        };
        for kind in [Kind::SevenZip, Kind::TarXz, Kind::TarZst] {
            let highest = new(kind, Level::Highest);
            assert_eq!(within(&highest, u64::MAX).level, Level::Highest, "{kind:?}");
            let fitted = within(&highest, 300 * MB);
            assert_ne!(fitted.level, Level::Highest, "{kind:?}");
            assert!(
                memory_needed(&fitted) <= 300 * MB,
                "{kind:?} at {:?}",
                fitted.level
            );
            // Where none fits, the lowest: it is made all the same.
            assert_eq!(within(&highest, 0).level, Level::Lowest, "{kind:?}");
        }
        // What takes little is left as it is.
        assert_eq!(
            within(&new(Kind::Zip, Level::Highest), 100 * MB).level,
            Level::Highest
        );
        assert_eq!(
            within(&new(Kind::SevenZip, Level::Store), 0).level,
            Level::Store
        );
    }

    #[test]
    fn what_is_compressed_already_is_known_by_its_extension() {
        let known = COMPRESSED_EXTENSIONS;
        for name in [
            "a.jpg",
            "b/c.PNG",
            "film.mkv",
            "x.tar.gz",
            "y.7z",
            "secret.age",
            "Setup.pkg",
        ] {
            assert!(has_extension_in(name, known), "{name}");
        }
        for name in [
            "a.txt",
            "report.pdf",
            "sheet.xlsx",
            "jpg",
            "noext",
            "a.jpg.txt",
            "app.ts",
            "abcpng",
        ] {
            assert!(!has_extension_in(name, known), "{name}");
        }
        assert!(has_extension_in("a.RAW", &["raw".to_owned()]));
    }

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
