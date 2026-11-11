//! The keys that move through a list, alike in Browse and on the Archive
//! page: Up and Down, Page Up and Page Down, Home and End; and on the Archive
//! page, Enter into a folder and Backspace out of it.

use std::path::PathBuf;

use packwing_core::{Control, NewArchive};

use super::{Ui, scenario};

/// Thirty files, `f00.txt` to `f29.txt`, and `sub/x.txt`, in `folder` of the
/// work folder.
fn many(ui: &Ui, folder: &str) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = (0..30)
        .map(|i| ui.file(&format!("{folder}/f{i:02}.txt"), "f"))
        .collect();
    ui.file(&format!("{folder}/sub/x.txt"), "x");
    paths.push(ui.work.join(folder).join("sub"));
    paths
}

/// The names selected on the Archive page.
fn chosen_in_archive(ui: &mut Ui) -> Vec<String> {
    ui.ws(|ws, _, cx| ws.archive_page().read(cx).selected_names())
}

/// The names Browse lists, in their order.
fn browsed_names(ui: &mut Ui) -> Vec<String> {
    ui.ws(|ws, _, cx| {
        let explorer = ws.browse_page().read(cx).explorer().clone();
        explorer.update(cx, |e, cx| e.list_now(cx));
        explorer.read(cx).shown_names()
    })
}

/// The names selected in Browse.
fn chosen_in_browse(ui: &mut Ui) -> Vec<String> {
    ui.ws(|ws, _, cx| {
        let explorer = ws.browse_page().read(cx).explorer().clone();
        explorer
            .read(cx)
            .selection()
            .into_iter()
            .map(|i| i.name)
            .collect()
    })
}

scenario!(
    /// The Archive page's list moves as Browse's does, and Enter and
    /// Backspace go into a folder and out of it.
    keys_move_through_an_archive,
    |ui| {
        let paths = many(ui, "src");
        let zip = ui.work.join("many.zip");
        packwing_core::create(&paths, &zip, &NewArchive::default(), &Control::default())
            .expect("the archive");
        ui.open(&zip);
        assert_eq!(ui.archive_failed(), None);
        let names = ui.shown_in_archive();
        assert_eq!(names.len(), 31);
        ui.click("entry-1");
        assert_eq!(chosen_in_archive(ui), [names[1].clone()]);
        ui.keys("down");
        assert_eq!(chosen_in_archive(ui), [names[2].clone()]);
        ui.keys("up up");
        assert_eq!(chosen_in_archive(ui), [names[0].clone()]);
        ui.keys("pagedown");
        let paged = names
            .iter()
            .position(|n| chosen_in_archive(ui) == [n.clone()])
            .expect("a row chosen");
        assert!(paged > 1, "Page Down moves more than a row: {paged}");
        ui.keys("pageup");
        assert_eq!(chosen_in_archive(ui), [names[0].clone()]);
        ui.keys("end");
        assert_eq!(chosen_in_archive(ui), [names[30].clone()]);
        ui.keys("home");
        assert_eq!(chosen_in_archive(ui), [names[0].clone()]);
        // The folder, listed first.
        assert_eq!(names[0], "sub");
        ui.keys("enter");
        ui.wait_until("the folder shown", |ui| ui.archive_folder() == ["sub"]);
        ui.keys("backspace");
        ui.wait_until("its parent shown", |ui| ui.archive_folder().is_empty());
    }
);

scenario!(
    /// Browse's list takes Page Up, Page Down, Home and End beside Up and Down.
    keys_move_through_browse,
    |ui| {
        many(ui, "browsed");
        let at = ui.work.join("browsed");
        ui.click("nav-browse");
        ui.ws(|ws, _, cx| ws.browse_page().update(cx, |b, cx| b.go_to(&at, cx)));
        ui.wait_until("the folder listed", |ui| browsed_names(ui).len() == 31);
        let names = browsed_names(ui);
        ui.click(&format!("row: {}", names[5]));
        assert_eq!(chosen_in_browse(ui), [names[5].clone()]);
        ui.keys("down");
        assert_eq!(chosen_in_browse(ui), [names[6].clone()]);
        ui.keys("pagedown");
        let paged = names
            .iter()
            .position(|n| chosen_in_browse(ui) == [n.clone()])
            .expect("a row chosen");
        assert!(paged > 7, "Page Down moves more than a row: {paged}");
        ui.keys("end");
        assert_eq!(chosen_in_browse(ui), [names[30].clone()]);
        ui.keys("home");
        assert_eq!(chosen_in_browse(ui), [names[0].clone()]);
    }
);
