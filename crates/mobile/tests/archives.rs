//! What the iOS app does with archives, through the bindings: made, opened
//! (with and without their passwords), searched, read and extracted.

use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

use packwing_mobile::{
    Archive, ArchiveFormat, Checksum, ChecksumKind, Compared, CompressionLevel, Job, JobListener,
    JobProgress, MobileError, NewArchiveHow, Secrets, checksum_kinds, compare_checksum,
    create_archive, stem,
};

/// A folder of files to make archives of: `Báo cáo/tổng hợp.txt` and
/// `Báo cáo/ghi chú.txt`.
fn files(dir: &Path) -> String {
    let folder = dir.join("Báo cáo");
    fs::create_dir_all(&folder).expect("folder");
    fs::write(folder.join("tổng hợp.txt"), "doanh thu quý 3\n".repeat(200)).expect("file");
    fs::write(folder.join("ghi chú.txt"), "hello").expect("file");
    folder.display().to_string()
}

fn make(dir: &Path, format: ArchiveFormat, password: Option<&str>, hide_names: bool) -> String {
    let name = match format {
        ArchiveFormat::Zip => "a.zip",
        ArchiveFormat::SevenZip => "a.7z",
    };
    create_archive(
        vec![files(dir)],
        dir.join(name).display().to_string(),
        NewArchiveHow {
            format,
            compression: CompressionLevel::Low,
            password: password.map(str::to_owned),
            hide_names,
        },
        Job::quiet(),
    )
    .expect("made")
}

fn with(password: &str) -> Secrets {
    Secrets {
        password: Some(password.to_owned()),
        passphrase: None,
    }
}

#[test]
fn a_7z_archive_with_hidden_names_asks_for_its_password() {
    let dir = tempfile::tempdir().expect("dir");
    let path = make(
        dir.path(),
        ArchiveFormat::SevenZip,
        Some("sample-password"),
        true,
    );

    let none = Archive::open(path.clone(), Secrets::default(), Job::quiet());
    assert!(
        matches!(
            none,
            Err(MobileError::Locked {
                passphrase: false,
                ..
            })
        ),
        "{:?}",
        none.err()
    );
    let wrong = Archive::open(path.clone(), with("not it"), Job::quiet());
    assert!(
        matches!(wrong, Err(MobileError::WrongPassword { .. })),
        "{:?}",
        wrong.err()
    );

    let archive = Archive::open(path, with("sample-password"), Job::quiet()).expect("opened");
    let info = archive.info();
    assert_eq!(info.files, 2);
    assert!(info.encryption.is_some());
    let top = archive.children(vec![]);
    assert_eq!(top.len(), 1);
    assert!(top[0].folder);
    assert_eq!(top[0].name, "Báo cáo");
    assert_eq!(top[0].files, 2);
    let names: Vec<String> = archive
        .children(vec!["Báo cáo".into()])
        .into_iter()
        .map(|e| e.name)
        .collect();
    assert_eq!(names, ["ghi chú.txt", "tổng hợp.txt"]);

    let out = dir.path().join("out");
    let done = archive
        .extract(
            out.display().to_string(),
            vec![],
            Default::default(),
            Job::quiet(),
        )
        .expect("extracted");
    assert_eq!(done.files, 2);
    assert_eq!(
        fs::read_to_string(out.join("Báo cáo/ghi chú.txt")).expect("read"),
        "hello"
    );
}

#[test]
fn a_search_finds_names_whatever_their_accents() {
    let dir = tempfile::tempdir().expect("dir");
    let archive = Archive::open(
        make(dir.path(), ArchiveFormat::Zip, None, false),
        Secrets::default(),
        Job::quiet(),
    )
    .expect("opened");
    let found = archive.search("TONG HOP".into());
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].path, ["Báo cáo", "tổng hợp.txt"]);
    assert!(!found[0].entry.folder);
}

#[test]
fn a_file_picked_is_read_and_extracted_on_its_own() {
    let dir = tempfile::tempdir().expect("dir");
    let archive = Archive::open(
        make(
            dir.path(),
            ArchiveFormat::Zip,
            Some("sample-password"),
            false,
        ),
        with("sample-password"),
        Job::quiet(),
    )
    .expect("opened");
    let picked = vec!["Báo cáo".to_owned(), "ghi chú.txt".to_owned()];

    let bytes = archive
        .preview(picked.clone(), 1 << 20, Job::quiet())
        .expect("read");
    assert_eq!(bytes, b"hello");

    let out = dir.path().join("out");
    let how = Default::default();
    let done = archive
        .extract(
            out.display().to_string(),
            vec![picked.clone()],
            how,
            Job::quiet(),
        )
        .expect("extracted");
    assert_eq!(done.files, 1);
    assert_eq!(
        fs::read_to_string(out.join("ghi chú.txt")).expect("read"),
        "hello"
    );
    // Not over what is there, unless asked.
    assert!(
        archive
            .extract(out.display().to_string(), vec![picked], how, Job::quiet())
            .is_err()
    );
}

#[test]
fn a_new_archive_takes_a_free_name_beside_one_taken() {
    let dir = tempfile::tempdir().expect("dir");
    let first = make(dir.path(), ArchiveFormat::Zip, None, false);
    let second = make(dir.path(), ArchiveFormat::Zip, None, false);
    assert_ne!(first, second);
    assert!(Path::new(&second).exists());
}

