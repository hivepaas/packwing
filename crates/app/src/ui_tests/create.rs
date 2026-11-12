//! Making archives through New archive, and from the right-click menu.

use super::{Ui, listed, scenario};

/// New archive open on the one file `a.txt`, with Advanced shown.
fn new_archive(ui: &mut Ui) -> std::path::PathBuf {
    let file = ui.file("a.txt", "held");
    let paths = vec![file];
    ui.ws(|ws, window, cx| ws.open_create(paths, window, cx));
    ui.wait_for("create");
    ui.work.clone()
}

scenario!(
    /// A name Windows can't use is said, and the archive made all the same.
    a_name_windows_cannot_use_is_said,
    |ui| {
        let work = new_archive(ui);
        assert!(!ui.shows("not-portable"));
        ui.click("cancel");
        if cfg!(windows) {
            return;
        }
        let file = ui.file("a:b.txt", "colon");
        ui.ws(|ws, window, cx| ws.open_create(vec![file], window, cx));
        ui.wait_for("not-portable");
        ui.click("create");
        ui.wait_for_no_dialog();
        assert_eq!(listed(&work.join("a:b.zip"), None), ["a:b.txt"]);
    }
);

scenario!(
    /// The dialog's defaults make a ZIP archive beside the file.
    a_zip_archive,
    |ui| {
        let work = new_archive(ui);
        ui.click("create");
        ui.wait_for_no_dialog();
        assert_eq!(listed(&work.join("a.zip"), None), ["a.txt"]);
    }
);

scenario!(
    /// A name taken in the folder is said before: the new archive gets the next one.
    a_name_taken_is_said_before,
    |ui| {
        let work = new_archive(ui);
        assert!(!ui.shows("renamed"));
        ui.click("cancel");
        assert!(!ui.dialog_open());
        std::fs::write(work.join("a.zip"), "there").expect("the file");
        new_archive(ui);
        assert!(ui.shows("renamed"));
        ui.click("create");
        ui.wait_for_no_dialog();
        assert_eq!(std::fs::read(work.join("a.zip")).expect("kept"), b"there");
        assert_eq!(listed(&work.join("a (2).zip"), None), ["a.txt"]);
    }
);

scenario!(
    /// With a password, the archive opens with it and not without.
    a_zip_archive_with_a_password,
    |ui| {
        let work = new_archive(ui);
        ui.click("advanced");
        ui.click("protect");
        ui.type_in("Password", "correct horse battery");
        ui.click("create");
        ui.wait_for_no_dialog();
        let made = work.join("a.zip");
        assert_eq!(listed(&made, Some("correct horse battery")), ["a.txt"]);
        let out = work.join("out");
        let keys = packwing_core::Keys::default();
        let control = packwing_core::Control::default();
        assert!(packwing_core::extract(&made, &out, &keys, &control).is_err());
    }
);

scenario!(
    /// A ZIP archive encrypted whole with age instead is saved as .zip.age, and opens
    /// with the passphrase alone.
    a_zip_archive_encrypted_with_age,
    |ui| {
        let work = new_archive(ui);
        ui.click("advanced");
        ui.click("protect");
        ui.click("whole-age");
        ui.type_in("Password", "correct horse battery");
        ui.click("create");
        ui.wait_for_no_dialog();
        assert!(!work.join("a.zip").exists());
        let made = work.join("a.zip.age");
        let control = packwing_core::Control::default();
        let none = packwing_core::Keys::default();
        assert!(packwing_core::list(packwing_core::Source::from(made.as_path()), &none).is_err());
        let keys = packwing_core::Keys {
            passphrase: Some(b"correct horse battery".to_vec()),
            ..packwing_core::Keys::default()
        };
        let out = work.join("out");
        packwing_core::extract(&made, &out, &keys, &control).expect("it opens with the passphrase");
        assert_eq!(std::fs::read_to_string(out.join("a.txt")).expect("a.txt came out"), "held");
    }
);

