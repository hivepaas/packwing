//! Extracting an archive, and testing it: every file read through, its
//! checks made, nothing written.

use packwing_i18n::tr;
use std::fs::OpenOptions;
use std::io::{self, Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::time::Duration;

use hivepack::extract::{
    Cancel, Damaged, ErrorKind, LinkOutside, Links, NameDecoder, NameError, NameRules, OnDamaged,
    OnLinksOutside, OnRefused, OnUnnamable, Options, Permissions, Progress, ProgressFn, RarProblem,
    Select, Sink, Summary, extract_7z, extract_7z_to, extract_cpio, extract_cpio_to, extract_iso,
    extract_iso_to, extract_rar, extract_rar_to, extract_tar, extract_tar_to, extract_udf,
    extract_udf_to, extract_xar, extract_xar_to, extract_zip, extract_zip_to,
};
use hivepack::open::{Container, Opened};
use hivepack::tar::Reader;
use hivepack::udf::{Udf, VType};
use hivepack::zip::{Archive, Flags, OpenFlags, ReadAt, Volumes};

use crate::rar;
use crate::source::Source;
use crate::spool::spool;
use crate::volume::walk;
use crate::{Error, Keys, opener, seven_zip, without_layers};

/// What a job tells how far it is, and what stops it.
#[derive(Clone, Default)]
pub struct Control {
    pub progress: Option<ProgressFn>,
    pub cancel: Option<Cancel>,
    /// Asked, while extracting, about each file whose data is damaged: to
    /// stop, to leave it out, or to keep what was read of it, and go on.
    /// None stops. A name, a link or a limit that is refused always stops.
    pub on_damaged: Option<DamagedFn>,
    /// Asked once, before anything is written, about every file whose name this
    /// system (or every system, with [`ExtractOptions::portable_names`]) cannot
    /// take, all of them together: to stop, to leave them out, or to rename them.
    /// None stops at the first. A name that leads outside always stops.
    pub on_unnamable: Option<UnnamableFn>,
    /// Asked once, before anything is written, about every symbolic link whose
    /// target leads outside the folder extracted to (an absolute path, a climb
    /// out, or through another link), all of them together: to stop, to leave
    /// them out, or to create them as they are. None does as
    /// [`ExtractOptions::links_outside`] says. Never asked on Windows, where
    /// links are written as files holding their targets.
    pub on_links_outside: Option<LinksOutsideFn>,
}

/// What [`Control::on_unnamable`] is: given the files, it says what to do.
pub type UnnamableFn = Arc<dyn Fn(&[BadName]) -> OnUnnamable + Send + Sync>;

/// What [`Control::on_links_outside`] is: given the links, it says what to do.
pub type LinksOutsideFn = Arc<dyn Fn(&[OutsideLink]) -> OnLinksOutside + Send + Sync>;

/// A symbolic link whose target leads outside the folder extracted to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutsideLink {
    /// Its path in the archive, as the archive names it.
    pub path: String,
    /// Where it leads, as the archive has it.
    pub target: String,
}

impl OutsideLink {
    fn of(link: &LinkOutside) -> OutsideLink {
        OutsideLink {
            path: link
                .error
                .name
                .as_deref()
                .map_or_else(String::new, |raw| String::from_utf8_lossy(raw).into_owned()),
            target: link.target.clone(),
        }
    }
}

/// A file that is not extracted as it is named: a name this system cannot take,
/// or a link that leads outside.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BadName {
    /// Its path in the archive, as the archive names it.
    pub path: String,
    /// Why, in a few words.
    pub why: String,
}

impl BadName {
    fn of(error: &hivepack::extract::Error) -> BadName {
        BadName {
            path: error
                .name
                .as_deref()
                .map_or_else(String::new, |raw| String::from_utf8_lossy(raw).into_owned()),
            why: why_unnamable(&error.kind),
        }
    }
}

/// Why a name is not taken, as a person reads it.
fn why_unnamable(kind: &ErrorKind) -> String {
    match kind {
        ErrorKind::Name(NameError::Colon) => tr!("a colon, which Windows does not allow"),
        ErrorKind::Name(NameError::Backslash | NameError::Reserved) => {
            tr!("a character Windows does not allow (\\ < > \" | ? *)")
        }
        ErrorKind::Name(NameError::Device) => tr!("a name Windows keeps for a device"),
        ErrorKind::Name(NameError::TrailingSpaceOrDot) => {
            tr!("ends in a space or a dot, which Windows does not allow")
        }
        ErrorKind::Name(NameError::TooLong) => tr!("longer than 255 bytes"),
        ErrorKind::Name(NameError::Control) => tr!("a control character"),
        ErrorKind::Name(NameError::NotUtf8) => tr!("not a name this system can read"),
        ErrorKind::Duplicate { .. } => {
            tr!("the same file here as another, its name differing only in case")
        }
        ErrorKind::LinkTarget(_) | ErrorKind::LinkChain { .. } => {
            tr!("a link that leads outside the folder")
        }
        other => other.to_string(),
    }
}

