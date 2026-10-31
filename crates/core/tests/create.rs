//! Archives packwing-core makes, read back through packwing-core and, where
//! it is installed, by Info-ZIP's `unzip`.

#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use packwing_core::{
    Cancel, Control, Error, Keys, Kind, Level, NewArchive, Progress, create, extract, list,
    measure, test,
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
    let made = create(
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
    let made = create(
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
    let made = create(
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
    let made = create(
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
    let made = create(&[root], &s.path("a.zip"), &new, &Control::default()).unwrap();
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
    let made = create(&[root], &s.path("a.zip"), &new, &Control::default()).unwrap();
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
    let made = create(&[file], &s.path("x.zip"), &new, &Control::default()).unwrap();
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
        let result = create(
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
    create(&[root], &s.path("a.zip"), &NewArchive::default(), &control).unwrap();
    assert_eq!(total.load(Ordering::Relaxed), measured.bytes);
    assert_eq!(bytes.load(Ordering::Relaxed), measured.bytes);
}

#[test]
fn a_file_that_cannot_be_read_is_named_and_leaves_nothing() {
    let s = Scratch::new("missing");
    let err = create(
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
    let made = create(
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
    let err = create(
        &[root],
        &s.path("a.zip"),
        &NewArchive::default(),
        &Control::default(),
    )
    .unwrap_err();
    assert!(err.to_string().contains("Unicode"), "{err}");
    assert!(!s.path("a.zip").exists());
}

const KINDS: [(Kind, &str); 5] = [
    (Kind::Zip, "zip"),
    (Kind::SevenZip, "7z"),
    (Kind::TarGz, "tar.gz"),
    (Kind::TarXz, "tar.xz"),
    (Kind::TarZst, "tar.zst"),
];

const LEVELS: [Level; 5] = [
    Level::Lowest,
    Level::Low,
    Level::Medium,
    Level::High,
    Level::Highest,
];

#[test]
fn every_kind_at_every_level_comes_back_as_it_went_in() {
    let s = Scratch::new("kinds");
    let root = s.tree();
    for (kind, extension) in KINDS {
        for level in LEVELS {
            let name = format!("{kind:?}-{level:?}");
            let new = NewArchive {
                kind,
                level,
                ..NewArchive::default()
            };
            let made = create(
                std::slice::from_ref(&root),
                &s.path(&format!("{name}.{extension}")),
                &new,
                &Control::default(),
            )
            .unwrap();
            let listing = list(&made, &Keys::default()).unwrap();
            assert_eq!(listing.files, 3, "{name}");
            test(&made, &Keys::default(), &Control::default()).unwrap();
            let out = s.path(&format!("out-{name}"));
            extract(&made, &out, &Keys::default(), &Control::default()).unwrap();
            same_tree(&out.join("Báo cáo"));
        }
    }
}

#[test]
fn a_higher_level_makes_a_smaller_archive() {
    let s = Scratch::new("levels");
    let file = s.path("words.txt");
    let words: Vec<String> = (0..200_000u32)
        .map(|i| format!("{} ", i.wrapping_mul(2_654_435_761) % 5_000))
        .collect();
    fs::write(&file, words.concat()).unwrap();
    for (kind, extension) in KINDS {
        let size = |level| {
            let new = NewArchive {
                kind,
                level,
                ..NewArchive::default()
            };
            let path = s.path(&format!("{kind:?}-{level:?}.{extension}"));
            let made = create(
                std::slice::from_ref(&file),
                &path,
                &new,
                &Control::default(),
            );
            fs::metadata(made.unwrap()).unwrap().len()
        };
        let (lowest, highest) = (size(Level::Lowest), size(Level::Highest));
        assert!(highest < lowest, "{kind:?}: {highest} and {lowest}");
    }
}

#[test]
fn the_parts_of_every_kind_open_as_one() {
    let s = Scratch::new("kind-parts");
    let root = s.tree();
    for (kind, extension) in KINDS {
        let new = NewArchive {
            kind,
            part_size: Some(1_000_000),
            ..NewArchive::default()
        };
        let wanted = s.path(&format!("a.{extension}"));
        let made = create(
            std::slice::from_ref(&root),
            &wanted,
            &new,
            &Control::default(),
        )
        .unwrap();
        assert_eq!(made, s.path(&format!("a.{extension}.001")), "{kind:?}");
        let second = s.path(&format!("a.{extension}.002"));
        assert!(second.exists(), "{kind:?}");
        // From the first part and from another, and with its size in all.
        for part in [&made, &second] {
            let listing = list(part, &Keys::default()).unwrap();
            assert_eq!(listing.files, 3, "{kind:?}");
            assert!(listing.packed > 3_000_000, "{kind:?}: {}", listing.packed);
        }
        test(&made, &Keys::default(), &Control::default()).unwrap();
        let out = s.path(&format!("out-{extension}"));
        extract(&made, &out, &Keys::default(), &Control::default()).unwrap();
        same_tree(&out.join("Báo cáo"));
    }
}

#[test]
fn a_taken_name_keeps_all_of_its_extension() {
    let s = Scratch::new("taken");
    let file = s.path("a.txt");
    fs::write(&file, "a").unwrap();
    fs::write(s.path("x.tar.gz"), "mine").unwrap();
    let new = NewArchive {
        kind: Kind::TarGz,
        ..NewArchive::default()
    };
    let made = create(&[file], &s.path("x.tar.gz"), &new, &Control::default()).unwrap();
    assert_eq!(made, s.path("x (2).tar.gz"));
    assert_eq!(fs::read(s.path("x.tar.gz")).unwrap(), b"mine");
}

#[test]
fn a_7z_password_locks_the_files_and_can_hide_the_names() {
    let s = Scratch::new("7z-password");
    let root = s.tree();
    let right = Keys {
        password: Some("mật khẩu".as_bytes().to_vec()),
        ..Keys::default()
    };
    for hide_names in [false, true] {
        let new = NewArchive {
            kind: Kind::SevenZip,
            password: right.password.clone(),
            hide_names,
            ..NewArchive::default()
        };
        let path = s.path(&format!("{hide_names}.7z"));
        let made = create(
            std::slice::from_ref(&root),
            &path,
            &new,
            &Control::default(),
        )
        .unwrap();
        let bytes = fs::read(&made).unwrap();
        // A name in UTF-16, as 7z writes them, not to be found when hidden.
        let name: Vec<u8> = "lớn.bin"
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect();
        let shown = bytes.windows(name.len()).any(|w| w == name);
        // Compressed headers do not show it either way: only what can be read.
        assert!(!shown || !hide_names);
        let names = list(&made, &Keys::default());
        assert_eq!(
            matches!(names, Err(Error::Locked { .. })),
            hide_names,
            "{names:?}"
        );
        assert!(matches!(
            extract(&made, &s.path("no"), &Keys::default(), &Control::default()),
            Err(Error::Locked { .. })
        ));
        assert!(list(&made, &right).unwrap().encryption.is_some());
        let out = s.path(&format!("out-{hide_names}"));
        extract(&made, &out, &right, &Control::default()).unwrap();
        same_tree(&out.join("Báo cáo"));
    }
}

#[test]
fn a_tar_archive_refuses_a_password_rather_than_dropping_it() {
    let s = Scratch::new("tar-password");
    let file = s.path("a.txt");
    fs::write(&file, "a").unwrap();
    for (kind, extension) in &KINDS[2..] {
        let new = NewArchive {
            kind: *kind,
            password: Some(b"p".to_vec()),
            ..NewArchive::default()
        };
        let path = s.path(&format!("a.{extension}"));
        assert!(
            create(
                std::slice::from_ref(&file),
                &path,
                &new,
                &Control::default()
            )
            .is_err()
        );
        assert!(!path.exists());
    }
}

#[test]
fn stopping_any_kind_leaves_nothing_and_progress_reaches_the_end() {
    let s = Scratch::new("kind-stop");
    let root = s.tree();
    let measured = measure(std::slice::from_ref(&root)).unwrap();
    for (kind, extension) in KINDS {
        let path = s.path(&format!("a.{extension}"));
        let new = NewArchive {
            kind,
            ..NewArchive::default()
        };
        let cancel = Cancel::new();
        let stop = cancel.clone();
        let control = Control {
            progress: Some(Arc::new(move |p: &Progress| {
                if p.bytes > 0 {
                    stop.cancel();
                }
            })),
            cancel: Some(cancel),
        };
        let stopped = create(std::slice::from_ref(&root), &path, &new, &control);
        assert!(matches!(stopped, Err(Error::Cancelled)), "{stopped:?}");
        assert!(!path.exists(), "{kind:?}");

        let (bytes, entries) = (Arc::new(AtomicU64::new(0)), Arc::new(AtomicU64::new(0)));
        let (b, e) = (bytes.clone(), entries.clone());
        let control = Control {
            progress: Some(Arc::new(move |p: &Progress| {
                b.store(p.bytes, Ordering::Relaxed);
                e.store(p.entries, Ordering::Relaxed);
            })),
            cancel: None,
        };
        create(std::slice::from_ref(&root), &path, &new, &control).unwrap();
        assert_eq!(bytes.load(Ordering::Relaxed), measured.bytes, "{kind:?}");
        assert_eq!(entries.load(Ordering::Relaxed), measured.files, "{kind:?}");
    }
}

#[cfg(unix)]
#[test]
fn every_kind_keeps_a_link_as_a_link() {
    use std::os::unix::fs::symlink;
    let s = Scratch::new("kind-links");
    let root = s.path("d");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("file"), "data").unwrap();
    symlink("file", root.join("link")).unwrap();
    let measured = measure(std::slice::from_ref(&root)).unwrap();
    assert_eq!((measured.files, measured.bytes), (2, 4));
    for (kind, extension) in KINDS {
        let new = NewArchive {
            kind,
            ..NewArchive::default()
        };
        let path = s.path(&format!("a.{extension}"));
        let made = create(
            std::slice::from_ref(&root),
            &path,
            &new,
            &Control::default(),
        )
        .unwrap();
        let out = s.path(&format!("out-{extension}"));
        extract(&made, &out, &Keys::default(), &Control::default()).unwrap();
        let link = out.join("d/link");
        assert_eq!(fs::read_link(&link).unwrap(), Path::new("file"), "{kind:?}");
    }
}

#[cfg(unix)]
#[test]
fn a_link_out_of_the_folder_stops_the_extraction() {
    use std::os::unix::fs::symlink;
    let s = Scratch::new("link-out");
    let root = s.path("d");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("file"), "data").unwrap();
    symlink("../../outside", root.join("link")).unwrap();
    let made = create(
        std::slice::from_ref(&root),
        &s.path("a.tar.gz"),
        &NewArchive {
            kind: Kind::TarGz,
            ..NewArchive::default()
        },
        &Control::default(),
    )
    .unwrap();
    let out = s.path("out");
    assert!(extract(&made, &out, &Keys::default(), &Control::default()).is_err());
    assert!(fs::symlink_metadata(out.join("d/link")).is_err());
}

/// Whether `program` is there to be run.
fn installed(program: &str) -> bool {
    Command::new(program).arg("--version").output().is_ok()
}

#[test]
fn tar_reads_the_tar_archives_with_their_names() {
    let s = Scratch::new("tar-tool");
    let root = s.tree();
    if !installed("tar") {
        return;
    }
    for (kind, extension, filter) in [
        (Kind::TarGz, "tar.gz", "gzip"),
        (Kind::TarXz, "tar.xz", "xz"),
        (Kind::TarZst, "tar.zst", "zstd"),
    ] {
        if !installed(filter) {
            continue;
        }
        let new = NewArchive {
            kind,
            ..NewArchive::default()
        };
        let path = s.path(&format!("a.{extension}"));
        let made = create(
            std::slice::from_ref(&root),
            &path,
            &new,
            &Control::default(),
        )
        .unwrap();
        let out = s.path(&format!("out-{filter}"));
        fs::create_dir_all(&out).unwrap();
        let said = Command::new("tar")
            .arg("-xf")
            .arg(&made)
            .arg("-C")
            .arg(&out)
            .output()
            .unwrap();
        assert!(said.status.success(), "{kind:?}: {said:?}");
        same_tree(&out.join("Báo cáo"));
    }
}

#[test]
fn seven_zip_reads_the_7z_archives_and_their_passwords() {
    let s = Scratch::new("7z-tool");
    let root = s.tree();
    let Some(tool) = ["7zz", "7z", "7za"]
        .into_iter()
        .find(|t| Command::new(t).output().is_ok())
    else {
        return; // not installed
    };
    for (password, hide_names) in [
        (None, false),
        (Some("mật khẩu"), false),
        (Some("mật khẩu"), true),
    ] {
        let new = NewArchive {
            kind: Kind::SevenZip,
            password: password.map(|p| p.as_bytes().to_vec()),
            hide_names,
            ..NewArchive::default()
        };
        let path = s.path(&format!("{hide_names}-{}.7z", password.is_some()));
        let made = create(
            std::slice::from_ref(&root),
            &path,
            &new,
            &Control::default(),
        )
        .unwrap();
        let out = s.path(&format!("out-{hide_names}-{}", password.is_some()));
        let mut command = Command::new(tool);
        command
            .arg("x")
            .arg(format!("-o{}", out.display()))
            .arg("-y");
        if let Some(password) = password {
            command.arg(format!("-p{password}"));
        }
        let said = command.arg(&made).output().unwrap();
        assert!(said.status.success(), "{said:?}");
        same_tree(&out.join("Báo cáo"));
    }
    // And the parts, from the first.
    let new = NewArchive {
        kind: Kind::SevenZip,
        part_size: Some(1_000_000),
        ..NewArchive::default()
    };
    let made = create(&[root], &s.path("parts.7z"), &new, &Control::default()).unwrap();
    assert_eq!(made, s.path("parts.7z.001"));
    let out = s.path("out-parts");
    let said = Command::new(tool)
        .arg("x")
        .arg(format!("-o{}", out.display()))
        .arg("-y")
        .arg(&made)
        .output()
        .unwrap();
    assert!(said.status.success(), "{said:?}");
    same_tree(&out.join("Báo cáo"));
}
