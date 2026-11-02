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
        let (one, two) = (ui.work.join("one"), ui.work.join("two"));
        add_far(ui, "One", &one);
        ui.wait_for_no_dialog();
        assert_eq!(ui.remote_storages(), 1);
        add_far(ui, "Two", &two);
        assert!(ui.dialog_open(), "the second was not refused");
        assert_eq!(ui.remote_storages(), 1);
        ui.click("close");
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
