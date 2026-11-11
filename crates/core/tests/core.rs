//! Archives made here, by hand and with hivepack's compressors, listed,
//! extracted and tested through packwing-core: tar, ZIP and cpio archives
//! and RPM packages, alone and under zstd, one file under zstd, and what is
//! not an archive; disc images and 7z archives of the same files
//! (`tests/data`), alone and under zstd; and a macOS package.

#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use packwing_core::{
    Asked, BadName, Cancel, Child, Container, Control, Damage, Error, ExtractOptions, Keys, Layer,
    OnDamaged, OnUnnamable, PASSPHRASE, PASSWORD, Progress, damages, extract, extract_with,
    left_out, list, renamings, test,
};

/// A folder in the system's temporary one, removed when dropped.
struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Scratch {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "packwing-core-{}-{}",
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

/// A USTAR archive of files and (names ending in `/`) folders.
fn tar(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut out = Vec::new();
    for (name, data) in entries {
        let mut b = vec![0u8; 512];
        b[..name.len()].copy_from_slice(name.as_bytes());
        let dir = name.ends_with('/');
        b[100..108].copy_from_slice(if dir { b"0000755\0" } else { b"0000644\0" });
        b[108..116].copy_from_slice(b"0001750\0");
        b[116..124].copy_from_slice(b"0001750\0");
        b[124..136].copy_from_slice(format!("{:011o}\0", data.len()).as_bytes());
        b[136..148].copy_from_slice(b"15000000000\0");
        b[156] = if dir { b'5' } else { b'0' };
        b[257..265].copy_from_slice(b"ustar\x0000");
        b[148..156].fill(b' ');
        let sum: u32 = b.iter().map(|&c| u32::from(c)).sum();
        b[148..156].copy_from_slice(format!("{sum:06o}\0 ").as_bytes());
        out.extend(b);
        out.extend_from_slice(data);
        out.resize(out.len().div_ceil(512) * 512, 0);
    }
    out.extend([0u8; 1024]);
    out
}

/// A ZIP archive of stored files, as the simplest writers make it.
fn zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut central = Vec::new();
    for (name, data) in entries {
        let offset = out.len() as u32;
        let crc = hivepack::deflate::crc32(0, data);
        let size = data.len() as u32;
        let mut common = Vec::new();
        common.extend(20u16.to_le_bytes()); // version needed
        common.extend(0u16.to_le_bytes()); // flags
        common.extend(0u16.to_le_bytes()); // stored
        common.extend(0u16.to_le_bytes()); // time
        common.extend(0x5B21u16.to_le_bytes()); // date: 2025-09-01
        common.extend(crc.to_le_bytes());
        common.extend(size.to_le_bytes());
        common.extend(size.to_le_bytes());
        common.extend((name.len() as u16).to_le_bytes());
        common.extend(0u16.to_le_bytes()); // extra
        out.extend(b"PK\x03\x04");
        out.extend(&common);
        out.extend(name.as_bytes());
        out.extend_from_slice(data);
        central.extend(b"PK\x01\x02");
        central.extend(20u16.to_le_bytes()); // made by
        central.extend(&common);
        central.extend([0u8; 6]); // comment, disk, internal attributes
        central.extend(if name.ends_with('/') { 0x10u32 } else { 0 }.to_le_bytes());
        central.extend(offset.to_le_bytes());
        central.extend(name.as_bytes());
    }
    let start = out.len() as u32;
    out.extend(&central);
    out.extend(b"PK\x05\x06");
    out.extend([0u8; 4]);
    out.extend((entries.len() as u16).to_le_bytes());
    out.extend((entries.len() as u16).to_le_bytes());
    out.extend((central.len() as u32).to_le_bytes());
    out.extend(start.to_le_bytes());
    out.extend(0u16.to_le_bytes());
    out
}

fn zstd(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0u8; hivepack::zstd::compress_bound(data.len()).unwrap()];
    let n = hivepack::zstd::compress(&mut out, data, 3).unwrap();
    out.truncate(n);
    out
}

/// What zstd's frames in `data` hold, which say their size.
fn unzstd(data: &[u8]) -> Vec<u8> {
    let hivepack::zstd::ContentSize::Known(size) = hivepack::zstd::find_decompressed_size(data)
    else {
        panic!("the frames do not say their size");
    };
    let mut out = vec![0u8; size as usize];
    let n = hivepack::zstd::decompress(&mut out, data).unwrap();
    out.truncate(n);
    out
}

/// A test image of `sample()`'s files, as `tests/data` keeps it: under
/// zstd.
fn image(name: &str) -> Vec<u8> {
    std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data")
            .join(name),
    )
    .unwrap()
}

/// `sample()`'s files as an ISO 9660 image, and as a UDF volume beside
/// ISO 9660, each alone and under zstd: name, data, layers, container.
fn images() -> Vec<(&'static str, Vec<u8>, Vec<Layer>, Container)> {
    let iso = image("sample.iso.zst");
    let udf = image("sample-udf.iso.zst");
    vec![
        ("a.iso", unzstd(&iso), vec![], Container::Iso),
        ("a.iso.zst", iso, vec![Layer::Zstd], Container::Iso),
        ("b.iso", unzstd(&udf), vec![], Container::Udf),
        ("b.iso.zst", udf, vec![Layer::Zstd], Container::Udf),
    ]
}

/// When the test images' files were last changed: 2026-09-01 12:00 UTC.
const IMAGE_TIME: i64 = 1_788_264_000;

/// A newc cpio archive (SVR4, `070701`) of files and (names ending in `/`)
/// folders, then `TRAILER!!!`, as GNU cpio and rpm write one.
fn newc(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut put = |name: &str, mode: u32, data: &[u8], ino: u32| {
        let name = name.trim_end_matches('/');
        let fields = [
            ino,
            mode,
            1000,
            1000,
            1,
            IMAGE_TIME as u32,
            data.len() as u32,
            8,
            1,
            0,
            0,
            name.len() as u32 + 1,
            0,
        ];
        out.extend(b"070701");
        for field in fields {
            out.extend(format!("{field:08x}").as_bytes());
        }
        out.extend(name.as_bytes());
        out.push(0);
        out.resize(out.len().div_ceil(4) * 4, 0);
        out.extend_from_slice(data);
        out.resize(out.len().div_ceil(4) * 4, 0);
    };
    for (i, &(name, data)) in entries.iter().enumerate() {
        let mode = if name.ends_with('/') {
            0o040755
        } else {
            0o100644
        };
        put(name, mode, data, i as u32 + 1);
    }
    put("TRAILER!!!", 0, b"", 0);
    out
}

/// An RPM package around `payload`: a lead (version 3, a binary package),
/// a signature header with one entry, padded to 8 bytes, and a header with
/// none, as rpm writes them.
fn rpm(payload: &[u8]) -> Vec<u8> {
    let mut out = vec![0xED, 0xAB, 0xEE, 0xDB, 3, 0, 0, 0, 0, 1];
    let mut name = b"sample-1.0-1".to_vec();
    name.resize(66, 0);
    out.extend(name);
    out.extend([0, 1, 0, 5]);
    out.extend([0; 16]);
    let header = |entries: &[[u8; 16]], data: &[u8]| {
        let mut h = vec![0x8E, 0xAD, 0xE8, 0x01, 0, 0, 0, 0];
        h.extend((entries.len() as u32).to_be_bytes());
        h.extend((data.len() as u32).to_be_bytes());
        for entry in entries {
            h.extend(entry);
        }
        h.extend(data);
        h
    };
    // RPMSIGTAG_SIZE, an INT32 at 0.
    let size_tag = [0, 0, 0x03, 0xE8, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1];
    out.extend(header(&[size_tag], &[0, 0, 0x30, 0x39]));
    out.resize(out.len().div_ceil(8) * 8, 0);
    out.extend(header(&[], &[]));
    out.extend_from_slice(payload);
    out
}

