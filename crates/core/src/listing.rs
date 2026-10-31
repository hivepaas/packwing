//! What an archive holds: its entries, as it names them, and its folders
//! as they show, those it only implies by its paths too.

use std::collections::BTreeMap;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use hivepack_open::{Container, Layer, Opener};
use hivepack_tar::{Reader, TYPE_DIR, TYPE_X_GLOBAL_HEADER};
use hivepack_udf::{Udf, VType};
use hivepack_zip::{Archive, Flags, OpenFlags, Volumes, em, volume_paths};

use crate::joined::{Joined, joined_size};
use crate::spool::spool;
use crate::volume::walk;
use crate::{Error, Keys, SevenZip, opener, seven_zip, without_layers};

/// The names of a macOS package's payloads, which hivepack-extract expands
/// into folders of their names, as `pkgutil --expand-full` does: a
/// component package's files and scripts, and a `.xip` file's content.
const PAYLOADS: [&str; 3] = ["Payload", "Scripts", "Content"];

/// GNU's volume label and dump directory, in a tar archive.
const TYPE_GNU_VOLUME: u8 = b'V';
const TYPE_GNU_DUMPDIR: u8 = b'D';

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
    /// How it is encrypted, if it is: `age`, `AES-256`, `ZipCrypto`.
    pub encryption: Option<String>,
    pub entries: Vec<Entry>,
}

impl Listing {
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
                });
            if !e.dir {
                child.size += e.size;
                child.files += 1;
            }
            child.modified = child.modified.max(e.modified);
        }
        found.into_values().collect()
    }
}

/// The path of a name in an archive, a component at a time: `/` between
/// them, a leading `./` and empty components dropped.
fn components(name: &str) -> Vec<String> {
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

/// What the archive at `path` holds, opened with `keys`.
pub fn list(path: &Path, keys: &Keys) -> Result<Listing, Error> {
    let mut packed = joined_size(path)?;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let folder = path.parent().map(Path::to_path_buf).unwrap_or_default();
    let mut opener = opener(path, keys);
    let mut opened = opener.open()?;
    let layers = opened.layers().to_vec();
    let container = opened.container();
    let mut encryption = layers.contains(&Layer::Age).then(|| "age".to_owned());
    let entries = match container {
        Container::Tar => tar_entries(&mut opened)?,
        Container::Zip => {
            let (entries, zip_encryption) = if layers.is_empty() {
                drop(opened);
                {
                    packed = zip_size(path)?;
                    zip_entries(Volumes::open(path)?)?
                }
            } else {
                let spooled = spool(&mut opened)?;
                zip_entries(&spooled.file)?
            };
            encryption = encryption.or(zip_encryption);
            entries
        }
        // Read in order; from the file itself, a file whose data is before
        // its directory is read by going back.
        Container::Iso if layers.is_empty() => {
            drop(opened);
            iso_entries(hivepack_iso::Reader::seekable(Joined::open(path)?))?
        }
        Container::Iso => iso_entries(hivepack_iso::Reader::new(&mut opened))?,
        Container::Udf if layers.is_empty() => {
            drop(opened);
            udf_entries(Joined::open(path)?)?
        }
        Container::Udf => {
            let spooled = spool(&mut opened)?;
            udf_entries(&spooled.file)?
        }
        Container::SevenZip => {
            let (entries, seven_zip_encryption) = if layers.is_empty() {
                drop(opened);
                seven_zip_entries(seven_zip(|| Joined::open(path), keys)?)
            } else {
                let spooled = spool(&mut opened)?;
                seven_zip_entries(seven_zip(|| spooled.rewound(), keys)?)
            };
            encryption = encryption.or(seven_zip_encryption);
            entries
        }
        // An RPM package is its payload: the package's lead and headers
        // are a layer.
        Container::Cpio => cpio_entries(&mut opened)?,
        Container::Xar => xar_entries(&mut opened)?,
        Container::Data if layers.is_empty() => return Err(Error::NotAnArchive),
        _ => {
            let size = io::copy(&mut opened, &mut io::sink())?;
            vec![Entry {
                path: vec![without_layers(&name, &layers)],
                dir: false,
                size,
                modified: None,
            }]
        }
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
        encryption,
        entries,
    })
}

fn tar_entries(r: impl Read) -> Result<Vec<Entry>, Error> {
    let mut tr = Reader::new(r);
    let mut entries = Vec::new();
    while let Some(h) = tr.next()? {
        if matches!(h.typeflag, TYPE_X_GLOBAL_HEADER | TYPE_GNU_VOLUME) {
            continue;
        }
        let path = components(&String::from_utf8_lossy(&h.name));
        if path.is_empty() {
            continue;
        }
        let dir = matches!(h.typeflag, TYPE_DIR | TYPE_GNU_DUMPDIR) || h.name.ends_with(b"/");
        entries.push(Entry {
            path,
            dir,
            size: if dir { 0 } else { h.size.max(0) as u64 },
            modified: h.mod_time.map(|t| t.secs),
        });
    }
    Ok(entries)
}

