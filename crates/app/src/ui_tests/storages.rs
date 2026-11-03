//! Storages: folders, those across a network, copying, and what a plan allows.

use std::path::Path;

use super::{Ui, scenario};
use packwing_license::Plan;

/// Adds a storage across a network through its dialog, named `name`, on the folder `folder`.
fn add_far(ui: &mut Ui, name: &str, folder: &Path) {
    std::fs::create_dir_all(folder).expect("the folder");
    ui.ws(|ws, window, cx| ws.open_add_storage(window, cx));
    ui.wait_for("kind-0");
    ui.click("kind-0");
    ui.type_in("Name", name);
    ui.type_in("Folder", &folder.display().to_string());
    ui.click("storage-add");
}

scenario!(
    /// A folder added shows what it holds, and a folder made in it appears.
    a_folder_is_browsed,
    |ui| {
        let folder = ui.work.join("place");
        ui.file("place/a.txt", "a");
        ui.storage(&folder);
        assert_eq!(ui.explorer_names(), ["a.txt"]);
        ui.ws(|ws, window, cx| {
            let explorer = ws.explorer(cx);
            explorer.update(cx, |e, cx| {
                e.type_new_folder(window, cx);
                e.type_name("made", cx);
                e.commit_name(window, cx);
            });
        });
        assert!(folder.join("made").is_dir());
    }
);

scenario!(
    /// Copying leaves the original, moving does not.
    copy_and_move_between_folders,
    |ui| {
        let (from, to) = (ui.work.join("from"), ui.work.join("to"));
        ui.file("from/a.txt", "a");
        ui.file("from/b.txt", "b");
        std::fs::create_dir_all(&to).expect("the folder");
        ui.storage(&from);
        ui.explorer_names();
        let target = to.clone();
        ui.ws(|ws, window, cx| {
            ws.explorer(cx).update(cx, |e, cx| e.choose("a.txt", cx));
            ws.start_transfer_to(false, &target, window, cx);
        });
        ui.wait_for_no_dialog();
        assert!(to.join("a.txt").exists() && from.join("a.txt").exists());
        let target = to.clone();
        ui.ws(|ws, window, cx| {
            ws.explorer(cx).update(cx, |e, cx| e.choose("b.txt", cx));
            ws.start_transfer_to(true, &target, window, cx);
        });
        ui.wait_for_no_dialog();
        assert!(to.join("b.txt").exists() && !from.join("b.txt").exists());
    }
);

scenario!(
    /// Free keeps one storage across a network: the second is refused, and folders on
    /// this computer are not limited.
    free_keeps_one_remote_storage,
    |ui| {
        assert_eq!(ui.plan(), Plan::Free);
        let one = ui.work.join("one");
        add_far(ui, "One", &one);
        ui.wait_for_no_dialog();
        assert_eq!(ui.remote_storages(), 1);
        // The second is not asked for: no field, no Add, but where a license is entered.
        ui.ws(|ws, window, cx| ws.open_add_storage(window, cx));
        ui.wait_for("kind-0");
        ui.click("kind-0");
        for hidden in ["field: Name", "storage-add", "storage-test"] {
            assert!(!ui.shows(hidden), "{hidden} is shown over the limit");
        }
        ui.click("storage-license");
        assert!(!ui.dialog_open());
        assert_eq!(
            ui.ws(|ws, _, cx| ws.settings_section(cx)),
            crate::pages::settings::Section::License
        );
        assert_eq!(ui.remote_storages(), 1);
        for i in 0..3 {
            let folder = ui.work.join(format!("here{i}"));
            ui.storage(&folder);
        }
        assert_eq!(ui.remote_storages(), 1);
    }
);

scenario!(
    /// A storage across a network comes back after a restart, and a paid key lifts the limit.
    a_paid_plan_keeps_any_number,
    |ui| {
        let key = include_str!("../../../license/testdata/pro.key").trim();
        ui.settings(5);
        ui.type_in("Paste your license key", key);
        ui.click("license-enter");
        assert_eq!(ui.plan(), Plan::Pro);
        for name in ["one", "two", "three"] {
            let folder = ui.work.join(name);
            add_far(ui, name, &folder);
            ui.wait_for_no_dialog();
        }
        assert_eq!(ui.remote_storages(), 3);
        ui.start();
        assert_eq!(ui.remote_storages(), 3);
        assert_eq!(ui.plan(), Plan::Pro);
    }
);