/// `sample()`'s files as a cpio archive and as an RPM package (its payload
/// under zstd, as Fedora's are), and as 7z archives, each alone and under
/// zstd: name, data, layers, container.
fn more_archives() -> Vec<(&'static str, Vec<u8>, Vec<Layer>, Container)> {
    let cpio = newc(&sample());
    let seven = image("sample.7z");
    vec![
        ("a.cpio", cpio.clone(), vec![], Container::Cpio),
        (
            "a.cpio.zst",
            zstd(&cpio),
            vec![Layer::Zstd],
            Container::Cpio,
        ),
        (
            "a.rpm",
            rpm(&zstd(&cpio)),
            vec![Layer::Rpm, Layer::Zstd],
            Container::Cpio,
        ),
        ("a.7z", seven.clone(), vec![], Container::SevenZip),
        (
            "a.7z.zst",
            zstd(&seven),
            vec![Layer::Zstd],
            Container::SevenZip,
        ),
    ]
}

const NOTES: &[u8] = b"Notes from inside the archive.\n";

fn sample() -> Vec<(&'static str, &'static [u8])> {
    vec![
        ("docs/", b""),
        ("docs/notes.txt", NOTES),
        ("docs/plans/a.txt", b"aaaa"),
        ("readme.md", b"# Read me\n"),
    ]
}

fn names(children: &[Child]) -> Vec<(&str, bool, u64, u64)> {
    children
        .iter()
        .map(|c| (c.name.as_str(), c.dir, c.size, c.files))
        .collect()
}

fn listed(path: &Path) -> packwing_core::Listing {
    list(path, &Keys::default()).unwrap()
}

#[test]
fn lists_tar_and_zip_alone_and_under_zstd() {
    let s = Scratch::new();
    let cases = [
        ("a.tar", tar(&sample()), vec![], Container::Tar),
        (
            "a.tar.zst",
            zstd(&tar(&sample())),
            vec![Layer::Zstd],
            Container::Tar,
        ),
        ("a.zip", zip(&sample()), vec![], Container::Zip),
        (
            "a.zip.zst",
            zstd(&zip(&sample())),
            vec![Layer::Zstd],
            Container::Zip,
        ),
    ];
    for (name, data, layers, container) in cases {
        let l = listed(&s.file(name, &data));
        assert_eq!(l.name, name);
        assert_eq!(l.folder, s.0);
        assert_eq!(l.packed, data.len() as u64);
        assert_eq!((&l.layers, l.container), (&layers, container), "{name}");
        assert_eq!(l.files, 3, "{name}");
        assert_eq!(l.unpacked, (NOTES.len() + 4 + 10) as u64, "{name}");
        assert_eq!(l.encryption, None);
        let top = l.children(&[]);
        assert_eq!(
            names(&top),
            [
                ("docs", true, NOTES.len() as u64 + 4, 2),
                ("readme.md", false, 10, 1)
            ],
            "{name}"
        );
        let docs = l.children(&["docs".into()]);
        assert_eq!(
            names(&docs),
            [
                ("plans", true, 4, 1),
                ("notes.txt", false, NOTES.len() as u64, 1)
            ]
        );
    }
}

#[test]
fn folders_only_implied_by_paths_show() {
    let s = Scratch::new();
    let l = listed(&s.file("a.zip", &zip(&[("x/y/z.txt", b"z"), ("./b.txt", b"bb")])));
    assert_eq!(
        names(&l.children(&[])),
        [("x", true, 1, 1), ("b.txt", false, 2, 1)]
    );
    assert_eq!(names(&l.children(&["x".into()])), [("y", true, 1, 1)]);
}

#[test]
fn one_file_under_layers() {
    let s = Scratch::new();
    let text = b"Just notes, many times.\n".repeat(100);
    let path = s.file("notes.txt.zst", &zstd(&text));
    let l = listed(&path);
    assert_eq!(l.container, Container::Data);
    assert_eq!(l.files, 1);
    assert_eq!(
        names(&l.children(&[])),
        [("notes.txt", false, text.len() as u64, 1)]
    );
    let out = s.0.join("out");
    let summary = extract(&path, &out, &Keys::default(), &Control::default()).unwrap();
    assert_eq!((summary.files, summary.bytes), (1, text.len() as u64));
    assert_eq!(std::fs::read(out.join("notes.txt")).unwrap(), text);
    // Not over what is there.
    let again = extract(&path, &out, &Keys::default(), &Control::default());
    assert!(matches!(again, Err(Error::Failed(_))), "{again:?}");
    assert_eq!(std::fs::read(out.join("notes.txt")).unwrap(), text);
}

#[test]
fn what_packwing_does_not_open_yet_is_named() {
    use packwing_core::Unsupported;
    let s = Scratch::new();
    let mut dmg = vec![0x78, 0x9c, 1, 2, 3];
    dmg.resize(4096, 0);
    let mut trailer = b"koly".to_vec();
    trailer.resize(512, 0);
    dmg.extend(trailer);
    // UDBZ: its first block a bzip2 stream, which more data follows.
    let mut bzip2_dmg = vec![
        0x42, 0x5a, 0x68, 0x39, 0x31, 0x41, 0x59, 0x26, 0x53, 0x59, 0x62, 0xa4, 0x89, 0x15, 0x00,
        0x00, 0x00, 0x81, 0x80, 0x01, 0x40, 0x08, 0x00, 0x20, 0x00, 0x21, 0x98, 0x19, 0x81, 0x61,
        0x77, 0x24, 0x53, 0x85, 0x09, 0x06, 0x2a, 0x48, 0x91, 0x50,
    ];
    bzip2_dmg.resize(4096, 0);
    bzip2_dmg.extend(&dmg[dmg.len() - 512..]);
    let mut appimage = b"\x7fELF\x02\x01\x01\x00AI\x02\x00\x00\x00\x00\x00\x00".to_vec();
    appimage.resize(2048, 0);
    for (name, bytes, what) in [
        (
            "tool.deb",
            b"!<arch>\ndebian-binary   1342".to_vec(),
            Unsupported::Deb,
        ),
        (
            "libz.a",
            b"!<arch>\nadler32.o/      0".to_vec(),
            Unsupported::Ar,
        ),
        ("App.dmg", dmg, Unsupported::Dmg),
        ("Bzip2.dmg", bzip2_dmg, Unsupported::Dmg),
        ("Tool.AppImage", appimage, Unsupported::AppImage),
        (
            "core.snap",
            b"hsqs\x05\x00\x00\x00".to_vec(),
            Unsupported::SquashFs,
        ),
    ] {
        let path = s.file(name, &bytes);
        let listed = list(&path, &Keys::default());
        assert!(
            matches!(listed, Err(Error::Unsupported(w)) if w == what),
            "{name}: {listed:?}"
        );
        let out = s.0.join("out");
        let extracted = extract(&path, &out, &Keys::default(), &Control::default());
        assert!(
            matches!(extracted, Err(Error::Unsupported(w)) if w == what),
            "{name}"
        );
        assert!(!out.exists());
    }
}

#[test]
fn not_an_archive() {
    let s = Scratch::new();
    let path = s.file("photo.jpg", b"\xFF\xD8\xFF\xE0 not an archive");
    assert!(matches!(
        list(&path, &Keys::default()),
        Err(Error::NotAnArchive)
    ));
    let out = s.0.join("out");
    assert!(matches!(
        extract(&path, &out, &Keys::default(), &Control::default()),
        Err(Error::NotAnArchive)
    ));
    assert!(!out.exists());
}

#[test]
fn extracts_with_progress() {
    let s = Scratch::new();
    for (name, data) in [
        ("a.tar.zst", zstd(&tar(&sample()))),
        ("a.zip", zip(&sample())),
        ("a.zip.zst", zstd(&zip(&sample()))),
    ] {
        let path = s.file(name, &data);
        let out = s.0.join(format!("out-{name}"));
        let last = Arc::new(Mutex::new(Progress::default()));
        let seen = last.clone();
        let control = Control {
            progress: Some(Arc::new(move |p: &Progress| *seen.lock().unwrap() = *p)),
            cancel: None,
            on_damaged: None,
            on_unnamable: None,
            on_links_outside: None,
            on_gone: None,
        };
        let summary = extract(&path, &out, &Keys::default(), &control).unwrap();
        assert_eq!(summary.files, 3, "{name}");
        assert_eq!(std::fs::read(out.join("docs/notes.txt")).unwrap(), NOTES);
        assert_eq!(
            std::fs::read(out.join("docs/plans/a.txt")).unwrap(),
            b"aaaa"
        );
        let p = *last.lock().unwrap();
        assert_eq!(p.bytes, p.total_bytes, "{name}");
        assert_eq!(p.bytes, (NOTES.len() + 14) as u64, "{name}");
    }
}

