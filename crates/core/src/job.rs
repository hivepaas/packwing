//! Extracting an archive, and testing it: every file read through, its
//! checks made, nothing written.

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, Write};
use std::path::Path;

use hivepack_extract::{
    Cancel, Options, Origin, Progress, ProgressFn, Summary, extract_7z, extract_cpio, extract_iso,
    extract_tar, extract_udf, extract_xar, extract_zip,
};
use hivepack_open::{Container, Opened};
use hivepack_tar::Reader;
use hivepack_udf::{Udf, VType};
use hivepack_zip::{Archive, Flags, OpenFlags, ReadAt};

use crate::spool::spool;
use crate::volume::walk;
use crate::{Error, Keys, opener, seven_zip, without_layers};

/// What a job tells how far it is, and what stops it.
#[derive(Clone, Default)]
pub struct Control {
    pub progress: Option<ProgressFn>,
    pub cancel: Option<Cancel>,
}

impl Control {
    fn check(&self) -> Result<(), Error> {
        match &self.cancel {
            Some(c) if c.is_cancelled() => Err(Error::Cancelled),
            _ => Ok(()),
        }
    }

    fn report(&self, p: &Progress) {
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

/// Extracts the archive at `path` into `destination`, made if it is not
/// there, with hivepack-extract's checks: nothing outside it, over what is
/// there, or through a link. The files keep the archive's mark of where it
/// came from.
pub fn extract(
    path: &Path,
    destination: &Path,
    keys: &Keys,
    control: &Control,
) -> Result<Summary, Error> {
    let mut options = Options::default();
    options.password = keys.password.clone();
    options.progress = control.progress.clone();
    options.cancel = control.cancel.clone();
    options.origin = Origin::of(path)?;
    let mut opener = opener(path, keys);
    let mut opened = opener.open()?;
    let layers = opened.layers().to_vec();
    match opened.container() {
        Container::Tar => {
            drop(opened);
            Ok(extract_tar(|| Ok(opener.open()?), destination, &options)?)
        }
        Container::Zip if layers.is_empty() => {
            drop(opened);
            let archive = zip_archive(File::open(path)?)?;
            Ok(extract_zip(&archive, destination, &options)?)
        }
        Container::Zip => {
            let spooled = spool(&mut opened)?;
            let archive = zip_archive(&spooled.file)?;
            Ok(extract_zip(&archive, destination, &options)?)
        }
        // Read twice, as a tar archive is: once to check, once to write.
        Container::Iso => {
            drop(opened);
            Ok(extract_iso(|| Ok(opener.open()?), destination, &options)?)
        }
        Container::Udf if layers.is_empty() => {
            drop(opened);
            let mut udf = Udf::open(File::open(path)?)?;
            Ok(extract_udf(&mut udf, destination, &options)?)
        }
        Container::Udf => {
            let spooled = spool(&mut opened)?;
            let mut udf = Udf::open(&spooled.file)?;
            Ok(extract_udf(&mut udf, destination, &options)?)
        }
        Container::SevenZip if layers.is_empty() => {
            drop(opened);
            let mut seven = seven_zip(|| File::open(path), keys)?;
            Ok(extract_7z(&mut seven.archive, destination, &options)?)
        }
        Container::SevenZip => {
            let spooled = spool(&mut opened)?;
            let mut seven = seven_zip(|| spooled.rewound(), keys)?;
            Ok(extract_7z(&mut seven.archive, destination, &options)?)
        }
        // Read twice, as a tar archive is; an RPM package's payload, its
        // lead and headers taken off, and a macOS package's payloads,
        // expanded into folders of their names.
        Container::Cpio => {
            drop(opened);
            Ok(extract_cpio(|| Ok(opener.open()?), destination, &options)?)
        }
        Container::Xar => {
            drop(opened);
            Ok(extract_xar(|| Ok(opener.open()?), destination, &options)?)
        }
        Container::Data if layers.is_empty() => Err(Error::NotAnArchive),
        _ => {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            extract_one(
                opened,
                &without_layers(&name, &layers),
                destination,
                control,
            )
        }
    }
}

/// One file under layers, written as `name` in `destination`: never over
/// what is there, and removed if it fails or is cancelled.
fn extract_one(
    mut opened: Opened<'_>,
    name: &str,
    destination: &Path,
    control: &Control,
) -> Result<Summary, Error> {
    std::fs::create_dir_all(destination)?;
    let path = destination.join(name);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|e| match e.kind() {
            io::ErrorKind::AlreadyExists => {
                Error::Failed(format!("{name} is already in the folder"))
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
pub fn test(path: &Path, keys: &Keys, control: &Control) -> Result<Summary, Error> {
    let mut opener = opener(path, keys);
    let mut opened = opener.open()?;
    let layers = opened.layers().to_vec();
    let mut done = Progress::default();
    let mut summary = Summary::default();
    match opened.container() {
        Container::Tar => {
            let mut tr = Reader::new(&mut opened);
            while let Some(h) = tr.next()? {
                copy(&mut tr, &mut io::sink(), control, &mut done)?;
                if h.typeflag == hivepack_tar::TYPE_REG || h.typeflag == hivepack_tar::TYPE_REG_A {
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
            };
            copy(&mut tr.into_inner(), &mut io::sink(), &quiet, &mut done)?;
        }
        Container::Zip if layers.is_empty() => {
            drop(opened);
            test_zip(zip_archive(File::open(path)?)?, keys, control, &mut summary)?;
        }
        Container::Zip => {
            let spooled = spool(&mut opened)?;
            test_zip(zip_archive(&spooled.file)?, keys, control, &mut summary)?;
        }
        Container::Iso => {
            test_iso(
                hivepack_iso::Reader::new(&mut opened),
                control,
                &mut summary,
            )?;
            read_to_end(&mut opened, control, &mut done)?;
        }
        Container::Udf if layers.is_empty() => {
            drop(opened);
            test_udf(Udf::open(File::open(path)?)?, control, &mut summary)?;
        }
        Container::Udf => {
            let spooled = spool(&mut opened)?;
            test_udf(Udf::open(&spooled.file)?, control, &mut summary)?;
        }
        Container::SevenZip if layers.is_empty() => {
            drop(opened);
            let seven = seven_zip(|| File::open(path), keys)?;
            test_7z(seven.archive, control, &mut summary)?;
        }
        Container::SevenZip => {
            let spooled = spool(&mut opened)?;
            let seven = seven_zip(|| spooled.rewound(), keys)?;
            test_7z(seven.archive, control, &mut summary)?;
        }
        Container::Cpio => {
            test_cpio(
                hivepack_cpio::Reader::new(&mut opened),
                control,
                &mut summary,
            )?;
            read_to_end(&mut opened, control, &mut done)?;
        }
        Container::Xar => {
            test_xar(
                hivepack_xar::Reader::new(&mut opened),
                control,
                &mut summary,
            )?;
            read_to_end(&mut opened, control, &mut done)?;
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
    };
    copy(r, &mut io::sink(), &quiet, done).map(|_| ())
}

/// Reads every file of a 7z archive through, each folder decoded once,
/// with every file's CRC and every folder's checks: the password of the
/// encrypted ones was given when it was opened.
fn test_7z<R: Read + Seek>(
    mut archive: hivepack_7z::Archive<R>,
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
    mut cpio: hivepack_cpio::Reader<R>,
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
        if h.file_type() == hivepack_cpio::AE_IFREG && read > 0 {
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
    mut xar: hivepack_xar::Reader<R>,
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
        if h.mode & hivepack_xar::AE_IFMT == hivepack_xar::AE_IFREG && h.hardlink.is_none() {
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
    mut iso: hivepack_iso::Reader<R>,
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
        if h.file_type() == hivepack_iso::AE_IFREG && h.hardlink.is_none() {
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
