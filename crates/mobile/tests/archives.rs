//! What the iOS app does with archives, through the bindings: made, opened
//! (with and without their passwords), searched, read and extracted.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use packwing_mobile::{
    Archive, ArchiveFormat, Asks, Checksum, ChecksumKind, Compared, CompressionLevel, ExtractHow,
    FileVerdict, Job, JobListener, JobProgress, MobileError, NewArchiveHow, OnExisting,
    PastedCompared, RemoteStorage, Secrets, SignatureState, SshQuestions, TypedSecret,
    age_key_is_post_quantum, age_key_problem, age_keys_in, archive_formats, ask_ssh_with,
    check_key_file, checksum_kinds, checksum_list, checksum_list_name, compare_checksum,
    compare_pasted, create_archive, default_stored_extensions, describe_storage, extensions_in,
    read_key, start, stem, storage_kinds, transfer_items,
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
        ArchiveFormat::SevenZip => "a.7z",
        _ => "a.zip",
    };
    create_archive(
        vec![files(dir)],
        dir.join(name).display().to_string(),
        NewArchiveHow {
            format,
            compression: CompressionLevel::Low,
            password: password.map(str::to_owned),
            hide_names,
            part_size: None,
            age_keys: vec![],
            ..NewArchiveHow::default()
        },
        Job::quiet(),
    )
    .expect("made")
}

fn with(password: &str) -> Secrets {
    Secrets {
        password: Some(password.to_owned()),
        ..Secrets::default()
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

#[test]
fn the_files_in_an_archive_have_their_checksums_in_one_read_of_it() {
    let dir = tempfile::tempdir().expect("dir");
    for format in [ArchiveFormat::Zip, ArchiveFormat::SevenZip] {
        let path = make(dir.path(), format, None, false);
        let archive =
            Archive::open(path.clone(), Secrets::default(), Job::quiet()).expect("opened");
        let sha256 = |bytes: &[u8]| {
            let control = packwing_core::Control::default();
            packwing_core::checksums_of(bytes, &[packwing_core::Algorithm::Sha256], &control)
                .expect("made")[0]
                .1
                .clone()
        };

        let all = archive
            .entry_checksums(vec![], vec![ChecksumKind::Sha256], Job::quiet())
            .expect("made");
        let names: Vec<&str> = all.files.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, ["Báo cáo/ghi chú.txt", "Báo cáo/tổng hợp.txt"]);
        assert_eq!(all.files[0].sums[0].sum, sha256(b"hello"));
        assert_eq!(all.files[0].size, 5);
        assert!(all.damaged.is_empty());

        // One picked: named as it is extracted, alone.
        let one = archive
            .entry_checksums(
                vec![vec!["Báo cáo".to_owned(), "ghi chú.txt".to_owned()]],
                vec![ChecksumKind::Sha256, ChecksumKind::Md5],
                Job::quiet(),
            )
            .expect("made");
        assert_eq!(one.files.len(), 1);
        assert_eq!(one.files[0].name, "ghi chú.txt");
        assert_eq!(one.files[0].sums.len(), 2);
        assert_eq!(
            checksum_list(one.files.clone(), ChecksumKind::Sha256),
            format!("{}  ghi chú.txt\n", sha256(b"hello"))
        );
        assert_eq!(
            checksum_list_name("a.zip".to_owned(), ChecksumKind::Sha256),
            "a.zip.contents.sha256"
        );

        // Compared with a list pasted: as made, then one line changed.
        let list = checksum_list(all.files.clone(), ChecksumKind::Sha256);
        let compare = |pasted: &str| {
            compare_pasted(
                pasted.to_owned(),
                all.files.clone(),
                vec![ChecksumKind::Sha256],
            )
        };
        let Some(PastedCompared::Listed {
            verdicts,
            not_found,
        }) = compare(&list)
        else {
            panic!("a list")
        };
        assert!(
            verdicts
                .iter()
                .all(|v| matches!(v, FileVerdict::Matches { .. }))
        );
        assert!(not_found.is_empty());
        let changed = list.replacen(&all.files[0].sums[0].sum, &"0".repeat(64), 1)
            + &format!("{}  gone.txt\n", "1".repeat(64));
        let Some(PastedCompared::Listed {
            verdicts,
            not_found,
        }) = compare(&changed)
        else {
            panic!("a list")
        };
        assert!(matches!(verdicts[0], FileVerdict::Differs { .. }));
        assert!(matches!(verdicts[1], FileVerdict::Matches { .. }));
        assert_eq!(not_found, ["gone.txt"]);
        // One checksum alone: the file it is of.
        let Some(PastedCompared::One { found, .. }) = compare(&all.files[1].sums[0].sum) else {
            panic!("one")
        };
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].index, 1);
        assert_eq!(compare("hello"), Some(PastedCompared::NotOne));
        assert_eq!(compare("  "), None);
        fs::remove_file(&path).expect("removed");
    }
}