/// A file extracted under another name than its own.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Renaming {
    /// Its path in the archive, as the archive names it.
    pub path: String,
    /// The path it was given.
    pub to: String,
    /// Why its own was not taken.
    pub why: String,
}

/// The files of `summary` left out, as their names could not be taken or as
/// they are links that lead outside, in the order they were met.
pub fn left_out(summary: &Summary) -> Vec<BadName> {
    summary.skipped.iter().map(BadName::of).collect()
}

/// The files of `summary` extracted under other names than theirs.
pub fn renamings(summary: &Summary) -> Vec<Renaming> {
    summary
        .renamed
        .iter()
        .map(|r| {
            let BadName { path, why } = BadName::of(&r.error);
            Renaming {
                path,
                to: r.to.join("/"),
                why,
            }
        })
        .collect()
}

/// What [`Control::on_damaged`] is: given the file, it says what to do.
pub type DamagedFn = Arc<dyn Fn(&Damage) -> OnDamaged + Send + Sync>;

/// A file whose data is damaged: a check that fails, data that does not
/// decompress, or that ends early.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Damage {
    /// Its path in the archive, as the archive names it.
    pub path: String,
    /// What is wrong with it.
    pub why: String,
    /// Once gone on from, the bytes of it kept, or None if it was left out.
    pub kept: Option<u64>,
}

impl Damage {
    fn of(error: &hivepack::extract::Error, kept: Option<u64>) -> Damage {
        Damage {
            path: error
                .name
                .as_deref()
                .map_or_else(String::new, |raw| String::from_utf8_lossy(raw).into_owned()),
            why: error.kind.to_string(),
            kept,
        }
    }
}

/// The files of `summary` whose data was damaged, gone on from, in the order
/// they were met.
pub fn damages(summary: &Summary) -> Vec<Damage> {
    summary
        .damaged
        .iter()
        .map(|d: &Damaged| Damage::of(&d.error, d.kept))
        .collect()
}

impl Control {
    pub(crate) fn check(&self) -> Result<(), Error> {
        match &self.cancel {
            Some(c) if c.is_cancelled() => Err(Error::Cancelled),
            _ => Ok(()),
        }
    }

    pub(crate) fn report(&self, p: &Progress) {
        if let Some(progress) = &self.progress {
            progress(p);
        }
    }
}

fn zip_archive<R: ReadAt>(source: R) -> Result<Archive<R>, Error> {
    Ok(Archive::open(
        source,
        OpenFlags::CHECKCONS | OpenFlags::RDONLY,
    )?)
}

/// The most entries an archive may hold: hivepack's default, 100,000, is less
/// than a folder of a program's sources may be.
const MAX_ENTRIES: u64 = 10_000_000;

/// What a user can choose about extracting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExtractOptions {
    /// Replaces files that are there; otherwise one of them stops the job.
    pub overwrite: bool,
    /// The most it may write, in bytes in all and so in a file: what stops an
    /// archive bomb. None for no limit, but the ratio of written to compressed
    /// stays.
    pub limit: Option<u64>,
    /// The files keep the archive's mark of where it came from.
    pub keep_mark: bool,
    /// Names are taken only if every system takes them: Windows' characters
    /// and device names, and two names that differ only in case, are refused on
    /// Linux too. Otherwise, as this system and the destination take them.
    pub portable_names: bool,
    /// What becomes of the symbolic links whose targets lead outside the folder,
    /// when nothing asks ([`Control::on_links_outside`]): left out and said (the
    /// default), created as they are, or the job stopped before anything is
    /// written. Links are written as files on Windows, whatever this says.
    pub links_outside: OnLinksOutside,
}

impl Default for ExtractOptions {
    fn default() -> ExtractOptions {
        ExtractOptions {
            overwrite: false,
            limit: Some(100_000_000_000),
            keep_mark: true,
            portable_names: false,
            links_outside: OnLinksOutside::Skip,
        }
    }
}

/// [`extract_with`] the default options.
pub fn extract(
    source: impl Into<Source>,
    destination: &Path,
    keys: &Keys,
    control: &Control,
) -> Result<Summary, Error> {
    extract_with(
        source,
        destination,
        keys,
        &ExtractOptions::default(),
        control,
    )
}

