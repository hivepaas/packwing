//! Where files are kept, and what can be done with them there: a folder on this
//! computer so far. The window browses a [`Storage`] without knowing which kind
//! it is, and asks for a folder's items a page at a time, as a storage across a
//! network gives them.
//!
//! Everything here blocks: the window calls it from the background.

use std::cmp::Ordering;
use std::fs;
use std::io::{self, Read, Seek};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, UNIX_EPOCH};

use packwing_i18n::tr;

use crate::Cancel;
use crate::listing::folded;

/// A file or folder in a folder of a storage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    /// Its name, as it can be shown. A name that is not valid Unicode shows
    /// with replacement characters, and that name finds nothing.
    pub name: String,
    pub dir: bool,
    /// A symbolic link. A count does not go into one.
    pub link: bool,
    pub hidden: bool,
    /// In bytes; 0 for a folder.
    pub size: u64,
    /// In seconds since 1970, if the storage knows.
    pub modified: Option<i64>,
    /// Its Unix permissions, if it has any.
    pub mode: Option<u32>,
}

/// A folder's items, handed out a page at a time.
pub trait Lister: Send {
    /// Up to `most` more; none when there are no more.
    fn next(&mut self, most: usize) -> io::Result<Vec<Item>>;

    /// Whether there is nothing left.
    fn done(&self) -> bool;
}

/// A place files are kept in. A folder is named by its path from the top, a
/// name at a time.
pub trait Storage: Send + Sync {
    fn list(&self, folder: &[String]) -> io::Result<Box<dyn Lister>>;

    /// Up to `most` bytes from the start of the file at `path`.
    fn read(&self, path: &[String], most: u64) -> io::Result<Vec<u8>>;

    /// Up to `len` bytes of the file at `path`, from `offset`; fewer only at its end.
    fn read_at(&self, path: &[String], offset: u64, len: usize) -> io::Result<Vec<u8>>;

    /// The item called `name` in `folder`, if there is one. A link is not followed.
    fn stat(&self, folder: &[String], name: &str) -> io::Result<Option<Item>>;

    /// Reads the file at `path`.
    fn open(&self, path: &[String]) -> io::Result<Box<dyn Read + Send>>;

    /// Writes the file at `path` from `from`. It appears under its name only once
    /// all of it is written, with the time and permissions of `like`; with
    /// `replace`, over a file that is there, and otherwise a name taken is an error.
    fn put(
        &self,
        path: &[String],
        from: &mut dyn Read,
        like: &Item,
        replace: bool,
    ) -> io::Result<u64>;

    /// Removes the file or empty folder at `path` for good. For what has been
    /// moved: [`Storage::delete`] is for what the user deletes.
    fn remove(&self, path: &[String]) -> io::Result<()>;

    /// Makes a new folder; a name taken is an error.
    fn make_folder(&self, folder: &[String], name: &str) -> io::Result<()>;

    /// Renames an item; a name taken is an error.
    fn rename(&self, folder: &[String], from: &str, to: &str) -> io::Result<()>;

    /// Deletes items, or moves them to the system's Trash where there is one.
    fn delete(&self, folder: &[String], names: &[String]) -> io::Result<()>;

    /// Whether [`Storage::delete`] leaves them in the system's Trash.
    fn has_trash(&self) -> bool;

    /// Where `path` is on this computer, if it is on this computer: archives
    /// are opened and files shown to other programs from there.
    fn local_path(&self, path: &[String]) -> Option<PathBuf>;

    /// `path` as the user would write it.
    fn show_path(&self, path: &[String]) -> String;
}

/// A setting of a storage, by its key.
pub type Values = std::collections::BTreeMap<String, String>;

/// Something asked for when a storage is added.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field {
    pub key: &'static str,
    pub label: String,
    /// What the empty field says.
    pub hint: String,
    /// Kept in the system's keychain, not with the settings, and not shown.
    pub secret: bool,
}