#[test]
fn a_damaged_archive_fails_and_writes_nothing() {
    let s = Scratch::new();
    let mut data = zstd(&tar(&sample()));
    let end = data.len() - 3;
    data.truncate(end);
    let path = s.file("a.tar.zst", &data);
    let out = s.0.join("out");
    let e = extract(&path, &out, &Keys::default(), &Control::default()).unwrap_err();
    assert!(matches!(e, Error::Failed(_)), "{e:?}");
    assert!(!out.exists());
    assert!(matches!(
        test(&path, &Keys::default(), &Control::default()),
        Err(Error::Failed(_))
    ));
}

#[test]
fn damaged_files_are_asked_about_and_gone_on_from() {
    let s = Scratch::new();
    let mut data = zip(&[("a.txt", b"aaaa"), ("b.txt", b"bbbb"), ("c.txt", b"cccc")]);
    let at = data.windows(4).position(|w| w == b"bbbb").unwrap();
    data[at] = b'x';
    let path = s.file("a.zip", &data);
    let asked = Arc::new(Mutex::new(Vec::new()));
    let seen = asked.clone();
    let control = Control {
        on_damaged: Some(Arc::new(move |d: &Damage| {
            seen.lock().unwrap().push(d.path.clone());
            OnDamaged::Skip
        })),
        ..Control::default()
    };
    let out = s.0.join("out");
    let summary = extract(&path, &out, &Keys::default(), &control).unwrap();
    assert_eq!(*asked.lock().unwrap(), ["b.txt"]);
    let damaged = damages(&summary);
    assert_eq!(damaged.len(), 1);
    assert_eq!((damaged[0].path.as_str(), damaged[0].kept), ("b.txt", None));
    assert!(out.join("a.txt").exists() && out.join("c.txt").exists());
    assert!(!out.join("b.txt").exists());
    // Without the question, it stops.
    let e = extract(
        &path,
        &s.0.join("again"),
        &Keys::default(),
        &Control::default(),
    );
    assert!(matches!(e, Err(Error::Failed(_))), "{e:?}");
}

/// Names Windows cannot take, and two that differ only in case.
fn awkward_names() -> Vec<u8> {
    zip(&[
        ("good.txt", b"ok"),
        ("a:b.txt", b"colon"),
        ("aux.txt", b"device"),
        ("README", b"upper"),
        ("readme", b"lower"),
    ])
}

/// On Linux, the names Linux takes extract as they are, and nothing is asked.
#[cfg(all(unix, not(target_os = "macos")))]
#[test]
fn names_this_system_takes_extract_as_they_are() {
    let s = Scratch::new();
    let path = s.file("a.zip", &awkward_names());
    let asked = Arc::new(Mutex::new(0));
    let seen = asked.clone();
    let control = Control {
        on_unnamable: Some(Arc::new(move |_: &[BadName]| {
            *seen.lock().unwrap() += 1;
            OnUnnamable::Stop
        })),
        ..Control::default()
    };
    let out = s.0.join("out");
    let summary = extract(&path, &out, &Keys::default(), &control).unwrap();
    assert_eq!(summary.files, 5);
    assert_eq!(*asked.lock().unwrap(), 0);
    assert_eq!(std::fs::read(out.join("a:b.txt")).unwrap(), b"colon");
    assert_eq!(std::fs::read(out.join("readme")).unwrap(), b"lower");
}

/// Names every system cannot take are asked about once, all together, and
/// left out or renamed as answered.
#[test]
fn names_not_every_system_takes_are_asked_about_once() {
    let s = Scratch::new();
    let path = s.file("a.zip", &awkward_names());
    let how = ExtractOptions {
        portable_names: true,
        ..ExtractOptions::default()
    };
    let asking = |answer: OnUnnamable| {
        let asked = Arc::new(Mutex::new(Vec::new()));
        let seen = asked.clone();
        let control = Control {
            on_unnamable: Some(Arc::new(move |bad: &[BadName]| {
                seen.lock().unwrap().push(bad.to_vec());
                answer
            })),
            ..Control::default()
        };
        (control, asked)
    };
    let (control, asked) = asking(OnUnnamable::Skip);
    let out = s.0.join("skipped");
    let summary = extract_with(&path, &out, &Keys::default(), &how, &control).unwrap();
    let asked = asked.lock().unwrap();
    assert_eq!(asked.len(), 1);
    let paths: Vec<&str> = asked[0].iter().map(|b| b.path.as_str()).collect();
    assert_eq!(paths, ["a:b.txt", "aux.txt", "readme"]);
    assert!(asked[0].iter().all(|b| !b.why.is_empty()));
    let left: Vec<String> = left_out(&summary).into_iter().map(|b| b.path).collect();
    assert_eq!(left, ["a:b.txt", "aux.txt", "readme"]);
    assert!(out.join("good.txt").exists() && out.join("README").exists());
    assert!(!out.join("aux.txt").exists());

    let (control, _) = asking(OnUnnamable::Rename);
    let out = s.0.join("renamed");
    let summary = extract_with(&path, &out, &Keys::default(), &how, &control).unwrap();
    let given: Vec<(String, String)> = renamings(&summary)
        .into_iter()
        .map(|r| (r.path, r.to))
        .collect();
    assert_eq!(
        given,
        [
            ("a:b.txt".into(), "a_b.txt".into()),
            ("aux.txt".into(), "_aux.txt".into()),
            ("readme".into(), "readme (2)".into()),
        ]
    );
    assert_eq!(std::fs::read(out.join("readme (2)")).unwrap(), b"lower");

    // Without the question, it stops, and writes nothing.
    let out = s.0.join("stopped");
    let e = extract_with(&path, &out, &Keys::default(), &how, &Control::default());
    assert!(matches!(e, Err(Error::Failed(_))), "{e:?}");
    assert!(!out.exists());
}

#[test]
fn tests_every_file() {
    let s = Scratch::new();
    for name in ["a.tar.zst", "a.zip", "a.zip.zst"] {
        let data = match name {
            "a.tar.zst" => zstd(&tar(&sample())),
            "a.zip" => zip(&sample()),
            _ => zstd(&zip(&sample())),
        };
        let summary = test(s.file(name, &data), &Keys::default(), &Control::default()).unwrap();
        assert_eq!(
            (summary.files, summary.bytes),
            (3, (NOTES.len() + 14) as u64),
            "{name}"
        );
    }
    // A ZIP file whose data does not match its CRC-32.
    let mut data = zip(&sample());
    let at = data.windows(5).position(|w| w == b"Notes").unwrap();
    data[at] = b'M';
    let e = test(
        s.file("bad.zip", &data),
        &Keys::default(),
        &Control::default(),
    )
    .unwrap_err();
    assert!(matches!(e, Error::Failed(_)), "{e:?}");
}

#[test]
fn cancelled_before_it_starts() {
    let s = Scratch::new();
    let path = s.file("a.tar", &tar(&sample()));
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
    let out = s.0.join("out");
    assert!(matches!(
        extract(&path, &out, &Keys::default(), &control),
        Err(Error::Cancelled)
    ));
    assert!(matches!(
        test(&path, &Keys::default(), &control),
        Err(Error::Cancelled)
    ));
}

#[test]
fn lists_disc_images_alone_and_under_zstd() {
    let s = Scratch::new();
    for (name, data, layers, container) in images() {
        let l = listed(&s.file(name, &data));
        assert_eq!(l.packed, data.len() as u64, "{name}");
        assert_eq!((&l.layers, l.container), (&layers, container), "{name}");
        assert_eq!(l.encryption, None, "{name}");
        assert_eq!(l.files, 3, "{name}");
        assert_eq!(l.unpacked, (NOTES.len() + 4 + 10) as u64, "{name}");
        assert_eq!(
            names(&l.children(&[])),
            [
                ("docs", true, NOTES.len() as u64 + 4, 2),
                ("readme.md", false, 10, 1)
            ],
            "{name}"
        );
        assert_eq!(
            names(&l.children(&["docs".into()])),
            [
                ("plans", true, 4, 1),
                ("notes.txt", false, NOTES.len() as u64, 1)
            ],
            "{name}"
        );
        let readme = l.entries.iter().find(|e| e.path == ["readme.md"]).unwrap();
        assert_eq!(readme.modified, Some(IMAGE_TIME), "{name}");
    }
}

