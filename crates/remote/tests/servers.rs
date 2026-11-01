//! Against servers, each skipped without its address:
//!
//! - `S3_TEST_ENDPOINT=http://127.0.0.1:18333`, with a bucket `test` that may be
//!   written to with no signature, as a local SeaweedFS allows;
//! - `WEBDAV_TEST_ENDPOINT=http://127.0.0.1:18335`, with no password.
//!
//! `cargo test -p packwing-remote -- --test-threads=4`

use std::io::Read;
use std::sync::Arc;

use packwing_core::{Control, Item, LocalFolder, OnExisting, Storage, Transfer, transfer};
use packwing_remote::{Dal, S3Settings, WebDavSettings, s3, webdav};

/// A folder for one test on each server there is.
fn storages(test: &str) -> Vec<(&'static str, Dal)> {
    let mut there = Vec::new();
    if let Ok(endpoint) = std::env::var("S3_TEST_ENDPOINT") {
        let settings = S3Settings {
            endpoint,
            bucket: "test".into(),
            prefix: format!("{}/{test}", std::process::id()),
            ..S3Settings::default()
        };
        there.push(("s3", s3(settings, None)));
    }
    if let Ok(address) = std::env::var("WEBDAV_TEST_ENDPOINT") {
        let top = webdav(
            WebDavSettings {
                address: address.clone(),
                ..WebDavSettings::default()
            },
            None,
        );
        let folder = format!("{}-{test}", std::process::id());
        // Its own folder, as the storage's top.
        let _ = top.make_folder(&[], &folder);
        let settings = WebDavSettings {
            address: format!("{}/{folder}", address.trim_end_matches('/')),
            ..WebDavSettings::default()
        };
        there.push(("webdav", webdav(settings, None)));
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

fn put(s: &Dal, path: &[&str], body: &[u8]) {
    let path: Vec<String> = path.iter().map(|p| (*p).to_owned()).collect();
    s.put(&path, &mut &body[..], &like("x"), false)
        .expect("put");
}

fn all(s: &Dal, folder: &[String]) -> Vec<Item> {
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
        put(&s, &["b.txt"], b"bb");
        put(&s, &["a.txt"], b"a");
        put(&s, &["sub", "deep", "c.txt"], b"ccc");
        s.make_folder(&[], "empty").expect("make");
        assert!(s.make_folder(&[], "empty").is_err());
        let items = all(&s, &[]);
        let names: Vec<(&str, bool)> = items.iter().map(|i| (i.name.as_str(), i.dir)).collect();
        assert!(names.contains(&("sub", true)), "{names:?}");
        assert!(names.contains(&("empty", true)), "{names:?}");
        assert!(names.contains(&("a.txt", false)), "{names:?}");
        assert_eq!(items.len(), 4);
        let a = items.iter().find(|i| i.name == "b.txt").expect("b");
        assert_eq!(a.size, 2);
        assert!(a.modified.is_some());
        assert!(all(&s, &["empty".into()]).is_empty());
        let deep = all(&s, &["sub".into()]);
        assert_eq!(deep.len(), 1);
        assert!(deep[0].dir);
    }
}

#[test]
fn a_thousand_and_more_come_in_pages() {
    for (server, s) in storages("many") {
        let _ = server;
        for i in 0..1205 {
            put(&s, &[&format!("f{i:05}.txt")], b"x");
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
        put(&s, &["big.bin"], &big);
        put(&s, &["d", "x"], b"x");
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
        put(&s, &["a"], b"a");
        put(&s, &["b"], b"b");
        assert!(s.rename(&[], "a", "b").is_err());
        assert_eq!(s.read(&["b".into()], 9).expect("read"), b"b");
        s.rename(&[], "a", "c").expect("rename");
        assert!(s.stat(&[], "a").expect("stat").is_none());
        assert_eq!(s.read(&["c".into()], 9).expect("read"), b"a");
        put(&s, &["d", "e", "f"], b"f");
        put(&s, &["d", "g"], b"g");
        assert!(s.rename(&[], "d", "z").is_err());
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
        put(&s, &["a"], b"old");
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
        let bucket: Arc<dyn Storage> = Arc::new(s);
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