scenario!(
    /// What is on a storage across a network opens where it is, and comes out here.
    an_archive_on_a_storage_opens_and_extracts,
    |ui| {
        let far = ui.work.join("far");
        std::fs::create_dir_all(&far).expect("the folder");
        std::fs::copy(crate::ui_tests::fixture("sample.7z"), far.join("sample.7z"))
            .expect("the fixture");
        add_far(ui, "Far", &far);
        ui.wait_for_no_dialog();
        assert_eq!(ui.explorer_names(), ["sample.7z"]);
        ui.ws(|ws, _, cx| {
            let explorer = ws.explorer(cx);
            explorer.update(cx, |e, cx| {
                let item = e.selection_of("sample.7z");
                e.open(&item, cx);
            });
        });
        ui.wait_until("the archive read", |ui| !ui.archive_names().is_empty());
        ui.click("extract");
        ui.wait_for("dest-folder");
    }
);

scenario!(
    /// With a storage across a network, the Archive page offers to open from one: its
    /// dialog shows folders and archives, and a click on an archive opens it.
    an_archive_is_chosen_on_a_storage,
    |ui| {
        ui.wait_for("open-archive");
        assert!(!ui.shows("open-stored"), "offered with no storage across a network");
        let far = ui.work.join("far");
        std::fs::create_dir_all(far.join("old")).expect("the folder");
        std::fs::copy(
            crate::ui_tests::fixture("sample.7z"),
            far.join("old/sample.7z"),
        )
        .expect("the fixture");
        std::fs::copy(
            crate::ui_tests::fixture("rar5_multiple_files.rar"),
            far.join("other.rar"),
        )
        .expect("the fixture");
        ui.file("far/notes.txt", "not an archive");
        add_far(ui, "Far", &far);
        ui.wait_for_no_dialog();
        ui.click("nav-archive");
        ui.click("open-stored");
        ui.wait_for("pick: old");
        assert!(!ui.shows("pick: notes.txt"), "a file that is no archive is offered");
        ui.click("pick: old");
        ui.wait_for("pick: sample.7z");
        ui.click("pick: sample.7z");
        ui.wait_for_no_dialog();
        ui.wait_until("the archive read", |ui| !ui.archive_names().is_empty());
        // Open, the archive's page offers another beside its search.
        let first = ui.archive_names();
        assert!(ui.shows("open-another"));
        ui.click("open-stored");
        ui.wait_for("pick: other.rar");
        ui.click("pick: other.rar");
        ui.wait_for_no_dialog();
        ui.wait_until("the other archive read", |ui| {
            let names = ui.archive_names();
            !names.is_empty() && names != first
        });
    }
);

#[cfg(any(windows, target_os = "macos"))]
scenario!(
    /// A file copied here from a storage across a network is marked as from the Internet.
    a_copy_from_a_storage_is_marked,
    |ui| {
        let (far, here) = (ui.work.join("far"), ui.work.join("here"));
        ui.file("far/a.txt", "a");
        std::fs::create_dir_all(&here).expect("the folder");
        add_far(ui, "Far", &far);
        ui.wait_for_no_dialog();
        ui.explorer_names();
        let target = here.clone();
        ui.ws(|ws, window, cx| {
            ws.explorer(cx).update(cx, |e, cx| e.choose("a.txt", cx));
            ws.start_transfer_to(false, &target, window, cx);
        });
        ui.wait_for_no_dialog();
        let copy = here.join("a.txt");
        assert!(copy.exists());
        assert!(
            packwing_core::came_from_internet(&packwing_core::Source::from(&copy)),
            "the copy carries no mark"
        );
    }
);

scenario!(
    /// Remove from this list asks first: Cancel keeps the storage, Remove takes it off
    /// the list and leaves its folder.
    removing_a_storage_is_asked_first,
    |ui| {
        let folder = ui.work.join("place");
        ui.storage(&folder);
        let count = |ui: &mut Ui| ui.ws(|ws, _, cx| ws.storages_page().read(cx).count());
        let before = count(ui);
        let last = format!("storage-{}", before - 1);
        ui.right_click(&last);
        ui.click("storage-remove");
        ui.wait_for("storage-remove-yes");
        ui.click("cancel");
        assert!(!ui.dialog_open());
        assert_eq!(count(ui), before);
        ui.right_click(&last);
        ui.click("storage-remove");
        ui.click("storage-remove-yes");
        assert!(!ui.dialog_open());
        assert_eq!(count(ui), before - 1);
        assert!(folder.is_dir());
        ui.start();
        assert_eq!(count(ui), before - 1);
    }
);
