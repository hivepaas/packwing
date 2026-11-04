//! Making an archive of what another holds, in another format, an entry at a
//! time: each entry of the one read goes into the new one as it comes, with no
//! folder in between, through the same checks as extracting it (its name, links,
//! the limits that stop a bomb).
//!
//! A file's data is held until it has all been read and checked, as it cannot be
//! taken back out of the new archive: in memory if it is small, in a temporary
//! file readable by its owner only if not. A file whose data is damaged can then
//! be left out ([`Control::on_damaged`]), as extracting it can.
//!
//! What the new archive is made as is what [`create`](crate::create) makes: its
//! format, level, password, age keys and parts, under the same policies. A hard
//! link in a ZIP or 7z archive is a symbolic link to its file, as neither holds
//! one. The new archive keeps the mark of the one read, if it came from the
//! Internet.

use packwing_i18n::tr;
use std::io::{self, Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use hivepack::age::BoxedRecipient;
use hivepack::extract::{Entry, Select, Sink, Summary};
use hivepack::zip::{cm, em};

use crate::create::{
    Kind, Level, NewArchive, PIECE, Squeezed, WriteSeek, age_recipients, age_writer,
    has_extension_in, obey_policy, seven_zip_options, with_age, write_new,
};
use crate::job::{ExtractOptions, Taker, WriteTo, extract_into};
use crate::source::Source;
use crate::spool::{Spooled, scratch};
use crate::{Control, Error, Keys};

/// The most of a file held in memory: one bigger waits in a temporary file.
const HELD_IN_MEMORY: usize = 32 << 20;

/// The type bits of a Unix mode.
const S_IFMT: u32 = 0o170_000;
const S_IFREG: u32 = 0o100_000;
const S_IFDIR: u32 = 0o040_000;
const S_IFLNK: u32 = 0o120_000;

/// What [`convert`] made.
#[derive(Debug)]
pub struct Converted {
    /// Where the new archive is: it, or its first part.
    pub made: PathBuf,
    /// What was read: the files, the bytes, and those whose data was damaged.
    pub summary: Summary,
}

/// Makes an archive of what the archive of `source` holds, as `new` says, named after
/// `wanted` or the first name after it that is free; see the module's documentation.
/// `keys` open the one read, and `how` limits what it may come to.
pub fn convert(
    source: impl Into<Source>,
    keys: &Keys,
    wanted: &Path,
    new: &NewArchive,
    how: &ExtractOptions,
    control: &Control,
) -> Result<Converted, Error> {
    // Refused before anything is read.
    obey_policy(new)?;
    let recipients = age_recipients(new)?;
    let source: Source = source.into();
    let mut summary = None;
    let (made, files) = write_new(wanted, new, |out| {
        let read =
            |new: &NewArchive, to: &mut dyn WriteSeek, recipients: Option<Vec<BoxedRecipient>>| {
                let format = Format::open(new, to, recipients)?;
                let mut taker = Converter {
                    format,
                    new,
                    held: None,
                    failed: None,
                    taken: 0,
                };
                let select = new
                    .skip_mac_files
                    .then(|| -> Select { Arc::new(not_mac_file) });
                let read = extract_into(
                    &source,
                    WriteTo::Taker(&mut taker),
                    keys,
                    how,
                    select,
                    control,
                );
                let read = match (read, taker.failed.take()) {
                    // What the new archive said is what went wrong, not how it was told.
                    (Err(_), Some(failed)) => return Err(failed),
                    (read, _) => read?,
                };
                if taker.taken == 0 {
                    return Err(Error::Failed(tr!("Nothing to compress.")));
                }
                taker.format.finish()?;
                Ok(read)
            };
        // A ZIP or 7z archive encrypted whole goes back over what it wrote, which
        // age's stream cannot: it is made in a temporary file first.
        let sealed = matches!(new.kind, Kind::Zip | Kind::SevenZip) && new.whole_with_age;
        match recipients {
            Some(recipients) if sealed => {
                // Its password is age's passphrase, not its own.
                let plain = NewArchive {
                    password: None,
                    age_keys: Vec::new(),
                    whole_with_age: false,
                    hide_names: false,
                    ..new.clone()
                };
                let mut unsealed = scratch()?;
                summary = Some(read(&plain, &mut unsealed.file, None)?);
                let mut sealed = age_writer(out, &recipients)?;
                io::copy(&mut unsealed.rewound()?, &mut sealed)?;
                sealed.finish()?;
            }
            recipients => summary = Some(read(new, out, recipients)?),
        }
        Ok(())
    })?;
    if let Err(e) = crate::mark::on_new_archive(&source, &files, how.keep_mark) {
        for file in &files {
            let _ = std::fs::remove_file(file);
        }
        return Err(e);
    }
    Ok(Converted {
        made,
        summary: summary.unwrap_or_default(),
    })
}

/// Whether what is at `path` is not what only a Mac uses, as
/// [`NewArchive::skip_mac_files`] leaves out.
fn not_mac_file(path: &[String]) -> bool {
    !path
        .iter()
        .any(|part| part == ".DS_Store" || part == "__MACOSX" || part.starts_with("._"))
}

/// What a tar archive, or one file, is written through: encrypted with age, or not.
enum Sealed<W: Write> {
    Plain(W),
    Age(hivepack::age::Encryptor<W>),
}

impl<W: Write> Sealed<W> {
    fn finish(self) -> Result<(), Error> {
        match self {
            Sealed::Plain(mut out) => out.flush()?,
            Sealed::Age(encrypted) => {
                encrypted.finish()?;
            }
        }
        Ok(())
    }
}

impl<W: Write> Write for Sealed<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Sealed::Plain(out) => out.write(buf),
            Sealed::Age(out) => out.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Sealed::Plain(out) => out.flush(),
            Sealed::Age(out) => out.flush(),
        }
    }
}

