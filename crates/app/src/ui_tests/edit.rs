//! Changing a ZIP or 7z archive: adding, deleting, renaming; and an archive
//! that cannot be changed offers none of it.

use std::path::{Path, PathBuf};

use packwing_core::{Control, Keys, Kind, NewArchive};

use super::{Ui, listed, scenario};

/// An archive of `paths` at `name` in the work folder, made by the app as
/// `new` says, and opened.
fn opened(ui: &mut Ui, paths: &[PathBuf], name: &str, new: &NewArchive) -> PathBuf {
    let made = packwing_core::create(paths, &ui.work.join(name), new, &Control::default())
        .expect("the archive");
    ui.open(&made);
    assert_eq!(ui.archive_failed(), None);
    made
}

/// A ZIP archive of `a.txt` and `b.txt`, made by the app, and opened.
fn opened_zip(ui: &mut Ui) -> PathBuf {
    let paths = vec![ui.file("a.txt", "a"), ui.file("b.txt", "b")];
    opened(ui, &paths, "two.zip", &NewArchive::default())
}

/// A 7z archive of `a.txt` and `b.txt`, with `password` (its names not
/// hidden), made by the app, and opened.
fn opened_7z(ui: &mut Ui, password: Option<&str>) -> PathBuf {
    let paths = vec![ui.file("a.txt", "a"), ui.file("b.txt", "b")];
    let new = NewArchive {
        kind: Kind::SevenZip,
        password: password.map(|p| p.as_bytes().to_vec()),
        ..NewArchive::default()
    };
    opened(ui, &paths, "two.7z", &new)
}

/// `name`'s text in `archive`, extracted with `password`; `None` if it does
/// not come out with it.
fn text_in(ui: &Ui, archive: &Path, name: &str, password: Option<&str>) -> Option<String> {
    let out = ui.work.join("out");
    let _ = std::fs::remove_dir_all(&out);
    let keys = Keys {
        password: password.map(|p| p.as_bytes().to_vec()),
        ..Keys::default()
    };
    packwing_core::extract(archive, &out, &keys, &Control::default()).ok()?;
    std::fs::read_to_string(out.join(name)).ok()
}

/// What `archive` holds, sorted, read with `password`.
fn sorted(archive: &Path, password: Option<&str>) -> Vec<String> {
    let mut held = listed(archive, password);
    held.sort();
    held
}

scenario!(
    /// A file dropped on an open ZIP archive goes in it.
    a_drop_adds_to_the_archive,
    |ui| {
        let zip = opened_zip(ui);
        let new = ui.file("c.txt", "c");
        ui.ws(|ws, window, cx| ws.dropped(vec![new], window, cx));
        ui.wait_for("add");
        ui.click("add");
        ui.wait_for_no_dialog();
        let mut held = listed(&zip, None);
        held.sort();
        assert_eq!(held, ["a.txt", "b.txt", "c.txt"]);
        ui.wait_until("the page to show it", |ui| ui.archive_names().len() == 3);
    }
);

scenario!(
    /// A right click on an entry opens its menu, which deletes it: the page
    /// has no buttons for it.
    the_right_click_menu_deletes,
    |ui| {
        let zip = opened_zip(ui);
        assert!(!ui.shows("delete") && !ui.shows("rename"));
        ui.right_click("entry-0");
        ui.wait_for("entry-menu");
        assert!(ui.shows("menu-open") && ui.shows("menu-extract") && ui.shows("menu-rename"));
        ui.click("menu-delete");
        ui.wait_for("go");
        ui.click("go");
        ui.wait_for_no_dialog();
        assert_eq!(listed(&zip, None), ["b.txt"]);
    }
);