/// The ZIP archive New archive made in the work folder, and its size.
fn zip_made(ui: &Ui) -> (std::path::PathBuf, u64) {
    let made = std::fs::read_dir(&ui.work)
        .expect("the work folder")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .find(|p| p.extension().is_some_and(|e| e == "zip"))
        .expect("a ZIP archive");
    let size = std::fs::metadata(&made).expect("its size").len();
    (made, size)
}

/// What compresses to almost nothing.
const SQUEEZABLE: usize = 200_000;

scenario!(
    /// Store puts the files in as they are.
    store_puts_the_files_in_as_they_are,
    |ui| {
        let paths = vec![ui.file("words.txt", &"x".repeat(SQUEEZABLE))];
        ui.ws(|ws, window, cx| ws.open_create(paths, window, cx));
        ui.wait_for("create");
        ui.click("advanced");
        ui.click("Store");
        ui.click("create");
        ui.wait_for_no_dialog();
        let (made, size) = zip_made(ui);
        assert!(size > SQUEEZABLE as u64, "{size}");
        assert_eq!(listed(&made, None), ["words.txt"]);
    }
);

scenario!(
    /// By default, a picture goes in as it is while text is compressed.
    what_is_compressed_already_goes_in_as_it_is,
    |ui| {
        let text = "x".repeat(SQUEEZABLE);
        let paths = vec![ui.file("photo.jpg", &text), ui.file("notes.txt", &text)];
        ui.ws(|ws, window, cx| ws.open_create(paths, window, cx));
        ui.wait_for("create");
        ui.click("create");
        ui.wait_for_no_dialog();
        let (_, size) = zip_made(ui);
        // The picture's bytes, and little more for the text.
        assert!(size > SQUEEZABLE as u64 && size < SQUEEZABLE as u64 * 11 / 10, "{size}");
    }
);

scenario!(
    /// Asked to protect it, an empty password stops it, and says so.
    no_password_makes_nothing,
    |ui| {
        let work = new_archive(ui);
        ui.click("advanced");
        ui.click("protect");
        ui.click("create");
        ui.cx.run_until_parked();
        assert!(ui.dialog_open());
        assert!(!work.join("a.zip").exists());
    }
);

scenario!(
    /// Suggest fills the field with a passphrase that makes an archive.
    a_suggested_passphrase_works,
    |ui| {
        let work = new_archive(ui);
        ui.click("advanced");
        ui.click("protect");
        ui.click("suggest-password");
        ui.click("create");
        ui.wait_for_no_dialog();
        assert!(work.join("a.zip").exists());
    }
);

scenario!(
    /// Compress to ZIP, from the right-click menu, asks nothing.
    compress_from_the_right_click_menu,
    |ui| {
        let file = ui.file("a.txt", "held");
        let request = crate::right_click::Request {
            action: crate::right_click::Action::CompressZip,
            paths: vec![file],
        };
        ui.ws(|ws, window, cx| ws.start(request, window, cx));
        ui.wait_for_no_dialog();
        assert_eq!(listed(&ui.work.join("a.zip"), None), ["a.txt"]);
    }
);

scenario!(
    /// A file dropped on the window starts a new archive of it.
    a_drop_starts_a_new_archive,
    |ui| {
        let a = ui.file("a.txt", "a");
        ui.ws(|ws, window, cx| ws.dropped(vec![a], window, cx));
        ui.wait_for("create");
        let held = ui.ws(|ws, _, cx| ws.create_dialog_paths(cx));
        assert_eq!(held.len(), 1, "{held:?}");
        ui.click("create");
        ui.wait_for_no_dialog();
        assert_eq!(listed(&ui.work.join("a.zip"), None), ["a.txt"]);
    }
);

/// The file New archive made in the work folder whose name ends with `ext`.
fn made(ui: &Ui, ext: &str) -> std::path::PathBuf {
    std::fs::read_dir(&ui.work)
        .expect("the work folder")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .find(|p| p.to_string_lossy().ends_with(ext))
        .unwrap_or_else(|| panic!("no {ext} was made"))
}

