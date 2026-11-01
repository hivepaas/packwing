//! Against servers, each skipped without its address:
//!
//! - `S3_TEST_ENDPOINT=http://127.0.0.1:18333`, with a bucket `test`, and
//!   `S3_TEST_ACCESS_KEY` and `S3_TEST_SECRET_KEY`, or none if it may be written to
//!   with no signature, as a local SeaweedFS allows;
//! - `WEBDAV_TEST_ENDPOINT=http://127.0.0.1:18335`, and `WEBDAV_TEST_USER` and
//!   `WEBDAV_TEST_PASSWORD`, or none if it asks for no password;
//! - `SFTP_TEST_HOST=tester@127.0.0.1:2222`, with `SFTP_TEST_KEY` the key file and
//!   `SFTP_TEST_KNOWN_HOSTS` a known_hosts that trusts it, or `SFTP_TEST_PASSWORD`
//!   in place of the key.
//!
//! `make s3-start`, `make webdav-start` and `make sftp-start` run them in Docker.
//!
//! `cargo test -p packwing-remote -- --test-threads=4`

use std::io::Read;
use std::sync::Arc;

use packwing_core::{
    Control, Item, Keys, LocalFolder, NewArchive, OnExisting, Source, Storage, Stored, Transfer,
    create, create_on, extract_selected, list, read_entry, transfer,
};
use packwing_remote::{S3Settings, SftpSettings, WebDavSettings, s3, sftp, webdav};