/// Extracts the archive at `path` into `destination`, made if it is not
/// there, with hivepack::extract's checks: nothing outside it, over what is
/// there, or through a link. A link that stays inside it is made, last; one
/// that leaves it stops the job. On Windows, where most accounts cannot make
/// links, a link is a file that holds its target. The files keep the archive's
/// mark of where it came from.
pub fn extract_with(
    source: impl Into<Source>,
    destination: &Path,
    keys: &Keys,
    how: &ExtractOptions,
    control: &Control,
) -> Result<Summary, Error> {
    extract_picked(&source.into(), destination, keys, how, None, control)
}

/// Extracts only `picked` of the archive at `path`, files or folders given by
/// their paths in it, each with what is under it, straight into `destination`:
/// `docs/a/b.txt` comes out as `b.txt`. They are written in a folder of their
/// own inside it first, and moved out once all of them are, so a job that fails
/// or is stopped leaves nothing. One already there stops it first, unless it is
/// to be replaced; a folder takes what is new in it.
pub fn extract_selected(
    source: impl Into<Source>,
    destination: &Path,
    picked: &[Vec<String>],
    keys: &Keys,
    how: &ExtractOptions,
    control: &Control,
) -> Result<Summary, Error> {
    if !how.overwrite {
        for name in picked.iter().filter_map(|p| p.last()) {
            if destination.join(name).symlink_metadata().is_ok() {
                return Err(Error::Failed(tr!(
                    "{name} is already in the folder.",
                    name = name
                )));
            }
        }
    }
    std::fs::create_dir_all(destination)?;
    let scratch = fresh_folder(destination, ".packwing-extracting")?;
    let chosen: Vec<Vec<String>> = picked.to_vec();
    let select: Select =
        Arc::new(move |entry: &[String]| chosen.iter().any(|p| entry.starts_with(p)));
    let result = extract_picked(&source.into(), &scratch, keys, how, Some(select), control)
        .and_then(|done| {
            for entry in picked {
                let Some(name) = entry.last() else { continue };
                let from = entry.iter().fold(scratch.clone(), |at, c| at.join(c));
                move_into(&from, &destination.join(name))?;
            }
            Ok(done)
        });
    let _ = std::fs::remove_dir_all(&scratch);
    result
}

/// A new folder in `parent`, readable by its owner only, whose name starts
/// with `stem`.
fn fresh_folder(parent: &Path, stem: &str) -> Result<PathBuf, Error> {
    for n in 0..10_000 {
        let path = parent.join(format!("{stem}-{n}"));
        #[cfg_attr(not(unix), allow(unused_mut))]
        let mut builder = std::fs::DirBuilder::new();
        #[cfg(unix)]
        std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
        match builder.create(&path) {
            Ok(()) => return Ok(path),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e.into()),
        }
    }
    Err(Error::Failed(tr!(
        "Could not make a folder in {parent}.",
        parent = parent.display()
    )))
}

/// `from` as `to`: over a file, and into a folder, what is in it.
fn move_into(from: &Path, to: &Path) -> Result<(), Error> {
    let from_kind = from.symlink_metadata()?.file_type();
    match to.symlink_metadata() {
        Err(_) => {}
        Ok(there) if from_kind.is_dir() && there.is_dir() => {
            for entry in std::fs::read_dir(from)? {
                let entry = entry?;
                move_into(&entry.path(), &to.join(entry.file_name()))?;
            }
            return Ok(());
        }
        Ok(there) if there.is_dir() => {
            return Err(Error::Failed(tr!(
                "{to} is a folder, and a file would replace it.",
                to = to.display()
            )));
        }
        Ok(_) => std::fs::remove_file(to)?,
    }
    std::fs::rename(from, to)?;
    Ok(())
}

/// The folder files opened from archives go in, under the system's temporary one.
fn opened_folder() -> PathBuf {
    std::env::temp_dir().join("packwing-opened")
}

/// How long a copy opened from an archive is kept: what opened it may still
/// have it for a while after packwing has gone.
const OPENED_FOR: Duration = Duration::from_secs(24 * 60 * 60);

/// Removes the copies opened from archives that are over a day old.
pub fn clean_opened_files() {
    let Ok(folders) = std::fs::read_dir(opened_folder()) else {
        return;
    };
    for folder in folders.flatten() {
        let old = folder
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.elapsed().ok())
            .is_some_and(|age| age > OPENED_FOR);
        if old {
            let _ = std::fs::remove_dir_all(folder.path());
        }
    }
}