scenario!(
    /// Each format offered for a file and a folder, chosen from its menu, makes an
    /// archive that holds them both.
    each_format_makes_an_archive,
    |ui| {
        ui.file("a.txt", "a");
        ui.file("docs/b.txt", "b");
        for ext in [
            ".7z", ".tar.zst", ".tar.gz", ".tar.xz", ".tar.bz2", ".tar.lz4", ".tar", ".zip",
        ] {
            let paths = vec![ui.work.join("a.txt"), ui.work.join("docs")];
            ui.ws(|ws, window, cx| ws.open_create(paths, window, cx));
            ui.wait_for("create");
            ui.choose("format", ext);
            ui.click("create");
            ui.wait_for_no_dialog();
            let made = made(ui, ext);
            let held = listed(&made, None);
            for name in ["a.txt", "docs/b.txt"] {
                assert!(held.iter().any(|n| n == name), "{ext}: {held:?}");
            }
            std::fs::remove_file(&made).expect("the archive");
        }
    }
);

scenario!(
    /// One file by itself can be compressed alone, with no archive around it.
    one_file_is_compressed_alone,
    |ui| {
        let text = "x".repeat(SQUEEZABLE);
        let paths = vec![ui.file("words.txt", &text)];
        ui.ws(|ws, window, cx| ws.open_create(paths, window, cx));
        ui.wait_for("create");
        ui.choose("format", ".zst");
        ui.click("create");
        ui.wait_for_no_dialog();
        let made = made(ui, ".zst");
        assert!(std::fs::metadata(&made).expect("its size").len() < 1000);
        let out = ui.work.join("out");
        let (keys, control) = (packwing_core::Keys::default(), packwing_core::Control::default());
        packwing_core::extract(&made, &out, &keys, &control).expect("it comes out");
        let came: Vec<_> = std::fs::read_dir(&out).expect("out").flatten().collect();
        assert_eq!(came.len(), 1, "{came:?}");
        assert_eq!(std::fs::read_to_string(came[0].path()).expect("the file"), text);
    }
);

scenario!(
    /// A 7z archive with a password hides the names in it too: nothing is listed
    /// without the password.
    a_7z_archive_hides_its_names,
    |ui| {
        let work = new_archive(ui);
        ui.choose("format", ".7z");
        ui.click("advanced");
        ui.click("protect");
        assert!(ui.shows("hide-names"));
        ui.type_in("Password", "correct horse battery");
        ui.click("create");
        ui.wait_for_no_dialog();
        let made = work.join("a.7z");
        let none = packwing_core::Keys::default();
        assert!(packwing_core::list(packwing_core::Source::from(made.as_path()), &none).is_err());
        assert_eq!(listed(&made, Some("correct horse battery")), ["a.txt"]);
    }
);

scenario!(
    /// A tar.zst archive encrypted to a public key opens with its key file alone.
    an_archive_is_encrypted_to_a_public_key,
    |ui| {
        let key = std::fs::read(crate::ui_tests::fixture("test-age.key")).expect("the key file");
        let public = String::from_utf8_lossy(&key)
            .lines()
            .find_map(|l| l.strip_prefix("# public key: ").map(str::to_owned))
            .expect("its public key");
        let work = new_archive(ui);
        ui.choose("format", ".tar.zst");
        ui.click("advanced");
        ui.click("protect");
        ui.click("age-key");
        ui.type_in("age1…, age1pq1… or ssh-ed25519 …", &public);
        ui.click("create");
        ui.wait_for_no_dialog();
        let made = work.join("a.tar.zst.age");
        let none = packwing_core::Keys::default();
        assert!(packwing_core::list(packwing_core::Source::from(made.as_path()), &none).is_err());
        let keys = packwing_core::Keys {
            key_file: Some(key),
            ..packwing_core::Keys::default()
        };
        let held: Vec<_> = packwing_core::list(packwing_core::Source::from(made.as_path()), &keys)
            .expect("it opens with the key file")
            .entries
            .iter()
            .map(|e| e.path.join("/"))
            .collect();
        assert_eq!(held, ["a.txt"]);
    }
);

