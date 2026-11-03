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
    ClickEvent, Context, Div, Entity, EventEmitter, FontWeight, Image, ImageFormat, MouseButton,
    ObjectFit, Pixels, Point, Render, StyledImage, Subscription, Task, Window, anchored, deferred,
    div, img, prelude::*, px, relative, rgb,
};
use packwing_core::{Cancel, Control, Error, Signed, image_fits, read_entry};

use super::panel;
use crate::colors::colors;
use crate::file_manager;
use crate::input::{Changed, TextInput};
use crate::model::{Archive, Entry, Kind, count, size, text_preview};
use crate::prefs;
use crate::signatures::{self, Looking, Standing};
use crate::text_size::ui;
use crate::widgets::{
    Fill, Size, Tone, button, chip, icon, icon_button, menu_item, menu_panel, note, tile,
    tonal_tint,
};

/// What the user asked of the archive.
pub enum ArchiveEvent {
    Extract,
    AddFiles,
    Test,
    /// Sign its file.
    Sign,
    /// Show its file's checksums.
    Checksums,
    /// Choose an archive to open.
    Open,
    /// Choose an archive on a storage to open.
    OpenStored,
    /// Delete what is selected, or rename the one selected.
    Delete,
    Rename,
    /// Open this file in it, as its path in the archive says.
    OpenEntry(Vec<String>),
}

/// The largest file a preview reads: all of it has to be read, as an archive
/// gives its files out whole.
pub(crate) const MOST_PREVIEWED: u64 = 32_000_000;