/// A copy of the entry at `entry` of the archive at `path`, to open: in a new
/// folder of the temporary one, readable by its owner only, with the archive's
/// mark of where it came from, whatever Settings says.
pub fn extract_to_open(
    source: impl Into<Source>,
    entry: &[String],
    keys: &Keys,
    control: &Control,
) -> Result<PathBuf, Error> {
    let name = entry.last().ok_or(Error::NotAnArchive)?.clone();
    let folder = opened_subfolder()?;
    let how = ExtractOptions::default();
    match extract_selected(source, &folder, &[entry.to_vec()], keys, &how, control) {
        Ok(_) => Ok(folder.join(name)),
        Err(e) => {
            let _ = std::fs::remove_dir_all(&folder);
            Err(e)
        }
    }
}

/// The most pixels a picture may have to be shown: decoded, one of these takes
/// 400 MB, and a small file can say it has far more.
const MOST_PIXELS: u64 = 100_000_000;

/// Whether the picture in `bytes` says a size that is safe to decode.
pub fn image_fits(bytes: &[u8]) -> bool {
    imagesize::blob_size(bytes)
        .is_ok_and(|size| (size.width as u64).saturating_mul(size.height as u64) <= MOST_PIXELS)
}

/// What the entry at `entry` of the archive at `path` holds, if that is `most`
/// bytes or less. It passes through a folder of the temporary one that its
/// owner only can read, and is gone from there when this returns.
pub fn read_entry(
    source: impl Into<Source>,
    entry: &[String],
    keys: &Keys,
    most: u64,
    control: &Control,
) -> Result<Vec<u8>, Error> {
    let how = ExtractOptions {
        limit: Some(most),
        keep_mark: false,
        ..ExtractOptions::default()
    };
    let name = entry.last().ok_or(Error::NotAnArchive)?;
    let folder = opened_subfolder()?;
    let read = extract_selected(source, &folder, &[entry.to_vec()], keys, &how, control)
        .and_then(|_| Ok(std::fs::read(folder.join(name))?));
    let _ = std::fs::remove_dir_all(&folder);
    read
}

/// A new folder for a copy of an entry, or a new archive on its way to a storage,
/// under the temporary one. If packwing goes before it does,
/// [`clean_opened_files`] takes it.
pub(crate) fn opened_subfolder() -> Result<PathBuf, Error> {
    let root = opened_folder();
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
    builder.create(&root)?;
    fresh_folder(&root, "open")
}

fn extract_picked(
    source: &Source,
    destination: &Path,
    keys: &Keys,
    how: &ExtractOptions,
    select: Option<Select>,
    control: &Control,
) -> Result<Summary, Error> {
    extract_into(
        source,
        WriteTo::Folder(destination),
        keys,
        how,
        select,
        control,
    )
}

