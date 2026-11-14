//! packwing in the system's Open With, for the archives the menu's items
//! are for, through context-menu-rs. Chosen there, packwing opens the
//! archive as Open with packwing does: `packwing --open a.zip`.
//!
//! On Windows packwing puts itself there when it starts, in the background,
//! when it is not there as it would put itself: packwing may have moved, or
//! been updated. It is offered, not chosen: the user makes it the app that
//! opens archives at a double click, in Open With or in Settings. On Linux
//! the .deb's launcher does it (packaging/linux/packwing.desktop); on macOS
//! the app's bundle does, in its Info.plist (packaging/macos/Info.plist), and
//! the files come as an event, not on the command line ([`path_of_url`]).

use std::path::{Path, PathBuf};

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

/// The file a URL that macOS asks packwing to open names: `file:///Users/a/My%20b.zip`,
/// percent-decoded, as LaunchServices sends what is chosen in Finder or dropped on the
/// Dock's icon. None for any other URL, or one whose escapes are not a path's.
pub fn path_of_url(url: &str) -> Option<PathBuf> {
    let rest = url.strip_prefix("file://")?;
    // A file URL's host is empty, or this computer's.
    let path = rest.strip_prefix("localhost").unwrap_or(rest);
    if !path.starts_with('/') {
        return None;
    }
    let mut bytes = Vec::with_capacity(path.len());
    let mut it = path.bytes();
    while let Some(b) = it.next() {
        if b == b'%' {
            let hex = [it.next()?, it.next()?];
            let hex = std::str::from_utf8(&hex).ok()?;
            bytes.push(u8::from_str_radix(hex, 16).ok()?);
        } else {
            bytes.push(b);
        }
    }
    // A name is any bytes on Unix; elsewhere, text.
    #[cfg(unix)]
    let path = <std::ffi::OsString as std::os::unix::ffi::OsStringExt>::from_vec(bytes);
    #[cfg(not(unix))]
    let path = String::from_utf8(bytes).ok()?;
    Some(PathBuf::from(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_url_is_the_path_it_names() {
        let path = |u| path_of_url(u).map(|p| p.display().to_string());
        assert_eq!(
            path("file:///Users/a/My%20b.zip").as_deref(),
            Some("/Users/a/My b.zip")
        );
        assert_eq!(
            path("file://localhost/tmp/x.7z").as_deref(),
            Some("/tmp/x.7z")
        );
        // Vietnamese, in NFD as macOS names it, escaped byte by byte.
        assert_eq!(
            path("file:///tmp/Ti%E1%BA%BFng%20Vi%E1%BB%87t.rar").as_deref(),
            Some("/tmp/Tiếng Việt.rar")
        );
        assert_eq!(
            path("file:///tmp/100%25.tar").as_deref(),
            Some("/tmp/100%.tar")
        );
        // What is not a file on this computer, or not escaped right.
        assert_eq!(path("https://example.com/a.zip"), None);
        assert_eq!(path("file://server/share/a.zip"), None);
        assert_eq!(path("file:///tmp/a%2"), None);
        assert_eq!(path("file:///tmp/a%zz.zip"), None);
    }

    /// Finder offers packwing in Open With for what Info.plist declares, by the last
    /// part of a name: an archive cut in parts as `.001`.
    #[test]
    fn the_macos_bundle_opens_what_the_items_are_for() {
        let info = include_str!("../../../packaging/macos/Info.plist");
        let declared = info
            .split("<key>CFBundleTypeExtensions</key>")
            .nth(1)
            .and_then(|rest| rest.split("</array>").next())
            .expect("packaging/macos/Info.plist declares CFBundleTypeExtensions");
        let declared: Vec<&str> = declared
            .split("<string>")
            .skip(1)
            .filter_map(|s| s.split("</string>").next())
            .collect();
        let mut wanted: Vec<&str> = Vec::new();
        for e in ARCHIVES {
            let last = e.rsplit('.').next().unwrap_or(e);
            if !wanted.contains(&last) {
                wanted.push(last);
            }
        }
        assert_eq!(
            declared, wanted,
            "packaging/macos/Info.plist's CFBundleTypeExtensions are not ARCHIVES' last parts, in order"
        );
    }

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
