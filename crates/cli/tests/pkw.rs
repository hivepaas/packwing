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
    /// environment, nothing to read, and the app's settings the test's own:
    /// `config/packwing/settings`, which [`Scratch::settings`] writes.
    fn pkw(&self, args: &[&str]) -> Output {
        self.pkw_with(args, &[])
    }

    /// [`Scratch::pkw`], with these in its environment too.
    fn pkw_with(&self, args: &[&str], env: &[(&str, &str)]) -> Output {
        let config = self.path("config");
        let mut pkw = Command::new(env!("CARGO_BIN_EXE_pkw"));
        pkw.args(args)
            .current_dir(&self.0)
            .env_remove("PACKWING_PASSWORD")
            .env_remove("PACKWING_MEMORY")
            .envs(env.iter().copied())
            .env("XDG_CONFIG_HOME", &config)
            .env("APPDATA", &config)
            .stdin(std::process::Stdio::null());
        if cfg!(target_os = "macos") {
            // Its settings are under HOME, in Library/Application Support.
            pkw.env("HOME", self.path("home"));
        }
        pkw.output().expect("pkw runs")
    }

    /// The app's settings, as it writes them.
    fn settings(&self, text: &str) {
        let folder = if cfg!(target_os = "macos") {
            self.path("home/Library/Application Support/packwing")
        } else {
            self.path("config/packwing")
        };
        std::fs::create_dir_all(&folder).expect("a folder");
        std::fs::write(folder.join("settings"), text).expect("the settings");
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
fn a_zip_or_7z_archive_is_changed_in_place() {
    let s = Scratch::new("change");
    ok(&s.pkw(&["create", "-q", "a.zip", "src"]));
    std::fs::write(s.path("extra.txt"), "new").expect("the file");
    ok(&s.pkw(&["add", "-q", "a.zip", "extra.txt", "--into", "src/docs"]));
    ok(&s.pkw(&["rename", "-q", "a.zip", "src/docs/extra.txt", "more.txt"]));
    ok(&s.pkw(&["delete", "-q", "a.zip", "src/numbers.txt"]));
    let listed = ok(&s.pkw(&["list", "a.zip"]));
    assert!(listed.contains("src/docs/more.txt"), "{listed}");
    assert!(!listed.contains("numbers.txt"), "{listed}");
    // A 7z archive changes too.
    ok(&s.pkw(&["create", "-q", "a.7z", "src"]));
    ok(&s.pkw(&["add", "-q", "a.7z", "extra.txt", "--into", "src/docs"]));
    ok(&s.pkw(&["delete", "-q", "a.7z", "src/numbers.txt"]));
    let listed = ok(&s.pkw(&["list", "a.7z"]));
    assert!(listed.contains("src/docs/extra.txt"), "{listed}");
    assert!(!listed.contains("numbers.txt"), "{listed}");
    ok(&s.pkw(&["rename", "-q", "a.7z", "src/docs/extra.txt", "more.txt"]));
    let listed = ok(&s.pkw(&["list", "a.7z"]));
    assert!(listed.contains("src/docs/more.txt"), "{listed}");
    ok(&s.pkw(&["test", "-q", "a.7z"]));
    // Nor is a tar archive changed.
    ok(&s.pkw(&["create", "-q", "a.tar.gz", "src"]));
    assert_eq!(
        s.pkw(&["delete", "a.tar.gz", "src/docs/notes.txt"])
            .status
            .code(),
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

/// A 64-bit ELF program marked as an AppImage of type `kind`, its section
/// headers at its end, then `image`: a type 2 AppImage of a SquashFS image.
/// A type 1 is an ISO 9660 image with the program in its first 32 KiB.
fn appimage(kind: u8, image: &[u8]) -> Vec<u8> {
    let mut f = vec![0u8; 4096 + 7 * 64];
    f[..4].copy_from_slice(b"\x7fELF");
    f[4..7].copy_from_slice(&[2, 1, 1]);
    f[8..11].copy_from_slice(&[b'A', b'I', kind]);
    f[40..48].copy_from_slice(&4096u64.to_le_bytes());
    f[58..60].copy_from_slice(&64u16.to_le_bytes());
    f[60..62].copy_from_slice(&7u16.to_le_bytes());
    if kind == 1 {
        let mut iso = image.to_vec();
        iso[..f.len()].copy_from_slice(&f);
        return iso;
    }
    f.extend_from_slice(image);
    f
}

#[test]
fn packages_snaps_and_appimages_are_listed_tested_extracted_and_converted() {
    let s = Scratch::new("packages");
    let read = |name: &str| std::fs::read(fixture(name)).expect("the fixture");
    let sqfs = read("sample.sqfs");
    let iso = {
        let zst = read("sample.iso.zst");
        let mut out = vec![0u8; 1 << 20];
        let n = hivepack::zstd::decompress(&mut out, &zst).expect("the image");
        out.truncate(n);
        out
    };
    std::fs::write(s.path("New.AppImage"), appimage(2, &sqfs)).expect("a file");
    std::fs::write(s.path("Old.AppImage"), appimage(1, &iso)).expect("a file");
    std::fs::write(s.path("tool.snap"), &sqfs).expect("a file");
    std::fs::copy(fixture("sample-xz.deb"), s.path("tool_1.0_all.deb")).expect("a copy");
    for (name, format, container) in [
        ("tool_1.0_all.deb", "Tar in Xz in Deb", "Tar"),
        ("tool.snap", "Squashfs", "Squashfs"),
        ("New.AppImage", "AppImageType2", "AppImageType2"),
        ("Old.AppImage", "AppImageType1", "AppImageType1"),
    ] {
        let listed = ok(&s.pkw(&["list", name]));
        assert!(
            listed.lines().any(|l| l == "docs/notes.txt"),
            "{name}: {listed}"
        );
        let info = ok(&s.pkw(&["info", name]));
        assert!(
            info.lines()
                .any(|l| l.starts_with("Format") && l.ends_with(format)),
            "{name}: {info}"
        );
        let json: serde_json::Value =
            serde_json::from_str(&ok(&s.pkw(&["--json", "info", name]))).expect("JSON");
        assert_eq!(json["container"], container, "{name}");
        assert_eq!(json["files"], 3, "{name}");
        ok(&s.pkw(&["test", "-q", name]));
        // Into a folder named after it, without its extension.
        ok(&s.pkw(&["extract", "-q", name]));
        let stem = name.rsplit_once('.').expect("an extension").0;
        assert_eq!(
            std::fs::read(s.path(stem).join("docs/notes.txt")).expect(name),
            b"Notes from inside the archive.\n"
        );
        let picked = format!("picked-{name}");
        ok(&s.pkw(&["x", "-q", name, "docs/plans", "-o", &picked]));
        assert!(s.path(&picked).join("plans/a.txt").is_file(), "{name}");
        assert!(!s.path(&picked).join("readme.md").exists(), "{name}");
        assert_eq!(
            ok(&s.pkw(&["cat", name, "readme.md"])),
            "# Read me\n",
            "{name}"
        );
        let zip = format!("{stem}.zip");
        ok(&s.pkw(&["convert", "-q", name, &zip]));
        let converted = ok(&s.pkw(&["list", &zip]));
        assert!(
            converted.lines().any(|l| l == "docs/plans/a.txt"),
            "{name}: {converted}"
        );
    }
}

#[test]
fn a_file_is_signed_and_its_signature_checked() {
    let s = Scratch::new("sign");
    std::fs::write(s.path("kp"), "k3y pass\n").expect("the file");
    // The folder is made if it is not there.
    let made = ok(&s.pkw(&["--password-file", "kp", "keygen", "-o", "keys/mine"]));
    let mut lines = made.lines();
    let key = lines.next().expect("the key's file").to_owned();
    let public = lines.next().expect("its public key").to_owned();
    assert!(Path::new(&public).is_file() || s.path(&public).is_file());
    ok(&s.pkw(&["create", "-q", "a.zip", "src"]));
    ok(&s.pkw(&["--password-file", "kp", "sign", "-q", "a.zip", "--by", &key]));
    assert!(s.path("a.zip.sig").is_file());
    // Trusted, it is good; by a key not trusted, it is not.
    ok(&s.pkw(&["verify", "a.zip", "--only", "--trust", &public]));
    assert_eq!(s.pkw(&["verify", "a.zip", "--only"]).status.code(), Some(1));
    // Changed after it was signed.
    std::fs::write(s.path("a.zip"), "changed").expect("the file");
    let changed = s.pkw(&["verify", "a.zip", "--only", "--trust", &public]);
    assert_eq!(changed.status.code(), Some(1));
    assert!(stdout(&changed).contains("changed since it was signed"));
    // A wrong passphrase for the key, and nothing to ask: 3.
    std::fs::write(s.path("bad"), "nope\n").expect("the file");
    let wrong = s.pkw(&[
        "--password-file",
        "bad",
        "sign",
        "src/numbers.txt",
        "--by",
        &key,
    ]);
    assert_eq!(wrong.status.code(), Some(3));
}

#[test]
fn an_archive_is_converted_and_a_passphrase_suggested() {
    let s = Scratch::new("convert");
    ok(&s.pkw(&["create", "-q", "a.zip", "src"]));
    ok(&s.pkw(&["convert", "-q", "a.zip", "b.tar.zst"]));
    let listed = ok(&s.pkw(&["list", "b.tar.zst"]));
    assert!(listed.contains("src/docs/notes.txt"), "{listed}");
    // Nothing of the work is left beside it.
    let left: Vec<_> = std::fs::read_dir(&s.0)
        .expect("the folder")
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with(".pkw-convert"))
        .collect();
    assert!(left.is_empty());
    let suggested = ok(&s.pkw(&["passphrase", "--min", "50"]));
    let suggested = suggested.trim();
    assert!(suggested.len() >= 50, "{suggested}");
    assert!(suggested.split('-').count() >= 6, "{suggested}");
}

#[test]
fn patterns_say_what_goes_in() {
    let s = Scratch::new("patterns");
    for name in [
        "src/build/o.o",
        "src/docs/build",
        "src/app.LOG",
        "src/gen/x/made.rs",
    ] {
        let path = s.path(name);
        std::fs::create_dir_all(path.parent().expect("a folder")).expect("a folder");
        std::fs::write(path, "x\n").expect("a file");
    }
    let files = |archive: &str| -> Vec<String> {
        let listed = ok(&s.pkw(&["list", archive]));
        let mut files: Vec<String> = listed
            .lines()
            .filter(|l| !l.ends_with('/'))
            .map(str::to_owned)
            .collect();
        files.sort();
        files
    };
    // A folder pattern leaves a file of that name; letters match in either
    // case; `**` is any folders; a path is in the folder given.
    ok(&s.pkw(&[
        "create",
        "-q",
        "out.zip",
        "src",
        "--exclude",
        "build/",
        "--exclude",
        "*.log",
        "--exclude",
        "gen/**/made.rs",
    ]));
    assert_eq!(
        files("out.zip"),
        ["src/docs/build", "src/docs/notes.txt", "src/numbers.txt"]
    );
    ok(&s.pkw(&[
        "create",
        "-q",
        "docs.zip",
        "src",
        "--include",
        "docs/",
        "--exclude",
        "build",
    ]));
    assert_eq!(files("docs.zip"), ["src/docs/notes.txt"]);
    // Converting, the paths are the archive's.
    ok(&s.pkw(&["create", "-q", "all.tar", "src"]));
    ok(&s.pkw(&[
        "convert",
        "-q",
        "all.tar",
        "picked.7z",
        "--exclude",
        "src/build/",
    ]));
    let picked = files("picked.7z");
    assert!(
        !picked.iter().any(|f| f.starts_with("src/build/")),
        "{picked:?}"
    );
    assert!(picked.contains(&"src/docs/build".to_owned()), "{picked:?}");
    // Adding, as creating.
    ok(&s.pkw(&[
        "add",
        "-q",
        "docs.zip",
        "src",
        "--into",
        "more",
        "--exclude",
        "*.log",
        "--exclude",
        "build/",
        "--exclude",
        "gen/",
    ]));
    assert_eq!(
        files("docs.zip"),
        [
            "more/src/docs/build",
            "more/src/docs/notes.txt",
            "more/src/numbers.txt",
            "src/docs/notes.txt",
        ]
    );
    let none = s.pkw(&["add", "-q", "docs.zip", "src", "--include", "*.pdf"]);
    assert_eq!(none.status.code(), Some(1), "{none:?}");
    // What nothing matches is said.
    let none = s.pkw(&["create", "-q", "none.zip", "src", "--include", "*.pdf"]);
    assert_eq!(none.status.code(), Some(1), "{none:?}");
    assert!(!s.path("none.zip").exists());
}

#[test]
fn a_folder_is_listed_and_files_copied_and_moved() {
    let s = Scratch::new("folders");
    std::fs::create_dir_all(s.path("out")).expect("a folder");
    let listed = ok(&s.pkw(&["list", "src"]));
    assert_eq!(listed, "docs/\nnumbers.txt\n", "folders first");
    ok(&s.pkw(&["copy", "-q", "src/docs", "src/numbers.txt", "out"]));
    assert_eq!(
        std::fs::read_to_string(s.path("out/docs/notes.txt")).expect("copied"),
        "xin chào\n"
    );
    assert!(s.path("src/numbers.txt").exists(), "a copy leaves it");
    // The name is taken now: stop, with code 1; skip, and nothing changes.
    assert_eq!(
        s.pkw(&["cp", "src/numbers.txt", "out"]).status.code(),
        Some(1)
    );
    let skipped = ok(&s.pkw(&[
        "--json",
        "cp",
        "--on-existing",
        "skip",
        "src/numbers.txt",
        "out",
    ]));
    let skipped: serde_json::Value = serde_json::from_str(&skipped).expect("JSON");
    assert_eq!(skipped["skipped"], 1);
    std::fs::create_dir_all(s.path("moved")).expect("a folder");
    ok(&s.pkw(&["mv", "-q", "out/docs", "moved"]));
    assert!(s.path("moved/docs/notes.txt").exists());
    assert!(!s.path("out/docs").exists(), "a move takes it away");
    // Into what is not a folder, a usage error.
    assert_eq!(
        s.pkw(&["cp", "src/docs", "src/numbers.txt"]).status.code(),
        Some(2)
    );
}

#[test]
fn a_storage_added_to_the_app_is_named_name_colon_path() {
    let s = Scratch::new("storages");
    let kept = s.path("kept");
    std::fs::create_dir_all(kept.join("in")).expect("a folder");
    s.settings(&format!(
        "storage=local Tải về\t{}\nstorage=other s3\tSao lưu\ts3-1\tbucket=b\tregion=r\tasks_first=yes\n",
        kept.display()
    ));
    let storages = ok(&s.pkw(&["--json", "storages"]));
    let storages: serde_json::Value = serde_json::from_str(&storages).expect("JSON");
    assert_eq!(storages[0]["name"], "Tải về");
    assert_eq!(storages[0]["opens"], true);
    assert_eq!(storages[1]["asks_first"], true);
    assert_eq!(storages[1]["opens"], false);
    // An archive made here, copied there, and read where it is.
    ok(&s.pkw(&["create", "-q", "a.zip", "src"]));
    ok(&s.pkw(&["cp", "-q", "a.zip", "Tải về:in"]));
    assert!(kept.join("in/a.zip").exists());
    assert_eq!(ok(&s.pkw(&["list", "Tải về:"])), "in/\n");
    assert!(
        ok(&s.pkw(&["list", "Tải về:in/a.zip"]))
            .lines()
            .any(|l| l == "src/numbers.txt")
    );
    assert_eq!(
        ok(&s.pkw(&["cat", "Tải về:in/a.zip", "src/numbers.txt"])),
        "1\n2\n3\n"
    );
    ok(&s.pkw(&["test", "-q", "Tải về:in/a.zip"]));
    ok(&s.pkw(&["extract", "-q", "Tải về:in/a.zip", "-o", "got"]));
    assert!(s.path("got/src/docs/notes.txt").exists());
    let sum = ok(&s.pkw(&["checksum", "Tải về:in/a.zip"]));
    assert!(sum.ends_with("  Tải về:in/a.zip\n"), "{sum}");
    // One that asks for this computer's password first is not opened.
    let refused = s.pkw(&["list", "Sao lưu:x.zip"]);
    assert_eq!(refused.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&refused.stderr).contains("open it in packwing"));
    // A name no storage has is a path here.
    assert_eq!(s.pkw(&["list", "Không có:a.zip"]).status.code(), Some(1));
    if !cfg!(feature = "remote") {
        // Built without S3, SFTP and WebDAV: one of them says so, and is not opened.
        s.settings("storage=other s3\tKho\ts3-2\tbucket=b\n");
        let without = s.pkw(&["list", "Kho:a.zip"]);
        assert_eq!(without.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&without.stderr).contains("built without"));
    }
}

