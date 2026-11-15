//! Self-extracting archives: a program with an archive after it, which
//! packwing reads as the archive. See crates/core/src/sfx.rs.

#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use packwing_core::{
    Container, Control, Keys, Kind, NewArchive, can_add, create, extract, list, test,
};

/// A folder in the system's temporary one, removed when dropped.
struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Scratch {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "packwing-sfx-{}-{}",
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

/// What every one of these holds: a folder, a file in it, and one beside it.
fn files(s: &Scratch) -> Vec<PathBuf> {
    let from = s.0.join("from");
    std::fs::create_dir_all(from.join("docs")).unwrap();
    std::fs::write(from.join("docs/notes.txt"), b"notes").unwrap();
    std::fs::write(from.join("readme.md"), b"# Read me\n").unwrap();
    vec![from.join("docs"), from.join("readme.md")]
}

/// A program's first bytes and nothing it could run: what matters here is
/// that an archive comes after them, as it comes after the real thing.
fn program() -> Vec<u8> {
    let mut program = b"MZ\x90\x00".to_vec();
    // Something for the archive's own first bytes not to be at the start, and
    // long enough for what is looked for to be looked for over more than one
    // read.
    program.extend(std::iter::repeat_n(b'\0', 200_000));
    program
}

/// `kind` made of [`files`], with a program before it: what a self-extracting
/// archive is.
fn after_a_program(s: &Scratch, kind: Kind, name: &str) -> PathBuf {
    let archive = create(
        &files(s),
        &s.0.join(name),
        &NewArchive {
            kind,
            ..NewArchive::default()
        },
        &Control::default(),
    )
    .unwrap();
    let mut whole = program();
    whole.extend(std::fs::read(&archive).unwrap());
    let sfx = s.0.join(format!("{name}.exe"));
    std::fs::write(&sfx, whole).unwrap();
    std::fs::remove_file(&archive).unwrap();
    sfx
}

/// What it holds, by path, as the listing names them.
fn held(path: &Path) -> Vec<String> {
    let mut names = list(path, &Keys::default())
        .unwrap()
        .entries
        .iter()
        .filter(|e| !e.dir)
        .map(|e| e.path.join("/"))
        .collect::<Vec<_>>();
    names.sort();
    names
}

#[test]
fn a_zip_after_a_program_is_read_as_the_zip_it_is() {
    let s = Scratch::new();
    let sfx = after_a_program(&s, Kind::Zip, "a.zip");
    let listing = list(&sfx, &Keys::default()).unwrap();
    assert_eq!(listing.container, Container::Zip);
    assert!(listing.at > 0, "the archive begins after the program");
    assert_eq!(held(&sfx), ["docs/notes.txt", "readme.md"]);
}

#[test]
fn a_7z_after_a_program_is_read_as_the_7z_it_is() {
    let s = Scratch::new();
    let sfx = after_a_program(&s, Kind::SevenZip, "a.7z");
    let listing = list(&sfx, &Keys::default()).unwrap();
    assert_eq!(listing.container, Container::SevenZip);
    assert!(listing.at > 0, "the archive begins after the program");
    assert_eq!(held(&sfx), ["docs/notes.txt", "readme.md"]);
}

/// A RAR archive after a program needs no offset of packwing's:
/// `hivepack::rar` looks for the archive after the program itself, as UnRAR
/// does, so what packwing has to do is try it at all.
#[test]
fn a_rar_after_a_program_is_read_as_the_rar_it_is() {
    let s = Scratch::new();
    let mut whole = program();
    whole.extend(
        std::fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/rar5_multiple_files.rar"),
        )
        .unwrap(),
    );
    let sfx = s.0.join("setup.exe");
    std::fs::write(&sfx, whole).unwrap();
    let listing = list(&sfx, &Keys::default()).unwrap();
    assert_eq!(listing.container, Container::Rar);
    assert_eq!(listing.at, 0, "hivepack::rar finds it after the program");
    assert!(listing.files > 1, "{}", listing.files);
    let into = s.0.join("out");
    let summary = extract(&sfx, &into, &Keys::default(), &Control::default()).unwrap();
    assert_eq!(summary.files, listing.files);
    assert_eq!(
        test(&sfx, &Keys::default(), &Control::default())
            .unwrap()
            .files,
        listing.files
    );
}

#[test]
fn what_is_after_a_program_comes_out_as_it_went_in() {
    for kind in [Kind::Zip, Kind::SevenZip] {
        let s = Scratch::new();
        let sfx = after_a_program(&s, kind, "a");
        let into = s.0.join("out");
        let summary = extract(&sfx, &into, &Keys::default(), &Control::default()).unwrap();
        assert_eq!(summary.files, 2, "{kind:?}");
        assert_eq!(
            std::fs::read(into.join("docs/notes.txt")).unwrap(),
            b"notes",
            "{kind:?}"
        );
        assert_eq!(
            std::fs::read(into.join("readme.md")).unwrap(),
            b"# Read me\n",
            "{kind:?}"
        );
    }
}

#[test]
fn one_is_checked_through_as_any_other_archive_is() {
    for kind in [Kind::Zip, Kind::SevenZip] {
        let s = Scratch::new();
        let sfx = after_a_program(&s, kind, "a");
        let summary = test(&sfx, &Keys::default(), &Control::default()).unwrap();
        assert_eq!(summary.files, 2, "{kind:?}");
    }
}

#[test]
fn nothing_is_added_to_one_or_deleted_from_it() {
    let s = Scratch::new();
    let sfx = after_a_program(&s, Kind::Zip, "a.zip");
    let listing = list(&sfx, &Keys::default()).unwrap();
    assert!(
        !can_add(&listing),
        "what holds the archive would have to be written again, and it is a program"
    );
    // The same archive, with nothing before it, is one files go into.
    let plain = create(
        &files(&s),
        &s.0.join("plain.zip"),
        &NewArchive::default(),
        &Control::default(),
    )
    .unwrap();
    assert!(can_add(&list(&plain, &Keys::default()).unwrap()));
}

#[test]
fn a_program_with_no_archive_after_it_is_not_an_archive() {
    let s = Scratch::new();
    let alone = s.0.join("setup.exe");
    std::fs::write(&alone, program()).unwrap();
    assert!(list(&alone, &Keys::default()).is_err());
}

#[test]
fn a_file_that_is_not_a_program_is_not_looked_through() {
    let s = Scratch::new();
    // A 7z archive's first bytes, in a file that is not a program: looking
    // through every file that is not an archive would cost every open one
    // pass over the file, and this is why it is not done.
    let archive = create(
        &files(&s),
        &s.0.join("a.7z"),
        &NewArchive {
            kind: Kind::SevenZip,
            ..NewArchive::default()
        },
        &Control::default(),
    )
    .unwrap();
    let mut whole = vec![b'\0'; 4096];
    whole.extend(std::fs::read(&archive).unwrap());
    let data = s.0.join("something.bin");
    std::fs::write(&data, whole).unwrap();
    assert!(list(&data, &Keys::default()).is_err());
}