/// A kind of storage a build of packwing can add, besides a folder on this
/// computer. Each storage of a kind has an id, under which its secrets are kept.
pub trait StorageKind: Send + Sync {
    /// Its name in the settings file.
    fn id(&self) -> &'static str;

    /// The letters on its tile.
    fn tag(&self) -> &'static str;

    /// What the user calls this kind.
    fn name(&self) -> String;

    fn fields(&self) -> Vec<Field>;

    /// Under its name in the list: where it is.
    fn describe(&self, values: &Values) -> String;

    /// The storage with these settings. It touches neither the network nor the
    /// keychain until first used, so that a list of them opens at once; with
    /// `typed`, the secrets are those and not the kept ones.
    fn open(&self, id: &str, values: &Values, typed: Option<&Values>) -> Arc<dyn Storage>;

    fn keep_secrets(&self, id: &str, secrets: &Values) -> io::Result<()>;

    fn forget_secrets(&self, id: &str);
}

/// Why `name` cannot be an item's name, if it cannot. A name is one step: it
/// leads nowhere else.
pub fn bad_name(name: &str) -> Option<String> {
    if name.trim().is_empty() {
        return Some(tr!("The name is empty."));
    }
    if name == "." || name == ".." {
        return Some(tr!("That name is kept for the system."));
    }
    let forbidden: &[char] = if cfg!(windows) {
        &['/', '\\', ':', '*', '?', '"', '<', '>', '|', '\0']
    } else {
        &['/', '\0']
    };
    if name.contains(forbidden) {
        return Some(tr!("A name cannot hold / or other special characters."));
    }
    if cfg!(windows) && name.ends_with(['.', ' ']) {
        return Some(tr!("On Windows a name cannot end with a dot or a space."));
    }
    None
}

/// The order of names as people read them: whatever the case and accents, and
/// numbers by their value (`file2` before `file10`).
pub fn by_name(a: &str, b: &str) -> Ordering {
    let (x, y) = (folded(a), folded(b));
    let (mut x, mut y) = (x.as_str(), y.as_str());
    while let (Some(p), Some(q)) = (x.chars().next(), y.chars().next()) {
        if p.is_ascii_digit() && q.is_ascii_digit() {
            let (m, rest_x) = digits(x);
            let (n, rest_y) = digits(y);
            let (m, n) = (m.trim_start_matches('0'), n.trim_start_matches('0'));
            match m.len().cmp(&n.len()).then_with(|| m.cmp(n)) {
                Ordering::Equal => (x, y) = (rest_x, rest_y),
                other => return other,
            }
        } else if p != q {
            return p.cmp(&q);
        } else {
            (x, y) = (&x[p.len_utf8()..], &y[q.len_utf8()..]);
        }
    }
    x.len().cmp(&y.len()).then_with(|| a.cmp(b))
}

fn digits(s: &str) -> (&str, &str) {
    let end = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
    s.split_at(end)
}

/// Whether `name` holds `query`, however they are written: `query` as
/// [`folded_query`] gives it.
pub fn name_holds(name: &str, query: &str) -> bool {
    folded(name).contains(query)
}

/// What is typed in a search, as [`name_holds`] takes it.
pub fn folded_query(typed: &str) -> String {
    folded(typed.trim())
}

/// How much a folder holds, with all under it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counted {
    pub files: u64,
    pub folders: u64,
    pub bytes: u64,
}

/// Counts what `folder` holds, saying how far it has come every so often. Links
/// are counted as themselves, not gone into: one may lead back up, or out of the
/// storage. A folder that cannot be read is left out, and `unread` says how many.
pub fn count(
    storage: &dyn Storage,
    folder: &[String],
    cancel: &Cancel,
    mut progress: impl FnMut(Counted),
) -> io::Result<(Counted, u64)> {
    const PAGE: usize = 1000;
    let mut counted = Counted::default();
    let mut unread = 0;
    let mut waiting = vec![folder.to_vec()];
    while let Some(at) = waiting.pop() {
        let Ok(mut lister) = storage.list(&at) else {
            unread += 1;
            continue;
        };
        while !lister.done() {
            if cancel.is_cancelled() {
                return Err(io::ErrorKind::Interrupted.into());
            }
            let Ok(page) = lister.next(PAGE) else {
                unread += 1;
                break;
            };
            for item in page {
                if item.dir && !item.link {
                    counted.folders += 1;
                    let mut next = at.clone();
                    next.push(item.name);
                    waiting.push(next);
                } else {
                    counted.files += 1;
                    counted.bytes += item.size;
                }
            }
            progress(counted);
        }
    }
    Ok((counted, unread))
}

