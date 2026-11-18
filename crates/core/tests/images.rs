//! Disk images: the partitions a table names, and the file systems in them.
//!
//! The committed image is a GPT disk of three partitions — an EFI system
//! partition holding FAT (`qemu-img`'s vvfat wrote it from a folder), a
//! Linux one holding ext2 (`mke2fs -d` wrote it), and a swap partition,
//! which holds no file system at all. Zstandard takes the 10 MiB disk to
//! 1.5 KB, since the rest of it is zeros, so the tests that matter need no
//! tool and run everywhere. The ones that take a tool (NTFS, and the
//! containers QEMU writes) say so and are skipped.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use packwing_core::{
    Container, Control, Entry, ExtractOptions, Keys, Layer, extract, extract_selected, list, test,
};

/// A folder in the system's temporary one, removed when dropped.
struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Scratch {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "packwing-images-{}-{}",
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

fn unzstd(bytes: &[u8]) -> Vec<u8> {
    let hivepack::zstd::ContentSize::Known(size) = hivepack::zstd::find_decompressed_size(bytes)
    else {
        panic!("the frames do not say their size");
    };
    let mut out = vec![0u8; size as usize];
    let read = hivepack::zstd::decompress(&mut out, bytes).unwrap();
    out.truncate(read);
    out
}

/// The committed card image, as it is.
fn card() -> Vec<u8> {
    unzstd(&data("card.img.zst"))
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

/// A tool, if it is installed and can be run.
fn tool(name: &str) -> Option<PathBuf> {
    let at = ["/usr/sbin", "/usr/bin", "/sbin", "/bin", "/usr/local/sbin"]
        .into_iter()
        .map(|dir| PathBuf::from(dir).join(name))
        .find(|at| at.exists())?;
    Command::new(&at).arg("--version").output().ok()?;
    Some(at)
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

/// A card image's partitions are folders, named by their number and what
/// they are for or hold, and the files of each file system are under them.
/// A partition packwing reads no file system in is one file of its bytes.
#[test]
fn a_disk_image_lists_its_partitions_as_folders() {
    let s = Scratch::new();
    let path = s.file("card.img", &card());
    let listing = list(&path, &Keys::default()).unwrap();
    assert_eq!(listing.container, Container::PartitionedDisk);
    assert!(listing.layers.is_empty());
    assert_eq!(
        paths(&listing.entries),
        [
            "1-EFI/",
            "1-EFI/README.TXT",
            "1-EFI/docs/",
            "1-EFI/docs/note.txt",
            "2-ext2/",
            "2-ext2/etc/",
            "2-ext2/etc/deeper.conf",
            "2-ext2/etc/hostname",
            "2-ext2/lost+found/",
            "3-swap.img",
        ]
    );
    // The swap partition's entry is as long as the partition is: a
    // mebibyte, which is what the table says.
    let swap = listing
        .entries
        .iter()
        .find(|entry| entry.path == ["3-swap.img"])
        .unwrap();
    assert_eq!(swap.size, 1 << 20);
    assert!(!swap.dir);
}

/// The same under a layer, which is how a card is kept: the image is
/// written to a file first, as a UDF volume is.
#[test]
fn a_disk_image_under_a_layer_lists_the_same() {
    let s = Scratch::new();
    let path = s.file("card.img.zst", &data("card.img.zst"));
    let listing = list(&path, &Keys::default()).unwrap();
    assert_eq!(listing.container, Container::PartitionedDisk);
    assert_eq!(listing.layers, vec![Layer::Zstd]);
    assert!(
        paths(&listing.entries).contains(&"2-ext2/etc/hostname".to_owned()),
        "{:?}",
        paths(&listing.entries)
    );
}

/// Extracted, the partitions are folders on disk, each file where the
/// listing said it is, and the swap partition one file of its bytes.
#[test]
fn a_disk_image_extracts_its_partitions_into_folders() {
    let s = Scratch::new();
    let path = s.file("card.img", &card());
    let out = s.0.join("out");
    let summary = extract(&path, &out, &Keys::default(), &Control::default()).unwrap();
    assert_eq!(
        written(&out),
        [
            "1-EFI/",
            "1-EFI/README.TXT",
            "1-EFI/docs/",
            "1-EFI/docs/note.txt",
            "2-ext2/",
            "2-ext2/etc/",
            "2-ext2/etc/deeper.conf",
            "2-ext2/etc/hostname",
            "2-ext2/lost+found/",
            "3-swap.img",
        ]
    );
    assert_eq!(
        std::fs::read(out.join("1-EFI/README.TXT")).unwrap(),
        b"hello from the boot partition\n"
    );
    assert_eq!(
        std::fs::read(out.join("2-ext2/etc/hostname")).unwrap(),
        b"root file system\n"
    );
    assert_eq!(
        std::fs::metadata(out.join("3-swap.img")).unwrap().len(),
        1 << 20
    );
    // Four files and the swap partition's bytes; the folders of the two
    // file systems and `lost+found`.
    assert_eq!(summary.files, 5, "{summary:?}");
    assert!(summary.bytes >= 1 << 20, "{summary:?}");
}

/// One file picked out of one partition. `extract_selected` puts each
/// picked entry in the destination **by its last name**, as it does for
/// every other format — so what lands is `hostname`, not the partition's
/// folder — and nothing of the other partitions is touched.
#[test]
fn picking_one_file_out_of_one_partition_writes_that_file() {
    let s = Scratch::new();
    let path = s.file("card.img", &card());
    let out = s.0.join("out");
    let picked = vec![vec![
        "2-ext2".to_owned(),
        "etc".to_owned(),
        "hostname".to_owned(),
    ]];
    let summary = extract_selected(
        &path,
        &out,
        &picked,
        &Keys::default(),
        &ExtractOptions::default(),
        &Control::default(),
    )
    .unwrap();
    assert_eq!(written(&out), ["hostname"]);
    assert_eq!(
        std::fs::read(out.join("hostname")).unwrap(),
        b"root file system\n"
    );
    assert_eq!(summary.files, 1, "{summary:?}");
    // Only that partition was read: the others' file systems were not
    // walked, and no folder was made for them.
    assert_eq!(summary.bytes, b"root file system\n".len() as u64);
}

/// Tested, every file of every partition is read where its file system says
/// its bytes are, and the swap partition's bytes with them.
#[test]
fn a_disk_image_is_tested_through_its_partitions() {
    let s = Scratch::new();
    let path = s.file("card.img", &card());
    let summary = test(&path, &Keys::default(), &Control::default()).unwrap();
    assert_eq!(summary.files, 5, "{summary:?}");
    assert!(summary.bytes >= 1 << 20, "{summary:?}");
}

/// A file system with no table in front of it: the EFI system partition
/// taken off the card on its own, which lists at the root.
#[test]
fn a_file_system_on_its_own_lists_at_the_root() {
    let card = card();
    // Where the table says that partition is: the test knows, as the
    // listing above showed it.
    let at = 2048 * 512;
    let size = 5760 * 512;
    let s = Scratch::new();
    let path = s.file("boot.img", &card[at..at + size]);
    let listing = list(&path, &Keys::default()).unwrap();
    assert_eq!(listing.container, Container::Fat);
    assert_eq!(
        paths(&listing.entries),
        ["README.TXT", "docs/", "docs/note.txt"]
    );
    let out = s.0.join("out");
    extract(&path, &out, &Keys::default(), &Control::default()).unwrap();
    assert_eq!(
        std::fs::read(out.join("README.TXT")).unwrap(),
        b"hello from the boot partition\n"
    );
}

/// The same disk in the containers QEMU writes: a QCOW2 and a VHDX hold the
/// same partitions and the same files, read through the container.
#[test]
fn the_containers_hold_the_same_disk() {
    let Some(qemu) = tool("qemu-img") else {
        eprintln!("qemu-img cannot be run here: the containers are not checked");
        return;
    };
    let s = Scratch::new();
    let raw = s.file("card.img", &card());
    for (format, name, want) in [
        ("qcow2", "card.qcow2", Container::Qcow2),
        ("vhdx", "card.vhdx", Container::Vhdx),
        ("vpc", "card.vhd", Container::Vhd),
    ] {
        let path = s.0.join(name);
        let out = Command::new(&qemu)
            .args(["convert", "-O", format])
            .arg(&raw)
            .arg(&path)
            .output()
            .unwrap();
        if !out.status.success() {
            eprintln!(
                "qemu-img wrote no {format} here: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            );
            continue;
        }
        let listing = list(&path, &Keys::default()).unwrap();
        assert_eq!(listing.container, want, "{name}");
        assert!(
            paths(&listing.entries).contains(&"2-ext2/etc/hostname".to_owned()),
            "{name}: {:?}",
            paths(&listing.entries)
        );
    }
}

/// A disk whose partition holds NTFS, which packwing reads behind
/// `catch_unwind`: `mkntfs` writes the file system, `ntfs-3g` puts a file
/// in it where it can mount, and the listing shows the partition as `1-ntfs`
/// with that file under it.
///
/// Tool-gated, and it says so when it skips: `mkntfs` is in ntfs-3g's
/// package, and mounting takes FUSE and the permission to use it. The rules
/// this is about — that the partition is named for the file system, and
/// that a panic inside that library becomes an error — are checked without
/// any tool in `crates/core/src/image.rs`.
#[test]
fn a_partition_holding_ntfs_is_read() {
    let (Some(mkntfs), Some(sfdisk)) = (tool("mkntfs"), tool("sfdisk")) else {
        eprintln!("mkntfs or sfdisk cannot be run here: NTFS in a partition is not checked");
        return;
    };
    let s = Scratch::new();
    // 24 MiB: mkntfs refuses a volume too small to hold its own records.
    let disk = s.file("ntfs.img", &vec![0u8; 24 << 20]);
    let mut table = Command::new(&sfdisk)
        .arg(&disk)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    {
        use std::io::Write;
        table
            .stdin
            .as_mut()
            .unwrap()
            .write_all(b"label: gpt\nsize=40960, type=EBD0A0A2-B9E5-4433-87C0-68B6B72699C7\n")
            .unwrap();
    }
    if !table.wait().unwrap().success() {
        eprintln!("sfdisk wrote no table here: NTFS in a partition is not checked");
        return;
    }

    // The partition on its own, formatted, then put back where the table
    // says it is.
    let at = 2048 * 512;
    let size = 40960 * 512;
    let part = s.0.join("part.img");
    std::fs::write(&part, vec![0u8; size]).unwrap();
    let out = Command::new(&mkntfs)
        .args(["-F", "-q", "-L", "windows"])
        .arg(&part)
        .output()
        .unwrap();
    if !out.status.success() {
        eprintln!(
            "this mkntfs wrote no file system: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
        return;
    }
    // A file in it, where ntfs-3g can mount: without one the file system is
    // empty, and the partition's name is still what this test is about.
    let mut wrote = None;
    if let (Some(ntfs3g), Some(fusermount)) = (
        tool("ntfs-3g"),
        tool("fusermount3").or_else(|| tool("fusermount")),
    ) {
        let mount = s.0.join("mnt");
        std::fs::create_dir_all(&mount).unwrap();
        let mounted = Command::new(&ntfs3g)
            .arg(&part)
            .arg(&mount)
            .output()
            .unwrap();
        if mounted.status.success() {
            std::fs::write(mount.join("notes.txt"), b"from windows\n").unwrap();
            let _ = Command::new(&fusermount).arg("-u").arg(&mount).output();
            wrote = Some("notes.txt");
        } else {
            eprintln!(
                "this ntfs-3g could not mount the image, so the file system is empty: {}",
                String::from_utf8_lossy(&mounted.stderr).trim()
            );
        }
    }

    let mut bytes = std::fs::read(&disk).unwrap();
    bytes[at..at + size].copy_from_slice(&std::fs::read(&part).unwrap());
    let path = s.file("windows.img", &bytes);

    let listing = list(&path, &Keys::default()).unwrap();
    assert_eq!(listing.container, Container::PartitionedDisk);
    let found = paths(&listing.entries);
    assert!(found.contains(&"1-ntfs/".to_owned()), "{found:?}");
    if let Some(name) = wrote {
        assert!(found.contains(&format!("1-ntfs/{name}")), "{found:?}");
        let out = s.0.join("out");
        extract(&path, &out, &Keys::default(), &Control::default()).unwrap();
        assert_eq!(
            std::fs::read(out.join("1-ntfs").join(name)).unwrap(),
            b"from windows\n"
        );
    }
}
