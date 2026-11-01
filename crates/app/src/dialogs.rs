//! The dialogs drawn over the main window: extracting, a new archive, adding
//! to one and a job's progress. Each is a view that says what the user
//! chose through [`DialogEvent`]; the workspace opens and closes them.

mod add;
mod add_storage;
mod chosen;
mod create;
mod delete_items;
mod edit;
mod extract;
mod open_file;
mod password;
mod progress;
mod transfer;
mod view_file;

pub use add::AddDialog;
pub use add_storage::AddStorageDialog;
pub use create::CreateDialog;
pub(crate) use create::{Format, LEVELS, Level, level_name};
pub use delete_items::DeleteDialog;
pub use edit::EditDialog;
pub use extract::ExtractDialog;
pub use open_file::OpenFileDialog;
pub use password::{PasswordDialog, PasswordEvent, asked_for};
pub use progress::ProgressDialog;
pub use transfer::TransferDialog;
pub use view_file::ViewDialog;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use gpui::{Div, FontWeight, MouseButton, SharedString, Stateful, div, prelude::*, px, rgb};
use packwing_core::{Asked, ExtractOptions, Keys, NewArchive, OnExisting, Transfer, Transferred};

use crate::colors::{Tint, colors};
use crate::widgets::{float_shadow, icon, logo, tile};

/// What a job does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verb {
    Extract,
    Create,
    Add,
    /// What an archive holds deleted or renamed.
    Edit,
    /// A file of an archive, copied out and opened.
    Open,
    Test,
    /// Items of a storage copied or moved.
    Copy,
    Move,
}

/// Items to copy or move, and how it went once it is over.
#[derive(Clone)]
pub struct Transferring(pub Transfer, pub Arc<Mutex<Transferred>>);

impl PartialEq for Transferring {
    fn eq(&self, other: &Transferring) -> bool {
        Arc::ptr_eq(&self.1, &other.1)
    }
}

impl std::fmt::Debug for Transferring {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Transferring({:?})", self.0.names)
    }
}

/// Files to add to an archive, through packwing-core.
#[derive(Clone, Debug, PartialEq)]
pub struct Adding {
    pub paths: Vec<PathBuf>,
    pub archive: PathBuf,
    /// The folder of it they go in.
    pub folder: Vec<String>,
    pub on_existing: OnExisting,
}

/// A change to what an archive holds, by paths in it.
#[derive(Clone, Debug, PartialEq)]
pub enum Editing {
    Delete(Vec<Vec<String>>),
    Rename { from: Vec<String>, to: String },
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
/// and adding to them run, through packwing-core; the samples
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
    /// What it extracts of the one archive, by path in it: all of it if empty.
    /// For [`Verb::Open`], the one file.
    pub picked: Vec<Vec<String>>,
    /// How it extracts, if that is what it does.
    pub how: ExtractOptions,
    /// The new archive it makes, if that is what it does.
    pub create: Option<Creating>,
    /// The files it adds, if that is what it does.
    pub add: Option<Adding>,
    /// The archive it changes, the folder shown in it, and how.
    pub edit: Option<(PathBuf, Vec<String>, Editing)>,
    /// The items it copies or moves, if that is what it does.
    pub transfer: Option<Transferring>,
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
            picked: Vec::new(),
            how: ExtractOptions::default(),
            create: None,
            add: None,
            edit: None,
            transfer: None,
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
    /// Items of a storage were copied or moved, and how it went.
    Transferred(Transferred),
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
                        .truncate()
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
