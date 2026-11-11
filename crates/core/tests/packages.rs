//! Debian packages, snaps and AppImages, listed, tested, extracted (all and
//! what is picked), converted and read where they are, through
//! packwing-core: `tests/data`'s packages, made by dpkg-deb with their
//! `data.tar` under xz, Zstandard and gzip, and SquashFS images, made by
//! mksquashfs (a snap's, with each compressor, and one of links, a hard
//! link, a hole and a setuid program); AppImages made here of them, of type
//! 2 (an ELF program, then the image) and of type 1 (an ISO 9660 image whose
//! system area holds the program); each alone and under zstd. packwing
//! never writes them.

#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use packwing_core::{
    Cancel, Container, Control, Error, ExtractOptions, Keys, Layer, LocalFolder, Progress, Source,
    Stored, extract, extract_selected, list, read_entry, test,
};

/// A folder in the system's temporary one, removed when dropped.
struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Scratch {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "packwing-packages-{}-{}",
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

fn data(name: &str) -> Vec<u8> {
    std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data")
            .join(name),
    )
    .unwrap()
}

fn zstd(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0u8; hivepack::zstd::compress_bound(data.len()).unwrap()];
    let n = hivepack::zstd::compress(&mut out, data, 3).unwrap();
    out.truncate(n);
    out
}

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

/// A 64-bit little-endian ELF program of 4,544 bytes, marked as an
/// AppImage of type `kind` (`AI` and 1 or 2 at byte 8): its seven section
/// headers at its end, which is where its image starts in type 2.
fn program(kind: u8) -> Vec<u8> {
    let (shoff, entsize, shnum) = (4096usize, 64usize, 7usize);
    let mut f = vec![0u8; shoff + entsize * shnum];
    f[..4].copy_from_slice(b"\x7fELF");
    f[4] = 2;
    f[5] = 1;
    f[6] = 1;
    f[8..11].copy_from_slice(&[b'A', b'I', kind]);
    f[40..48].copy_from_slice(&(shoff as u64).to_le_bytes());
    f[58..60].copy_from_slice(&(entsize as u16).to_le_bytes());
    f[60..62].copy_from_slice(&(shnum as u16).to_le_bytes());
    // The last section, before the headers.
    let last = shoff + entsize * (shnum - 1);
    f[last + 24..last + 32].copy_from_slice(&2048u64.to_le_bytes());
    f[last + 32..last + 40].copy_from_slice(&1024u64.to_le_bytes());
    f
}

/// A type 2 AppImage of the SquashFS image `image`.
fn appimage_2(image: &[u8]) -> Vec<u8> {
    let mut file = program(2);
    file.extend_from_slice(image);
    file
}

/// A type 1 AppImage of the ISO 9660 image `iso`: its program in the
/// image's system area, its first 32 KiB, which genisoimage leaves zeros.
fn appimage_1(iso: &[u8]) -> Vec<u8> {
    let mut file = iso.to_vec();
    assert!(file[..32768].iter().all(|&b| b == 0));
    let program = program(1);
    file[..program.len()].copy_from_slice(&program);
    file
}

/// The test images' files, as `tests/data/README.md` makes them:
/// `docs/notes.txt`, `docs/plans/a.txt` and `readme.md`, changed last at
/// 2026-09-01 12:00 UTC.
const NOTES: &[u8] = b"Notes from inside the archive.\n";
const IMAGE_TIME: i64 = 1_788_264_000;

/// `tests/data`'s Debian packages, snap and ISO 9660 image of the files
/// above, as packages, snaps and AppImages: name, data, layers, container.
fn packages() -> Vec<(&'static str, Vec<u8>, Vec<Layer>, Container)> {
    let sqfs = data("sample.sqfs");
    let iso = unzstd(&data("sample.iso.zst"));
    vec![
        (
            "sample_1.0_all.deb",
            data("sample-xz.deb"),
            vec![Layer::Deb, Layer::Xz],
            Container::Tar,
        ),
        (
            "zstd.deb",
            data("sample-zstd.deb"),
            vec![Layer::Deb, Layer::Zstd],
            Container::Tar,
        ),
        (
            "gzip.udeb",
            data("sample-gzip.deb"),
            vec![Layer::Deb, Layer::Gzip],
            Container::Tar,
        ),
        ("sample.snap", sqfs.clone(), vec![], Container::Squashfs),
        (
            "sample.sqfs.zst",
            zstd(&sqfs),
            vec![Layer::Zstd],
            Container::Squashfs,
        ),
        (
            "Sample.AppImage",
            appimage_2(&sqfs),
            vec![],
            Container::AppImageType2,
        ),
        (
            "Sample.AppImage.zst",
            zstd(&appimage_2(&sqfs)),
            vec![Layer::Zstd],
            Container::AppImageType2,
        ),
        (
            "Old.AppImage",
            appimage_1(&iso),
            vec![],
            Container::AppImageType1,
        ),
        (
            "Old.AppImage.zst",
            zstd(&appimage_1(&iso)),
            vec![Layer::Zstd],
            Container::AppImageType1,
        ),
    ]
}

