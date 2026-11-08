//! The audit log as packwing's jobs write it: a record when each starts, one
//! when it ends and how, a job inside another the outer one's, and the chain
//! whole. One test, as the log is the process's.

#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::PathBuf;

use packwing_core::audit::{self, Config};
use packwing_core::{
    Control, ExtractOptions, Filter, Keys, Kind, NewArchive, convert, create, extract_selected,
    extract_with, preview_entry, read_entry, test,
};
use serde_json::{Map, Value};

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Scratch {
        let dir = std::env::temp_dir().join(format!("packwing-audit-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        Scratch(dir)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn config(folder: PathBuf, entries: bool) -> Config {
    Config {
        folder,
        retention_days: 90,
        max_bytes: 0,
        entries,
        system: false,
        app: "pkw".into(),
        version: "test".into(),
    }
}

/// The records of `action` that end, from `records`.
fn ends<'a>(records: &'a [Map<String, Value>], action: &str) -> Vec<&'a Map<String, Value>> {
    records
        .iter()
        .filter(|r| r["action"] == action && r["phase"] == "end")
        .collect()
}

#[test]
fn jobs_are_recorded_as_they_start_and_end_and_the_chain_holds() {
    let s = Scratch::new();
    let log = s.0.join("audit");
    let files = s.0.join("files");
    fs::create_dir_all(files.join("in")).unwrap();
    fs::write(files.join("in/ghi chú.txt"), "xin chào\n").unwrap();
    fs::write(files.join("in/b.txt"), "b").unwrap();

    // Not kept: nothing written.
    audit::set(None);
    let none = s.0.join("none.zip");
    create(
        &[files.join("in")],
        &none,
        &NewArchive::default(),
        &Control::default(),
    )
    .unwrap();
    assert!(!log.exists());

    audit::set(Some(config(log.clone(), false)));
    let zip = s.0.join("a.zip");
    let new = NewArchive {
        kind: Kind::Zip,
        password: Some(b"correct horse battery".to_vec()),
        ..NewArchive::default()
    };
    let made = create(&[files.join("in")], &zip, &new, &Control::default()).unwrap();
    let keys = Keys {
        password: Some(b"correct horse battery".to_vec()),
        ..Keys::default()
    };
    extract_with(
        &made,
        &s.0.join("out"),
        &keys,
        &ExtractOptions::default(),
        &Control::default(),
    )
    .unwrap();
    test(&made, &keys, &Control::default()).unwrap();
    // Failed: the wrong password.
    let wrong = Keys {
        password: Some(b"nope".to_vec()),
        ..Keys::default()
    };
    assert!(test(&made, &wrong, &Control::default()).is_err());
    // Converting extracts inside: one job.
    convert(
        &made,
        &keys,
        &s.0.join("a.tar.zst"),
        &NewArchive {
            kind: Kind::TarZst,
            ..NewArchive::default()
        },
        &ExtractOptions::default(),
        &Filter::default(),
        &Control::default(),
    )
    .unwrap();

    let records = audit::read(&log, None).unwrap();
    let created = ends(&records, "create");
    assert_eq!(created.len(), 1);
    let start = records
        .iter()
        .find(|r| r["action"] == "create" && r["phase"] == "start")
        .unwrap();
    assert_eq!(start["job"], created[0]["job"]);
    assert_eq!(start["kind"], "zip");
    assert_eq!(start["encryption"], "zip-aes256");
    assert_eq!(
        start["path_names"][0],
        files.join("in").to_string_lossy().as_ref()
    );
    assert_eq!(created[0]["result"], "ok");
    // Never the password.
    let text: String = fs::read_dir(&log)
        .unwrap()
        .map(|e| fs::read_to_string(e.unwrap().path()).unwrap_or_default())
        .collect();
    assert!(!text.contains("correct horse"), "a password in the log");

    let extracted = ends(&records, "extract");
    assert_eq!(extracted.len(), 1, "converting is no extract of its own");
    assert_eq!(extracted[0]["result"], "ok");
    assert_eq!(extracted[0]["files"], 2);
    assert_eq!(extracted[0]["format"], "zip");
    // Summary only: no names of the archive's files.
    assert!(extracted[0].get("damaged_names").is_none());

    let tested = ends(&records, "test");
    assert_eq!(tested.len(), 2);
    assert_eq!(tested[0]["result"], "ok");
    assert_eq!(tested[1]["result"], "failed");
    assert!(tested[1]["error"].as_str().is_some_and(|e| !e.is_empty()));

    let converted = ends(&records, "convert");
    assert_eq!(converted.len(), 1);
    assert_eq!(converted[0]["result"], "ok");
    assert_eq!(converted[0]["files"], 2);

    for r in &records {
        assert_eq!(r["app"], "pkw");
        assert_eq!(r["v"], 1);
        assert!(r["time"].as_str().is_some_and(|t| t.ends_with('Z')));
    }

    // What the user picked is named, as what the job is of.
    extract_selected(
        &made,
        &s.0.join("picked"),
        &[vec!["in".into(), "b.txt".into()]],
        &keys,
        &ExtractOptions::default(),
        &Control::default(),
    )
    .unwrap();
    let records = audit::read(&log, None).unwrap();
    let start = records
        .iter()
        .rev()
        .find(|r| r["action"] == "extract" && r["phase"] == "start")
        .unwrap();
    assert_eq!(start["picked_names"][0], "in/b.txt");

    // A preview is recorded the first time only; `pkw cat` each time.
    let entry = ["in".to_owned(), "b.txt".to_owned()];
    let before = audit::read(&log, None).unwrap().len();
    for _ in 0..3 {
        preview_entry(&made, &entry, &keys, 1 << 20, &Control::default()).unwrap();
    }
    read_entry(&made, &entry, &keys, 1 << 20, &Control::default()).unwrap();
    let records = audit::read(&log, None).unwrap();
    let read: Vec<_> = records[before..]
        .iter()
        .filter(|r| r["action"] == "read_entry" && r["phase"] == "start")
        .collect();
    assert_eq!(read.len(), 2);
    assert_eq!(read[0]["via"], "preview");
    assert_eq!(read[0]["entry"], "in/b.txt");
    assert!(read[1].get("via").is_none());
    // A preview that failed is recorded again when it works.
    let other = ["in".to_owned(), "ghi chú.txt".to_owned()];
    assert!(preview_entry(&made, &other, &wrong, 1 << 20, &Control::default()).is_err());
    preview_entry(&made, &other, &keys, 1 << 20, &Control::default()).unwrap();
    let records = audit::read(&log, None).unwrap();
    let finished = ends(&records, "read_entry");
    let last: Vec<_> = finished
        .iter()
        .rev()
        .take(2)
        .map(|r| &r["result"])
        .collect();
    assert_eq!(last, ["ok", "failed"]);

    // Names of what packwing came on, where the organization asks for them.
    audit::set(Some(config(log.clone(), true)));
    extract_selected(
        &made,
        &s.0.join("picked-again"),
        &[vec!["in".into(), "b.txt".into()]],
        &keys,
        &ExtractOptions::default(),
        &Control::default(),
    )
    .unwrap();
    let records = audit::read(&log, None).unwrap();
    let start = records
        .iter()
        .rev()
        .find(|r| r["action"] == "extract" && r["phase"] == "start")
        .unwrap();
    assert_eq!(start["picked_names"][0], "in/b.txt");

    // A file with no signature beside it is no check: the app looks for one
    // each time a file is chosen, and nothing is recorded.
    let before = audit::read(&log, None).unwrap().len();
    let here = packwing_core::LocalFolder::new(&s.0);
    let signed =
        packwing_core::check_signature(&here, &["a.zip".to_owned()], &[], &Control::default())
            .unwrap();
    assert_eq!(signed, packwing_core::Signed::No);
    assert_eq!(audit::read(&log, None).unwrap().len(), before);
    let records = audit::read(&log, None).unwrap();

    // A record written by hand, or changed, shows.
    let report = audit::verify(&log).unwrap();
    assert!(report.breaks.is_empty(), "{report:?}");
    assert_eq!(report.records, records.len());
    let (_, day) = audit::day_files(&log).unwrap().pop().unwrap();
    let text = fs::read_to_string(&day).unwrap();
    fs::write(&day, text.replacen("\"ok\"", "\"failed\"", 1)).unwrap();
    assert_eq!(audit::verify(&log).unwrap().breaks.len(), 1);
    audit::set(None);
}