#[test]
fn extracts_disc_images_with_progress() {
    let s = Scratch::new();
    for (name, data, _, _) in images() {
        let path = s.file(name, &data);
        let out = s.0.join(format!("out-{name}"));
        let last = Arc::new(Mutex::new(Progress::default()));
        let seen = last.clone();
        let control = Control {
            progress: Some(Arc::new(move |p: &Progress| *seen.lock().unwrap() = *p)),
            cancel: None,
            on_damaged: None,
            on_unnamable: None,
            on_links_outside: None,
            on_gone: None,
        };
        let summary = extract(&path, &out, &Keys::default(), &control).unwrap();
        assert_eq!(summary.files, 3, "{name}");
        assert_eq!(std::fs::read(out.join("docs/notes.txt")).unwrap(), NOTES);
        assert_eq!(
            std::fs::read(out.join("docs/plans/a.txt")).unwrap(),
            b"aaaa"
        );
        assert_eq!(
            std::fs::read(out.join("readme.md")).unwrap(),
            b"# Read me\n"
        );
        let p = *last.lock().unwrap();
        assert_eq!(p.bytes, p.total_bytes, "{name}");
        assert_eq!(p.bytes, (NOTES.len() + 14) as u64, "{name}");
        // Not over what is there.
        let again = extract(&path, &out, &Keys::default(), &Control::default());
        assert!(matches!(again, Err(Error::Failed(_))), "{name}: {again:?}");
    }
}

#[test]
fn tests_every_file_of_disc_images() {
    let s = Scratch::new();
    for (name, data, _, _) in images() {
        let summary = test(s.file(name, &data), &Keys::default(), &Control::default()).unwrap();
        assert_eq!(
            (summary.files, summary.bytes),
            (3, (NOTES.len() + 14) as u64),
            "{name}"
        );
    }
}

#[test]
fn a_disc_image_cut_short_fails_and_writes_nothing() {
    let s = Scratch::new();
    for (name, data, _, _) in images().into_iter().filter(|(_, _, l, _)| l.is_empty()) {
        // The descriptors are there, the files' data is not.
        let at = data.windows(NOTES.len()).position(|w| w == NOTES).unwrap();
        let path = s.file(name, &data[..at]);
        let out = s.0.join(format!("out-{name}"));
        let e = extract(&path, &out, &Keys::default(), &Control::default()).unwrap_err();
        assert!(matches!(e, Error::Failed(_)), "{name}: {e:?}");
        assert!(!out.join("docs/notes.txt").exists(), "{name}");
        let e = test(&path, &Keys::default(), &Control::default()).unwrap_err();
        assert!(matches!(e, Error::Failed(_)), "{name}: {e:?}");
    }
}

#[test]
fn disc_images_cancelled_before_they_start() {
    let s = Scratch::new();
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
    for (name, data, _, _) in images() {
        let path = s.file(name, &data);
        let out = s.0.join(format!("out-{name}"));
        let e = extract(&path, &out, &Keys::default(), &control).unwrap_err();
        assert!(matches!(e, Error::Cancelled), "{name}: {e:?}");
        let e = test(&path, &Keys::default(), &control).unwrap_err();
        assert!(matches!(e, Error::Cancelled), "{name}: {e:?}");
    }
}

#[test]
fn lists_cpio_rpm_and_7z_alone_and_under_zstd() {
    let s = Scratch::new();
    for (name, data, layers, container) in more_archives() {
        let l = listed(&s.file(name, &data));
        assert_eq!((&l.layers, l.container), (&layers, container), "{name}");
        assert_eq!(l.encryption, None, "{name}");
        assert_eq!(l.files, 3, "{name}");
        assert_eq!(l.unpacked, (NOTES.len() + 4 + 10) as u64, "{name}");
        assert_eq!(
            names(&l.children(&[])),
            [
                ("docs", true, NOTES.len() as u64 + 4, 2),
                ("readme.md", false, 10, 1)
            ],
            "{name}"
        );
        assert_eq!(
            names(&l.children(&["docs".into()])),
            [
                ("plans", true, 4, 1),
                ("notes.txt", false, NOTES.len() as u64, 1)
            ],
            "{name}"
        );
        let readme = l.entries.iter().find(|e| e.path == ["readme.md"]).unwrap();
        assert_eq!(readme.modified, Some(IMAGE_TIME), "{name}");
    }
}

#[test]
fn extracts_and_tests_cpio_rpm_and_7z() {
    let s = Scratch::new();
    for (name, data, _, _) in more_archives() {
        let path = s.file(name, &data);
        let out = s.0.join(format!("out-{name}"));
        let last = Arc::new(Mutex::new(Progress::default()));
        let seen = last.clone();
        let control = Control {
            progress: Some(Arc::new(move |p: &Progress| *seen.lock().unwrap() = *p)),
            cancel: None,
            on_damaged: None,
            on_unnamable: None,
            on_links_outside: None,
            on_gone: None,
        };
        let summary = extract(&path, &out, &Keys::default(), &control).unwrap();
        assert_eq!(summary.files, 3, "{name}");
        assert_eq!(std::fs::read(out.join("docs/notes.txt")).unwrap(), NOTES);
        assert_eq!(
            std::fs::read(out.join("docs/plans/a.txt")).unwrap(),
            b"aaaa"
        );
        assert_eq!(
            std::fs::read(out.join("readme.md")).unwrap(),
            b"# Read me\n"
        );
        let p = *last.lock().unwrap();
        assert_eq!(p.bytes, p.total_bytes, "{name}");
        assert_eq!(p.bytes, (NOTES.len() + 14) as u64, "{name}");
        let again = extract(&path, &out, &Keys::default(), &Control::default());
        assert!(matches!(again, Err(Error::Failed(_))), "{name}: {again:?}");

        let summary = test(&path, &Keys::default(), &Control::default()).unwrap();
        assert_eq!(
            (summary.files, summary.bytes),
            (3, (NOTES.len() + 14) as u64),
            "{name}"
        );
    }
}

#[test]
fn a_7z_archive_needs_its_password() {
    let s = Scratch::new();
    let secret = Keys {
        password: Some(b"secret".to_vec()),
        ..Keys::default()
    };
    let wrong = Keys {
        password: Some(b"wrong".to_vec()),
        ..Keys::default()
    };
    // Its files encrypted, its names not: it lists without the password,
    // and says it is encrypted.
    let path = s.file("secret.7z", &image("secret.7z"));
    let l = listed(&path);
    assert_eq!(l.encryption.as_deref(), Some("AES-256"));
    assert_eq!(l.files, 3);
    let out = s.0.join("out-secret");
    let e = extract(&path, &out, &Keys::default(), &Control::default()).unwrap_err();
    assert!(
        matches!(&e, Error::Locked { needs } if needs == &[PASSWORD]),
        "{e:?}"
    );
    assert_eq!(e.to_string(), "This archive needs its password.");
    let e = test(&path, &wrong, &Control::default()).unwrap_err();
    assert!(matches!(e, Error::WrongPassword), "{e:?}");
    let e = extract(&path, &s.0.join("out-wrong"), &wrong, &Control::default()).unwrap_err();
    assert!(matches!(e, Error::WrongPassword), "{e:?}");
    extract(&path, &out, &secret, &Control::default()).unwrap();
    assert_eq!(std::fs::read(out.join("docs/notes.txt")).unwrap(), NOTES);
    assert_eq!(test(&path, &secret, &Control::default()).unwrap().files, 3);

    // Its names encrypted too: not even listed without it.
    let path = s.file("hidden.7z", &image("hidden.7z"));
    let e = list(&path, &Keys::default()).unwrap_err();
    assert!(
        matches!(&e, Error::Locked { needs } if needs == &[PASSWORD]),
        "{e:?}"
    );
    let e = list(&path, &wrong).unwrap_err();
    assert!(matches!(e, Error::WrongPassword), "{e:?}");
    let l = list(&path, &secret).unwrap();
    assert_eq!((l.files, l.encryption.as_deref()), (3, Some("AES-256")));
    let out = s.0.join("out-hidden");
    extract(&path, &out, &secret, &Control::default()).unwrap();
    assert_eq!(
        std::fs::read(out.join("readme.md")).unwrap(),
        b"# Read me\n"
    );
}