fn zip_entries<R: hivepack_zip::ReadAt>(source: R) -> Result<(Vec<Entry>, Option<String>), Error> {
    let archive = Archive::open(source, OpenFlags::CHECKCONS | OpenFlags::RDONLY)?;
    let mut entries = Vec::new();
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
        entries.push(Entry {
            path,
            dir,
            size: if dir { 0 } else { stat.size },
            modified: Some(stat.mtime),
        });
    }
    Ok((entries, encryption))
}

/// An ISO 9660 image's entries: Rock Ridge's names, else Joliet's, else
/// ISO 9660's. The root directory, `.`, is not one, nor is an entry
/// whose name could not be made.
fn iso_entries<R: Read>(mut iso: hivepack_iso::Reader<R>) -> Result<Vec<Entry>, Error> {
    let mut entries = Vec::new();
    while let Some(h) = iso.next()? {
        let Some(name) = &h.path else {
            continue;
        };
        let path = components(&String::from_utf8_lossy(name));
        if path.is_empty() {
            continue;
        }
        let dir = h.is_dir();
        entries.push(Entry {
            path,
            dir,
            // A hard link has no size: its data, the entry's before it, is
            // counted there, once.
            size: if dir { 0 } else { h.size.unwrap_or(0) },
            modified: h.mod_time,
        });
    }
    Ok(entries)
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
        entries.push(Entry {
            path,
            dir: e.is_dir,
            size: if e.is_dir { 0 } else { e.size },
            modified: e.mtime.map(ntfs_to_unix),
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
fn cpio_entries(r: impl Read) -> Result<Vec<Entry>, Error> {
    let mut cpio = hivepack_cpio::Reader::new(r);
    let mut entries = Vec::new();
    while let Some(h) = cpio.next()? {
        let path = components(&String::from_utf8_lossy(&h.path));
        if path.is_empty() {
            continue;
        }
        let kind = h.file_type();
        let dir = kind == hivepack_cpio::AE_IFDIR;
        entries.push(Entry {
            path,
            dir,
            size: if kind == hivepack_cpio::AE_IFREG {
                h.size
            } else {
                0
            },
            modified: Some(h.mod_time),
        });
    }
    Ok(entries)
}

/// A XAR archive's entries. A macOS package's payloads (`Payload`,
/// `Scripts`, a `.xip` file's `Content`), cpio archives under gzip or
/// pbzx, show as folders of what they hold, as they extract.
fn xar_entries(r: impl Read + Send) -> Result<Vec<Entry>, Error> {
    let mut xar = hivepack_xar::Reader::new(r);
    let mut entries = Vec::new();
    while let Some(h) = xar.next()? {
        let Some(name) = &h.path else {
            continue;
        };
        let path = components(&String::from_utf8_lossy(name));
        if path.is_empty() {
            continue;
        }
        let kind = h.mode & hivepack_xar::AE_IFMT;
        let is_payload = kind == hivepack_xar::AE_IFREG
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
            });
            entries.extend(inside.into_iter().map(|mut e| {
                e.path.splice(0..0, path.iter().cloned());
                e
            }));
            continue;
        }
        let dir = kind == hivepack_xar::AE_IFDIR;
        entries.push(Entry {
            path,
            dir,
            size: if kind == hivepack_xar::AE_IFREG && h.hardlink.is_none() {
                h.size
            } else {
                0
            },
            modified: h.mtime,
        });
    }
    Ok(entries)
}

/// What the payload the XAR reader is at holds, if it is a cpio archive
/// under its layers; `None` if it is something else, which shows as a
/// file.
fn payload_entries<R: Read + Send>(
    xar: &mut hivepack_xar::Reader<R>,
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
    cpio_entries(opened).map(Some)
}

/// A UDF volume's entries, as its directories list them.
fn udf_entries<R: Read + io::Seek>(image: R) -> Result<Vec<Entry>, Error> {
    let mut udf = Udf::open(image)?;
    let mut entries = Vec::new();
    walk(&mut udf, |_, path, _, stat| {
        let dir = stat.kind == VType::VDIR;
        entries.push(Entry {
            path: path.to_vec(),
            dir,
            size: if stat.kind == VType::VREG {
                stat.size
            } else {
                0
            },
            modified: Some(stat.mtime.sec),
        });
        Ok(())
    })?;
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn components_of_names() {
        assert_eq!(components("./a/b.txt"), ["a", "b.txt"]);
        assert_eq!(components("a//b/"), ["a", "b"]);
        assert_eq!(components("/abs/x"), ["abs", "x"]);
        assert_eq!(components("a/./b"), ["a", ".", "b"]);
        assert!(components("./").is_empty());
    }
}
