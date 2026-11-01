//! The open archive: a search over all of it, its header with what it is and what
//! to do with it, the folder's files, and those chosen. Before one is
//! open, or while it is read, or when it cannot be, the page says so.
//!
//! The file chosen last shows in the preview, a picture or the start of a text,
//! read in the background once it is clicked. Only what asks no more of the user
//! is shown: a file that needs a password says so.

use packwing_core::Found;
use packwing_i18n::{tr, tr_n};

use std::collections::BTreeSet;

use std::sync::Arc;

use gpui::{
    Context, Div, Entity, EventEmitter, FontWeight, Image, ImageFormat, ObjectFit, Render,
    StyledImage, Subscription, Task, Window, div, img, prelude::*, px, relative, rgb,
};
use packwing_core::{Cancel, Control, Error, image_fits, read_entry};

use super::panel;
use crate::colors::colors;
use crate::input::{Changed, TextInput};
use crate::model::{Archive, Entry, Kind, count, size, text_preview};
use crate::widgets::{Fill, Size, Tone, button, chip, icon, note, tile, tonal_tint};

/// What the user asked of the archive.
pub enum ArchiveEvent {
    Extract,
    AddFiles,
    Test,
    /// Choose an archive to open.
    Open,
    /// Delete what is selected, or rename the one selected.
    Delete,
    Rename,
    /// Open this file in it, as its path in the archive says.
    OpenEntry(Vec<String>),
}

/// The largest file a preview reads: all of it has to be read, as an archive
/// gives its files out whole.
const MOST_PREVIEWED: u64 = 32_000_000;

/// How much of a text a preview shows.
const MOST_TEXT: usize = 8_000;

/// What the preview shows of the file chosen.
#[derive(Clone, Debug, Default)]
enum Preview {
    /// Its tag.
    #[default]
    Nothing,
    Reading,
    Picture(Arc<Image>),
    Text(String),
    /// Why it cannot be shown.
    Not(String),
}

/// What a picture of this name is, for those GPUI decodes. SVG is left out: it
/// can point at other files.
fn picture_format(name: &str) -> Option<ImageFormat> {
    let (_, extension) = name.rsplit_once('.')?;
    match extension.to_ascii_lowercase().as_str() {
        "png" => Some(ImageFormat::Png),
        "jpg" | "jpeg" => Some(ImageFormat::Jpeg),
        "webp" => Some(ImageFormat::Webp),
        "gif" => Some(ImageFormat::Gif),
        "bmp" => Some(ImageFormat::Bmp),
        "tif" | "tiff" => Some(ImageFormat::Tiff),
        _ => None,
    }
}

/// The most a search lists: more is not read, and takes long to draw.
const MOST_FOUND: usize = 500;

/// What the page shows.
#[derive(Clone, Debug)]
pub enum Shown {
    /// Nothing opened yet.
    Nothing,
    /// The file named, being read.
    Opening(String),
    Open(Box<Archive>),
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
    /// Those selected, to extract: clicked, with Ctrl or Cmd for more, with
    /// Shift for those between. What only shows in the preview is not one.
    selected: BTreeSet<usize>,
    preview: Preview,
    /// What stops the preview being read, and the reading.
    previewing: Option<(Cancel, Task<()>)>,
    /// What the search found: all of it, as it was typed, in an archive with
    /// a listing. A sample's search looks in its one folder.
    found: Vec<Found>,
    _typing: Subscription,
}

impl EventEmitter<ArchiveEvent> for ArchivePage {}

