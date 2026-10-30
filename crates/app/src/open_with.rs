//! packwing in the system's Open With, for the archives the menu's items
//! are for, through context-menu-rs. Chosen there, packwing opens the
//! archive as Open with packwing does: `packwing --open a.zip`.
//!
//! On Windows packwing puts itself there when it starts, in the background,
//! when it is not there as it would put itself: packwing may have moved, or
//! been updated. It is offered, not chosen: the user makes it the app that
//! opens archives at a double click, in Open With or in Settings. On Linux
//! the .deb's launcher does it (packaging/linux/packwing.desktop); on macOS
//! the app's bundle will, in its Info.plist, and the files will come as an
//! event, not on the command line.

use std::path::Path;

use context_menu::open_with::Handler;
use gpui::{App, AppContext as _};

use crate::right_click::{ARCHIVES, Action, ID};

/// packwing in Open With, running `program`.
pub fn handler(program: &Path) -> Handler {
    let handler = Handler::new(ID, "packwing", program)
        .arg(Action::Open.option())
        .extensions(ARCHIVES);
    if cfg!(windows) {
        // The program's own icon, for the archives the user makes it open.
        handler.icon(format!("{},0", program.display()))
    } else {
        handler
    }
}

/// Puts packwing in Open With, in the background, when it is not there as
/// it would be: on Windows, where no package does it.
pub fn keep_current(cx: &App) {
    if !cfg!(windows) {
        return;
    }
    let Ok(program) = std::env::current_exe() else {
        return;
    };
    cx.background_spawn(async move {
        let handler = handler(&program);
        let done = match context_menu::open_with::is_current(&handler) {
            Ok(true) => Ok(()),
            Ok(false) => context_menu::open_with::install(&handler).map(|_| ()),
            Err(e) => Err(e),
        };
        if let Err(e) = done {
            eprintln!("packwing: cannot put itself in Open With: {e}");
        }
    })
    .detach();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_with_opens_what_the_items_are_for() {
        let program = std::env::current_exe().expect("the test has a program");
        let h = handler(&program);
        assert!(h.validate().is_ok(), "{h:?}");
        assert_eq!(h.args, [Action::Open.option()]);
        assert_eq!(h.extensions, ARCHIVES);
        assert_eq!(h.icon.is_some(), cfg!(windows));
        // What it runs is what Open with packwing's item runs.
        let r = crate::right_click::Request::parse(&h.args[0], [program.into_os_string()])
            .expect("the option is an item's");
        assert_eq!(r.action, Action::Open);
    }
}