scenario!(
    /// Split at a size typed in, an archive is cut into parts of that size, and they
    /// read back as one.
    an_archive_is_split_into_parts,
    |ui| {
        let paths = vec![ui.file("big.txt", &"x".repeat(2_500_000))];
        ui.ws(|ws, window, cx| ws.open_create(paths, window, cx));
        ui.wait_for("create");
        ui.click("advanced");
        ui.click("Store");
        ui.choose("split", "Custom size…");
        ui.type_in("Size", "1");
        ui.click("create");
        ui.wait_for_no_dialog();
        let part = |n: u32| ui.work.join(format!("big.zip.{n:03}"));
        for n in 1..=3 {
            let size = std::fs::metadata(part(n)).expect("the part").len();
            assert!(n == 3 || size == 1_000_000, "part {n}: {size}");
        }
        assert!(!part(4).exists());
        assert_eq!(listed(&part(1), None), ["big.txt"]);
    }
);

scenario!(
    /// With Store what is compressed turned off in Settings, a picture is compressed
    /// like the rest.
    a_picture_is_compressed_when_asked,
    |ui| {
        ui.settings(2);
        ui.click("store-compressed");
        assert!(!ui.ws(|_, _, cx| crate::prefs::prefs(cx).store_compressed));
        let paths = vec![ui.file("photo.jpg", &"x".repeat(SQUEEZABLE))];
        ui.ws(|ws, window, cx| ws.open_create(paths, window, cx));
        ui.wait_for("create");
        ui.click("create");
        ui.wait_for_no_dialog();
        let (_, size) = zip_made(ui);
        assert!(size < SQUEEZABLE as u64 / 10, "{size}");
    }
);

scenario!(
    /// The format chosen in Settings is the one New archive starts with, after a restart too.
    the_format_in_settings_is_the_first_offered,
    |ui| {
        ui.settings(2);
        ui.choose("format", ".7z");
        ui.start();
        let work = new_archive(ui);
        ui.click("create");
        ui.wait_for_no_dialog();
        assert_eq!(listed(&work.join("a.7z"), None), ["a.txt"]);
    }
);

scenario!(
    /// New archive offers to save on a storage only once the user has one.
    a_storage_is_offered_once_there_is_one,
    |ui| {
        new_archive(ui);
        assert!(ui.shows("change-folder"));
        assert!(!ui.shows("change-storage"), "offered with no storage of the user's");
        ui.click("cancel");
        ui.wait_for_no_dialog();
        ui.storage(&ui.work.join("place"));
        new_archive(ui);
        ui.click("change-storage");
        ui.wait_for("save-here");
    }
);

scenario!(
    /// A level that takes more memory than is free, even in one thread, is
    /// said under the levels; the archive is made all the same.
    a_level_that_takes_more_memory_than_is_free_is_said,
    |ui| {
        // Medium's 7z, of 7-Zip's dictionary of 32 MiB, takes about 500 MB
        // in one thread; Highest's, of 256 MiB, about 3 GB.
        packwing_core::memory::set_available(1_000_000_000);
        let work = new_archive(ui);
        ui.click("advanced");
        ui.choose("format", ".7z");
        ui.click("Medium");
        assert!(!ui.shows("create-memory"));
        ui.click("Highest");
        ui.wait_for("create-memory");
        ui.click("create");
        ui.wait_for_no_dialog();
        packwing_core::memory::set_available(0);
        assert_eq!(listed(&work.join("a.7z"), None), ["a.txt"]);
    }
);