/// A folder on this computer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalFolder {
    top: PathBuf,
}

impl LocalFolder {
    pub fn new(top: impl Into<PathBuf>) -> LocalFolder {
        LocalFolder { top: top.into() }
    }

    /// Where `path` is, each step of it a plain name: nothing a storage is
    /// asked for leaves it by way of its names. A link in it may.
    fn at(&self, path: &[String]) -> io::Result<PathBuf> {
        let mut at = self.top.clone();
        for step in path {
            if let Some(why) = bad_name(step) {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, why));
            }
            at.push(step);
        }
        Ok(at)
    }
}

/// Whether two names are one, on a system that may not tell their cases apart:
/// the file itself is renamed then, not another one written over.
fn same_file_differently_written(from: &Path, to: &Path) -> bool {
    let same_name = match (from.file_name(), to.file_name()) {
        (Some(a), Some(b)) => {
            a.to_string_lossy().to_lowercase() == b.to_string_lossy().to_lowercase()
        }
        _ => false,
    };
    same_name
        && match (fs::symlink_metadata(from), fs::symlink_metadata(to)) {
            (Ok(a), Ok(b)) => same_entry(&a, &b),
            _ => false,
        }
}

#[cfg(unix)]
fn same_entry(a: &fs::Metadata, b: &fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    a.dev() == b.dev() && a.ino() == b.ino()
}

#[cfg(not(unix))]
fn same_entry(a: &fs::Metadata, b: &fs::Metadata) -> bool {
    a.len() == b.len() && a.modified().ok() == b.modified().ok() && a.is_dir() == b.is_dir()
}