scenario!(
    /// The menu renames, with the one entry right-clicked selected; with two
    /// selected it deletes both, but renames neither.
    the_right_click_menu_renames_one,
    |ui| {
        let zip = opened_zip(ui);
        ui.click("entry-0");
        // A right click on another selects it alone.
        ui.right_click("entry-1");
        ui.wait_for("entry-menu");
        ui.click("menu-rename");
        ui.wait_for("go");
        ui.replace_in("Name", "renamed.txt");
        ui.click("go");
        ui.wait_for_no_dialog();
        let mut held = listed(&zip, None);
        held.sort();
        assert_eq!(held, ["a.txt", "renamed.txt"]);
        ui.wait_until("the page to show it", |ui| {
            ui.archive_names().iter().any(|n| n == "renamed.txt")
        });
        ui.ws(|ws, _, cx| ws.select_in_archive(&[0, 1], cx));
        ui.right_click("entry-0");
        ui.wait_for("entry-menu");
        assert!(ui.shows("menu-delete"));
        assert!(!ui.shows("menu-rename"), "two are not renamed at once");
        assert!(!ui.shows("menu-open"));
        ui.keys("escape");
        ui.wait_until("the menu to close", |ui| !ui.shows("entry-menu"));
    }
);

scenario!(
    /// F2 renames and Delete deletes what is selected, as in Browse.
    keys_rename_and_delete,
    |ui| {
        let zip = opened_zip(ui);
        ui.click("entry-0");
        ui.keys("f2");
        ui.wait_for("go");
        ui.replace_in("Name", "first.txt");
        ui.click("go");
        ui.wait_for_no_dialog();
        ui.wait_until("the page to show it", |ui| {
            ui.archive_names().iter().any(|n| n == "first.txt")
        });
        ui.click("entry-0");
        ui.keys("delete");
        ui.wait_for("go");
        ui.click("go");
        ui.wait_for_no_dialog();
        assert_eq!(listed(&zip, None).len(), 1);
    }
);

scenario!(
    /// What is selected can be deleted.
    delete_takes_out_what_is_selected,
    |ui| {
        let zip = opened_zip(ui);
        ui.click("entry-0");
        ui.ws(|ws, window, cx| ws.open_edit(false, window, cx));
        ui.wait_for("go");
        ui.click("go");
        ui.wait_for_no_dialog();
        assert_eq!(listed(&zip, None).len(), 1);
    }
);

scenario!(
    /// What is selected can be renamed.
    rename_changes_the_name,
    |ui| {
        let zip = opened_zip(ui);
        ui.click("entry-0");
        ui.ws(|ws, window, cx| ws.open_edit(true, window, cx));
        ui.wait_for("go");
        ui.replace_in("Name", "renamed.txt");
        ui.click("go");
        ui.wait_for_no_dialog();
        let held = listed(&zip, None);
        assert!(held.iter().any(|n| n == "renamed.txt"), "{held:?}");
        assert_eq!(held.len(), 2, "{held:?}");
    }
);

scenario!(
    /// A file added under a name the archive has already replaces it, or is skipped, as
    /// chosen.
    an_added_file_replaces_or_is_skipped,
    |ui| {
        let zip = opened_zip(ui);
        let held = |ui: &Ui| text_in(ui, &zip, "b.txt", None).expect("b.txt");
        for (choice, after) in [("Skip it", "b"), ("Replace it", "new")] {
            let new = ui.file("new/b.txt", "new");
            ui.ws(|ws, window, cx| ws.dropped(vec![new], window, cx));
            ui.wait_for("add");
            ui.choose("on-existing", choice);
            ui.click("add");
            ui.wait_for_no_dialog();
            assert_eq!(held(ui), after, "{choice}");
            assert_eq!(listed(&zip, None).len(), 2, "{choice}");
        }
    }
);

scenario!(
    /// A 7z archive is changed as a ZIP archive is: a file dropped on it goes
    /// in, and the menu renames one and deletes one.
    a_7z_archive_takes_a_drop_a_rename_and_a_delete,
    |ui| {
        let archive = opened_7z(ui, None);
        let new = ui.file("c.txt", "c");
        ui.ws(|ws, window, cx| ws.dropped(vec![new], window, cx));
        ui.wait_for("add");
        ui.click("add");
        ui.wait_for_no_dialog();
        assert_eq!(sorted(&archive, None), ["a.txt", "b.txt", "c.txt"]);
        ui.wait_until("the page to show it", |ui| ui.archive_names().len() == 3);
        ui.right_click("entry-0");
        ui.wait_for("entry-menu");
        assert!(ui.shows("menu-rename"), "a 7z archive's names change");
        ui.click("menu-rename");
        ui.wait_for("go");
        ui.replace_in("Name", "renamed.txt");
        ui.click("go");
        ui.wait_for_no_dialog();
        ui.wait_until("the page to show it", |ui| {
            ui.archive_names().iter().any(|n| n == "renamed.txt")
        });
        assert_eq!(sorted(&archive, None).len(), 3);
        assert!(listed(&archive, None).iter().any(|n| n == "renamed.txt"));
        ui.click("entry-0");
        ui.keys("delete");
        ui.wait_for("go");
        ui.click("go");
        ui.wait_for_no_dialog();
        assert_eq!(listed(&archive, None).len(), 2);
        assert_eq!(text_in(ui, &archive, "c.txt", None).as_deref(), Some("c"));
    }
);

