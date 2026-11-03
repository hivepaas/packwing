//! The dialogs drawn over the main window: extracting, a new archive, adding
//! to one and a job's progress. Each is a view that says what the user
//! chose through [`DialogEvent`]; the workspace opens and closes them.

mod add;
mod add_storage;
mod checksum;
mod chosen;
mod create;
mod delete_items;
mod edit;
mod extract;
mod open_file;
mod open_stored;
mod password;
mod places;
mod progress;
mod remove_storage;
mod sign;
mod transfer;
mod view_file;

pub use add::AddDialog;
pub use add_storage::AddStorageDialog;
pub use checksum::ChecksumDialog;
pub use create::CreateDialog;
pub(crate) use create::{Format, LEVELS, Level, level_name};
pub use delete_items::DeleteDialog;
pub use edit::EditDialog;
pub use extract::ExtractDialog;
pub use open_file::OpenFileDialog;
pub use open_stored::OpenStoredDialog;
pub use password::{PasswordDialog, PasswordEvent, asked_for};
pub use progress::{ProgressDialog, State};
pub use remove_storage::RemoveStorageDialog;
pub use sign::SignDialog;
pub use transfer::TransferDialog;
pub use view_file::ViewDialog;

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use gpui::{Div, FontWeight, MouseButton, SharedString, Stateful, div, prelude::*, rgb};
use packwing_core::{
    Asked, ExtractOptions, Keys, NewArchive, OnExisting, SigningKey, Source, Storage, Transfer,
    Transferred,
};
use packwing_i18n::tr;

use crate::colors::{Tint, colors};
use crate::explorer::Place;
use crate::text_size::ui;
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
    /// A file signed.
    Sign,
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

/// A file to sign: its storage, its path there, and the key.
#[derive(Clone)]
pub struct Signing(pub Arc<dyn Storage>, pub Vec<String>, pub SigningKey);

impl PartialEq for Signing {
    fn eq(&self, other: &Signing) -> bool {
        Arc::ptr_eq(&self.0, &other.0) && self.1 == other.1 && self.2 == other.2
    }
}

impl std::fmt::Debug for Signing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Signing({:?} with {:?})", self.1, self.2)
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
    /// See [`packwing_core::NewArchive::stored_extensions`].
    pub stored_extensions: Vec<String>,
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
    pub to: Saving,
    pub how: NewArchive,
}

/// Where a new archive goes: what it is called and where, or the first free name
/// after that.
#[derive(Clone, Debug, PartialEq)]
pub enum Saving {
    Here(PathBuf),
    On(SavingOn),
}

/// A new archive's place on a storage.
#[derive(Clone)]
pub struct SavingOn {
    pub place: Place,
    pub folder: Vec<String>,
    pub name: String,
    /// It has been made, and is on its way there.
    pub uploading: Arc<AtomicBool>,
}

impl PartialEq for SavingOn {
    fn eq(&self, other: &SavingOn) -> bool {
        Arc::ptr_eq(&self.uploading, &other.uploading)
    }
}

impl std::fmt::Debug for SavingOn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SavingOn({} {:?})", self.place.name, self.name)
    }
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
    pub work: Vec<(Source, Option<PathBuf>)>,
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
    /// The file it signs, if that is what it does.
    pub sign: Option<Signing>,
}

impl Job {
    /// Why it cannot start while `other` runs, if it cannot: the two would write the
    /// same new archive, or extract into the same folder.
    pub fn clash(&self, other: &Job) -> Option<String> {
        if let (Some(a), Some(b)) = (&self.create, &other.create) {
            let same = match (&a.to, &b.to) {
                (Saving::Here(x), Saving::Here(y)) => x == y,
                (Saving::On(x), Saving::On(y)) => {
                    (Arc::ptr_eq(&x.place.storage, &y.place.storage)
                        || x.place.name == y.place.name)
                        && x.folder == y.folder
                        && x.name == y.name
                }
                _ => false,
            };
            return same.then(|| {
                tr!(
                    "Another job is making {name} there now: give this one another name, or \
                     wait for it to end.",
                    name = self.name
                )
            });
        }
        if self.verb != Verb::Extract || other.verb != Verb::Extract {
            return None;
        }
        let folders =
            |j: &Job| -> Vec<PathBuf> { j.work.iter().filter_map(|(_, to)| to.clone()).collect() };
        let theirs = folders(other);
        folders(self)
            .into_iter()
            .find(|to| theirs.contains(to))
            .map(|to| {
                tr!(
                    "Another job is extracting into {folder} now: choose another folder, or \
                     wait for it to end.",
                    folder = to.display()
                )
            })
    }

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
            sign: None,
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
    /// A file was signed.
    Signed,
    /// A file was kept beside one shown, and the dialog stays open.
    Saved,
    /// The job goes on behind, on the Jobs page: its dialog closes, not it.
    Behind,
    /// An archive on a storage was chosen, to open.
    Open(Source),
    /// A license key is to be entered: Settings › License.
    ShowLicense,
    /// A new archive went in this folder of this storage: shown, if `show`.
    Stored {
        place: Place,
        folder: Vec<String>,
        show: bool,
    },
}

/// The card a dialog is drawn on.
pub fn card() -> Div {
    let c = colors();
    div()
        .w(ui(640.))
        .max_w_full()
        .max_h_full()
        .flex()
        .flex_col()
        .gap(ui(20.))
        .p(ui(28.))
        .rounded(ui(28.))
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
        .gap(ui(14.))
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
                        .text_size(ui(22.))
                        .font_weight(FontWeight::BOLD)
                        .truncate()
                        .child(title.into()),
                )
                .child(
                    div()
                        .text_size(ui(14.))
                        .text_color(rgb(colors().muted))
                        .truncate()
                        .child(detail.into()),
                ),
        )
        .children(close)
}

/// A dialog's row of buttons, the main one last.
pub fn footer() -> Div {
    div().flex().items_center().gap(ui(10.))
}