#[test]
fn completions_and_manual_pages_are_written() {
    let s = Scratch::new("man");
    for shell in ["bash", "zsh", "fish", "powershell", "elvish"] {
        let written = ok(&s.pkw(&["completions", shell]));
        assert!(written.contains("extract"), "{shell}: {written}");
    }
    assert_eq!(s.pkw(&["completions", "tcsh"]).status.code(), Some(2));
    ok(&s.pkw(&["man", "pages"]));
    let page = std::fs::read_to_string(s.path("pages/pkw.1")).expect("pkw.1");
    assert!(page.contains("Exit codes"), "{page}");
    assert!(s.path("pages/pkw-extract.1").exists());
    assert!(s.path("pages/pkw-storages.1").exists());
    assert!(!s.path("pages/pkw-help.1").exists());
    assert!(
        !s.path("pages/pkw-man.1").exists(),
        "a hidden command has none"
    );
}

#[test]
fn what_only_a_mac_uses_is_left_out_unless_kept() {
    let s = Scratch::new("mac");
    std::fs::create_dir_all(s.path("src/__MACOSX")).expect("a folder");
    std::fs::write(s.path("src/__MACOSX/._numbers.txt"), "fork").expect("a file");
    std::fs::write(s.path("src/.DS_Store"), "ds").expect("a file");
    std::fs::write(s.path("src/docs/._notes.txt"), "fork").expect("a file");
    let mac = |listed: &str| {
        listed
            .lines()
            .filter(|l| l.contains("__MACOSX") || l.contains(".DS_Store") || l.contains("/._"))
            .count()
    };
    ok(&s.pkw(&["create", "-q", "left.zip", "src"]));
    let left = ok(&s.pkw(&["list", "left.zip"]));
    assert_eq!(mac(&left), 0, "{left}");
    assert!(left.lines().any(|l| l == "src/docs/notes.txt"));
    ok(&s.pkw(&["create", "-q", "--keep-mac-files", "kept.tar", "src"]));
    assert_eq!(
        mac(&ok(&s.pkw(&["list", "kept.tar"]))),
        4,
        "the folder, and three files"
    );
    // A conversion leaves them out alike.
    ok(&s.pkw(&["convert", "-q", "kept.tar", "converted.7z"]));
    assert_eq!(mac(&ok(&s.pkw(&["list", "converted.7z"]))), 0);
}