impl Storage for LocalFolder {
    fn list(&self, folder: &[String]) -> io::Result<Box<dyn Lister>> {
        let at = self.at(folder)?;
        let mut names: Vec<(String, std::ffi::OsString, bool)> = Vec::new();
        for entry in fs::read_dir(&at)? {
            // One gone since the folder was opened is not there.
            let Ok(entry) = entry else { continue };
            let os = entry.file_name();
            // A link to a folder sorts with the folders.
            let dir = match entry.file_type() {
                Ok(t) if t.is_symlink() => fs::metadata(entry.path()).is_ok_and(|m| m.is_dir()),
                Ok(t) => t.is_dir(),
                Err(_) => false,
            };
            names.push((os.to_string_lossy().into_owned(), os, dir));
        }
        names.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| by_name(&a.0, &b.0)));
        Ok(Box::new(LocalLister {
            folder: at,
            names: names.into_iter(),
            left: None,
        }))
    }

    fn read(&self, path: &[String], most: u64) -> io::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        fs::File::open(self.at(path)?)?
            .take(most)
            .read_to_end(&mut bytes)?;
        Ok(bytes)
    }

    fn read_at(&self, path: &[String], offset: u64, len: usize) -> io::Result<Vec<u8>> {
        let mut file = fs::File::open(self.at(path)?)?;
        file.seek(io::SeekFrom::Start(offset))?;
        let mut bytes = Vec::with_capacity(len);
        file.take(len as u64).read_to_end(&mut bytes)?;
        Ok(bytes)
    }

    fn stat(&self, folder: &[String], name: &str) -> io::Result<Option<Item>> {
        let mut path = folder.to_vec();
        path.push(name.to_owned());
        let at = self.at(&path)?;
        let meta = match fs::symlink_metadata(&at) {
            Ok(meta) => meta,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e),
        };
        let dir = if meta.file_type().is_symlink() {
            fs::metadata(&at).is_ok_and(|m| m.is_dir())
        } else {
            meta.is_dir()
        };
        Ok(Some(item_of(name.to_owned(), dir, &meta)))
    }

    fn open(&self, path: &[String]) -> io::Result<Box<dyn Read + Send>> {
        Ok(Box::new(fs::File::open(self.at(path)?)?))
    }

    fn put(
        &self,
        path: &[String],
        from: &mut dyn Read,
        like: &Item,
        replace: bool,
    ) -> io::Result<u64> {
        let to = self.at(path)?;
        let folder = to.parent().unwrap_or(&self.top);
        if !replace && fs::symlink_metadata(&to).is_ok() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                tr!("That name is taken in this folder."),
            ));
        }
        let (part, mut file) = part_file(folder)?;
        // Taken away again unless it gets its name.
        let mut guard = RemoveUnlessKept(Some(part));
        let written = io::copy(from, &mut file)?;
        if let Some(when) = like.modified.and_then(|s| u64::try_from(s).ok()) {
            file.set_modified(UNIX_EPOCH + Duration::from_secs(when))?;
        }
        #[cfg(unix)]
        if let Some(mode) = like.mode {
            use std::os::unix::fs::PermissionsExt;
            // Not setuid or the like: a copy is not to gain what it should not.
            file.set_permissions(fs::Permissions::from_mode(mode & 0o777))?;
        }
        file.sync_all()?;
        drop(file);
        let part = guard.0.as_ref().expect("the part file is kept till here");
        fs::rename(part, &to)?;
        guard.0 = None;
        Ok(written)
    }

    fn remove(&self, path: &[String]) -> io::Result<()> {
        let at = self.at(path)?;
        if fs::symlink_metadata(&at)?.is_dir() {
            fs::remove_dir(at)
        } else {
            fs::remove_file(at)
        }
    }

    fn make_folder(&self, folder: &[String], name: &str) -> io::Result<()> {
        let mut path = folder.to_vec();
        path.push(name.to_owned());
        fs::create_dir(self.at(&path)?)
    }

    fn rename(&self, folder: &[String], from: &str, to: &str) -> io::Result<()> {
        let (mut a, mut b) = (folder.to_vec(), folder.to_vec());
        a.push(from.to_owned());
        b.push(to.to_owned());
        let (a, b) = (self.at(&a)?, self.at(&b)?);
        if fs::symlink_metadata(&b).is_ok() && !same_file_differently_written(&a, &b) {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                tr!("That name is taken in this folder."),
            ));
        }
        fs::rename(a, b)
    }

    fn delete(&self, folder: &[String], names: &[String]) -> io::Result<()> {
        let mut paths = Vec::with_capacity(names.len());
        for name in names {
            let mut path = folder.to_vec();
            path.push(name.clone());
            paths.push(self.at(&path)?);
        }
        trash::delete_all(paths).map_err(io::Error::other)
    }

    fn has_trash(&self) -> bool {
        true
    }

    fn local_path(&self, path: &[String]) -> Option<PathBuf> {
        self.at(path).ok()
    }

    fn show_path(&self, path: &[String]) -> String {
        self.at(path)
            .unwrap_or_else(|_| self.top.clone())
            .display()
            .to_string()
    }
}

/// A new file in `folder` to write to, under a name of its own.
fn part_file(folder: &Path) -> io::Result<(PathBuf, fs::File)> {
    for n in 0.. {
        let part = folder.join(format!(".packwing-part-{}-{n}", std::process::id()));
        match fs::File::options().write(true).create_new(true).open(&part) {
            Ok(file) => return Ok((part, file)),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e),
        }
    }
    unreachable!("the numbers do not run out")
}

/// A part file, removed when it goes out of scope unless taken back.
struct RemoveUnlessKept(Option<PathBuf>);

impl Drop for RemoveUnlessKept {
    fn drop(&mut self) {
        if let Some(part) = &self.0 {
            let _ = fs::remove_file(part);
        }
    }
}