/// How much of a text a preview shows.
pub(crate) const MOST_TEXT: usize = 8_000;

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
pub(crate) fn picture_format(name: &str) -> Option<ImageFormat> {
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
    /// The preview's text, when it is one: it can be chosen and copied.
    preview_text: Entity<TextInput>,
    /// What stops the preview being read, and the reading.
    previewing: Option<(Cancel, Task<()>)>,
    /// What the search found: all of it, as it was typed, in an archive with
    /// a listing. A sample's search looks in its one folder.
    found: Vec<Found>,
    /// What its signature says, none while it is looked at or for a sample.
    signature: Option<Signed>,
    looking: Option<Looking>,
    /// Where the `…` menu is open.
    more_at: Option<Point<Pixels>>,
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
            preview_text: cx.new(|cx| TextInput::new("", cx).read_only()),
            previewing: None,
            found: Vec::new(),
            signature: None,
            looking: None,
            more_at: None,
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
        self.more_at = None;
        self.look_at_signature(cx);
        cx.notify();
    }

    /// Looks at the signature of the archive open: all of the file if it is on this
    /// computer, and across a network only when asked.
    pub fn look_at_signature(&mut self, cx: &mut Context<Self>) {
        let local = self
            .archive()
            .and_then(|a| a.source.as_ref())
            .is_some_and(|s| s.path().is_some());
        self.read_signature(local, cx);
    }

    fn read_signature(&mut self, read_all: bool, cx: &mut Context<Self>) {
        self.signature = None;
        self.looking = None;
        let Some((storage, path)) = self
            .archive()
            .and_then(|a| a.source.as_ref())
            .and_then(|s| s.place())
        else {
            return;
        };
        self.looking = Some(signatures::start(
            storage,
            path,
            read_all,
            cx,
            |this: &mut Self, signed, cx| {
                this.signature = Some(signed);
                this.looking = None;
                cx.notify();
            },
        ));
        cx.notify();
    }

    #[cfg(all(test, feature = "test-hooks"))]
    pub fn open_more_for_shot(&mut self, at: Point<Pixels>, cx: &mut Context<Self>) {
        self.more_at = Some(at);
        cx.notify();
    }

    #[cfg(all(test, feature = "test-hooks"))]
    pub fn signature_for_test(&self) -> Option<Signed> {
        self.signature.clone()
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
        let Some(source) = a.source.clone() else {
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
                this.show_preview(preview, cx);
            });
        });
        self.previewing = Some((cancel, task));
    }

    fn show_preview(&mut self, preview: Preview, cx: &mut Context<Self>) {
        if let Preview::Text(text) = &preview {
            self.preview_text
                .update(cx, |t, cx| t.set_text(text, 0..0, cx));
        }
        self.preview = preview;
        cx.notify();
    }

    /// Why the file named could not be opened, if it could not, for tests.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn shown_failed(&self) -> Option<String> {
        match &self.shown {
            Shown::Failed { why, .. } => Some(why.clone()),
            _ => None,
        }
    }

    /// The names the list shows, searched or not, for tests.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn shown_names(&self, cx: &gpui::App) -> Vec<String> {
        self.archive()
            .map(|a| {
                self.shown(a, cx)
                    .into_iter()
                    .map(|(_, e, _)| e.name)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Where in the archive the list is, for tests.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn shown_folder(&self) -> Vec<String> {
        self.archive().map(|a| a.folder.clone()).unwrap_or_default()
    }

    /// The text the preview shows, and whether it is still reading, for tests.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn previewed(&self) -> (Option<String>, bool) {
        match &self.preview {
            Preview::Text(text) => (Some(text.clone()), false),
            Preview::Reading => (None, true),
            _ => (None, false),
        }
    }

    /// Shows `preview`, as the screenshots draw it.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn preview_for_shot(
        &mut self,
        picture: Option<Vec<u8>>,
        text: &str,
        cx: &mut Context<Self>,
    ) {
        let preview = match picture {
            Some(bytes) => Preview::Picture(Arc::new(Image::from_bytes(ImageFormat::Png, bytes))),
            None => Preview::Text(text.to_owned()),
        };
        self.show_preview(preview, cx);
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
    fn shown(&self, a: &Archive, cx: &gpui::App) -> Vec<(usize, Entry, Option<Vec<String>>)> {
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
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn select_for_shot(&mut self, at: &[usize], cx: &mut Context<Self>) {
        self.selected = at.iter().copied().collect();
        cx.notify();
    }

    /// Types `query` in the search, as the screenshots do.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn search_for(&mut self, query: &str, cx: &mut Context<Self>) {
        self.search
            .update(cx, |s, cx| s.set_text(query, query.len()..query.len(), cx));
    }

    /// The search, and beside it what opens another archive.
    fn search_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let search = div()
            .flex()
            .flex_1()
            .min_w_0()
            .items_center()
            .gap(ui(12.))
            .h(ui(48.))
            .px(ui(20.))
            .rounded_full()
            .bg(rgb(c.field))
            .text_size(ui(16.))
            .child(icon("search", 20., c.muted))
            .child(self.search.clone());
        div()
            .flex()
            .flex_shrink_0()
            .items_center()
            .gap(ui(8.))
            .child(search)
            .child(
                button(
                    "open-another",
                    tr!("Open another…"),
                    Fill::Tonal,
                    Size::Large,
                )
                .flex_shrink_0()
                .on_click(cx.listener(|_, _, _, cx| cx.emit(ArchiveEvent::Open))),
            )
            // Beside the system's dialog, the storages the user added.
            .when(prefs::has_storage(cx), |d| {
                d.child(
                    button(
                        "open-stored",
                        tr!("From a storage…"),
                        Fill::Tonal,
                        Size::Large,
                    )
                    .flex_shrink_0()
                    .on_click(cx.listener(|_, _, _, cx| cx.emit(ArchiveEvent::OpenStored))),
                )
            })
    }

    fn header(&self, a: &Archive, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let signature = self.signature_chip(cx);
        let safe = div()
            .flex()
            .flex_shrink_0()
            .items_center()
            .gap(ui(6.))
            .px(ui(12.))
            .py(ui(6.))
            .rounded_full()
            .bg(rgb(c.accent))
            .text_color(rgb(c.on_accent))
            .text_size(ui(13.))
            .font_weight(FontWeight::SEMIBOLD)
            .child(icon("check", 14., c.on_accent))
            .child(tr!("Safe to extract"));
        div()
            .flex()
            .flex_shrink_0()
            .flex_wrap()
            .items_center()
            .gap_x(ui(24.))
            .gap_y(ui(16.))
            .px(ui(28.))
            .py(ui(24.))
            .rounded(ui(28.))
            .bg(rgb(c.tonal))
            .text_color(rgb(c.on_tonal))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(ui(10.))
                    .child(
                        div()
                            .text_size(ui(28.))
                            .font_weight(FontWeight::BOLD)
                            .line_height(relative(1.2))
                            .truncate()
                            .child(a.name.clone()),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap(ui(8.))
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
                            .children(a.source.as_ref().and_then(|s| s.fetched()).map(
                                |(fetched, whole)| {
                                    chip(
                                        tr!(
                                            "Read {fetched} of {whole}",
                                            fetched = size(fetched),
                                            whole = size(whole)
                                        ),
                                        c.accent,
                                        c.on_accent,
                                    )
                                },
                            ))
                            // The samples show the checks' verdict; an
                            // archive read is checked as it is extracted.
                            .when(a.listing.is_none(), |d| d.child(safe))
                            .children(signature),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(ui(10.))
                    .child(
                        button("extract", tr!("Extract"), Fill::Accent, Size::Large)
                            .px(ui(28.))
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(ArchiveEvent::Extract))),
                    )
                    // A 7z or tar archive has to be written whole again, and
                    // packwing does not do that yet.
                    .when(
                        a.listing
                            .as_ref()
                            .is_none_or(|l| packwing_core::can_add(l) && a.path.is_some()),
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
                    )
                    .child(
                        icon_button("archive-more", "more", 48.)
                            .bg(rgb(c.surface))
                            .on_click(cx.listener(|this, e: &ClickEvent, _, cx| {
                                this.more_at = Some(e.position());
                                cx.notify();
                            })),
                    ),
            )
    }

    /// What the header says of the signature, if there is anything to say.
    fn signature_chip(&self, cx: &mut Context<Self>) -> Option<impl IntoElement + use<>> {
        let c = colors();
        if self.looking.is_some() {
            return Some(div().id("signature").child(chip(
                signatures::looking_words(),
                c.surface,
                c.on_tonal,
            )));
        }
        let signed = self.signature.as_ref()?;
        let (words, standing): (String, Standing) = signatures::said(signed, cx)?;
        let (ground, ink) = standing.colors();
        let not_checked = matches!(signed, Signed::NotChecked { .. });
        Some(
            div()
                .id("signature")
                .child(chip(words, ground, ink))
                .when(not_checked, |d| {
                    d.cursor_pointer()
                        .on_click(cx.listener(|this, _, _, cx| this.read_signature(true, cx)))
                }),
        )
    }

    /// The `…` menu: what is used less, so that the header's buttons stay few.
    fn more_menu(&self, at: Point<Pixels>, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let item = |id: &'static str, label: String, asked: fn() -> ArchiveEvent| {
            menu_item(id, label, "").on_click(cx.listener(move |this, _, _, cx| {
                this.more_at = None;
                cx.emit(asked());
                cx.notify();
            }))
        };
        let here = self
            .archive()
            .and_then(|a| a.source.as_ref())
            .and_then(|s| s.place())
            .and_then(|(storage, path)| storage.local_path(&path));
        deferred(
            anchored()
                .position(at)
                .snap_to_window_with_margin(px(8.))
                .child(
                    menu_panel("archive-menu")
                        .min_w(ui(180.))
                        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                        .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                            this.more_at = None;
                            cx.notify();
                        }))
                        .child(item("menu-sign", tr!("Sign…"), || ArchiveEvent::Sign))
                        .child(item("menu-checksums", tr!("Checksums…"), || {
                            ArchiveEvent::Checksums
                        }))
                        .when_some(here, |d, here| {
                            d.child(
                                menu_item("menu-file-manager", file_manager::show_label(), "")
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.more_at = None;
                                        file_manager::show(&here, cx);
                                        cx.notify();
                                    })),
                            )
                        }),
                ),
        )
        .with_priority(1)
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
                .debug_selector(|| format!("entry-{i}"))
                .id(("entry", i))
                .flex()
                .items_center()
                .gap(ui(14.))
                .min_h(ui(56.))
                .px(ui(14.))
                .py(ui(6.))
                .rounded(ui(18.))
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
                        .text_size(ui(11.))
                        .font_weight(FontWeight::BOLD)
                        .child(e.tag()),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .text_size(ui(15.))
                        .font_weight(if on {
                            FontWeight::SEMIBOLD
                        } else {
                            FontWeight::MEDIUM
                        })
                        .child(e.name.clone())
                        .children(under.map(|under| {
                            div()
                                .truncate()
                                .text_size(ui(13.))
                                .font_weight(FontWeight::NORMAL)
                                .text_color(rgb(c.muted))
                                .child(under)
                        })),
                )
                .child(
                    div()
                        .flex_shrink_0()
                        .text_size(ui(13.))
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
            .p(ui(10.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(ui(8.))
                    .child(div().flex_1().min_w_0().child(crumb))
                    .when(changeable && one, |d| {
                        d.child(
                            button("rename", tr!("Rename"), Fill::Outline, Size::Medium)
                                .on_click(cx.listener(|_, _, _, cx| cx.emit(ArchiveEvent::Rename))),
                        )
                    })
                    .when(changeable, |d| {
                        d.child(
                            button("delete", tr!("Delete"), Fill::Outline, Size::Medium)
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
                                .px(ui(14.))
                                .py(ui(12.))
                                .text_size(ui(14.))
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
                                .px(ui(14.))
                                .py(ui(12.))
                                .text_size(ui(14.))
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
            .gap_x(ui(6.))
            .px(ui(14.))
            .pt(ui(10.))
            .pb(ui(6.))
            .text_size(ui(13.))
            .text_color(rgb(c.muted))
            .children(steps.into_iter().enumerate().map(move |(i, step)| {
                let here = i == last;
                div()
                    .flex()
                    .min_w_0()
                    .max_w_full()
                    .items_center()
                    .gap_x(ui(6.))
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
            .gap(ui(16.))
            .p(ui(40.))
            .child(tile(72., 24., tonal_tint()).child(icon(
                if failed { "warning" } else { "archive" },
                32.,
                c.on_tonal,
            )))
            .child(
                div()
                    .text_size(ui(24.))
                    .font_weight(FontWeight::BOLD)
                    .text_center()
                    .child(title),
            )
            .child(
                div()
                    .max_w(ui(520.))
                    .text_size(ui(15.))
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
                    .mt(ui(8.))
                    .px(ui(28.))
                    .on_click(cx.listener(|_, _, _, cx| cx.emit(ArchiveEvent::Open))),
                )
            })
            // Beside the system's dialog, the storages the user added.
            .when(offer && prefs::has_storage(cx), |d| {
                d.child(
                    button(
                        "open-stored",
                        tr!("From a storage…"),
                        Fill::Text,
                        Size::Medium,
                    )
                    .on_click(cx.listener(|_, _, _, cx| cx.emit(ArchiveEvent::OpenStored))),
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
            .when(showing, |d| d.flex_1().min_h(ui(200.)))
            .when(!showing, |d| d.h(ui(200.)))
            .rounded(ui(20.))
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
                                .text_size(ui(13.))
                                .text_color(rgb(c.muted))
                                .child(tr!("It could not be shown"))
                                .into_any_element()
                        }),
                ),
                (Some(_), Preview::Text(_)) => d.items_start().justify_start().child(
                    div()
                        .debug_selector(|| "preview-text".to_owned())
                        .size_full()
                        .flex()
                        .p(ui(12.))
                        .text_size(ui(12.))
                        .child(self.preview_text.clone()),
                ),
                (Some(_), Preview::Reading) => d.child(
                    div()
                        .text_size(ui(13.))
                        .text_color(rgb(c.muted))
                        .child(tr!("Reading…")),
                ),
                (Some(_), Preview::Not(why)) => d.child(
                    div()
                        .px(ui(16.))
                        .text_size(ui(13.))
                        .text_color(rgb(c.muted))
                        .child(why.clone()),
                ),
                (Some(e), _) if e.kind == Kind::Image => d.child(icon("image", 40., c.muted)),
                (Some(e), _) => d.child(
                    tile(64., 20., e.kind.tint(&c))
                        .text_size(ui(15.))
                        .font_weight(FontWeight::BOLD)
                        .child(e.tag()),
                ),
                (None, _) => d.child(
                    div()
                        .text_size(ui(13.))
                        .text_color(rgb(c.muted))
                        .child(tr!("Choose a file to see it here")),
                ),
            });
        panel()
            // Near a third of the row, as the window grows.
            .w(relative(0.3))
            .min_w(ui(300.))
            .flex_shrink_0()
            .when(showing, |d| d.h_full())
            .max_h_full()
            .flex()
            .flex_col()
            .gap(ui(14.))
            .p(ui(18.))
            .child(picture)
            .children(entry.map(|e| {
                div()
                    .flex_shrink_0()
                    .min_w_0()
                    .child(
                        div()
                            .text_size(ui(16.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .truncate()
                            .child(e.name),
                    )
                    .child(div().text_size(ui(13.)).text_color(rgb(c.muted)).child(
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
                        .px(ui(14.))
                        .py(ui(12.))
                        .rounded(ui(18.))
                        .child(tr_n!(
                            marked,
                            "{count} executable in this folder will keep its Internet mark when extracted.",
                            "{count} executables in this folder will keep their Internet mark when extracted.",
                            count = count(marked)
                        )),
                )
            })
    }
}

impl Render for ArchivePage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let page = div().size_full().flex().flex_col().gap(ui(18.));
        let Some(a) = self.archive() else {
            return page.child(self.empty(cx));
        };
        let more = self.more_at.map(|at| self.more_menu(at, cx));
        page.child(self.search_bar(cx))
            .child(self.header(a, cx))
            .children(more)
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .items_start()
                    .gap(ui(18.))
                    .child(self.contents(a, cx))
                    .child(self.preview(a)),
            )
    }
}
