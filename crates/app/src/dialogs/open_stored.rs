//! Opening an archive on a storage: the storages beside a picker showing their
//! folders and archives. A click on an archive opens it.

use gpui::{Context, Entity, EventEmitter, Render, Subscription, Window, div, prelude::*, px, rgb};
use packwing_i18n::tr;

use super::{DialogEvent, footer, header, places};
use crate::colors::colors;
use crate::explorer::{Explorer, ExplorerEvent, Place};
use crate::pages::storages::StoragesPage;
use crate::widgets::{Fill, Size, button, icon_button, tonal_tint};

pub struct OpenStoredDialog {
    places: Vec<Place>,
    chosen: usize,
    picker: Entity<Explorer>,
    storages: Entity<StoragesPage>,
    _picker_changed: Subscription,
    _picked: Subscription,
    _lock_changed: Subscription,
}

impl EventEmitter<DialogEvent> for OpenStoredDialog {}

impl OpenStoredDialog {
    /// Starts at the first of `places` across a network, or at `chosen` if none is.
    pub fn new(
        places: Vec<Place>,
        chosen: usize,
        storages: Entity<StoragesPage>,
        cx: &mut Context<Self>,
    ) -> OpenStoredDialog {
        let chosen = places
            .iter()
            .position(|p| p.storage.local_path(&[]).is_none())
            .unwrap_or(chosen);
        let start = places::open_place(&places, chosen, &storages, cx);
        let picker = cx.new(|cx| {
            let mut picker = Explorer::archive_picker(cx);
            picker.show_place(start, cx);
            picker
        });
        let changed = cx.observe(&picker, |_, _, cx| cx.notify());
        let picked = cx.subscribe(&picker, |_, _, e: &ExplorerEvent, cx| {
            if let ExplorerEvent::OpenArchive(source) = e {
                cx.emit(DialogEvent::Open(source.clone()));
            }
        });
        let lock_changed = cx.observe(&storages, |this: &mut Self, _, cx| {
            places::follow_lock(&this.places, this.chosen, &this.picker, &this.storages, cx);
            cx.notify();
        });
        OpenStoredDialog {
            places,
            chosen,
            picker,
            storages,
            _picker_changed: changed,
            _picked: picked,
            _lock_changed: lock_changed,
        }
    }

    fn pick_place(&mut self, i: usize, cx: &mut Context<Self>) {
        self.chosen = i;
        let place = places::open_place(&self.places, i, &self.storages, cx);
        self.picker.update(cx, |p, cx| p.show_place(place, cx));
        cx.notify();
    }

    /// The picker, for tests.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn picker(&self) -> Entity<Explorer> {
        self.picker.clone()
    }
}

impl Render for OpenStoredDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let close = icon_button("close", "x", 44.)
            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close)));
        let list = places::list(&self.places, self.chosen, Self::pick_place, cx);
        let beside = places::beside(&self.places, self.chosen, &self.picker, &self.storages, cx);
        super::card()
            .w(px(900.))
            .child(header(
                "storages",
                tonal_tint(),
                tr!("Open from a storage"),
                tr!("Choose a storage, and an archive in it"),
                Some(close),
            ))
            .child(
                div()
                    .flex()
                    .gap(px(8.))
                    .h(px(340.))
                    .flex_shrink_0()
                    .p(px(10.))
                    .rounded(px(22.))
                    .bg(rgb(c.well))
                    .child(list)
                    .child(div().flex_1().min_w_0().child(beside)),
            )
            .child(
                footer().child(div().flex_1()).child(
                    button("cancel", tr!("Cancel"), Fill::Outline, Size::Medium)
                        .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close))),
                ),
            )
    }
}
