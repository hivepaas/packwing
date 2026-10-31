//! Archives packwing-core makes, read back through packwing-core and, where
//! it is installed, by Info-ZIP's `unzip`.

#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use packwing_core::{
    Cancel, Control, Error, Keys, NewArchive, Progress, create_zip, extract, list, measure, test,
};

/// A folder of a test's own, gone when it ends.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Scratch {
        let dir =
            std::env::temp_dir().join(format!("packwing-create-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        Scratch(dir)
    }

    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }

    /// `Báo cáo/`: two files, one of them over a megabyte, a file in a
    /// folder, an empty file and an empty folder.
    fn tree(&self) -> PathBuf {
        let root = self.path("Báo cáo");
        fs::create_dir_all(root.join("Hình ảnh")).unwrap();
        fs::create_dir_all(root.join("trống")).unwrap();
        fs::write(root.join("ghi chú.txt"), "xin chào\n").unwrap();
        fs::write(root.join("rỗng"), "").unwrap();
        fs::write(root.join("Hình ảnh/lớn.bin"), big()).unwrap();
        root
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// 3 MB that does not compress much, and is the same every time.
fn big() -> Vec<u8> {
    let mut state = 0x2545_f491_u32;
    (0..3_000_000)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            state as u8
        })
        .collect()
}

fn same_tree(out: &Path) {
    assert_eq!(
        fs::read_to_string(out.join("ghi chú.txt")).unwrap(),
        "xin chào\n"
    );
    assert_eq!(fs::read(out.join("rỗng")).unwrap(), b"");
    assert_eq!(fs::read(out.join("Hình ảnh/lớn.bin")).unwrap(), big());
    assert!(out.join("trống").is_dir());
}

#[test]
fn a_folder_comes_back_as_it_went_in() {
    let s = Scratch::new("back");
    let root = s.tree();
    let made = create_zip(
        std::slice::from_ref(&root),
        &s.path("a.zip"),
        &NewArchive::default(),
        &Control::default(),
    )
    .unwrap();
    assert_eq!(made, s.path("a.zip"));
    let listing = list(&made, &Keys::default()).unwrap();
    assert_eq!(listing.files, 3);
    test(&made, &Keys::default(), &Control::default()).unwrap();
    extract(&made, &s.path("out"), &Keys::default(), &Control::default()).unwrap();
    same_tree(&s.path("out/Báo cáo"));
}

#[test]
fn unzip_reads_it_with_its_names() {
    let s = Scratch::new("unzip");
    let root = s.tree();
    let made = create_zip(
        &[root],
        &s.path("a.zip"),
        &NewArchive::default(),
        &Control::default(),
    )
    .unwrap();
    let Ok(out) = Command::new("unzip").arg("-t").arg(&made).output() else {
        return; // not installed
    };
    assert!(out.status.success(), "{out:?}");
    let said = String::from_utf8_lossy(&out.stdout);
    assert!(said.contains("Báo cáo/Hình ảnh/lớn.bin"), "{said}");
}

#[test]
fn nothing_is_written_over_what_is_there() {
    let s = Scratch::new("free");
    let root = s.tree();
    fs::write(s.path("a.zip"), "mine").unwrap();
    let made = create_zip(
        &[root],
        &s.path("a.zip"),
        &NewArchive::default(),
        &Control::default(),
    )
    .unwrap();
    assert_eq!(made, s.path("a (2).zip"));
    assert_eq!(fs::read(s.path("a.zip")).unwrap(), b"mine");
}

#[test]
fn several_paths_sit_side_by_side() {
    let s = Scratch::new("several");
    let (a, b) = (s.path("a.txt"), s.path("b.txt"));
    fs::write(&a, "a").unwrap();
    fs::write(&b, "b").unwrap();
    let made = create_zip(
        &[a, b],
        &s.path("x.zip"),
        &NewArchive::default(),
        &Control::default(),
    )
    .unwrap();
    extract(&made, &s.path("out"), &Keys::default(), &Control::default()).unwrap();
    assert_eq!(fs::read(s.path("out/a.txt")).unwrap(), b"a");
    assert_eq!(fs::read(s.path("out/b.txt")).unwrap(), b"b");
}

#[test]
fn a_password_locks_it() {
    let s = Scratch::new("password");
    let root = s.tree();
    let new = NewArchive {
        password: Some(b"mat khau".to_vec()),
        ..NewArchive::default()
    };
    let made = create_zip(&[root], &s.path("a.zip"), &new, &Control::default()).unwrap();
    assert!(matches!(
        extract(&made, &s.path("no"), &Keys::default(), &Control::default()),
        Err(Error::Locked { .. })
    ));
    let wrong = Keys {
        password: Some(b"other".to_vec()),
        ..Keys::default()
    };
    assert!(matches!(
        test(&made, &wrong, &Control::default()),
        Err(Error::WrongPassword)
    ));
    let right = Keys {
        password: new.password.clone(),
        ..Keys::default()
    };
    extract(&made, &s.path("out"), &right, &Control::default()).unwrap();
    same_tree(&s.path("out/Báo cáo"));
}

