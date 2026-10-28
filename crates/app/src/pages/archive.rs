//! The open archive: a search over it, its header with what it is and what
//! to do with it, the folder's files, and the one chosen.

use gpui::{
    Context, Entity, EventEmitter, FontWeight, Render, Subscription, Window, div, prelude::*, px,
    relative, rgb,
};

use super::panel;
use crate::colors::colors;
use crate::demo::{Archive, Entry, Kind, counted, size};
use crate::input::{Changed, TextInput};
use crate::widgets::{Fill, Size, Tone, button, chip, icon, note, tile};

/// What the user asked of the archive.
pub enum ArchiveEvent {
    Extract,
    AddFiles,
    Test,
}

pub struct ArchivePage {
    archive: Archive,
    search: Entity<TextInput>,
    /// The entry chosen, by its index in the archive's.
    chosen: Option<usize>,
    _typing: Subscription,
}

impl EventEmitter<ArchiveEvent> for ArchivePage {}

impl ArchivePage {
    pub fn new(archive: Archive, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| TextInput::new("Search inside this archive", cx));
        let typing = cx.subscribe(&search, |_, _, _: &Changed, cx| cx.notify());
        ArchivePage {
            archive,
            search,
            // The first image, so the preview has something to show.
            chosen: archive.entries.iter().position(|e| e.kind == Kind::Image),
            _typing: typing,
        }
    }

    /// The entries whose names hold what is typed in the search.
    fn shown(&self, cx: &Context<Self>) -> Vec<(usize, Entry)> {
        let query = self.search.read(cx).text().trim().to_lowercase();
        self.archive
            .entries
            .iter()
            .copied()
            .enumerate()
            .filter(|(_, e)| query.is_empty() || e.name.to_lowercase().contains(&query))
            .collect()
    }

    fn search_bar(&self) -> impl IntoElement {
        let c = colors();
        div()
            .flex()
            .flex_shrink_0()
            .items_center()
            .gap(px(12.))
            .h(px(52.))
            .px(px(20.))
            .rounded_full()
            .bg(rgb(c.field))
            .text_size(px(16.))
            .child(icon("search", 20., c.muted))
            .child(self.search.clone())
    }

    fn header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let a = self.archive;
        let safe = div()
            .flex()
            .flex_shrink_0()
            .items_center()
            .gap(px(6.))
            .px(px(12.))
            .py(px(6.))
            .rounded_full()
            .bg(rgb(c.accent))
            .text_color(rgb(c.on_accent))
            .text_size(px(13.))
            .font_weight(FontWeight::SEMIBOLD)
            .child(icon("check", 14., c.on_accent))
            .child("Safe to extract");
        div()
            .flex()
            .flex_shrink_0()
            .flex_wrap()
            .items_center()
            .gap_x(px(24.))
            .gap_y(px(16.))
            .px(px(28.))
            .py(px(24.))
            .rounded(px(28.))
            .bg(rgb(c.tonal))
            .text_color(rgb(c.on_tonal))
            .child(
                div()
                    .flex_1()
                    .min_w(px(320.))
                    .flex()
                    .flex_col()
                    .gap(px(10.))
                    .child(
                        div()
                            .text_size(px(28.))
                            .font_weight(FontWeight::BOLD)
                            .line_height(relative(1.2))
                            .child(a.name),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap(px(8.))
                            .child(chip(
                                format!("{} · {}%", size(a.packed), a.ratio()),
                                c.surface,
                                c.on_tonal,
                            ))
                            .child(chip(
                                counted(a.files, "file", "files"),
                                c.surface,
                                c.on_tonal,
                            ))
                            .children(a.encryption.map(|e| chip(e, c.surface, c.on_tonal)))
                            .child(safe),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(10.))
                    .child(
                        button("extract", "Extract", Fill::Accent, Size::Large)
                            .px(px(28.))
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(ArchiveEvent::Extract))),
                    )
                    .child(
                        button("add-files", "Add files", Fill::Raised, Size::Large)
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(ArchiveEvent::AddFiles))),
                    )
                    .child(
                        button("test", "Test", Fill::Raised, Size::Large)
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(ArchiveEvent::Test))),
                    ),
            )
    }

    fn contents(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let shown = self.shown(cx);
        let query = self.search.read(cx).text().trim().to_string();
        let rows = shown.into_iter().map(|(i, e)| {
            let on = self.chosen == Some(i);
            let tint = e.kind.tint(&c);
            div()
                .id(("entry", i))
                .flex()
                .items_center()
                .gap(px(14.))
                .min_h(px(56.))
                .px(px(14.))
                .py(px(6.))
                .rounded(px(18.))
                .cursor_pointer()
                .when(on, |d| d.bg(rgb(c.chosen)))
                .when(!on, |d| d.hover(|s| s.bg(rgb(c.hover))))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.chosen = Some(i);
                    cx.notify();
                }))
                .child(
                    tile(40., 14., tint)
                        .text_size(px(11.))
                        .font_weight(FontWeight::BOLD)
                        .child(e.tag()),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .text_size(px(15.))
                        .font_weight(if on {
                            FontWeight::SEMIBOLD
                        } else {
                            FontWeight::MEDIUM
                        })
                        .child(e.name),
                )
                .child(
                    div()
                        .flex_shrink_0()
                        .text_size(px(13.))
                        .text_color(rgb(c.muted))
                        .child(size(e.size)),
                )
        });
        let empty = rows.len() == 0;
        panel()
            .flex_1()
            .min_w_0()
            .max_h_full()
            .flex()
            .flex_col()
            .p(px(10.))
            .child(
                div()
                    .px(px(14.))
                    .pt(px(10.))
                    .pb(px(6.))
                    .text_size(px(13.))
                    .text_color(rgb(c.muted))
                    .child(self.archive.folder.join(" › ")),
            )
            .child(
                div()
                    .id("entries")
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .overflow_y_scroll()
                    .children(rows)
                    .when(empty, |d| {
                        d.child(
                            div()
                                .px(px(14.))
                                .py(px(12.))
                                .text_size(px(14.))
                                .text_color(rgb(c.muted))
                                .child(format!("Nothing in this folder matches “{query}”.")),
                        )
                    }),
            )
    }

    fn preview(&self) -> impl IntoElement {
        let c = colors();
        let entry = self
            .chosen
            .and_then(|i| self.archive.entries.get(i).copied());
        let marked = self.archive.marked.len() as u64;
        let picture = div()
            .h(px(200.))
            .rounded(px(20.))
            .bg(rgb(c.field))
            .flex()
            .items_center()
            .justify_center()
            .map(|d| match entry {
                Some(e) if e.kind == Kind::Image => d.child(icon("image", 40., c.muted)),
                Some(e) => d.child(
                    tile(64., 20., e.kind.tint(&c))
                        .text_size(px(15.))
                        .font_weight(FontWeight::BOLD)
                        .child(e.tag()),
                ),
                None => d.child(
                    div()
                        .text_size(px(13.))
                        .text_color(rgb(c.muted))
                        .child("Choose a file to see it here"),
                ),
            });
        panel()
            .w(px(300.))
            .flex_shrink_0()
            .flex()
            .flex_col()
            .gap(px(14.))
            .p(px(18.))
            .child(picture)
            .children(entry.map(|e| {
                div()
                    .child(
                        div()
                            .text_size(px(16.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(e.name),
                    )
                    .child(
                        div()
                            .text_size(px(13.))
                            .text_color(rgb(c.muted))
                            .child(format!("{} · {}", size(e.size), e.modified)),
                    )
            }))
            .when(marked > 0, |d| {
                d.child(
                    note(Tone::Warn)
                        .px(px(14.))
                        .py(px(12.))
                        .rounded(px(18.))
                        .child(format!(
                            "{} in this folder will keep {} Internet mark when extracted.",
                            counted(marked, "executable", "executables"),
                            if marked == 1 { "its" } else { "their" }
                        )),
                )
            })
    }
}

impl Render for ArchivePage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .gap(px(18.))
            .child(self.search_bar())
            .child(self.header(cx))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .items_start()
                    .gap(px(18.))
                    .child(self.contents(cx))
                    .child(self.preview()),
            )
    }
}