/// A folder for one test on each server there is.
fn storages(test: &str) -> Vec<(&'static str, Box<dyn Storage>)> {
    let mut there: Vec<(&'static str, Box<dyn Storage>)> = Vec::new();
    if let Ok(endpoint) = std::env::var("S3_TEST_ENDPOINT") {
        let settings = S3Settings {
            endpoint,
            bucket: "test".into(),
            prefix: format!("{}/{test}", std::process::id()),
            access_key: std::env::var("S3_TEST_ACCESS_KEY").unwrap_or_default(),
            secret_key: std::env::var("S3_TEST_SECRET_KEY").unwrap_or_default(),
            ..S3Settings::default()
        };
        there.push(("s3", Box::new(s3(settings, None))));
    }
    if let Ok(address) = std::env::var("WEBDAV_TEST_ENDPOINT") {
        let user = std::env::var("WEBDAV_TEST_USER").unwrap_or_default();
        let password = std::env::var("WEBDAV_TEST_PASSWORD").unwrap_or_default();
        let top = webdav(
            WebDavSettings {
                address: address.clone(),
                user: user.clone(),
                password: password.clone(),
            },
            None,
        );
        let folder = format!("{}-{test}", std::process::id());
        // Its own folder, as the storage's top.
        let _ = top.make_folder(&[], &folder);
        let settings = WebDavSettings {
            address: format!("{}/{folder}", address.trim_end_matches('/')),
            user,
            password,
        };
        there.push(("webdav", Box::new(webdav(settings, None))));
    }
    if let Ok(host) = std::env::var("SFTP_TEST_HOST") {
        let settings = |start: String| SftpSettings {
            host: host.clone(),
            start,
            password: std::env::var("SFTP_TEST_PASSWORD").unwrap_or_default(),
            key_file: std::env::var_os("SFTP_TEST_KEY").map(Into::into),
            known_hosts: std::env::var_os("SFTP_TEST_KNOWN_HOSTS").map(Into::into),
        };
        let folder = format!("packwing-{}-{test}", std::process::id());
        sftp(settings(String::new()), None)
            .make_folder(&[], &folder)
            .expect("a folder for the test");
        there.push(("sftp", Box::new(sftp(settings(folder), None))));
    }
    there
}

fn like(name: &str) -> Item {
    Item {
        name: name.into(),
        dir: false,
        link: false,
        hidden: false,
        size: 0,
        modified: None,
        mode: None,
    }
}

fn put(s: &dyn Storage, path: &[&str], body: &[u8]) {
    let path: Vec<String> = path.iter().map(|p| (*p).to_owned()).collect();
    // Its folders first, as a transfer makes them: one is there, or that fails.
    for depth in 0..path.len() - 1 {
        let _ = s.make_folder(&path[..depth], &path[depth]);
    }
    s.put(&path, &mut &body[..], &like("x"), false)
        .expect("put");
}

fn all(s: &dyn Storage, folder: &[String]) -> Vec<Item> {
    let mut lister = s.list(folder).expect("list");
    let mut items = Vec::new();
    while !lister.done() {
        items.extend(lister.next(2).expect("page"));
    }
    items
}

fn none() -> Control {
    Control {
        progress: None,
        cancel: None,
    }
}

#[test]
fn folders_are_prefixes_and_pages_end() {
    for (server, s) in storages("list") {
        let _ = server;
        put(s.as_ref(), &["b.txt"], b"bb");
        put(s.as_ref(), &["a.txt"], b"a");
        put(s.as_ref(), &["sub", "deep", "c.txt"], b"ccc");
        s.make_folder(&[], "empty").expect("make");
        assert!(s.make_folder(&[], "empty").is_err());
        let items = all(s.as_ref(), &[]);
        let names: Vec<(&str, bool)> = items.iter().map(|i| (i.name.as_str(), i.dir)).collect();
        assert!(names.contains(&("sub", true)), "{names:?}");
        assert!(names.contains(&("empty", true)), "{names:?}");
        assert!(names.contains(&("a.txt", false)), "{names:?}");
        assert_eq!(items.len(), 4);
        let a = items.iter().find(|i| i.name == "b.txt").expect("b");
        assert_eq!(a.size, 2);
        assert!(a.modified.is_some());
        assert!(all(s.as_ref(), &["empty".into()]).is_empty());
        let deep = all(s.as_ref(), &["sub".into()]);
        assert_eq!(deep.len(), 1);
        assert!(deep[0].dir);
    }
}

#[test]
fn a_thousand_and_more_come_in_pages() {
    for (server, s) in storages("many") {
        let _ = server;
        for i in 0..1205 {
            put(s.as_ref(), &[&format!("f{i:05}.txt")], b"x");
        }
        let mut lister = s.list(&[]).expect("list");
        assert_eq!(lister.next(1000).expect("page").len(), 1000);
        assert!(!lister.done());
        assert_eq!(lister.next(1000).expect("page").len(), 205);
        assert!(lister.done());
    }
}

#[test]
fn stat_read_and_open_agree() {
    for (server, s) in storages("read") {
        let _ = server;
        let big: Vec<u8> = (0..(9u32 << 20)).map(|i| (i % 251) as u8).collect();
        put(s.as_ref(), &["big.bin"], &big);
        put(s.as_ref(), &["d", "x"], b"x");
        let item = s.stat(&[], "big.bin").expect("stat").expect("there");
        assert_eq!(item.size, big.len() as u64);
        assert!(!item.dir);
        assert!(s.stat(&[], "d").expect("stat").expect("there").dir);
        assert!(s.stat(&[], "nope").expect("stat").is_none());
        assert_eq!(s.read(&["big.bin".into()], 5).expect("read"), &big[..5]);
        let mut back = Vec::new();
        s.open(&["big.bin".into()])
            .expect("open")
            .read_to_end(&mut back)
            .expect("read");
        assert_eq!(back, big);
    }
}

#[test]
fn renaming_and_deleting() {
    for (server, s) in storages("rename") {
        let _ = server;
        put(s.as_ref(), &["a"], b"a");
        put(s.as_ref(), &["b"], b"b");
        assert!(s.rename(&[], "a", "b").is_err());
        assert_eq!(s.read(&["b".into()], 9).expect("read"), b"b");
        s.rename(&[], "a", "c").expect("rename");
        assert!(s.stat(&[], "a").expect("stat").is_none());
        assert_eq!(s.read(&["c".into()], 9).expect("read"), b"a");
        put(s.as_ref(), &["d", "e", "f"], b"f");
        put(s.as_ref(), &["d", "g"], b"g");
        let renamed = s.rename(&[], "d", "z");
        // A server renames a folder itself; the others do not.
        if server == "sftp" {
            renamed.expect("a folder renamed");
            s.rename(&[], "z", "d").expect("and back");
        } else {
            assert!(renamed.is_err());
        }
        s.delete(&[], &["d".into(), "b".into()]).expect("delete");
        // Every object under it goes. SeaweedFS goes on listing an emptied folder as a
        // prefix, as AWS would not, so the objects are what is asked after.
        assert!(s.stat(&["d".into()], "g").expect("stat").is_none());
        assert!(
            s.stat(&["d".into(), "e".into()], "f")
                .expect("stat")
                .is_none()
        );
        assert!(s.stat(&[], "b").expect("stat").is_none());
        assert!(s.stat(&[], "c").expect("stat").is_some());
    }
}

#[test]
fn a_taken_name_is_not_written_over_unless_asked() {
    for (server, s) in storages("taken") {
        let _ = server;
        put(s.as_ref(), &["a"], b"old");
        let path = ["a".to_owned()];
        assert!(s.put(&path, &mut &b"new"[..], &like("a"), false).is_err());
        assert_eq!(s.read(&path, 9).expect("read"), b"old");
        s.put(&path, &mut &b"new"[..], &like("a"), true)
            .expect("replace");
        assert_eq!(s.read(&path, 9).expect("read"), b"new");
    }
}

#[test]
fn what_cannot_be_sent_in_parts_says_how_much_it_takes() {
    for (server, s) in storages("big") {
        let path = ["big".to_owned()];
        let mut item = like("big");
        item.size = 2_000_000_000;
        let refused = s.put(&path, &mut &b"x"[..], &item, false);
        assert_eq!(
            refused.is_err(),
            server == "webdav",
            "{server}: {refused:?}"
        );
    }
}

#[test]
fn a_failed_upload_leaves_no_object() {
    struct Fails(usize);
    impl Read for Fails {
        fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
            if self.0 == 0 {
                return Err(std::io::Error::other("stopped"));
            }
            self.0 -= 1;
            out.fill(7);
            Ok(out.len())
        }
    }
    for (server, s) in storages("fail") {
        let _ = server;
        let path = ["a".to_owned()];
        assert!(s.put(&path, &mut Fails(40), &like("a"), false).is_err());
        assert!(s.stat(&[], "a").expect("stat").is_none());
    }
}