#[test]
fn parts_join_into_the_archive() {
    let s = Scratch::new("parts");
    let root = s.tree();
    let new = NewArchive {
        part_size: Some(1_000_000),
        ..NewArchive::default()
    };
    let made = create_zip(&[root], &s.path("a.zip"), &new, &Control::default()).unwrap();
    assert_eq!(made, s.path("a.zip.001"));
    let mut joined = Vec::new();
    for (i, part) in ["001", "002", "003", "004"].iter().enumerate() {
        let data = fs::read(s.path(&format!("a.zip.{part}"))).unwrap();
        assert!(
            i == 3 || data.len() == 1_000_000,
            "part {part}: {}",
            data.len()
        );
        joined.extend(data);
    }
    assert!(!s.path("a.zip.005").exists());
    fs::write(s.path("joined.zip"), joined).unwrap();
    extract(
        &s.path("joined.zip"),
        &s.path("out"),
        &Keys::default(),
        &Control::default(),
    )
    .unwrap();
    same_tree(&s.path("out/Báo cáo"));
}

#[test]
fn an_archive_that_fits_one_part_is_just_an_archive() {
    let s = Scratch::new("one-part");
    let file = s.path("a.txt");
    fs::write(&file, "a").unwrap();
    let new = NewArchive {
        part_size: Some(1_000_000),
        ..NewArchive::default()
    };
    let made = create_zip(&[file], &s.path("x.zip"), &new, &Control::default()).unwrap();
    assert_eq!(made, s.path("x.zip"));
    assert!(!s.path("x.zip.001").exists());
    test(&made, &Keys::default(), &Control::default()).unwrap();
}

#[test]
fn stopping_leaves_nothing() {
    let s = Scratch::new("stop");
    let root = s.tree();
    for part_size in [None, Some(1_000_000)] {
        let cancel = Cancel::new();
        let stop = cancel.clone();
        let control = Control {
            // Once it has read something.
            progress: Some(Arc::new(move |p: &Progress| {
                if p.bytes > 0 {
                    stop.cancel();
                }
            })),
            cancel: Some(cancel),
        };
        let new = NewArchive {
            part_size,
            ..NewArchive::default()
        };
        let result = create_zip(
            std::slice::from_ref(&root),
            &s.path("a.zip"),
            &new,
            &control,
        );
        assert!(matches!(result, Err(Error::Cancelled)), "{result:?}");
        assert!(!s.path("a.zip").exists());
        assert!(!s.path("a.zip.001").exists());
    }
}

#[test]
fn progress_reaches_every_byte_once() {
    let s = Scratch::new("progress");
    let root = s.tree();
    let measured = measure(std::slice::from_ref(&root)).unwrap();
    assert_eq!(measured.files, 3);
    let (bytes, total) = (Arc::new(AtomicU64::new(0)), Arc::new(AtomicU64::new(0)));
    let (b, t) = (bytes.clone(), total.clone());
    let control = Control {
        progress: Some(Arc::new(move |p: &Progress| {
            b.store(p.bytes, Ordering::Relaxed);
            t.store(p.total_bytes, Ordering::Relaxed);
        })),
        cancel: None,
    };
    create_zip(&[root], &s.path("a.zip"), &NewArchive::default(), &control).unwrap();
    assert_eq!(total.load(Ordering::Relaxed), measured.bytes);
    assert_eq!(bytes.load(Ordering::Relaxed), measured.bytes);
}

#[test]
fn a_file_that_cannot_be_read_is_named_and_leaves_nothing() {
    let s = Scratch::new("missing");
    let err = create_zip(
        &[s.path("not there")],
        &s.path("a.zip"),
        &NewArchive::default(),
        &Control::default(),
    )
    .unwrap_err();
    assert!(err.to_string().contains("not there"), "{err}");
    assert!(!s.path("a.zip").exists());
}

#[cfg(unix)]
#[test]
fn links_stay_links_and_pipes_are_left_out() {
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::symlink;
    let s = Scratch::new("links");
    let root = s.path("d");
    fs::create_dir_all(&root).unwrap();
    fs::write(s.path("secret"), "outside").unwrap();
    symlink(s.path("secret"), root.join("link")).unwrap();
    let piped = Command::new("mkfifo").arg(root.join("pipe")).status();
    let made = create_zip(
        std::slice::from_ref(&root),
        &s.path("a.zip"),
        &NewArchive::default(),
        &Control::default(),
    )
    .unwrap();
    let mut data = Vec::new();
    let za = hivepack_zip::Archive::open(
        fs::File::open(&made).unwrap(),
        hivepack_zip::OpenFlags::RDONLY,
    )
    .unwrap();
    let mut entry = za.fopen(b"d/link", hivepack_zip::Flags::NONE).unwrap();
    std::io::Read::read_to_end(&mut entry, &mut data).unwrap();
    assert_eq!(data, s.path("secret").as_os_str().as_bytes());
    if piped.is_ok_and(|p| p.success()) {
        assert!(
            za.name_locate(b"d/pipe", hivepack_zip::Flags::NONE)
                .is_err()
        );
    }
}

#[cfg(unix)]
#[test]
fn a_name_that_is_not_unicode_is_refused() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let s = Scratch::new("bytes");
    let root = s.path("d");
    fs::create_dir_all(&root).unwrap();
    if fs::write(root.join(OsStr::from_bytes(b"caf\xe9")), "x").is_err() {
        return; // the file system does not take it
    }
    let err = create_zip(
        &[root],
        &s.path("a.zip"),
        &NewArchive::default(),
        &Control::default(),
    )
    .unwrap_err();
    assert!(err.to_string().contains("Unicode"), "{err}");
    assert!(!s.path("a.zip").exists());
}
