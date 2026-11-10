//! Archives packwing-core makes, read back through packwing-core and, where
//! it is installed, by Info-ZIP's `unzip`.

#![allow(clippy::unwrap_used)]

use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use packwing_core::{
    Blocks, COMPRESSED_EXTENSIONS, Cancel, Control, Error, ExtractOptions, Frames, Item, Keys,
    Kind, Level, Lister, LocalFolder, NewArchive, OnExisting, Progress, Route, SEVERAL_FRAMES_FROM,
    Source, Storage, StoredFile, add, add_with, age_key_is_post_quantum, age_key_problem,
    age_keys_in, can_add, create, create_on, delete, extract, extract_selected, extract_to_open,
    extract_with, image_fits, list, measure, read_entry, rename, route, test,
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
    // Apple's unzip reads the archive whole but prints a letter it has no charset for as
    // `?`, whatever the archive says.
    if !cfg!(target_os = "macos") {
        assert!(said.contains("Báo cáo/Hình ảnh/lớn.bin"), "{said}");
    }
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
        s.path("joined.zip"),
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
            on_damaged: None,
            on_unnamable: None,
            on_links_outside: None,
            on_gone: None,
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
        on_damaged: None,
        on_unnamable: None,
        on_links_outside: None,
        on_gone: None,
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
    let za = hivepack::zip::Archive::open(
        fs::File::open(&made).unwrap(),
        hivepack::zip::OpenFlags::RDONLY,
    )
    .unwrap();
    let mut entry = za.fopen(b"d/link", hivepack::zip::Flags::NONE).unwrap();
    std::io::Read::read_to_end(&mut entry, &mut data).unwrap();
    assert_eq!(data, s.path("secret").as_os_str().as_bytes());
    if piped.is_ok_and(|p| p.success()) {
        assert!(
            za.name_locate(b"d/pipe", hivepack::zip::Flags::NONE)
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

/// Each file of the ZIP archive at `path`, and whether it went in as it is.
fn stored_in(path: &Path) -> Vec<(String, bool)> {
    let za = hivepack::zip::Archive::open(
        fs::File::open(path).unwrap(),
        hivepack::zip::OpenFlags::RDONLY,
    )
    .unwrap();
    (0..za.get_num_entries(hivepack::zip::Flags::NONE))
        .map(|i| za.stat_index(i, hivepack::zip::Flags::NONE).unwrap())
        .map(|stat| {
            let name = String::from_utf8_lossy(stat.name.unwrap_or_default()).into_owned();
            (name, stat.comp_method == hivepack::zip::cm::STORE)
        })
        .filter(|(name, _)| !name.ends_with('/'))
        .collect()
}

#[test]
fn stored_a_zip_or_7z_archive_holds_its_files_as_they_are() {
    let s = Scratch::new("store");
    let file = s.path("words.txt");
    let words = "xin chào thế giới ".repeat(60_000);
    fs::write(&file, &words).unwrap();
    for (kind, extension) in [(Kind::Zip, "zip"), (Kind::SevenZip, "7z")] {
        let made = |level| {
            let new = NewArchive {
                kind,
                level,
                ..NewArchive::default()
            };
            let path = s.path(&format!("{kind:?}-{level:?}.{extension}"));
            create(
                std::slice::from_ref(&file),
                &path,
                &new,
                &Control::default(),
            )
            .unwrap()
        };
        let (stored, lowest) = (made(Level::Store), made(Level::Lowest));
        let size = |path: &PathBuf| fs::metadata(path).unwrap().len();
        assert!(size(&stored) >= words.len() as u64, "{kind:?}");
        assert!(size(&lowest) < words.len() as u64 / 10, "{kind:?}");
        let out = s.path(&format!("out-{extension}"));
        extract(&stored, &out, &Keys::default(), &Control::default()).unwrap();
        assert_eq!(fs::read_to_string(out.join("words.txt")).unwrap(), words);
    }
    // A tar archive compressed whole takes its lowest level.
    let new = NewArchive {
        kind: Kind::TarZst,
        level: Level::Store,
        ..NewArchive::default()
    };
    let tar = create(
        std::slice::from_ref(&file),
        &s.path("words.tar.zst"),
        &new,
        &Control::default(),
    )
    .unwrap();
    assert!(fs::metadata(&tar).unwrap().len() < words.len() as u64 / 10);
}

#[test]
fn a_zip_archive_stores_the_extensions_it_is_told() {
    let s = Scratch::new("store-compressed");
    let what = s.path("what");
    fs::create_dir_all(&what).unwrap();
    // As compressible as the text: only its name says it is compressed.
    let text = "xin chào ".repeat(20_000);
    for name in ["notes.txt", "photo.JPG", "clip.mp4", "old.zip", "shot.raw"] {
        fs::write(what.join(name), &text).unwrap();
    }
    let known: Vec<String> = COMPRESSED_EXTENSIONS
        .iter()
        .map(|e| e.to_string())
        .collect();
    let mine = vec!["raw".to_owned(), "TXT".to_owned()];
    for (tag, stored_extensions) in [("known", known), ("mine", mine), ("none", Vec::new())] {
        let new = NewArchive {
            stored_extensions: stored_extensions.clone(),
            ..NewArchive::default()
        };
        let made = create(
            std::slice::from_ref(&what),
            &s.path(&format!("{tag}.zip")),
            &new,
            &Control::default(),
        )
        .unwrap();
        let wanted = |name: &str| match tag {
            "known" => !name.ends_with(".txt") && !name.ends_with(".raw"),
            "mine" => name.ends_with(".txt") || name.ends_with(".raw"),
            _ => false,
        };
        for (name, stored) in stored_in(&made) {
            assert_eq!(stored, wanted(&name), "{tag}: {name}");
        }
        // What is added follows the same rule.
        let more = s.path(&format!("more-{tag}.png"));
        fs::write(&more, &text).unwrap();
        add_with(
            &[more],
            &made,
            &[],
            OnExisting::Stop,
            &stored_extensions,
            &Keys::default(),
            &Control::default(),
        )
        .unwrap();
        let added = stored_in(&made)
            .into_iter()
            .find(|(name, _)| name.ends_with(".png"))
            .unwrap();
        assert_eq!(added.1, tag == "known", "{tag}");
    }
}

#[test]
fn a_7z_archive_stores_the_extensions_it_is_told_in_a_folder_of_their_own() {
    let s = Scratch::new("7z-store-compressed");
    let what = s.path("what");
    fs::create_dir_all(&what).unwrap();
    // As compressible as the text: only its name says it is compressed.
    let text = "xin chào ".repeat(20_000);
    for name in ["a.txt", "photo.JPG", "b.txt", "clip.mp4"] {
        fs::write(what.join(name), &text).unwrap();
    }
    let known: Vec<String> = COMPRESSED_EXTENSIONS
        .iter()
        .map(|e| e.to_string())
        .collect();
    let new = NewArchive {
        kind: Kind::SevenZip,
        stored_extensions: known,
        ..NewArchive::default()
    };
    let made = create(
        std::slice::from_ref(&what),
        &s.path("a.7z"),
        &new,
        &Control::default(),
    )
    .unwrap();
    let a = hivepack::sevenz::Archive::open(fs::File::open(&made).unwrap()).unwrap();
    let folder_of = |end: &str| {
        a.entries()
            .find(|e| e.name.ends_with(end))
            .and_then(|e| e.folder)
            .unwrap()
    };
    // The text together, compressed; the picture and the clip after, stored.
    let (text_folder, stored_folder) = (folder_of("a.txt"), folder_of("photo.JPG"));
    assert_eq!(folder_of("b.txt"), text_folder);
    assert_eq!(folder_of("clip.mp4"), stored_folder);
    assert_ne!(text_folder, stored_folder);
    assert_eq!(
        a.folder_packed_size(stored_folder),
        a.folder_size(stored_folder)
    );
    assert!(a.folder_packed_size(text_folder) < a.folder_size(text_folder) / 10);
    let out = s.path("out");
    extract(&made, &out, &Keys::default(), &Control::default()).unwrap();
    for name in ["a.txt", "photo.JPG", "b.txt", "clip.mp4"] {
        assert_eq!(
            fs::read_to_string(out.join("what").join(name)).unwrap(),
            text
        );
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
fn a_tar_archive_with_a_password_is_encrypted_rather_than_dropping_it() {
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
        let made = create(
            std::slice::from_ref(&file),
            &path,
            &new,
            &Control::default(),
        )
        .unwrap();
        assert!(
            fs::read(made)
                .unwrap()
                .starts_with(b"age-encryption.org/v1")
        );
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
                // Once it has read a file: what is compressed in threads
                // may count no bytes until the end.
                if p.entries > 0 || p.bytes > 0 {
                    stop.cancel();
                }
            })),
            cancel: Some(cancel),
            on_damaged: None,
            on_unnamable: None,
            on_links_outside: None,
            on_gone: None,
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
            on_damaged: None,
            on_unnamable: None,
            on_links_outside: None,
            on_gone: None,
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

/// A link is listed as one, with where it leads when the archive says without
/// its data being decompressed: tar and ZIP do, 7z does not; nor does a ZIP
/// archive whose link is encrypted.
#[cfg(unix)]
#[test]
fn a_link_is_listed_as_one_with_its_target_where_the_archive_says() {
    use std::os::unix::fs::symlink;
    let s = Scratch::new("list-links");
    let root = s.path("d");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("file"), "data").unwrap();
    symlink("file", root.join("link")).unwrap();
    let locked = NewArchive {
        kind: Kind::Zip,
        password: Some(b"pw".to_vec()),
        ..NewArchive::default()
    };
    let kinds = KINDS.map(|(kind, extension)| {
        let new = NewArchive {
            kind,
            ..NewArchive::default()
        };
        let target = (kind != Kind::SevenZip).then(|| "file".to_owned());
        (new, extension, target)
    });
    for (new, extension, target) in kinds.into_iter().chain([(locked, "locked.zip", None)]) {
        let path = s.path(&format!("a.{extension}"));
        let made = create(
            std::slice::from_ref(&root),
            &path,
            &new,
            &Control::default(),
        )
        .unwrap();
        let listing = list(made, &Keys::default()).unwrap();
        let found = |name: &str| {
            listing
                .entries
                .iter()
                .find(|e| e.path == ["d", name])
                .unwrap_or_else(|| panic!("{extension}: no {name}"))
        };
        assert_eq!(found("file").link, None, "{extension}");
        let link = found("link").link.clone().expect(extension);
        assert_eq!(link.target, target, "{extension}");
        let shown = listing.children(&["d".to_owned()]);
        assert_eq!(
            shown
                .iter()
                .find(|c| c.name == "link")
                .and_then(|c| c.link.clone()),
            Some(link),
            "{extension}"
        );
    }
}

/// What goes in is measured with the names some system can't use: Windows'
/// characters, and a name differing from another only in case.
#[cfg(unix)]
#[test]
fn names_some_system_cannot_use_are_found_when_measured() {
    let s = Scratch::new("not-portable");
    let root = s.path("d");
    fs::create_dir_all(&root).unwrap();
    for name in ["a:b.txt", "README", "readme", "fine.txt"] {
        fs::write(root.join(name), name).unwrap();
    }
    let measured = measure(std::slice::from_ref(&root)).unwrap();
    // A file system that takes `README` and `readme` for one, as macOS's does
    // by default, holds one of them: written twice, under the first name.
    if fs::read_dir(&root).unwrap().count() == 3 {
        assert_eq!(measured.files, 3);
        assert_eq!(measured.not_portable, ["d/a:b.txt"]);
    } else {
        assert_eq!(measured.files, 4);
        assert_eq!(measured.not_portable, ["d/a:b.txt", "d/readme"]);
    }
    let fine = measure(&[s.tree()]).unwrap();
    assert!(fine.not_portable.is_empty());
}

#[cfg(unix)]
#[test]
fn a_link_out_of_the_folder_is_left_out() {
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
    let summary = extract(&made, &out, &Keys::default(), &Control::default()).unwrap();
    assert!(fs::symlink_metadata(out.join("d/link")).is_err());
    assert_eq!(fs::read(out.join("d/file")).unwrap(), b"data");
    let left = packwing_core::left_out(&summary);
    assert_eq!(left.len(), 1);
    assert_eq!(left[0].path, "d/link");
}

/// Whether `program` is there to be run.
fn installed(program: &str) -> bool {
    Command::new(program).arg("--version").output().is_ok()
}

#[test]
fn tar_reads_the_tar_archives_with_their_names() {
    // Windows' tar (bsdtar) skips a PAX name it cannot put in the system's code page, as
    // with `ớ` under 1252, whoever made the archive: GNU tar's and Python's too.
    if cfg!(windows) {
        return;
    }
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

/// `a.zip`, holding `Báo cáo/`, to add to.
fn to_add_to(s: &Scratch, password: Option<&str>) -> PathBuf {
    let new = NewArchive {
        password: password.map(|p| p.as_bytes().to_vec()),
        ..NewArchive::default()
    };
    create(&[s.tree()], &s.path("a.zip"), &new, &Control::default()).unwrap()
}

/// Nothing beside the archive but what the test made: no `.a.zip.adding`.
fn nothing_left_beside(s: &Scratch) {
    for entry in fs::read_dir(&s.0).unwrap() {
        let name = entry.unwrap().file_name();
        assert!(!name.to_string_lossy().contains("adding"), "{name:?}");
    }
}

#[test]
fn added_files_join_what_is_there() {
    let s = Scratch::new("add");
    let made = to_add_to(&s, None);
    let (one, folder) = (s.path("mới.txt"), s.path("thêm"));
    fs::write(&one, "mới").unwrap();
    fs::create_dir_all(folder.join("trong")).unwrap();
    fs::write(folder.join("trong/x.txt"), "x").unwrap();
    add(
        &[one, folder],
        &made,
        &[],
        &Keys::default(),
        &Control::default(),
    )
    .unwrap();
    // Into a folder it holds, whose other files stay.
    let two = s.path("two.txt");
    fs::write(&two, "2").unwrap();
    let inner = ["Báo cáo".to_owned(), "Hình ảnh".to_owned()];
    add(&[two], &made, &inner, &Keys::default(), &Control::default()).unwrap();

    assert_eq!(list(&made, &Keys::default()).unwrap().files, 6);
    test(&made, &Keys::default(), &Control::default()).unwrap();
    let out = s.path("out");
    extract(&made, &out, &Keys::default(), &Control::default()).unwrap();
    same_tree(&out.join("Báo cáo"));
    assert_eq!(fs::read_to_string(out.join("mới.txt")).unwrap(), "mới");
    assert_eq!(
        fs::read_to_string(out.join("thêm/trong/x.txt")).unwrap(),
        "x"
    );
    assert_eq!(
        fs::read_to_string(out.join("Báo cáo/Hình ảnh/two.txt")).unwrap(),
        "2"
    );
    nothing_left_beside(&s);
    if let Ok(said) = Command::new("unzip").arg("-t").arg(&made).output() {
        assert!(said.status.success(), "{said:?}");
    }
}

#[test]
fn a_name_it_holds_changes_nothing() {
    let s = Scratch::new("add-taken");
    let made = to_add_to(&s, None);
    let before = fs::read(&made).unwrap();
    let dir = s.path("x");
    fs::create_dir_all(&dir).unwrap();
    let taken = dir.join("ghi chú.txt");
    fs::write(&taken, "other").unwrap();
    let err = add(
        &[taken],
        &made,
        &["Báo cáo".to_owned()],
        &Keys::default(),
        &Control::default(),
    )
    .unwrap_err();
    assert!(err.to_string().contains("ghi chú.txt"), "{err}");
    assert_eq!(fs::read(&made).unwrap(), before);
    nothing_left_beside(&s);
}

#[test]
fn a_folder_it_holds_takes_new_files_without_a_second_entry() {
    let s = Scratch::new("add-merge");
    let made = to_add_to(&s, None);
    let source = s.path("nguồn/Báo cáo");
    fs::create_dir_all(&source).unwrap();
    fs::write(source.join("mới.txt"), "mới").unwrap();
    add(&[source], &made, &[], &Keys::default(), &Control::default()).unwrap();
    let za = hivepack::zip::Archive::open(
        fs::File::open(&made).unwrap(),
        hivepack::zip::OpenFlags::RDONLY,
    )
    .unwrap();
    let mut folders = 0;
    for i in 0..za.get_num_entries(hivepack::zip::Flags::NONE) {
        let stat = za.stat_index(i, hivepack::zip::Flags::NONE).unwrap();
        folders += usize::from(stat.name == Some("Báo cáo/".as_bytes()));
    }
    assert_eq!(folders, 1);
    assert_eq!(list(&made, &Keys::default()).unwrap().files, 4);
}

#[test]
fn an_encrypted_archive_needs_its_password_and_encrypts_what_is_added() {
    let s = Scratch::new("add-password");
    let made = to_add_to(&s, Some("mật khẩu"));
    let before = fs::read(&made).unwrap();
    let new = s.path("mới.txt");
    fs::write(&new, "bí mật").unwrap();
    let paths = std::slice::from_ref(&new);
    let with = |p: &str| Keys {
        password: Some(p.as_bytes().to_vec()),
        ..Keys::default()
    };
    assert!(matches!(
        add(paths, &made, &[], &Keys::default(), &Control::default()),
        Err(Error::Locked { .. })
    ));
    assert!(matches!(
        add(paths, &made, &[], &with("sai"), &Control::default()),
        Err(Error::WrongPassword)
    ));
    assert_eq!(fs::read(&made).unwrap(), before);
    nothing_left_beside(&s);

    let right = with("mật khẩu");
    add(paths, &made, &[], &right, &Control::default()).unwrap();
    assert!(
        !fs::read(&made)
            .unwrap()
            .windows("bí mật".len())
            .any(|w| w == "bí mật".as_bytes())
    );
    assert!(matches!(
        extract(&made, &s.path("no"), &Keys::default(), &Control::default()),
        Err(Error::Locked { .. })
    ));
    let out = s.path("out");
    extract(&made, &out, &right, &Control::default()).unwrap();
    same_tree(&out.join("Báo cáo"));
    assert_eq!(fs::read_to_string(out.join("mới.txt")).unwrap(), "bí mật");
}

#[test]
fn stopping_or_failing_leaves_the_archive_as_it_was() {
    let s = Scratch::new("add-stop");
    let made = to_add_to(&s, None);
    let before = fs::read(&made).unwrap();
    let big_one = s.path("lớn.bin");
    fs::write(&big_one, big()).unwrap();
    let cancel = Cancel::new();
    let stop = cancel.clone();
    let control = Control {
        progress: Some(Arc::new(move |p: &Progress| {
            if p.bytes > 0 {
                stop.cancel();
            }
        })),
        cancel: Some(cancel),
        on_damaged: None,
        on_unnamable: None,
        on_links_outside: None,
        on_gone: None,
    };
    let stopped = add(&[big_one], &made, &[], &Keys::default(), &control);
    assert!(matches!(stopped, Err(Error::Cancelled)), "{stopped:?}");
    assert_eq!(fs::read(&made).unwrap(), before);
    nothing_left_beside(&s);

    let err = add(
        &[s.path("not there")],
        &made,
        &[],
        &Keys::default(),
        &Control::default(),
    )
    .unwrap_err();
    assert!(err.to_string().contains("not there"), "{err}");
    assert_eq!(fs::read(&made).unwrap(), before);
}

#[test]
fn only_a_zip_or_7z_archive_of_one_file_takes_more() {
    let s = Scratch::new("add-kinds");
    let root = s.tree();
    let extra = s.path("x.txt");
    fs::write(&extra, "x").unwrap();
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
        let listing = list(&made, &Keys::default()).unwrap();
        let changes = matches!(kind, Kind::Zip | Kind::SevenZip);
        assert_eq!(can_add(&listing), changes, "{kind:?}");
        let added = add(
            std::slice::from_ref(&extra),
            &made,
            &[],
            &Keys::default(),
            &Control::default(),
        );
        assert_eq!(added.is_ok(), changes, "{kind:?}: {added:?}");
    }
    let parts = NewArchive {
        part_size: Some(1_000_000),
        ..NewArchive::default()
    };
    let made = create(
        std::slice::from_ref(&root),
        &s.path("p.zip"),
        &parts,
        &Control::default(),
    )
    .unwrap();
    assert!(!can_add(&list(&made, &Keys::default()).unwrap()));
    assert!(add(&[extra], &made, &[], &Keys::default(), &Control::default()).is_err());
}

#[test]
fn an_archive_does_not_go_inside_itself() {
    let s = Scratch::new("add-itself");
    let made = to_add_to(&s, None);
    let before = fs::read(&made).unwrap();
    let err = add(
        std::slice::from_ref(&made),
        &made,
        &[],
        &Keys::default(),
        &Control::default(),
    )
    .unwrap_err();
    assert!(err.to_string().contains("itself"), "{err}");
    assert_eq!(fs::read(&made).unwrap(), before);
}

#[cfg(unix)]
#[test]
fn an_archive_keeps_its_permissions_when_added_to() {
    use std::os::unix::fs::PermissionsExt;
    let s = Scratch::new("add-mode");
    let made = to_add_to(&s, None);
    fs::set_permissions(&made, fs::Permissions::from_mode(0o640)).unwrap();
    let new = s.path("mới.txt");
    fs::write(&new, "x").unwrap();
    add(&[new], &made, &[], &Keys::default(), &Control::default()).unwrap();
    let mode = fs::metadata(&made).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o640);
}

#[test]
fn mac_files_are_left_out_when_asked() {
    let s = Scratch::new("mac");
    let root = s.path("d");
    fs::create_dir_all(root.join("__MACOSX/x")).unwrap();
    fs::create_dir_all(root.join("giữ")).unwrap();
    fs::write(root.join(".DS_Store"), "x").unwrap();
    fs::write(root.join("giữ/._a.txt"), "x").unwrap();
    fs::write(root.join("__MACOSX/x/y"), "x").unwrap();
    fs::write(root.join("giữ/a.txt"), "a").unwrap();
    fs::write(root.join("giữ/.hidden"), "h").unwrap();
    for (skip, files) in [(false, 5), (true, 2)] {
        for (kind, extension) in KINDS {
            let new = NewArchive {
                kind,
                skip_mac_files: skip,
                ..NewArchive::default()
            };
            let path = s.path(&format!("{skip}.{extension}"));
            let made = create(
                std::slice::from_ref(&root),
                &path,
                &new,
                &Control::default(),
            )
            .unwrap();
            let listing = list(&made, &Keys::default()).unwrap();
            assert_eq!(listing.files, files, "{kind:?} {skip}");
        }
    }
}

/// `a.zip` holding `Báo cáo/`, extracted once to `out`.
fn extracted_once(s: &Scratch) -> (PathBuf, PathBuf) {
    let made = to_add_to(s, None);
    let out = s.path("out");
    extract(&made, &out, &Keys::default(), &Control::default()).unwrap();
    (made, out)
}

#[test]
fn a_file_that_is_there_stops_the_job_unless_it_is_to_be_replaced() {
    let s = Scratch::new("replace");
    let (made, out) = extracted_once(&s);
    let note = out.join("Báo cáo/ghi chú.txt");
    fs::write(&note, "của tôi").unwrap();
    assert!(extract(&made, &out, &Keys::default(), &Control::default()).is_err());
    assert_eq!(fs::read_to_string(&note).unwrap(), "của tôi");
    let how = ExtractOptions {
        overwrite: true,
        ..ExtractOptions::default()
    };
    extract_with(&made, &out, &Keys::default(), &how, &Control::default()).unwrap();
    same_tree(&out.join("Báo cáo"));
}

#[test]
fn the_limit_stops_what_unpacks_to_more() {
    let s = Scratch::new("limit");
    let made = to_add_to(&s, None);
    let keys = Keys::default();
    let small = ExtractOptions {
        limit: Some(1_000_000),
        ..ExtractOptions::default()
    };
    let out = s.path("out");
    assert!(extract_with(&made, &out, &keys, &small, &Control::default()).is_err());
    assert!(!out.join("Báo cáo/Hình ảnh/lớn.bin").exists());
    let none = ExtractOptions {
        limit: None,
        ..ExtractOptions::default()
    };
    extract_with(&made, &out, &keys, &none, &Control::default()).unwrap();
    same_tree(&out.join("Báo cáo"));
}

fn path_of(text: &str) -> Vec<String> {
    text.split('/').map(str::to_owned).collect()
}

#[test]
fn what_is_picked_comes_out_straight_into_the_folder() {
    let s = Scratch::new("picked");
    let root = s.tree();
    let how = ExtractOptions::default();
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
        let picked = [path_of("Báo cáo/Hình ảnh"), path_of("Báo cáo/ghi chú.txt")];
        extract_selected(
            &made,
            &out,
            &picked,
            &Keys::default(),
            &how,
            &Control::default(),
        )
        .unwrap();
        let mut names: Vec<String> = fs::read_dir(&out)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        assert_eq!(names, ["Hình ảnh", "ghi chú.txt"], "{kind:?}");
        assert_eq!(fs::read(out.join("Hình ảnh/lớn.bin")).unwrap(), big());
        assert_eq!(
            fs::read_to_string(out.join("ghi chú.txt")).unwrap(),
            "xin chào\n"
        );
    }
}

