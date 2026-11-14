//! An archive extracted into something that takes its entries, in place of a
//! folder: what a phone's document tree needs, which cannot be written to by
//! path. See crates/core/src/destination.rs.

#![allow(clippy::unwrap_used)]

use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use packwing_core::{
    Control, Destination, ExtractOptions, Keys, NewArchive, create, extract_to, list,
};

/// A folder in the system's temporary one, removed when dropped.
struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Scratch {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "packwing-destination-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).unwrap();
        Scratch(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// What it was given, to look at afterwards.
#[derive(Default)]
struct Recorded {
    folders: Vec<String>,
    files: Vec<(String, Vec<u8>)>,
    open: Option<(String, Vec<u8>)>,
}

impl Destination for Recorded {
    fn directory(&mut self, path: &[String]) -> io::Result<()> {
        self.folders.push(path.join("/"));
        Ok(())
    }

    fn start_file(&mut self, path: &[String], _size: Option<u64>) -> io::Result<()> {
        assert!(self.open.is_none(), "a file was started inside another");
        self.open = Some((path.join("/"), Vec::new()));
        Ok(())
    }

    fn write(&mut self, data: &[u8]) -> io::Result<()> {
        let (_, held) = self.open.as_mut().expect("data with no file started");
        held.extend_from_slice(data);
        Ok(())
    }

    fn finish_file(&mut self) -> io::Result<()> {
        self.files.push(self.open.take().expect("no file to finish"));
        Ok(())
    }

    fn discard_file(&mut self) {
        self.open = None;
    }
}

impl Recorded {
    /// Its files by name, in order, with what each holds.
    fn named(&self) -> Vec<(&str, &[u8])> {
        let mut named: Vec<(&str, &[u8])> = self
            .files
            .iter()
            .map(|(name, data)| (name.as_str(), data.as_slice()))
            .collect();
        named.sort_by_key(|(name, _)| *name);
        named
    }
}

/// A ZIP of three files in two folders, and what it holds.
fn an_archive(s: &Scratch) -> PathBuf {
    let from = s.0.join("from");
    std::fs::create_dir_all(from.join("docs/plans")).unwrap();
    std::fs::write(from.join("docs/notes.txt"), b"notes").unwrap();
    std::fs::write(from.join("docs/plans/a.txt"), b"aaaa").unwrap();
    std::fs::write(from.join("readme.md"), b"# Read me\n").unwrap();
    create(
        &[from.join("docs"), from.join("readme.md")],
        &s.0.join("a.zip"),
        &NewArchive::default(),
        &Control::default(),
    )
    .unwrap()
}

#[test]
fn every_entry_goes_to_the_destination_with_its_path() {
    let s = Scratch::new();
    let archive = an_archive(&s);
    let mut taken = Recorded::default();
    let summary = extract_to(
        &archive,
        &mut taken,
        &[],
        &Keys::default(),
        &ExtractOptions::default(),
        &Control::default(),
    )
    .unwrap();

    assert_eq!(summary.files, 3);
    assert_eq!(summary.bytes, 19);
    assert_eq!(
        taken.named(),
        [
            ("docs/notes.txt", b"notes".as_slice()),
            ("docs/plans/a.txt", b"aaaa".as_slice()),
            ("readme.md", b"# Read me\n".as_slice()),
        ]
    );
    // The folders the archive holds entries for, which is both of them.
    let mut folders = taken.folders.clone();
    folders.sort();
    assert_eq!(folders, ["docs", "docs/plans"]);
    // What the archive says it holds is what came out.
    let listed = list(&archive, &Keys::default()).unwrap();
    assert_eq!(listed.files, summary.files);
}

#[test]
fn only_what_is_picked_goes_and_it_keeps_its_folders() {
    let s = Scratch::new();
    let archive = an_archive(&s);
    let mut taken = Recorded::default();
    let picked = vec![vec!["docs".to_owned(), "plans".to_owned()]];
    let summary = extract_to(
        &archive,
        &mut taken,
        &picked,
        &Keys::default(),
        &ExtractOptions::default(),
        &Control::default(),
    )
    .unwrap();

    assert_eq!(summary.files, 1);
    // Its path in the archive, not the name alone: unlike extract_selected,
    // which takes each picked entry out of the folders it is in.
    assert_eq!(taken.named(), [("docs/plans/a.txt", b"aaaa".as_slice())]);
}

/// What will not take anything, as a folder that is full would not.
struct Refuses;

impl Destination for Refuses {
    fn directory(&mut self, _path: &[String]) -> io::Result<()> {
        Ok(())
    }

    fn start_file(&mut self, _path: &[String], _size: Option<u64>) -> io::Result<()> {
        Err(io::Error::other("there is no room"))
    }

    fn write(&mut self, _data: &[u8]) -> io::Result<()> {
        unreachable!("no file was ever started")
    }

    fn finish_file(&mut self) -> io::Result<()> {
        unreachable!("no file was ever started")
    }

    fn discard_file(&mut self) {}
}

#[test]
fn a_destination_that_will_not_take_it_stops_the_job() {
    let s = Scratch::new();
    let archive = an_archive(&s);
    let failed = extract_to(
        &archive,
        &mut Refuses,
        &[],
        &Keys::default(),
        &ExtractOptions::default(),
        &Control::default(),
    );
    let said = failed.unwrap_err().to_string();
    assert!(said.contains("there is no room"), "{said}");
}