#[test]
fn no_mark_is_taken_by_extract_copy_and_move() {
    let s = Scratch::new("no-mark");
    std::fs::create_dir_all(s.path("out")).expect("a folder");
    ok(&s.pkw(&["create", "-q", "a.zip", "src"]));
    ok(&s.pkw(&["extract", "-q", "--no-mark", "a.zip", "-o", "got"]));
    assert!(s.path("got/src/numbers.txt").exists());
    ok(&s.pkw(&["cp", "-q", "--no-mark", "a.zip", "out"]));
    ok(&s.pkw(&["mv", "-q", "--no-mark", "out/a.zip", "got"]));
    assert!(s.path("got/a.zip").exists());
}

/// Names Windows cannot take extract as they are where they can, and with
/// --portable-names are left out by default, or renamed, and said.
#[cfg(unix)]
#[test]
fn names_a_system_cannot_take_are_left_out_or_renamed() {
    let s = Scratch::new("bad-names");
    std::fs::write(s.path("src/a:b.txt"), "colon").expect("a file");
    ok(&s.pkw(&["create", "a.zip", "src"]));
    // As they are, here.
    ok(&s.pkw(&["extract", "a.zip", "-o", "native"]));
    assert!(s.path("native/src/a:b.txt").exists());
    // Left out, by default, and said: exit 5.
    let left = s.pkw(&["extract", "a.zip", "-o", "left", "--portable-names"]);
    assert_eq!(left.status.code(), Some(5));
    let said = String::from_utf8_lossy(&left.stderr);
    assert!(said.contains("left out: src/a:b.txt"), "{said}");
    assert!(s.path("left/src/numbers.txt").exists());
    assert!(!s.path("left/src/a:b.txt").exists());
    // Renamed, as JSON says.
    let json: serde_json::Value = serde_json::from_str(&ok(&s.pkw(&[
        "--json",
        "extract",
        "a.zip",
        "-o",
        "renamed",
        "--portable-names",
        "--on-bad-name",
        "rename",
    ])))
    .expect("JSON");
    assert_eq!(json["renamed"][0]["to"], "src/a_b.txt");
    assert_eq!(json["skipped"], serde_json::json!([]));
    assert!(s.path("renamed/src/a_b.txt").exists());
    // Asked, with no terminal: not understood.
    let asked = s.pkw(&["extract", "a.zip", "-o", "asked", "--on-bad-name", "ask"]);
    assert_eq!(asked.status.code(), Some(2));
}