/// The new archive, being written.
enum Format<'a> {
    Zip(Box<hivepack::zip::Writer<&'a mut dyn WriteSeek>>),
    SevenZip {
        archive: Box<hivepack::sevenz::Writer<&'a mut dyn WriteSeek>>,
        /// Its files are stored now, not compressed.
        storing: bool,
    },
    Tar(hivepack::tar::Writer<Squeezed<Sealed<&'a mut dyn WriteSeek>>>),
    /// One file compressed by itself: whether it has been written.
    One(Squeezed<Sealed<&'a mut dyn WriteSeek>>, bool),
}

impl<'a> Format<'a> {
    fn open(
        new: &NewArchive,
        out: &'a mut dyn WriteSeek,
        recipients: Option<Vec<BoxedRecipient>>,
    ) -> Result<Format<'a>, Error> {
        let sealed = |out: &'a mut dyn WriteSeek| -> Result<_, Error> {
            Ok(match recipients.as_deref().filter(|_| with_age(new)) {
                Some(recipients) => Sealed::Age(age_writer(out, recipients)?),
                None => Sealed::Plain(out),
            })
        };
        Ok(match new.kind {
            Kind::Zip => {
                let mut zip = hivepack::zip::Writer::new(out);
                zip.set_random_source(Some(Arc::new(|buffer: &mut [u8]| {
                    getrandom::fill(buffer).map_err(io::Error::other)
                })));
                if let Some(password) = &new.password {
                    zip.set_encryption(em::AES_256, password);
                }
                Format::Zip(Box::new(zip))
            }
            Kind::SevenZip => Format::SevenZip {
                archive: Box::new(hivepack::sevenz::Writer::with_options(
                    out,
                    seven_zip_options(new)?,
                )?),
                storing: new.level == Level::Store,
            },
            kind if kind.one_file() => Format::One(Squeezed::new(new, sealed(out)?)?, false),
            _ => Format::Tar(hivepack::tar::Writer::new(Squeezed::new(
                new,
                sealed(out)?,
            )?)),
        })
    }

    fn directory(
        &mut self,
        name: &str,
        mtime: Option<i64>,
        mode: Option<u32>,
    ) -> Result<(), Error> {
        match self {
            Format::Zip(zip) => zip.add_directory(&zip_entry(name, mtime, mode, None))?,
            Format::SevenZip { archive, .. } => archive.add_directory(seven_entry(
                name,
                mtime,
                Some(S_IFDIR | mode.unwrap_or(0o755)),
            ))?,
            Format::Tar(tar) => tar.write_header(&tar_header(
                &format!("{name}/"),
                mtime,
                mode.unwrap_or(0o755),
                hivepack::tar::TYPE_DIR,
            ))?,
            Format::One(..) => return Err(one_file_only()),
        }
        Ok(())
    }

    fn file(
        &mut self,
        new: &NewArchive,
        name: &str,
        mtime: Option<i64>,
        mode: Option<u32>,
        data: &mut dyn Read,
        len: u64,
    ) -> Result<(), Error> {
        let stored = new.level == Level::Store || has_extension_in(name, &new.stored_extensions);
        match self {
            Format::Zip(zip) => {
                if stored {
                    zip.set_compression(i32::from(cm::STORE), 0);
                } else {
                    zip.set_compression(cm::DEFAULT, new.level.deflate());
                }
                zip.add_file(&zip_entry(name, mtime, mode, Some(len)), data)?;
            }
            Format::SevenZip { archive, storing } => {
                // Each change of method starts a folder of its own.
                if stored != *storing {
                    archive.set_method(match stored {
                        true => hivepack::sevenz::Method::Copy,
                        false => hivepack::sevenz::Method::Lzma2 {
                            preset: new.level.xz(),
                        },
                    })?;
                    *storing = stored;
                }
                archive.start_file(seven_entry(
                    name,
                    mtime,
                    Some(S_IFREG | mode.unwrap_or(0o644)),
                ))?;
                copy(data, archive)?;
            }
            Format::Tar(tar) => {
                let mut header =
                    tar_header(name, mtime, mode.unwrap_or(0o644), hivepack::tar::TYPE_REG);
                header.size = i64::try_from(len)
                    .map_err(|_| Error::Failed(tr!("{path} is too big.", path = name)))?;
                tar.write_header(&header)?;
                copy(data, tar)?;
            }
            Format::One(out, written) => {
                if *written {
                    return Err(one_file_only());
                }
                copy(data, out)?;
                *written = true;
            }
        }
        Ok(())
    }

    fn symlink(&mut self, name: &str, mtime: Option<i64>, target: &str) -> Result<(), Error> {
        match self {
            Format::Zip(zip) => zip.add_symlink(&zip_entry(name, mtime, None, None), target)?,
            Format::SevenZip { archive, .. } => {
                archive.start_file(seven_entry(name, mtime, Some(S_IFLNK | 0o777)))?;
                archive.write_all(target.as_bytes())?;
            }
            Format::Tar(tar) => {
                let mut header = tar_header(name, mtime, 0o777, hivepack::tar::TYPE_SYMLINK);
                header.linkname = target.as_bytes().to_vec();
                tar.write_header(&header)?;
            }
            Format::One(..) => return Err(one_file_only()),
        }
        Ok(())
    }

    fn hard_link(
        &mut self,
        name: &str,
        mtime: Option<i64>,
        path: &[String],
        target: &[String],
    ) -> Result<(), Error> {
        match self {
            Format::Tar(tar) => {
                let mut header = tar_header(name, mtime, 0o644, hivepack::tar::TYPE_LINK);
                header.linkname = target.join("/").into_bytes();
                tar.write_header(&header)?;
                Ok(())
            }
            _ => self.symlink(name, mtime, &relative(path, target)),
        }
    }

    fn finish(self) -> Result<(), Error> {
        match self {
            Format::Zip(zip) => {
                zip.finish()?;
            }
            Format::SevenZip { archive, .. } => {
                archive.finish()?;
            }
            Format::Tar(mut tar) => {
                tar.close()?;
                tar.into_inner().finish()?.finish()?;
            }
            Format::One(out, written) => {
                if !written {
                    return Err(Error::Failed(tr!("Nothing to compress.")));
                }
                out.finish()?.finish()?;
            }
        }
        Ok(())
    }
}