fn children(l: &packwing_core::Listing, folder: &[&str]) -> Vec<(String, bool, u64, u64)> {
    let folder: Vec<String> = folder.iter().map(|&c| c.to_owned()).collect();
    l.children(&folder)
        .into_iter()
        .map(|c| (c.name, c.dir, c.size, c.files))
        .collect()
}

fn quiet_with(progress: Option<hivepack::extract::ProgressFn>, cancel: Option<Cancel>) -> Control {
    Control {
        progress,
        cancel,
        on_damaged: None,
        on_unnamable: None,
        on_links_outside: None,
        on_gone: None,
    }
}

#[test]
fn packages_snaps_and_appimages_list() {
    let s = Scratch::new();
    for (name, bytes, layers, container) in packages() {
        let l = list(s.file(name, &bytes), &Keys::default()).unwrap();
        assert_eq!(l.packed, bytes.len() as u64, "{name}");
        assert_eq!((&l.layers, l.container), (&layers, container), "{name}");
        assert_eq!(l.encryption, None, "{name}");
        assert_eq!(l.files, 3, "{name}");
        assert_eq!(l.unpacked, (NOTES.len() + 4 + 10) as u64, "{name}");
        assert_eq!(
            children(&l, &[]),
            [
                ("docs".to_owned(), true, NOTES.len() as u64 + 4, 2),
                ("readme.md".to_owned(), false, 10, 1)
            ],
            "{name}"
        );
        assert_eq!(
            children(&l, &["docs"]),
            [
                ("plans".to_owned(), true, 4, 1),
                ("notes.txt".to_owned(), false, NOTES.len() as u64, 1)
            ],
            "{name}"
        );
        let readme = l.entries.iter().find(|e| e.path == ["readme.md"]).unwrap();
        assert_eq!(readme.modified, Some(IMAGE_TIME), "{name}");
    }
}