/// Links that lead outside the folder are left out by default and said,
/// created as they are with --links-outside create, and stop it with stop;
/// converting keeps them, as the new archive holds any link.
#[cfg(unix)]
#[test]
fn links_that_lead_outside_are_left_out_created_or_stop() {
    let s = Scratch::new("links-outside");
    std::os::unix::fs::symlink("/etc/hostname", s.path("src/outside")).expect("a link");
    std::os::unix::fs::symlink("numbers.txt", s.path("src/inside")).expect("a link");
    ok(&s.pkw(&["create", "a.tar", "src"]));
    let target = |link: &str| std::fs::read_link(s.path(link)).ok();
    // Left out, by default with no terminal, and said: exit 5.
    let left = s.pkw(&["extract", "a.tar", "-o", "left"]);
    assert_eq!(left.status.code(), Some(5));
    let said = String::from_utf8_lossy(&left.stderr);
    assert!(said.contains("left out: src/outside"), "{said}");
    assert_eq!(target("left/src/inside"), Some("numbers.txt".into()));
    assert_eq!(target("left/src/outside"), None);
    // Created as they are.
    ok(&s.pkw(&[
        "extract",
        "a.tar",
        "-o",
        "made",
        "--links-outside",
        "create",
    ]));
    assert_eq!(target("made/src/outside"), Some("/etc/hostname".into()));
    // Stopped: nothing written.
    let stopped = s.pkw(&[
        "extract",
        "a.tar",
        "-o",
        "stopped",
        "--links-outside",
        "stop",
    ]);
    assert!(!stopped.status.success());
    assert!(!s.path("stopped").exists());
    // Asked, with no terminal: not understood.
    let asked = s.pkw(&["extract", "a.tar", "-o", "asked", "--links-outside", "ask"]);
    assert_eq!(asked.status.code(), Some(2));
    // Converted, the link kept.
    ok(&s.pkw(&["convert", "a.tar", "b.tar.zst"]));
    ok(&s.pkw(&[
        "extract",
        "b.tar.zst",
        "-o",
        "converted",
        "--links-outside",
        "create",
    ]));
    assert_eq!(
        target("converted/src/outside"),
        Some("/etc/hostname".into())
    );
}

