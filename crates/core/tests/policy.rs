//! What is refused where an organization requires encryption. The policy is the
//! process's, so these share one.

use std::fs;
use std::path::{Path, PathBuf};

use packwing_core::{Control, Keys, Kind, NewArchive, add, create};
use packwing_policy::{
    ALLOWED_ENCRYPTION_METHODS, Found, MIN_PASSWORD_LENGTH, Policy, REQUIRE_ENCRYPTION, Value,
};

fn folder(name: &str) -> PathBuf {
    packwing_policy::install(Policy::of(Found {
        origin: "test".into(),
        values: vec![
            (REQUIRE_ENCRYPTION.into(), Value::Bool(true)),
            (MIN_PASSWORD_LENGTH.into(), Value::Int(15)),
            (
                ALLOWED_ENCRYPTION_METHODS.into(),
                Value::Text("7z-aes256, age-passphrase".into()),
            ),
        ],
        failure: None,
    }));
    let dir = std::env::temp_dir().join(format!("packwing-policy-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("a folder");
    fs::write(dir.join("file.txt"), "held").expect("a file");
    dir
}

fn make(dir: &Path, kind: Kind, password: Option<&str>) -> Result<PathBuf, String> {
    let new = NewArchive {
        kind,
        password: password.map(|p| p.as_bytes().to_vec()),
        ..NewArchive::default()
    };
    create(
        &[dir.join("file.txt")],
        &dir.join("made"),
        &new,
        &Control::default(),
    )
    .map_err(|e| e.to_string())
}

const LONG: &str = "fifteen or more characters";

#[test]
fn nothing_goes_out_unencrypted() {
    let dir = folder("clear");
    for kind in [Kind::Zip, Kind::SevenZip, Kind::TarZst] {
        let why = make(&dir, kind, None).expect_err("it has to be encrypted");
        assert!(
            why.contains("requires new archives to be encrypted"),
            "{why}"
        );
    }
    assert!(!dir.join("made").exists());
}

#[test]
fn a_kind_of_encryption_not_allowed_is_refused() {
    let dir = folder("method");
    let why = make(&dir, Kind::Zip, Some(LONG)).expect_err("ZIP's is not allowed");
    assert!(why.contains("zip-aes256"), "{why}");
}

#[test]
fn a_short_password_is_refused() {
    let dir = folder("short");
    let why = make(&dir, Kind::SevenZip, Some("short")).expect_err("too short");
    assert!(why.contains("at least 15"), "{why}");
}

#[test]
fn what_is_allowed_is_made() {
    let dir = folder("allowed");
    assert!(make(&dir, Kind::SevenZip, Some(LONG)).expect("7z").exists());
    assert!(make(&dir, Kind::TarZst, Some(LONG)).expect("age").exists());
}

#[test]
fn a_zip_archive_encrypted_whole_with_age_is_age_s_to_allow() {
    let dir = folder("zip-age");
    let new = NewArchive {
        kind: Kind::Zip,
        password: Some(LONG.as_bytes().to_vec()),
        whole_with_age: true,
        ..NewArchive::default()
    };
    let made = create(
        &[dir.join("file.txt")],
        &dir.join("made"),
        &new,
        &Control::default(),
    )
    .expect("age-passphrase is allowed, though zip-aes256 is not");
    assert!(made.exists());
}

/// An empty ZIP-with-one-empty-file archive, in the clear, as one from elsewhere.
fn clear_zip(path: &Path) {
    let name = b"a.txt";
    let mut local = Vec::new();
    local.extend_from_slice(b"PK\x03\x04\x14\0\0\0\0\0\0\0\0\0");
    local.extend_from_slice(&[0; 12]);
    local.extend_from_slice(&(name.len() as u16).to_le_bytes());
    local.extend_from_slice(&[0; 2]);
    local.extend_from_slice(name);
    let mut central = Vec::new();
    central.extend_from_slice(b"PK\x01\x02\x14\0\x14\0\0\0\0\0\0\0\0\0");
    central.extend_from_slice(&[0; 12]);
    central.extend_from_slice(&(name.len() as u16).to_le_bytes());
    central.extend_from_slice(&[0; 16]);
    central.extend_from_slice(name);
    let mut zip = local.clone();
    zip.extend_from_slice(&central);
    zip.extend_from_slice(b"PK\x05\x06\0\0\0\0\x01\0\x01\0");
    zip.extend_from_slice(&(central.len() as u32).to_le_bytes());
    zip.extend_from_slice(&(local.len() as u32).to_le_bytes());
    zip.extend_from_slice(&[0; 2]);
    fs::write(path, zip).expect("the archive");
}

#[test]
fn nothing_is_added_in_the_clear() {
    let dir = folder("adding");
    let zip = dir.join("clear.zip");
    clear_zip(&zip);
    let before = fs::read(&zip).expect("the archive");
    let why = add(
        &[dir.join("file.txt")],
        &zip,
        &[],
        &Keys::default(),
        &Control::default(),
    )
    .expect_err("it would add a file in the clear")
    .to_string();
    assert!(
        why.contains("requires new archives to be encrypted"),
        "{why}"
    );
    assert_eq!(fs::read(&zip).expect("the archive"), before);
}
