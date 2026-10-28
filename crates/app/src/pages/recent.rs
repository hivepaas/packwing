//! The archives opened before, the latest first.

use gpui::{Context, EventEmitter, FontWeight, Render, Window, div, prelude::*, px, rgb};

use super::{panel, title};
use crate::colors::colors;
use crate::demo::{Kind, Recent, size};
use crate::widgets::{chip, tile};

/// One of them was asked for: its index.
pub struct OpenRecent(pub usize);

pub struct RecentPage {
    archives: &'static [Recent],
}

impl EventEmitter<OpenRecent> for RecentPage {}

impl RecentPage {
    pub fn new(archives: &'static [Recent]) -> Self {
        RecentPage { archives }
    }
}

impl Render for RecentPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let rows = self.archives.iter().enumerate().map(|(i, r)| {
            div()
                .id(("recent", i))
                .flex()
                .items_center()
                .gap(px(14.))
                .min_h(px(64.))
                .px(px(14.))
                .py(px(8.))
                .rounded(px(18.))
                .cursor_pointer()
                .hover(|s| s.bg(rgb(c.hover)))
                .on_click(cx.listener(move |_, _, _, cx| cx.emit(OpenRecent(i))))
                .child(
                    tile(40., 14., Kind::Archive.tint(&c))
                        .text_size(px(11.))
                        .font_weight(FontWeight::BOLD)
                        .child(r.tag()),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .child(
                                    div()
                                        .min_w_0()
                                        .truncate()
                                        .text_size(px(15.))
                                        .font_weight(FontWeight::MEDIUM)
                                        .child(r.name),
                                )
                                .when(r.damaged, |d| {
                                    d.child(chip("Damaged", c.bad_bg, c.bad_text))
                                }),
                        )
                        .child(
                            div()
                                .text_size(px(13.))
                                .text_color(rgb(c.muted))
                                .child(format!("{} · {}", r.location, size(r.size))),
                        ),
                )
                .child(
                    div()
                        .flex_shrink_0()
                        .text_size(px(13.))
                        .text_color(rgb(c.muted))
                        .child(r.opened),
                )
        });
        div()
            .size_full()
            .flex()
            .flex_col()
            .gap(px(18.))
            .child(title("Recent", 26.).px(px(8.)).pt(px(12.)))
            .child(
                panel()
                    .id("recent-list")
                    .max_h_full()
                    .min_h_0()
                    .overflow_y_scroll()
                    .flex()
                    .flex_col()
                    .p(px(10.))
                    .children(rows),
            )
    }
}
