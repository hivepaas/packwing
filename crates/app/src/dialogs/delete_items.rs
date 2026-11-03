//! Asked before items of a storage are deleted: to the system's Trash where
//! there is one, and for good where there is not.

use gpui::{Context, Entity, EventEmitter, Render, Window, div, prelude::*, rgb};
use packwing_i18n::{tr, tr_n};

use super::{DialogEvent, footer, header};
use crate::colors::colors;
use crate::explorer::Explorer;
use crate::text_size::ui;
use crate::widgets::{Fill, Size, button, icon_button, tonal_tint};

/// The most names listed.
const MOST_LISTED: usize = 5;

pub struct DeleteDialog {
    names: Vec<String>,
    storage: String,
    has_trash: bool,
    explorer: Entity<Explorer>,
}

impl EventEmitter<DialogEvent> for DeleteDialog {}

impl DeleteDialog {
    pub fn new(names: Vec<String>, explorer: Entity<Explorer>, cx: &Context<Self>) -> DeleteDialog {
        let place = explorer.read(cx).place();
        let storage = place.map_or(String::new(), |p| p.name.clone());
        let has_trash = place.is_some_and(|p| p.storage.has_trash());
        DeleteDialog {
            names,
            storage,
            has_trash,
            explorer,
        }
    }
}

impl Render for DeleteDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let n = self.names.len() as u64;
        let what = match self.names.as_slice() {
            [one] => one.clone(),
            _ => tr_n!(n, "{n} item", "{n} items"),
        };
        let title = if self.has_trash {
            tr!("Move {what} to the Trash?", what = what)
        } else {
            tr!("Delete {what}?", what = what)
        };
        let note = if self.has_trash {
            tr!("It can be put back from the system’s Trash.")
        } else {
            tr!("This cannot be undone from packwing.")
        };
        let left = self.names.len().saturating_sub(MOST_LISTED) as u64;
        let close = icon_button("close", "x", 44.)
            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close)));
        super::card()
            .w(ui(520.))
            .child(header(
                "warning",
                tonal_tint(),
                title,
                self.storage.clone(),
                Some(close),
            ))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(ui(4.))
                    .px(ui(14.))
                    .py(ui(12.))
                    .rounded(ui(18.))
                    .bg(rgb(c.well))
                    .text_size(ui(14.))
                    .children(
                        self.names
                            .iter()
                            .take(MOST_LISTED)
                            .map(|n| div().child(n.clone())),
                    )
                    .when(left > 0, |d| {
                        d.child(div().text_color(rgb(c.muted)).child(tr_n!(
                            left,
                            "and {n} more",
                            "and {n} more"
                        )))
                    }),
            )
            .child(
                div()
                    .text_size(ui(14.))
                    .text_color(rgb(c.muted))
                    .child(note),
            )
            .child(
                footer()
                    .justify_end()
                    .child(
                        button("cancel", tr!("Cancel"), Fill::Outline, Size::Medium)
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close))),
                    )
                    .child(
                        button(
                            "delete",
                            if self.has_trash {
                                tr!("Move to Trash")
                            } else {
                                tr!("Delete")
                            },
                            Fill::Accent,
                            Size::Medium,
                        )
                        .on_click(cx.listener(|this, _, _, cx| {
                            let names = std::mem::take(&mut this.names);
                            this.explorer.update(cx, |e, cx| e.delete(names, cx));
                            cx.emit(DialogEvent::Close);
                        })),
                    ),
            )
    }
}