#[test]
fn moving_between_a_folder_and_a_bucket() {
    for (server, s) in storages("transfer") {
        let _ = server;
        let dir = std::env::temp_dir().join(format!("packwing-pro-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("up/d")).expect("mkdir");
        std::fs::create_dir_all(dir.join("down")).expect("mkdir");
        std::fs::write(dir.join("up/d/one"), "11111").expect("write");
        std::fs::write(dir.join("up/two"), vec![9u8; 20 << 20]).expect("write");
        let bucket: Arc<dyn Storage> = Arc::from(s);
        let t = |from: Arc<dyn Storage>, to: Arc<dyn Storage>, names: &[&str], moving| Transfer {
            from,
            from_folder: Vec::new(),
            names: names.iter().map(|n| (*n).to_owned()).collect(),
            to,
            to_folder: Vec::new(),
            moving,
            on_existing: OnExisting::Stop,
        };
        let up: Arc<dyn Storage> = Arc::new(LocalFolder::new(dir.join("up")));
        let down: Arc<dyn Storage> = Arc::new(LocalFolder::new(dir.join("down")));
        let went = transfer(&t(up, bucket.clone(), &["d", "two"], true), &none()).expect("up");
        assert_eq!(went.files, 2);
        assert!(!dir.join("up/two").exists());
        assert!(!dir.join("up/d").exists());
        assert!(bucket.stat(&["d".into()], "one").expect("stat").is_some());
        transfer(&t(bucket.clone(), down, &["d", "two"], false), &none()).expect("down");
        assert_eq!(
            std::fs::read(dir.join("down/d/one")).expect("read"),
            b"11111"
        );
        assert_eq!(
            std::fs::metadata(dir.join("down/two")).expect("meta").len(),
            20 << 20
        );
        // A move within the bucket, and the original goes.
        let mut within = t(bucket.clone(), bucket.clone(), &["two"], true);
        within.to_folder = vec!["d".into()];
        transfer(&within, &none()).expect("within");
        assert!(bucket.stat(&[], "two").expect("stat").is_none());
        assert!(bucket.stat(&["d".into()], "two").expect("stat").is_some());
        let _ = std::fs::remove_dir_all(&dir);
    }
}

/// Writes to this user's keychain, and takes it out again, so it is run by hand:
/// `cargo test -p packwing-remote secrets_are_kept -- --ignored`.
#[test]
#[ignore]
fn secrets_are_kept_and_forgotten() {
    use packwing_core::{StorageKind, Values};
    let kind = packwing_remote::S3Kind;
    let id = format!("test-{}", std::process::id());
    let secrets: Values = [("secret_key".to_owned(), "hush".to_owned())].into();
    kind.keep_secrets(&id, &secrets).expect("kept");
    assert_eq!(
        packwing_remote::secret(&id, "secret_key").expect("read"),
        "hush"
    );
    kind.forget_secrets(&id);
    assert_eq!(
        packwing_remote::secret(&id, "secret_key").expect("read"),
        ""
    );
}

#[test]
fn an_unknown_host_is_refused_and_says_so() {
    let (Ok(host), Some(key)) = (
        std::env::var("SFTP_TEST_HOST"),
        std::env::var_os("SFTP_TEST_KEY"),
    ) else {
        return;
    };
    let hosts = std::env::temp_dir().join(format!("packwing-no-hosts-{}", std::process::id()));
    std::fs::write(&hosts, "").expect("write");
    let s = sftp(
        SftpSettings {
            host,
            start: String::new(),
            password: String::new(),
            key_file: Some(key.into()),
            known_hosts: Some(hosts.clone()),
        },
        None,
    );
    let e = s.list(&[]).map(|_| ()).expect_err("not known");
    let _ = std::fs::remove_file(hosts);
    println!("said: {e}");
    assert!(!e.to_string().is_empty());
}

#[test]
fn a_link_is_told_from_what_it_points_at() {
    let (Ok(host), Ok(key), Ok(hosts)) = (
        std::env::var("SFTP_TEST_HOST"),
        std::env::var("SFTP_TEST_KEY"),
        std::env::var("SFTP_TEST_KNOWN_HOSTS"),
    ) else {
        return;
    };
    let s = sftp(
        SftpSettings {
            host: host.clone(),
            start: String::new(),
            password: String::new(),
            key_file: Some(key.clone().into()),
            known_hosts: Some(hosts.clone().into()),
        },
        None,
    );
    let (user_host, port) = host.rsplit_once(':').expect("a port");
    let folder = format!("packwing-{}-links", std::process::id());
    s.make_folder(&[], &folder).expect("a folder");
    s.make_folder(std::slice::from_ref(&folder), "real")
        .expect("a folder");
    let made = std::process::Command::new("ssh")
        .args([
            "-F",
            "none",
            "-i",
            &key,
            "-p",
            port,
            "-o",
            "IdentitiesOnly=yes",
        ])
        .args(["-o", "StrictHostKeyChecking=yes"])
        .args(["-o", &format!("UserKnownHostsFile={hosts}")])
        .args([user_host, &format!("ln -s real {folder}/to-real")])
        .stdin(std::process::Stdio::null())
        .status()
        .expect("ssh");
    assert!(made.success());
    let f = std::slice::from_ref(&folder);
    let link = s.stat(f, "to-real").expect("stat").expect("there");
    assert!(link.link && link.dir);
    let listed = all(&s, f);
    let link = listed.iter().find(|i| i.name == "to-real").expect("listed");
    assert!(link.link && link.dir);
    let real = listed.iter().find(|i| i.name == "real").expect("listed");
    assert!(!real.link && real.dir);
    // Deleting the link leaves what it points at.
    s.delete(f, &["to-real".to_owned()]).expect("delete");
    assert!(s.stat(f, "real").expect("stat").is_some());
}

#[test]
fn an_archive_on_a_server_is_read_in_place() {
    for (server, s) in storages("archive") {
        let dir =
            std::env::temp_dir().join(format!("packwing-remote-{}-{server}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("in")).expect("mkdir");
        std::fs::write(dir.join("in/small.txt"), "small").expect("write");
        // Does not compress, so most of the archive is what is not asked for.
        let mut noise = vec![0u8; 5 << 20];
        let mut x = 88172645463325252u64;
        for b in noise.iter_mut() {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            *b = x as u8;
        }
        std::fs::write(dir.join("in/big.bin"), &noise).expect("write");
        let made = create(
            &[dir.join("in")],
            &dir.join("a.zip"),
            &NewArchive::default(),
            &none(),
        )
        .expect("an archive");
        let body = std::fs::read(&made).expect("read");
        put(s.as_ref(), &["a.zip"], &body);

        let storage: Arc<dyn Storage> = Arc::from(s);
        let stored = Stored::new(storage, vec!["a.zip".into()], body.len() as u64);
        let source = Source::Stored(stored.clone());
        let keys = Keys::default();
        let listing = list(&source, &keys).expect("list");
        assert_eq!(listing.files, 2, "{server}");
        let small = ["in", "small.txt"].map(String::from).to_vec();
        let got = read_entry(&source, &small, &keys, 100, &none()).expect("a file");
        assert_eq!(got, b"small", "{server}");
        assert!(
            stored.fetched() < body.len() as u64 / 4,
            "{server}: {} of {}",
            stored.fetched(),
            body.len()
        );
        let to = dir.join("out");
        let big = ["in", "big.bin"].map(String::from).to_vec();
        extract_selected(&source, &to, &[big], &keys, &Default::default(), &none())
            .expect("the large file");
        assert_eq!(std::fs::read(to.join("big.bin")).expect("read"), noise);
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[test]
fn a_new_archive_goes_up_and_reads_back() {
    let here = std::env::temp_dir().join(format!("packwing-up-{}", std::process::id()));
    std::fs::create_dir_all(&here).expect("a folder");
    let what = here.join("what.txt");
    std::fs::write(&what, "hello ".repeat(100_000)).expect("a file");
    for (kind, s) in storages("up") {
        let keys = Keys::default();
        let made = create_on(
            std::slice::from_ref(&what),
            s.as_ref(),
            &[],
            "new.zip",
            &NewArchive::default(),
            &none(),
            &|| {},
        )
        .expect(kind);
        assert_eq!(made, None, "{kind}");
        // A second takes the next name, and reads back from where it is.
        create_on(
            std::slice::from_ref(&what),
            s.as_ref(),
            &[],
            "new.zip",
            &NewArchive::default(),
            &none(),
            &|| {},
        )
        .expect(kind);
        let names: Vec<String> = all(s.as_ref(), &[]).into_iter().map(|i| i.name).collect();
        assert!(
            names.contains(&"new (2).zip".to_owned()),
            "{kind}: {names:?}"
        );
        let size = s.stat(&[], "new (2).zip").expect(kind).expect(kind).size;
        let storage: Arc<dyn Storage> = Arc::from(s);
        let source = Source::Stored(Stored::new(storage, vec!["new (2).zip".into()], size));
        let body =
            read_entry(&source, &["what.txt".to_owned()], &keys, 1 << 20, &none()).expect(kind);
        assert_eq!(body.len(), 600_000, "{kind}");
    }
    let _ = std::fs::remove_dir_all(&here);
}