/// Where an extraction writes: a folder, or what takes each entry in its place.
pub(crate) enum WriteTo<'a> {
    Folder(&'a Path),
    Taker(&'a mut dyn Taker),
}

/// What takes an archive's entries in place of a folder: as a [`Sink`] does, and
/// the one file that is under layers alone, as `a.sql.gz` is.
pub(crate) trait Taker: Sink {
    /// The one file named `name`, its data read from `data`: the bytes read.
    fn one_file(
        &mut self,
        name: &str,
        data: &mut dyn Read,
        control: &Control,
    ) -> Result<u64, Error>;
}

/// The archive of `source` extracted `into` a folder or a [`Taker`].
pub(crate) fn extract_into(
    source: &Source,
    into: WriteTo<'_>,
    keys: &Keys,
    how: &ExtractOptions,
    select: Option<Select>,
    control: &Control,
) -> Result<Summary, Error> {
    let mut options = Options::default();
    options.password = keys.password.clone();
    options.progress = control.progress.clone();
    options.cancel = control.cancel.clone();
    options.origin = crate::mark::for_extracting(source, how.keep_mark)?;
    options.overwrite = how.overwrite;
    options.limits.max_entries = MAX_ENTRIES;
    let limit = how.limit.unwrap_or(u64::MAX);
    options.limits.max_entry_size = limit;
    options.limits.max_total_size = limit;
    options.select = select;
    // ZIP's files written at once, and 7z's LZMA2 blocks decoded so.
    options.threads = crate::threads();
    // A name in an old encoding is no reason to stop: the file comes out, its
    // name perhaps wrong. What the name then says is checked all the same.
    options.decode_names = Some(NameDecoder::lossy());
    options.name_rules = if how.portable_names {
        NameRules::Portable
    } else {
        NameRules::Native
    };
    // A link that leads outside is asked about, or as the options say: left
    // out and said, by default, rather than stop the rest.
    options.links = if cfg!(windows) {
        Links::AsFile
    } else {
        Links::Inside
    };
    match how.links_outside {
        OnLinksOutside::Stop => options.links_outside = OnRefused::Stop,
        OnLinksOutside::Create if !cfg!(windows) => options.links = Links::Any,
        _ => options.links_outside = OnRefused::Skip,
    }
    if let (Some(asked), Links::Inside) = (control.on_links_outside.clone(), options.links) {
        options.on_links_outside = Some(Arc::new(move |links: &[LinkOutside]| {
            asked(&links.iter().map(OutsideLink::of).collect::<Vec<_>>())
        }));
    }
    if let WriteTo::Taker(_) = into {
        // What takes the links is no folder but an archive: they stay links, on
        // every system, wherever they lead, as in the archive converted; and
        // the permission bits are as the archive has them.
        options.links = Links::Any;
        options.on_links_outside = None;
        options.permissions = Permissions::Preserve;
    }
    // Each entry to one or the other, as `into` says.
    macro_rules! into {
        ($folder:ident, $taker:ident, $from:expr) => {
            match into {
                WriteTo::Folder(destination) => $folder($from, destination, &options),
                WriteTo::Taker(taker) => $taker($from, taker, &options),
            }
        };
    }
    let mut opener = opener(source, keys);
    let mut opened = opener.open()?;
    let layers = opened.layers().to_vec();
    // RAR 3 and earlier have nothing but data that fails its hash to tell a
    // wrong password by: with one given, that stops the job, as a wrong
    // password, rather than be kept.
    let guessing = opened.container() == Container::Rar && keys.password.is_some();
    if let Some(asked) = control.on_damaged.clone().filter(|_| !guessing) {
        options.on_damaged = Some(Arc::new(move |e: &hivepack::extract::Error| {
            asked(&Damage::of(e, None))
        }));
    }
    if let Some(asked) = control.on_unnamable.clone() {
        options.on_unnamable = Some(Arc::new(move |errors: &[hivepack::extract::Error]| {
            asked(&errors.iter().map(BadName::of).collect::<Vec<_>>())
        }));
    }
    match opened.container() {
        Container::Tar => {
            drop(opened);
            Ok(into!(extract_tar, extract_tar_to, || Ok(opener.open()?))?)
        }
        Container::Zip if layers.is_empty() => {
            drop(opened);
            match source {
                Source::File(path) => {
                    let archive = zip_archive(Volumes::open(path)?)?;
                    Ok(into!(extract_zip, extract_zip_to, &archive)?)
                }
                Source::Stored(stored) => {
                    let archive = zip_archive(stored.stored_reader())?;
                    Ok(into!(extract_zip, extract_zip_to, &archive)?)
                }
            }
        }
        Container::Zip => {
            let spooled = spool(&mut opened)?;
            let archive = zip_archive(&spooled.file)?;
            Ok(into!(extract_zip, extract_zip_to, &archive)?)
        }
        // Read twice, as a tar archive is: once to check, once to write.
        Container::Iso => {
            drop(opened);
            Ok(into!(extract_iso, extract_iso_to, || Ok(opener.open()?))?)
        }
        Container::Udf if layers.is_empty() => {
            drop(opened);
            let mut udf = Udf::open(source.reader()?)?;
            Ok(into!(extract_udf, extract_udf_to, &mut udf)?)
        }
        Container::Udf => {
            let spooled = spool(&mut opened)?;
            let mut udf = Udf::open(&spooled.file)?;
            Ok(into!(extract_udf, extract_udf_to, &mut udf)?)
        }
        Container::SevenZip if layers.is_empty() => {
            drop(opened);
            let mut seven = seven_zip(|| source.reader(), keys)?;
            Ok(into!(extract_7z, extract_7z_to, &mut seven.archive)?)
        }
        Container::SevenZip => {
            let spooled = spool(&mut opened)?;
            let mut seven = seven_zip(|| spooled.rewound(), keys)?;
            Ok(into!(extract_7z, extract_7z_to, &mut seven.archive)?)
        }
        // Read twice, as a tar archive is; an RPM package's payload, its
        // lead and headers taken off, and a macOS package's payloads,
        // expanded into folders of their names.
        Container::Cpio => {
            drop(opened);
            Ok(into!(extract_cpio, extract_cpio_to, || Ok(opener.open()?))?)
        }
        Container::Xar => {
            drop(opened);
            Ok(into!(extract_xar, extract_xar_to, || Ok(opener.open()?))?)
        }
        Container::Rar if layers.is_empty() && source.path().is_some() => {
            drop(opened);
            let path = source.path().expect("just looked at");
            let packed = Arc::new(AtomicU64::new(0));
            let result = into!(extract_rar, extract_rar_to, |password| rar::open(
                path,
                password,
                packed.clone()
            ));
            rar_result(result, keys)
        }
        Container::Rar => {
            let spooled = spool(&mut opened)?;
            let result = into!(extract_rar, extract_rar_to, |password| {
                hivepack::rar::Archive::open_with_password(spooled.rewound()?, false, password)
            });
            rar_result(result, keys)
        }
        Container::Data if layers.is_empty() => Err(Error::NotAnArchive),
        _ => {
            let name = without_layers(&source.name(), &layers);
            match into {
                WriteTo::Folder(destination) => {
                    extract_one(opened, &name, destination, how, control)
                }
                WriteTo::Taker(taker) => {
                    let mut summary = Summary::default();
                    summary.bytes = taker.one_file(&name, &mut opened, control)?;
                    summary.files = 1;
                    Ok(summary)
                }
            }
        }
    }
}

/// What extracting a RAR archive came to: data that fails its hash with a
/// password given is the wrong password, which RAR 3 and earlier have nothing
/// else to tell by.
fn rar_result(
    result: Result<Summary, hivepack::extract::Error>,
    keys: &Keys,
) -> Result<Summary, Error> {
    result.map_err(|e| match e.kind {
        hivepack::extract::ErrorKind::RarEntry(
            RarProblem::BrokenData | RarProblem::BrokenArchive,
        ) if keys.password.is_some() => Error::WrongPassword,
        _ => Error::from(e),
    })
}

/// `bytes` more were read.
pub(crate) fn copy_progress(control: &Control, done: &mut Progress, bytes: usize) {
    done.bytes += bytes as u64;
    control.report(done);
}

/// One file under layers, written as `name` in `destination`: over what is
/// there only if asked, and removed if it fails or is cancelled.
fn extract_one(
    mut opened: Opened<'_>,
    name: &str,
    destination: &Path,
    how: &ExtractOptions,
    control: &Control,
) -> Result<Summary, Error> {
    std::fs::create_dir_all(destination)?;
    let path = destination.join(name);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(!how.overwrite)
        .create(how.overwrite)
        .truncate(how.overwrite)
        .open(&path)
        .map_err(|e| match e.kind() {
            io::ErrorKind::AlreadyExists => {
                Error::Failed(tr!("{name} is already in the folder", name = name))
            }
            _ => Error::from(e),
        })?;
    let mut done = Progress::default();
    done.total_entries = 1;
    let result = copy(&mut opened, &mut file, control, &mut done);
    let written = match result.and_then(|n| file.sync_all().map(|_| n).map_err(Error::from)) {
        Ok(n) => n,
        Err(e) => {
            drop(file);
            let _ = std::fs::remove_file(&path);
            return Err(e);
        }
    };
    done.entries = 1;
    control.report(&done);
    let mut summary = Summary::default();
    summary.files = 1;
    summary.bytes = written;
    Ok(summary)
}

/// `from` into `to`, a piece at a time, `done` told each.
fn copy(
    from: &mut impl Read,
    to: &mut impl Write,
    control: &Control,
    done: &mut Progress,
) -> Result<u64, Error> {
    let mut buf = vec![0u8; 64 << 10];
    let mut n = 0u64;
    loop {
        control.check()?;
        match from.read(&mut buf) {
            Ok(0) => return Ok(n),
            Ok(k) => {
                to.write_all(&buf[..k])?;
                n += k as u64;
                done.bytes += k as u64;
                control.report(done);
            }
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e.into()),
        }
    }
}

