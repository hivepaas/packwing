//! The Browse page: its folders, the archives opened before, and what is dropped.

use super::scenario;

scenario!(
    /// An archive opened shows on Recent after a restart, and opens again from there.
    an_archive_opened_is_on_recent,
    |ui| {
        let archive = ui.copy_of("sample.7z");
        ui.open(&archive);
        ui.start();
        ui.click("nav-browse");
        assert!(ui.shows("drop-hint"));
        ui.click("tab-recent");
        ui.wait_for("recent: sample.7z");
        assert!(ui.archive_names().is_empty());
        ui.click("recent: sample.7z");
        ui.wait_until("the archive read", |ui| !ui.archive_names().is_empty());
        assert_eq!(ui.archive_failed(), None);
    }
);

scenario!(
    /// Browse goes back to the folder it was left on, and Home leads home.
    browse_is_left_where_it_was,
    |ui| {
        let docs = ui.work.join("docs");
        ui.file("docs/a.txt", "a");
        ui.click("nav-browse");
        let at = docs.clone();
        ui.ws(|ws, _, cx| ws.browse_page().update(cx, |b, cx| b.go_to(&at, cx)));
        assert_eq!(ui.browsed().as_deref(), Some(docs.as_path()));
        ui.start();
        assert_eq!(ui.browsed().as_deref(), Some(docs.as_path()));
        ui.click("nav-browse");
        ui.click("go-home");
        assert_eq!(ui.browsed(), dirs::home_dir());
    }
);

scenario!(
    /// An archive dropped on the window opens.
    a_dropped_archive_opens,
    |ui| {
        let archive = ui.copy_of("sample.7z");
        ui.ws(|ws, window, cx| ws.dropped(vec![archive], window, cx));
        ui.wait_until("the archive read", |ui| !ui.archive_names().is_empty());
        assert_eq!(ui.archive_failed(), None);
    }
);

scenario!(
    /// A file in Browse has Checksums… and Show in the file manager in its menu, as one
    /// of a storage has.
    browse_has_checksums_and_the_file_manager,
    |ui| {
        let docs = ui.work.join("docs");
        ui.file("docs/a.txt", "abc");
        ui.click("nav-browse");
        let at = docs.clone();
        ui.ws(|ws, _, cx| ws.browse_page().update(cx, |b, cx| b.go_to(&at, cx)));
        let menu = |ui: &mut crate::ui_tests::Ui| {
            ui.ws(|ws, _, cx| {
                let explorer = ws.browse_page().read(cx).explorer().clone();
                explorer.update(cx, |e, cx| {
                    e.list_now(cx);
                    e.choose("a.txt", cx);
                    e.show_menu(400., 300., false, cx);
                });
            });
        };
        menu(ui);
        ui.click("menu-file-manager");
        assert_eq!(
            crate::file_manager::asked_for_test(),
            [("show", docs.join("a.txt"))]
        );
        menu(ui);
        ui.click("menu-checksums");
        ui.wait_for("checksum-copy-0");
    }
);

scenario!(
    /// A right click on an archive of Recent opens its menu: it shows it in
    /// the file manager, copies its path, asks its checksums, and takes it
    /// out of the list, leaving the file.
    the_recent_menu,
    |ui| {
        let archive = ui.copy_of("sample.7z");
        ui.open(&archive);
        ui.click("nav-browse");
        ui.click("tab-recent");
        ui.wait_for("recent: sample.7z");
        ui.right_click("recent: sample.7z");
        ui.wait_for("recent-menu");
        for item in ["menu-open", "menu-extract", "menu-sign", "menu-checksums"] {
            assert!(ui.shows(item), "{item}");
        }
        ui.click("menu-file-manager");
        assert!(!ui.shows("recent-menu"));
        assert!(
            crate::file_manager::asked_for_test().contains(&("show", archive.clone())),
            "{:?}",
            crate::file_manager::asked_for_test()
        );
        // Escape closes it.
        ui.right_click("recent: sample.7z");
        ui.wait_for("recent-menu");
        ui.keys("escape");
        ui.wait_until("the menu to close", |ui| !ui.shows("recent-menu"));
        ui.right_click("recent: sample.7z");
        ui.click("menu-copy-path");
        let copied = ui.cx.read_from_clipboard().and_then(|c| c.text());
        assert_eq!(copied.as_deref(), Some(archive.display().to_string().as_str()));
        ui.right_click("recent: sample.7z");
        ui.click("menu-checksums");
        ui.wait_until("the checksums", |ui| ui.dialog_open());
        ui.keys("escape");
        ui.wait_for_no_dialog();
        ui.right_click("recent: sample.7z");
        ui.click("menu-forget");
        ui.wait_until("it out of the list", |ui| !ui.shows("recent: sample.7z"));
        assert!(archive.exists(), "the file stays");
        // Out of the settings too: not back after a restart.
        ui.start();
        ui.click("nav-browse");
        ui.click("tab-recent");
        assert!(!ui.shows("recent: sample.7z"));
    }
);