fn one_file_only() -> Error {
    Error::Failed(tr!("This format holds one file, and no more."))
}

/// The path from the folder of the entry at `path` to `target`, with `/` between.
fn relative(path: &[String], target: &[String]) -> String {
    let folder = &path[..path.len().saturating_sub(1)];
    let within = &target[..target.len().saturating_sub(1)];
    let common = folder
        .iter()
        .zip(within)
        .take_while(|(a, b)| a == b)
        .count();
    let up = std::iter::repeat_n("..", folder.len() - common);
    up.chain(target[common..].iter().map(String::as_str))
        .collect::<Vec<_>>()
        .join("/")
}

fn zip_entry(
    name: &str,
    mtime: Option<i64>,
    mode: Option<u32>,
    size: Option<u64>,
) -> hivepack::zip::NewEntry {
    let mut entry = hivepack::zip::NewEntry::new(name);
    entry.mtime = mtime;
    entry.mode = mode;
    entry.size = size;
    entry
}

/// An entry of a 7z archive, its Unix `mode` (with its type) kept as 7-Zip keeps it.
fn seven_entry(name: &str, mtime: Option<i64>, mode: Option<u32>) -> hivepack::sevenz::NewEntry {
    let attributes = mode.map(|mode| {
        let directory = if mode & S_IFMT == S_IFDIR { 0x10 } else { 0 };
        (mode << 16) | 0x8000 | directory | u32::from(mode & 0o200 == 0)
    });
    // In 100 ns since 1601, which an earlier time cannot be.
    let ntfs = mtime
        .and_then(|secs| secs.checked_add(11_644_473_600))
        .and_then(|secs| u64::try_from(secs).ok())
        .and_then(|secs| secs.checked_mul(10_000_000));
    hivepack::sevenz::NewEntry {
        name: name.to_owned(),
        attributes,
        mtime: ntfs,
        ctime: None,
    }
}