/// Reads every file of the archive at `path` through, with every check its
/// format has (CRC-32s, the compressors' checks, age's authentication),
/// and writes nothing. The summary counts the files and bytes read.
pub fn test(source: impl Into<Source>, keys: &Keys, control: &Control) -> Result<Summary, Error> {
    let source: Source = source.into();
    let source = &source;
    let mut opener = opener(source, keys);
    let mut opened = opener.open()?;
    let layers = opened.layers().to_vec();
    let mut done = Progress::default();
    let mut summary = Summary::default();
    match opened.container() {
        Container::Tar => {
            let mut tr = Reader::new(&mut opened);
            while let Some(h) = tr.next()? {
                copy(&mut tr, &mut io::sink(), control, &mut done)?;
                if h.typeflag == hivepack::tar::TYPE_REG || h.typeflag == hivepack::tar::TYPE_REG_A
                {
                    summary.files += 1;
                }
                done.entries += 1;
                control.report(&done);
            }
            summary.bytes = done.bytes;
            // The stream's own checks come at its end, past the archive's:
            // read, not counted.
            let quiet = Control {
                progress: None,
                cancel: control.cancel.clone(),
                on_damaged: None,
                on_unnamable: None,
                on_links_outside: None,
            };
            copy(&mut tr.into_inner(), &mut io::sink(), &quiet, &mut done)?;
        }
        Container::Zip if layers.is_empty() => {
            drop(opened);
            match source {
                Source::File(path) => test_zip(
                    zip_archive(Volumes::open(path)?)?,
                    keys,
                    control,
                    &mut summary,
                )?,
                Source::Stored(stored) => test_zip(
                    zip_archive(stored.stored_reader())?,
                    keys,
                    control,
                    &mut summary,
                )?,
            }
        }
        Container::Zip => {
            let spooled = spool(&mut opened)?;
            test_zip(zip_archive(&spooled.file)?, keys, control, &mut summary)?;
        }
        Container::Iso => {
            test_iso(
                hivepack::iso::Reader::new(&mut opened),
                control,
                &mut summary,
            )?;
            read_to_end(&mut opened, control, &mut done)?;
        }
        Container::Udf if layers.is_empty() => {
            drop(opened);
            test_udf(Udf::open(source.reader()?)?, control, &mut summary)?;
        }
        Container::Udf => {
            let spooled = spool(&mut opened)?;
            test_udf(Udf::open(&spooled.file)?, control, &mut summary)?;
        }
        Container::SevenZip if layers.is_empty() => {
            drop(opened);
            let seven = seven_zip(|| source.reader(), keys)?;
            test_7z(seven.archive, control, &mut summary)?;
        }
        Container::SevenZip => {
            let spooled = spool(&mut opened)?;
            let seven = seven_zip(|| spooled.rewound(), keys)?;
            test_7z(seven.archive, control, &mut summary)?;
        }
        Container::Cpio => {
            test_cpio(
                hivepack::cpio::Reader::new(&mut opened),
                control,
                &mut summary,
            )?;
            read_to_end(&mut opened, control, &mut done)?;
        }
        Container::Xar => {
            test_xar(
                hivepack::xar::Reader::new(&mut opened),
                control,
                &mut summary,
            )?;
            read_to_end(&mut opened, control, &mut done)?;
        }
        Container::Rar if layers.is_empty() && source.path().is_some() => {
            drop(opened);
            let path = source.path().expect("just looked at");
            let packed = Arc::new(AtomicU64::new(0));
            let arc = rar::open(path, &rar::password(keys), packed)?;
            rar::test(arc, keys, control, &mut summary)?;
        }
        Container::Rar => {
            let spooled = spool(&mut opened)?;
            let arc = hivepack::rar::Archive::open_with_password(
                spooled.rewound()?,
                false,
                &rar::password(keys),
            )?;
            rar::test(arc, keys, control, &mut summary)?;
        }
        Container::Data if layers.is_empty() => return Err(Error::NotAnArchive),
        _ => {
            summary.bytes = copy(&mut opened, &mut io::sink(), control, &mut done)?;
            summary.files = 1;
        }
    }
    Ok(summary)
}

