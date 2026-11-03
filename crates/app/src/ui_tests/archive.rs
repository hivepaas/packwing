//! Opening archives, and what comes out of them.

use super::{Ui, fixture, listed, scenario};

scenario!(
    /// Each format packwing reads opens on the Archive page.
    each_format_opens,
    |ui| {
        for name in [
            "sample.7z",
            "rar5_multiple_files.rar",
            "sample.iso.zst",
            "sample-udf.iso.zst",
            "default.xar",
            "tiny.pkg",
            "test_rar_multivolume_multiple_files.part1.rar",
        ] {
            ui.open(&fixture(name));
            assert_eq!(ui.archive_failed(), None, "{name}");
            assert!(!ui.archive_names().is_empty(), "{name} shows nothing");
        }
    }
);

scenario!(
    /// A file that is no archive says so, and the window goes on working.
    what_is_no_archive_says_so,
    |ui| {
        let file = ui.file("photo.jpg", "not an archive");
        ui.open(&file);
        assert!(ui.archive_failed().is_some());
        let good = fixture("sample.7z");
        ui.open(&good);
        assert_eq!(ui.archive_failed(), None);
    }
);

scenario!(
    /// Extract asks where, and the files come out there.
    extract_writes_the_files,
    |ui| {
        let archive = ui.copy_of("sample.7z");
        let held = listed(&archive, None);
        ui.open(&archive);
        ui.click("extract");
        ui.wait_for("dest-new");
        ui.click("dest-new");
        ui.click("extract");
        ui.wait_for_no_dialog();
        let out = ui.work.join("sample");
        for name in held.iter().filter(|n| !n.ends_with('/')) {
            assert!(out.join(name).exists(), "{name} did not come out in {out:?}");
        }
    }
);

scenario!(
    /// An archive with a password asks for it, again when it is wrong, and opens with
    /// the right one.
    a_wrong_password_asks_again,
    |ui| {
        ui.open(&fixture("hidden.7z"));
        ui.wait_for("open");
        ui.type_in("The archive’s password", "wrong");
        ui.click("open");
        ui.wait_for("open");
        assert!(ui.archive_names().is_empty(), "it opened with the wrong password");
        ui.replace_in("The archive’s password", "secret");
        ui.click("open");
        ui.wait_for_no_dialog();
        assert!(!ui.archive_names().is_empty());
    }
);

scenario!(
    /// A click on a file shows it, and a click on Test checks the archive.
    test_passes,
    |ui| {
        ui.open(&fixture("sample.7z"));
        ui.click("test");
        ui.wait_for("close");
        ui.click("close");
        ui.wait_for_no_dialog();
    }
);

scenario!(
    /// A file's text in the preview can be chosen and copied, and not changed.
    the_preview_text_copies_and_stays,
    |ui| {
        let text = "xin chào\nthế giới\n";
        let made = packwing_core::create(
            &[ui.file("notes.txt", text)],
            &ui.work.join("notes.zip"),
            &packwing_core::NewArchive::default(),
            &packwing_core::Control::default(),
        )
        .expect("the archive");
        ui.open(&made);
        ui.click("entry-0");
        ui.wait_for("preview-text");
        ui.click("preview-text");
        let cmd = if cfg!(target_os = "macos") { "cmd" } else { "ctrl" };
        let copied = |ui: &mut Ui| {
            ui.cx.write_to_clipboard(gpui::ClipboardItem::new_string(String::new()));
            ui.keys(&format!("{cmd}-a {cmd}-c"));
            ui.cx.read_from_clipboard().and_then(|c| c.text())
        };
        assert_eq!(copied(ui).as_deref(), Some(text));
        // Typing, deleting and pasting at its end, past the selection.
        ui.keys("right");
        let window = ui.window();
        ui.cx.simulate_input(window.into(), "x");
        ui.keys(&format!("backspace backspace {cmd}-v"));
        assert_eq!(copied(ui).as_deref(), Some(text));
    }
);