impl ArchivePage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| TextInput::new(tr!("Search inside this archive"), cx));
        let typing = cx.subscribe(&search, |this: &mut Self, field, _: &Changed, cx| {
            let query = field.read(cx).text().to_owned();
            this.found = this
                .archive()
                .and_then(|a| a.listing.as_ref())
                .map(|listing| listing.search(&query))
                .unwrap_or_default();
            cx.notify();
        });
        ArchivePage {
            shown: Shown::Nothing,
            search,
            chosen: None,
            selected: BTreeSet::new(),
            preview: Preview::Nothing,
            previewing: None,
            found: Vec::new(),
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
        self.stop_preview();
        self.selected.clear();
        self.found.clear();
        self.search.update(cx, |s, cx| s.set_text("", 0..0, cx));
        cx.notify();
    }

    /// The entries selected in the folder shown, by their paths in the archive.
    pub fn picked(&self) -> Vec<Vec<String>> {
        let Some(a) = self.archive() else {
            return Vec::new();
        };
        self.selected
            .iter()
            .filter_map(|i| a.entries.get(*i))
            .map(|e| {
                let mut path = a.folder.clone();
                path.push(e.name.clone());
                path
            })
            .collect()
    }

    fn stop_preview(&mut self) {
        if let Some((cancel, _)) = self.previewing.take() {
            cancel.cancel();
        }
        self.preview = Preview::Nothing;
    }

    /// Reads the file chosen for the preview, if it is one that shows.
    fn start_preview(&mut self, cx: &mut Context<Self>) {
        self.stop_preview();
        let Some(a) = self.archive() else {
            return;
        };
        let Some(entry) = self.chosen.and_then(|i| a.entries.get(i)) else {
            return;
        };
        let format = picture_format(&entry.name);
        if format.is_none() && entry.kind != Kind::Text {
            return;
        }
        let Some(source) = a.path.clone() else {
            return;
        };
        if entry.size > MOST_PREVIEWED {
            self.preview = Preview::Not(tr!("Too big to show here"));
            return;
        }
        let mut path = a.folder.clone();
        path.push(entry.name.clone());
        let keys = a.keys.clone();
        let cancel = Cancel::new();
        let control = Control {
            progress: None,
            cancel: Some(cancel.clone()),
        };
        let reading = cx.background_executor().spawn(async move {
            match read_entry(&source, &path, &keys, MOST_PREVIEWED, &control) {
                Ok(bytes) => match format {
                    Some(format) if image_fits(&bytes) => {
                        Preview::Picture(Arc::new(Image::from_bytes(format, bytes)))
                    }
                    Some(_) => Preview::Not(tr!("Too big to show here")),
                    None => text_preview(&bytes, MOST_TEXT)
                        .map_or(Preview::Not(tr!("Not text it can show")), Preview::Text),
                },
                Err(Error::Cancelled) => Preview::Nothing,
                Err(Error::Locked { .. } | Error::WrongPassword) => {
                    Preview::Not(tr!("It needs the password to show it"))
                }
                Err(_) => Preview::Not(tr!("It could not be read")),
            }
        });
        self.preview = Preview::Reading;
        let task = cx.spawn(async move |this, cx| {
            let preview = reading.await;
            let _ = this.update(cx, |this, cx| {
                this.previewing = None;
                this.preview = preview;
                cx.notify();
            });
        });
        self.previewing = Some((cancel, task));
    }

    /// The text the preview shows, and whether it is still reading, for tests.
    #[cfg(all(test, feature = "shots"))]
    pub fn previewed(&self) -> (Option<String>, bool) {
        match &self.preview {
            Preview::Text(text) => (Some(text.clone()), false),
            Preview::Reading => (None, true),
            _ => (None, false),
        }
    }

    /// Shows `preview`, as the screenshots draw it.
    #[cfg(all(test, feature = "shots"))]
    pub fn preview_for_shot(
        &mut self,
        picture: Option<Vec<u8>>,
        text: &str,
        cx: &mut Context<Self>,
    ) {
        self.preview = match picture {
            Some(bytes) => Preview::Picture(Arc::new(Image::from_bytes(ImageFormat::Png, bytes))),
            None => Preview::Text(text.to_owned()),
        };
        cx.notify();
    }

    /// A click on the entry at `i`: it alone, or with `more` as well, or with
    /// `range` those from the one chosen last.
    fn select(&mut self, i: usize, more: bool, range: bool) {
        match (range, self.chosen) {
            (true, Some(from)) => self.selected.extend(from.min(i)..=from.max(i)),
            _ if more => {
                if !self.selected.remove(&i) {
                    self.selected.insert(i);
                }
            }
            _ => {
                self.selected.clear();
                self.selected.insert(i);
            }
        }
        self.chosen = Some(i);
    }

    /// The archive open, if one is.
    pub fn archive(&self) -> Option<&Archive> {
        match &self.shown {
            Shown::Open(a) => Some(a.as_ref()),
            _ => None,
        }
    }

    /// Shows `folder` of the open archive.
    pub fn enter(&mut self, folder: Vec<String>, cx: &mut Context<Self>) {
        if let Shown::Open(a) = &mut self.shown {
            a.show(folder);
            self.stop_preview();
            self.chosen = None;
            self.selected.clear();
            cx.notify();
        }
    }

    /// Shows what a search found where it is: a folder opened, a file chosen in
    /// its folder. The search is over.
    fn reveal(&mut self, path: &[String], dir: bool, cx: &mut Context<Self>) {
        let Some((name, parent)) = path.split_last() else {
            return;
        };
        self.search.update(cx, |s, cx| s.set_text("", 0..0, cx));
        if dir {
            self.enter(path.to_vec(), cx);
            return;
        }
        self.enter(parent.to_vec(), cx);
        self.chosen = self
            .archive()
            .and_then(|a| a.entries.iter().position(|e| &e.name == name));
        self.selected.extend(self.chosen);
        self.start_preview(cx);
        cx.notify();
    }

    /// What to list: the folder's entries; with something typed, what the
    /// search found, each with where it is.
    fn shown(&self, a: &Archive, cx: &Context<Self>) -> Vec<(usize, Entry, Option<Vec<String>>)> {
        let query = self.search.read(cx).text().trim().to_lowercase();
        if !query.is_empty() && a.listing.is_some() {
            return self
                .found
                .iter()
                .take(MOST_FOUND)
                .enumerate()
                .map(|(i, f)| (i, Entry::of(&f.child), Some(f.path.clone())))
                .collect();
        }
        a.entries
            .iter()
            .cloned()
            .enumerate()
            .filter(|(_, e)| query.is_empty() || e.name.to_lowercase().contains(&query))
            .map(|(i, e)| (i, e, None))
            .collect()
    }

    /// Selects the entries at `at`, as the screenshots do.
    #[cfg(all(test, feature = "shots"))]
    pub fn select_for_shot(&mut self, at: &[usize], cx: &mut Context<Self>) {
        self.selected = at.iter().copied().collect();
        cx.notify();
    }

    /// Types `query` in the search, as the screenshots do.
    #[cfg(all(test, feature = "shots"))]
    pub fn search_for(&mut self, query: &str, cx: &mut Context<Self>) {
        self.search
            .update(cx, |s, cx| s.set_text(query, query.len()..query.len(), cx));
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
            .child(tr!("Safe to extract"));
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
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(px(10.))
                    .child(
                        div()
                            .text_size(px(28.))
                            .font_weight(FontWeight::BOLD)
                            .line_height(relative(1.2))
                            .truncate()
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
                                tr_n!(
                                    a.files,
                                    "{count} file",
                                    "{count} files",
                                    count = count(a.files)
                                ),
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
                        button("extract", tr!("Extract"), Fill::Accent, Size::Large)
                            .px(px(28.))
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(ArchiveEvent::Extract))),
                    )
                    // A 7z or tar archive has to be written whole again, and
                    // packwing does not do that yet.
                    .when(
                        a.listing.as_ref().is_none_or(|l| packwing_core::can_add(l)),
                        |d| {
                            d.child(
                                button("add-files", tr!("Add files"), Fill::Raised, Size::Large)
                                    .on_click(
                                        cx.listener(|_, _, _, cx| cx.emit(ArchiveEvent::AddFiles)),
                                    ),
                            )
                        },
                    )
                    .child(
                        button("test", tr!("Test"), Fill::Raised, Size::Large)
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(ArchiveEvent::Test))),
                    ),
            )
    }

    fn contents(&self, a: &Archive, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let crumb = self.breadcrumb(a, cx);
        let shown = self.shown(a, cx);
        let query = self.search.read(cx).text().trim().to_string();
        let more = self.found.len().saturating_sub(MOST_FOUND);
        let searching = !query.is_empty() && a.listing.is_some();
        let rows = shown.into_iter().map(|(i, e, found)| {
            let on = !searching && self.selected.contains(&i);
            let file_path = (e.kind != Kind::Folder && found.is_none()).then(|| {
                let mut path = a.folder.clone();
                path.push(e.name.clone());
                path
            });
            let under = found.as_ref().map(|path| {
                let folders = &path[..path.len().saturating_sub(1)];
                if folders.is_empty() {
                    a.name.clone()
                } else {
                    folders.join(" › ")
                }
            });
            let dir = e.kind == Kind::Folder;
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
                // A click selects; a double click opens.
                .on_click(cx.listener(move |this, e: &gpui::ClickEvent, _, cx| {
                    if let Some(path) = &found {
                        this.reveal(path, dir, cx);
                        return;
                    }
                    match (&inside, &file_path) {
                        (Some(folder), _) if e.click_count() >= 2 => this.enter(folder.clone(), cx),
                        (_, Some(path)) if e.click_count() >= 2 => {
                            cx.emit(ArchiveEvent::OpenEntry(path.clone()));
                        }
                        _ => {
                            let keys = e.modifiers();
                            this.select(i, keys.secondary(), keys.shift);
                            this.start_preview(cx);
                            cx.notify();
                        }
                    }
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
                        .child(e.name.clone())
                        .children(under.map(|under| {
                            div()
                                .truncate()
                                .text_size(px(13.))
                                .font_weight(FontWeight::NORMAL)
                                .text_color(rgb(c.muted))
                                .child(under)
                        })),
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
        let changeable = !searching
            && !self.selected.is_empty()
            && a.listing
                .as_ref()
                .is_some_and(|l| packwing_core::can_add(l));
        let one = self.selected.len() == 1;
        panel()
            .flex_1()
            .min_w_0()
            .max_h_full()
            .flex()
            .flex_col()
            .p(px(10.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(div().flex_1().min_w_0().child(crumb))
                    .when(changeable && one, |d| {
                        d.child(
                            button("rename", tr!("Rename"), Fill::Outline, Size::Small)
                                .on_click(cx.listener(|_, _, _, cx| cx.emit(ArchiveEvent::Rename))),
                        )
                    })
                    .when(changeable, |d| {
                        d.child(
                            button("delete", tr!("Delete"), Fill::Outline, Size::Small)
                                .on_click(cx.listener(|_, _, _, cx| cx.emit(ArchiveEvent::Delete))),
                        )
                    }),
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
                                .child(if searching {
                                    tr!("Nothing in this archive matches “{query}”.", query = query)
                                } else {
                                    tr!("Nothing in this folder matches “{query}”.", query = query)
                                }),
                        )
                    })
                    .when(searching && more > 0, |d| {
                        d.child(
                            div()
                                .px(px(14.))
                                .py(px(12.))
                                .text_size(px(14.))
                                .text_color(rgb(c.muted))
                                .child(tr!(
                                    "And {more} more. Type more to narrow it down.",
                                    more = more
                                )),
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
            .min_w_0()
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
                    .min_w_0()
                    .max_w_full()
                    .items_center()
                    .gap_x(px(6.))
                    .when(i > 0, |d| d.child("›"))
                    .child(
                        div()
                            .id(("step", i))
                            .min_w_0()
                            .truncate()
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
            Shown::Opening(name) => (name.clone(), tr!("Reading what’s inside…").to_owned(), false),
            Shown::Failed { name, why } => (name.clone(), why.clone(), true),
            _ => (
                tr!("Open an archive").to_owned(),
                tr!("ZIP, 7z and tar archives, disc images and packages, compressed or encrypted with age: .zip, .7z, .tar.gz, .tar.zst, .iso, .rpm, .pkg, .zst.age…")
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
                            tr!("Open another…")
                        } else {
                            tr!("Open an archive…")
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
        // What it shows takes the height there is; a tag needs little.
        let showing =
            entry.is_some() && matches!(self.preview, Preview::Picture(_) | Preview::Text(_));
        let picture = div()
            .when(showing, |d| d.flex_1().min_h(px(200.)))
            .when(!showing, |d| d.h(px(200.)))
            .rounded(px(20.))
            .bg(rgb(c.field))
            .flex()
            .items_center()
            .justify_center()
            .overflow_hidden()
            .map(|d| match (&entry, &self.preview) {
                (Some(_), Preview::Picture(picture)) => d.child(
                    img(picture.clone())
                        .size_full()
                        .object_fit(ObjectFit::Contain)
                        .with_fallback(move || {
                            div()
                                .text_size(px(13.))
                                .text_color(rgb(c.muted))
                                .child(tr!("It could not be shown"))
                                .into_any_element()
                        }),
                ),
                (Some(_), Preview::Text(text)) => d.items_start().justify_start().child(
                    div()
                        .id("preview-text")
                        .size_full()
                        .overflow_y_scroll()
                        .p(px(12.))
                        .text_size(px(12.))
                        .child(text.clone()),
                ),
                (Some(_), Preview::Reading) => d.child(
                    div()
                        .text_size(px(13.))
                        .text_color(rgb(c.muted))
                        .child(tr!("Reading…")),
                ),
                (Some(_), Preview::Not(why)) => d.child(
                    div()
                        .px(px(16.))
                        .text_size(px(13.))
                        .text_color(rgb(c.muted))
                        .child(why.clone()),
                ),
                (Some(e), _) if e.kind == Kind::Image => d.child(icon("image", 40., c.muted)),
                (Some(e), _) => d.child(
                    tile(64., 20., e.kind.tint(&c))
                        .text_size(px(15.))
                        .font_weight(FontWeight::BOLD)
                        .child(e.tag()),
                ),
                (None, _) => d.child(
                    div()
                        .text_size(px(13.))
                        .text_color(rgb(c.muted))
                        .child(tr!("Choose a file to see it here")),
                ),
            });
        panel()
            // A quarter of the row, as the window grows.
            .w(relative(0.25))
            .min_w(px(300.))
            .flex_shrink_0()
            .when(showing, |d| d.h_full())
            .max_h_full()
            .flex()
            .flex_col()
            .gap(px(14.))
            .p(px(18.))
            .child(picture)
            .children(entry.map(|e| {
                div()
                    .flex_shrink_0()
                    .min_w_0()
                    .child(
                        div()
                            .text_size(px(16.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .truncate()
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
                        .child(tr!("{counted} in this folder will keep {their} Internet mark when extracted.", counted = tr_n!(marked, "{count} executable", "{count} executables", count = count(marked)), their = if marked == 1 { "its" } else { "their" })),
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
