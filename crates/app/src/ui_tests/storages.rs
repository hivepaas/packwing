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
    /// Checksums… in a file's menu shows its checksums, and says whether one pasted matches.
    a_file_s_checksums_are_shown_and_compared,
    |ui| {
        let folder = ui.work.join("place");
        ui.file("place/a.txt", "abc");
        ui.storage(&folder);
        ui.explorer_names();
        ui.ws(|ws, _, cx| {
            ws.explorer(cx).update(cx, |e, cx| {
                e.choose("a.txt", cx);
                e.show_menu(400., 300., false, cx);
            });
        });
        ui.click("menu-checksums");
        ui.wait_for("checksum-copy-0");
        assert!(ui.shows("checksum: sha1"));
        assert!(!ui.shows("checksum: md5"));
        ui.click("checksum-compare");
        ui.type_in(
            "Paste the checksum given with the file",
            "BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD  a.txt",
        );
        assert!(ui.shows("checksum-matches"));
        ui.replace_in("Paste the checksum given with the file", &"0".repeat(40));
        assert!(ui.shows("checksum-differs"));
        // One not shown is made when asked, and then compared.
        ui.replace_in(
            "Paste the checksum given with the file",
            "900150983cd24fb0d6963f7d28e17f72",
        );
        ui.click("checksum-make");
        ui.wait_for("checksum-matches");
        assert!(ui.shows("checksum: md5"));
        // Each is kept beside the file, as sha256sum writes one, and the folder shows it.
        ui.click("checksum-save");
        ui.wait_for("checksum-saved");
        assert_eq!(
            std::fs::read_to_string(folder.join("a.txt.sha256")).expect("kept"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad  a.txt\n"
        );
        assert_eq!(
            std::fs::read_to_string(folder.join("a.txt.md5")).expect("kept"),
            "900150983cd24fb0d6963f7d28e17f72  a.txt\n"
        );
        assert!(folder.join("a.txt.sha1").exists());
        ui.wait_until("the folder read again", |ui| {
            ui.explorer_names().contains(&"a.txt.sha256".to_owned())
        });
    }
);

scenario!(
    /// An item of a folder on this computer is shown in the system's file manager, and
    /// the folder opened in it.
    the_file_manager_shows_an_item_and_opens_the_folder,
    |ui| {
        let folder = ui.work.join("place");
        ui.file("place/a.txt", "a");
        ui.storage(&folder);
        ui.explorer_names();
        ui.ws(|ws, _, cx| {
            ws.explorer(cx).update(cx, |e, cx| {
                e.choose("a.txt", cx);
                e.show_menu(400., 300., false, cx);
            });
        });
        ui.click("menu-file-manager");
        assert!(!ui.shows("explorer-menu"));
        ui.ws(|ws, _, cx| {
            ws.explorer(cx)
                .update(cx, |e, cx| e.show_menu(400., 500., true, cx))
        });
        ui.click("menu-file-manager");
        assert_eq!(
            crate::file_manager::asked_for_test(),
            [("show", folder.join("a.txt")), ("open", folder.clone())]
        );
        // Across a network there is nothing here to show.
        let far = ui.work.join("far");
        ui.file("far/b.txt", "b");
        add_far(ui, "Far", &far);
        ui.wait_for_no_dialog();
        assert_eq!(ui.explorer_names(), ["b.txt"]);
        for on_folder in [false, true] {
            ui.ws(|ws, _, cx| {
                ws.explorer(cx).update(cx, |e, cx| {
                    e.choose("b.txt", cx);
                    e.show_menu(400., 300., on_folder, cx);
                })
            });
            assert!(ui.shows("explorer-menu"));
            assert!(!ui.shows("menu-file-manager"));
        }
    }
);

scenario!(
    /// Across a network, checksums wait for Read it, as reading may cost; Copy copies
    /// one, and a list of them pasted is compared by the file's line.
    checksums_across_a_network,
    |ui| {
        let sha256 = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
        let far = ui.work.join("far");
        ui.file("far/a.txt", "abc");
        add_far(ui, "Far", &far);
        ui.wait_for_no_dialog();
        ui.click("nav-storages");
        assert_eq!(ui.explorer_names(), ["a.txt"]);
        ui.ws(|ws, _, cx| {
            ws.explorer(cx).update(cx, |e, cx| {
                e.choose("a.txt", cx);
                e.show_menu(400., 300., false, cx);
            });
        });
        ui.click("menu-checksums");
        assert!(ui.shows("checksum-read"));
        assert!(!ui.shows("checksum-copy-0"), "nothing is read before it is asked");
        ui.click("checksum-read");
        ui.wait_for("checksum-copy-0");
        ui.cx
            .write_to_clipboard(gpui::ClipboardItem::new_string(String::new()));
        ui.click("checksum-copy-0");
        assert_eq!(
            ui.cx.read_from_clipboard().and_then(|c| c.text()).as_deref(),
            Some(sha256)
        );
        ui.click("checksum-compare");
        let list = format!("{}  other.txt\n{sha256}  a.txt\n", "1".repeat(64));
        ui.type_in("Paste the checksum given with the file", &list);
        assert!(ui.shows("checksum-matches"));
        // Kept beside it there, as across a network files are.
        ui.click("checksum-save");
        ui.wait_for("checksum-saved");
        assert!(far.join("a.txt.sha256").exists());
    }
);