#[test]
fn threads_change_how_soon_not_what_is_made_or_read() {
    let s = Scratch::new("threads");
    for name in ["a.tar.xz", "a.tar.zst", "a.tar.bz2", "a.7z", "a.zip"] {
        let made: Vec<Vec<u8>> = [&["-T", "1"][..], &["--threads", "3"], &[]]
            .iter()
            .enumerate()
            .map(|(i, threads)| {
                let file = format!("{i}-{name}");
                let mut args = vec!["create", "-q", file.as_str(), "src"];
                args.extend_from_slice(threads);
                ok(&s.pkw(&args));
                std::fs::read(s.path(&file)).expect("what was made")
            })
            .collect();
        assert!(made.windows(2).all(|w| w[0] == w[1]), "{name}");
        let file = format!("0-{name}");
        let in_one = ok(&s.pkw(&["list", "-l", "-T", "1", &file]));
        assert!(in_one.contains("src/docs/notes.txt"), "{in_one}");
        assert_eq!(in_one, ok(&s.pkw(&["list", "-l", "-T", "0", &file])));
    }
    // The app's setting, when none is given; one not understood is all.
    s.settings("threads=2\n");
    ok(&s.pkw(&["test", "-q", "0-a.tar.xz"]));
    s.settings("threads=lots\n");
    ok(&s.pkw(&["test", "-q", "0-a.tar.xz"]));
    assert_eq!(
        s.pkw(&["list", "-T", "some", "0-a.tar.xz"]).status.code(),
        Some(2)
    );
}

