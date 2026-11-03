//! Marks, on the systems that have them: what arrives through packwing stays checked.
#![cfg(any(windows, target_os = "macos"))]

use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use hivepack::extract::Origin;
use packwing_core::{
    Control, Item, Keys, Kind, Lister, LocalFolder, NewArchive, OnExisting, Source, Storage,
    Stored, Transfer, create, extract, transfer,
};

/// A folder that does not say where it is, as one across a network.
struct Far(LocalFolder);

impl Storage for Far {
    fn list(&self, f: &[String]) -> io::Result<Box<dyn Lister>> {
        self.0.list(f)
    }
    fn read(&self, p: &[String], most: u64) -> io::Result<Vec<u8>> {
        self.0.read(p, most)
    }
    fn read_at(&self, p: &[String], offset: u64, len: usize) -> io::Result<Vec<u8>> {
        self.0.read_at(p, offset, len)
    }
    fn stat(&self, f: &[String], n: &str) -> io::Result<Option<Item>> {
        self.0.stat(f, n)
    }
    fn open(&self, p: &[String]) -> io::Result<Box<dyn Read + Send>> {
        self.0.open(p)
    }
    fn put(
        &self,
        p: &[String],
        from: &mut dyn Read,
        like: &Item,
        replace: bool,
    ) -> io::Result<u64> {
        self.0.put(p, from, like, replace)
    }
    fn remove(&self, p: &[String]) -> io::Result<()> {
        self.0.remove(p)
    }
    fn make_folder(&self, f: &[String], n: &str) -> io::Result<()> {
        self.0.make_folder(f, n)
    }
    fn rename(&self, f: &[String], a: &str, b: &str) -> io::Result<()> {
        self.0.rename(f, a, b)
    }
    fn delete(&self, f: &[String], n: &[String]) -> io::Result<()> {
        self.0.delete(f, n)
    }
    fn has_trash(&self) -> bool {
        false
    }
    fn local_path(&self, _: &[String]) -> Option<PathBuf> {
        None
    }
    fn show_path(&self, p: &[String]) -> String {
        format!("far://{}", p.join("/"))
    }
}

fn folder(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("packwing-mark-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("a folder");
    fs::write(dir.join("file.txt"), "held").expect("a file");
    dir
}

fn marked(path: &Path) -> bool {
    Origin::of(path).expect("its mark").is_some()
}

/// Copies `file.txt` from `from` to the folder `to`, made here.
fn copy(from: Arc<dyn Storage>, to: &Path, keep_mark: bool) -> PathBuf {
    fs::create_dir_all(to).expect("a folder");
    let t = Transfer {
        from,
        from_folder: Vec::new(),
        names: vec!["file.txt".into()],
        to: Arc::new(LocalFolder::new(to)),
        to_folder: Vec::new(),
        moving: false,
        on_existing: OnExisting::Stop,
        keep_mark,
    };
    transfer(&t, &Control::default()).expect("the copy");
    to.join("file.txt")
}

#[test]
fn a_copy_from_a_storage_is_marked_unless_the_user_chose_not() {
    let dir = folder("copy");
    let far = || Arc::new(Far(LocalFolder::new(&dir))) as Arc<dyn Storage>;
    assert!(marked(&copy(far(), &dir.join("yes"), true)));
    assert!(!marked(&copy(far(), &dir.join("no"), false)));
}

#[test]
fn a_mark_survives_a_copy_between_folders_here() {
    let dir = folder("here");
    let far: Arc<dyn Storage> = Arc::new(Far(LocalFolder::new(&dir)));
    let first = copy(far, &dir.join("first"), true);
    let here = Arc::new(LocalFolder::new(first.parent().expect("a folder")));
    assert!(marked(&copy(here, &dir.join("second"), true)));
}

#[test]
fn a_copy_of_what_has_no_mark_gets_none() {
    let dir = folder("clean");
    let here = Arc::new(LocalFolder::new(&dir));
    assert!(!marked(&copy(here, &dir.join("out"), true)));
}

#[test]
fn what_is_extracted_from_a_storage_is_marked() {
    let dir = folder("extract");
    let new = NewArchive {
        kind: Kind::Zip,
        ..NewArchive::default()
    };
    let made = create(
        &[dir.join("file.txt")],
        &dir.join("a.zip"),
        &new,
        &Control::default(),
    )
    .expect("the archive");
    let name = made.file_name().expect("a name").to_string_lossy();
    let size = fs::metadata(&made).expect("the archive").len();
    let far: Arc<dyn Storage> = Arc::new(Far(LocalFolder::new(&dir)));
    let source = Source::Stored(Stored::new(far, vec![name.into_owned()], size));
    assert!(packwing_core::came_from_internet(&source));
    let out = dir.join("out");
    extract(source, &out, &Keys::default(), &Control::default()).expect("the extraction");
    assert!(marked(&out.join("file.txt")));
}