scenario!(
    /// An archive no longer there offers its path and to take it out, alone.
    the_recent_menu_of_one_gone,
    |ui| {
        let archive = ui.copy_of("sample.7z");
        ui.open(&archive);
        std::fs::remove_file(&archive).expect("it goes");
        ui.click("nav-browse");
        ui.click("tab-recent");
        ui.wait_for("recent: sample.7z");
        ui.right_click("recent: sample.7z");
        ui.wait_for("recent-menu");
        assert!(ui.shows("menu-copy-path") && ui.shows("menu-forget"));
        for item in ["menu-open", "menu-extract", "menu-sign", "menu-checksums", "menu-file-manager"] {
            assert!(!ui.shows(item), "{item}");
        }
        ui.click("menu-forget");
        ui.wait_until("it out of the list", |ui| !ui.shows("recent: sample.7z"));
    }
);

/// Opens the menu of `name`, in the folder Browse shows.
fn menu_of(ui: &mut crate::ui_tests::Ui, name: &str) {
    let name = name.to_owned();
    ui.ws(|ws, _, cx| {
        let explorer = ws.browse_page().read(cx).explorer().clone();
        explorer.update(cx, |e, cx| {
            e.list_now(cx);
            e.choose(&name, cx);
            e.show_menu(400., 300., false, cx);
        });
    });
}

scenario!(
    /// A file in Browse is split into parts beside it, and joined back from one of
    /// them into what it was.
    a_file_is_split_and_joined_back,
    |ui| {
        let docs = ui.work.join("docs");
        let body = "y".repeat(2_500_000);
        ui.file("docs/big.txt", &body);
        ui.click("nav-browse");
        let at = docs.clone();
        ui.ws(|ws, _, cx| ws.browse_page().update(cx, |b, cx| b.go_to(&at, cx)));
        menu_of(ui, "big.txt");
        ui.click("menu-split");
        ui.wait_for("split");
        ui.choose("part-size", "Custom size…");
        ui.type_in("Size", "1");
        ui.click("split");
        ui.wait_for_no_dialog();
        let part = |n: u32| docs.join(format!("big.txt.{n:03}"));
        for n in 1..=3 {
            let size = std::fs::metadata(part(n)).expect("the part").len();
            assert!(n == 3 || size == 1_000_000, "part {n}: {size}");
        }
        assert!(!part(4).exists());
        std::fs::remove_file(docs.join("big.txt")).expect("the file");
        menu_of(ui, "big.txt.002");
        ui.click("menu-join");
        ui.wait_for_no_dialog();
        assert_eq!(std::fs::read_to_string(docs.join("big.txt")).expect("joined"), body);
    }
);

scenario!(
    /// An archive in Browse is converted into another format beside it, holding what
    /// it held.
    an_archive_is_converted,
    |ui| {
        let archive = ui.copy_of("sample.7z");
        let held = crate::ui_tests::listed(&archive, None);
        ui.click("nav-browse");
        let at = ui.work.clone();
        ui.ws(|ws, _, cx| ws.browse_page().update(cx, |b, cx| b.go_to(&at, cx)));
        menu_of(ui, "sample.7z");
        ui.click("menu-convert");
        ui.wait_for("create");
        ui.choose("format", ".tar.zst");
        ui.click("create");
        ui.wait_for_no_dialog();
        let made = ui.work.join("sample.tar.zst");
        assert_eq!(crate::ui_tests::listed(&made, None), held);
    }
);

scenario!(
    /// The first part of a 7z archive cut in parts is an archive in Browse, as
    /// on Recent: its menu has Open and Extract…, and Open reads its parts as
    /// one.
    a_cut_archive_opens_from_browse,
    |ui| {
        let parts = ui.work.join("parts");
        let paths = vec![ui.file("src/a.txt", &"packwing ".repeat(4_000))];
        let new = packwing_core::NewArchive {
            kind: packwing_core::Kind::SevenZip,
            level: packwing_core::Level::Store,
            part_size: Some(10_000),
            ..packwing_core::NewArchive::default()
        };
        std::fs::create_dir_all(&parts).expect("a folder");
        packwing_core::create(&paths, &parts.join("cut.7z"), &new, &packwing_core::Control::default())
            .expect("the archive");
        assert!(parts.join("cut.7z.002").exists(), "it is in parts");
        ui.click("nav-browse");
        let at = parts.clone();
        ui.ws(|ws, _, cx| ws.browse_page().update(cx, |b, cx| b.go_to(&at, cx)));
        menu_of(ui, "cut.7z.001");
        ui.wait_for("menu-open");
        assert!(ui.shows("menu-extract"));
        // A part is joined, not converted alone.
        assert!(ui.shows("menu-join") && !ui.shows("menu-convert"));
        ui.click("menu-open");
        ui.wait_until("the archive read", |ui| !ui.archive_names().is_empty());
        assert_eq!(ui.archive_failed(), None);
        assert_eq!(ui.archive_names(), ["a.txt"]);
    }
);