#[test]
fn packages_snaps_and_appimages_extract_and_test() {
    let s = Scratch::new();
    for (name, bytes, _, _) in packages() {
        let path = s.file(name, &bytes);
        let out = s.0.join(format!("out-{name}"));
        let last = Arc::new(Mutex::new(Progress::default()));
        let seen = last.clone();
        let control = quiet_with(
            Some(Arc::new(move |p: &Progress| *seen.lock().unwrap() = *p)),
            None,
        );
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
        // A package's control files are not what it installs.
        assert!(!out.join("control").exists(), "{name}");
        let p = *last.lock().unwrap();
        assert_eq!(p.bytes, p.total_bytes, "{name}");
        assert_eq!(p.bytes, (NOTES.len() + 14) as u64, "{name}");
        // Not over what is there.
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
fn what_is_picked_is_extracted_and_a_file_read() {
    let s = Scratch::new();
    for (name, bytes, _, _) in packages() {
        let path = s.file(name, &bytes);
        let out = s.0.join(format!("picked-{name}"));
        let picked = [
            vec!["docs".to_owned(), "plans".to_owned()],
            vec!["readme.md".to_owned()],
        ];
        let done = extract_selected(
            &path,
            &out,
            &picked,
            &Keys::default(),
            &ExtractOptions::default(),
            &Control::default(),
        )
        .unwrap();
        assert_eq!(done.files, 2, "{name}");
        assert_eq!(std::fs::read(out.join("plans/a.txt")).unwrap(), b"aaaa");
        assert_eq!(
            std::fs::read(out.join("readme.md")).unwrap(),
            b"# Read me\n"
        );
        let mut there: Vec<String> = std::fs::read_dir(&out)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        there.sort();
        assert_eq!(there, ["plans", "readme.md"], "{name}: nothing else left");
        let entry = ["docs".to_owned(), "notes.txt".to_owned()];
        let read = read_entry(
            &path,
            &entry,
            &Keys::default(),
            1 << 20,
            &Control::default(),
        );
        assert_eq!(read.unwrap(), NOTES, "{name}");
    }
}

#[test]
fn packages_snaps_and_appimages_convert_entry_by_entry() {
    use packwing_core::{Filter, Kind, NewArchive, convert};
    let s = Scratch::new();
    for (name, bytes, _, _) in packages() {
        let from = s.file(name, &bytes);
        let to = s.0.join(format!("{name}.zip"));
        let new = NewArchive {
            kind: Kind::Zip,
            ..NewArchive::default()
        };
        let done = convert(
            &from,
            &Keys::default(),
            &to,
            &new,
            &ExtractOptions::default(),
            &Filter::default(),
            &Control::default(),
        )
        .unwrap();
        assert_eq!(done.summary.files, 3, "{name}");
        let l = list(&done.made, &Keys::default()).unwrap();
        assert_eq!(l.container, Container::Zip, "{name}");
        assert_eq!(l.files, 3, "{name}");
        let out = s.0.join(format!("out-{name}"));
        extract(&done.made, &out, &Keys::default(), &Control::default()).unwrap();
        assert_eq!(std::fs::read(out.join("docs/notes.txt")).unwrap(), NOTES);
    }
}

#[test]
fn packages_snaps_and_appimages_cancelled_before_they_start() {
    let s = Scratch::new();
    let cancel = Cancel::new();
    cancel.cancel();
    let control = quiet_with(None, Some(cancel));
    for (name, bytes, _, _) in packages() {
        let path = s.file(name, &bytes);
        let out = s.0.join(format!("out-{name}"));
        let e = extract(&path, &out, &Keys::default(), &control).unwrap_err();
        assert!(matches!(e, Error::Cancelled), "{name}: {e:?}");
        let e = test(&path, &Keys::default(), &control).unwrap_err();
        assert!(matches!(e, Error::Cancelled), "{name}: {e:?}");
    }
}

#[test]
fn one_cut_short_fails_and_writes_nothing() {
    let s = Scratch::new();
    for (name, bytes, _, _) in packages() {
        // Cut before its last bytes: a SquashFS image's tables are at its
        // end, a package's data.tar too; an ISO 9660 image where the files'
        // data starts, its descriptors there.
        let at = bytes
            .windows(NOTES.len())
            .position(|w| w == NOTES)
            .unwrap_or(bytes.len() * 3 / 4);
        let cut = &bytes[..at];
        let path = s.file(name, cut);
        let out = s.0.join(format!("out-{name}"));
        let e = extract(&path, &out, &Keys::default(), &Control::default()).unwrap_err();
        assert!(matches!(e, Error::Failed(_)), "{name}: {e:?}");
        assert!(!out.join("docs/notes.txt").exists(), "{name}");
        let e = test(&path, &Keys::default(), &Control::default()).unwrap_err();
        assert!(matches!(e, Error::Failed(_)), "{name}: {e:?}");
    }
}

/// A file on a storage: what it reads is fetched by ranges, as from S3.
fn stored(s: &Scratch, name: &str, bytes: &[u8]) -> Source {
    s.file(name, bytes);
    Source::Stored(Stored::new(
        Arc::new(LocalFolder::new(&s.0)),
        vec![name.to_owned()],
        bytes.len() as u64,
    ))
}

#[test]
fn on_a_storage_they_are_read_where_they_are() {
    let s = Scratch::new();
    for (name, bytes, layers, container) in packages() {
        let source = stored(&s, name, &bytes);
        let l = list(source.clone(), &Keys::default()).unwrap();
        assert_eq!((&l.layers, l.container), (&layers, container), "{name}");
        assert_eq!(l.files, 3, "{name}");
        let entry = ["docs".to_owned(), "notes.txt".to_owned()];
        let read = read_entry(
            source.clone(),
            &entry,
            &Keys::default(),
            1 << 20,
            &Control::default(),
        );
        assert_eq!(read.unwrap(), NOTES, "{name}");
        let summary = test(source, &Keys::default(), &Control::default()).unwrap();
        assert_eq!(summary.files, 3, "{name}");
    }
}

#[test]
fn a_snap_of_each_compressor_lists_tests_and_extracts() {
    let s = Scratch::new();
    // hivepack's images: blocks.txt (two blocks of 4 KiB and a tail),
    // hello.txt and its second name dir/hardlink.txt, dir/inner.txt, a
    // link to hello.txt, a block device and a FIFO.
    for compressor in ["gzip", "lzma", "lzo", "xz", "lz4", "zstd", "uncompressed"] {
        let name = format!("{compressor}.snap");
        let path = s.file(&name, &data(&format!("squashfs/{compressor}.sqfs")));
        let l = list(&path, &Keys::default()).unwrap();
        assert_eq!(l.container, Container::Squashfs, "{name}");
        let blocks = l.entries.iter().find(|e| e.path == ["blocks.txt"]).unwrap();
        assert_eq!(blocks.size, 9000, "{name}");
        let link = l.entries.iter().find(|e| e.path == ["link"]).unwrap();
        assert_eq!(
            link.link.as_ref().and_then(|l| l.target.as_deref()),
            Some("hello.txt"),
            "{name}"
        );
        let summary = test(&path, &Keys::default(), &Control::default()).unwrap();
        // A second name is read once.
        assert_eq!(summary.files, 3, "{name}");
        // A device is never made: it stops the extraction before anything is
        // written, as a tar archive's does.
        let out = s.0.join(format!("out-{compressor}"));
        let e = extract(&path, &out, &Keys::default(), &Control::default()).unwrap_err();
        assert!(e.to_string().contains("device"), "{name}: {e}");
        assert!(!out.join("hello.txt").exists(), "{name}");
        // What is picked around them comes out.
        let picked: Vec<Vec<String>> = ["blocks.txt", "dir", "hello.txt", "link"]
            .iter()
            .map(|&n| vec![n.to_owned()])
            .collect();
        let done = extract_selected(
            &path,
            &out,
            &picked,
            &Keys::default(),
            &ExtractOptions::default(),
            &Control::default(),
        )
        .unwrap();
        // On Windows the link comes out as a file holding its target.
        let files = if cfg!(windows) { 5 } else { 4 };
        assert_eq!(done.files, files, "{name}");
        let blocks = std::fs::read(out.join("blocks.txt")).unwrap();
        assert_eq!(blocks.len(), 9000, "{name}");
        let hello = std::fs::read(out.join("hello.txt")).unwrap();
        assert_eq!(std::fs::read(out.join("dir/hardlink.txt")).unwrap(), hello);
        assert!(out.join("device").symlink_metadata().is_err(), "{name}");
        assert!(out.join("fifo").symlink_metadata().is_err(), "{name}");
    }
}

#[test]
fn a_snaps_links_holes_and_modes_come_out_safe() {
    let s = Scratch::new();
    let image = data("links.sqfs");
    for (name, bytes) in [
        ("links.snap", image.clone()),
        ("Links.AppImage", appimage_2(&image)),
    ] {
        let path = s.file(name, &bytes);
        let l = list(&path, &Keys::default()).unwrap();
        let size = |n: &str| {
            l.entries
                .iter()
                .find(|e| e.path.join("/") == n)
                .unwrap()
                .size
        };
        // A second name has no size: its data is counted once, at the first
        // by name.
        assert_eq!(size("hello-again.txt"), 19, "{name}");
        assert_eq!(size("hello.txt"), 0, "{name}");
        assert_eq!(size("sparse.bin"), 8196, "{name}");
        let target = |n: &str| {
            l.entries
                .iter()
                .find(|e| e.path.join("/") == n)
                .and_then(|e| e.link.clone())
                .and_then(|l| l.target)
        };
        assert_eq!(target("link-inside").as_deref(), Some("hello.txt"));
        assert_eq!(target("link-outside").as_deref(), Some("/etc/passwd"));
        assert_eq!(l.unpacked, 19 + 19 + 8196, "{name}");

        let summary = test(&path, &Keys::default(), &Control::default()).unwrap();
        assert_eq!(
            (summary.files, summary.bytes),
            (3, 19 + 19 + 8196),
            "{name}"
        );

        let out = s.0.join(format!("out-{name}"));
        extract(&path, &out, &Keys::default(), &Control::default()).unwrap();
        let hello = std::fs::read(out.join("hello.txt")).unwrap();
        assert_eq!(hello, b"Hello from a snap.\n");
        assert_eq!(std::fs::read(out.join("hello-again.txt")).unwrap(), hello);
        // The hole reads as zeros.
        let mut sparse = vec![0u8; 8192];
        sparse.extend(b"end\n");
        assert_eq!(std::fs::read(out.join("sparse.bin")).unwrap(), sparse);
        #[cfg(unix)]
        {
            // A link that leads outside is left out, by default.
            assert!(
                out.join("link-outside").symlink_metadata().is_err(),
                "{name}"
            );
            use std::os::unix::fs::{MetadataExt, PermissionsExt};
            let meta = |n: &str| out.join(n).symlink_metadata().unwrap();
            // Another name of one file is a hard link to it.
            assert_eq!(meta("hello.txt").ino(), meta("hello-again.txt").ino());
            assert!(meta("link-inside").file_type().is_symlink(), "{name}");
            assert_eq!(
                std::fs::read_link(out.join("link-inside")).unwrap(),
                Path::new("hello.txt")
            );
            // A program runs, but not as its owner: setuid is masked.
            let mode = meta("bin/run").permissions().mode();
            assert_eq!(mode & 0o7000, 0, "{name}: {mode:o}");
            assert_eq!(mode & 0o100, 0o100, "{name}: {mode:o}");
        }
    }
}