fn test_zip<R: ReadAt>(
    mut archive: Archive<R>,
    keys: &Keys,
    control: &Control,
    summary: &mut Summary,
) -> Result<(), Error> {
    archive.set_default_password(keys.password.as_deref());
    let mut done = Progress::default();
    let entries = archive.get_num_entries(Flags::NONE);
    done.total_entries = entries;
    for i in 0..entries {
        let stat = archive.stat_index(i, Flags::NONE)?;
        if stat.name.is_some_and(|n| n.ends_with(b"/")) {
            continue;
        }
        let mut file = archive.fopen_index(i, Flags::NONE)?;
        summary.bytes += copy(&mut file, &mut io::sink(), control, &mut done)?;
        summary.files += 1;
        done.entries += 1;
        control.report(&done);
    }
    Ok(())
}

/// What is past an archive's end, read to the end of the stream for the
/// layers' own checks there (a compressor's, age's authentication): read,
/// not counted.
fn read_to_end(r: &mut impl Read, control: &Control, done: &mut Progress) -> Result<(), Error> {
    let quiet = Control {
        progress: None,
        cancel: control.cancel.clone(),
        on_damaged: None,
        on_unnamable: None,
        on_links_outside: None,
    };
    copy(r, &mut io::sink(), &quiet, done).map(|_| ())
}

