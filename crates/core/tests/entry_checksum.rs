//! The checksums of the files in an archive, made as it is read, with nothing
//! written; the list of them, which checks what is extracted; and a list given,
//! compared with them. See crates/core/src/entry_checksum.rs.

#![allow(clippy::unwrap_used)]

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use packwing_core::{
    Algorithm, Control, Damage, Error, ExtractOptions, Keys, LocalFolder, OnDamaged, Verdict,
    checksum_list, checksum_list_in, checksums_of, compare_list, entry_checksums, extract_selected,
    save_checksum_list,
};

/// A folder in the system's temporary one, removed when dropped.
struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Scratch {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "packwing-entry-checksum-{}-{}",
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

/// What a tar archive's entry is.
enum Kind<'a> {
    File(&'a [u8]),
    Folder,
    Symlink(&'a str),
    HardLink(&'a str),
}

/// A USTAR archive of these.
fn tar(entries: &[(&str, Kind<'_>)]) -> Vec<u8> {
    let mut out = Vec::new();
    for (name, kind) in entries {
        let (flag, data, link): (u8, &[u8], &str) = match kind {
            Kind::File(data) => (b'0', data, ""),
            Kind::Folder => (b'5', b"", ""),
            Kind::Symlink(to) => (b'2', b"", to),
            Kind::HardLink(to) => (b'1', b"", to),
        };
        let mut b = vec![0u8; 512];
        b[..name.len()].copy_from_slice(name.as_bytes());
        b[100..108].copy_from_slice(if flag == b'5' {
            b"0000755\0"
        } else {
            b"0000644\0"
        });
        b[108..116].copy_from_slice(b"0001750\0");
        b[116..124].copy_from_slice(b"0001750\0");
        b[124..136].copy_from_slice(format!("{:011o}\0", data.len()).as_bytes());
        b[136..148].copy_from_slice(b"15000000000\0");
        b[156] = flag;
        b[157..157 + link.len()].copy_from_slice(link.as_bytes());
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

fn sha256(data: &[u8]) -> String {
    checksums_of(data, &[Algorithm::Sha256], &Control::default())
        .unwrap()
        .remove(0)
        .1
}

fn path(p: &str) -> Vec<String> {
    p.split('/').map(str::to_owned).collect()
}

const SAMPLE: [(&str, &[u8]); 3] = [
    ("docs/notes.txt", b"notes"),
    ("docs/plans/a.txt", b"aaaa"),
    ("readme.md", b"# Read me\n"),
];

#[test]
fn every_file_has_its_own_and_links_have_none() {
    let s = Scratch::new();
    let data = zstd(&tar(&[
        ("docs/", Kind::Folder),
        ("docs/notes.txt", Kind::File(b"notes")),
        ("docs/plans/", Kind::Folder),
        ("docs/plans/a.txt", Kind::File(b"aaaa")),
        ("readme.md", Kind::File(b"# Read me\n")),
        ("docs/again.txt", Kind::HardLink("docs/notes.txt")),
        ("docs/latest", Kind::Symlink("plans/a.txt")),
    ]));
    let archive = s.file("a.tar.zst", &data);
    let made = entry_checksums(
        &archive,
        &[],
        &Keys::default(),
        &[Algorithm::Sha256, Algorithm::Blake3],
        &Control::default(),
    )
    .unwrap();
    let named: Vec<(&str, u64, Option<&str>)> = made
        .files
        .iter()
        .map(|f| (f.name.as_str(), f.size, f.sum(Algorithm::Sha256)))
        .collect();
    let (notes, a, readme) = (sha256(b"notes"), sha256(b"aaaa"), sha256(b"# Read me\n"));
    assert_eq!(
        named,
        [
            // A hard link is the file it is another name for, as it is extracted.
            ("docs/again.txt", 5, Some(notes.as_str())),
            ("docs/notes.txt", 5, Some(notes.as_str())),
            ("docs/plans/a.txt", 4, Some(a.as_str())),
            ("readme.md", 10, Some(readme.as_str())),
        ]
    );
    assert_eq!(made.links, 1);
    assert!(made.damaged.is_empty());
    for f in &made.files {
        assert_eq!(f.path, path(&f.name));
        assert_eq!(f.sums.len(), 2, "{}", f.name);
        assert_eq!(f.sums[1].0, Algorithm::Blake3);
    }
}

#[test]
fn what_is_picked_is_named_as_it_is_extracted_and_its_list_checks_it() {
    let s = Scratch::new();
    let archive = s.file("a.zip", &zip(&SAMPLE));
    let picked = [path("docs/plans"), path("readme.md")];
    let made = entry_checksums(
        &archive,
        &picked,
        &Keys::default(),
        &[Algorithm::Sha256],
        &Control::default(),
    )
    .unwrap();
    let names: Vec<&str> = made.files.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, ["plans/a.txt", "readme.md"]);
    assert_eq!(made.files[0].path, path("docs/plans/a.txt"));

    // Extracted as the app and pkw extract what is picked, the list holds what
    // came out, by the names it came out as.
    let out = s.0.join("out");
    extract_selected(
        &archive,
        &out,
        &picked,
        &Keys::default(),
        &ExtractOptions::default(),
        &Control::default(),
    )
    .unwrap();
    let list = checksum_list(&made.files, Algorithm::Sha256);
    for line in checksum_list_in(&list) {
        let data = std::fs::read(out.join(&line.name)).unwrap();
        assert_eq!(sha256(&data), line.sum, "{}", line.name);
    }
    std::fs::write(out.join("SUMS"), &list).unwrap();
    match std::process::Command::new("sha256sum")
        .args(["-c", "SUMS"])
        .current_dir(&out)
        .output()
    {
        Ok(done) => assert!(done.status.success(), "sha256sum: {done:?}"),
        Err(_) => eprintln!("no sha256sum: skipped"),
    }
}

#[test]
fn a_damaged_file_is_left_out_and_said_and_the_rest_are_made() {
    let s = Scratch::new();
    let mut data = zip(&[("a.txt", b"aaaa"), ("b.txt", b"bbbb"), ("c.txt", b"cccc")]);
    let at = data.windows(4).position(|w| w == b"bbbb").unwrap();
    data[at] = b'x';
    let archive = s.file("a.zip", &data);
    // Whatever the control says of damage: a checksum is of all of a file.
    let control = Control {
        on_damaged: Some(Arc::new(|_: &Damage| OnDamaged::Keep)),
        ..Control::default()
    };
    let made = entry_checksums(
        &archive,
        &[],
        &Keys::default(),
        &[Algorithm::Sha256],
        &control,
    )
    .unwrap();
    let names: Vec<&str> = made.files.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, ["a.txt", "c.txt"]);
    assert_eq!(made.damaged.len(), 1);
    assert_eq!(made.damaged[0].path, "b.txt");
}

#[test]
fn a_stopped_read_says_so() {
    let s = Scratch::new();
    let archive = s.file("a.zip", &zip(&SAMPLE));
    let cancel = packwing_core::Cancel::new();
    cancel.cancel();
    let control = Control {
        cancel: Some(cancel),
        ..Control::default()
    };
    let e = entry_checksums(
        &archive,
        &[],
        &Keys::default(),
        &[Algorithm::Sha256],
        &control,
    )
    .unwrap_err();
    assert!(matches!(e, Error::Cancelled), "{e:?}");
}

#[test]
fn the_list_kept_beside_the_archive_matches_what_it_was_made_of() {
    let s = Scratch::new();
    s.file("a.zip", &zip(&SAMPLE));
    let folder = LocalFolder::new(&s.0);
    let made = entry_checksums(
        s.0.join("a.zip"),
        &[],
        &Keys::default(),
        &Algorithm::ALL,
        &Control::default(),
    )
    .unwrap();
    for a in Algorithm::ALL {
        let kept = save_checksum_list(&folder, &["a.zip".to_owned()], a, &made.files).unwrap();
        assert_eq!(kept, format!("a.zip.contents.{}", a.key()));
        let text = std::fs::read_to_string(s.0.join(&kept)).unwrap();
        assert_eq!(text.lines().count(), SAMPLE.len(), "{kept}");
        let compared = compare_list(&made.files, &checksum_list_in(&text));
        assert!(compared.all_match(), "{kept}: {compared:?}");
    }
    // One changed, one gone, and one more than the archive has.
    let text = std::fs::read_to_string(s.0.join("a.zip.contents.sha256")).unwrap();
    let mut listed = checksum_list_in(&text);
    listed[0].sum = sha256(b"other");
    listed.remove(1);
    listed.push(packwing_core::Listed {
        sum: sha256(b"x"),
        name: "extra.txt".into(),
    });
    let compared = compare_list(&made.files, &listed);
    assert_eq!(
        compared.verdicts,
        [
            Verdict::Differs {
                made: vec![Algorithm::Sha256, Algorithm::Blake3],
                not_made: vec![]
            },
            Verdict::NotListed,
            Verdict::Matches(Algorithm::Sha256),
        ]
    );
    assert_eq!(compared.not_found, ["extra.txt"]);
}