#[test]
fn a_picked_name_that_is_there_stops_it_unless_replacing() {
    let s = Scratch::new("picked-there");
    let made = to_add_to(&s, None);
    let out = s.path("out");
    fs::create_dir_all(&out).unwrap();
    fs::write(out.join("ghi chú.txt"), "của tôi").unwrap();
    let picked = [path_of("Báo cáo/ghi chú.txt")];
    let keep = ExtractOptions::default();
    assert!(
        extract_selected(
            &made,
            &out,
            &picked,
            &Keys::default(),
            &keep,
            &Control::default()
        )
        .is_err()
    );
    assert_eq!(
        fs::read_to_string(out.join("ghi chú.txt")).unwrap(),
        "của tôi"
    );
    let replace = ExtractOptions {
        overwrite: true,
        ..keep
    };
    extract_selected(
        &made,
        &out,
        &picked,
        &Keys::default(),
        &replace,
        &Control::default(),
    )
    .unwrap();
    assert_eq!(
        fs::read_to_string(out.join("ghi chú.txt")).unwrap(),
        "xin chào\n"
    );
    let names: Vec<_> = fs::read_dir(&out).unwrap().collect();
    assert_eq!(names.len(), 1, "nothing but it is left");
}

#[test]
fn stopping_a_picked_extraction_leaves_nothing() {
    let s = Scratch::new("picked-stop");
    let made = to_add_to(&s, None);
    let out = s.path("out");
    let cancel = Cancel::new();
    let stop = cancel.clone();
    let control = Control {
        progress: Some(Arc::new(move |p: &Progress| {
            if p.bytes > 0 {
                stop.cancel();
            }
        })),
        cancel: Some(cancel),
        on_damaged: None,
        on_unnamable: None,
        on_links_outside: None,
        on_gone: None,
    };
    let picked = [path_of("Báo cáo/Hình ảnh")];
    let how = ExtractOptions::default();
    let stopped = extract_selected(&made, &out, &picked, &Keys::default(), &how, &control);
    assert!(matches!(stopped, Err(Error::Cancelled)), "{stopped:?}");
    assert_eq!(fs::read_dir(&out).unwrap().count(), 0);
}

