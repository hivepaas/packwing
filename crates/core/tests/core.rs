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
    Asked, Cancel, Child, Container, Control, Error, Keys, Layer, PASSPHRASE, PASSWORD, Progress,
    extract, list, test,
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
        let crc = hivepack_deflate::crc32(0, data);
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
    let mut out = vec![0u8; hivepack_zstd::compress_bound(data.len()).unwrap()];
    let n = hivepack_zstd::compress(&mut out, data, 3).unwrap();
    out.truncate(n);
    out
}

/// What zstd's frames in `data` hold, which say their size.
fn unzstd(data: &[u8]) -> Vec<u8> {
    let hivepack_zstd::ContentSize::Known(size) = hivepack_zstd::find_decompressed_size(data)
    else {
        panic!("the frames do not say their size");
    };
    let mut out = vec![0u8; size as usize];
    let n = hivepack_zstd::decompress(&mut out, data).unwrap();
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
fn tests_every_file() {
    let s = Scratch::new();
    for name in ["a.tar.zst", "a.zip", "a.zip.zst"] {
        let data = match name {
            "a.tar.zst" => zstd(&tar(&sample())),
            "a.zip" => zip(&sample()),
            _ => zstd(&zip(&sample())),
        };
        let summary = test(&s.file(name, &data), &Keys::default(), &Control::default()).unwrap();
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
        &s.file("bad.zip", &data),
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
        let summary = test(&s.file(name, &data), &Keys::default(), &Control::default()).unwrap();
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
    // A cpio archive cut in its last file's data: the first reading reads
    // it, before anything is written.
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
        // It holds a name Windows does not allow, refused everywhere before
        // anything is written.
        let out = s.0.join(format!("out-{name}"));
        let e = extract(&path, &out, &Keys::default(), &Control::default()).unwrap_err();
        assert!(e.to_string().contains("name refused"), "{name}: {e}");
        assert!(!out.exists(), "{name}");
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
/// `password`, as hivepack-zip writes it.
fn encrypted_zip(password: &[u8]) -> Vec<u8> {
    use hivepack_zip::{Archive, BufferSource, Flags, OpenFlags, em};
    let mut za = Archive::open(&[][..], OpenFlags(0)).unwrap();
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
    // A key packwing cannot ask a person for.
    assert_eq!(Keys::asks_for(&needs(&["X25519"])), None);
    assert_eq!(Keys::asks_for(&needs(&["ssh-ed25519", "X25519"])), None);
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
