//! The dialogs drawn over the main window: extracting, a new archive, adding
//! to one, a job's progress and a damaged archive. Each is a view that says what the user
//! chose through [`DialogEvent`]; the workspace opens and closes them.

mod add;
mod create;
mod damaged;
mod extract;
mod password;
mod progress;

pub use add::AddDialog;
pub use create::CreateDialog;
pub(crate) use create::{Format, LEVELS, Level};
pub use damaged::DamagedDialog;
pub use extract::{ENCODINGS, ExtractDialog};
pub use password::{PasswordDialog, PasswordEvent, asked_for};
pub use progress::ProgressDialog;

use std::path::PathBuf;

use gpui::{Div, FontWeight, MouseButton, SharedString, Stateful, div, prelude::*, px, rgb};
use packwing_core::{Asked, Keys, NewArchive};

use crate::colors::{Tint, colors};
use crate::widgets::{float_shadow, icon, logo, tile};

/// What a job does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verb {
    Extract,
    Create,
    Add,
    Repair,
    Test,
}

/// Files to add to an archive, through packwing-core.
#[derive(Clone, Debug, PartialEq)]
pub struct Adding {
    pub paths: Vec<PathBuf>,
    pub archive: PathBuf,
    /// The folder of it they go in.
    pub folder: Vec<String>,
}

/// A new archive to make, through packwing-core.
#[derive(Clone, Debug, PartialEq)]
pub struct Creating {
    pub paths: Vec<PathBuf>,
    /// What it is called and where, or the first free name after that.
    pub archive: PathBuf,
    pub how: NewArchive,
}

/// A job the user started. Extracting and testing archives, making new ones
/// and adding to them run, through packwing-core; the rest (a repair, and the samples)
/// count up at a made-up speed.
#[derive(Clone, Debug, PartialEq)]
pub struct Job {
    pub verb: Verb,
    /// The archive.
    pub name: SharedString,
    /// Where the files (or the new archive) go, as the user knows the place;
    /// none for a test.
    pub to: Option<SharedString>,
    /// What it holds, as far as is known before it runs: 0 if not.
    pub files: u64,
    pub bytes: u64,
    /// The archives it reads, one after another, and where each one's files
    /// go (none for a test). Empty for a job that does not run.
    pub work: Vec<(PathBuf, Option<PathBuf>)>,
    /// What opens the archive: the passphrase and password it was opened
    /// with, or one typed in Extract.
    pub keys: Keys,
    /// The new archive it makes, if that is what it does.
    pub create: Option<Creating>,
    /// The files it adds, if that is what it does.
    pub add: Option<Adding>,
}

impl Job {
    /// A job that counts up at a made-up speed.
    pub fn shown(
        verb: Verb,
        name: SharedString,
        to: Option<SharedString>,
        files: u64,
        bytes: u64,
    ) -> Job {
        Job {
            verb,
            name,
            to,
            files,
            bytes,
            work: Vec::new(),
            keys: Keys::default(),
            create: None,
            add: None,
        }
    }
}

/// What the user chose in a dialog.
pub enum DialogEvent {
    Close,
    Start(Job),
    /// A job stopped for a passphrase or password (`Asked`): asked for, it
    /// runs again with it. The text says why it is asked again, when the
    /// one given was wrong.
    Ask(Job, Asked, Option<String>),
    /// Files went in the archive at this path, in this folder: it is read
    /// again, with these keys.
    Added(PathBuf, Vec<String>, Keys),
}

/// The card a dialog is drawn on.
pub fn card() -> Div {
    let c = colors();
    div()
        .w(px(640.))
        .max_w_full()
        .max_h_full()
        .flex()
        .flex_col()
        .gap(px(20.))
        .p(px(28.))
        .rounded(px(28.))
        .bg(rgb(c.surface))
        .text_color(rgb(c.text))
        .shadow(float_shadow(12., 40.))
        // A press on the card stays in it: the shade around it closes nothing.
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
}

/// A dialog's header: its icon, its title and the line under it, and a close
/// button if it has one.
pub fn header(
    icon_name: &str,
    tint: Tint,
    title: impl Into<SharedString>,
    detail: impl Into<SharedString>,
    close: Option<Stateful<Div>>,
) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(14.))
        .child(tile(48., 16., tint).child(if icon_name == "logo" {
            logo(24., tint).into_any_element()
        } else {
            icon(icon_name, 24., tint.ink).into_any_element()
        }))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .child(
                    div()
                        .text_size(px(22.))
                        .font_weight(FontWeight::BOLD)
                        .child(title.into()),
                )
                .child(
                    div()
                        .text_size(px(14.))
                        .text_color(rgb(colors().muted))
                        .truncate()
                        .child(detail.into()),
                ),
        )
        .children(close)
}

/// A dialog's row of buttons, the main one last.
pub fn footer() -> Div {
    div().flex().items_center().gap(px(10.))
}