#[test]
fn a_file_to_open_is_a_copy_in_a_folder_of_its_own() {
    let s = Scratch::new("to-open");
    let made = to_add_to(&s, Some("mật khẩu"));
    let entry = path_of("Báo cáo/ghi chú.txt");
    assert!(matches!(
        extract_to_open(&made, &entry, &Keys::default(), &Control::default()),
        Err(Error::Locked { .. })
    ));
    let keys = Keys {
        password: Some("mật khẩu".as_bytes().to_vec()),
        ..Keys::default()
    };
    let copy = extract_to_open(&made, &entry, &keys, &Control::default()).unwrap();
    assert_eq!(fs::read_to_string(&copy).unwrap(), "xin chào\n");
    assert_eq!(copy.file_name().unwrap().to_string_lossy(), "ghi chú.txt");
    let again = extract_to_open(&made, &entry, &keys, &Control::default()).unwrap();
    assert_ne!(copy, again);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let folder = fs::metadata(copy.parent().unwrap()).unwrap();
        assert_eq!(folder.permissions().mode() & 0o777, 0o700);
    }
    let _ = fs::remove_dir_all(copy.parent().unwrap());
    let _ = fs::remove_dir_all(again.parent().unwrap());
}

#[test]
fn an_entry_can_be_read_up_to_a_limit_and_leaves_nothing() {
    let s = Scratch::new("read-entry");
    let made = to_add_to(&s, None);
    let keys = Keys::default();
    let control = Control::default();
    let note = path_of("Báo cáo/ghi chú.txt");
    assert_eq!(
        read_entry(&made, &note, &keys, 1 << 20, &control).unwrap(),
        "xin chào\n".as_bytes()
    );
    let large = path_of("Báo cáo/Hình ảnh/lớn.bin");
    assert!(read_entry(&made, &large, &keys, 1_000_000, &control).is_err());
    assert_eq!(
        read_entry(&made, &large, &keys, 4_000_000, &control).unwrap(),
        big()
    );
    let locked = to_add_to_named(&s, "b.zip", "mật khẩu");
    assert!(matches!(
        read_entry(&locked, &note, &keys, 1 << 20, &control),
        Err(Error::Locked { .. })
    ));
}

/// A ZIP archive of `Báo cáo/` with a password, named `name`.
fn to_add_to_named(s: &Scratch, name: &str, password: &str) -> PathBuf {
    let new = NewArchive {
        password: Some(password.as_bytes().to_vec()),
        ..NewArchive::default()
    };
    create(
        &[s.path("Báo cáo")],
        &s.path(name),
        &new,
        &Control::default(),
    )
    .unwrap()
}

/// A PNG's first bytes, which say its size.
fn png_of_size(width: u32, height: u32) -> Vec<u8> {
    let mut png = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR".to_vec();
    png.extend(width.to_be_bytes());
    png.extend(height.to_be_bytes());
    png.extend([8, 6, 0, 0, 0, 0, 0, 0, 0]);
    png
}

#[test]
fn a_picture_that_says_it_is_huge_is_not_decoded() {
    assert!(image_fits(&png_of_size(4_000, 3_000)));
    assert!(!image_fits(&png_of_size(50_000, 50_000)));
    assert!(!image_fits(b"not a picture"));
}

fn names_in(path: &Path, keys: &Keys) -> Vec<String> {
    let mut names: Vec<String> = list(path, keys)
        .unwrap()
        .entries
        .iter()
        .filter(|e| !e.dir)
        .map(|e| e.path.join("/"))
        .collect();
    names.sort();
    names
}

#[test]
fn a_name_it_holds_can_be_replaced_or_skipped() {
    let s = Scratch::new("on-existing");
    let made = to_add_to(&s, None);
    let source = s.path("nguồn/Báo cáo");
    fs::create_dir_all(&source).unwrap();
    fs::write(source.join("ghi chú.txt"), "mới").unwrap();
    fs::write(source.join("thêm.txt"), "thêm").unwrap();
    let keys = Keys::default();
    let control = Control::default();
    let read_note = |s: &Scratch, name: &str| {
        let out = s.path(name);
        extract(&made, &out, &keys, &control).unwrap();
        fs::read_to_string(out.join("Báo cáo/ghi chú.txt")).unwrap()
    };
    assert!(add(std::slice::from_ref(&source), &made, &[], &keys, &control).is_err());
    add_with(
        std::slice::from_ref(&source),
        &made,
        &[],
        OnExisting::Skip,
        &[],
        &keys,
        &control,
    )
    .unwrap();
    assert_eq!(read_note(&s, "skipped"), "xin chào\n");
    assert_eq!(list(&made, &keys).unwrap().files, 4);
    fs::write(source.join("thêm.txt"), "lần hai").unwrap();
    add_with(
        std::slice::from_ref(&source),
        &made,
        &[],
        OnExisting::Replace,
        &[],
        &keys,
        &control,
    )
    .unwrap();
    assert_eq!(read_note(&s, "replaced"), "mới");
    assert_eq!(list(&made, &keys).unwrap().files, 4, "no name twice");
    test(&made, &keys, &control).unwrap();
    if let Ok(said) = Command::new("unzip").arg("-t").arg(&made).output() {
        assert!(said.status.success(), "{said:?}");
    }
    // A file where it holds a folder is never replaced.
    let clash = s.path("khác");
    fs::create_dir_all(&clash).unwrap();
    fs::write(clash.join("Hình ảnh"), "x").unwrap();
    let inner = ["Báo cáo".to_owned()];
    let file = [clash.join("Hình ảnh")];
    assert!(
        add_with(
            &file,
            &made,
            &inner,
            OnExisting::Replace,
            &[],
            &keys,
            &control
        )
        .is_err()
    );
}

#[test]
fn entries_can_be_deleted_folders_with_what_they_hold() {
    let s = Scratch::new("delete");
    let made = to_add_to(&s, None);
    let keys = Keys::default();
    delete(
        &made,
        &[path_of("Báo cáo/rỗng")],
        &keys,
        &Control::default(),
    )
    .unwrap();
    delete(
        &made,
        &[path_of("Báo cáo/Hình ảnh")],
        &keys,
        &Control::default(),
    )
    .unwrap();
    assert_eq!(names_in(&made, &keys), ["Báo cáo/ghi chú.txt"]);
    test(&made, &keys, &Control::default()).unwrap();
    if let Ok(said) = Command::new("unzip").arg("-t").arg(&made).output() {
        assert!(said.status.success(), "{said:?}");
    }
    let before = fs::read(&made).unwrap();
    assert!(delete(&made, &[path_of("không có")], &keys, &Control::default()).is_err());
    assert!(
        delete(&made, &[path_of("Báo cáo")], &keys, &Control::default()).is_err(),
        "all of it"
    );
    assert_eq!(fs::read(&made).unwrap(), before);
    nothing_left_beside(&s);
}

#[test]
fn entries_can_be_renamed_folders_with_what_they_hold() {
    let s = Scratch::new("rename");
    let made = to_add_to(&s, Some("mật khẩu"));
    let keys = Keys {
        password: Some("mật khẩu".as_bytes().to_vec()),
        ..Keys::default()
    };
    rename(&made, &path_of("Báo cáo/ghi chú.txt"), "đã đổi.txt", &keys).unwrap();
    rename(&made, &path_of("Báo cáo/Hình ảnh"), "Ảnh", &keys).unwrap();
    assert_eq!(
        names_in(&made, &keys),
        ["Báo cáo/rỗng", "Báo cáo/đã đổi.txt", "Báo cáo/Ảnh/lớn.bin"]
    );
    // The data is as it was, and still needs the password.
    let out = s.path("out");
    assert!(extract(&made, &s.path("no"), &Keys::default(), &Control::default()).is_err());
    extract(&made, &out, &keys, &Control::default()).unwrap();
    assert_eq!(fs::read(out.join("Báo cáo/Ảnh/lớn.bin")).unwrap(), big());
    let before = fs::read(&made).unwrap();
    for bad in ["", "a/b", "a\\b", "..", "rỗng"] {
        assert!(
            rename(&made, &path_of("Báo cáo/đã đổi.txt"), bad, &keys).is_err(),
            "{bad:?}"
        );
    }
    assert_eq!(fs::read(&made).unwrap(), before);
    nothing_left_beside(&s);
}

#[test]
fn only_a_zip_archive_of_one_file_is_changed() {
    let s = Scratch::new("change-kinds");
    let root = s.tree();
    let new = NewArchive {
        kind: Kind::TarGz,
        ..NewArchive::default()
    };
    let made = create(&[root], &s.path("a.tar.gz"), &new, &Control::default()).unwrap();
    let keys = Keys::default();
    assert!(
        delete(
            &made,
            &[path_of("Báo cáo/rỗng")],
            &keys,
            &Control::default()
        )
        .is_err()
    );
    assert!(rename(&made, &path_of("Báo cáo/rỗng"), "x", &keys).is_err());
}

/// A folder as a storage across a network is: no path on this computer. `refuse`
/// fails the write of a file named that.
struct Across {
    folder: LocalFolder,
    refuse: Option<&'static str>,
    sends_as_read: bool,
    writes_anywhere: bool,
}

/// A file on [`Across`]: under a part name in its folder till it is finished.
struct AcrossFile {
    file: fs::File,
    part: PathBuf,
    to: PathBuf,
    finished: bool,
}

impl io::Write for AcrossFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

impl io::Seek for AcrossFile {
    fn seek(&mut self, to: io::SeekFrom) -> io::Result<u64> {
        self.file.seek(to)
    }
}

impl StoredFile for AcrossFile {
    fn finish(mut self: Box<Self>) -> io::Result<u64> {
        fs::rename(&self.part, &self.to)?;
        self.finished = true;
        Ok(self.file.metadata()?.len())
    }
}

impl Drop for AcrossFile {
    fn drop(&mut self) {
        if !self.finished {
            let _ = fs::remove_file(&self.part);
        }
    }
}

impl Storage for Across {
    fn list(&self, folder: &[String]) -> io::Result<Box<dyn Lister>> {
        self.folder.list(folder)
    }
    fn read(&self, path: &[String], most: u64) -> io::Result<Vec<u8>> {
        self.folder.read(path, most)
    }
    fn read_at(&self, path: &[String], offset: u64, len: usize) -> io::Result<Vec<u8>> {
        self.folder.read_at(path, offset, len)
    }
    fn stat(&self, folder: &[String], name: &str) -> io::Result<Option<Item>> {
        self.folder.stat(folder, name)
    }
    fn open(&self, path: &[String]) -> io::Result<Box<dyn Read + Send>> {
        self.folder.open(path)
    }
    fn put(
        &self,
        path: &[String],
        from: &mut dyn Read,
        like: &Item,
        replace: bool,
    ) -> io::Result<u64> {
        if self.refuse == path.last().map(String::as_str) {
            return Err(io::Error::other("refused"));
        }
        self.folder.put(path, from, like, replace)
    }
    fn remove(&self, path: &[String]) -> io::Result<()> {
        self.folder.remove(path)
    }
    fn make_folder(&self, folder: &[String], name: &str) -> io::Result<()> {
        self.folder.make_folder(folder, name)
    }
    fn rename(&self, folder: &[String], from: &str, to: &str) -> io::Result<()> {
        self.folder.rename(folder, from, to)
    }
    fn delete(&self, folder: &[String], names: &[String]) -> io::Result<()> {
        self.folder.delete(folder, names)
    }
    fn has_trash(&self) -> bool {
        false
    }
    fn local_path(&self, _: &[String]) -> Option<PathBuf> {
        None
    }
    fn show_path(&self, path: &[String]) -> String {
        path.join("/")
    }
    fn sends_as_read(&self) -> bool {
        self.sends_as_read
    }
    fn writes_anywhere(&self) -> bool {
        self.writes_anywhere
    }
    fn new_file(&self, path: &[String], _: &Item) -> io::Result<Box<dyn StoredFile>> {
        let to = self.folder.local_path(path).expect("a folder here");
        let part = to.with_extension("part");
        let file = fs::File::create_new(&part)?;
        Ok(Box::new(AcrossFile {
            file,
            part,
            to,
            finished: false,
        }))
    }
}