scenario!(
    /// A .tar.zst of more than a frame offers frames: one for under 256 MiB, as
    /// the size says, and several when chosen, which end in a seek table.
    zstd_frames_are_offered_and_chosen,
    |ui| {
        let small = vec![ui.file("a.txt", "a")];
        ui.ws(|ws, window, cx| ws.open_create(small, window, cx));
        ui.wait_for("create");
        ui.click("advanced");
        ui.choose("format", ".tar.zst");
        assert!(!ui.shows("frames"), "a file of a byte has one frame");
        ui.click("cancel");

        let big = ui.work.join("big.bin");
        std::fs::write(&big, vec![7u8; 40 << 20]).expect("the big file");
        ui.ws(|ws, window, cx| ws.open_create(vec![big], window, cx));
        ui.wait_for("create");
        ui.click("advanced");
        ui.choose("format", ".tar.zst");
        // Once the files are counted, it is shown.
        ui.wait_for("frames");
        ui.click("frames-several");
        ui.click("create");
        ui.wait_for_no_dialog();
        let made = made(ui, ".tar.zst");
        let bytes = std::fs::read(&made).expect("the archive");
        assert!(bytes.ends_with(&0x8F92_EAB1u32.to_le_bytes()), "a seek table");
        assert_eq!(listed(&made, None), ["big.bin"]);
    }
);

scenario!(
    /// A .tar.xz offers its blocks where small ones would be more than xz's
    /// own: large by default, small when chosen.
    xz_blocks_are_offered_and_chosen,
    |ui| {
        let small = vec![ui.file("a.txt", "a")];
        ui.ws(|ws, window, cx| ws.open_create(small, window, cx));
        ui.wait_for("create");
        ui.click("advanced");
        ui.choose("format", ".tar.xz");
        assert!(!ui.shows("blocks"), "a file of a byte is one block");
        ui.click("cancel");

        // 40 MiB at Medium: two of xz's own blocks of 24 MiB, five small ones.
        let big = ui.work.join("big.bin");
        std::fs::write(&big, vec![7u8; 40 << 20]).expect("the big file");
        ui.ws(|ws, window, cx| ws.open_create(vec![big], window, cx));
        ui.wait_for("create");
        ui.click("advanced");
        ui.choose("format", ".tar.xz");
        ui.wait_for("blocks");
        ui.click("blocks-small");
        ui.click("create");
        ui.wait_for_no_dialog();
        let made = made(ui, ".tar.xz");
        assert_eq!(listed(&made, None), ["big.bin"]);
        // The archive the core makes in small blocks, not in large.
        let of = |blocks| {
            let to = ui.work.join(format!("{blocks:?}.tar.xz"));
            let new = packwing_core::NewArchive {
                kind: packwing_core::Kind::TarXz,
                blocks,
                ..packwing_core::NewArchive::default()
            };
            let control = packwing_core::Control::default();
            packwing_core::create(&[ui.work.join("big.bin")], &to, &new, &control).expect("made");
            std::fs::read(to).expect("read")
        };
        let bytes = std::fs::read(&made).expect("the archive");
        assert!(bytes == of(packwing_core::Blocks::Small), "in small blocks");
        assert!(bytes != of(packwing_core::Blocks::Large));
    }
);

