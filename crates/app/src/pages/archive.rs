//! The open archive: a search over it, its header with what it is and what
//! to do with it, the folder's files, and the one chosen. Before one is
//! open, or while it is read, or when it cannot be, the page says so.

use gpui::{
    Context, Div, Entity, EventEmitter, FontWeight, Render, Subscription, Window, div, prelude::*,
    px, relative, rgb,
};

use super::panel;
use crate::colors::colors;
use crate::input::{Changed, TextInput};
use crate::model::{Archive, Entry, Kind, counted, size};
use crate::widgets::{Fill, Size, Tone, button, chip, icon, note, tile, tonal_tint};

/// What the user asked of the archive.
pub enum ArchiveEvent {
    Extract,
    AddFiles,
    Test,
    /// Choose an archive to open.
    Open,
}

/// What the page shows.
#[derive(Clone, Debug)]
pub enum Shown {
    /// Nothing opened yet.
    Nothing,
    /// The file named, being read.
    Opening(String),
    Open(Archive),
    /// The file named could not be opened, and why.
    Failed {
        name: String,
        why: String,
    },
}

pub struct ArchivePage {
    shown: Shown,
    search: Entity<TextInput>,
    /// The entry chosen, by its index in the folder's.
    chosen: Option<usize>,
    _typing: Subscription,
}

impl EventEmitter<ArchiveEvent> for ArchivePage {}