scenario!(
    /// An age file asks for its passphrase and opens with it.
    an_age_file_opens_with_its_passphrase,
    |ui| {
        ui.open(&fixture("notes.tar.zst.age"));
        ui.wait_for("open");
        ui.type_in("The passphrase", "password");
        ui.click("open");
        ui.wait_for_no_dialog();
        assert_eq!(ui.archive_names(), ["docs"]);
    }
);

scenario!(
    /// The right-click menu's Extract here needs no dialog of its own choosing.
    extract_here_from_the_right_click_menu,
    |ui| {
        let archive = ui.copy_of("sample.7z");
        let held = listed(&archive, None);
        let request = crate::right_click::Request {
            action: crate::right_click::Action::ExtractHere,
            paths: vec![archive],
        };
        ui.ws(|ws, window, cx| ws.start(request, window, cx));
        ui.wait_for_no_dialog();
        let work = ui.work.clone();
        for name in held.iter().filter(|n| !n.ends_with('/')) {
            assert!(work.join(name).exists(), "{name} did not come out");
        }
    }
);

/// A ZIP archive of `a.txt`, `b.txt` and `docs/deep/needle.txt`, made by the core, and opened.
fn opened_tree(ui: &mut Ui) -> std::path::PathBuf {
    ui.file("docs/deep/needle.txt", "found");
    let paths = vec![
        ui.file("a.txt", "a"),
        ui.file("b.txt", "b"),
        ui.work.join("docs"),
    ];
    let made = packwing_core::create(
        &paths,
        &ui.work.join("tree.zip"),
        &packwing_core::NewArchive::default(),
        &packwing_core::Control::default(),
    )
    .expect("the archive");
    ui.open(&made);
    assert_eq!(ui.archive_failed(), None);
    made
}

scenario!(
    /// The search finds what is deep in the archive, and a click on what it found goes
    /// to its folder and chooses it.
    search_finds_and_goes_to_it,
    |ui| {
        opened_tree(ui);
        assert_eq!(ui.archive_folder(), Vec::<String>::new());
        ui.type_in("Search inside this archive", "NEEDLE");
        ui.wait_until("the search", |ui| ui.shown_in_archive() == ["needle.txt"]);
        ui.click("entry-0");
        assert_eq!(ui.archive_folder(), ["docs", "deep"]);
        assert_eq!(ui.shown_in_archive(), ["needle.txt"]);
        ui.wait_for("preview-text");
    }
);

scenario!(
    /// With a file chosen, Extract takes out that file alone; asked for all of it, the rest.
    extract_takes_out_what_is_picked,
    |ui| {
        opened_tree(ui);
        let a = ui.shown_in_archive().iter().position(|n| n == "a.txt").expect("a.txt");
        ui.click(&format!("entry-{a}"));
        ui.click("extract");
        ui.wait_for("only-picked");
        ui.click("dest-new");
        ui.click("extract");
        ui.wait_for_no_dialog();
        let out = ui.work.join("tree");
        let came = |out: &std::path::Path| {
            let mut names: Vec<_> = std::fs::read_dir(out)
                .expect("out")
                .flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect();
            names.sort();
            names
        };
        assert_eq!(came(&out), ["a.txt"]);
        std::fs::remove_dir_all(&out).expect("out");
        ui.click("extract");
        ui.wait_for("all-of-it");
        ui.click("all-of-it");
        ui.click("dest-new");
        ui.click("extract");
        ui.wait_for_no_dialog();
        assert_eq!(came(&out), ["a.txt", "b.txt", "docs"]);
        assert!(out.join("docs/deep/needle.txt").exists());
    }
);

scenario!(
    /// Extract to a folder, from the right-click menu, puts the files in a folder named
    /// after the archive.
    extract_to_a_folder_from_the_right_click_menu,
    |ui| {
        let archive = ui.copy_of("sample.7z");
        let held = listed(&archive, None);
        let request = crate::right_click::Request {
            action: crate::right_click::Action::ExtractToFolder,
            paths: vec![archive],
        };
        ui.ws(|ws, window, cx| ws.start(request, window, cx));
        ui.wait_for_no_dialog();
        let out = ui.work.join("sample");
        for name in held.iter().filter(|n| !n.ends_with('/')) {
            assert!(out.join(name).exists(), "{name} did not come out in {out:?}");
        }
    }
);