/// A folder to make an archive of, and an empty one as the storage.
fn up_scratch(name: &str) -> (PathBuf, PathBuf, Across) {
    let top = std::env::temp_dir().join(format!("packwing-up-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&top);
    let (what, there) = (top.join("what"), top.join("there"));
    fs::create_dir_all(&what).unwrap();
    fs::create_dir_all(&there).unwrap();
    // Random, so that it does not fit in one small part.
    let mut noise = vec![0u8; 3_000_000];
    let mut state = 1u32;
    for byte in &mut noise {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        *byte = (state >> 24) as u8;
    }
    fs::write(what.join("noise.bin"), noise).unwrap();
    let across = Across {
        folder: LocalFolder::new(&there),
        refuse: None,
        sends_as_read: false,
        writes_anywhere: false,
    };
    (what, there, across)
}

fn files_in(folder: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(folder)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn an_archive_goes_up_whole_and_reads_back() {
    let (what, there, across) = up_scratch("whole");
    let told = std::sync::atomic::AtomicU32::new(0);
    let made = create_on(
        std::slice::from_ref(&what),
        &across,
        &[],
        "a.zip",
        &NewArchive::default(),
        &Control::default(),
        &|| {
            told.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        },
    )
    .unwrap();
    assert_eq!(made, None);
    assert_eq!(told.into_inner(), 1);
    assert_eq!(files_in(&there), ["a.zip"]);
    let out = there.join("out");
    extract(
        there.join("a.zip"),
        &out,
        &Keys::default(),
        &Control::default(),
    )
    .unwrap();
    assert_eq!(
        fs::metadata(out.join("what/noise.bin")).unwrap().len(),
        3_000_000
    );
}

#[test]
fn a_name_taken_there_moves_on_to_the_next() {
    let (what, there, across) = up_scratch("taken");
    fs::write(there.join("a.tar.gz"), "someone else's").unwrap();
    let how = NewArchive {
        kind: Kind::TarGz,
        ..NewArchive::default()
    };
    create_on(
        &[what],
        &across,
        &[],
        "a.tar.gz",
        &how,
        &Control::default(),
        &|| {},
    )
    .unwrap();
    assert_eq!(files_in(&there), ["a (2).tar.gz", "a.tar.gz"]);
    assert_eq!(fs::read(there.join("a.tar.gz")).unwrap(), b"someone else's");
}

#[test]
fn parts_all_go_up_and_a_failure_takes_back_those_sent() {
    let (what, there, mut across) = up_scratch("parts");
    let how = NewArchive {
        part_size: Some(1_000_000),
        ..NewArchive::default()
    };
    across.refuse = Some("a.zip.003");
    let e = create_on(
        std::slice::from_ref(&what),
        &across,
        &[],
        "a.zip",
        &how,
        &Control::default(),
        &|| {},
    );
    assert!(e.is_err());
    assert!(files_in(&there).is_empty(), "{:?}", files_in(&there));
    across.refuse = None;
    create_on(
        &[what],
        &across,
        &[],
        "a.zip",
        &how,
        &Control::default(),
        &|| {},
    )
    .unwrap();
    assert_eq!(
        files_in(&there),
        ["a.zip.001", "a.zip.002", "a.zip.003", "a.zip.004"]
    );
}

#[test]
fn on_a_folder_here_it_is_made_where_it_goes() {
    let (what, there, _) = up_scratch("here");
    let made = create_on(
        &[what],
        &LocalFolder::new(&there),
        &[],
        "a.zip",
        &NewArchive::default(),
        &Control::default(),
        &|| panic!("nothing goes up"),
    )
    .unwrap();
    assert_eq!(made, Some(there.join("a.zip")));
}

#[test]
fn stopped_while_it_looks_through_the_files_nothing_is_left() {
    let (what, there, across) = up_scratch("stopped");
    let cancel = Cancel::new();
    cancel.cancel();
    let control = Control {
        progress: None,
        cancel: Some(cancel),
        on_damaged: None,
        on_unnamable: None,
        on_links_outside: None,
        on_gone: None,
    };
    let e = create_on(
        &[what],
        &across,
        &[],
        "a.zip",
        &NewArchive::default(),
        &control,
        &|| {},
    );
    assert!(matches!(e, Err(Error::Cancelled)), "{e:?}");
    assert!(files_in(&there).is_empty());
}

fn straight(kind: Kind) -> NewArchive {
    NewArchive {
        kind,
        stream_above: Some(0),
        ..NewArchive::default()
    }
}

/// `create_on` with `how`, and whether it went through a temporary file.
fn up(
    what: &Path,
    across: &Across,
    name: &str,
    how: &NewArchive,
    control: &Control,
) -> Result<bool, Error> {
    let through = std::sync::atomic::AtomicBool::new(false);
    create_on(
        &[what.to_path_buf()],
        across,
        &[],
        name,
        how,
        control,
        &|| through.store(true, Ordering::Relaxed),
    )?;
    Ok(through.into_inner())
}

#[test]
fn the_route_follows_the_storage_the_format_and_the_size() {
    let (_, there, mut across) = up_scratch("route");
    let tar = |above| NewArchive {
        kind: Kind::TarZst,
        stream_above: above,
        ..NewArchive::default()
    };
    let zip = NewArchive {
        stream_above: Some(100),
        ..NewArchive::default()
    };
    let parts = NewArchive {
        part_size: Some(1_000_000),
        ..tar(Some(100))
    };
    let seven = NewArchive {
        kind: Kind::SevenZip,
        stream_above: Some(100),
        ..NewArchive::default()
    };
    // A folder here takes anything where it is.
    assert_eq!(
        route(&LocalFolder::new(&there), &[], &zip, 1000),
        Route::Here
    );
    // A storage that holds a file first takes nothing straight.
    assert_eq!(route(&across, &[], &tar(Some(100)), 1000), Route::Temporary);
    across.sends_as_read = true;
    assert_eq!(route(&across, &[], &tar(Some(100)), 1000), Route::Straight);
    assert_eq!(route(&across, &[], &tar(Some(100)), 100), Route::Temporary);
    assert_eq!(route(&across, &[], &tar(None), 1000), Route::Temporary);
    assert_eq!(route(&across, &[], &zip, 1000), Route::Straight);
    assert_eq!(route(&across, &[], &seven, 1000), Route::Temporary);
    // Encrypted whole with age, a 7z archive is made first and goes forwards.
    let sealed = NewArchive {
        whole_with_age: true,
        ..seven.clone()
    };
    assert_eq!(route(&across, &[], &sealed, 1000), Route::Straight);
    assert_eq!(route(&across, &[], &parts, 1000), Route::Temporary);
    across.writes_anywhere = true;
    assert_eq!(route(&across, &[], &seven, 1000), Route::Straight);
    assert_eq!(route(&across, &[], &parts, 1000), Route::Temporary);
}

#[test]
fn a_tar_or_zip_archive_goes_straight_as_it_is_written_and_reads_back() {
    let (what, there, mut across) = up_scratch("piped");
    across.sends_as_read = true;
    for (kind, name) in [
        (Kind::TarGz, "a.tar.gz"),
        (Kind::TarXz, "a.tar.xz"),
        (Kind::TarZst, "a.tar.zst"),
        (Kind::Zip, "a.zip"),
    ] {
        let through = up(&what, &across, name, &straight(kind), &Control::default()).unwrap();
        assert!(!through, "{name}");
        let out = there.join(format!("out-{name}"));
        extract(
            there.join(name),
            &out,
            &Keys::default(),
            &Control::default(),
        )
        .unwrap();
        assert_eq!(
            fs::metadata(out.join("what/noise.bin")).unwrap().len(),
            3_000_000
        );
    }
}

#[test]
fn a_zip_or_7z_archive_goes_straight_where_it_can_write_anywhere() {
    let (what, there, mut across) = up_scratch("anywhere");
    across.writes_anywhere = true;
    for (kind, name) in [(Kind::Zip, "a.zip"), (Kind::SevenZip, "a.7z")] {
        let through = up(&what, &across, name, &straight(kind), &Control::default()).unwrap();
        assert!(!through, "{name}");
        let out = there.join(format!("out-{name}"));
        extract(
            there.join(name),
            &out,
            &Keys::default(),
            &Control::default(),
        )
        .unwrap();
        assert_eq!(
            fs::metadata(out.join("what/noise.bin")).unwrap().len(),
            3_000_000
        );
    }
}

#[test]
fn an_upload_refused_part_way_says_so_and_leaves_nothing() {
    let (what, there, mut across) = up_scratch("refused");
    across.sends_as_read = true;
    across.refuse = Some("a.tar.gz");
    let e = up(
        &what,
        &across,
        "a.tar.gz",
        &straight(Kind::TarGz),
        &Control::default(),
    );
    let said = e.expect_err("refused").to_string();
    assert!(said.contains("refused"), "{said}");
    assert!(files_in(&there).is_empty(), "{:?}", files_in(&there));
}

#[test]
fn stopped_on_its_way_straight_nothing_is_left() {
    let (what, there, mut across) = up_scratch("stopped-straight");
    let cancel = Cancel::new();
    let told = cancel.clone();
    let control = Control {
        progress: Some(Arc::new(move |p: &Progress| {
            if p.bytes > 0 {
                told.cancel();
            }
        })),
        cancel: Some(cancel),
        on_damaged: None,
        on_unnamable: None,
        on_links_outside: None,
        on_gone: None,
    };
    across.sends_as_read = true;
    let e = up(&what, &across, "a.tar.gz", &straight(Kind::TarGz), &control);
    assert!(matches!(e, Err(Error::Cancelled)), "{e:?}");
    assert!(files_in(&there).is_empty(), "{:?}", files_in(&there));
    across.sends_as_read = false;
    across.writes_anywhere = true;
    let e = up(&what, &across, "a.zip", &straight(Kind::Zip), &control);
    assert!(matches!(e, Err(Error::Cancelled)), "{e:?}");
    assert!(files_in(&there).is_empty(), "{:?}", files_in(&there));
}

#[test]
fn a_zip_archive_with_a_password_goes_straight_and_opens_with_it() {
    let (what, there, mut across) = up_scratch("piped-password");
    across.sends_as_read = true;
    let how = NewArchive {
        password: Some(b"pass".to_vec()),
        ..straight(Kind::Zip)
    };
    assert!(!up(&what, &across, "a.zip", &how, &Control::default()).unwrap());
    let keys = Keys {
        password: Some(b"pass".to_vec()),
        ..Keys::default()
    };
    let out = there.join("out");
    extract(there.join("a.zip"), &out, &keys, &Control::default()).unwrap();
    assert_eq!(
        fs::metadata(out.join("what/noise.bin")).unwrap().len(),
        3_000_000
    );
}

#[test]
fn seven_zip_goes_through_a_temporary_file_where_it_cannot_go_back() {
    let (what, there, mut across) = up_scratch("seven-temporary");
    across.sends_as_read = true;
    assert!(
        up(
            &what,
            &across,
            "a.7z",
            &straight(Kind::SevenZip),
            &Control::default()
        )
        .unwrap()
    );
    assert_eq!(files_in(&there), ["a.7z"]);
}

/// A tool of this name finds `path` sound, if it is on this machine.
fn tool_agrees(tool: &str, path: &Path) {
    let run = Command::new(tool).arg("-t").arg(path).output();
    if let Ok(out) = run {
        assert!(
            out.status.success(),
            "{tool}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

#[test]
fn the_new_tar_kinds_read_back() {
    let (what, there, _) = up_scratch("new-tars");
    for (kind, name, tool) in [
        (Kind::Tar, "a.tar", None),
        (Kind::TarBz2, "a.tar.bz2", Some("bzip2")),
        (Kind::TarLz4, "a.tar.lz4", Some("lz4")),
    ] {
        for level in [Level::Lowest, Level::Medium, Level::Highest] {
            let how = NewArchive {
                kind,
                level,
                ..NewArchive::default()
            };
            let made = create(
                std::slice::from_ref(&what),
                &there.join(name),
                &how,
                &Control::default(),
            )
            .unwrap();
            if let Some(tool) = tool {
                tool_agrees(tool, &made);
            }
            let out = there.join(format!("out-{name}-{level:?}"));
            extract(&made, &out, &Keys::default(), &Control::default()).unwrap();
            assert_eq!(
                fs::read(out.join("what/noise.bin")).unwrap(),
                fs::read(what.join("noise.bin")).unwrap(),
                "{name}"
            );
            fs::remove_file(&made).unwrap();
        }
    }
}

#[test]
fn one_file_compressed_by_itself_comes_back_under_its_name() {
    let (what, there, _) = up_scratch("one-file");
    let file = what.join("dump.sql");
    let body = b"insert into t values (1);\n".repeat(50_000);
    fs::write(&file, &body).unwrap();
    for (kind, name, tool) in [
        (Kind::Gz, "dump.sql.gz", "gzip"),
        (Kind::Xz, "dump.sql.xz", "xz"),
        (Kind::Zst, "dump.sql.zst", "zstd"),
        (Kind::Bz2, "dump.sql.bz2", "bzip2"),
        (Kind::Lz4, "dump.sql.lz4", "lz4"),
    ] {
        let how = NewArchive {
            kind,
            ..NewArchive::default()
        };
        let made = create(
            std::slice::from_ref(&file),
            &there.join(name),
            &how,
            &Control::default(),
        )
        .unwrap();
        tool_agrees(tool, &made);
        assert!(
            fs::metadata(&made).unwrap().len() < body.len() as u64 / 10,
            "{name}"
        );
        let out = there.join(format!("out-{name}"));
        extract(&made, &out, &Keys::default(), &Control::default()).unwrap();
        assert_eq!(fs::read(out.join("dump.sql")).unwrap(), body, "{name}");
    }
}

#[test]
fn a_folder_or_two_files_do_not_go_in_a_format_for_one_file() {
    let (what, there, _) = up_scratch("not-one-file");
    let how = NewArchive {
        kind: Kind::Zst,
        ..NewArchive::default()
    };
    let e = create(
        std::slice::from_ref(&what),
        &there.join("a.zst"),
        &how,
        &Control::default(),
    );
    assert!(e.is_err());
    assert!(files_in(&there).is_empty(), "{:?}", files_in(&there));
}

#[test]
fn one_file_goes_straight_as_it_is_compressed() {
    let (what, there, mut across) = up_scratch("one-file-straight");
    across.sends_as_read = true;
    let file = what.join("noise.bin");
    let how = straight(Kind::Zst);
    let through = std::sync::atomic::AtomicBool::new(false);
    create_on(
        std::slice::from_ref(&file),
        &across,
        &[],
        "noise.bin.zst",
        &how,
        &Control::default(),
        &|| through.store(true, Ordering::Relaxed),
    )
    .unwrap();
    assert!(!through.into_inner());
    let out = there.join("out");
    extract(
        there.join("noise.bin.zst"),
        &out,
        &Keys::default(),
        &Control::default(),
    )
    .unwrap();
    assert_eq!(
        fs::metadata(out.join("noise.bin")).unwrap().len(),
        3_000_000
    );
}

/// A key made by `age-keygen` for this test, and its public half.
const AGE_KEY: &str = "AGE-SECRET-KEY-17PWZMFQ7LGDHF4VS2U5FVRVTGDN747AW6RW8Q9QZKNKV5WNJ7QASFAFKUD";
const AGE_PUBLIC: &str = "age1ml7zyg7w7uqg5qrlunupaygdcckn0vun4xwvlpx2cdmwn7vqdugqhzawyg";

/// The file of a post-quantum key made by age v1.3.2 for the tests, and its public half.
fn pq_key() -> (Vec<u8>, String) {
    let file =
        fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/test-age-pq.key")).unwrap();
    let text = String::from_utf8_lossy(&file);
    let public = text
        .lines()
        .find_map(|line| line.strip_prefix("# public key: "))
        .unwrap()
        .to_owned();
    (file, public)
}

/// What `age` makes of `path`, with these arguments, if it is on this machine.
fn age_decrypts(args: &[&str], path: &Path) -> Option<Vec<u8>> {
    let out = Command::new("age")
        .arg("--decrypt")
        .args(args)
        .arg(path)
        .output()
        .ok()?;
    assert!(
        out.status.success(),
        "age: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    Some(out.stdout)
}

#[test]
fn a_tar_archive_encrypted_to_a_passphrase_reads_back_with_it_alone() {
    let (what, there, _) = up_scratch("age-passphrase");
    let how = NewArchive {
        kind: Kind::TarZst,
        password: Some(b"correct horse".to_vec()),
        ..NewArchive::default()
    };
    let made = create(
        std::slice::from_ref(&what),
        &there.join("a.tar.zst.age"),
        &how,
        &Control::default(),
    )
    .unwrap();
    // Its names cannot be read without it.
    assert!(list(Source::from(made.clone()), &Keys::default()).is_err());
    let keys = Keys {
        passphrase: Some(b"correct horse".to_vec()),
        ..Keys::default()
    };
    let out = there.join("out");
    extract(&made, &out, &keys, &Control::default()).unwrap();
    assert_eq!(
        fs::read(out.join("what/noise.bin")).unwrap(),
        fs::read(what.join("noise.bin")).unwrap()
    );
    let wrong = Keys {
        passphrase: Some(b"wrong".to_vec()),
        ..Keys::default()
    };
    assert!(extract(&made, &there.join("no"), &wrong, &Control::default()).is_err());
}

#[test]
fn encrypted_to_keys_it_opens_with_the_private_key_and_age_agrees() {
    let (what, there, _) = up_scratch("age-keys");
    let file = what.join("dump.sql");
    let body = b"insert into t values (1);\n".repeat(10_000);
    fs::write(&file, &body).unwrap();
    let how = NewArchive {
        kind: Kind::Gz,
        age_keys: vec![AGE_PUBLIC.to_owned()],
        ..NewArchive::default()
    };
    let made = create(
        std::slice::from_ref(&file),
        &there.join("dump.sql.gz.age"),
        &how,
        &Control::default(),
    )
    .unwrap();
    let keys = Keys {
        key_file: Some(AGE_KEY.as_bytes().to_vec()),
        ..Keys::default()
    };
    let out = there.join("out");
    extract(&made, &out, &keys, &Control::default()).unwrap();
    assert_eq!(fs::read(out.join("dump.sql")).unwrap(), body);
    assert!(
        extract(
            &made,
            &there.join("no"),
            &Keys::default(),
            &Control::default()
        )
        .is_err()
    );
    let key_file = there.join("key.txt");
    fs::write(&key_file, AGE_KEY).unwrap();
    let identity = key_file.to_string_lossy().into_owned();
    if let Some(gz) = age_decrypts(&["-i", &identity], &made) {
        // gzip's magic: age gave back what was compressed.
        assert_eq!(&gz[..2], &[0x1f, 0x8b]);
    }
}

#[test]
fn it_goes_straight_encrypted_and_nothing_is_left_when_it_fails() {
    let (what, there, mut across) = up_scratch("age-straight");
    across.sends_as_read = true;
    let how = NewArchive {
        kind: Kind::TarZst,
        stream_above: Some(0),
        age_keys: vec![AGE_PUBLIC.to_owned()],
        ..NewArchive::default()
    };
    assert!(!up(&what, &across, "a.tar.zst.age", &how, &Control::default()).unwrap());
    let keys = Keys {
        key_file: Some(AGE_KEY.as_bytes().to_vec()),
        ..Keys::default()
    };
    let out = there.join("out");
    extract(
        there.join("a.tar.zst.age"),
        &out,
        &keys,
        &Control::default(),
    )
    .unwrap();
    assert_eq!(
        fs::metadata(out.join("what/noise.bin")).unwrap().len(),
        3_000_000
    );
    fs::remove_dir_all(&out).unwrap();
    fs::remove_file(there.join("a.tar.zst.age")).unwrap();
    across.refuse = Some("b.tar.zst.age");
    assert!(up(&what, &across, "b.tar.zst.age", &how, &Control::default()).is_err());
    assert!(files_in(&there).is_empty(), "{:?}", files_in(&there));
}

#[test]
fn a_name_taken_moves_on_before_what_says_it_is_encrypted() {
    let (what, there, _) = up_scratch("age-name");
    let how = NewArchive {
        kind: Kind::TarGz,
        password: Some(b"p".to_vec()),
        ..NewArchive::default()
    };
    for _ in 0..2 {
        create(
            std::slice::from_ref(&what),
            &there.join("a.tar.gz.age"),
            &how,
            &Control::default(),
        )
        .unwrap();
    }
    assert_eq!(files_in(&there), ["a (2).tar.gz.age", "a.tar.gz.age"]);
}

#[test]
fn encrypted_to_a_post_quantum_key_it_opens_with_that_key_and_age_agrees() {
    let (what, there, _) = up_scratch("age-pq");
    let file = what.join("dump.sql");
    let body = b"insert into t values (1);\n".repeat(10_000);
    fs::write(&file, &body).unwrap();
    let (key_file, public) = pq_key();
    assert!(age_key_problem(&public).is_none());
    assert!(age_key_is_post_quantum(&public));
    assert!(!age_key_is_post_quantum(AGE_PUBLIC));
    let how = NewArchive {
        kind: Kind::Gz,
        age_keys: vec![public],
        ..NewArchive::default()
    };
    let made = create(
        std::slice::from_ref(&file),
        &there.join("dump.sql.gz.age"),
        &how,
        &Control::default(),
    )
    .unwrap();
    let text = fs::read(&made).unwrap();
    assert!(text[..200].windows(14).any(|w| w == b"mlkem768x25519"));
    let keys = Keys {
        key_file: Some(key_file.clone()),
        ..Keys::default()
    };
    let out = there.join("out");
    extract(&made, &out, &keys, &Control::default()).unwrap();
    assert_eq!(fs::read(out.join("dump.sql")).unwrap(), body);
    // Another key does not open it.
    let other = Keys {
        key_file: Some(AGE_KEY.as_bytes().to_vec()),
        ..Keys::default()
    };
    let no = there.join("no");
    assert!(extract(&made, &no, &other, &Control::default()).is_err());
    // age reads these keys from 1.3 on.
    let knows_them = Command::new("age-keygen")
        .arg("-pq")
        .output()
        .is_ok_and(|o| o.status.success());
    if knows_them {
        let path = there.join("key.txt");
        fs::write(&path, &key_file).unwrap();
        let identity = path.to_string_lossy().into_owned();
        let gz = age_decrypts(&["-i", &identity], &made).unwrap();
        assert_eq!(&gz[..2], &[0x1f, 0x8b]);
    }
}

#[test]
fn post_quantum_keys_do_not_go_with_others() {
    let (what, there, _) = up_scratch("age-pq-mixed");
    let (_, public) = pq_key();
    let how = NewArchive {
        kind: Kind::Tar,
        age_keys: vec![public.clone(), AGE_PUBLIC.to_owned()],
        ..NewArchive::default()
    };
    let e = create(
        std::slice::from_ref(&what),
        &there.join("a"),
        &how,
        &Control::default(),
    );
    assert!(e.is_err());
    assert!(files_in(&there).is_empty());
    assert!(age_keys_in(format!("{public}\n{AGE_PUBLIC}\n").as_bytes()).is_err());
    let alone = format!("# mine\n{public}\n");
    assert_eq!(age_keys_in(alone.as_bytes()).unwrap(), (vec![public], 0));
}

#[test]
fn what_age_cannot_do_is_refused_before_anything_is_made() {
    let (what, there, _) = up_scratch("age-refused");
    for how in [
        NewArchive {
            kind: Kind::Tar,
            age_keys: vec!["not a key".to_owned()],
            ..NewArchive::default()
        },
        NewArchive {
            kind: Kind::Tar,
            password: Some(b"p".to_vec()),
            age_keys: vec![AGE_PUBLIC.to_owned()],
            ..NewArchive::default()
        },
        NewArchive {
            kind: Kind::Zip,
            age_keys: vec![AGE_PUBLIC.to_owned()],
            ..NewArchive::default()
        },
    ] {
        let e = create(
            std::slice::from_ref(&what),
            &there.join("a"),
            &how,
            &Control::default(),
        );
        assert!(e.is_err(), "{how:?}");
        assert!(files_in(&there).is_empty());
    }
}

#[test]
fn zip_and_7z_encrypted_whole_with_age_read_back_and_age_agrees() {
    let (what, there, _) = up_scratch("age-whole");
    let passphrase = Keys {
        passphrase: Some(b"correct horse".to_vec()),
        ..Keys::default()
    };
    let key = Keys {
        key_file: Some(AGE_KEY.as_bytes().to_vec()),
        ..Keys::default()
    };
    let key_file = there.join("key.txt");
    fs::write(&key_file, AGE_KEY).unwrap();
    let identity = key_file.to_string_lossy().into_owned();
    for (kind, ext, magic) in [
        (Kind::Zip, "zip", &b"PK\x03\x04"[..]),
        (Kind::SevenZip, "7z", &b"7z\xbc\xaf\x27\x1c"[..]),
    ] {
        for (to_keys, keys) in [(false, &passphrase), (true, &key)] {
            let how = NewArchive {
                kind,
                password: (!to_keys).then(|| b"correct horse".to_vec()),
                age_keys: if to_keys {
                    vec![AGE_PUBLIC.to_owned()]
                } else {
                    Vec::new()
                },
                whole_with_age: true,
                ..NewArchive::default()
            };
            let made = create(
                std::slice::from_ref(&what),
                &there.join(format!("a-{to_keys}.{ext}.age")),
                &how,
                &Control::default(),
            )
            .unwrap();
            // Its names cannot be read without what it was encrypted to.
            assert!(
                list(Source::from(made.clone()), &Keys::default()).is_err(),
                "{how:?}"
            );
            assert!(
                names_in(&made, keys).contains(&"what/noise.bin".to_owned()),
                "{how:?}"
            );
            let out = there.join(format!("out-{ext}-{to_keys}"));
            extract(&made, &out, keys, &Control::default()).unwrap();
            assert_eq!(
                fs::read(out.join("what/noise.bin")).unwrap(),
                fs::read(what.join("noise.bin")).unwrap(),
                "{how:?}"
            );
            if to_keys && let Some(inner) = age_decrypts(&["-i", &identity], &made) {
                // Under age, the archive itself, with no password of its own.
                assert_eq!(&inner[..magic.len()], magic, "{how:?}");
            }
        }
    }
}

#[test]
fn a_7z_archive_encrypted_whole_with_age_goes_straight() {
    let (what, there, mut across) = up_scratch("age-whole-straight");
    across.sends_as_read = true;
    let how = NewArchive {
        kind: Kind::SevenZip,
        stream_above: Some(0),
        age_keys: vec![AGE_PUBLIC.to_owned()],
        whole_with_age: true,
        ..NewArchive::default()
    };
    assert!(!up(&what, &across, "a.7z.age", &how, &Control::default()).unwrap());
    let keys = Keys {
        key_file: Some(AGE_KEY.as_bytes().to_vec()),
        ..Keys::default()
    };
    let out = there.join("out");
    extract(there.join("a.7z.age"), &out, &keys, &Control::default()).unwrap();
    assert_eq!(
        fs::read(out.join("what/noise.bin")).unwrap(),
        fs::read(what.join("noise.bin")).unwrap()
    );
}

#[test]
fn keys_are_read_from_a_file_as_age_reads_them() {
    assert!(age_key_problem(AGE_PUBLIC).is_none());
    assert!(age_key_problem("nonsense").is_some());
    let file = format!("# mine\n\n{AGE_PUBLIC}\n");
    assert_eq!(
        age_keys_in(file.as_bytes()).unwrap(),
        (vec![AGE_PUBLIC.to_owned()], 0)
    );
    assert!(age_keys_in(b"# nothing here\n").is_err());
    // A private key has no place there.
    assert!(age_keys_in(AGE_KEY.as_bytes()).is_err());
}

/// What is a ZIP, a 7z or a tar archive under another name, as Java, Android,
/// Firefox, ebooks, comic books and the short names of compressed tars give
/// it, opens as one: packwing goes by its bytes.
#[test]
fn an_archive_under_another_name_opens() {
    let s = Scratch::new("names");
    let root = s.path("Báo cáo");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("ghi chú.txt"), "xin chào\n").unwrap();
    let cases: [(Kind, &str, &[&str]); 8] = [
        (
            Kind::Zip,
            "zip",
            &["jar", "war", "ear", "apk", "xpi", "epub", "cbz"],
        ),
        (Kind::SevenZip, "7z", &["cb7"]),
        (Kind::Tar, "tar", &["cbt"]),
        (Kind::TarGz, "tar.gz", &["tgz"]),
        (Kind::TarBz2, "tar.bz2", &["tbz2", "tbz"]),
        (Kind::TarXz, "tar.xz", &["txz"]),
        (Kind::TarZst, "tar.zst", &["tzst"]),
        (Kind::TarLz4, "tar.lz4", &["tlz4"]),
    ];
    for (kind, ext, names) in cases {
        let new = NewArchive {
            kind,
            ..NewArchive::default()
        };
        let made = create(
            std::slice::from_ref(&root),
            &s.path(&format!("a.{ext}")),
            &new,
            &Control::default(),
        )
        .unwrap();
        for name in names {
            let renamed = s.path(&format!("a.{name}"));
            fs::copy(&made, &renamed).unwrap();
            let listing = list(&renamed, &Keys::default())
                .unwrap_or_else(|e| panic!(".{name}, a {ext}: {e}"));
            assert_eq!(listing.files, 1, ".{name}");
            test(&renamed, &Keys::default(), &Control::default())
                .unwrap_or_else(|e| panic!(".{name}, a {ext}: {e}"));
        }
    }
}

/// `.tlz`, `.tar.lzma`'s short name: .lzma has no magic, and is told by its name.
#[test]
fn a_tar_lzma_under_its_short_name_opens() {
    let tlz = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/notes.tlz");
    let listing = list(&tlz, &Keys::default()).unwrap();
    assert_eq!(listing.files, 3);
    test(&tlz, &Keys::default(), &Control::default()).unwrap();
}

/// How many frames the seek table at the end of a zstd stream says it has, if
/// it has one: zstd's seekable format's.
fn frames_in_seek_table(made: &[u8]) -> Option<u32> {
    let footer = made.get(made.len().checked_sub(9)?..)?;
    (footer[5..] == 0x8F92_EAB1u32.to_le_bytes())
        .then(|| u32::from_le_bytes([footer[0], footer[1], footer[2], footer[3]]))
}

/// A `.tar.zst` or `.zst` in several frames, as asked or for a large input, has
/// a frame for each 32 MiB and a seek table, and comes back as it went in; in
/// one, it has neither.
#[test]
fn zstd_goes_in_frames_as_asked_or_as_the_size_says() {
    let s = Scratch::new("frames");
    let root = s.path("big");
    fs::create_dir_all(&root).unwrap();
    // 70 MiB: three frames.
    let piece = big();
    let mut data = Vec::with_capacity(70 << 20);
    while data.len() < 70 << 20 {
        data.extend_from_slice(&piece);
        data.push(data.len() as u8);
    }
    data.truncate(70 << 20);
    fs::write(root.join("data.bin"), &data).unwrap();
    fs::write(root.join("small.txt"), "xin chào\n").unwrap();
    for (kind, extension, from) in [
        (Kind::TarZst, "tar.zst", root.clone()),
        (Kind::Zst, "zst", root.join("data.bin")),
    ] {
        for (frames, want) in [
            (Frames::Several, Some(3)),
            (Frames::One, None),
            // 70 MiB is under what makes several.
            (Frames::Auto, None),
        ] {
            let name = format!("{frames:?}.{extension}");
            let new = NewArchive {
                kind,
                level: Level::Lowest,
                frames,
                ..NewArchive::default()
            };
            let made = create(
                std::slice::from_ref(&from),
                &s.path(&name),
                &new,
                &Control::default(),
            )
            .unwrap();
            let bytes = fs::read(&made).unwrap();
            assert_eq!(frames_in_seek_table(&bytes), want, "{name}");
            test(&made, &Keys::default(), &Control::default()).unwrap();
            let out = s.path(&format!("out-{name}"));
            extract(&made, &out, &Keys::default(), &Control::default()).unwrap();
            // A file alone comes back named after the archive.
            let back = if kind == Kind::Zst {
                out.join(format!("{frames:?}"))
            } else {
                out.join("big/data.bin")
            };
            assert!(fs::read(back).unwrap() == data, "{name}");
        }
    }
    assert!(Frames::Auto.several(Some(SEVERAL_FRAMES_FROM)));
    assert!(!Frames::Auto.several(Some(SEVERAL_FRAMES_FROM - 1)));
    assert!(!Frames::Auto.several(None));
}

/// The headers of the plain tar archive at `path`, by name.
fn tar_headers(path: &Path) -> Vec<hivepack::tar::Header> {
    let mut reader = hivepack::tar::Reader::new(fs::File::open(path).unwrap());
    let mut headers = Vec::new();
    while let Some(header) = reader.next().unwrap() {
        headers.push(header);
    }
    headers
}

fn header<'a>(headers: &'a [hivepack::tar::Header], name: &str) -> &'a hivepack::tar::Header {
    headers
        .iter()
        .find(|h| h.name == name.as_bytes())
        .unwrap_or_else(|| panic!("no {name}"))
}

fn plain_tar(keep: packwing_core::Keep, times: packwing_core::Times) -> NewArchive {
    NewArchive {
        kind: Kind::Tar,
        keep,
        times,
        ..NewArchive::default()
    }
}

/// A file with two names is in a tar archive once, its second name a hard link
/// to the first, and comes out as one file with two names; ZIP keeps both whole.
#[cfg(unix)]
#[test]
fn a_file_with_two_names_goes_in_a_tar_archive_once() {
    use std::os::unix::fs::MetadataExt;
    let s = Scratch::new("hard-links");
    let root = s.path("d");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("a"), big()).unwrap();
    fs::hard_link(root.join("a"), root.join("b")).unwrap();
    let new = plain_tar(Default::default(), Default::default());
    let made = create(
        std::slice::from_ref(&root),
        &s.path("a.tar"),
        &new,
        &Control::default(),
    )
    .unwrap();
    let headers = tar_headers(&made);
    assert_eq!(header(&headers, "d/a").typeflag, hivepack::tar::TYPE_REG);
    let b = header(&headers, "d/b");
    assert_eq!(
        (b.typeflag, b.linkname.as_slice()),
        (hivepack::tar::TYPE_LINK, &b"d/a"[..])
    );
    assert!(fs::metadata(&made).unwrap().len() < 2 * big().len() as u64);
    let out = s.path("out");
    extract(&made, &out, &Keys::default(), &Control::default()).unwrap();
    let (a, b) = (
        fs::metadata(out.join("d/a")).unwrap(),
        fs::metadata(out.join("d/b")).unwrap(),
    );
    assert_eq!(a.ino(), b.ino());
    assert_eq!(fs::read(out.join("d/b")).unwrap(), big());
    // ZIP has no hard links: both are whole.
    let zip = NewArchive::default();
    let made = create(
        std::slice::from_ref(&root),
        &s.path("a.zip"),
        &zip,
        &Control::default(),
    )
    .unwrap();
    let listing = list(made, &Keys::default()).unwrap();
    let sizes: Vec<u64> = listing
        .entries
        .iter()
        .filter(|e| !e.dir)
        .map(|e| e.size)
        .collect();
    assert_eq!(sizes, [big().len() as u64; 2]);
}

/// Owners are kept in a tar archive only when asked: by number and by name.
#[cfg(unix)]
#[test]
fn owners_are_kept_when_asked() {
    use std::os::unix::fs::MetadataExt;
    let s = Scratch::new("owners");
    let file = s.path("f");
    fs::write(&file, "x").unwrap();
    let mine = fs::metadata(&file).unwrap();
    let made = |keep, name: &str| {
        let new = plain_tar(keep, Default::default());
        create(
            std::slice::from_ref(&file),
            &s.path(name),
            &new,
            &Control::default(),
        )
        .unwrap()
    };
    let plain = tar_headers(&made(Default::default(), "plain.tar"));
    let h = header(&plain, "f");
    assert_eq!((h.uid, h.gid, h.uname.len(), h.gname.len()), (0, 0, 0, 0));
    let keep = packwing_core::Keep {
        owners: true,
        ..Default::default()
    };
    let kept = tar_headers(&made(keep, "kept.tar"));
    let h = header(&kept, "f");
    assert_eq!(
        (h.uid, h.gid),
        (i64::from(mine.uid()), i64::from(mine.gid()))
    );
    let user = nix::unistd::User::from_uid(nix::unistd::Uid::from_raw(mine.uid()))
        .unwrap()
        .map(|u| u.name)
        .unwrap_or_default();
    assert_eq!(String::from_utf8_lossy(&h.uname), user);
}

/// Extended attributes are kept in a tar archive only when asked, as PAX's
/// `SCHILY.xattr.` records, which GNU tar reads.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn extended_attributes_are_kept_when_asked() {
    let s = Scratch::new("xattrs");
    let file = s.path("f");
    fs::write(&file, "x").unwrap();
    if let Err(e) = xattr::set(&file, "user.packwing", b"b\x00inary") {
        eprintln!("skipping: this file system takes no extended attributes ({e})");
        return;
    }
    let made = |keep, name: &str| {
        let new = plain_tar(keep, Default::default());
        create(
            std::slice::from_ref(&file),
            &s.path(name),
            &new,
            &Control::default(),
        )
        .unwrap()
    };
    let plain = tar_headers(&made(Default::default(), "plain.tar"));
    assert!(header(&plain, "f").xattrs.is_empty());
    let keep = packwing_core::Keep {
        xattrs: true,
        ..Default::default()
    };
    let path = made(keep, "kept.tar");
    let kept = tar_headers(&path);
    assert_eq!(
        header(&kept, "f")
            .xattrs
            .get(&b"user.packwing"[..])
            .map(Vec::as_slice),
        Some(&b"b\x00inary"[..])
    );
    if installed("tar") && cfg!(target_os = "linux") {
        let out = s.path("gnu");
        fs::create_dir_all(&out).unwrap();
        let status = Command::new("tar")
            .args(["--xattrs", "--xattrs-include=user.*", "-xf"])
            .arg(&path)
            .arg("-C")
            .arg(&out)
            .status()
            .unwrap();
        assert!(status.success());
        assert_eq!(
            xattr::get(out.join("f"), "user.packwing")
                .unwrap()
                .as_deref(),
            Some(&b"b\x00inary"[..])
        );
    }
}

/// The same files, changed at other times, make the same archive with one time
/// for all, of every kind; a time is taken back to the latest one allowed.
#[test]
fn one_time_for_all_makes_the_same_archive() {
    let s = Scratch::new("times");
    let make_tree = |name: &str, secs: u64| {
        let root = s.path(name).join("d");
        fs::create_dir_all(root.join("sub")).unwrap();
        for (file, text) in [("a.txt", "a"), ("sub/b.txt", "b")] {
            let path = root.join(file);
            fs::write(&path, text).unwrap();
            let when = std::time::UNIX_EPOCH + std::time::Duration::from_secs(secs);
            fs::File::options()
                .write(true)
                .open(&path)
                .unwrap()
                .set_modified(when)
                .unwrap();
        }
        root
    };
    let first = make_tree("one", 1_700_000_000);
    let second = make_tree("two", 1_800_000_000);
    for (kind, extension) in KINDS {
        let new = NewArchive {
            kind,
            times: packwing_core::Times::All(1_600_000_000),
            ..NewArchive::default()
        };
        let made = |root: &PathBuf, name: &str| {
            let path = create(
                std::slice::from_ref(root),
                &s.path(&format!("{name}.{extension}")),
                &new,
                &Control::default(),
            )
            .unwrap();
            fs::read(path).unwrap()
        };
        assert_eq!(made(&first, "one"), made(&second, "two"), "{extension}");
    }
    let new = plain_tar(
        Default::default(),
        packwing_core::Times::AtMost(1_750_000_000),
    );
    let headers = |root: &PathBuf, name: &str| {
        let path = create(
            std::slice::from_ref(root),
            &s.path(name),
            &new,
            &Control::default(),
        )
        .unwrap();
        tar_headers(&path)
    };
    let early = headers(&first, "early.tar");
    let late = headers(&second, "late.tar");
    let secs =
        |headers: &[hivepack::tar::Header]| header(headers, "d/a.txt").mod_time.map(|t| t.secs);
    assert_eq!(secs(&early), Some(1_700_000_000));
    assert_eq!(secs(&late), Some(1_750_000_000));
}

#[test]
fn gzip_goes_in_blocks_compressed_in_threads() {
    let s = Scratch::new("gzip-threads");
    // Several blocks of 128 KiB, of words and noise.
    let mut data = Vec::new();
    let mut state = 7u64;
    while data.len() < 900_000 {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        match state % 8 {
            0 => data.extend((0..(state % 200)).map(|i| (state >> (i % 56)) as u8)),
            n => data.extend_from_slice(
                ["packwing ", "gzip ", "thread ", "block\n"][n as usize % 4].as_bytes(),
            ),
        }
    }
    let file = s.path("a.txt");
    fs::write(&file, &data).unwrap();
    let new = NewArchive {
        kind: Kind::Gz,
        level: Level::Medium,
        ..NewArchive::default()
    };
    let made = create(&[file], &s.path("a.txt.gz"), &new, &Control::default()).unwrap();
    // pigz's stream, as one thread makes it: the same whatever the threads.
    let mut one = hivepack::deflate::DeflateMt::new(&hivepack::deflate::MtOptions {
        level: 6,
        ..hivepack::deflate::MtOptions::default()
    })
    .unwrap();
    let mut want = vec![0; data.len() + (64 << 10)];
    let mut output = hivepack::deflate::OutBuffer::new(&mut want);
    let mut input = hivepack::deflate::InBuffer::new(&data);
    let status = one
        .deflate(&mut output, &mut input, hivepack::deflate::Flush::Finish)
        .unwrap();
    assert_eq!(status, hivepack::deflate::Status::StreamEnd);
    let n = output.pos;
    want.truncate(n);
    assert_eq!(fs::read(&made).unwrap(), want);
}

/// What a tar archive kept for a backup comes back only when asked: a file's
/// group, here another of the user's where there is one, by number and by name;
/// the user's own otherwise.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn owners_come_back_when_asked() {
    use nix::unistd::{Gid, Group};
    use packwing_core::Owners;
    use std::os::unix::fs::MetadataExt;
    let s = Scratch::new("owners-back");
    let file = s.path("f");
    fs::write(&file, "x").unwrap();
    let mine = fs::metadata(&file).unwrap();
    // Another of the user's groups, which a file extracted is not given unasked:
    // as `id -G` says them, since nix has no getgroups on macOS.
    let groups = Command::new("id")
        .arg("-G")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default();
    let Some(other) = groups
        .split_whitespace()
        .filter_map(|g| g.parse::<u32>().ok())
        .find(|&g| g != mine.gid() && Group::from_gid(Gid::from_raw(g)).is_ok_and(|g| g.is_some()))
    else {
        eprintln!("skipping: the user has one group");
        return;
    };
    std::os::unix::fs::chown(&file, None, Some(other)).unwrap();
    let keep = packwing_core::Keep {
        owners: true,
        ..Default::default()
    };
    let made = create(
        std::slice::from_ref(&file),
        &s.path("kept.tar"),
        &plain_tar(keep, Default::default()),
        &Control::default(),
    )
    .unwrap();
    let group_of = |owners, name: &str| {
        let how = ExtractOptions {
            owners,
            ..ExtractOptions::default()
        };
        let out = s.path(name);
        let summary =
            extract_with(&made, &out, &Keys::default(), &how, &Control::default()).unwrap();
        assert_eq!(
            summary.owners_left, 0,
            "{owners:?}: the user's own and their group"
        );
        fs::metadata(out.join("f")).unwrap().gid()
    };
    assert_eq!(group_of(Owners::Ignore, "unasked"), mine.gid());
    assert_eq!(group_of(Owners::ByNumber, "by-number"), other);
    assert_eq!(group_of(Owners::ByName, "by-name"), other);
}

/// The extended attributes a tar archive kept come back only when asked.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn extended_attributes_come_back_when_asked() {
    let s = Scratch::new("xattrs-back");
    let file = s.path("f");
    fs::write(&file, "x").unwrap();
    if let Err(e) = xattr::set(&file, "user.packwing", b"kept") {
        eprintln!("skipping: this file system takes no extended attributes ({e})");
        return;
    }
    let keep = packwing_core::Keep {
        xattrs: true,
        ..Default::default()
    };
    let made = create(
        std::slice::from_ref(&file),
        &s.path("kept.tar"),
        &plain_tar(keep, Default::default()),
        &Control::default(),
    )
    .unwrap();
    let attribute_of = |xattrs, name: &str| {
        let how = ExtractOptions {
            xattrs,
            ..ExtractOptions::default()
        };
        let out = s.path(name);
        extract_with(&made, &out, &Keys::default(), &how, &Control::default()).unwrap();
        xattr::get(out.join("f"), "user.packwing").unwrap()
    };
    assert_eq!(attribute_of(false, "unasked"), None);
    assert_eq!(attribute_of(true, "asked").as_deref(), Some(&b"kept"[..]));
}