#[test]
fn a_damaged_cpio_or_7z_archive_fails_and_writes_nothing() {
    let s = Scratch::new();
    // A cpio archive cut in its last file's data: read once, the cut is met
    // before anything moves into place.
    let mut cpio = newc(&sample());
    let at = cpio.windows(10).position(|w| w == b"# Read me\n").unwrap();
    cpio.truncate(at + 4);
    let path = s.file("cut.cpio", &cpio);
    let out = s.0.join("out-cpio");
    let e = extract(&path, &out, &Keys::default(), &Control::default()).unwrap_err();
    assert!(matches!(e, Error::Failed(_)), "{e:?}");
    assert!(!out.exists());
    assert!(test(&path, &Keys::default(), &Control::default()).is_err());
    // A 7z archive cut short: its headers, at its end, are gone.
    let seven = image("sample.7z");
    let path = s.file("cut.7z", &seven[..seven.len() - 20]);
    assert!(list(&path, &Keys::default()).is_err());
    let out = s.0.join("out-7z");
    assert!(extract(&path, &out, &Keys::default(), &Control::default()).is_err());
    assert!(!out.exists());
}

#[test]
fn a_macos_package_shows_and_extracts_its_payload() {
    let s = Scratch::new();
    let path = s.file("tiny.pkg", &image("tiny.pkg"));
    let l = listed(&path);
    assert_eq!((&l.layers, l.container), (&vec![], Container::Xar));
    // Its payload, a cpio archive under gzip, shows as the folder it
    // extracts to, as `pkgutil --expand-full` lays it out.
    let readme = ["Payload", "usr", "local", "share", "example", "readme.txt"];
    let file = l.entries.iter().find(|e| e.path == readme).unwrap();
    assert!(!file.dir && file.size > 0);
    let top = l.children(&[]);
    let payload = top.iter().find(|c| c.name == "Payload").unwrap();
    assert!(payload.dir && payload.files == 1 && payload.size == file.size);
    let out = s.0.join("out");
    extract(&path, &out, &Keys::default(), &Control::default()).unwrap();
    let written = std::fs::read(out.join(readme.join("/"))).unwrap();
    assert_eq!(written.len() as u64, file.size);
    test(&path, &Keys::default(), &Control::default()).unwrap();
}

#[test]
fn xar_archives_list_and_test_alone_and_under_zstd() {
    let s = Scratch::new();
    let xar = image("default.xar");
    for (name, data, layers) in [
        ("a.xar", xar.clone(), vec![]),
        ("a.xar.zst", zstd(&xar), vec![Layer::Zstd]),
    ] {
        let path = s.file(name, &data);
        let l = listed(&path);
        assert_eq!(
            (&l.layers, l.container),
            (&layers, Container::Xar),
            "{name}"
        );
        assert!(l.files > 0, "{name}");
        let summary = test(&path, &Keys::default(), &Control::default()).unwrap();
        assert!(summary.files > 0, "{name}");
        // It holds a name Windows does not allow: refused before anything is
        // written where names are to be every system's, and on Windows.
        let out = s.0.join(format!("out-{name}"));
        let portable = ExtractOptions {
            portable_names: true,
            ..ExtractOptions::default()
        };
        let e = extract_with(
            &path,
            &out,
            &Keys::default(),
            &portable,
            &Control::default(),
        )
        .unwrap_err();
        assert!(e.to_string().contains("name can’t be used"), "{name}: {e}");
        assert!(!out.exists(), "{name}");
        let native = extract(&path, &out, &Keys::default(), &Control::default());
        assert_eq!(native.is_ok(), !cfg!(windows), "{name}: {native:?}");
    }
}

#[test]
fn cpio_and_7z_cancelled_before_they_start() {
    let s = Scratch::new();
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
    for (name, data, _, _) in more_archives() {
        let path = s.file(name, &data);
        let out = s.0.join(format!("out-{name}"));
        let e = extract(&path, &out, &Keys::default(), &control).unwrap_err();
        assert!(matches!(e, Error::Cancelled), "{name}: {e:?}");
        let e = test(&path, &Keys::default(), &control).unwrap_err();
        assert!(matches!(e, Error::Cancelled), "{name}: {e:?}");
    }
}

/// Keys with only a passphrase, or only a password.
fn passphrase(p: &[u8]) -> Keys {
    Keys {
        passphrase: Some(p.to_vec()),
        ..Keys::default()
    }
}

fn password(p: &[u8]) -> Keys {
    Keys {
        password: Some(p.to_vec()),
        ..Keys::default()
    }
}

#[test]
fn an_age_file_says_it_needs_a_passphrase_and_opens_with_it() {
    let s = Scratch::new();
    let path = s.file("notes.tar.zst.age", &image("notes.tar.zst.age"));
    // None given: it says what it needs, which the window asks for.
    let e = list(&path, &Keys::default()).unwrap_err();
    assert!(
        matches!(&e, Error::Locked { needs } if needs == &[PASSPHRASE]),
        "{e:?}"
    );
    assert_eq!(e.to_string(), "This file is encrypted with a passphrase.");
    let Error::Locked { needs } = &e else {
        unreachable!()
    };
    assert_eq!(Keys::asks_for(needs), Some(Asked::Passphrase));
    // A wrong one.
    let e = list(&path, &passphrase(b"wrong")).unwrap_err();
    assert!(matches!(e, Error::WrongPassphrase), "{e:?}");
    assert_eq!(e.to_string(), "The passphrase is wrong.");
    // The right one: what is under the layers.
    let keys = Keys::default().with(Asked::Passphrase, b"password".to_vec());
    let l = list(&path, &keys).unwrap();
    assert_eq!(
        (&l.layers, l.container),
        (&vec![Layer::Age, Layer::Zstd], Container::Tar)
    );
    assert_eq!(l.encryption.as_deref(), Some("age"));
    assert_eq!(l.files, 2);
    let out = s.0.join("out");
    let summary = extract(&path, &out, &keys, &Control::default()).unwrap();
    assert_eq!(summary.files, 2);
    assert_eq!(
        std::fs::read(out.join("docs/notes.txt")).unwrap(),
        b"Hello from inside the layers.\n"
    );
    assert_eq!(test(&path, &keys, &Control::default()).unwrap().files, 2);
    // Extracting and testing without it say the same as listing.
    let e = test(&path, &Keys::default(), &Control::default()).unwrap_err();
    assert!(matches!(e, Error::Locked { .. }), "{e:?}");
    let e = extract(
        &path,
        &s.0.join("no"),
        &Keys::default(),
        &Control::default(),
    )
    .unwrap_err();
    assert!(matches!(e, Error::Locked { .. }), "{e:?}");
    assert!(!s.0.join("no").exists());
}

/// A ZIP archive of `sample()`'s files, encrypted with WinZip AES-256 to
/// `password`, as hivepack::zip writes it.
fn encrypted_zip(password: &[u8]) -> Vec<u8> {
    use hivepack::zip::{Archive, BufferSource, Flags, OpenFlags, em};
    let mut za = Archive::open(&[][..], OpenFlags(0)).unwrap();
    // As packwing does: hivepack::zip has none of its own on every system.
    za.set_random_source(Some(std::sync::Arc::new(|buffer: &mut [u8]| {
        getrandom::fill(buffer).map_err(std::io::Error::other)
    })));
    for (name, data) in sample() {
        if name.ends_with('/') {
            za.dir_add(name.as_bytes(), Flags::NONE).unwrap();
            continue;
        }
        let i = za
            .file_add(
                name.as_bytes(),
                BufferSource::with_mtime(data, IMAGE_TIME),
                Flags::NONE,
            )
            .unwrap();
        za.file_set_encryption(i, em::AES_256, Some(password))
            .unwrap();
    }
    let mut out = std::io::Cursor::new(Vec::new());
    za.close_to(&mut out).unwrap();
    out.into_inner()
}

