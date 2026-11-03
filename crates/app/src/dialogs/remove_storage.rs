//! Asked before a storage is taken off the list: its files stay where they are,
//! and what it signs in with leaves the keychain.

use gpui::{App, Context, Entity, EventEmitter, Render, Window, div, prelude::*, rgb};
use packwing_i18n::tr;

use super::{DialogEvent, footer, header};
use crate::colors::colors;
use crate::pages::storages::StoragesPage;
use crate::prefs::KeptPlace;
use crate::text_size::ui;
use crate::widgets::{Fill, Size, button, icon_button, tonal_tint};

pub struct RemoveStorageDialog {
    page: Entity<StoragesPage>,
    at: usize,
    name: String,
    /// Across a network: its secrets are in the keychain.
    remote: bool,
}

impl EventEmitter<DialogEvent> for RemoveStorageDialog {}

impl RemoveStorageDialog {
    /// For the storage at `at` on `page`, if there is one.
    pub fn new(page: Entity<StoragesPage>, at: usize, cx: &App) -> Option<Self> {
        let kept = page.read(cx).storage(at)?;
        let (name, remote) = (
            kept.name.clone(),
            matches!(kept.place, KeptPlace::Other { .. }),
        );
        Some(RemoveStorageDialog {
            page,
            at,
            name,
            remote,
        })
    }
}

impl Render for RemoveStorageDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let close = icon_button("close", "x", 44.)
            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close)));
        let note = if self.remote {
            tr!(
                "Its files stay where they are. What packwing signs in to it with is taken out of the system’s keychain."
            )
        } else {
            tr!("The folder and its files stay where they are.")
        };
        super::card()
            .w(ui(520.))
            .child(header(
                "warning",
                tonal_tint(),
                tr!("Remove “{name}” from the list?", name = self.name),
                tr!("It can be added again at any time."),
                Some(close),
            ))
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
                            "storage-remove-yes",
                            tr!("Remove"),
                            Fill::Accent,
                            Size::Medium,
                        )
                        .on_click(cx.listener(|this, _, _, cx| {
                            let (at, name) = (this.at, this.name.clone());
                            this.page.update(cx, |p, cx| p.remove_named(at, &name, cx));
                            cx.emit(DialogEvent::Close);
                        })),
                    ),
            )
    }
}