/// A Control that removes `doomed` once the files are found and before any is
/// read, as another program's files may go while an archive is made, and
/// keeps what it is told was gone.
fn removing(doomed: Vec<PathBuf>) -> (Control, Arc<std::sync::Mutex<Vec<PathBuf>>>) {
    let gone = Arc::new(std::sync::Mutex::new(Vec::new()));
    let told = gone.clone();
    let control = Control {
        progress: Some(Arc::new(move |_: &Progress| {
            for path in &doomed {
                if path.is_dir() {
                    let _ = fs::remove_dir_all(path);
                } else {
                    let _ = fs::remove_file(path);
                }
            }
        })),
        on_gone: Some(Arc::new(move |path: &Path| {
            told.lock().unwrap().push(path.to_path_buf());
        })),
        ..Control::default()
    };
    (control, gone)
}

#[test]
fn a_file_gone_before_it_is_read_is_left_out() {
    let s = Scratch::new("gone");
    for (kind, extension) in KINDS {
        let root = s.path(&format!("in-{extension}"));
        fs::create_dir_all(root.join("sub")).unwrap();
        for name in ["a.txt", "b.txt", "c.txt", "sub/d.txt"] {
            fs::write(root.join(name), format!("{name}\n").repeat(1000)).unwrap();
        }
        #[cfg(unix)]
        std::os::unix::fs::symlink("a.txt", root.join("link")).unwrap();
        let mut doomed = vec![root.join("b.txt")];
        if cfg!(unix) {
            doomed.push(root.join("link"));
        }
        let (control, gone) = removing(doomed.clone());
        let new = NewArchive {
            kind,
            ..NewArchive::default()
        };
        let made = create(
            std::slice::from_ref(&root),
            &s.path(&format!("a.{extension}")),
            &new,
            &control,
        )
        .unwrap_or_else(|e| panic!("{kind:?}: {e}"));
        let mut told = gone.lock().unwrap().clone();
        told.sort();
        assert_eq!(told, doomed, "{kind:?}");
        test(&made, &Keys::default(), &Control::default()).unwrap();
        let out = s.path(&format!("out-{extension}"));
        extract(&made, &out, &Keys::default(), &Control::default()).unwrap();
        let back = out.join(format!("in-{extension}"));
        assert_eq!(
            fs::read(back.join("a.txt")).unwrap(),
            fs::read(root.join("a.txt")).unwrap()
        );
        assert!(back.join("c.txt").exists(), "{kind:?}");
        assert!(back.join("sub/d.txt").exists(), "{kind:?}");
        assert!(!back.join("b.txt").exists(), "{kind:?}");
        assert!(fs::symlink_metadata(back.join("link")).is_err(), "{kind:?}");
    }
}