#[test]
fn an_encrypted_zip_archive_needs_its_password() {
    let s = Scratch::new();
    let path = s.file("secret.zip", &encrypted_zip(b"secret"));
    // Its names are not encrypted: it lists, and says it is.
    let l = listed(&path);
    assert_eq!((l.files, l.encryption.as_deref()), (3, Some("AES-256")));
    // None given: what it needs, before anything is written.
    let out = s.0.join("out");
    let e = extract(&path, &out, &Keys::default(), &Control::default()).unwrap_err();
    assert!(
        matches!(&e, Error::Locked { needs } if needs == &[PASSWORD]),
        "{e:?}"
    );
    let Error::Locked { needs } = &e else {
        unreachable!()
    };
    assert_eq!(Keys::asks_for(needs), Some(Asked::Password));
    let e = test(&path, &Keys::default(), &Control::default()).unwrap_err();
    assert!(matches!(e, Error::Locked { .. }), "{e:?}");
    // A wrong one.
    let e = extract(&path, &out, &password(b"wrong"), &Control::default()).unwrap_err();
    assert!(matches!(e, Error::WrongPassword), "{e:?}");
    let e = test(&path, &password(b"wrong"), &Control::default()).unwrap_err();
    assert!(matches!(e, Error::WrongPassword), "{e:?}");
    // The right one.
    let summary = extract(&path, &out, &password(b"secret"), &Control::default()).unwrap();
    assert_eq!(summary.files, 3);
    assert_eq!(std::fs::read(out.join("docs/notes.txt")).unwrap(), NOTES);
    assert_eq!(
        test(&path, &password(b"secret"), &Control::default())
            .unwrap()
            .files,
        3
    );
}

#[test]
fn keys_ask_for_what_a_person_types() {
    let needs = |n: &[&str]| n.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    assert_eq!(Keys::asks_for(&needs(&["scrypt"])), Some(Asked::Passphrase));
    assert_eq!(Keys::asks_for(&needs(&["password"])), Some(Asked::Password));
    // A key is asked for as its file.
    assert_eq!(Keys::asks_for(&needs(&["X25519"])), Some(Asked::KeyFile));
    assert_eq!(
        Keys::asks_for(&needs(&["ssh-ed25519", "X25519"])),
        Some(Asked::KeyFile)
    );
    assert_eq!(
        Keys::asks_for(&needs(&["mlkem768x25519"])),
        Some(Asked::KeyFile)
    );
    // One hivepack does not read is not.
    assert_eq!(Keys::asks_for(&needs(&["p256tag"])), None);
    // What is given keeps what was given before.
    let keys = Keys::default()
        .with(Asked::Passphrase, b"one".to_vec())
        .with(Asked::Password, b"two".to_vec());
    assert_eq!(keys.passphrase.as_deref(), Some(&b"one"[..]));
    assert_eq!(keys.password.as_deref(), Some(&b"two"[..]));
    // Neither shows when printed.
    let shown = format!("{keys:?}");
    assert!(!shown.contains("one") && !shown.contains("two"), "{shown}");
}

/// An age file and its key file, of `tests/data`.
fn keyed(s: &Scratch, file: &str) -> PathBuf {
    s.file(file, &image(file))
}

fn with_key(key: &str) -> Keys {
    Keys::default().with(Asked::KeyFile, image(key))
}

fn opens_with(path: &Path, keys: &Keys, s: &Scratch) {
    let l = list(path, keys).unwrap();
    assert_eq!(
        (&l.layers, l.container, l.files),
        (&vec![Layer::Age, Layer::Zstd], Container::Tar, 1)
    );
    test(path, keys, &Control::default()).unwrap();
    let out = s.0.join("out");
    let _ = std::fs::remove_dir_all(&out);
    extract(path, &out, keys, &Control::default()).unwrap();
    assert_eq!(
        std::fs::read_to_string(out.join("docs/notes.txt")).unwrap(),
        "Ghi chú về khoá\n"
    );
}

#[test]
fn an_age_file_encrypted_to_a_key_asks_for_the_key_file() {
    let s = Scratch::new();
    for (file, key, stanza) in [
        ("x25519.tar.zst.age", "test-age.key", "X25519"),
        ("pq.tar.zst.age", "test-age-pq.key", "mlkem768x25519"),
        ("ssh.tar.zst.age", "test-ssh.key", "ssh-ed25519"),
    ] {
        let path = keyed(&s, file);
        let e = list(&path, &Keys::default()).unwrap_err();
        let Error::Locked { needs } = &e else {
            panic!("{e:?}")
        };
        assert_eq!(needs, &[stanza]);
        assert_eq!(Keys::asks_for(needs), Some(Asked::KeyFile));
        opens_with(&path, &with_key(key), &s);
    }
    // Another key is no key.
    let path = keyed(&s, "x25519.tar.zst.age");
    assert!(matches!(
        list(&path, &with_key("test-ssh.key")),
        Err(Error::Locked { .. })
    ));
    // Nor does a post-quantum key open a file of another, or the other way round.
    assert!(matches!(
        list(&path, &with_key("test-age-pq.key")),
        Err(Error::Locked { .. })
    ));
    let path = keyed(&s, "pq.tar.zst.age");
    assert!(matches!(
        list(&path, &with_key("test-age.key")),
        Err(Error::Locked { .. })
    ));
}

#[test]
fn a_key_file_with_a_passphrase_asks_for_it() {
    let s = Scratch::new();
    for (file, key, secret) in [
        ("ssh-locked.tar.zst.age", "test-ssh-locked.key", "khoá ssh"),
        ("x25519.tar.zst.age", "test-age-locked.key", "khoá age"),
    ] {
        let path = keyed(&s, file);
        let keys = with_key(key);
        let e = list(&path, &keys).unwrap_err();
        let Error::Locked { needs } = &e else {
            panic!("{key}: {e:?}")
        };
        assert_eq!(Keys::asks_for(needs), Some(Asked::KeyPassphrase), "{key}");
        let wrong = keys.with(Asked::KeyPassphrase, b"sai".to_vec());
        let e = list(&path, &wrong).unwrap_err();
        assert!(matches!(e, Error::WrongKeyPassphrase), "{key}: {e:?}");
        opens_with(
            &path,
            &keys.with(Asked::KeyPassphrase, secret.as_bytes().to_vec()),
            &s,
        );
    }
}

#[test]
fn what_is_no_key_file_says_so() {
    for key in [
        "test-age.key",
        "test-age-pq.key",
        "test-age-locked.key",
        "test-ssh.key",
        "test-ssh-locked.key",
    ] {
        packwing_core::check_key_file(&image(key)).unwrap();
    }
    assert!(packwing_core::check_key_file(b"not a key\n").is_err());
    assert!(packwing_core::check_key_file(b"").is_err());
}

/// `tests/data`'s RAR files, copied beside each other as they were made.
fn rar_files(s: &Scratch, names: &[&str]) -> Vec<PathBuf> {
    names.iter().map(|n| s.file(n, &image(n))).collect()
}

fn with_password(p: &str) -> Keys {
    Keys::default().with(Asked::Password, p.as_bytes().to_vec())
}

#[test]
fn a_rar_archive_lists_tests_and_extracts() {
    let s = Scratch::new();
    let path = rar_files(&s, &["rar5_multiple_files.rar"]).remove(0);
    let l = listed(&path);
    assert_eq!(l.container, Container::Rar);
    assert!(l.files > 1, "{}", l.files);
    assert_eq!(l.encryption, None);
    let summary = test(&path, &Keys::default(), &Control::default()).unwrap();
    assert_eq!(summary.files, l.files);
    assert_eq!(summary.bytes, l.unpacked);
    let out = s.0.join("out");
    let done = extract(&path, &out, &Keys::default(), &Control::default()).unwrap();
    assert_eq!(done.files, l.files);
    assert_eq!(done.bytes, l.unpacked);
}

