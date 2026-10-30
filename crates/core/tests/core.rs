//! Archives made here, by hand and with hivepack's compressors, listed,
//! extracted and tested through packwing-core: tar and ZIP archives, alone
//! and under zstd, one file under zstd, and what is not an archive.

#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use packwing_core::{
    Cancel, Child, Container, Control, Error, Keys, Layer, Progress, extract, list, test,
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