fn item_of(name: String, dir: bool, meta: &fs::Metadata) -> Item {
    Item {
        hidden: is_hidden(&name, meta),
        link: meta.file_type().is_symlink(),
        size: if dir { 0 } else { meta.len() },
        modified: meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64),
        mode: mode_of(meta),
        name,
        dir,
    }
}

struct LocalLister {
    folder: PathBuf,
    names: std::vec::IntoIter<(String, std::ffi::OsString, bool)>,
    /// The next one, looked at to know whether any are left.
    left: Option<(String, std::ffi::OsString, bool)>,
}

impl Lister for LocalLister {
    fn next(&mut self, most: usize) -> io::Result<Vec<Item>> {
        let mut items = Vec::with_capacity(most.min(self.names.len() + 1));
        while items.len() < most {
            let Some((name, os, dir)) = self.left.take().or_else(|| self.names.next()) else {
                break;
            };
            // One gone since the folder was read is not there.
            let Ok(meta) = fs::symlink_metadata(self.folder.join(&os)) else {
                continue;
            };
            items.push(item_of(name, dir, &meta));
        }
        self.left = self.names.next();
        Ok(items)
    }

    fn done(&self) -> bool {
        self.left.is_none() && self.names.len() == 0
    }
}

#[cfg(unix)]
fn mode_of(meta: &fs::Metadata) -> Option<u32> {
    use std::os::unix::fs::PermissionsExt;
    Some(meta.permissions().mode() & 0o7777)
}

#[cfg(not(unix))]
fn mode_of(_: &fs::Metadata) -> Option<u32> {
    None
}

#[cfg(windows)]
fn is_hidden(name: &str, meta: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_HIDDEN: u32 = 2;
    name.starts_with('.') || meta.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0
}