#[test]
fn a_level_too_large_for_the_memory_free_is_lowered_and_said() {
    let s = Scratch::new("memory");
    // 300 MB free: 7z's and xz's highest level take more than a GB in one
    // thread, which would have the system end pkw.
    let little = [("PACKWING_MEMORY", "300000000")];
    for name in ["a.7z", "a.tar.xz"] {
        let made = s.pkw_with(
            &["create", "-l", "highest", "-T", "0", name, "src"],
            &little,
        );
        let said = String::from_utf8_lossy(&made.stderr).into_owned();
        assert!(made.status.success(), "{name}: {said}");
        assert!(said.contains("it is made at"), "{name}: {said}");
        ok(&s.pkw(&["test", "-q", name]));
    }
    // With memory enough, nothing is said.
    let made = s.pkw(&["create", "-l", "highest", "b.7z", "src"]);
    assert!(made.status.success());
    assert!(
        made.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&made.stderr)
    );
    // Nor when quiet.
    let made = s.pkw_with(&["create", "-q", "-l", "highest", "c.7z", "src"], &little);
    assert!(made.status.success());
    assert!(made.stderr.is_empty());
}

/// Under `ulimit -v`, as under too little memory free: a level whose one
/// thread would take more address space than is left is lowered, and said,
/// where it would have been refused memory part way.
#[cfg(target_os = "linux")]
#[test]
fn under_ulimit_a_level_too_large_is_lowered_and_said() {
    let s = Scratch::new("ulimit");
    for name in ["a.7z", "a.tar.xz"] {
        let made = Command::new("sh")
            .arg("-c")
            .arg("ulimit -v 900000 && exec \"$0\" \"$@\"")
            .arg(env!("CARGO_BIN_EXE_pkw"))
            .args(["create", "-l", "highest", "-T", "0", name, "src"])
            .current_dir(&s.0)
            .env_remove("PACKWING_MEMORY")
            .env("XDG_CONFIG_HOME", s.path("config"))
            .stdin(std::process::Stdio::null())
            .output()
            .expect("sh runs");
        let said = String::from_utf8_lossy(&made.stderr).into_owned();
        assert!(made.status.success(), "{name}: {said}");
        assert!(said.contains("it is made at"), "{name}: {said}");
        ok(&s.pkw(&["test", "-q", name]));
    }
}

