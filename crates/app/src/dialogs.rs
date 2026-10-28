//! The dialogs drawn over the main window: extracting, a new archive, a job's
//! progress and a damaged archive. Each is a view that says what the user
//! chose through [`DialogEvent`]; the workspace opens and closes them.

mod create;
mod damaged;
mod extract;
mod progress;

pub use create::CreateDialog;
pub(crate) use create::{LEVELS, Level, PURPOSES, Purpose};
pub use damaged::DamagedDialog;
pub use extract::{ENCODINGS, ExtractDialog};
pub use progress::ProgressDialog;

use gpui::{Div, FontWeight, MouseButton, SharedString, Stateful, div, prelude::*, px, rgb};

use crate::colors::{Tint, colors};
use crate::widgets::{float_shadow, icon, tile};

/// What a job does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verb {
    Extract,
    Create,
    Repair,
    Test,
}

/// A job the user started. Nothing runs yet: the progress dialog counts up
/// at a made-up speed.
#[derive(Clone, Debug, PartialEq)]
pub struct Job {
    pub verb: Verb,
    /// The archive.
    pub name: SharedString,
    /// Where the files (or the new archive) go; none for a test.
    pub to: Option<SharedString>,
    pub files: u64,
    pub bytes: u64,
}

/// What the user chose in a dialog.
pub enum DialogEvent {
    Close,
    Start(Job),
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
        .child(tile(48., 16., tint).child(icon(icon_name, 24., tint.ink)))
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