#[test]
fn a_folder_gone_before_it_is_read_leaves_its_files_out() {
    let s = Scratch::new("gone-folder");
    let root = s.path("in");
    fs::create_dir_all(root.join("sub")).unwrap();
    fs::write(root.join("a.txt"), "a").unwrap();
    fs::write(root.join("sub/b.txt"), "b").unwrap();
    let (control, gone) = removing(vec![root.join("sub")]);
    let new = NewArchive {
        kind: Kind::SevenZip,
        ..NewArchive::default()
    };
    let made = create(std::slice::from_ref(&root), &s.path("a.7z"), &new, &control).unwrap();
    assert_eq!(*gone.lock().unwrap(), [root.join("sub/b.txt")]);
    let out = s.path("out");
    extract(&made, &out, &Keys::default(), &Control::default()).unwrap();
    assert!(out.join("in/a.txt").exists());
    assert!(!out.join("in/sub/b.txt").exists());
}

#[test]
fn a_chosen_path_that_is_gone_is_an_error() {
    let s = Scratch::new("gone-chosen");
    let err = create(
        &[s.path("not-there")],
        &s.path("a.7z"),
        &NewArchive {
            kind: Kind::SevenZip,
            ..NewArchive::default()
        },
        &Control::default(),
    )
    .unwrap_err();
    assert!(err.to_string().contains("not-there"), "{err}");
}

