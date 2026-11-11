//! Opening a file of an archive: an archive in packwing, inside the one open
//! and back; any other file shown, or offered to the system's app; and a file
//! that is an archive by another name, opened as one when asked.

use std::path::PathBuf;

use packwing_core::{Control, NewArchive};

use super::{Ui, scenario};

/// A ZIP archive of `paths` at `name` in the work folder.
fn zip(ui: &Ui, paths: &[PathBuf], name: &str) -> PathBuf {
    let path = ui.work.join(name);
    packwing_core::create(paths, &path, &NewArchive::default(), &Control::default())
        .expect("the archive");
    path
}

/// `outer.zip`, holding `inner.zip` (of `a.txt`), `tool.jar` (a ZIP of `b.txt`
/// by a package's name), `note.txt` and `data.bin`; opened.
fn opened_nest(ui: &mut Ui) -> PathBuf {
    let inner = zip(ui, &[ui.file("in/a.txt", "a")], "inner.zip");
    let jar = zip(ui, &[ui.file("jar/b.txt", "b")], "tool.jar");
    let note = ui.file("note.txt", "a note");
    let data = ui.work.join("data.bin");
    std::fs::write(&data, [0u8, 1, 2, 255, 254, 0, 7]).expect("a file");
    let outer = zip(ui, &[inner, jar, note, data], "outer.zip");
    ui.open(&outer);
    assert_eq!(ui.archive_failed(), None);
    outer
}

/// Opens the right-click menu of `name` in the folder the Archive page shows.
fn menu_of(ui: &mut Ui, name: &str) {
    let at = ui
        .shown_in_archive()
        .iter()
        .position(|n| n == name)
        .unwrap_or_else(|| panic!("{name} is not listed"));
    ui.right_click(&format!("entry-{at}"));
    ui.wait_for("entry-menu");
}

/// The archives the one shown was opened from, by name.
fn outer(ui: &mut Ui) -> Vec<String> {
    ui.ws(|ws, _, cx| ws.archive_page().read(cx).outer_names())
}

scenario!(
    /// Open on an archive in the archive opens it in packwing, where the one
    /// it is in was, only to be read; the breadcrumb goes back to that one, at
    /// the folder it was left at, and the copy is gone.
    an_archive_inside_opens_in_packwing_and_goes_back,
    |ui| {
        opened_nest(ui);
        menu_of(ui, "inner.zip");
        assert!(ui.shows("menu-app"), "the system's app is still offered");
        assert!(!ui.shows("menu-as-archive"), "an archive opens so already");
        ui.click("menu-open");
        ui.wait_until("the inner archive read", |ui| ui.archive_names() == ["a.txt"]);
        assert_eq!(outer(ui), ["outer.zip"]);
        let copy = ui.ws(|ws, _, cx| {
            let page = ws.archive_page();
            let page = page.read(cx);
            page.archive().and_then(|a| a.path.clone())
        });
        let copy = copy.expect("a copy on this computer");
        assert!(copy.exists());
        // Only read: what is added to a copy would be lost with it.
        assert!(!ui.shows("add-files"));
        menu_of(ui, "a.txt");
        assert!(!ui.shows("menu-delete") && !ui.shows("menu-rename"));
        ui.keys("escape");
        // Not in Recent: it is gone once it is left.
        let recent = ui.ws(|_, _, cx| crate::prefs::prefs(cx).recent.clone());
        assert!(recent.iter().all(|r| r.path != copy), "{recent:?}");
        ui.click("archive-back-0");
        ui.wait_until("the outer archive shown", |ui| {
            ui.archive_names().iter().any(|n| n == "inner.zip")
        });
        assert!(outer(ui).is_empty());
        assert!(!copy.exists(), "the copy is gone");
        assert!(ui.shows("add-files"));
    }
);

scenario!(
    /// A file that is not an archive opens in the viewer: a text shown; what
    /// it cannot show offered to the system's app and to packwing as an
    /// archive. Back goes back to the archive, the file chosen there.
    a_file_inside_is_shown_or_offered,
    |ui| {
        opened_nest(ui);
        menu_of(ui, "note.txt");
        ui.click("menu-open");
        ui.wait_for("view-app");
        // A text is shown: nothing else is offered for it.
        assert!(!ui.shows("view-as-archive"));
        ui.click("view-back");
        ui.wait_until("the viewer closed", |ui| !ui.shows("viewer"));
        menu_of(ui, "data.bin");
        ui.click("menu-open");
        ui.wait_for("view-as-archive");
        assert!(ui.shows("view-app"));
        ui.click("view-back");
        ui.wait_until("the viewer closed", |ui| !ui.shows("viewer"));
        assert!(outer(ui).is_empty());
    }
);

scenario!(
    /// A package is a program: Open does not open it as an archive, Open as
    /// archive does, inside the one open.
    a_package_inside_opens_as_an_archive_when_asked,
    |ui| {
        opened_nest(ui);
        menu_of(ui, "tool.jar");
        assert!(ui.shows("menu-as-archive"));
        ui.click("menu-as-archive");
        ui.wait_until("the package read", |ui| ui.archive_names() == ["b.txt"]);
        assert_eq!(outer(ui), ["outer.zip"]);
    }
);

scenario!(
    /// A file that does not open as an archive says so, with the way back to
    /// the one it is in.
    what_is_no_archive_says_so_and_goes_back,
    |ui| {
        opened_nest(ui);
        menu_of(ui, "data.bin");
        ui.click("menu-as-archive");
        ui.wait_until("it to fail", |ui| ui.archive_failed().is_some());
        ui.wait_for("archive-back");
        ui.click("archive-back");
        ui.wait_until("the outer archive shown", |ui| {
            ui.archive_names().iter().any(|n| n == "inner.zip")
        });
        assert!(outer(ui).is_empty());
    }
);

scenario!(
    /// Browse opens a file as an archive when asked, whatever its name says.
    browse_opens_a_file_as_an_archive,
    |ui| {
        let book = zip(ui, &[ui.file("book/chapter.txt", "once")], "story.epub");
        ui.click("nav-browse");
        let at = book.parent().expect("its folder").to_path_buf();
        ui.ws(|ws, _, cx| ws.browse_page().update(cx, |b, cx| b.go_to(&at, cx)));
        ui.ws(|ws, _, cx| {
            let explorer = ws.browse_page().read(cx).explorer().clone();
            explorer.update(cx, |e, cx| {
                e.list_now(cx);
                e.choose("story.epub", cx);
                e.show_menu(400., 300., false, cx);
            });
        });
        ui.wait_for("menu-as-archive");
        ui.click("menu-as-archive");
        ui.wait_until("the book read", |ui| ui.archive_names() == ["chapter.txt"]);
        assert_eq!(ui.archive_failed(), None);
    }
);