#[test]
fn a_rar_archive_in_volumes_reads_from_the_first() {
    let s = Scratch::new();
    let names: Vec<String> = (1..=6)
        .map(|i| format!("test_rar_multivolume_multiple_files.part{i}.rar"))
        .collect();
    let refs: Vec<&str> = names.iter().map(String::as_str).collect();
    let paths = rar_files(&s, &refs);
    let all: u64 = paths
        .iter()
        .map(|p| std::fs::metadata(p).unwrap().len())
        .sum();
    let l = listed(&paths[0]);
    assert_eq!(l.packed, all);
    assert!(l.files > 1, "{}", l.files);
    let summary = test(&paths[0], &Keys::default(), &Control::default()).unwrap();
    assert_eq!(summary.files, l.files);
    let done = extract(
        &paths[0],
        &s.0.join("out"),
        &Keys::default(),
        &Control::default(),
    )
    .unwrap();
    assert_eq!((done.files, done.bytes), (l.files, l.unpacked));
    // Without a volume, it says so and writes nothing.
    std::fs::remove_file(&paths[3]).unwrap();
    assert!(test(&paths[0], &Keys::default(), &Control::default()).is_err());
    let cut = s.0.join("cut");
    assert!(extract(&paths[0], &cut, &Keys::default(), &Control::default()).is_err());
    assert!(!cut.exists());
}

#[test]
fn an_encrypted_rar_archive_asks_for_its_password() {
    let s = Scratch::new();
    for (file, names_hidden) in [
        ("rar4_solid_encrypted.rar", false),
        ("rar5_solid_encrypted.rar", false),
        ("rar4_encrypted_filenames.rar", true),
        ("rar5_encrypted_filenames.rar", true),
    ] {
        let path = rar_files(&s, &[file]).remove(0);
        let none = Keys::default();
        let right = with_password("password");
        if names_hidden {
            let e = list(&path, &none).unwrap_err();
            assert!(matches!(e, Error::Locked { .. }), "{file}: {e:?}");
        } else {
            assert!(list(&path, &none).unwrap().encryption.is_some(), "{file}");
        }
        let e = test(&path, &none, &Control::default()).unwrap_err();
        assert!(matches!(e, Error::Locked { .. }), "{file}: {e:?}");
        let e = test(&path, &with_password("sai"), &Control::default()).unwrap_err();
        assert!(matches!(e, Error::WrongPassword), "{file}: {e:?}");
        let out = s.0.join(format!("wrong-{file}"));
        let e = extract(&path, &out, &with_password("sai"), &Control::default()).unwrap_err();
        assert!(matches!(e, Error::WrongPassword), "{file}: {e:?}");
        // RAR 3 has only the data to tell by: what failed its check is gone.
        let left = std::fs::read_dir(&out)
            .map_or(0, |d| d.flatten().filter(|e| e.path().is_file()).count());
        assert_eq!(left, 0, "{file}");
        if names_hidden {
            let e = list(&path, &with_password("sai")).unwrap_err();
            assert!(matches!(e, Error::WrongPassword), "{file}: {e:?}");
        }
        test(&path, &right, &Control::default()).unwrap();
        let out = s.0.join(format!("out-{file}"));
        let done = extract(&path, &out, &right, &Control::default()).unwrap();
        assert!(done.files > 0, "{file}");
    }
}

#[test]
fn stopping_a_rar_job_stops_it() {
    let s = Scratch::new();
    let path = rar_files(&s, &["rar5_multiple_files.rar"]).remove(0);
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
    assert!(matches!(
        test(&path, &Keys::default(), &control),
        Err(Error::Cancelled)
    ));
    assert!(matches!(
        extract(&path, &s.0.join("out"), &Keys::default(), &control),
        Err(Error::Cancelled)
    ));
}

#[test]
fn a_search_finds_names_at_any_depth_whatever_their_case_and_accents() {
    let entry = |path: &str, dir, size| packwing_core::Entry {
        path: path.split('/').map(str::to_owned).collect(),
        dir,
        size,
        modified: None,
        link: None,
    };
    let listing = packwing_core::Listing {
        name: "a.zip".into(),
        folder: PathBuf::new(),
        packed: 0,
        unpacked: 0,
        files: 0,
        layers: Vec::new(),
        container: Container::Zip,
        encryption: None,
        entries: vec![
            entry("Báo cáo/quý 1/Tổng hợp.xlsx", false, 10),
            // As macOS writes it: the accents apart.
            entry("Bao\u{301} ca\u{301}o nháp.txt", false, 5),
            entry("Đơn hàng.pdf", false, 1),
            entry("khác/báo cáo.doc", false, 2),
            entry("rỗng/", true, 0),
        ],
    };
    let paths =
        |q: &str| -> Vec<String> { listing.search(q).iter().map(|f| f.path.join("/")).collect() };
    assert_eq!(
        paths("bao cao"),
        [
            "Báo cáo",
            "Bao\u{301} ca\u{301}o nháp.txt",
            "khác/báo cáo.doc"
        ]
    );
    assert_eq!(paths("BÁO CÁO"), paths("bao cao"));
    assert_eq!(paths("don hang"), ["Đơn hàng.pdf"]);
    assert_eq!(paths("tong"), ["Báo cáo/quý 1/Tổng hợp.xlsx"]);
    assert_eq!(paths("quy"), ["Báo cáo/quý 1"]);
    assert!(paths("  ").is_empty());
    assert!(paths("không có").is_empty());
    // A folder has what it holds counted, and says it is one.
    let folder = &listing.search("bao cao")[0];
    assert!(folder.child.dir);
    assert_eq!((folder.child.files, folder.child.size), (1, 10));
}

/// A tar archive of a file named by these bytes, as GNU's format takes any.
fn tar_named(name: &[u8]) -> Vec<u8> {
    let mut tar = hivepack::tar::Writer::new(Vec::new());
    tar.write_header(&hivepack::tar::Header {
        typeflag: hivepack::tar::TYPE_REG,
        name: name.to_vec(),
        mode: 0o644,
        size: 2,
        format: hivepack::tar::Format::GNU,
        ..Default::default()
    })
    .unwrap();
    std::io::Write::write_all(&mut tar, b"hi").unwrap();
    tar.close().unwrap();
    tar.into_inner()
}

#[test]
fn a_name_in_an_old_encoding_does_not_stop_a_tar_archive() {
    let s = Scratch::new();
    let path = s.file("old.tar", &tar_named(b"B\xe1o.txt"));
    let keys = Keys::default();
    let control = Control::default();
    let shown = "B\u{fffd}o.txt";
    let l = list(&path, &keys).unwrap();
    assert_eq!(l.entries[0].path, [shown]);
    let out = s.0.join("out");
    extract(&path, &out, &keys, &control).unwrap();
    assert_eq!(std::fs::read(out.join(shown)).unwrap(), b"hi");
    // By the name the list shows, what is selected or opened is found.
    let some = s.0.join("some");
    let how = packwing_core::ExtractOptions::default();
    let picked = [vec![shown.to_owned()]];
    packwing_core::extract_selected(&path, &some, &picked, &keys, &how, &control).unwrap();
    assert_eq!(std::fs::read(some.join(shown)).unwrap(), b"hi");
    let read = packwing_core::read_entry(&path, &picked[0], &keys, 100, &control).unwrap();
    assert_eq!(read, b"hi");
}

#[test]
fn a_name_that_cannot_be_used_says_which_and_writes_nothing() {
    let s = Scratch::new();
    let path = s.file("bad.tar", &tar_named(b"../evil.txt"));
    let out = s.0.join("out");
    let e = extract(&path, &out, &Keys::default(), &Control::default()).unwrap_err();
    let said = e.to_string();
    assert!(
        said.contains("../evil.txt") && said.contains("name"),
        "{said}"
    );
    assert!(!s.0.join("evil.txt").exists());
}

/// What `out` holds, a file at a time: its path under `out`, and its bytes.
fn files_under(out: &Path) -> Vec<(String, Vec<u8>)> {
    fn walk(at: &Path, base: &Path, found: &mut Vec<(String, Vec<u8>)>) {
        let mut children: Vec<_> = std::fs::read_dir(at)
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        children.sort();
        for child in children {
            if child.is_dir() {
                walk(&child, base, found);
            } else {
                let name = child
                    .strip_prefix(base)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                found.push((name, std::fs::read(&child).unwrap()));
            }
        }
    }
    let mut found = Vec::new();
    walk(out, out, &mut found);
    found
}

