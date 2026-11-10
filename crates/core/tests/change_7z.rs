//! Changing a 7z archive: files added to it and deleted from it, through
//! hivepack's `Writer::copy_from`, written beside it and put in its place. Its
//! folders that keep all their files are copied as they are; what is written
//! again, and what is added, is compressed as the archive says it was. Where
//! `7zz` or `7z` is installed, it tests what comes out.

#![allow(clippy::unwrap_used)]

use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use hivepack::sevenz::{
    Archive, Coding, Compression, Filter, Method, NewEntry, Options, WriteOptions, Writer,
};
use packwing_core::{
    COMPRESSED_EXTENSIONS, Cancel, Control, Error, Keys, Kind, NewArchive, OnExisting, Progress,
    add, add_with, can_add, can_rename, create, delete, extract, list, rename, test,
};

/// A folder of a test's own, gone when it ends.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Scratch {
        let dir = std::env::temp_dir().join(format!("packwing-7z-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        Scratch(dir)
    }

    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }

    /// `Báo cáo/`: a note, an empty file, an empty folder, 3 MB in a folder,
    /// and a `.zip` of more than a MiB, which packwing stores in a folder of
    /// its own.
    fn tree(&self) -> PathBuf {
        let root = self.path("Báo cáo");
        fs::create_dir_all(root.join("Hình ảnh")).unwrap();
        fs::create_dir_all(root.join("trống")).unwrap();
        fs::write(root.join("ghi chú.txt"), "xin chào\n").unwrap();
        fs::write(root.join("rỗng"), "").unwrap();
        fs::write(root.join("Hình ảnh/lớn.bin"), big(3_000_000)).unwrap();
        fs::write(root.join("đã nén.zip"), big(1_100_000)).unwrap();
        root
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// `len` bytes that compress a little, the same every time.
fn big(len: usize) -> Vec<u8> {
    let mut state = 0x2545_f491_u32;
    (0..len)
        .map(|i| {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            if i % 3 == 0 {
                b'a'
            } else {
                (state >> 24) as u8
            }
        })
        .collect()
}

fn keys(password: Option<&str>) -> Keys {
    Keys {
        password: password.map(|p| p.as_bytes().to_vec()),
        ..Keys::default()
    }
}

/// A 7z archive packwing makes of the tree, with `password`, its names
/// hidden too if `hide_names`.
fn made(s: &Scratch, password: Option<&str>, hide_names: bool) -> PathBuf {
    let new = NewArchive {
        kind: Kind::SevenZip,
        password: password.map(|p| p.as_bytes().to_vec()),
        hide_names,
        stored_extensions: COMPRESSED_EXTENSIONS
            .iter()
            .map(|e| (*e).to_owned())
            .collect(),
        ..NewArchive::default()
    };
    create(&[s.tree()], &s.path("a.7z"), &new, &Control::default()).unwrap()
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

fn path_of(text: &str) -> Vec<String> {
    text.split('/').map(str::to_owned).collect()
}

/// The archive at `path`, opened by hivepack.
fn opened(path: &Path, password: Option<&str>) -> Archive<Cursor<Vec<u8>>> {
    let options = Options {
        password: password.map(str::to_owned),
        ..Options::default()
    };
    Archive::open_with(Cursor::new(fs::read(path).unwrap()), &options).unwrap()
}

/// Each folder's packed bytes, by the names of its files.
fn packed(path: &Path, password: Option<&str>) -> Vec<(Vec<String>, u64)> {
    let archive = opened(path, password);
    (0..archive.folders())
        .map(|f| {
            let names = archive
                .entries()
                .filter(|e| e.folder == Some(f))
                .map(|e| e.name)
                .collect();
            (names, archive.folder_packed_size(f))
        })
        .collect()
}

/// The coding of the folder that holds `name`.
fn coding_of(path: &Path, name: &str) -> Coding {
    let archive = opened(path, None);
    let entry = archive.entries().find(|e| e.name == name).unwrap();
    archive.folder_coding(entry.folder.unwrap()).unwrap()
}

/// Nothing beside the archive but what the test made: no `.a.7z.adding`.
fn nothing_left_beside(s: &Scratch) {
    for entry in fs::read_dir(&s.0).unwrap() {
        let name = entry.unwrap().file_name();
        assert!(!name.to_string_lossy().contains("adding"), "{name:?}");
    }
}

/// `7zz t`, or `7z t`, where one is installed, finds the archive whole.
fn seven_zip_tests(path: &Path, password: Option<&str>) {
    for tool in ["7zz", "7z"] {
        let Ok(out) = Command::new(tool)
            .args(["t", "-bd"])
            .arg(format!("-p{}", password.unwrap_or("")))
            .arg(path)
            .output()
        else {
            continue;
        };
        let text = String::from_utf8_lossy(&out.stdout);
        assert!(out.status.success(), "{tool} t:\n{text}");
        assert!(text.contains("Everything is Ok"), "{tool} t:\n{text}");
        return;
    }
}

#[test]
fn a_7z_archive_of_one_file_takes_more_and_gives_up_some() {
    let s = Scratch::new("can");
    let archive = made(&s, None, false);
    let listing = list(&archive, &Keys::default()).unwrap();
    assert!(can_add(&listing));
    assert!(can_rename(&listing));
    // In parts, it does not.
    let parts = NewArchive {
        kind: Kind::SevenZip,
        part_size: Some(1_000_000),
        ..NewArchive::default()
    };
    let first = create(
        &[s.path("Báo cáo")],
        &s.path("p.7z"),
        &parts,
        &Control::default(),
    )
    .unwrap();
    assert!(!can_add(&list(&first, &Keys::default()).unwrap()));
}

#[test]
fn added_files_join_what_is_there_whose_folders_are_copied() {
    let s = Scratch::new("add");
    let archive = made(&s, None, false);
    let before = packed(&archive, None);
    let (one, folder) = (s.path("mới.txt"), s.path("thêm"));
    fs::write(&one, "mới").unwrap();
    fs::create_dir_all(folder.join("trong")).unwrap();
    fs::write(folder.join("trong/x.txt"), "x").unwrap();
    let none = Keys::default();
    add(&[one, folder], &archive, &[], &none, &Control::default()).unwrap();
    let two = s.path("two.txt");
    fs::write(&two, "2").unwrap();
    let inner = path_of("Báo cáo/Hình ảnh");
    add(&[two], &archive, &inner, &none, &Control::default()).unwrap();

    assert_eq!(
        names_in(&archive, &none),
        [
            "Báo cáo/Hình ảnh/lớn.bin",
            "Báo cáo/Hình ảnh/two.txt",
            "Báo cáo/ghi chú.txt",
            "Báo cáo/rỗng",
            "Báo cáo/đã nén.zip",
            "mới.txt",
            "thêm/trong/x.txt",
        ]
    );
    // What was there is copied as it was, its folders first.
    let after = packed(&archive, None);
    assert_eq!(after[..before.len()], before[..]);
    test(&archive, &none, &Control::default()).unwrap();
    let out = s.path("out");
    extract(&archive, &out, &none, &Control::default()).unwrap();
    assert_eq!(
        fs::read(out.join("Báo cáo/Hình ảnh/lớn.bin")).unwrap(),
        big(3_000_000)
    );
    assert_eq!(fs::read_to_string(out.join("mới.txt")).unwrap(), "mới");
    assert_eq!(
        fs::read_to_string(out.join("thêm/trong/x.txt")).unwrap(),
        "x"
    );
    assert_eq!(
        fs::read_to_string(out.join("Báo cáo/Hình ảnh/two.txt")).unwrap(),
        "2"
    );
    assert!(out.join("Báo cáo/trống").is_dir());
    nothing_left_beside(&s);
    seven_zip_tests(&archive, None);
}

#[test]
fn a_name_it_holds_is_replaced_skipped_or_stops_it() {
    let s = Scratch::new("on-existing");
    let archive = made(&s, None, false);
    let source = s.path("nguồn/Báo cáo");
    fs::create_dir_all(&source).unwrap();
    fs::write(source.join("ghi chú.txt"), "mới").unwrap();
    fs::write(source.join("thêm.txt"), "thêm").unwrap();
    let none = Keys::default();
    let control = Control::default();
    let note = |name: &str| {
        let out = s.path(name);
        extract(&archive, &out, &none, &control).unwrap();
        fs::read_to_string(out.join("Báo cáo/ghi chú.txt")).unwrap()
    };
    let before = fs::read(&archive).unwrap();
    assert!(
        add(
            std::slice::from_ref(&source),
            &archive,
            &[],
            &none,
            &control
        )
        .is_err()
    );
    assert_eq!(fs::read(&archive).unwrap(), before);
    let with = |on_existing| {
        add_with(
            std::slice::from_ref(&source),
            &archive,
            &[],
            on_existing,
            &[],
            &none,
            &control,
        )
    };
    with(OnExisting::Skip).unwrap();
    assert_eq!(note("skipped"), "xin chào\n");
    assert_eq!(list(&archive, &none).unwrap().files, 5);
    with(OnExisting::Replace).unwrap();
    assert_eq!(note("replaced"), "mới");
    assert_eq!(list(&archive, &none).unwrap().files, 5, "no name twice");
    test(&archive, &none, &control).unwrap();
    nothing_left_beside(&s);
}

#[test]
fn entries_are_deleted_their_folder_written_again() {
    let s = Scratch::new("delete");
    let archive = made(&s, None, false);
    let none = Keys::default();
    let control = Control::default();
    // From the solid folder: written again; the stored one copied.
    let stored = |path: &Path| {
        packed(path, None)
            .into_iter()
            .find(|(names, _)| names.iter().any(|n| n.ends_with(".zip")))
            .unwrap()
    };
    let kept = stored(&archive);
    delete(&archive, &[path_of("Báo cáo/ghi chú.txt")], &none, &control).unwrap();
    assert_eq!(stored(&archive), kept);
    delete(&archive, &[path_of("Báo cáo/Hình ảnh")], &none, &control).unwrap();
    assert_eq!(
        names_in(&archive, &none),
        ["Báo cáo/rỗng", "Báo cáo/đã nén.zip"]
    );
    test(&archive, &none, &control).unwrap();
    seven_zip_tests(&archive, None);

    let before = fs::read(&archive).unwrap();
    assert!(delete(&archive, &[path_of("không có")], &none, &control).is_err());
    assert!(
        delete(&archive, &[path_of("Báo cáo")], &none, &control).is_err(),
        "all of it"
    );
    assert_eq!(fs::read(&archive).unwrap(), before);
    nothing_left_beside(&s);
}

#[test]
fn an_encrypted_archive_takes_its_own_password() {
    for hide_names in [false, true] {
        let s = Scratch::new(&format!("password-{hide_names}"));
        let archive = made(&s, Some("mật khẩu"), hide_names);
        let right = keys(Some("mật khẩu"));
        let extra = s.path("thêm.txt");
        fs::write(&extra, "thêm").unwrap();
        let before = fs::read(&archive).unwrap();
        let control = Control::default();

        let added = add(
            std::slice::from_ref(&extra),
            &archive,
            &[],
            &Keys::default(),
            &control,
        );
        assert!(matches!(added, Err(Error::Locked { .. })), "{added:?}");
        let added = add(
            std::slice::from_ref(&extra),
            &archive,
            &[],
            &keys(Some("sai")),
            &control,
        );
        assert!(matches!(added, Err(Error::WrongPassword)), "{added:?}");
        assert_eq!(fs::read(&archive).unwrap(), before);

        add(
            std::slice::from_ref(&extra),
            &archive,
            &[],
            &right,
            &control,
        )
        .unwrap();
        let back = opened(&archive, Some("mật khẩu"));
        assert_eq!(back.headers_encrypted(), hide_names);
        let added = back.entries().find(|e| e.name == "thêm.txt").unwrap();
        assert!(added.encrypted, "what is added is encrypted as the rest");
        if hide_names {
            assert!(matches!(
                list(&archive, &Keys::default()),
                Err(Error::Locked { .. })
            ));
        }
        // A file taken out of an encrypted solid folder: it is decrypted and
        // encrypted again.
        delete(
            &archive,
            &[path_of("Báo cáo/ghi chú.txt")],
            &right,
            &control,
        )
        .unwrap();
        let out = s.path("out");
        extract(&archive, &out, &right, &control).unwrap();
        assert_eq!(
            fs::read(out.join("Báo cáo/Hình ảnh/lớn.bin")).unwrap(),
            big(3_000_000)
        );
        assert_eq!(fs::read_to_string(out.join("thêm.txt")).unwrap(), "thêm");
        assert!(!out.join("Báo cáo/ghi chú.txt").exists());
        seven_zip_tests(&archive, Some("mật khẩu"));
        nothing_left_beside(&s);
    }
}

#[test]
fn stopping_leaves_the_archive_as_it_was() {
    let s = Scratch::new("stop");
    let archive = made(&s, None, false);
    let before = fs::read(&archive).unwrap();
    let cancel = Cancel::new();
    let stop = cancel.clone();
    let control = Control {
        // Once it has written something.
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
    let deleted = delete(
        &archive,
        &[path_of("Báo cáo/ghi chú.txt")],
        &Keys::default(),
        &control,
    );
    assert!(matches!(deleted, Err(Error::Cancelled)), "{deleted:?}");
    assert_eq!(fs::read(&archive).unwrap(), before);
    nothing_left_beside(&s);
}

/// An archive hivepack writes of three files of `options`, each of `size`,
/// with a stored one besides, as packwing's own archives have, where there
/// is no filter.
fn written(path: &Path, options: WriteOptions, size: usize) {
    let filtered = options.filter != Filter::None;
    let mut writer = Writer::with_options(Cursor::new(Vec::new()), options).unwrap();
    for name in ["một", "hai", "ba"] {
        writer
            .start_file(NewEntry {
                name: name.to_owned(),
                attributes: None,
                mtime: None,
                ctime: None,
            })
            .unwrap();
        writer.write_all(&big(size)).unwrap();
    }
    if filtered {
        fs::write(path, writer.finish().unwrap().into_inner()).unwrap();
        return;
    }
    writer.set_method(Method::Copy).unwrap();
    writer
        .start_file(NewEntry {
            name: "đã nén.zip".to_owned(),
            attributes: None,
            mtime: None,
            ctime: None,
        })
        .unwrap();
    writer.write_all(&big(1000)).unwrap();
    fs::write(path, writer.finish().unwrap().into_inner()).unwrap();
}

/// An archive's options, its files' size, and what is added is compressed with:
/// method, filter, dictionary.
type Case = (
    &'static str,
    WriteOptions,
    usize,
    Compression,
    Filter,
    Option<u32>,
);

#[test]
fn what_is_added_is_compressed_as_the_archive_was() {
    let s = Scratch::new("like");
    let extra = s.path("thêm.txt");
    fs::write(&extra, big(200_000)).unwrap();
    let none = Keys::default();
    let lzma2 = |preset| Method::Lzma2 { preset };
    // packwing's own: LZMA2 of xz's preset 6, of 8 MiB.
    let own = Some(8 << 20);
    let cases: [Case; 6] = [
        // A dictionary smaller than its folder is what was asked for: kept.
        (
            "dictionary",
            WriteOptions {
                method: Method::Lzma { preset: 6 },
                dictionary: Some(64 << 10),
                ..WriteOptions::default()
            },
            300_000,
            Compression::Lzma,
            Filter::None,
            Some(64 << 10),
        ),
        // As large as its folder, 7-Zip may have cut it: packwing's own.
        (
            "cut",
            WriteOptions {
                method: lzma2(6),
                dictionary: Some(1 << 20),
                ..WriteOptions::default()
            },
            1000,
            Compression::Lzma2,
            Filter::None,
            own,
        ),
        (
            "filter",
            WriteOptions {
                method: lzma2(6),
                filter: Filter::Arm64,
                ..WriteOptions::default()
            },
            1000,
            Compression::Lzma2,
            Filter::Arm64,
            own,
        ),
        (
            "bzip2",
            WriteOptions {
                method: Method::BZip2 { level: 1 },
                ..WriteOptions::default()
            },
            1000,
            Compression::BZip2,
            Filter::None,
            None,
        ),
        // Stored only: packwing's own.
        (
            "stored",
            WriteOptions {
                method: Method::Copy,
                ..WriteOptions::default()
            },
            1000,
            Compression::Lzma2,
            Filter::None,
            own,
        ),
        (
            "deflate",
            WriteOptions {
                method: Method::Deflate { level: 9 },
                ..WriteOptions::default()
            },
            1000,
            Compression::Deflate,
            Filter::None,
            None,
        ),
    ];
    for (name, options, size, method, filter, dictionary) in cases {
        let archive = s.path(&format!("{name}.7z"));
        written(&archive, options, size);
        add(
            std::slice::from_ref(&extra),
            &archive,
            &[],
            &none,
            &Control::default(),
        )
        .unwrap();
        let coding = coding_of(&archive, "thêm.txt");
        assert_eq!(coding.method, method, "{name}");
        assert_eq!(coding.filter, filter, "{name}");
        assert_eq!(coding.dictionary, dictionary, "{name}");
        test(&archive, &none, &Control::default()).unwrap();
    }

    // With a filter kept, what is compressed already is stored apart still,
    // in a folder of its own with none.
    let filtered = s.path("filter.7z");
    let zip = s.path("đã nén.zip");
    fs::write(&zip, big(1 << 20)).unwrap();
    let stored: Vec<String> = COMPRESSED_EXTENSIONS
        .iter()
        .map(|e| (*e).to_owned())
        .collect();
    add_with(
        std::slice::from_ref(&zip),
        &filtered,
        &[],
        OnExisting::Replace,
        &stored,
        &none,
        &Control::default(),
    )
    .unwrap();
    let zip = coding_of(&filtered, "đã nén.zip");
    assert_eq!((zip.method, zip.filter), (Compression::Copy, Filter::None));
    test(&filtered, &none, &Control::default()).unwrap();

    // Each file a folder of its own: so are those added.
    let alone = s.path("alone.7z");
    let options = WriteOptions {
        solid_block_size: None,
        ..WriteOptions::default()
    };
    written(&alone, options, 1000);
    let second = s.path("hai.txt");
    fs::write(&second, "hai").unwrap();
    add(
        &[extra.clone(), second],
        &alone,
        &[],
        &none,
        &Control::default(),
    )
    .unwrap();
    let back = opened(&alone, None);
    let folder = |name: &str| back.entries().find(|e| e.name == name).unwrap().folder;
    assert_ne!(folder("thêm.txt"), folder("hai.txt"));

    // Folders that disagree: packwing's own.
    let mixed = s.path("mixed.7z");
    let mut writer = Writer::with_options(
        Cursor::new(Vec::new()),
        WriteOptions {
            method: Method::BZip2 { level: 1 },
            solid_block_size: None,
            ..WriteOptions::default()
        },
    )
    .unwrap();
    for (name, method) in [
        ("a", Method::BZip2 { level: 1 }),
        ("b", Method::Deflate { level: 6 }),
    ] {
        writer.set_method(method).unwrap();
        writer
            .start_file(NewEntry {
                name: name.to_owned(),
                attributes: None,
                mtime: None,
                ctime: None,
            })
            .unwrap();
        writer.write_all(&big(1000)).unwrap();
    }
    fs::write(&mixed, writer.finish().unwrap().into_inner()).unwrap();
    add(
        std::slice::from_ref(&extra),
        &mixed,
        &[],
        &none,
        &Control::default(),
    )
    .unwrap();
    let coding = coding_of(&mixed, "thêm.txt");
    assert_eq!(coding.method, Compression::Lzma2);
    assert_eq!(coding.dictionary, own);
    nothing_left_beside(&s);
}

/// As 7-Zip chooses filters by file: programs with BCJ in a folder, the rest
/// without. A folder written again keeps its own; what is added, where the
/// folders disagree, takes none.
#[test]
fn a_folder_written_again_is_compressed_as_it_was() {
    let s = Scratch::new("folder-like");
    let archive = s.path("programs.7z");
    let mut writer = Writer::with_options(
        Cursor::new(Vec::new()),
        WriteOptions {
            filter: Filter::X86,
            dictionary: Some(256 << 10),
            ..WriteOptions::default()
        },
    )
    .unwrap();
    let add_file = |writer: &mut Writer<_>, name: &str| {
        writer
            .start_file(NewEntry {
                name: name.to_owned(),
                attributes: None,
                mtime: None,
                ctime: None,
            })
            .unwrap();
        writer.write_all(&big(200_000)).unwrap();
    };
    for name in ["bin/a", "bin/b", "bin/c"] {
        add_file(&mut writer, name);
    }
    writer
        .set_compression(Method::Lzma { preset: 6 }, Filter::None, Some(1 << 20))
        .unwrap();
    for name in ["doc/x", "doc/y"] {
        add_file(&mut writer, name);
    }
    fs::write(&archive, writer.finish().unwrap().into_inner()).unwrap();
    let none = Keys::default();

    delete(&archive, &[path_of("bin/b")], &none, &Control::default()).unwrap();
    let coding = coding_of(&archive, "bin/a");
    assert_eq!(
        (coding.method, coding.filter, coding.dictionary),
        (Compression::Lzma2, Filter::X86, Some(256 << 10))
    );
    let doc = coding_of(&archive, "doc/x");
    assert_eq!(
        (doc.method, doc.dictionary),
        (Compression::Lzma, Some(1 << 20))
    );
    delete(&archive, &[path_of("doc/y")], &none, &Control::default()).unwrap();
    let doc = coding_of(&archive, "doc/x");
    assert_eq!(
        (doc.method, doc.filter, doc.dictionary),
        (Compression::Lzma, Filter::None, Some(1 << 20))
    );

    let extra = s.path("thêm.txt");
    fs::write(&extra, big(1000)).unwrap();
    add(
        std::slice::from_ref(&extra),
        &archive,
        &[],
        &none,
        &Control::default(),
    )
    .unwrap();
    let added = coding_of(&archive, "thêm.txt");
    assert_eq!(
        (added.method, added.filter),
        (Compression::Lzma2, Filter::None)
    );
    assert_eq!(
        names_in(&archive, &none),
        ["bin/a", "bin/c", "doc/x", "thêm.txt"]
    );
    test(&archive, &none, &Control::default()).unwrap();
    seven_zip_tests(&archive, None);
}

#[test]
fn entries_are_renamed_their_folders_copied_as_they_are() {
    let s = Scratch::new("rename");
    let archive = made(&s, None, false);
    let none = Keys::default();
    let control = Control::default();
    let before = packed(&archive, None);
    rename(
        &archive,
        &path_of("Báo cáo/ghi chú.txt"),
        "đã đổi.txt",
        &none,
        &control,
    )
    .unwrap();
    // A folder, with what is in it, and an empty one.
    rename(
        &archive,
        &path_of("Báo cáo/Hình ảnh"),
        "Ảnh",
        &none,
        &control,
    )
    .unwrap();
    rename(
        &archive,
        &path_of("Báo cáo/trống"),
        "rỗng tuếch",
        &none,
        &control,
    )
    .unwrap();
    assert_eq!(
        names_in(&archive, &none),
        [
            "Báo cáo/rỗng",
            "Báo cáo/đã nén.zip",
            "Báo cáo/đã đổi.txt",
            "Báo cáo/Ảnh/lớn.bin",
        ]
    );
    let sizes = |p: &[(Vec<String>, u64)]| p.iter().map(|(_, s)| *s).collect::<Vec<_>>();
    assert_eq!(
        sizes(&packed(&archive, None)),
        sizes(&before),
        "copied whole"
    );
    let back = opened(&archive, None);
    let dir = back
        .entries()
        .find(|e| e.name == "Báo cáo/rỗng tuếch")
        .unwrap();
    assert!(dir.is_dir);
    let out = s.path("out");
    extract(&archive, &out, &none, &control).unwrap();
    assert_eq!(
        fs::read(out.join("Báo cáo/Ảnh/lớn.bin")).unwrap(),
        big(3_000_000)
    );
    seven_zip_tests(&archive, None);

    let before = fs::read(&archive).unwrap();
    for bad in ["", "a/b", "a\\b", "..", "rỗng"] {
        assert!(
            rename(
                &archive,
                &path_of("Báo cáo/đã đổi.txt"),
                bad,
                &none,
                &control
            )
            .is_err(),
            "{bad:?}"
        );
    }
    assert!(rename(&archive, &path_of("không có"), "x", &none, &control).is_err());
    assert_eq!(fs::read(&archive).unwrap(), before);
    nothing_left_beside(&s);
}

#[test]
fn an_encrypted_archive_is_renamed_in_with_no_password_but_for_its_names() {
    for hide_names in [false, true] {
        let s = Scratch::new(&format!("rename-password-{hide_names}"));
        let archive = made(&s, Some("mật khẩu"), hide_names);
        let right = keys(Some("mật khẩu"));
        let control = Control::default();
        let from = path_of("Báo cáo/ghi chú.txt");
        if hide_names {
            let renamed = rename(&archive, &from, "x.txt", &Keys::default(), &control);
            assert!(matches!(renamed, Err(Error::Locked { .. })), "{renamed:?}");
            rename(&archive, &from, "x.txt", &right, &control).unwrap();
            assert!(opened(&archive, Some("mật khẩu")).headers_encrypted());
        } else {
            // Its data is copied as it is: no password.
            rename(&archive, &from, "x.txt", &Keys::default(), &control).unwrap();
        }
        let out = s.path("out");
        extract(&archive, &out, &right, &control).unwrap();
        assert_eq!(
            fs::read_to_string(out.join("Báo cáo/x.txt")).unwrap(),
            "xin chào\n"
        );
        seven_zip_tests(&archive, Some("mật khẩu"));
        nothing_left_beside(&s);
    }
}

/// A new 7z archive at each level takes 7-Zip's dictionary at its level, and
/// its programs go through their branch converter in a folder of their own.
#[test]
fn a_new_archive_compresses_as_seven_zip_does_at_its_level() {
    let s = Scratch::new("levels");
    let root = s.path("dự án");
    fs::create_dir_all(root.join("bin")).unwrap();
    fs::write(root.join("notes.txt"), big(200_000)).unwrap();
    // A program for x86-64, as Windows has them, of 64 KB.
    let mut program = vec![0u8; 64 << 10];
    program[..2].copy_from_slice(b"MZ");
    program[0x3C..0x40].copy_from_slice(&0x80u32.to_le_bytes());
    program[0x80..0x84].copy_from_slice(b"PE\0\0");
    program[0x84..0x86].copy_from_slice(&0x8664u16.to_le_bytes());
    program[0x200..].copy_from_slice(&big((64 << 10) - 0x200));
    fs::write(root.join("bin/tool.exe"), &program).unwrap();
    // Named as one, but not one: with the rest.
    fs::write(root.join("bin/fake.exe"), big(8192)).unwrap();
    // Compressed already, but small: with the rest too; large, stored.
    fs::write(root.join("small.zip"), big(4096)).unwrap();
    fs::write(root.join("large.zip"), big(1 << 20)).unwrap();
    let mib = |n: u32| Some(n << 20);
    let stored: Vec<String> = COMPRESSED_EXTENSIONS
        .iter()
        .map(|e| (*e).to_owned())
        .collect();
    for (level, dictionary) in [
        (packwing_core::Level::Lowest, mib(1)),
        (packwing_core::Level::Low, mib(4)),
        (packwing_core::Level::Medium, mib(32)),
        (packwing_core::Level::High, mib(128)),
        (packwing_core::Level::Highest, mib(256)),
    ] {
        let new = NewArchive {
            kind: Kind::SevenZip,
            level,
            stored_extensions: stored.clone(),
            ..NewArchive::default()
        };
        let path = s.path(&format!("{level:?}.7z"));
        create(
            std::slice::from_ref(&root),
            &path,
            &new,
            &Control::default(),
        )
        .unwrap();
        let text = coding_of(&path, "dự án/notes.txt");
        assert_eq!(
            (text.method, text.filter, text.dictionary),
            (Compression::Lzma2, Filter::None, dictionary),
            "{level:?}"
        );
        let tool = coding_of(&path, "dự án/bin/tool.exe");
        assert_eq!(
            (tool.method, tool.filter, tool.dictionary),
            (Compression::Lzma2, Filter::X86, dictionary),
            "{level:?}"
        );
        assert_eq!(coding_of(&path, "dự án/bin/fake.exe").filter, Filter::None);
        assert_eq!(
            coding_of(&path, "dự án/small.zip").method,
            Compression::Lzma2
        );
        assert_eq!(
            coding_of(&path, "dự án/large.zip").method,
            Compression::Copy
        );
        test(&path, &Keys::default(), &Control::default()).unwrap();
        seven_zip_tests(&path, None);
        let out = s.path(&format!("out-{level:?}"));
        extract(&path, &out, &Keys::default(), &Control::default()).unwrap();
        assert_eq!(fs::read(out.join("dự án/bin/tool.exe")).unwrap(), program);
    }
}
