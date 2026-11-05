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
    /// The archive open has Checksums… in its `…` menu, for its file.
    an_archive_s_checksums_are_shown,
    |ui| {
        let archive = ui.copy_of("sample.7z");
        ui.open(&archive);
        ui.ws(|ws, _, cx| {
            ws.archive_page().update(cx, |p, cx| {
                p.open_more_for_shot(gpui::point(gpui::px(600.), gpui::px(200.)), cx)
            });
        });
        ui.click("menu-checksums");
        ui.wait_for("checksum-copy-0");
        ui.click("checksum-save");
        ui.wait_for("checksum-saved");
        assert!(ui.work.join("sample.7z.sha256").exists());
    }
);

scenario!(
    /// The archive open, from this computer, is shown in the system's file manager.
    an_archive_is_shown_in_the_file_manager,
    |ui| {
        let archive = ui.copy_of("sample.7z");
        ui.open(&archive);
        ui.ws(|ws, _, cx| {
            ws.archive_page().update(cx, |p, cx| {
                p.open_more_for_shot(gpui::point(gpui::px(600.), gpui::px(200.)), cx)
            });
        });
        ui.click("menu-file-manager");
        assert!(!ui.shows("archive-menu"));
        assert_eq!(crate::file_manager::asked_for_test(), [("show", archive)]);
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
    /// A file whose data is damaged stops the extraction to ask: left out, the rest
    /// come out, and the dialog says which it was.
    a_damaged_file_is_asked_about,
    |ui| {
        let files = [
            ui.file("in/a.txt", &"a".repeat(2000)),
            ui.file("in/b.txt", &"b".repeat(2000)),
            ui.file("in/c.txt", &"c".repeat(2000)),
        ];
        let new = packwing_core::NewArchive {
            level: packwing_core::Level::Store,
            ..packwing_core::NewArchive::default()
        };
        let made = packwing_core::create(
            &files,
            &ui.work.join("damaged.zip"),
            &new,
            &packwing_core::Control::default(),
        )
        .expect("the archive");
        let mut bytes = std::fs::read(&made).expect("its bytes");
        let at = bytes
            .windows(100)
            .position(|w| w.iter().all(|&b| b == b'b'))
            .expect("b.txt's data");
        bytes[at + 50] = b'x';
        std::fs::write(&made, bytes).expect("damaged");
        ui.open(&made);
        ui.click("extract");
        ui.wait_for("dest-new");
        ui.click("dest-new");
        ui.click("extract");
        ui.wait_for("skip");
        ui.click("skip");
        ui.wait_for("copy-list");
        let out = ui.work.join("damaged");
        assert!(out.join("a.txt").exists() && out.join("c.txt").exists());
        assert!(!out.join("b.txt").exists());
        ui.click("close");
        ui.wait_for_no_dialog();
    }
);

scenario!(
    /// With Settings' "Only names every system can use", a name Windows can't use
    /// is asked about before anything is written, and renamed when asked.
    a_name_not_every_system_takes_is_asked_about,
    |ui| {
        ui.settings(1);
        ui.click("portable-names");
        assert!(ui.ws(|_, _, cx| crate::prefs::prefs(cx).portable_names));
        let name = if cfg!(windows) { "in/a.txt" } else { "in/a:b.txt" };
        let files = [ui.file(name, "colon"), ui.file("in/ok.txt", "ok")];
        let made = packwing_core::create(
            &files,
            &ui.work.join("names.zip"),
            &packwing_core::NewArchive::default(),
            &packwing_core::Control::default(),
        )
        .expect("the archive");
        if cfg!(windows) {
            return;
        }
        ui.open(&made);
        ui.click("extract");
        ui.wait_for("dest-new");
        ui.click("dest-new");
        ui.click("extract");
        ui.wait_for("rename");
        let out = ui.work.join("names");
        assert!(!out.exists(), "written before it was answered");
        ui.click("rename");
        ui.wait_for("copy-list");
        assert!(out.join("a_b.txt").exists() && out.join("ok.txt").exists());
        ui.click("close");
        ui.wait_for_no_dialog();
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
    /// A right click on text to read offers Copy all, and Copy once some is chosen.
    the_preview_text_has_a_copy_menu,
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
        let clipboard = |ui: &mut Ui| ui.cx.read_from_clipboard().and_then(|c| c.text());
        ui.cx
            .write_to_clipboard(gpui::ClipboardItem::new_string(String::new()));
        ui.right_click("preview-text");
        assert!(ui.shows("text-copy-all"));
        assert!(!ui.shows("text-copy"), "nothing is chosen to copy");
        ui.click("text-copy-all");
        assert!(!ui.shows("text-menu"));
        assert_eq!(clipboard(ui).as_deref(), Some(text));
        // What is chosen, its first word, is what Copy takes.
        let cmd = if cfg!(target_os = "macos") { "cmd" } else { "ctrl" };
        ui.keys(&format!("{cmd}-a left shift-right shift-right shift-right"));
        ui.right_click("preview-text");
        ui.click("text-copy");
        assert_eq!(clipboard(ui).as_deref(), Some("xin"));
        // Escape closes it, and the text stays as it was.
        ui.right_click("preview-text");
        ui.keys("escape");
        assert!(!ui.shows("text-menu"));
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

scenario!(
    /// The archive open, from this computer, has Convert…, Split into parts… and,
    /// for a part, Join the parts in its `…` menu: converted, it holds what it held.
    the_archive_menu_converts_splits_and_joins,
    |ui| {
        let archive = ui.copy_of("sample.7z");
        let held = listed(&archive, None);
        ui.open(&archive);
        let more = |ui: &mut Ui| {
            ui.ws(|ws, _, cx| {
                ws.archive_page().update(cx, |p, cx| {
                    p.open_more_for_shot(gpui::point(gpui::px(600.), gpui::px(200.)), cx)
                });
            });
        };
        more(ui);
        assert!(ui.shows("menu-split"));
        assert!(!ui.shows("menu-join"));
        ui.click("menu-convert");
        ui.wait_for("create");
        ui.choose("format", ".zip");
        ui.click("create");
        ui.wait_for_no_dialog();
        assert_eq!(listed(&ui.work.join("sample.zip"), None), held);

        more(ui);
        ui.click("menu-split");
        ui.wait_for("split");
        ui.click("split");
        ui.wait_for_no_dialog();
        let first = ui.work.join("sample.7z.001");
        assert!(first.exists());
        ui.open(&first);
        more(ui);
        ui.click("menu-join");
        ui.wait_for_no_dialog();
        assert_eq!(
            std::fs::read(ui.work.join("sample (2).7z")).expect("joined"),
            std::fs::read(&archive).expect("the archive")
        );
    }
);

/// Whether the archive open keeps a secret.
fn keeps_keys(ui: &mut Ui) -> bool {
    ui.ws(|ws, _, cx| {
        let page = ws.archive_page();
        let page = page.read(cx);
        page.archive()
            .is_some_and(|a| a.keys != packwing_core::Keys::default())
    })
}

scenario!(
    /// A ZIP archive whose files alone have a password opens without it; once
    /// it is typed for one thing (here, Test), the preview, Extract and Test
    /// need it no more, for ten minutes after it was last used.
    the_password_is_kept_for_a_while,
    |ui| {
        let new = packwing_core::NewArchive {
            password: Some(b"secret".to_vec()),
            ..packwing_core::NewArchive::default()
        };
        let made = packwing_core::create(
            &[ui.file("notes.txt", "xin chào\n")],
            &ui.work.join("locked.zip"),
            &new,
            &packwing_core::Control::default(),
        )
        .expect("the archive");
        ui.open(&made);
        assert!(!ui.dialog_open(), "its names are not encrypted");
        assert!(!keeps_keys(ui));
        ui.click("entry-0");
        assert!(!ui.shows("preview-text"), "the preview needs the password");
        // Test asks for it, and goes through with it.
        ui.click("test");
        ui.wait_for("field: The archive’s password");
        ui.type_in("The archive’s password", "secret");
        ui.click("open");
        ui.wait_for("close");
        ui.click("close");
        ui.wait_for_no_dialog();
        assert!(keeps_keys(ui));
        // The preview shows it now, and Extract asks for no password.
        ui.click("entry-0");
        ui.wait_for("preview-text");
        ui.ws(|ws, window, cx| ws.open_extract(window, cx));
        ui.wait_for("extract");
        assert!(!ui.shows("field: The archive’s password"));
        ui.keys("escape");
        ui.wait_for_no_dialog();
        // A use keeps it from then: nine minutes, a use, nine more, kept.
        let minutes = |n: u64| std::time::Duration::from_secs(60 * n);
        ui.cx.executor().advance_clock(minutes(9));
        ui.cx.run_until_parked();
        assert!(keeps_keys(ui));
        ui.click("test");
        ui.wait_for("close");
        ui.click("close");
        ui.wait_for_no_dialog();
        ui.cx.executor().advance_clock(minutes(9));
        ui.cx.run_until_parked();
        assert!(keeps_keys(ui), "the test used it nine minutes ago");
        // Ten minutes unused, it is forgotten, and asked for again.
        ui.cx.executor().advance_clock(minutes(2));
        ui.cx.run_until_parked();
        assert!(!keeps_keys(ui));
        ui.click("test");
        ui.wait_for("field: The archive’s password");
    }
);

/// An entry found by a reading, as packwing-core tells it.
fn found(path: &str, dir: bool) -> packwing_core::Entry {
    packwing_core::Entry {
        path: path.split('/').map(str::to_owned).collect(),
        dir,
        size: if dir { 0 } else { 1 },
        modified: None,
    }
}

scenario!(
    /// An archive being read shows at once, what is found in it as it is,
    /// with how far in the reading is; what needs it all waits, and what
    /// is selected stays so as more comes, and when it is read through.
    an_archive_shows_as_it_is_read,
    |ui| {
        let paths = vec![ui.file("a.txt", "a"), ui.file("c.txt", "c")];
        let zip = packwing_core::create(
            &paths,
            &ui.work.join("two.zip"),
            &packwing_core::NewArchive::default(),
            &packwing_core::Control::default(),
        )
        .expect("the archive");
        let source = packwing_core::Source::from(zip.as_path());
        let heard = crate::pages::archive::Heard::default();
        let (from, told) = (source.clone(), heard.clone());
        ui.ws(move |ws, _, cx| {
            ws.archive_page().update(cx, |p, cx| {
                p.start_reading(from, packwing_core::Keys::default(), told, cx)
            });
            ws.show_page(crate::workspace::Page::Archive, cx);
        });
        let reading = |ui: &mut Ui| ui.ws(|ws, _, cx| ws.archive_page().read(cx).is_reading());
        assert!(reading(ui));
        assert!(ui.shows("archive-reading"));
        assert!(ui.shown_in_archive().is_empty());

        let hear = |ui: &mut Ui, more: Vec<packwing_core::Entry>, read: u32| {
            heard.found.lock().expect("not poisoned").extend(more);
            heard.read.store(read, std::sync::atomic::Ordering::Relaxed);
            ui.cx
                .executor()
                .advance_clock(std::time::Duration::from_millis(500));
            ui.cx.run_until_parked();
        };
        hear(ui, vec![found("c.txt", false)], 420);
        assert_eq!(ui.shown_in_archive(), ["c.txt"]);
        ui.click("entry-0");
        // Neither extracted nor changed before it is all read.
        ui.click("extract");
        assert!(!ui.dialog_open());
        ui.right_click("entry-0");
        assert!(!ui.shows("entry-menu"));

        // More comes before it: what was selected stays so.
        hear(ui, vec![found("a.txt", false), found("b/d.txt", false)], 900);
        assert_eq!(ui.shown_in_archive(), ["b", "a.txt", "c.txt"]);
        let selected = |ui: &mut Ui| ui.ws(|ws, _, cx| ws.archive_page().read(cx).selected_names());
        assert_eq!(selected(ui), ["c.txt"]);
        assert!(reading(ui));

        let listing = packwing_core::list(source.clone(), &packwing_core::Keys::default())
            .expect("the listing");
        let archive = crate::model::Archive::read(source, listing, Default::default(), false);
        ui.ws(move |ws, _, cx| {
            ws.archive_page().update(cx, |p, cx| p.read_through(archive, cx));
        });
        assert!(!reading(ui));
        assert!(!ui.shows("archive-reading"));
        assert_eq!(ui.shown_in_archive(), ["a.txt", "c.txt"]);
        assert_eq!(selected(ui), ["c.txt"]);
        ui.click("extract");
        assert!(ui.dialog_open());
    }
);

scenario!(
    /// A folder of many shows only the rows in sight; one found by a search is
    /// scrolled to.
    a_folder_of_many_draws_what_is_in_sight,
    |ui| {
        let paths: Vec<_> = (0..400)
            .map(|i| ui.file(&format!("many/f{i:03}.txt"), "x"))
            .collect();
        let zip = packwing_core::create(
            &paths,
            &ui.work.join("many.zip"),
            &packwing_core::NewArchive::default(),
            &packwing_core::Control::default(),
        )
        .expect("the archive");
        ui.open(&zip);
        assert_eq!(ui.shown_in_archive().len(), 400);
        assert!(ui.shows("entry-0"));
        assert!(!ui.shows("entry-399"));
        ui.ws(|ws, _, cx| {
            ws.archive_page()
                .update(cx, |p, cx| p.search_for("f399", cx))
        });
        ui.click("entry-0");
        assert_eq!(
            ui.ws(|ws, _, cx| ws.archive_page().read(cx).selected_names()),
            ["f399.txt"]
        );
        assert!(ui.shows("entry-399"));
    }
);