scenario!(
    /// The checksums Settings chooses are the ones a file's Checksums… shows.
    the_checksums_shown_follow_settings,
    |ui| {
        let folder = ui.work.join("place");
        ui.file("place/a.txt", "abc");
        ui.settings(3);
        ui.click("checksum-1");
        ui.click("checksum-2");
        ui.storage(&folder);
        ui.explorer_names();
        ui.ws(|ws, _, cx| {
            ws.explorer(cx).update(cx, |e, cx| {
                e.choose("a.txt", cx);
                e.show_menu(400., 300., false, cx);
            });
        });
        ui.click("menu-checksums");
        ui.wait_for("checksum: sha512");
        assert!(ui.shows("checksum: sha256"));
        assert!(!ui.shows("checksum: sha1"), "turned off");
    }
);

scenario!(
    /// A text file viewed has the right-click menu of text to read: Copy all, with
    /// nothing chosen.
    a_text_viewed_has_a_copy_menu,
    |ui| {
        let folder = ui.work.join("place");
        let text = "xin chào\nthế giới\n";
        ui.file("place/a.txt", text);
        ui.storage(&folder);
        ui.explorer_names();
        ui.ws(|ws, _, cx| {
            ws.explorer(cx).update(cx, |e, cx| {
                let item = e.selection_of("a.txt");
                e.open(&item, cx);
            });
        });
        ui.wait_for("field: ");
        ui.cx
            .write_to_clipboard(gpui::ClipboardItem::new_string(String::new()));
        ui.right_click("field: ");
        assert!(!ui.shows("text-copy"), "nothing is chosen");
        ui.click("text-copy-all");
        assert_eq!(
            ui.cx.read_from_clipboard().and_then(|c| c.text()).as_deref(),
            Some(text)
        );
        assert!(ui.dialog_open(), "the file is still shown");
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
        assert!(!ui.shows("open-stored"), "offered with no storage of the user's");
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

scenario!(
    /// A folder on this computer added as a storage is enough for the Archive page to
    /// offer opening from a storage, and an archive in it opens from there.
    a_folder_added_offers_opening_from_it,
    |ui| {
        ui.wait_for("open-archive");
        assert!(!ui.shows("open-stored"), "offered before any storage was added");
        let here = ui.work.join("here");
        std::fs::create_dir_all(&here).expect("the folder");
        std::fs::copy(crate::ui_tests::fixture("sample.7z"), here.join("sample.7z"))
            .expect("the fixture");
        ui.storage(&here);
        ui.click("nav-archive");
        ui.click("open-stored");
        ui.wait_for("pick: sample.7z");
        ui.click("pick: sample.7z");
        ui.wait_for_no_dialog();
        ui.wait_until("the archive read", |ui| !ui.archive_names().is_empty());
    }
);

/// What says it is the user: the password `right`, typed in packwing.
struct Confirms;

impl crate::confirm::Confirmer for Confirms {
    fn how(&self) -> Option<crate::confirm::How> {
        Some(crate::confirm::How::Password)
    }
    fn ask(&self, _: &str, _: &str) -> Result<bool, String> {
        Err("it is typed here".into())
    }
    fn check_password(&self, password: &str) -> Result<bool, String> {
        Ok(password == "right")
    }
}

scenario!(
    /// A storage that asks for this computer's password first is offered locked:
    /// chosen in a dialog, it asks for the password there, and shows its archives
    /// once it is right. Left long enough, it asks again in New archive's Save in.
    a_locked_storage_asks_in_the_dialog,
    |ui| {
        ui.cx.update(|cx| {
            cx.set_global(crate::confirm::ConfirmWith(std::sync::Arc::new(Confirms)));
        });
        let far = ui.work.join("far");
        std::fs::create_dir_all(&far).expect("the folder");
        std::fs::copy(crate::ui_tests::fixture("sample.7z"), far.join("sample.7z"))
            .expect("the fixture");
        ui.ws(|ws, window, cx| ws.open_add_storage(window, cx));
        ui.wait_for("kind-0");
        ui.click("kind-0");
        ui.type_in("Name", "Far");
        ui.type_in("Folder", &far.display().to_string());
        ui.click("storage-asks-first");
        ui.click("storage-add");
        ui.wait_for_no_dialog();
        ui.click("nav-archive");
        ui.click("open-stored");
        ui.click("place: Far");
        ui.wait_for("storage-unlock");
        assert!(!ui.shows("pick: sample.7z"), "shown while locked");
        ui.type_in("Password", "wrong");
        ui.click("storage-unlock");
        assert!(ui.shows("storage-unlock"), "opened with the wrong password");
        ui.replace_in("Password", "right");
        ui.click("storage-unlock");
        ui.wait_for("pick: sample.7z");
        ui.click("pick: sample.7z");
        ui.wait_for_no_dialog();
        ui.wait_until("the archive read", |ui| !ui.archive_names().is_empty());

        // Left long enough, it shuts again; Save in asks for it as well.
        let shut = ui.ws(|ws, _, cx| ws.storages_page().update(cx, |p, cx| p.leave_for_long(cx)));
        assert!(shut);
        let paths = vec![ui.file("a.txt", "a")];
        ui.ws(|ws, window, cx| ws.open_create(paths, window, cx));
        ui.wait_for("create");
        ui.click("change-storage");
        ui.click("place: Far");
        ui.wait_for("storage-unlock");
        ui.click("save-here");
        assert!(ui.shows("storage-unlock"), "saved on it while locked");
        ui.type_in("Password", "right");
        ui.click("storage-unlock");
        ui.wait_until("the storage open", |ui| !ui.shows("storage-unlock"));
        ui.click("save-here");
        ui.click("create");
        ui.wait_for_no_dialog();
        ui.wait_until("the archive there", |_| far.join("a.zip").exists());
    }
);
