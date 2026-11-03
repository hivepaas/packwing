//! Making archives through New archive, and from the right-click menu.

use super::{Ui, listed, scenario};

/// New archive open on the one file `a.txt`, with Advanced shown.
fn new_archive(ui: &mut Ui) -> std::path::PathBuf {
    let file = ui.file("a.txt", "held");
    let paths = vec![file];
    ui.ws(|ws, window, cx| ws.open_create(paths, window, cx));
    ui.wait_for("create");
    ui.work.clone()
}

scenario!(
    /// The dialog's defaults make a ZIP archive beside the file.
    a_zip_archive,
    |ui| {
        let work = new_archive(ui);
        ui.click("create");
        ui.wait_for_no_dialog();
        assert_eq!(listed(&work.join("a.zip"), None), ["a.txt"]);
    }
);

scenario!(
    /// With a password, the archive opens with it and not without.
    a_zip_archive_with_a_password,
    |ui| {
        let work = new_archive(ui);
        ui.click("advanced");
        ui.click("protect");
        ui.type_in("Password", "correct horse battery");
        ui.click("create");
        ui.wait_for_no_dialog();
        let made = work.join("a.zip");
        assert_eq!(listed(&made, Some("correct horse battery")), ["a.txt"]);
        let out = work.join("out");
        let keys = packwing_core::Keys::default();
        let control = packwing_core::Control::default();
        assert!(packwing_core::extract(&made, &out, &keys, &control).is_err());
    }
);

scenario!(
    /// A ZIP archive encrypted whole with age instead is saved as .zip.age, and opens
    /// with the passphrase alone.
    a_zip_archive_encrypted_with_age,
    |ui| {
        let work = new_archive(ui);
        ui.click("advanced");
        ui.click("protect");
        ui.click("whole-age");
        ui.type_in("Password", "correct horse battery");
        ui.click("create");
        ui.wait_for_no_dialog();
        assert!(!work.join("a.zip").exists());
        let made = work.join("a.zip.age");
        let control = packwing_core::Control::default();
        let none = packwing_core::Keys::default();
        assert!(packwing_core::list(packwing_core::Source::from(made.as_path()), &none).is_err());
        let keys = packwing_core::Keys {
            passphrase: Some(b"correct horse battery".to_vec()),
            ..packwing_core::Keys::default()
        };
        let out = work.join("out");
        packwing_core::extract(&made, &out, &keys, &control).expect("it opens with the passphrase");
        assert_eq!(std::fs::read_to_string(out.join("a.txt")).expect("a.txt came out"), "held");
    }
);

scenario!(
    /// Asked to protect it, an empty password stops it, and says so.
    no_password_makes_nothing,
    |ui| {
        let work = new_archive(ui);
        ui.click("advanced");
        ui.click("protect");
        ui.click("create");
        ui.cx.run_until_parked();
        assert!(ui.dialog_open());
        assert!(!work.join("a.zip").exists());
    }
);

scenario!(
    /// Suggest fills the field with a passphrase that makes an archive.
    a_suggested_passphrase_works,
    |ui| {
        let work = new_archive(ui);
        ui.click("advanced");
        ui.click("protect");
        ui.click("suggest-password");
        ui.click("create");
        ui.wait_for_no_dialog();
        assert!(work.join("a.zip").exists());
    }
);

scenario!(
    /// Compress to ZIP, from the right-click menu, asks nothing.
    compress_from_the_right_click_menu,
    |ui| {
        let file = ui.file("a.txt", "held");
        let request = crate::right_click::Request {
            action: crate::right_click::Action::CompressZip,
            paths: vec![file],
        };
        ui.ws(|ws, window, cx| ws.start(request, window, cx));
        ui.wait_for_no_dialog();
        assert_eq!(listed(&ui.work.join("a.zip"), None), ["a.txt"]);
    }
);

scenario!(
    /// A file dropped on the window starts a new archive of it.
    a_drop_starts_a_new_archive,
    |ui| {
        let a = ui.file("a.txt", "a");
        ui.ws(|ws, window, cx| ws.dropped(vec![a], window, cx));
        ui.wait_for("create");
        let held = ui.ws(|ws, _, cx| ws.create_dialog_paths(cx));
        assert_eq!(held.len(), 1, "{held:?}");
        ui.click("create");
        ui.wait_for_no_dialog();
        assert_eq!(listed(&ui.work.join("a.zip"), None), ["a.txt"]);
    }
);