/// Words and noise, `len` bytes of them: what compresses as files do.
fn words_and_noise(len: usize, seed: u64) -> Vec<u8> {
    let mut data = Vec::with_capacity(len + 256);
    let mut state = seed | 1;
    while data.len() < len {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        match state % 8 {
            0 => data.extend((0..(state % 200)).map(|i| (state >> (i % 56)) as u8)),
            n => data.extend_from_slice(
                ["packwing ", "xz ", "thread ", "block\n"][n as usize % 4].as_bytes(),
            ),
        }
    }
    data.truncate(len);
    data
}

/// xz's blocks are its own, or, small, cut so that the cores all have one,
/// from the input's size alone: the same archive whatever the threads, as
/// one thread makes it with blocks of that size.
#[test]
fn xz_goes_in_blocks_large_or_small() {
    let s = Scratch::new("xz-blocks");
    let data = words_and_noise(9 << 20, 5);
    let file = s.path("a.txt");
    fs::write(&file, &data).unwrap();
    // Level Low is preset 3, of 4 MiB dictionary: xz's own blocks are 12 MiB,
    // one for 9 MiB; small ones are a dictionary, the least, three of them.
    assert!(Blocks::differ(Kind::Xz, Level::Low, 9 << 20));
    assert!(
        !Blocks::differ(Kind::Xz, Level::Low, 4 << 20),
        "one dictionary is one block"
    );
    assert!(
        !Blocks::differ(Kind::Xz, Level::Low, 32 * (12 << 20)),
        "as many as the cores"
    );
    for (blocks, block_size) in [(Blocks::Large, 0), (Blocks::Small, 4 << 20)] {
        let new = NewArchive {
            kind: Kind::Xz,
            level: Level::Low,
            blocks,
            ..NewArchive::default()
        };
        let made = create(
            std::slice::from_ref(&file),
            &s.path(&format!("{blocks:?}.xz")),
            &new,
            &Control::default(),
        )
        .unwrap();
        let mut one = hivepack::xz::Stream::stream_encoder_mt(&hivepack::xz::MtOptions {
            threads: 1,
            preset: 3,
            block_size,
            check: hivepack::xz::Check::CRC64,
            ..hivepack::xz::MtOptions::default()
        })
        .unwrap();
        let mut want = vec![0; data.len() + (1 << 20)];
        let mut output = hivepack::xz::OutBuffer::new(&mut want);
        let mut input = hivepack::xz::InBuffer::new(&data);
        loop {
            let status = one
                .code(&mut output, &mut input, hivepack::xz::Action::Finish)
                .unwrap();
            if status == hivepack::xz::Status::StreamEnd {
                break;
            }
        }
        let n = output.pos;
        want.truncate(n);
        assert!(
            fs::read(&made).unwrap() == want,
            "{blocks:?}: not in blocks of {block_size}"
        );
    }
}

/// A tar compressed in threads, and a 7z archive, tell what is compressed,
/// not what is read: never less than before, never more than all, and all at
/// the end.
#[test]
fn a_compressed_tar_tells_what_is_compressed() {
    let s = Scratch::new("compressed-progress");
    let root = s.path("files");
    fs::create_dir(&root).unwrap();
    for i in 0..6 {
        fs::write(
            root.join(format!("{i}.txt")),
            words_and_noise(3 << 20, i + 1),
        )
        .unwrap();
    }
    let total = 6 * (3 << 20);
    for (kind, name, frames) in [
        (Kind::TarXz, "a.tar.xz", Frames::One),
        (Kind::TarZst, "a.tar.zst", Frames::One),
        (Kind::TarZst, "frames.tar.zst", Frames::Several),
        (Kind::SevenZip, "a.7z", Frames::One),
    ] {
        let told = Arc::new(std::sync::Mutex::new(Vec::new()));
        let keep = told.clone();
        let control = Control {
            progress: Some(Arc::new(move |p: &Progress| {
                keep.lock().unwrap().push((p.entries, p.bytes));
            })),
            ..Control::default()
        };
        let new = NewArchive {
            kind,
            level: Level::Medium,
            frames,
            ..NewArchive::default()
        };
        let made = create(std::slice::from_ref(&root), &s.path(name), &new, &control).unwrap();
        let told = told.lock().unwrap();
        for pair in told.windows(2) {
            assert!(pair[1].1 >= pair[0].1, "{name}: {pair:?}");
        }
        assert!(told.iter().all(|&(_, bytes)| bytes <= total), "{name}");
        assert_eq!(told.last().map(|&(_, bytes)| bytes), Some(total), "{name}");
        // When the last file is read, its threads have not compressed it all:
        // a block of xz's, a job of zstd's, a frame in flight, or the piece
        // not yet a frame.
        let all_read = told.iter().find(|&&(entries, _)| entries == 6).unwrap();
        assert!(all_read.1 < total, "{name}: {all_read:?} when all was read");
        let out = s.path(&format!("{name}-out"));
        extract_with(
            &made,
            &out,
            &Keys::default(),
            &ExtractOptions::default(),
            &Control::default(),
        )
        .unwrap();
        assert_eq!(
            fs::read(out.join("files/5.txt")).unwrap(),
            words_and_noise(3 << 20, 6),
            "{name}"
        );
    }
}

