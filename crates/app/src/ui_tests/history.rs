//! Back and forward: the mouse's buttons and the keys go through the folders and
//! pages the user went to.

use super::*;
use crate::pages::settings::Section;
use crate::workspace::Page;

/// The folder the Storages page shows.
fn folder(ui: &mut Ui) -> Vec<String> {
    ui.ws(|ws, _, cx| ws.explorer(cx).read(cx).folder().to_vec())
}

fn open(ui: &mut Ui, folder: &[&str]) {
    let folder: Vec<String> = folder.iter().map(|s| s.to_string()).collect();
    ui.ws(|ws, _, cx| {
        ws.explorer(cx)
            .update(cx, |e, cx| e.open_folder(folder, cx))
    });
}

fn page(ui: &mut Ui) -> Page {
    ui.ws(|ws, _, _| ws.page())
}

/// Back, by the key for it here.
const BACK: &str = if cfg!(target_os = "macos") {
    "cmd-["
} else {
    "alt-left"
};
const FORWARD: &str = if cfg!(target_os = "macos") {
    "cmd-]"
} else {
    "alt-right"
};

scenario!(
    /// The mouse's back and forward buttons, and their keys, go through the folders
    /// opened and the pages shown; going somewhere new forgets what was ahead.
    back_and_forward_go_through_folders_and_pages,
    |ui| {
        ui.file("place/a/b/c.txt", "c");
        ui.file("place/d/e.txt", "e");
        let place = ui.work.join("place");
        ui.storage(&place);
        open(ui, &["a"]);
        open(ui, &["a", "b"]);
        ui.mouse_back(true);
        assert_eq!(folder(ui), ["a"]);
        ui.keys(BACK);
        assert!(folder(ui).is_empty());
        ui.mouse_back(false);
        assert_eq!(folder(ui), ["a"]);
        ui.keys(FORWARD);
        assert_eq!(folder(ui), ["a", "b"]);
        // To another page, and back to the folder it left.
        ui.click("nav-settings");
        assert_eq!(page(ui), Page::Settings);
        ui.mouse_back(true);
        assert_eq!(page(ui), Page::Storages);
        assert_eq!(folder(ui), ["a", "b"]);
        ui.mouse_back(false);
        assert_eq!(page(ui), Page::Settings);
        assert_eq!(
            ui.ws(|ws, _, cx| ws.settings_section(cx)),
            Section::General
        );
        // Back, then somewhere new: there is nothing ahead.
        ui.mouse_back(true);
        ui.mouse_back(true);
        assert_eq!(folder(ui), ["a"]);
        open(ui, &["d"]);
        ui.mouse_back(false);
        assert_eq!(folder(ui), ["d"]);
        ui.mouse_back(true);
        assert_eq!(folder(ui), ["a"]);
    }
);

scenario!(
    /// While a dialog is up, back and forward leave everything as it is.
    back_waits_for_a_dialog,
    |ui| {
        ui.file("place/a/b.txt", "b");
        let place = ui.work.join("place");
        ui.storage(&place);
        open(ui, &["a"]);
        ui.ws(|ws, window, cx| ws.open_checksums_of("b.txt", window, cx));
        ui.wait_for("checksum-copy-0");
        ui.mouse_back(true);
        assert!(ui.dialog_open());
        assert_eq!(folder(ui), ["a"]);
    }
);

scenario!(
    /// In a field, Alt+Left moves through what is typed, not back.
    a_field_keeps_its_keys,
    |ui| {
        if cfg!(target_os = "macos") {
            return;
        }
        ui.file("place/a/b.txt", "b");
        let place = ui.work.join("place");
        ui.storage(&place);
        open(ui, &["a"]);
        ui.type_in("Search this folder", "b");
        ui.keys("alt-left");
        assert_eq!(folder(ui), ["a"]);
    }
);

/// The archive open's name and the folder shown in it.
fn in_archive(ui: &mut Ui) -> (String, Vec<String>) {
    ui.ws(|ws, _, cx| {
        let page = ws.archive_page();
        let a = page.read(cx).archive().expect("an archive").clone();
        (a.name.clone(), a.folder.clone())
    })
}

scenario!(
    /// Back goes through the folders of the archive open; those of one closed
    /// for another are passed.
    back_goes_through_an_archive_s_folders,
    |ui| {
        ui.file("tree/x/y.txt", "y");
        let make = |ui: &mut Ui, from: &str, to: &str| {
            packwing_core::create(
                &[ui.work.join(from)],
                &ui.work.join(to),
                &packwing_core::NewArchive::default(),
                &packwing_core::Control::default(),
            )
            .expect("the archive")
        };
        let one = make(ui, "tree", "one.zip");
        let two = make(ui, "tree/x/y.txt", "two.zip");
        ui.open(&one);
        let enter = |ui: &mut Ui, folder: &[&str]| {
            let folder: Vec<String> = folder.iter().map(|s| s.to_string()).collect();
            ui.ws(|ws, _, cx| ws.archive_page().update(cx, |p, cx| p.enter(folder, cx)));
        };
        enter(ui, &["tree"]);
        enter(ui, &["tree", "x"]);
        ui.mouse_back(true);
        assert_eq!(in_archive(ui), ("one.zip".to_owned(), vec!["tree".to_owned()]));
        ui.open(&two);
        ui.mouse_back(true);
        assert_eq!(in_archive(ui), ("two.zip".to_owned(), Vec::new()));
        assert_eq!(page(ui), Page::Archive);
    }
);
