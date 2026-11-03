//! The list of storages a dialog chooses a place from, beside a picker browsing
//! the one chosen. A storage that asks for this computer's password first is on
//! the list with a lock, and asks for it, as its page does, before it is shown.

use gpui::{AnyElement, App, Context, Div, Entity, FontWeight, Stateful, div, prelude::*, rgb};

use crate::colors::colors;
use crate::explorer::{Explorer, Place};
use crate::pages::storages::StoragesPage;
use crate::text_size::ui;
use crate::widgets::{icon, tile};

/// The storages to choose from: `pick` is told the index of the one pressed.
pub fn list<V: 'static>(
    places: &[Place],
    chosen: usize,
    pick: fn(&mut V, usize, &mut Context<V>),
    cx: &mut Context<V>,
) -> Stateful<Div> {
    let c = colors();
    let rows = places.iter().enumerate().map(|(i, place)| {
        let on = i == chosen;
        let name = place.name.clone();
        div()
            .debug_selector(move || format!("place: {name}"))
            .id(("to-place", i))
            .flex()
            .items_center()
            .gap(ui(10.))
            .min_h(ui(48.))
            .px(ui(10.))
            .rounded(ui(14.))
            .cursor_pointer()
            .when(on, |d| d.bg(rgb(c.tonal)))
            .when(!on, |d| d.hover(|s| s.bg(rgb(c.well_hover))))
            .on_click(cx.listener(move |this, _, _, cx| pick(this, i, cx)))
            .child(
                tile(34., 10., place.tint(&c))
                    .h(ui(30.))
                    .text_size(ui(10.))
                    .font_weight(FontWeight::BOLD)
                    .child(place.tag),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_size(ui(14.))
                    .when(on, |d| d.font_weight(FontWeight::SEMIBOLD))
                    .child(place.name.clone()),
            )
            .when(place.asks_first, |d| d.child(icon("lock", 14., c.muted)))
    });
    div()
        .id("to-places")
        .w(ui(200.))
        .flex_shrink_0()
        .overflow_y_scroll()
        .flex()
        .flex_col()
        .gap(ui(4.))
        .children(rows)
}

/// The place at `chosen`, to browse: none while it waits to be told it is the user.
pub fn open_place(
    places: &[Place],
    chosen: usize,
    storages: &Entity<StoragesPage>,
    cx: &App,
) -> Option<Place> {
    let place = places.get(chosen)?;
    (!storages.read(cx).is_locked(place)).then(|| place.clone())
}

/// Keeps `picker` on the place at `chosen` as it opens, or shuts again when left.
pub fn follow_lock(
    places: &[Place],
    chosen: usize,
    picker: &Entity<Explorer>,
    storages: &Entity<StoragesPage>,
    cx: &mut App,
) {
    let place = open_place(places, chosen, storages, cx);
    if place.is_some() != picker.read(cx).place().is_some() {
        picker.update(cx, |p, cx| p.show_place(place, cx));
    }
}

/// Beside the list: `picker`, or what opens the storage chosen if it asks first.
pub fn beside(
    places: &[Place],
    chosen: usize,
    picker: &Entity<Explorer>,
    storages: &Entity<StoragesPage>,
    cx: &mut App,
) -> AnyElement {
    match places.get(chosen) {
        Some(place) if storages.read(cx).is_locked(place) => {
            let name = place.name.clone();
            storages
                .update(cx, |p, cx| p.locked_view(name, cx))
                .into_any_element()
        }
        _ => picker.clone().into_any_element(),
    }
}

/// A folder of a storage, as the user knows it.
pub fn shown(place: &str, folder: &[String]) -> String {
    std::iter::once(place.to_owned())
        .chain(folder.iter().cloned())
        .collect::<Vec<_>>()
        .join(" › ")
}
