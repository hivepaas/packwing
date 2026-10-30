//! What an archive holds: its entries, as it names them, and its folders
//! as they show, those it only implies by its paths too.

use std::collections::BTreeMap;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use hivepack_open::{Container, Layer};
use hivepack_tar::{Reader, TYPE_DIR, TYPE_X_GLOBAL_HEADER};
use hivepack_zip::{Archive, Flags, OpenFlags, em};

use crate::spool::spool;
use crate::{Error, Keys, opener, without_layers};

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

/// What the archive at `path` holds, opened with `keys`.
pub fn list(path: &Path, keys: &Keys) -> Result<Listing, Error> {
    let packed = std::fs::metadata(path)?.len();
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
                zip_entries(std::fs::File::open(path)?)?
            } else {
                let spooled = spool(&mut opened)?;
                zip_entries(&spooled.file)?
            };
            encryption = encryption.or(zip_encryption);
            entries
        }
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
