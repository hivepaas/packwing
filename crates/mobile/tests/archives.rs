//! What the iOS app does with archives, through the bindings: made, opened
//! (with and without their passwords), searched, read and extracted.

use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

use packwing_mobile::{
    Archive, ArchiveFormat, Checksum, ChecksumKind, Compared, CompressionLevel, Job, JobListener,
    JobProgress, MobileError, NewArchiveHow, OnExisting, RemoteStorage, Secrets, SignatureState,
    SshQuestions, TypedSecret, ask_ssh_with, checksum_kinds, compare_checksum, create_archive,
    describe_storage, read_key, start, stem, storage_kinds,
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

#[test]
fn an_archive_s_signature_says_who_signed_it_only_by_a_key_trusted() {
    let dir = tempfile::tempdir().expect("dir");
    let path = make(dir.path(), ArchiveFormat::Zip, None, false);
    let keys = dir.path().join("keys");
    fs::create_dir_all(&keys).expect("keys");
    let (key, _) = packwing_core::make_signing_key(&keys, b"a passphrase").expect("a key");
    let control = packwing_core::Control::default();
    packwing_core::sign_file(
        &packwing_core::LocalFolder::new(dir.path()),
        &["a.zip".to_owned()],
        &key,
        &control,
    )
    .expect("signed");
    let signature = format!("{path}.sig");
    let mine = read_key(key.public().to_string()).expect("a key");
    assert_eq!(mine.fingerprint, key.public().fingerprint());

    let archive = Archive::open(path.clone(), Secrets::default(), Job::quiet()).expect("opened");
    let check = |trusted: Vec<String>| {
        archive
            .check_signature(signature.clone(), trusted, Job::quiet())
            .expect("checked")
    };
    // Not trusted: who it is by, and the key to trust.
    assert_eq!(
        check(vec![]),
        SignatureState::Stranger {
            fingerprint: mine.fingerprint.clone(),
            key: mine.line.clone(),
        }
    );
    assert_eq!(
        check(vec![mine.line.clone()]),
        SignatureState::Good {
            fingerprint: mine.fingerprint.clone()
        }
    );

    // Another archive, under the same signature.
    fs::write(&path, b"not what was signed").expect("write");
    let changed = Archive::open(path, Secrets::default(), Job::quiet());
    assert!(changed.is_err(), "no archive any more");
    assert!(matches!(
        read_key("not a key".to_owned()),
        Err(MobileError::Failed { .. })
    ));
}

#[test]
fn an_archive_is_changed_where_it_is_or_through_a_copy() {
    let dir = tempfile::tempdir().expect("dir");
    for (format, password) in [
        (ArchiveFormat::Zip, None),
        (ArchiveFormat::SevenZip, Some("sample-password")),
    ] {
        let path = make(dir.path(), format, password, false);
        let secrets = password.map(with).unwrap_or_default();
        let archive = Archive::open(path.clone(), secrets, Job::quiet()).expect("opened");
        assert!(archive.can_change());
        let folder = vec!["Báo cáo".to_owned()];
        let names = |a: &Archive| -> Vec<String> {
            a.children(folder.clone())
                .into_iter()
                .map(|e| e.name)
                .collect()
        };

        // Renamed where it is.
        archive
            .rename(
                path.clone(),
                vec!["Báo cáo".to_owned(), "ghi chú.txt".to_owned()],
                "notes.txt".to_owned(),
                Job::quiet(),
            )
            .expect("renamed");
        let again = archive.reopen(Job::quiet()).expect("read again");
        assert_eq!(names(&again), ["notes.txt", "tổng hợp.txt"]);

        // Added to a copy, which is then the archive: as a phone that may not
        // write beside the file does.
        let copy = dir.path().join("copy");
        fs::copy(&path, &copy).expect("copied");
        let new = dir.path().join("mới.txt");
        fs::write(&new, "thêm").expect("file");
        let added = |on: OnExisting| {
            again.add(
                copy.display().to_string(),
                vec![new.display().to_string()],
                folder.clone(),
                on,
                Job::quiet(),
            )
        };
        added(OnExisting::Stop).expect("added");
        // A name there already stops it, unless it is to be replaced.
        assert!(added(OnExisting::Stop).is_err());
        added(OnExisting::Replace).expect("replaced");
        fs::copy(&copy, &path).expect("written back");
        let again = again.reopen(Job::quiet()).expect("read again");
        assert_eq!(names(&again), ["mới.txt", "notes.txt", "tổng hợp.txt"]);
        // Encrypted as the rest are: its password reads it.
        let preview = again
            .preview(
                vec!["Báo cáo".to_owned(), "mới.txt".to_owned()],
                1024,
                Job::quiet(),
            )
            .expect("read");
        assert_eq!(preview, "thêm".as_bytes());

        again
            .delete(
                path.clone(),
                vec![vec!["Báo cáo".to_owned(), "notes.txt".to_owned()]],
                Job::quiet(),
            )
            .expect("deleted");
        let again = again.reopen(Job::quiet()).expect("read again");
        assert_eq!(names(&again), ["mới.txt", "tổng hợp.txt"]);
        // Not all of it.
        assert!(
            again
                .delete(path.clone(), vec![vec!["Báo cáo".to_owned()]], Job::quiet())
                .is_err()
        );
        fs::remove_file(&path).expect("removed");
        fs::remove_file(&copy).expect("removed");
    }
}

#[test]
fn a_phone_adds_storages_of_every_kind() {
    let kinds = storage_kinds();
    let ids: Vec<&str> = kinds.iter().map(|k| k.id.as_str()).collect();
    assert_eq!(ids, ["s3", "sftp", "webdav"]);
    // SFTP signs in with a key kept, chosen apart from its fields.
    let sftp = kinds.iter().find(|k| k.id == "sftp").expect("sftp");
    assert_eq!(sftp.key_value.as_deref(), Some("key"));
    let values = [(
        "address".to_owned(),
        "https://dav.example.com/files".to_owned(),
    )]
    .into_iter()
    .collect();
    assert!(describe_storage("webdav".to_owned(), values).contains("dav.example.com"));
}

/// Against a WebDAV server, skipped without `WEBDAV_TEST_ENDPOINT` (`make
/// webdav-start`, with `WEBDAV_TEST_USER` and `WEBDAV_TEST_PASSWORD`).
#[test]
fn an_archive_on_a_storage_is_listed_and_read_where_it_is() {
    let Ok(address) = std::env::var("WEBDAV_TEST_ENDPOINT") else {
        return;
    };
    let dir = tempfile::tempdir().expect("dir");
    let path = make(dir.path(), ArchiveFormat::Zip, None, false);
    let folder = format!("mobile-{}", std::process::id());
    let values: std::collections::HashMap<String, String> = [
        ("address".to_owned(), format!("{address}/{folder}")),
        (
            "user".to_owned(),
            std::env::var("WEBDAV_TEST_USER").unwrap_or_default(),
        ),
    ]
    .into_iter()
    .collect();
    let typed = [(
        "password".to_owned(),
        std::env::var("WEBDAV_TEST_PASSWORD").unwrap_or_default(),
    )]
    .into_iter()
    .collect();
    // Put there by the desktop's way, then read by the phone's.
    let top = packwing_remote::webdav(
        packwing_remote::WebDavSettings {
            address: address.clone(),
            user: std::env::var("WEBDAV_TEST_USER").unwrap_or_default(),
            password: std::env::var("WEBDAV_TEST_PASSWORD").unwrap_or_default(),
        },
        None,
    );
    use packwing_core::Storage as _;
    let _ = top.make_folder(&[], &folder);
    let like = packwing_core::Item {
        name: "a.zip".to_owned(),
        dir: false,
        link: false,
        hidden: false,
        size: fs::metadata(&path).expect("size").len(),
        modified: None,
        mode: None,
    };
    top.put(
        &[folder.clone(), "a.zip".to_owned()],
        &mut fs::File::open(&path).expect("file"),
        &like,
        true,
    )
    .expect("put");

    let storage = RemoteStorage::open("webdav".to_owned(), "test".to_owned(), values, Some(typed))
        .expect("opened");
    let items = storage.list(vec![], Job::quiet()).expect("listed");
    let zip = items.iter().find(|i| i.name == "a.zip").expect("there");
    let archive = storage
        .open_archive(
            vec!["a.zip".to_owned()],
            zip.size,
            Secrets::default(),
            Job::quiet(),
        )
        .expect("opened");
    assert!(!archive.can_change(), "changed on the phone alone");
    let names: Vec<String> = archive
        .children(vec!["Báo cáo".to_owned()])
        .into_iter()
        .map(|e| e.name)
        .collect();
    assert_eq!(names, ["ghi chú.txt", "tổng hợp.txt"]);
    let read = archive
        .preview(
            vec!["Báo cáo".to_owned(), "ghi chú.txt".to_owned()],
            100,
            Job::quiet(),
        )
        .expect("read");
    assert_eq!(read, b"hello");
    let _ = top.delete(std::slice::from_ref(&folder), &["a.zip".to_owned()]);
    let _ = top.remove(&[folder]);
}

/// Answers as a person would: trusts the server, signs in with `password`.
struct Person {
    password: String,
    asked: Mutex<Vec<String>>,
}

impl SshQuestions for Person {
    fn host_key(&self, host: String, port: u16, _: String, fingerprint: String) -> bool {
        self.asked
            .lock()
            .expect("asked")
            .push(format!("{host}:{port} {fingerprint}"));
        true
    }

    fn password(&self, _: String, _: bool) -> Option<TypedSecret> {
        Some(TypedSecret {
            secret: self.password.clone(),
            keep: false,
        })
    }

    fn passphrase(&self, _: String, _: bool) -> Option<TypedSecret> {
        None
    }

    fn code(&self, _: String, _: String, _: String, _: bool) -> Option<String> {
        None
    }
}

/// Against an SFTP server, skipped without `SFTP_TEST_HOST` (`tester@127.0.0.1:18322`
/// of `make sftp-start`, with `SFTP_TEST_PASSWORD`) and with a known_hosts of the
/// user's that knows it: a phone has none.
#[test]
fn sftp_asks_of_a_server_it_does_not_know_once() {
    let (Ok(host), Ok(password)) = (
        std::env::var("SFTP_TEST_HOST"),
        std::env::var("SFTP_TEST_PASSWORD"),
    ) else {
        return;
    };
    let dir = tempfile::tempdir().expect("dir");
    start(dir.path().display().to_string(), None);
    let person = Arc::new(Person {
        password,
        asked: Mutex::new(Vec::new()),
    });
    ask_ssh_with(Some(person.clone()));
    let values: std::collections::HashMap<String, String> =
        [("host".to_owned(), host)].into_iter().collect();
    let list = || {
        // A typed password of none: the server asks, and the person answers.
        RemoteStorage::open(
            "sftp".to_owned(),
            "test".to_owned(),
            values.clone(),
            Some(std::collections::HashMap::new()),
        )
        .expect("opened")
        .list(vec![], Job::quiet())
    };
    list().expect("listed");
    assert_eq!(person.asked.lock().expect("asked").len(), 1, "asked once");
    // Trusted now, in packwing's own known_hosts: not asked again.
    list().expect("listed again");
    assert_eq!(person.asked.lock().expect("asked").len(), 1);
    assert!(dir.path().join("ssh").join("known_hosts").exists());
    ask_ssh_with(None);
}
