//! Changing a ZIP archive: adding, deleting, renaming.

use std::path::PathBuf;

use super::{Ui, listed, scenario};

/// A ZIP archive of `a.txt` and `b.txt`, made by the app, and opened.
fn opened_zip(ui: &mut Ui) -> PathBuf {
    let paths = vec![ui.file("a.txt", "a"), ui.file("b.txt", "b")];
    let made = packwing_core::create(
        &paths,
        &ui.work.join("two.zip"),
        &packwing_core::NewArchive::default(),
        &packwing_core::Control::default(),
    )
    .expect("the archive");
    ui.open(&made);
    assert_eq!(ui.archive_failed(), None);
    made
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
        let held = |ui: &Ui| {
            let out = ui.work.join("out");
            let _ = std::fs::remove_dir_all(&out);
            let (keys, control) = (packwing_core::Keys::default(), packwing_core::Control::default());
            packwing_core::extract(&zip, &out, &keys, &control).expect("it comes out");
            std::fs::read_to_string(out.join("b.txt")).expect("b.txt")
        };
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
