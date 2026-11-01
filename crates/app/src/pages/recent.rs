//! The archives opened before, the latest first.

use gpui::{Context, EventEmitter, FontWeight, Render, Window, div, prelude::*, px, rgb};
use packwing_i18n::tr;

use super::{panel, title};
use crate::colors::colors;
use crate::model::{Kind, date, size, tag_of};
use crate::prefs::Opened;
use crate::widgets::tile;

/// One of them was asked for: its index.
pub struct OpenRecent(pub usize);

/// A row of the page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub name: String,
    pub location: String,
    /// Its size; none if it is not there any more.
    pub size: Option<u64>,
    pub opened: String,
}

impl Row {
    /// What `opened` is now, as its file says.
    pub fn of(opened: &Opened) -> Row {
        let path = &opened.path;
        Row {
            name: path.file_name().map_or_else(
                || path.display().to_string(),
                |n| n.to_string_lossy().into_owned(),
            ),
            location: path
                .parent()
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
            size: std::fs::metadata(path).ok().map(|m| m.len()),
            opened: date(opened.at),
        }
    }
}

pub struct RecentPage {
    archives: Vec<Row>,
}

impl EventEmitter<OpenRecent> for RecentPage {}

impl RecentPage {
    pub fn new(archives: Vec<Row>) -> Self {
        RecentPage { archives }
    }

    pub fn show(&mut self, archives: Vec<Row>, cx: &mut Context<Self>) {
        self.archives = archives;
        cx.notify();
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
                        .child(tag_of(&r.name, Kind::Archive)),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .child(
                            div().flex().items_center().gap(px(8.)).child(
                                div()
                                    .min_w_0()
                                    .truncate()
                                    .text_size(px(15.))
                                    .font_weight(FontWeight::MEDIUM)
                                    .child(r.name.clone()),
                            ),
                        )
                        .child(div().text_size(px(13.)).text_color(rgb(c.muted)).child(
                            match r.size {
                                Some(bytes) => format!("{} · {}", r.location, size(bytes)),
                                None => {
                                    tr!("{location} · not there any more", location = r.location)
                                }
                            },
                        )),
                )
                .child(
                    div()
                        .flex_shrink_0()
                        .text_size(px(13.))
                        .text_color(rgb(c.muted))
                        .child(r.opened.clone()),
                )
        });
        div()
            .size_full()
            .flex()
            .flex_col()
            .gap(px(18.))
            .child(title(tr!("Recent"), 26.).px(px(8.)).pt(px(12.)))
            .child(
                panel()
                    .id("recent-list")
                    .max_h_full()
                    .min_h_0()
                    .overflow_y_scroll()
                    .flex()
                    .flex_col()
                    .p(px(10.))
                    .when(self.archives.is_empty(), |d| {
                        d.child(
                            div()
                                .p(px(16.))
                                .text_size(px(14.))
                                .text_color(rgb(c.muted))
                                .child(tr!("The archives you open will show here.")),
                        )
                    })
                    .children(rows),
            )
    }
}