/// Against a WebDAV server, skipped without `WEBDAV_TEST_ENDPOINT`: a storage's
/// files made, renamed, copied both ways, made an archive of, and deleted.
#[test]
fn a_storage_is_written_to_from_the_phone() {
    let Ok(address) = std::env::var("WEBDAV_TEST_ENDPOINT") else {
        return;
    };
    let dir = tempfile::tempdir().expect("dir");
    let folder = format!("writes-{}", std::process::id());
    let values: std::collections::HashMap<String, String> = [
        ("address".to_owned(), address),
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
    let there = RemoteStorage::open("webdav".to_owned(), "w".to_owned(), values, Some(typed))
        .expect("opened");
    let names = |at: Vec<String>| -> Vec<String> {
        there
            .list(at, Job::quiet())
            .expect("listed")
            .into_iter()
            .map(|i| i.name)
            .collect()
    };
    there.make_folder(vec![], folder.clone()).expect("made");
    let at = vec![folder.clone()];

    // Up from the phone.
    let here = RemoteStorage::local(files(dir.path()));
    let sent = transfer_items(
        here.clone(),
        vec![],
        vec!["ghi chú.txt".to_owned()],
        there.clone(),
        at.clone(),
        false,
        OnExisting::Stop,
        Job::quiet(),
    )
    .expect("sent");
    assert_eq!(sent.files, 1);
    there
        .rename(at.clone(), "ghi chú.txt".to_owned(), "notes.txt".to_owned())
        .expect("renamed");

    // An archive made there.
    let made = there
        .create_archive(
            vec![dir.path().join("Báo cáo").display().to_string()],
            at.clone(),
            "Báo cáo.zip".to_owned(),
            NewArchiveHow {
                format: ArchiveFormat::Zip,
                compression: CompressionLevel::Low,
                password: None,
                hide_names: false,
                part_size: None,
                age_keys: vec![],
                ..NewArchiveHow::default()
            },
            Job::quiet(),
        )
        .expect("made");
    assert_eq!(made, "Báo cáo.zip");
    assert_eq!(names(at.clone()), ["Báo cáo.zip", "notes.txt"]);

    // Down to the phone.
    let down = dir.path().join("down");
    fs::create_dir_all(&down).expect("down");
    transfer_items(
        there.clone(),
        at.clone(),
        vec!["notes.txt".to_owned()],
        RemoteStorage::local(down.display().to_string()),
        vec![],
        true,
        OnExisting::Stop,
        Job::quiet(),
    )
    .expect("moved");
    assert_eq!(fs::read(down.join("notes.txt")).expect("read"), b"hello");
    assert_eq!(names(at.clone()), ["Báo cáo.zip"]);

    there.delete(vec![], vec![folder.clone()]).expect("deleted");
    assert!(!names(vec![]).contains(&folder));
}

#[test]
fn every_format_the_desktop_makes_is_made_and_opened_again() {
    let dir = tempfile::tempdir().expect("dir");
    let folder = files(dir.path());
    let one = format!("{folder}/ghi chú.txt");
    for row in archive_formats(true) {
        // One file by itself, or the folder.
        let what = if row.one_file {
            one.clone()
        } else {
            folder.clone()
        };
        for password in [None, Some("sample-password")] {
            if password.is_some() && row.format == ArchiveFormat::Tar {
                // age around a .tar: as any other; once is enough.
                continue;
            }
            let wanted = dir.path().join(format!("made{}", row.ext));
            let made = create_archive(
                vec![what.clone()],
                wanted.display().to_string(),
                NewArchiveHow {
                    format: row.format,
                    compression: CompressionLevel::Store,
                    password: password.map(str::to_owned),
                    hide_names: false,
                    part_size: None,
                    age_keys: vec![],
                    ..NewArchiveHow::default()
                },
                Job::quiet(),
            )
            .unwrap_or_else(|e| panic!("{} made: {e}", row.ext));
            assert!(made.ends_with(&row.ext), "{made}");
            let secrets = match (password, row.uses_age) {
                (Some(p), true) => Secrets {
                    passphrase: Some(p.to_owned()),
                    ..Secrets::default()
                },
                (Some(p), false) => with(p),
                (None, _) => Secrets::default(),
            };
            let archive = Archive::open(made.clone(), secrets, Job::quiet())
                .unwrap_or_else(|e| panic!("{} opened: {e}", row.ext));
            assert!(archive.info().files >= 1, "{}", row.ext);
            fs::remove_file(&made).expect("removed");
        }
    }
    // Without one file, those for one file are not offered.
    assert_eq!(archive_formats(false).len(), 8);
}

/// The core's test data: age files, and the key files they were encrypted to.
fn core_data(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../core/tests/data")
        .join(name)
}

#[test]
fn an_age_file_opens_with_its_key_file_and_that_files_passphrase() {
    let file = core_data("x25519.tar.zst.age").display().to_string();
    // Nothing given: its key file asked for.
    match Archive::open(file.clone(), Secrets::default(), Job::quiet()) {
        Err(MobileError::Locked {
            asks, passphrase, ..
        }) => {
            assert_eq!(asks, Asks::KeyFile);
            assert!(!passphrase);
        }
        other => panic!("{:?}", other.map(|a| a.info())),
    }
    // Not a key file.
    assert!(check_key_file(b"hello".to_vec()).is_err());
    let key = fs::read(core_data("test-age.key")).expect("key");
    check_key_file(key.clone()).expect("a key file");
    let archive = Archive::open(
        file.clone(),
        Secrets {
            key_file: Some(key),
            ..Secrets::default()
        },
        Job::quiet(),
    )
    .expect("opened with its key");
    assert!(archive.info().files >= 1);

    // A key file protected itself: its passphrase asked for, then a wrong one said.
    let file = core_data("ssh-locked.tar.zst.age").display().to_string();
    let key = fs::read(core_data("test-ssh-locked.key")).expect("key");
    check_key_file(key.clone()).expect("a key file, protected");
    let given = |passphrase: Option<&str>| Secrets {
        key_file: Some(key.clone()),
        key_passphrase: passphrase.map(str::to_owned),
        ..Secrets::default()
    };
    match Archive::open(file.clone(), given(None), Job::quiet()) {
        Err(MobileError::Locked { asks, .. }) => assert_eq!(asks, Asks::KeyPassphrase),
        other => panic!("{:?}", other.map(|a| a.info())),
    }
    assert!(matches!(
        Archive::open(file.clone(), given(Some("sai")), Job::quiet()),
        Err(MobileError::WrongPassword { .. })
    ));
    Archive::open(file, given(Some("khoá ssh")), Job::quiet()).expect("opened");
}

#[test]
fn a_new_age_file_is_encrypted_to_public_keys() {
    let dir = tempfile::tempdir().expect("dir");
    let folder = files(dir.path());
    let key = fs::read(core_data("test-age.key")).expect("key");
    let public = String::from_utf8_lossy(&key)
        .lines()
        .find_map(|l| l.strip_prefix("# public key: "))
        .expect("public")
        .to_owned();
    assert_eq!(age_key_problem(public.clone()), None);
    assert!(age_key_problem("age1nope".to_owned()).is_some());
    assert!(!age_key_is_post_quantum(public.clone()));

    // A file of keys, as `age -R` reads it.
    let read = age_keys_in(format!("# mine\n{public}\n").into_bytes()).expect("keys");
    assert_eq!(read.keys, [public.as_str()]);
    assert!(age_keys_in(b"# none\n".to_vec()).is_err());

    let wanted = dir.path().join("made.tar.zst.age");
    let made = create_archive(
        vec![folder],
        wanted.display().to_string(),
        NewArchiveHow {
            format: ArchiveFormat::TarZst,
            compression: CompressionLevel::Low,
            age_keys: read.keys,
            ..NewArchiveHow::default()
        },
        Job::quiet(),
    )
    .expect("made");
    assert!(matches!(
        Archive::open(made.clone(), Secrets::default(), Job::quiet()),
        Err(MobileError::Locked {
            asks: Asks::KeyFile,
            ..
        })
    ));
    let archive = Archive::open(
        made,
        Secrets {
            key_file: Some(key),
            ..Secrets::default()
        },
        Job::quiet(),
    )
    .expect("opened with the key");
    assert!(archive.info().files >= 1);
}

#[test]
fn what_only_a_mac_uses_is_left_out_unless_asked_and_a_limit_stops_a_bomb() {
    let dir = tempfile::tempdir().expect("dir");
    let folder = files(dir.path());
    fs::write(format!("{folder}/.DS_Store"), "mac").expect("file");
    let names = |skip_mac_files: bool| {
        let wanted = dir.path().join(format!("mac-{skip_mac_files}.zip"));
        let made = create_archive(
            vec![folder.clone()],
            wanted.display().to_string(),
            NewArchiveHow {
                skip_mac_files,
                ..NewArchiveHow::default()
            },
            Job::quiet(),
        )
        .expect("made");
        let archive =
            Archive::open(made.clone(), Secrets::default(), Job::quiet()).expect("opened");
        let names: Vec<String> = archive
            .children(vec!["Báo cáo".to_owned()])
            .into_iter()
            .map(|e| e.name)
            .collect();
        (made, names)
    };
    let (_, left_out) = names(true);
    assert!(!left_out.contains(&".DS_Store".to_owned()), "{left_out:?}");
    let (made, kept) = names(false);
    assert!(kept.contains(&".DS_Store".to_owned()), "{kept:?}");

    // 3 kB of files, against a limit of 1 kB: stopped; with none, extracted.
    let archive = Archive::open(made, Secrets::default(), Job::quiet()).expect("opened");
    let out = dir.path().join("out");
    let stopped = archive.extract(
        out.display().to_string(),
        vec![],
        ExtractHow {
            limit: 1_000,
            ..ExtractHow::default()
        },
        Job::quiet(),
    );
    assert!(stopped.is_err());
    let out = dir.path().join("out-all");
    archive
        .extract(
            out.display().to_string(),
            vec![],
            ExtractHow {
                limit: 0,
                ..ExtractHow::default()
            },
            Job::quiet(),
        )
        .expect("no limit");
}

#[test]
fn a_zip_archive_is_encrypted_whole_with_age_to_a_passphrase_or_keys() {
    let dir = tempfile::tempdir().expect("dir");
    let folder = files(dir.path());
    let key = fs::read(core_data("test-age.key")).expect("key");
    let public = String::from_utf8_lossy(&key)
        .lines()
        .find_map(|l| l.strip_prefix("# public key: "))
        .expect("public")
        .to_owned();
    for (password, keys) in [(Some("sample-password"), vec![]), (None, vec![public])] {
        let wanted = dir
            .path()
            .join(format!("whole-{}.zip.age", password.is_some()));
        let made = create_archive(
            vec![folder.clone()],
            wanted.display().to_string(),
            NewArchiveHow {
                format: ArchiveFormat::Zip,
                password: password.map(str::to_owned),
                age_keys: keys.clone(),
                whole_with_age: true,
                ..NewArchiveHow::default()
            },
            Job::quiet(),
        )
        .expect("made");
        assert!(made.ends_with(".zip.age"), "{made}");
        // age's, not ZIP's: its header says so.
        let head = fs::read(&made).expect("read");
        assert!(head.starts_with(b"age-encryption.org/v1"), "{made}");
        let asked = if password.is_some() {
            Asks::Passphrase
        } else {
            Asks::KeyFile
        };
        match Archive::open(made.clone(), Secrets::default(), Job::quiet()) {
            Err(MobileError::Locked { asks, .. }) => assert_eq!(asks, asked),
            other => panic!("{:?}", other.map(|a| a.info())),
        }
        let secrets = match password {
            Some(p) => Secrets {
                passphrase: Some(p.to_owned()),
                ..Secrets::default()
            },
            None => Secrets {
                key_file: Some(key.clone()),
                ..Secrets::default()
            },
        };
        let archive = Archive::open(made, secrets, Job::quiet()).expect("opened");
        assert!(archive.info().files >= 2);
    }
}

#[test]
fn extensions_are_read_as_the_desktop_reads_them() {
    assert_eq!(
        extensions_in("*.JPG .png, mp4;jpg  heic".to_owned()),
        ["jpg", "png", "mp4", "heic"]
    );
    let defaults = default_stored_extensions();
    assert!(defaults.contains(&"jpg".to_owned()) && defaults.contains(&"zip".to_owned()));
}
