//! `pkw` as a script runs it: its output, its JSON and its exit codes, on
//! archives it makes itself and on the core's test data.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// A folder of a test's own, gone when it ends, holding a small tree.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Scratch {
        let dir = std::env::temp_dir().join(format!("pkw-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src/docs")).expect("a folder");
        std::fs::write(dir.join("src/docs/notes.txt"), "xin chào\n").expect("a file");
        std::fs::write(dir.join("src/numbers.txt"), "1\n2\n3\n").expect("a file");
        Scratch(dir)
    }

    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }

    /// `pkw args…`, run in the folder, with no password from the test's own
    /// environment and nothing to read.
    fn pkw(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_pkw"))
            .args(args)
            .current_dir(&self.0)
            .env_remove("PACKWING_PASSWORD")
            .stdin(std::process::Stdio::null())
            .output()
            .expect("pkw runs")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn ok(o: &Output) -> String {
    assert!(
        o.status.success(),
        "{}\n{}",
        stdout(o),
        String::from_utf8_lossy(&o.stderr)
    );
    stdout(o)
}

fn fixture(name: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../core/tests/data")
        .join(name)
        .display()
        .to_string()
}

#[test]
fn each_format_is_made_listed_tested_and_extracted() {
    let s = Scratch::new("formats");
    for name in [
        "a.zip",
        "a.7z",
        "a.tar",
        "a.tar.zst",
        "a.tgz",
        "a.tar.xz",
        "a.tbz2",
    ] {
        ok(&s.pkw(&["create", "-q", name, "src"]));
        let listed = ok(&s.pkw(&["list", name]));
        assert!(
            listed.lines().any(|l| l == "src/docs/notes.txt"),
            "{name}: {listed}"
        );
        ok(&s.pkw(&["test", "-q", name]));
        let out = format!("out-{name}");
        ok(&s.pkw(&["extract", "-q", name, "-o", &out]));
        let notes = std::fs::read_to_string(s.path(&out).join("src/docs/notes.txt"));
        assert_eq!(notes.expect(name), "xin chào\n");
    }
    // Its format by --format, whatever its name.
    ok(&s.pkw(&["create", "-q", "-f", "7z", "made.bin", "src"]));
    assert!(stdout(&s.pkw(&["info", "made.bin"])).contains("SevenZip"));
}

#[test]
fn extract_makes_a_folder_named_after_the_archive_and_picks_files() {
    let s = Scratch::new("extract");
    ok(&s.pkw(&["create", "-q", "Báo cáo.tar.zst", "src"]));
    ok(&s.pkw(&["extract", "-q", "Báo cáo.tar.zst"]));
    assert!(s.path("Báo cáo/src/numbers.txt").is_file());
    // What is picked goes straight in, as the app extracts it.
    ok(&s.pkw(&["x", "-q", "Báo cáo.tar.zst", "src/docs", "-o", "picked"]));
    assert!(s.path("picked/docs/notes.txt").is_file());
    assert!(!s.path("picked/numbers.txt").exists());
    // Again, the file is there: it stops, unless told to replace it.
    let again = s.pkw(&["x", "-q", "Báo cáo.tar.zst", "src/docs", "-o", "picked"]);
    assert_eq!(again.status.code(), Some(1));
    ok(&s.pkw(&[
        "x",
        "-q",
        "Báo cáo.tar.zst",
        "src/docs",
        "-o",
        "picked",
        "--overwrite",
    ]));
    assert_eq!(
        ok(&s.pkw(&["cat", "Báo cáo.tar.zst", "src/docs/notes.txt"])),
        "xin chào\n"
    );
}

#[test]
fn a_password_comes_from_a_file_and_age_files_are_named_so() {
    let s = Scratch::new("secrets");
    std::fs::write(s.path("pw"), "khoá bí mật\n").expect("the file");
    ok(&s.pkw(&[
        "--password-file",
        "pw",
        "create",
        "-q",
        "-p",
        "--hide-names",
        "s.7z",
        "src",
    ]));
    // Without it, and nothing to ask: 3.
    let locked = s.pkw(&["list", "s.7z"]);
    assert_eq!(locked.status.code(), Some(3));
    let listed = ok(&s.pkw(&["--password-file", "pw", "list", "s.7z"]));
    assert!(listed.contains("src/numbers.txt"));
    // A tar archive with a password is encrypted whole with age, and named so.
    ok(&s.pkw(&[
        "--password-file",
        "pw",
        "create",
        "-q",
        "-p",
        "t.tar.zst",
        "src",
    ]));
    assert!(s.path("t.tar.zst.age").is_file());
    ok(&s.pkw(&["--password-file", "pw", "test", "-q", "t.tar.zst.age"]));
    std::fs::write(s.path("wrong"), "nope\n").expect("the file");
    let wrong = s.pkw(&["--password-file", "wrong", "test", "t.tar.zst.age"]);
    assert_eq!(wrong.status.code(), Some(3));
    // Asked for one with nowhere to read it from: not understood.
    assert_eq!(
        s.pkw(&["create", "-p", "u.zip", "src"]).status.code(),
        Some(2)
    );
}

#[test]
fn a_zip_archive_is_changed_in_place() {
    let s = Scratch::new("change");
    ok(&s.pkw(&["create", "-q", "a.zip", "src"]));
    std::fs::write(s.path("extra.txt"), "new").expect("the file");
    ok(&s.pkw(&["add", "-q", "a.zip", "extra.txt", "--into", "src/docs"]));
    ok(&s.pkw(&["rename", "-q", "a.zip", "src/docs/extra.txt", "more.txt"]));
    ok(&s.pkw(&["delete", "-q", "a.zip", "src/numbers.txt"]));
    let listed = ok(&s.pkw(&["list", "a.zip"]));
    assert!(listed.contains("src/docs/more.txt"), "{listed}");
    assert!(!listed.contains("numbers.txt"), "{listed}");
    // Only a ZIP archive changes.
    ok(&s.pkw(&["create", "-q", "a.7z", "src"]));
    assert_eq!(
        s.pkw(&["delete", "a.7z", "src/numbers.txt"]).status.code(),
        Some(1)
    );
}

#[test]
fn checksums_are_written_as_sha256sum_does_and_checked() {
    let s = Scratch::new("sums");
    let sums = ok(&s.pkw(&["checksum", "src/numbers.txt", "src/docs/notes.txt"]));
    // As `printf '1\n2\n3\n' | sha256sum` says.
    assert!(sums.starts_with(
        "14c5e74c4b96ccef41cd94db73a9ec3348038ac094feca4fd897cecffa07cdae  src/numbers.txt\n"
    ));
    std::fs::write(s.path("SUMS"), &sums).expect("the file");
    ok(&s.pkw(&["checksum", "--check", "SUMS"]));
    std::fs::write(s.path("src/numbers.txt"), "changed").expect("the file");
    let failed = s.pkw(&["checksum", "-c", "SUMS"]);
    assert_eq!(failed.status.code(), Some(1));
    assert!(stdout(&failed).contains("src/numbers.txt: FAILED"));
    ok(&s.pkw(&["checksum", "-q", "--save", "src/numbers.txt"]));
    assert!(s.path("src/numbers.txt.sha256").is_file());
}

#[test]
fn json_is_what_a_script_reads() {
    let s = Scratch::new("json");
    ok(&s.pkw(&["create", "-q", "a.zip", "src"]));
    let listed: serde_json::Value =
        serde_json::from_str(&ok(&s.pkw(&["--json", "list", "a.zip"]))).expect("JSON");
    let paths: Vec<&str> = listed
        .as_array()
        .expect("a list")
        .iter()
        .filter_map(|e| e["path"].as_str())
        .collect();
    assert!(paths.contains(&"src/docs/notes.txt"), "{paths:?}");
    let info: serde_json::Value =
        serde_json::from_str(&ok(&s.pkw(&["--json", "info", "a.zip"]))).expect("JSON");
    assert_eq!(info["container"], "Zip");
    assert_eq!(info["files"], 2);
}

#[test]
fn what_is_not_an_archive_or_not_understood_says_so_by_its_code() {
    let s = Scratch::new("codes");
    assert_eq!(s.pkw(&["list", "src/numbers.txt"]).status.code(), Some(4));
    assert_eq!(s.pkw(&["frobnicate"]).status.code(), Some(2));
    assert_eq!(s.pkw(&["create", "a.rar", "src"]).status.code(), Some(2));
    assert_eq!(
        s.pkw(&["create", "-l", "max", "a.zip", "src"])
            .status
            .code(),
        Some(2)
    );
    // The core's archives: a RAR and an ISO image, read.
    let rar = ok(&s.pkw(&["list", &fixture("rar5_multiple_files.rar")]));
    assert!(!rar.is_empty());
    ok(&s.pkw(&["test", "-q", &fixture("sample.iso.zst")]));
}
