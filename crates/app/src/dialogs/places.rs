//! The list of storages a dialog chooses a place from, beside a picker browsing
//! the one chosen.

use gpui::{Context, Div, FontWeight, Stateful, div, prelude::*, px, rgb};

use crate::colors::colors;
use crate::explorer::Place;
use crate::widgets::tile;

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
        div()
            .id(("to-place", i))
            .flex()
            .items_center()
            .gap(px(10.))
            .min_h(px(48.))
            .px(px(10.))
            .rounded(px(14.))
            .cursor_pointer()
            .when(on, |d| d.bg(rgb(c.tonal)))
            .when(!on, |d| d.hover(|s| s.bg(rgb(c.well_hover))))
            .on_click(cx.listener(move |this, _, _, cx| pick(this, i, cx)))
            .child(
                tile(34., 10., place.tint(&c))
                    .h(px(30.))
                    .text_size(px(10.))
                    .font_weight(FontWeight::BOLD)
                    .child(place.tag),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_size(px(14.))
                    .when(on, |d| d.font_weight(FontWeight::SEMIBOLD))
                    .child(place.name.clone()),
            )
    });
    div()
        .id("to-places")
        .w(px(200.))
        .flex_shrink_0()
        .overflow_y_scroll()
        .flex()
        .flex_col()
        .gap(px(4.))
        .children(rows)
}

/// A folder of a storage, as the user knows it.
pub fn shown(place: &str, folder: &[String]) -> String {
    std::iter::once(place.to_owned())
        .chain(folder.iter().cloned())
        .collect::<Vec<_>>()
        .join(" › ")
}