scenario!(
    /// A folder of a 7z archive renamed with F2 renames what is in it.
    a_7z_folder_renamed_takes_its_files_along,
    |ui| {
        let folder = ui.file("docs/a.txt", "a").parent().expect("docs").to_path_buf();
        ui.file("docs/b.txt", "b");
        let new = NewArchive {
            kind: Kind::SevenZip,
            ..NewArchive::default()
        };
        let archive = opened(ui, &[folder], "docs.7z", &new);
        ui.click("entry-0");
        ui.keys("f2");
        ui.wait_for("go");
        ui.replace_in("Name", "papers");
        ui.click("go");
        ui.wait_for_no_dialog();
        let held = sorted(&archive, None);
        assert!(held.iter().all(|n| n.starts_with("papers")), "{held:?}");
        for name in ["papers/a.txt", "papers/b.txt"] {
            assert!(held.iter().any(|n| n == name), "{name} in {held:?}");
        }
        assert_eq!(text_in(ui, &archive, "papers/b.txt", None).as_deref(), Some("b"));
    }
);

scenario!(
    /// A 7z archive whose files alone are encrypted opens without its
    /// password; a file dropped on it asks for it, again when it is wrong, and
    /// goes in encrypted with the right one.
    adding_to_an_encrypted_7z_asks_for_its_password,
    |ui| {
        let archive = opened_7z(ui, Some("secret"));
        assert!(!ui.dialog_open(), "its names are not encrypted");
        let new = ui.file("c.txt", "c");
        ui.ws(|ws, window, cx| ws.dropped(vec![new], window, cx));
        ui.wait_for("add");
        ui.click("add");
        ui.wait_for("field: The archive’s password");
        ui.type_in("The archive’s password", "wrong");
        ui.click("open");
        ui.wait_for("open");
        assert_eq!(listed(&archive, Some("secret")).len(), 2, "added with the wrong one");
        ui.replace_in("The archive’s password", "secret");
        ui.click("open");
        ui.wait_for_no_dialog();
        assert_eq!(sorted(&archive, Some("secret")), ["a.txt", "b.txt", "c.txt"]);
        assert_eq!(text_in(ui, &archive, "c.txt", Some("secret")).as_deref(), Some("c"));
        assert_eq!(text_in(ui, &archive, "c.txt", None), None, "c.txt in the clear");
    }
);

scenario!(
    /// An archive that cannot be changed, as a .tar.gz, offers neither rename
    /// nor delete: not in its menu, not by its keys.
    what_cannot_be_changed_offers_no_change,
    |ui| {
        let paths = vec![ui.file("a.txt", "a"), ui.file("b.txt", "b")];
        let new = NewArchive {
            kind: Kind::TarGz,
            ..NewArchive::default()
        };
        let archive = opened(ui, &paths, "two.tar.gz", &new);
        ui.right_click("entry-0");
        ui.wait_for("entry-menu");
        assert!(ui.shows("menu-extract"));
        assert!(!ui.shows("menu-rename") && !ui.shows("menu-delete"));
        ui.keys("escape");
        ui.wait_until("the menu to close", |ui| !ui.shows("entry-menu"));
        ui.click("entry-0");
        for key in ["f2", "delete"] {
            ui.keys(key);
            ui.cx.run_until_parked();
            assert!(!ui.dialog_open(), "{key} asked to change it");
        }
        assert_eq!(listed(&archive, None).len(), 2);
    }
);