/// Reads every file of a 7z archive through, each folder decoded once,
/// with every file's CRC and every folder's checks: the password of the
/// encrypted ones was given when it was opened.
fn test_7z<R: Read + Seek>(
    mut archive: hivepack::sevenz::Archive<R>,
    control: &Control,
    summary: &mut Summary,
) -> Result<(), Error> {
    let mut done = Progress::default();
    done.total_entries = archive.len() as u64;
    let mut files = archive.files();
    while let Some(entry) = files.next()? {
        control.check()?;
        if !entry.is_dir {
            summary.bytes += copy(&mut files, &mut io::sink(), control, &mut done)?;
            summary.files += 1;
        }
        done.entries += 1;
        control.report(&done);
    }
    Ok(())
}

/// Reads every file of a cpio archive through: an RPM package's payload,
/// decompressed and checked by its layers. An entry found after bytes that
/// are not a header fails it, as it fails an extraction.
fn test_cpio<R: Read>(
    mut cpio: hivepack::cpio::Reader<R>,
    control: &Control,
    summary: &mut Summary,
) -> Result<(), Error> {
    let mut done = Progress::default();
    while let Some(h) = cpio.next()? {
        control.check()?;
        if let Some(warning) = h.warning {
            let name = String::from_utf8_lossy(&h.path);
            return Err(Error::Failed(format!("{name}: {warning}")));
        }
        let read = copy(&mut cpio, &mut io::sink(), control, &mut done)?;
        if h.file_type() == hivepack::cpio::AE_IFREG && read > 0 {
            summary.bytes += read;
            summary.files += 1;
        }
        done.entries += 1;
        control.report(&done);
    }
    Ok(())
}

/// Reads every file of a XAR archive through, each checked against its
/// checksums, as its reader does at the end of its data. A macOS package's
/// payloads are read as the files they are: their own layers check them
/// when they extract.
fn test_xar<R: Read>(
    mut xar: hivepack::xar::Reader<R>,
    control: &Control,
    summary: &mut Summary,
) -> Result<(), Error> {
    let mut done = Progress::default();
    while let Some(h) = xar.next()? {
        control.check()?;
        if let Some(warning) = h.warning {
            let name = h.path.as_deref().map(String::from_utf8_lossy);
            return Err(Error::Failed(match name {
                Some(name) => format!("{name}: {warning}"),
                None => warning.to_string(),
            }));
        }
        let read = copy(&mut xar, &mut io::sink(), control, &mut done)?;
        if h.mode & hivepack::xar::AE_IFMT == hivepack::xar::AE_IFREG && h.hardlink.is_none() {
            summary.bytes += read;
            summary.files += 1;
        }
        done.entries += 1;
        control.report(&done);
    }
    Ok(())
}

/// Reads every file of an ISO 9660 image through: zisofs's data
/// decompressed, and every entry's extents found on the image. An entry
/// with something missing (a directory left out, a file past the image's
/// end) fails it, as it fails an extraction.
fn test_iso<R: Read>(
    mut iso: hivepack::iso::Reader<R>,
    control: &Control,
    summary: &mut Summary,
) -> Result<(), Error> {
    let mut done = Progress::default();
    while let Some(h) = iso.next()? {
        control.check()?;
        if let Some(warning) = h.warning {
            let name = h.path.as_deref().map(String::from_utf8_lossy);
            return Err(Error::Failed(match name {
                Some(name) => format!("{name}: {warning}"),
                None => warning.to_string(),
            }));
        }
        let read = copy(&mut iso, &mut io::sink(), control, &mut done)?;
        if h.file_type() == hivepack::iso::AE_IFREG && h.hardlink.is_none() {
            summary.bytes += read;
            summary.files += 1;
        }
        done.entries += 1;
        control.report(&done);
    }
    Ok(())
}

/// Reads every file of a UDF volume through, each of its blocks where its
/// allocation descriptors say; what is not recorded reads as zeros.
fn test_udf<R: Read + Seek>(
    mut udf: Udf<R>,
    control: &Control,
    summary: &mut Summary,
) -> Result<(), Error> {
    let mut done = Progress::default();
    walk(&mut udf, |udf, _, node, stat| {
        control.check()?;
        if stat.kind == VType::VREG {
            summary.bytes += copy(&mut udf.file(node), &mut io::sink(), control, &mut done)?;
            summary.files += 1;
        }
        done.entries += 1;
        control.report(&done);
        Ok(())
    })
}