#[test]
fn converts_an_archive_into_each_format_entry_by_entry() {
    use packwing_core::{ExtractOptions, Kind, NewArchive, convert};
    let s = Scratch::new();
    let from = s.file("a.tar", &tar(&sample()));
    let mut want: Vec<(String, Vec<u8>)> = sample()
        .into_iter()
        .filter(|(name, _)| !name.ends_with('/'))
        .map(|(name, data)| (name.to_owned(), data.to_vec()))
        .collect();
    want.sort();
    for (kind, name) in [
        (Kind::Zip, "b.zip"),
        (Kind::SevenZip, "b.7z"),
        (Kind::TarZst, "b.tar.zst"),
        (Kind::TarGz, "b.tar.gz"),
    ] {
        let new = NewArchive {
            kind,
            ..NewArchive::default()
        };
        let done = convert(
            &from,
            &Keys::default(),
            &s.0.join(name),
            &new,
            &ExtractOptions::default(),
            &packwing_core::Filter::default(),
            &Control::default(),
        )
        .unwrap();
        assert_eq!(done.made, s.0.join(name));
        assert_eq!(done.summary.files, 3, "{name}");
        let out = s.0.join(format!("out-{name}"));
        extract(&done.made, &out, &Keys::default(), &Control::default()).unwrap();
        assert_eq!(files_under(&out), want, "{name}");
    }
    // A name taken goes to the next that is free, as a new archive's does.
    let taken = convert(
        &from,
        &Keys::default(),
        &s.0.join("b.zip"),
        &NewArchive::default(),
        &ExtractOptions::default(),
        &packwing_core::Filter::default(),
        &Control::default(),
    )
    .unwrap();
    assert_eq!(taken.made, s.0.join("b (2).zip"));
    // Only what the filter takes goes in.
    let filter = packwing_core::Filter::new(&["docs".to_owned()], &["plans".to_owned()]).unwrap();
    let picked = convert(
        &from,
        &Keys::default(),
        &s.0.join("picked.zip"),
        &NewArchive::default(),
        &ExtractOptions::default(),
        &filter,
        &Control::default(),
    )
    .unwrap();
    let out = s.0.join("out-picked");
    extract(&picked.made, &out, &Keys::default(), &Control::default()).unwrap();
    let names: Vec<String> = files_under(&out).into_iter().map(|(n, _)| n).collect();
    assert_eq!(names, ["docs/notes.txt"]);
}

#[test]
fn a_damaged_file_is_left_out_of_what_is_converted() {
    use packwing_core::{ExtractOptions, NewArchive, convert};
    let s = Scratch::new();
    let mut data = zip(&[("a.txt", b"aaaa"), ("b.txt", b"bbbb"), ("c.txt", b"cccc")]);
    let at = data.windows(4).position(|w| w == b"bbbb").unwrap();
    data[at] = b'x';
    let from = s.file("a.zip", &data);
    let to = s.0.join("b.7z");
    let how = ExtractOptions::default();
    let all = packwing_core::Filter::default();
    // Stopped, it leaves nothing.
    let e = convert(
        &from,
        &Keys::default(),
        &to,
        &NewArchive::default(),
        &how,
        &all,
        &Control::default(),
    );
    assert!(matches!(e, Err(Error::Failed(_))), "{e:?}");
    assert!(std::fs::read_dir(&s.0).unwrap().count() == 1);
    let control = Control {
        on_damaged: Some(Arc::new(|_: &Damage| OnDamaged::Skip)),
        ..Control::default()
    };
    let new = packwing_core::NewArchive {
        kind: packwing_core::Kind::SevenZip,
        ..packwing_core::NewArchive::default()
    };
    let done = convert(&from, &Keys::default(), &to, &new, &how, &all, &control).unwrap();
    assert_eq!(damages(&done.summary).len(), 1);
    let out = s.0.join("out");
    extract(&done.made, &out, &Keys::default(), &Control::default()).unwrap();
    let names: Vec<String> = files_under(&out).into_iter().map(|(n, _)| n).collect();
    assert_eq!(names, ["a.txt", "c.txt"]);
}

#[test]
fn a_file_split_joins_back_as_it_was() {
    use packwing_core::{join_parts, parts_of, split_file};
    let s = Scratch::new();
    let data: Vec<u8> = (0..10_000u32).flat_map(|n| n.to_le_bytes()).collect();
    let file = s.file("a.bin", &data);
    let parts = split_file(&file, 15_000, &Control::default()).unwrap();
    let names: Vec<String> = parts
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, ["a.bin.001", "a.bin.002", "a.bin.003"]);
    assert_eq!(std::fs::metadata(&parts[2]).unwrap().len(), 10_000);
    // Never over what is there.
    assert!(split_file(&file, 15_000, &Control::default()).is_err());
    // From any part, beside the file it was, under a name that is free.
    let joined = join_parts(&parts[1], &Control::default()).unwrap();
    assert_eq!(joined, s.0.join("a (2).bin"));
    assert_eq!(std::fs::read(&joined).unwrap(), data);
    // One missing, or cut short, is said.
    std::fs::rename(&parts[1], s.0.join("held")).unwrap();
    assert!(matches!(parts_of(&parts[0]), Err(Error::Failed(_))));
    std::fs::write(&parts[1], b"short").unwrap();
    assert!(matches!(parts_of(&parts[0]), Err(Error::Failed(_))));
    assert!(matches!(parts_of(&file), Err(Error::Failed(_))));
}

/// An RPM package's payload and a macOS package into a folder are read once,
/// as a tar archive is: until their entries are checked, how far it is, is
/// told from the archive's bytes read, against its size; what is told never
/// goes back, ends at all of it, and no private folder is left.
#[test]
fn a_cpio_or_a_package_extracted_tells_how_far_through_the_archive() {
    let s = Scratch::new();
    // Data that does not compress, so that the archive is as large.
    let mut seed = 7u64;
    let mut noise = |n: usize| -> Vec<u8> {
        (0..n)
            .map(|_| {
                seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                (seed >> 56) as u8
            })
            .collect()
    };
    let files: Vec<Vec<u8>> = (0..4).map(|_| noise(1 << 20)).collect();
    let names = ["a", "b", "c", "d"];
    let entries: Vec<(&str, &[u8])> = names
        .iter()
        .copied()
        .zip(files.iter().map(Vec::as_slice))
        .collect();
    let all = 4 << 20;
    for (name, data, written) in [
        ("big.rpm", rpm(&zstd(&newc(&entries))), names.to_vec()),
        (
            "tiny.pkg",
            image("tiny.pkg"),
            vec!["Bom", "PackageInfo", "Payload"],
        ),
    ] {
        let path = s.file(name, &data);
        let size = data.len() as u64;
        let told = Arc::new(Mutex::new(Vec::new()));
        let keep = told.clone();
        let control = Control {
            progress: Some(Arc::new(move |p: &Progress| {
                keep.lock().unwrap().push((p.bytes, p.total_bytes));
            })),
            ..Control::default()
        };
        let out = s.0.join(format!("out-{name}"));
        extract(&path, &out, &Keys::default(), &control).unwrap();
        let told = told.lock().unwrap();
        assert!(
            told.iter().any(|&(_, total)| total == size),
            "{name}: {told:?}"
        );
        let fractions: Vec<f64> = told
            .iter()
            .filter(|&&(_, total)| total > 0)
            .map(|&(bytes, total)| bytes as f64 / total as f64)
            .collect();
        assert!(
            fractions.windows(2).all(|w| w[1] >= w[0]),
            "{name}: {fractions:?}"
        );
        let last = *told.last().unwrap();
        assert_eq!(last.0, last.1, "{name}");
        let mut left: Vec<_> = std::fs::read_dir(&out)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        left.sort();
        assert_eq!(left, written, "{name}");
        if name == "big.rpm" {
            assert_eq!(last.1, all);
            assert!(std::fs::read(out.join("c")).unwrap() == files[2]);
        }
    }
}