/// `--frames several` writes a zstd stream in frames, with zstd's seek table at
/// its end; one frame has none; and the option is for zstd alone.
#[test]
fn frames_are_chosen_for_zstd() {
    let s = Scratch::new("frames");
    let seek_table = |name: &str| {
        let made = std::fs::read(s.path(name)).expect(name);
        made.ends_with(&0x8F92_EAB1u32.to_le_bytes())
    };
    ok(&s.pkw(&["create", "-q", "--frames", "several", "a.tar.zst", "src"]));
    assert!(seek_table("a.tar.zst"));
    ok(&s.pkw(&["extract", "-q", "a.tar.zst", "-o", "a"]));
    assert!(s.path("a/src/numbers.txt").is_file());
    ok(&s.pkw(&["create", "-q", "--frames", "one", "b.tar.zst", "src"]));
    assert!(!seek_table("b.tar.zst"));
    // Small, it is one frame unless asked.
    ok(&s.pkw(&["create", "-q", "c.tar.zst", "src"]));
    assert!(!seek_table("c.tar.zst"));
    let usage = |o: Output| assert_eq!(o.status.code(), Some(2), "{o:?}");
    usage(s.pkw(&["create", "-q", "--frames", "several", "d.zip", "src"]));
    usage(s.pkw(&["create", "-q", "--frames", "two", "d.tar.zst", "src"]));
}

