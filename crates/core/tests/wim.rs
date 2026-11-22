//! Windows images: the images one WIM archive holds, each a folder, and the
//! tree of names in each.
//!
//! The two committed archives were written by `wimlib-imagex`
//! (`tests/data/README.md` says with what), so these tests need no tool and
//! run everywhere: one of **two** images, whose first holds a file under two
//! names (a hard link), and one of a single image, which is the tree itself.
//! packwing never writes a WIM.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use packwing_core::{
    Container, Control, Entry, ExtractOptions, Keys, extract, extract_selected, list, test,
};

/// A folder in the system's temporary one, removed when dropped.
struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Scratch {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "packwing-wim-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).unwrap();
        Scratch(path)
    }

    fn file(&self, name: &str, data: &[u8]) -> PathBuf {
        let path = self.0.join(name);
        std::fs::write(&path, data).unwrap();
        path
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn data(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data")
        .join(name)
}

/// Every path of a listing, with `/` after a folder.
fn paths(entries: &[Entry]) -> Vec<String> {
    let mut found: Vec<String> = entries
        .iter()
        .map(|entry| {
            let path = entry.path.join("/");
            if entry.dir { format!("{path}/") } else { path }
        })
        .collect();
    found.sort();
    found
}

/// The paths under a folder, with `/` after a folder, sorted.
fn written(at: &Path) -> Vec<String> {
    let mut found = Vec::new();
    let mut stack = vec![at.to_path_buf()];
    while let Some(next) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&next) else {
            continue;
        };
        for entry in entries {
            let path = entry.unwrap().path();
            let shown = path
                .strip_prefix(at)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            if path.is_dir() {
                found.push(format!("{shown}/"));
                stack.push(path);
            } else {
                found.push(shown);
            }
        }
    }
    found.sort();
    found
}

/// An archive of several images: each one a folder, named by its number and
/// the name the archive's XML gives it, with its tree under it.
#[test]
fn each_image_is_a_folder_named_by_its_number() {
    let listing = list(data("two-images.wim"), &Keys::default()).unwrap();
    assert_eq!(listing.container, Container::Wim);
    assert_eq!(
        paths(&listing.entries),
        [
            "1-Windows 11 Pro/",
            "1-Windows 11 Pro/Windows/",
            "1-Windows 11 Pro/Windows/System32/",
            "1-Windows 11 Pro/Windows/System32/kernel32.dll",
            "1-Windows 11 Pro/Windows/explorer.exe",
            "1-Windows 11 Pro/link.txt",
            "1-Windows 11 Pro/notes.txt",
            "2-Windows 11 Home/",
            "2-Windows 11 Home/Users/",
            "2-Windows 11 Home/Users/readme.md",
            "2-Windows 11 Home/notes.txt",
        ]
    );
    // A file under two names is counted once: its bytes are at the first
    // name the tree gives, and the other says 0.
    let of = |path: &str| {
        listing
            .entries
            .iter()
            .find(|e| e.path.join("/") == path)
            .unwrap()
    };
    assert_eq!(of("1-Windows 11 Pro/link.txt").size, 27);
    assert_eq!(of("1-Windows 11 Pro/notes.txt").size, 0);
    // Both names are files, as a disk image's hard links are: six names of
    // five files, whose bytes are counted once.
    assert_eq!(listing.files, 6, "{listing:?}");
    assert_eq!(listing.unpacked, 27 + 4098 + 21 + 28 + 23);
}

/// An archive of one image is that image's tree, with no folder in front of
/// it: a folder there would say the archive holds a level it does not.
#[test]
fn one_image_is_the_tree_itself() {
    let listing = list(data("one-image.wim"), &Keys::default()).unwrap();
    assert_eq!(listing.container, Container::Wim);
    assert_eq!(
        paths(&listing.entries),
        ["docs/", "docs/note.txt", "root.txt"]
    );
}

