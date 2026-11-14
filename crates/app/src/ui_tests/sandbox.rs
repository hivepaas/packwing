//! The Mac App Store's sandbox, played by `access::fake`: a folder packwing cannot
//! reach is asked for before an archive is read from it or a job writes in it.

use super::{Ui, fixture, listed, scenario};
use crate::access::fake;

/// Extracts the archive open into a new folder beside it, as the user does.
fn extract_beside(ui: &mut Ui) {
    ui.click("extract");
    ui.wait_for("dest-new");
    ui.click("dest-new");
    ui.click("extract");
}

scenario!(
    /// An archive out of reach (one of Recent, from a run that did not keep it) asks
    /// for its folder, and is read once given; extracting beside it then asks nothing.
    an_archive_out_of_reach_asks_for_its_folder,
    |ui| {
        let archive = ui.copy_of("sample.7z");
        let held = listed(&archive, None);
        fake::start(vec![ui.work.clone()], Vec::new());
        fake::answer(Some(ui.work.clone()));
        ui.open(&archive);
        assert_eq!(ui.archive_failed(), None);
        assert_eq!(fake::asked(), std::slice::from_ref(&ui.work));
        extract_beside(ui);
        ui.wait_for_no_dialog();
        let out = ui.work.join("sample");
        for name in held.iter().filter(|n| !n.ends_with('/')) {
            assert!(out.join(name).exists(), "{name} did not come out in {out:?}");
        }
        assert_eq!(fake::asked().len(), 1);
    }
);

scenario!(
    /// An archive opened from Finder is read, but its folder is not packwing's to write
    /// in: extracting beside it asks for the folder, and when that is cancelled, the job
    /// says why it did not start, and nothing comes out.
    extracting_where_packwing_may_not_write_asks_first,
    |ui| {
        let archive = ui.copy_of("sample.7z");
        fake::start(vec![ui.work.clone()], vec![archive.clone()]);
        ui.open(&archive);
        assert_eq!(ui.archive_failed(), None);
        assert!(fake::asked().is_empty());
        fake::answer(None);
        extract_beside(ui);
        ui.wait_until("the job refused", |ui| {
            ui.ws(|ws, _, cx| ws.job_failed(cx)).is_some()
        });
        let why = ui.ws(|ws, _, cx| ws.job_failed(cx)).unwrap_or_default();
        assert!(why.contains("was not given"), "{why}");
        assert_eq!(fake::asked(), std::slice::from_ref(&ui.work));
        assert!(!ui.work.join("sample").exists());
    }
);

scenario!(
    /// The first part of an archive in parts, opened from Finder, asks for its folder,
    /// which holds the others, and is read whole once given.
    an_archive_in_parts_asks_for_its_folder,
    |ui| {
        let folder = ui.work.join("parts");
        std::fs::create_dir_all(&folder).expect("the folder");
        let name = |n: u32| format!("test_rar_multivolume_multiple_files.part{n}.rar");
        for n in 1..=6 {
            std::fs::copy(fixture(&name(n)), folder.join(name(n))).expect("a part");
        }
        let first = folder.join(name(1));
        fake::start(vec![folder.clone()], vec![first.clone()]);
        fake::answer(Some(folder.clone()));
        ui.open(&first);
        assert_eq!(ui.archive_failed(), None);
        assert_eq!(fake::asked(), [folder]);
    }
);

scenario!(
    /// A folder the sandbox keeps packwing from shows why, with Allow access…, which
    /// asks for it and shows it once given.
    a_folder_out_of_reach_is_asked_for_where_it_is_shown,
    |ui| {
        use std::os::unix::fs::PermissionsExt as _;
        let folder = ui.work.join("locked");
        ui.file("locked/inside.txt", "inside");
        fake::start(vec![folder.clone()], Vec::new());
        // As the sandbox does, the folder cannot be listed until it is given.
        let mode = |m| std::fs::set_permissions(&folder, std::fs::Permissions::from_mode(m));
        mode(0o000).expect("locked");
        ui.storage(&folder);
        ui.wait_for("allow-access");
        assert!(ui.explorer_names().is_empty());
        mode(0o755).expect("unlocked");
        fake::answer(Some(folder.clone()));
        ui.click("allow-access");
        ui.wait_until("the folder listed", |ui| ui.explorer_names() == ["inside.txt"]);
        assert_eq!(fake::asked(), [folder]);
    }
);
