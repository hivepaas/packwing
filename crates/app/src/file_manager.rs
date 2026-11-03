//! The system's file manager: Finder, File Explorer, or the one a Linux desktop has.
//! An item is shown chosen in its folder; a folder is opened in it. Only what is on
//! this computer: a storage across a network has nothing there to show.

use std::path::Path;

use gpui::App;
use packwing_i18n::tr;

/// What showing an item in it is called, by the system's own name for it.
pub fn show_label() -> String {
    if cfg!(target_os = "macos") {
        tr!("Show in Finder")
    } else if cfg!(windows) {
        tr!("Show in File Explorer")
    } else {
        tr!("Show in the file manager")
    }
}

/// What opening a folder in it is called.
pub fn open_label() -> String {
    if cfg!(target_os = "macos") {
        tr!("Open in Finder")
    } else if cfg!(windows) {
        tr!("Open in File Explorer")
    } else {
        tr!("Open in the file manager")
    }
}

/// Shows the file or folder at `path` chosen in its folder.
pub fn show(path: &Path, cx: &App) {
    #[cfg(all(test, feature = "ui-tests"))]
    asked("show", path, cx);
    #[cfg(not(all(test, feature = "ui-tests")))]
    cx.reveal_path(path);
}

/// Opens the folder at `path`.
pub fn open(path: &Path, cx: &App) {
    #[cfg(all(test, feature = "ui-tests"))]
    asked("open", path, cx);
    #[cfg(not(all(test, feature = "ui-tests")))]
    cx.open_with_system(path);
}

/// A scenario has no file manager: what it was asked is kept, to look at.
#[cfg(all(test, feature = "ui-tests"))]
static ASKED: std::sync::Mutex<Vec<(&str, std::path::PathBuf)>> = std::sync::Mutex::new(Vec::new());

#[cfg(all(test, feature = "ui-tests"))]
fn asked(what: &'static str, path: &Path, _: &App) {
    ASKED
        .lock()
        .expect("not poisoned")
        .push((what, path.to_path_buf()));
}

/// What it was asked, as `("show" | "open", path)`, since the scenario started.
#[cfg(all(test, feature = "ui-tests"))]
pub fn asked_for_test() -> Vec<(&'static str, std::path::PathBuf)> {
    ASKED.lock().expect("not poisoned").clone()
}
