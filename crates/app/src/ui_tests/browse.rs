//! The Browse page: its folders, the archives opened before, and what is dropped.

use super::scenario;

scenario!(
    /// An archive opened shows on Recent after a restart, and opens again from there.
    an_archive_opened_is_on_recent,
    |ui| {
        let archive = ui.copy_of("sample.7z");
        ui.open(&archive);
        ui.start();
        ui.click("nav-browse");
        assert!(ui.shows("drop-hint"));
        ui.click("tab-recent");
        ui.wait_for("recent: sample.7z");
        assert!(ui.archive_names().is_empty());
        ui.click("recent: sample.7z");
        ui.wait_until("the archive read", |ui| !ui.archive_names().is_empty());
        assert_eq!(ui.archive_failed(), None);
    }
);

scenario!(
    /// Browse goes back to the folder it was left on, and Home leads home.
    browse_is_left_where_it_was,
    |ui| {
        let docs = ui.work.join("docs");
        ui.file("docs/a.txt", "a");
        ui.click("nav-browse");
        let at = docs.clone();
        ui.ws(|ws, _, cx| ws.browse_page().update(cx, |b, cx| b.go_to(&at, cx)));
        assert_eq!(ui.browsed().as_deref(), Some(docs.as_path()));
        ui.start();
        assert_eq!(ui.browsed().as_deref(), Some(docs.as_path()));
        ui.click("nav-browse");
        ui.click("go-home");
        assert_eq!(ui.browsed(), dirs::home_dir());
    }
);

scenario!(
    /// An archive dropped on the window opens.
    a_dropped_archive_opens,
    |ui| {
        let archive = ui.copy_of("sample.7z");
        ui.ws(|ws, window, cx| ws.dropped(vec![archive], window, cx));
        ui.wait_until("the archive read", |ui| !ui.archive_names().is_empty());
        assert_eq!(ui.archive_failed(), None);
    }
);