/// `--blocks small` cuts xz's and 7z's LZMA2 into more blocks than the
/// format's own, large by default, where there is less than enough of those
/// for every core; and the option is for them alone.
#[test]
fn blocks_are_chosen_for_xz_and_7z() {
    let s = Scratch::new("blocks");
    // At the lowest level, xz's own blocks are 3 MiB: one for this, three
    // small ones of a dictionary, 1 MiB.
    let big = s.path("big");
    std::fs::create_dir(&big).expect("big");
    let data: Vec<u8> = (0..(5u32 << 19))
        .flat_map(|i| (i % 251).to_le_bytes())
        .collect();
    std::fs::write(big.join("data.bin"), &data[..(5 << 19)]).expect("data");
    for (blocks, name) in [("small", "a.tar.xz"), ("large", "b.tar.xz")] {
        ok(&s.pkw(&[
            "create", "-q", "-l", "lowest", "--blocks", blocks, name, "big",
        ]));
        ok(&s.pkw(&["test", "-q", name]));
    }
    ok(&s.pkw(&["create", "-q", "-l", "lowest", "c.tar.xz", "big"]));
    let read = |name: &str| std::fs::read(s.path(name)).expect(name);
    assert!(
        read("a.tar.xz") != read("b.tar.xz"),
        "small blocks are other blocks"
    );
    assert!(read("c.tar.xz") == read("b.tar.xz"), "large by default");
    // 7z too, whose own blocks at the lowest level are 4 MiB, one dictionary
    // being 1 MiB.
    for (blocks, name) in [("small", "a.7z"), ("large", "b.7z")] {
        ok(&s.pkw(&[
            "create", "-q", "-l", "lowest", "--blocks", blocks, name, "big",
        ]));
        ok(&s.pkw(&["test", "-q", name]));
    }
    ok(&s.pkw(&["create", "-q", "-l", "lowest", "c.7z", "big"]));
    assert!(
        read("a.7z") != read("b.7z"),
        "7z: small blocks are other blocks"
    );
    assert!(read("c.7z") == read("b.7z"), "7z: large by default");
    let usage = |o: Output| assert_eq!(o.status.code(), Some(2), "{o:?}");
    usage(s.pkw(&["create", "-q", "--blocks", "small", "d.tar.zst", "big"]));
    usage(s.pkw(&["create", "-q", "--blocks", "tiny", "d.tar.xz", "big"]));
}

/// Ctrl-C (SIGINT) or SIGTERM while an archive is made stops the job, as the
/// app's Cancel does: pkw ends with 130, and nothing is left of the archive.
#[cfg(unix)]
#[test]
fn a_signal_stops_the_job_and_leaves_nothing() {
    let s = Scratch::new("ctrl-c");
    // Long enough to compress that the signal comes while it is made.
    let mut data = Vec::with_capacity(48 << 20);
    let mut state = 0x9e37_79b9_7f4a_7c15u64;
    while data.len() < 48 << 20 {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        data.extend_from_slice(&state.to_le_bytes()[..(state % 7) as usize]);
        data.extend_from_slice(b"packwing ");
    }
    std::fs::write(s.path("big.bin"), &data).expect("the data");
    for (signal, name) in [
        ("-INT", "a.tar.xz"),
        ("-TERM", "b.tar.zst"),
        ("-INT", "c.7z"),
    ] {
        let child = Command::new(env!("CARGO_BIN_EXE_pkw"))
            .args(["create", "-q", "-l", "highest", name, "big.bin"])
            .current_dir(&s.0)
            .env("XDG_CONFIG_HOME", s.path("config"))
            .stdin(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("pkw");
        std::thread::sleep(std::time::Duration::from_millis(500));
        let sent = Command::new("kill")
            .args([signal, &child.id().to_string()])
            .status()
            .expect("kill");
        assert!(sent.success());
        let out = child.wait_with_output().expect("pkw ends");
        let said = String::from_utf8_lossy(&out.stderr);
        assert_eq!(out.status.code(), Some(130), "{name}: {said}");
        let left: Vec<_> = std::fs::read_dir(&s.0)
            .expect("the folder")
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.contains(name))
            .collect();
        assert!(left.is_empty(), "{name}: left {left:?}");
    }
}