/// Stopped once every file is read, while its threads compress what they
/// hold, a compressed tar or a 7z archive stops there, and leaves nothing.
#[test]
fn stopped_while_the_threads_end_leaves_nothing() {
    let s = Scratch::new("stopped-ending");
    let root = s.path("files");
    fs::create_dir(&root).unwrap();
    for i in 0..3 {
        fs::write(
            root.join(format!("{i}.txt")),
            words_and_noise(2 << 20, i + 1),
        )
        .unwrap();
    }
    for (kind, name, frames) in [
        (Kind::TarXz, "a.tar.xz", Frames::One),
        (Kind::TarZst, "a.tar.zst", Frames::One),
        (Kind::TarZst, "frames.tar.zst", Frames::Several),
        (Kind::SevenZip, "a.7z", Frames::One),
    ] {
        let cancel = Cancel::new();
        let stop = cancel.clone();
        let control = Control {
            progress: Some(Arc::new(move |p: &Progress| {
                if p.entries == 3 {
                    stop.cancel();
                }
            })),
            cancel: Some(cancel),
            ..Control::default()
        };
        let new = NewArchive {
            kind,
            level: Level::Medium,
            frames,
            ..NewArchive::default()
        };
        let path = s.path(name);
        let stopped = create(std::slice::from_ref(&root), &path, &new, &control);
        assert!(
            matches!(stopped, Err(Error::Cancelled)),
            "{name}: {stopped:?}"
        );
        assert!(!path.exists(), "{name}");
    }
}

/// A ZIP archive's large file is deflated in blocks, as pigz does, on every
/// core: its stream is what one thread makes of it in blocks.
#[test]
fn a_large_file_in_zip_is_deflated_in_blocks() {
    let s = Scratch::new("zip-blocks");
    let data = words_and_noise(40 << 20, 3);
    let file = s.path("big.txt");
    fs::write(&file, &data).unwrap();
    let made = create(
        &[file],
        &s.path("a.zip"),
        &NewArchive::default(),
        &Control::default(),
    )
    .unwrap();
    assert!(
        compressed_entry(&made, 0) == in_blocks(&data),
        "not deflated in blocks"
    );
    let out = s.path("out");
    extract_with(
        &made,
        &out,
        &Keys::default(),
        &ExtractOptions::default(),
        &Control::default(),
    )
    .unwrap();
    assert!(fs::read(out.join("big.txt")).unwrap() == data);
}

/// A 7z archive's LZMA2 blocks are 7-Zip's own, or, small, enough for every
/// core, from the input's size alone: the same archive whatever the threads,
/// and both read back.
#[test]
fn seven_zip_goes_in_blocks_large_or_small() {
    let s = Scratch::new("7z-blocks");
    let data = words_and_noise(9 << 20, 6);
    let file = s.path("a.txt");
    fs::write(&file, &data).unwrap();
    // Level Low is preset 3, of 4 MiB dictionary: 7-Zip's own blocks are
    // 16 MiB, one for 9 MiB; small ones are a dictionary, three of them.
    assert!(Blocks::differ(Kind::SevenZip, Level::Low, 9 << 20));
    assert!(!Blocks::differ(Kind::SevenZip, Level::Low, 4 << 20));
    assert!(!Blocks::differ(Kind::SevenZip, Level::Low, 32 * (16 << 20)));
    assert!(!Blocks::differ(Kind::SevenZip, Level::Store, 9 << 20));
    assert!(!Blocks::differ(Kind::Zip, Level::Low, 9 << 20));
    let made = |blocks: Blocks, threads: u32, name: &str| {
        packwing_core::use_threads(threads);
        let new = NewArchive {
            kind: Kind::SevenZip,
            level: Level::Low,
            blocks,
            ..NewArchive::default()
        };
        let made = create(
            std::slice::from_ref(&file),
            &s.path(name),
            &new,
            &Control::default(),
        );
        packwing_core::use_threads(0);
        fs::read(made.unwrap()).unwrap()
    };
    let large = made(Blocks::Large, 4, "large.7z");
    let small = made(Blocks::Small, 4, "small.7z");
    assert!(small != large, "small blocks are other blocks");
    assert!(
        made(Blocks::Small, 1, "small-one.7z") == small,
        "whatever the threads"
    );
    for name in ["large.7z", "small.7z"] {
        let out = s.path(&format!("{name}-out"));
        extract_with(
            s.path(name),
            &out,
            &Keys::default(),
            &ExtractOptions::default(),
            &Control::default(),
        )
        .unwrap();
        assert!(fs::read(out.join("a.txt")).unwrap() == data, "{name}");
    }
}

/// Entry `i` of the ZIP archive at `zip`, as it is stored, compressed.
fn compressed_entry(zip: &Path, i: u64) -> Vec<u8> {
    let za = hivepack::zip::Archive::open(
        fs::File::open(zip).unwrap(),
        hivepack::zip::OpenFlags::RDONLY,
    )
    .unwrap();
    let mut stream = Vec::new();
    io::copy(
        &mut za.fopen_index(i, hivepack::zip::Flags::COMPRESSED).unwrap(),
        &mut stream,
    )
    .unwrap();
    stream
}

/// `data` deflated at level 6 in blocks, as pigz does, raw: as packwing
/// deflates a ZIP archive's files.
fn in_blocks(data: &[u8]) -> Vec<u8> {
    let mut one = hivepack::deflate::DeflateMt::new(&hivepack::deflate::MtOptions {
        level: 6,
        raw: true,
        ..hivepack::deflate::MtOptions::default()
    })
    .unwrap();
    let mut want = vec![0; data.len() + (1 << 20)];
    let mut output = hivepack::deflate::OutBuffer::new(&mut want);
    let mut input = hivepack::deflate::InBuffer::new(data);
    let status = one
        .deflate(&mut output, &mut input, hivepack::deflate::Flush::Finish)
        .unwrap();
    assert_eq!(status, hivepack::deflate::Status::StreamEnd);
    let n = output.pos;
    want.truncate(n);
    want
}

/// A large file added to a ZIP archive is deflated in blocks, as one in a
/// new archive is, on every core; what was there stays as it was.
#[test]
fn a_large_file_added_to_a_zip_is_deflated_in_blocks() {
    let s = Scratch::new("zip-add-blocks");
    let small = s.path("small.txt");
    fs::write(&small, "kept\n").unwrap();
    let made = create(
        &[small],
        &s.path("a.zip"),
        &NewArchive::default(),
        &Control::default(),
    )
    .unwrap();
    let kept = compressed_entry(&made, 0);
    let data = words_and_noise(40 << 20, 4);
    let big = s.path("big.txt");
    fs::write(&big, &data).unwrap();
    add_with(
        &[big],
        &made,
        &[],
        OnExisting::Stop,
        &[],
        &Keys::default(),
        &Control::default(),
    )
    .unwrap();
    assert!(compressed_entry(&made, 0) == kept, "what was there changed");
    assert!(
        compressed_entry(&made, 1) == in_blocks(&data),
        "not deflated in blocks"
    );
    let out = s.path("out");
    extract_with(
        &made,
        &out,
        &Keys::default(),
        &ExtractOptions::default(),
        &Control::default(),
    )
    .unwrap();
    assert!(fs::read(out.join("big.txt")).unwrap() == data);
    assert_eq!(fs::read_to_string(out.join("small.txt")).unwrap(), "kept\n");
}

/// LZ4 goes in independent blocks of 4 MB, each compressed in a thread, with
/// a checksum of the content, as `lz4` writes it: the frame one thread makes
/// with those preferences, byte for byte.
#[test]
fn lz4_goes_in_blocks_compressed_in_threads() {
    use hivepack::lz4::frame::{
        BlockMode, BlockSizeId, FrameInfo, Preferences, compress_frame, compress_frame_bound,
    };
    let s = Scratch::new("lz4-threads");
    let data = words_and_noise((9 << 20) + 777, 8);
    let file = s.path("a.txt");
    fs::write(&file, &data).unwrap();
    for (level, lz4_level) in [(Level::Medium, 1), (Level::High, 9)] {
        let new = NewArchive {
            kind: Kind::Lz4,
            level,
            ..NewArchive::default()
        };
        let made = create(
            std::slice::from_ref(&file),
            &s.path(&format!("{lz4_level}.lz4")),
            &new,
            &Control::default(),
        )
        .unwrap();
        let prefs = Preferences {
            frame_info: FrameInfo {
                block_size_id: BlockSizeId::Max4Mb,
                block_mode: BlockMode::Independent,
                content_checksum: true,
                ..FrameInfo::default()
            },
            compression_level: lz4_level,
            ..Preferences::default()
        };
        let mut want = vec![0; compress_frame_bound(data.len(), Some(&prefs))];
        let n = compress_frame(&mut want, &data, Some(&prefs)).unwrap();
        want.truncate(n);
        assert!(fs::read(&made).unwrap() == want, "{level:?}");
        let out = s.path(&format!("{lz4_level}-out"));
        extract_with(
            &made,
            &out,
            &Keys::default(),
            &ExtractOptions::default(),
            &Control::default(),
        )
        .unwrap();
        // A frame has no name: the file is the archive's, without `.lz4`.
        let back = fs::read(out.join(lz4_level.to_string())).unwrap();
        assert!(back == data, "{level:?}");
    }
}

/// A compressed tar into a folder is read once: until its entries are checked,
/// how far it is, is told from the archive's bytes read, against its size;
/// what is told never goes back, ends at all of it, and no private folder is
/// left.
#[test]
fn a_compressed_tar_extracted_tells_how_far_through_the_archive() {
    let s = Scratch::new("tar-once-progress");
    let root = s.path("files");
    fs::create_dir(&root).unwrap();
    for i in 0..4 {
        fs::write(
            root.join(format!("{i}.txt")),
            words_and_noise(2 << 20, i + 20),
        )
        .unwrap();
    }
    let new = NewArchive {
        kind: Kind::TarGz,
        ..NewArchive::default()
    };
    let made = create(
        std::slice::from_ref(&root),
        &s.path("a.tar.gz"),
        &new,
        &Control::default(),
    )
    .unwrap();
    let size = fs::metadata(&made).unwrap().len();
    let told = Arc::new(std::sync::Mutex::new(Vec::new()));
    let keep = told.clone();
    let control = Control {
        progress: Some(Arc::new(move |p: &Progress| {
            keep.lock().unwrap().push((p.bytes, p.total_bytes));
        })),
        ..Control::default()
    };
    let out = s.path("out");
    extract_with(
        &made,
        &out,
        &Keys::default(),
        &ExtractOptions::default(),
        &control,
    )
    .unwrap();
    let told = told.lock().unwrap();
    assert!(told.iter().any(|&(_, total)| total == size), "{told:?}");
    let fractions: Vec<f64> = told
        .iter()
        .filter(|&&(_, total)| total > 0)
        .map(|&(bytes, total)| bytes as f64 / total as f64)
        .collect();
    assert!(fractions.windows(2).all(|w| w[1] >= w[0]), "{fractions:?}");
    let all = 4 * (2 << 20);
    assert_eq!(told.last(), Some(&(all, all)));
    let left: Vec<_> = fs::read_dir(&out)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(left, ["files"]);
}

/// "Extract selected" writes into a private folder of packwing's own, with a
/// locked `owner`, and moves what was picked out of it: none is left after,
/// and one a process left behind there, which no one holds, is removed first.
#[test]
fn extract_selected_leaves_no_private_folder_and_removes_one_left() {
    let s = Scratch::new("selected-private");
    let root = s.path("files");
    fs::create_dir(&root).unwrap();
    fs::write(root.join("a.txt"), "a").unwrap();
    fs::write(root.join("b.txt"), "b").unwrap();
    let made = create(
        std::slice::from_ref(&root),
        &s.path("a.zip"),
        &NewArchive::default(),
        &Control::default(),
    )
    .unwrap();
    let out = s.path("out");
    let left = out.join(".packwing-extracting-7");
    fs::create_dir_all(left.join("files")).unwrap();
    fs::write(left.join("owner"), "").unwrap();
    packwing_core::extract_selected(
        &made,
        &out,
        &[vec!["files".into(), "a.txt".into()]],
        &Keys::default(),
        &ExtractOptions::default(),
        &Control::default(),
    )
    .unwrap();
    let mut names: Vec<_> = fs::read_dir(&out)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(names, ["a.txt"]);
}