#[cfg(not(windows))]
fn is_hidden(name: &str, _: &fs::Metadata) -> bool {
    name.starts_with('.')
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A folder for one test, removed when it is over.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Scratch {
            let dir = std::env::temp_dir()
                .join(format!("packwing-storage-{}-{name}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).expect("cannot make a folder");
            Scratch(dir)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn all(s: &dyn Storage, folder: &[String]) -> Vec<Item> {
        let mut lister = s.list(folder).expect("list");
        let mut items = Vec::new();
        while !lister.done() {
            items.extend(lister.next(2).expect("page"));
        }
        items
    }

    #[test]
    fn names_sort_as_people_read_them() {
        let mut names = ["file10", "File2", "ảnh", "anh 2", "a", "file2"];
        names.sort_by(|a, b| by_name(a, b));
        assert_eq!(names, ["a", "ảnh", "anh 2", "File2", "file2", "file10"]);
        assert_eq!(by_name("007", "7"), Ordering::Less);
    }

    #[test]
    fn folders_come_first_a_page_at_a_time() {
        let t = Scratch::new("pages");
        for name in ["b.txt", "a10.txt", "a2.txt", ".hidden"] {
            fs::write(t.0.join(name), name).expect("write");
        }
        fs::create_dir(t.0.join("z-folder")).expect("mkdir");
        let s = LocalFolder::new(&t.0);
        let mut lister = s.list(&[]).expect("list");
        assert!(!lister.done());
        let first: Vec<String> = lister
            .next(3)
            .expect("page")
            .into_iter()
            .map(|i| i.name)
            .collect();
        assert_eq!(first, ["z-folder", ".hidden", "a2.txt"]);
        assert!(!lister.done());
        let rest = lister.next(10).expect("page");
        assert_eq!(rest.len(), 2);
        assert!(lister.done());
        assert!(lister.next(10).expect("page").is_empty());

        let items = all(&s, &[]);
        assert!(items[0].dir);
        assert!(items[1].hidden);
        assert_eq!(items[2].size, "a2.txt".len() as u64);
        assert!(items[2].modified.is_some());
    }

    #[test]
    fn an_empty_folder_is_done_after_its_first_page() {
        let t = Scratch::new("empty");
        let s = LocalFolder::new(&t.0);
        let mut lister = s.list(&[]).expect("list");
        assert!(lister.next(10).expect("page").is_empty());
        assert!(lister.done());
    }

    #[test]
    fn a_name_cannot_lead_out_of_the_storage() {
        let t = Scratch::new("escape");
        let s = LocalFolder::new(&t.0);
        for bad in ["..", "a/b", ".", "", "  "] {
            assert!(bad_name(bad).is_some(), "{bad:?}");
            assert!(s.list(&[bad.to_owned()]).is_err(), "{bad:?}");
            assert!(s.make_folder(&[], bad).is_err(), "{bad:?}");
        }
        assert!(s.read(&["..".into(), "secret".into()], 10).is_err());
        assert!(bad_name("báo cáo 2026.zip").is_none());
    }

    #[test]
    fn renaming_keeps_what_is_there() {
        let t = Scratch::new("rename");
        fs::write(t.0.join("a"), "a").expect("write");
        fs::write(t.0.join("b"), "b").expect("write");
        let s = LocalFolder::new(&t.0);
        let taken = s.rename(&[], "a", "b").expect_err("the name is taken");
        assert_eq!(taken.kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(fs::read(t.0.join("b")).expect("read"), b"b");
        s.rename(&[], "a", "c").expect("rename");
        assert_eq!(fs::read(t.0.join("c")).expect("read"), b"a");
        s.rename(&[], "c", "C").expect("a change of case");
        assert_eq!(fs::read(t.0.join("C")).expect("read"), b"a");
        assert!(s.make_folder(&[], "b").is_err());
        s.make_folder(&[], "new").expect("mkdir");
        assert!(t.0.join("new").is_dir());
    }

    /// Puts a file in this user's Trash, so it is run by hand:
    /// `cargo test -p packwing-core goes_to_the_trash -- --ignored`.
    #[test]
    #[ignore]
    fn deleting_goes_to_the_trash() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/trash-test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("mkdir");
        fs::write(dir.join("packwing-trash-test.txt"), "x").expect("write");
        let s = LocalFolder::new(&dir);
        assert!(s.has_trash());
        s.delete(&[], &["packwing-trash-test.txt".into()])
            .expect("delete");
        assert!(!dir.join("packwing-trash-test.txt").exists());
    }

    #[test]
    fn reading_stops_where_it_is_told() {
        let t = Scratch::new("read");
        fs::write(t.0.join("a"), "abcdef").expect("write");
        let s = LocalFolder::new(&t.0);
        assert_eq!(s.read(&["a".into()], 4).expect("read"), b"abcd");
    }

    #[cfg(unix)]
    #[test]
    fn a_count_does_not_follow_links() {
        let t = Scratch::new("count");
        fs::create_dir_all(t.0.join("d/e")).expect("mkdir");
        fs::write(t.0.join("d/one"), "12345").expect("write");
        fs::write(t.0.join("d/e/two"), "678").expect("write");
        std::os::unix::fs::symlink(&t.0, t.0.join("d/up")).expect("link");
        let s = LocalFolder::new(&t.0);
        let (counted, unread) = count(&s, &["d".into()], &Cancel::new(), |_| {}).expect("count");
        assert_eq!(counted.folders, 1);
        assert_eq!(counted.files, 3);
        assert!(counted.bytes >= 8);
        assert_eq!(unread, 0);
        let items = all(&s, &["d".into()]);
        let link = items.iter().find(|i| i.name == "up").expect("the link");
        assert!(link.dir && link.link);
    }

    #[test]
    fn a_count_stops_when_cancelled() {
        let t = Scratch::new("cancel");
        fs::write(t.0.join("a"), "a").expect("write");
        let s = LocalFolder::new(&t.0);
        let cancel = Cancel::new();
        cancel.cancel();
        let e = count(&s, &[], &cancel, |_| {}).expect_err("cancelled");
        assert_eq!(e.kind(), io::ErrorKind::Interrupted);
    }
}