scenario!(
    /// A 7z archive offers its blocks as xz does: large by default, small
    /// when chosen.
    seven_zip_blocks_are_offered_and_chosen,
    |ui| {
        // 40 MiB at Medium: two of 7-Zip's own blocks of 32 MiB, five small
        // ones of a dictionary, 8 MiB.
        let big = ui.work.join("big.bin");
        std::fs::write(&big, vec![7u8; 40 << 20]).expect("the big file");
        ui.ws(|ws, window, cx| ws.open_create(vec![big], window, cx));
        ui.wait_for("create");
        ui.click("advanced");
        ui.choose("format", ".7z");
        ui.wait_for("blocks");
        ui.click("blocks-small");
        ui.click("create");
        ui.wait_for_no_dialog();
        let made = made(ui, ".7z");
        assert_eq!(listed(&made, None), ["big.bin"]);
        let of = |blocks| {
            let to = ui.work.join(format!("{blocks:?}.7z"));
            let new = packwing_core::NewArchive {
                kind: packwing_core::Kind::SevenZip,
                blocks,
                ..packwing_core::NewArchive::default()
            };
            let control = packwing_core::Control::default();
            packwing_core::create(&[ui.work.join("big.bin")], &to, &new, &control).expect("made");
            std::fs::read(to).expect("read")
        };
        let bytes = std::fs::read(&made).expect("the archive");
        assert!(bytes == of(packwing_core::Blocks::Small), "in small blocks");
        assert!(bytes != of(packwing_core::Blocks::Large));
    }
);

#[cfg(unix)]
scenario!(
    /// A tar archive keeps the owners, and the extended attributes, when asked
    /// in Keep for a backup; ZIP suggests tar.
    a_tar_archive_keeps_owners_and_attributes_for_a_backup,
    |ui| {
        use std::os::unix::fs::MetadataExt;
        let work = new_archive(ui);
        let file = work.join("a.txt");
        let attributes = cfg!(any(target_os = "linux", target_os = "macos"))
            && xattr::set(&file, "user.packwing", b"kept").is_ok();
        ui.click("advanced");
        ui.wait_for("backup");
        assert!(!ui.shows("keep-owners"), "ZIP has no place for owners");
        ui.choose("format", ".tar");
        ui.click("keep-owners");
        if attributes {
            ui.click("keep-xattrs");
        }
        ui.click("create");
        ui.wait_for_no_dialog();
        let made = std::fs::File::open(work.join("a.tar")).expect("the archive");
        let mut reader = hivepack::tar::Reader::new(made);
        let header = reader.next().expect("a header").expect("a.txt");
        assert_eq!(header.name, b"a.txt");
        let mine = std::fs::metadata(&file).expect("the file");
        assert_eq!((header.uid, header.gid), (i64::from(mine.uid()), i64::from(mine.gid())));
        if attributes {
            assert_eq!(
                header.xattrs.get(&b"user.packwing"[..]).map(Vec::as_slice),
                Some(&b"kept"[..])
            );
        }
    }
);

scenario!(
    /// Patterns typed and added from the usual ones leave files out of it.
    patterns_leave_files_out,
    |ui| {
        let work = ui.work.clone();
        let proj = work.join("proj");
        for name in ["a.txt", "b.log", "node_modules/pkg/x.js", "docs/c.pdf"] {
            let path = proj.join(name);
            std::fs::create_dir_all(path.parent().expect("a folder")).expect("a folder");
            std::fs::write(path, "x").expect("a file");
        }
        ui.ws(|ws, window, cx| ws.open_create(vec![proj], window, cx));
        ui.wait_for("create");
        ui.click("edit-patterns");
        ui.type_in("node_modules/, *.log, src/**/gen/", "*.LOG");
        ui.keys("enter");
        assert!(ui.shows("pattern: *.LOG"));
        ui.click("presets");
        ui.click("preset-builds");
        assert!(ui.shows("pattern: node_modules/"));
        // Backspace in the empty field takes the last back.
        ui.type_in("node_modules/, *.log, src/**/gen/", "*.tmp");
        ui.keys("enter");
        assert!(ui.shows("pattern: *.tmp"));
        ui.keys("backspace");
        assert!(!ui.shows("pattern: *.tmp"));
        // Counted: what they leave out is said.
        ui.wait_for("left-out");
        ui.click("create");
        ui.wait_for_no_dialog();
        let mut files: Vec<String> = listed(&work.join("proj.zip"), None)
            .into_iter()
            .filter(|n| n.contains('.'))
            .collect();
        files.sort();
        assert_eq!(files, ["proj/a.txt", "proj/docs/c.pdf"]);
    }
);