#[derive(Default)]
struct Told(Mutex<Vec<JobProgress>>);

impl JobListener for Told {
    fn progress(&self, progress: JobProgress) {
        self.0.lock().expect("lock").push(progress);
    }
}

#[test]
fn a_job_says_how_far_it_is_and_when_it_is_through() {
    let dir = tempfile::tempdir().expect("dir");
    let archive = Archive::open(
        make(dir.path(), ArchiveFormat::SevenZip, None, false),
        Secrets::default(),
        Job::quiet(),
    )
    .expect("opened");
    let told = Arc::new(Told::default());
    archive
        .extract(
            dir.path().join("out").display().to_string(),
            vec![],
            Default::default(),
            Job::new(told.clone()),
        )
        .expect("extracted");
    let told = told.0.lock().expect("lock");
    let last = told.last().expect("told");
    assert_eq!(last.bytes, last.total_bytes);
    assert_eq!(last.total_bytes, archive.info().unpacked);
}

#[test]
fn a_job_cancelled_stops_and_leaves_nothing() {
    let dir = tempfile::tempdir().expect("dir");
    let archive = Archive::open(
        make(dir.path(), ArchiveFormat::Zip, None, false),
        Secrets::default(),
        Job::quiet(),
    )
    .expect("opened");
    let job = Job::quiet();
    job.cancel();
    let out = dir.path().join("out");
    let stopped = archive.extract(out.display().to_string(), vec![], Default::default(), job);
    assert!(
        matches!(stopped, Err(MobileError::Cancelled { .. })),
        "{stopped:?}"
    );
    assert!(fs::read_dir(&out).map_or(true, |mut d| d.next().is_none()));
}

#[test]
fn a_file_opened_is_a_copy_of_its_own() {
    let dir = tempfile::tempdir().expect("dir");
    let archive = Archive::open(
        make(dir.path(), ArchiveFormat::SevenZip, None, false),
        Secrets::default(),
        Job::quiet(),
    )
    .expect("opened");
    let copy = archive
        .open_copy(vec!["Báo cáo".into(), "ghi chú.txt".into()], Job::quiet())
        .expect("copied");
    assert!(copy.ends_with("ghi chú.txt"), "{copy}");
    assert_eq!(fs::read_to_string(&copy).expect("read"), "hello");
    let _ = fs::remove_dir_all(Path::new(&copy).parent().expect("folder"));
}

#[test]
fn a_folder_extracted_is_named_without_the_extensions() {
    assert_eq!(stem("Báo cáo.tar.gz".into()), "Báo cáo");
    assert_eq!(stem("notes".into()), "notes");
}

#[test]
fn an_archive_s_checksums_are_those_of_its_file_read_once() {
    let dir = tempfile::tempdir().expect("dir");
    let path = make(dir.path(), ArchiveFormat::Zip, None, false);
    let archive = Archive::open(path.clone(), Secrets::default(), Job::quiet()).expect("opened");
    let made = archive
        .checksums(vec![ChecksumKind::Sha256, ChecksumKind::Md5], Job::quiet())
        .expect("made");

    let file = fs::File::open(&path).expect("file");
    let control = packwing_core::Control::default();
    let expected = packwing_core::checksums_of(
        file,
        &[
            packwing_core::Algorithm::Sha256,
            packwing_core::Algorithm::Md5,
        ],
        &control,
    )
    .expect("read");
    assert_eq!(
        made,
        vec![
            Checksum {
                kind: ChecksumKind::Sha256,
                sum: expected[0].1.clone()
            },
            Checksum {
                kind: ChecksumKind::Md5,
                sum: expected[1].1.clone()
            },
        ]
    );
    assert_eq!(made[0].sum.len(), 64);
}

#[test]
fn a_checksum_pasted_matches_differs_or_waits() {
    let sha256 = "a".repeat(64);
    let made = vec![Checksum {
        kind: ChecksumKind::Sha256,
        sum: sha256.clone(),
    }];
    let compare = |pasted: &str, making: Vec<ChecksumKind>| {
        compare_checksum(pasted.to_owned(), "a.zip".to_owned(), made.clone(), making)
    };

    assert_eq!(compare("  ", vec![]), None);
    assert_eq!(compare("hello", vec![]), Some(Compared::NotOne));
    assert_eq!(
        compare(&format!("{}  a.zip", sha256.to_uppercase()), vec![]),
        Some(Compared::Matches {
            kind: ChecksumKind::Sha256
        })
    );
    // As long as SHA-256's and BLAKE3's: not SHA-256's, perhaps BLAKE3's.
    assert_eq!(
        compare(&"b".repeat(64), vec![]),
        Some(Compared::Differs {
            made: vec![ChecksumKind::Sha256],
            not_made: vec![ChecksumKind::Blake3],
        })
    );
    assert_eq!(
        compare(&"b".repeat(40), vec![ChecksumKind::Sha1]),
        Some(Compared::Waiting)
    );
    assert_eq!(
        compare(&"b".repeat(50), vec![]),
        Some(Compared::Unknown { digits: 50 })
    );
    let kinds = checksum_kinds();
    assert_eq!(kinds[0].name, "SHA-256");
    assert!(kinds.iter().any(|k| k.kind == ChecksumKind::Md5 && k.weak));
}