impl ArchivePage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| TextInput::new("Search inside this archive", cx));
        let typing = cx.subscribe(&search, |_, _, _: &Changed, cx| cx.notify());
        ArchivePage {
            shown: Shown::Nothing,
            search,
            chosen: None,
            _typing: typing,
        }
    }

    pub fn show(&mut self, shown: Shown, cx: &mut Context<Self>) {
        // The first image, so the preview has something to show.
        self.chosen = match &shown {
            Shown::Open(a) => a.entries.iter().position(|e| e.kind == Kind::Image),
            _ => None,
        };
        self.shown = shown;
        self.search.update(cx, |s, cx| s.set_text("", 0..0, cx));
        cx.notify();
    }

    /// The archive open, if one is.
    pub fn archive(&self) -> Option<&Archive> {
        match &self.shown {
            Shown::Open(a) => Some(a),
            _ => None,
        }
    }

    /// Shows `folder` of the open archive.
    fn enter(&mut self, folder: Vec<String>, cx: &mut Context<Self>) {
        if let Shown::Open(a) = &mut self.shown {
            a.show(folder);
            self.chosen = None;
            cx.notify();
        }
    }

    /// The entries of the folder whose names hold what is typed in the
    /// search.
    fn shown(&self, a: &Archive, cx: &Context<Self>) -> Vec<(usize, Entry)> {
        let query = self.search.read(cx).text().trim().to_lowercase();
        a.entries
            .iter()
            .cloned()
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

    fn header(&self, a: &Archive, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
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
                            .child(a.name.clone()),
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
                            .children(a.encryption.clone().map(|e| chip(e, c.surface, c.on_tonal)))
                            // The samples show the checks' verdict; an
                            // archive read is checked as it is extracted.
                            .when(a.listing.is_none(), |d| d.child(safe)),
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

    fn contents(&self, a: &Archive, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let crumb = self.breadcrumb(a, cx);
        let shown = self.shown(a, cx);
        let query = self.search.read(cx).text().trim().to_string();
        let rows = shown.into_iter().map(|(i, e)| {
            let on = self.chosen == Some(i);
            let tint = e.kind.tint(&c);
            let inside = (e.kind == Kind::Folder).then(|| {
                let mut folder = a.folder.clone();
                folder.push(e.name.clone());
                folder
            });
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
                // A click chooses; a double click opens a folder.
                .on_click(
                    cx.listener(move |this, e: &gpui::ClickEvent, _, cx| match &inside {
                        Some(folder) if e.click_count() >= 2 => this.enter(folder.clone(), cx),
                        _ => {
                            this.chosen = Some(i);
                            cx.notify();
                        }
                    }),
                )
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
                        .child(e.name.clone()),
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
            .child(crumb)
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

    /// The folder shown, from the archive's top: each step leads back to
    /// it. The samples show theirs as text.
    fn breadcrumb(&self, a: &Archive, cx: &mut Context<Self>) -> Div {
        let c = colors();
        let steps = if a.listing.is_some() {
            std::iter::once(a.stem().to_owned())
                .chain(a.folder.iter().cloned())
                .collect()
        } else {
            a.folder.clone()
        };
        let last = steps.len().saturating_sub(1);
        let navigable = a.listing.is_some();
        div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_x(px(6.))
            .px(px(14.))
            .pt(px(10.))
            .pb(px(6.))
            .text_size(px(13.))
            .text_color(rgb(c.muted))
            .children(steps.into_iter().enumerate().map(move |(i, step)| {
                let here = i == last;
                div()
                    .flex()
                    .items_center()
                    .gap_x(px(6.))
                    .when(i > 0, |d| d.child("›"))
                    .child(
                        div()
                            .id(("step", i))
                            .when(here, |d| d.font_weight(FontWeight::SEMIBOLD))
                            .when(navigable && !here, |d| {
                                d.cursor_pointer()
                                    .hover(|s| s.text_color(rgb(c.text)))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        let folder = this
                                            .archive()
                                            .map(|a| a.folder[..i].to_vec())
                                            .unwrap_or_default();
                                        this.enter(folder, cx);
                                    }))
                            })
                            .child(step),
                    )
            }))
    }

    /// What the page shows with no archive open: an invitation, the file
    /// being read, or why it could not be.
    fn empty(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let (title, line, offer) = match &self.shown {
            Shown::Opening(name) => (name.clone(), "Reading what’s inside…".to_owned(), false),
            Shown::Failed { name, why } => (name.clone(), why.clone(), true),
            _ => (
                "Open an archive".to_owned(),
                "ZIP and tar archives, compressed or encrypted with age: .zip, .tar.gz, .tar.zst, .tar.xz, .tgz, .zst.age…"
                    .to_owned(),
                true,
            ),
        };
        let failed = matches!(self.shown, Shown::Failed { .. });
        panel()
            .flex_1()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(16.))
            .p(px(40.))
            .child(tile(72., 24., tonal_tint()).child(icon(
                if failed { "warning" } else { "archive" },
                32.,
                c.on_tonal,
            )))
            .child(
                div()
                    .text_size(px(24.))
                    .font_weight(FontWeight::BOLD)
                    .text_center()
                    .child(title),
            )
            .child(
                div()
                    .max_w(px(520.))
                    .text_size(px(15.))
                    .text_center()
                    .line_height(relative(1.45))
                    .text_color(rgb(c.muted))
                    .child(line),
            )
            .when(offer, |d| {
                d.child(
                    button(
                        "open-archive",
                        if failed {
                            "Open another…"
                        } else {
                            "Open an archive…"
                        },
                        Fill::Accent,
                        Size::Large,
                    )
                    .mt(px(8.))
                    .px(px(28.))
                    .on_click(cx.listener(|_, _, _, cx| cx.emit(ArchiveEvent::Open))),
                )
            })
    }

    fn preview(&self, a: &Archive) -> impl IntoElement {
        let c = colors();
        let entry = self.chosen.and_then(|i| a.entries.get(i).cloned());
        let marked = a.marked.len() as u64;
        let picture = div()
            .h(px(200.))
            .rounded(px(20.))
            .bg(rgb(c.field))
            .flex()
            .items_center()
            .justify_center()
            .map(|d| match &entry {
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
                    .child(div().text_size(px(13.)).text_color(rgb(c.muted)).child(
                        if e.modified.is_empty() {
                            size(e.size)
                        } else {
                            format!("{} · {}", size(e.size), e.modified)
                        },
                    ))
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
        let page = div().size_full().flex().flex_col().gap(px(18.));
        let Some(a) = self.archive() else {
            return page.child(self.empty(cx));
        };
        page.child(self.search_bar())
            .child(self.header(a, cx))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .items_start()
                    .gap(px(18.))
                    .child(self.contents(a, cx))
                    .child(self.preview(a)),
            )
    }
}