/// Extracted, each image comes out under its folder, and the names one
/// image says share a file are hard links to one file on the disk.
#[test]
fn an_image_comes_out_under_its_folder_with_its_hard_links() {
    let s = Scratch::new();
    let out = s.0.join("out");
    let summary = extract(
        data("two-images.wim"),
        &out,
        &Keys::default(),
        &Control::default(),
    )
    .unwrap();
    assert_eq!(
        written(&out),
        [
            "1-Windows 11 Pro/",
            "1-Windows 11 Pro/Windows/",
            "1-Windows 11 Pro/Windows/System32/",
            "1-Windows 11 Pro/Windows/System32/kernel32.dll",
            "1-Windows 11 Pro/Windows/explorer.exe",
            "1-Windows 11 Pro/link.txt",
            "1-Windows 11 Pro/notes.txt",
            "2-Windows 11 Home/",
            "2-Windows 11 Home/Users/",
            "2-Windows 11 Home/Users/readme.md",
            "2-Windows 11 Home/notes.txt",
        ]
    );
    // Each image's own tree, not the other's.
    assert_eq!(
        std::fs::read(out.join("1-Windows 11 Pro/notes.txt")).unwrap(),
        b"The first image, its note.\n"
    );
    assert_eq!(
        std::fs::read(out.join("2-Windows 11 Home/notes.txt")).unwrap(),
        b"The second image, its note.\n"
    );
    assert_eq!(summary.files, 6, "{summary:?}");

    // One file, two names: written once and linked, as the image says.
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let note = std::fs::metadata(out.join("1-Windows 11 Pro/notes.txt")).unwrap();
        let link = std::fs::metadata(out.join("1-Windows 11 Pro/link.txt")).unwrap();
        assert_eq!(note.ino(), link.ino());
        assert_eq!(note.nlink(), 2);
    }
}

/// Picked inside one image, only that image is read and only what was
/// picked comes out: the folder an image is stands in the path, as the
/// listing shows it.
#[test]
fn only_what_is_picked_of_an_image_comes_out() {
    let s = Scratch::new();
    let out = s.0.join("out");
    let picked = vec![vec!["2-Windows 11 Home".to_owned(), "Users".to_owned()]];
    let summary = extract_selected(
        data("two-images.wim"),
        &out,
        &picked,
        &Keys::default(),
        &ExtractOptions::default(),
        &Control::default(),
    )
    .unwrap();
    assert_eq!(written(&out), ["Users/", "Users/readme.md"]);
    assert_eq!(summary.files, 1, "{summary:?}");
    assert_eq!(summary.bytes, 23);
}

/// Tested, every file of every image is read through and held to the SHA-1
/// the image names it by. A file under two names is read once.
#[test]
fn every_file_of_every_image_is_read_when_tested() {
    let summary = test(
        data("two-images.wim"),
        &Keys::default(),
        &Control::default(),
    )
    .unwrap();
    assert_eq!(summary.files, 5, "{summary:?}");
    assert_eq!(summary.bytes, 27 + 4098 + 21 + 28 + 23);
}

/// A file whose bytes were changed does not read: the SHA-1 the image
/// names it by is the only check a WIM carries, and it is made as the file
/// is read.
#[test]
fn a_file_whose_bytes_were_changed_does_not_read() {
    let s = Scratch::new();
    let mut bytes = std::fs::read(data("two-images.wim")).unwrap();
    // 500 bytes in is the compressed data of the image's largest file,
    // `kernel32.dll`, whose resource is the first on the file: changed
    // there it still decompresses, and what comes out does not hash to
    // what the image names the file by.
    bytes[500] ^= 0xff;
    let path = s.file("changed.wim", &bytes);
    let out = s.0.join("out");
    let said = extract(&path, &out, &Keys::default(), &Control::default())
        .unwrap_err()
        .to_string();
    assert!(said.contains("SHA-1"), "{said}");
    // Nothing of that file is left behind: a write that failed is removed.
    assert!(
        !out.join("1-Windows 11 Pro/Windows/System32/kernel32.dll")
            .exists(),
        "{:?}",
        written(&out)
    );
}

/// One part of a split image is not a whole archive, and says so rather
/// than reading the names in it: the rest of the data is in the other
/// parts, which packwing does not join.
#[test]
fn a_part_of_a_split_image_says_what_it_needs() {
    let s = Scratch::new();
    let mut bytes = std::fs::read(data("two-images.wim")).unwrap();
    // The header's count of parts, as `wimlib-imagex split` writes it.
    bytes[42..44].copy_from_slice(&2u16.to_le_bytes());
    let path = s.file("part.swm", &bytes);
    let said = list(&path, &Keys::default()).unwrap_err().to_string();
    assert!(said.contains("split"), "{said}");
}