fn tar_header(name: &str, mtime: Option<i64>, mode: u32, typeflag: u8) -> hivepack::tar::Header {
    hivepack::tar::Header {
        name: name.as_bytes().to_vec(),
        mode: i64::from(mode & 0o7777),
        mod_time: mtime.map(|secs| hivepack::tar::Time { secs, nanos: 0 }),
        typeflag,
        ..hivepack::tar::Header::default()
    }
}

/// `from` into `to`, a piece at a time.
fn copy(from: &mut dyn Read, to: &mut impl Write) -> io::Result<u64> {
    let mut buf = vec![0; PIECE];
    let mut n = 0;
    loop {
        match from.read(&mut buf) {
            Ok(0) => return Ok(n),
            Ok(k) => {
                to.write_all(&buf[..k])?;
                n += k as u64;
            }
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
}

/// `time`, in seconds since 1970.
fn seconds(time: SystemTime) -> i64 {
    match time.duration_since(UNIX_EPOCH) {
        Ok(d) => i64::try_from(d.as_secs()).unwrap_or(i64::MAX),
        Err(e) => -i64::try_from(e.duration().as_secs()).unwrap_or(i64::MAX),
    }
}

/// A file being given, held until all of it is.
struct Held {
    name: String,
    mtime: Option<i64>,
    mode: Option<u32>,
    data: Kept,
    len: u64,
}

/// Where what is held of a file is.
enum Kept {
    Memory(Vec<u8>),
    Spilled(Spooled),
}

impl Held {
    fn write(&mut self, data: &[u8]) -> io::Result<()> {
        match &mut self.data {
            Kept::Memory(held) if held.len() + data.len() <= HELD_IN_MEMORY => {
                held.extend_from_slice(data)
            }
            Kept::Memory(held) => {
                let mut spilled = scratch()?;
                spilled.file.write_all(held)?;
                spilled.file.write_all(data)?;
                self.data = Kept::Spilled(spilled);
            }
            Kept::Spilled(spilled) => spilled.file.write_all(data)?,
        }
        self.len += data.len() as u64;
        Ok(())
    }
}

/// What takes each entry of the archive read into the new one.
struct Converter<'a, 'n> {
    format: Format<'a>,
    new: &'n NewArchive,
    held: Option<Held>,
    /// What went wrong writing the new archive, said as it is: the extraction is
    /// told only that something did.
    failed: Option<Error>,
    /// The entries written.
    taken: u64,
}

impl Converter<'_, '_> {
    /// `result`, as the extraction is told it.
    fn told(&mut self, result: Result<(), Error>) -> io::Result<()> {
        match result {
            Ok(()) => {
                self.taken += 1;
                Ok(())
            }
            Err(e) => {
                let said = e.to_string();
                self.failed.get_or_insert(e);
                Err(io::Error::other(said))
            }
        }
    }
}

/// What `entry` is named in the new archive.
fn name_of(entry: &Entry<'_>) -> String {
    entry.path.join("/")
}

impl Sink for Converter<'_, '_> {
    fn directory(&mut self, entry: &Entry<'_>) -> io::Result<()> {
        let mtime = entry.mtime.map(seconds);
        let result = self.format.directory(&name_of(entry), mtime, entry.mode);
        self.told(result)
    }

    fn start_file(&mut self, entry: &Entry<'_>) -> io::Result<()> {
        self.held = Some(Held {
            name: name_of(entry),
            mtime: entry.mtime.map(seconds),
            mode: entry.mode,
            data: Kept::Memory(Vec::new()),
            len: 0,
        });
        Ok(())
    }

    fn write(&mut self, data: &[u8]) -> io::Result<()> {
        match &mut self.held {
            Some(held) => held.write(data),
            None => Err(io::Error::other("no file is being given")),
        }
    }

    fn finish_file(&mut self) -> io::Result<()> {
        let Some(mut held) = self.held.take() else {
            return Err(io::Error::other("no file is being given"));
        };
        let new = self.new;
        let result = (|| {
            let mut data: Box<dyn Read + '_> = match &mut held.data {
                Kept::Memory(bytes) => Box::new(Cursor::new(std::mem::take(bytes))),
                Kept::Spilled(spilled) => {
                    spilled.file.flush()?;
                    Box::new(spilled.rewound()?)
                }
            };
            self.format
                .file(new, &held.name, held.mtime, held.mode, &mut data, held.len)
        })();
        self.told(result)
    }

    fn discard_file(&mut self) {
        self.held = None;
    }

    fn symlink(&mut self, entry: &Entry<'_>, target: &str) -> io::Result<()> {
        let mtime = entry.mtime.map(seconds);
        let result = self.format.symlink(&name_of(entry), mtime, target);
        self.told(result)
    }

    fn hard_link(&mut self, entry: &Entry<'_>, target: &[String]) -> io::Result<()> {
        let mtime = entry.mtime.map(seconds);
        let result = self
            .format
            .hard_link(&name_of(entry), mtime, entry.path, target);
        self.told(result)
    }
}

impl Taker for Converter<'_, '_> {
    fn one_file(
        &mut self,
        name: &str,
        data: &mut dyn Read,
        control: &Control,
    ) -> Result<u64, Error> {
        // Read through a check of whether to stop, and held, as any file is.
        let mut held = Held {
            name: name.to_owned(),
            mtime: None,
            mode: None,
            data: Kept::Memory(Vec::new()),
            len: 0,
        };
        let mut buf = vec![0; PIECE];
        loop {
            control.check()?;
            match data.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => held.write(&buf[..n])?,
                Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => return Err(e.into()),
            }
        }
        let len = held.len;
        self.held = Some(held);
        self.finish_file()
            .map_err(|e| self.failed.take().unwrap_or_else(|| e.into()))?;
        Ok(len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(p: &str) -> Vec<String> {
        p.split('/').map(str::to_owned).collect()
    }

    #[test]
    fn a_hard_link_s_target_is_from_its_folder() {
        assert_eq!(relative(&path("a/b/link"), &path("a/b/file")), "file");
        assert_eq!(relative(&path("a/link"), &path("a/b/file")), "b/file");
        assert_eq!(relative(&path("a/b/link"), &path("a/c/file")), "../c/file");
        assert_eq!(relative(&path("link"), &path("x/y/file")), "x/y/file");
        assert_eq!(
            relative(&path("a/b/c/link"), &path("file")),
            "../../../file"
        );
    }

    #[test]
    fn mac_files_are_told_anywhere_in_a_path() {
        assert!(!not_mac_file(&path("docs/.DS_Store")));
        assert!(!not_mac_file(&path("__MACOSX/docs/._a")));
        assert!(not_mac_file(&path("docs/a.txt")));
    }
}
