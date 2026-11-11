//! The open archive: a search over all of it, its header with what it is and what
//! to do with it, the folder's files, and those chosen. Before one is
//! open, or while it is read, or when it cannot be, the page says so.
//!
//! The file chosen last shows in the preview, a picture or the start of a text,
//! read in the background once it is clicked. Only what asks no more of the user
//! is shown: a file that needs a password says so.

use packwing_core::Found;
use packwing_i18n::{tr, tr_n};

use std::collections::{BTreeSet, HashSet};

use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use gpui::{
    ClickEvent, Context, Div, Entity, EventEmitter, FocusHandle, FontWeight, Image, ImageFormat,
    KeyDownEvent, MouseButton, MouseDownEvent, ObjectFit, Pixels, Point, Render, ScrollStrategy,
    StyledImage, Subscription, Task, UniformListScrollHandle, Window, anchored, deferred, div, img,
    prelude::*, px, relative, rgb, uniform_list,
};
use packwing_core::{Cancel, Control, Error, Keys, Signed, Source, image_fits, preview_entry};

use super::panel;
use crate::colors::colors;
use crate::file_manager;
use crate::input::{Changed, TextInput};
use crate::list_keys;
use crate::model::{Archive, Entry, Kind, count, size, text_preview};
use crate::prefs;
use crate::signatures::{self, Looking, Standing};
use crate::text_size::ui;
use crate::widgets::{
    Fill, Size, Tone, button, chip, icon, icon_button, kind_tile, menu_item, menu_panel, note,
    tile, tonal_tint,
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
    /// Make an archive of what it holds, in another format.
    Convert,
    /// Cut its file into parts.
    Split,
    /// Join the parts its file is one of.
    Join,
    /// Choose an archive to open.
    Open,
    /// Choose an archive on a storage to open.
    OpenStored,
    /// Delete what is selected, or rename the one selected.
    Delete,
    Rename,
    /// Open this file, as its path in the archive says: an archive in
    /// packwing, any other file shown, if packwing can show it.
    OpenEntry(Vec<String>),
    /// Open this file with the app the system has for it.
    OpenEntryWithApp(Vec<String>),
    /// Open this file in packwing, as an archive, whatever its name says.
    OpenEntryAsArchive(Vec<String>),
    /// Go back to an archive the one shown was opened from: the first, the
    /// one opened from a storage, is 0.
    Back(usize),
}

/// An archive the one shown was opened from, as it was left: the folder shown,
/// without its keys, which are asked for again.
#[derive(Clone, Debug)]
struct Outer {
    archive: Box<Archive>,
    /// Its copy, if it was opened from another.
    copy: Option<PathBuf>,
}

/// Where a step of the breadcrumb leads.
#[derive(Clone, Copy)]
enum Step {
    /// Back to an archive the one shown was opened from.
    Back(usize),
    /// To the folder this deep in the archive shown: 0 its top.
    Folder(usize),
}

/// Removes a copy of an archive opened from another, with the folder made
/// for it alone.
fn remove_copy(copy: &Path) {
    if let Some(folder) = copy.parent() {
        let _ = std::fs::remove_dir_all(folder);
    }
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

/// How tall a row of the list is, in rems' pixels: each the same, so
/// that only those in sight are made.
const ROW_H: f32 = 56.;

/// The most a search lists: more is not read, and takes long to draw.
const MOST_FOUND: usize = 500;

/// A row of the list: its index, among the folder's entries or what the
/// search found, the entry, and where it was found.
type Row = (usize, Entry, Option<Vec<String>>);

/// What the list holds.
enum Listed {
    /// The folder's entries, so many.
    Folder(usize),
    /// Those of them whose names have what was typed, by their indexes: the
    /// samples', which have no listing to search.
    Matching(Vec<usize>),
    /// What the search found, so many.
    Found(usize),
}

impl Listed {
    fn len(&self) -> usize {
        match self {
            Listed::Folder(n) | Listed::Found(n) => *n,
            Listed::Matching(at) => at.len(),
        }
    }
}

/// What a reading of an archive found that its page has not shown yet, and
/// how far in it is, in thousandths of its file: what packwing-core is told
/// as it reads.
#[derive(Clone, Default)]
pub struct Heard {
    pub found: Arc<Mutex<Vec<packwing_core::Entry>>>,
    pub read: Arc<AtomicU32>,
}

/// How often the page shows what a reading found, at most.
const HEAR_EVERY: Duration = Duration::from_millis(150);

/// An archive being read, as its page shows it.
struct Reading {
    heard: Heard,
    /// What shows what it found, now and then.
    _hearing: Task<()>,
}

/// What the page shows.
#[derive(Clone, Debug)]
pub enum Shown {
    /// Nothing opened yet.
    Nothing,
    /// An archive, read through or being read.
    Open(Box<Archive>),
    /// The file named could not be opened, and why.
    Failed { name: String, why: String },
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
    /// Where the right-click menu of what is selected is open.
    menu_at: Option<Point<Pixels>>,
    /// What forgets the secrets of the archive open, [`KEYS_KEPT_FOR`] after
    /// they were last used: a use puts a new one in its place.
    forgetting: Option<Task<()>>,
    /// The list's, for its keys: F2 renames, Delete deletes.
    focus: FocusHandle,
    /// The archive shown is being read: what was found so far shows, and
    /// what is done with it waits.
    reading: Option<Reading>,
    /// Where the list is scrolled to.
    scroll: UniformListScrollHandle,
    /// The archives the one shown was opened from, the first opened first.
    outer: Vec<Outer>,
    /// The copy of the archive shown, when it was opened from another: what
    /// it is changed into would be lost with the copy, so it is only read.
    copy: Option<PathBuf>,
    _typing: Subscription,
}

impl EventEmitter<ArchiveEvent> for ArchivePage {}

/// How long the password or passphrase of the archive open is kept in
/// memory after its last use: within it, viewing a file, testing and
/// extracting ask for it no more. It is never written anywhere.
pub const KEYS_KEPT_FOR: Duration = Duration::from_secs(10 * 60);

impl ArchivePage {
    /// `keys` opened `source` (a job with them went through): if it is the
    /// archive open, they are kept with it, for the next that needs them.
    pub fn remember_keys(&mut self, source: &Source, keys: &Keys, cx: &mut Context<Self>) {
        let Shown::Open(a) = &mut self.shown else {
            return;
        };
        if a.source.as_ref() != Some(source) || *keys == Keys::default() {
            return;
        }
        a.keys = keys.clone();
        self.keys_used(cx);
        cx.notify();
    }

    /// The secrets of the archive open were used: they are kept
    /// [`KEYS_KEPT_FOR`] from now.
    fn keys_used(&mut self, cx: &mut Context<Self>) {
        if self.current().is_none_or(|a| a.keys == Keys::default()) {
            return;
        }
        self.forgetting = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(KEYS_KEPT_FOR).await;
            let _ = this.update(cx, |this, cx| this.forget_keys(cx));
        }));
    }

    /// Forgets the secrets of the archive open: what needs them asks again.
    pub fn forget_keys(&mut self, cx: &mut Context<Self>) {
        if let Shown::Open(a) = &mut self.shown {
            a.keys = Keys::default();
        }
        cx.notify();
    }

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
            menu_at: None,
            forgetting: None,
            focus: cx.focus_handle(),
            reading: None,
            scroll: UniformListScrollHandle::new(),
            outer: Vec::new(),
            copy: None,
            _typing: typing,
        }
    }

    /// The archive shown is left for the one in it at `copy`, made from `from`
    /// to open it: it is kept, to go back to, without its keys. If another
    /// is shown by now, there is nothing to go back to, and the copy is only
    /// read all the same.
    pub fn go_inside(&mut self, copy: PathBuf, from: &Source) {
        match &self.shown {
            Shown::Open(a) if a.source.as_ref() == Some(from) => {
                let mut archive = a.clone();
                archive.keys = Keys::default();
                self.outer.push(Outer {
                    archive,
                    copy: self.copy.take(),
                });
            }
            _ => self.leave_inside(),
        }
        self.copy = Some(copy);
    }

    /// The size of the file at `path` of the archive open, as its listing
    /// says; 0 if it does not.
    pub fn size_of(&self, path: &[String]) -> u64 {
        let Some((name, folder)) = path.split_last() else {
            return 0;
        };
        let Some(a) = self.archive() else {
            return 0;
        };
        match &a.listing {
            Some(listing) => listing
                .children(folder)
                .into_iter()
                .find(|c| &c.name == name)
                .map_or(0, |c| c.size),
            None => a
                .entries
                .iter()
                .find(|e| &e.name == name)
                .map_or(0, |e| e.size),
        }
    }

    /// Whether `source` is the copy of an archive opened from the one shown,
    /// or the one shown, opened from another.
    pub fn is_inside(&self, source: &Source) -> bool {
        self.copy
            .as_deref()
            .is_some_and(|copy| source.path() == Some(copy))
    }

    /// The archives the one shown was opened from are left: their copies go.
    pub fn leave_inside(&mut self) {
        for copy in self.outer.drain(..).filter_map(|o| o.copy) {
            remove_copy(&copy);
        }
        if let Some(copy) = self.copy.take() {
            remove_copy(&copy);
        }
    }

    /// Shows again the archive the one shown was opened from, `level` of those
    /// it is in, at the folder it was left at. The copies after it go.
    pub fn back_to(&mut self, level: usize, cx: &mut Context<Self>) {
        if level >= self.outer.len() {
            return;
        }
        let mut left = self.outer.split_off(level);
        let back = left.remove(0);
        for copy in left.into_iter().filter_map(|o| o.copy) {
            remove_copy(&copy);
        }
        if let Some(copy) = self.copy.take() {
            remove_copy(&copy);
        }
        self.copy = back.copy;
        let folder = back.archive.folder.clone();
        self.show(Shown::Open(back.archive), cx);
        self.enter(folder, cx);
    }

    /// The names of the archives the one shown was opened from, the first
    /// first.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn outer_names(&self) -> Vec<String> {
        self.outer.iter().map(|o| o.archive.name.clone()).collect()
    }

    /// Whether the archive shown was opened from another, and is only read.
    pub fn inside(&self) -> bool {
        self.copy.is_some()
    }

    pub fn show(&mut self, shown: Shown, cx: &mut Context<Self>) {
        // The first image, so the preview has something to show.
        self.chosen = match &shown {
            Shown::Open(a) => a.entries.iter().position(|e| e.kind == Kind::Image),
            _ => None,
        };
        self.shown = shown;
        self.reading = None;
        self.scroll.scroll_to_item(0, ScrollStrategy::Top);
        self.menu_at = None;
        // What it was opened with is kept a while, as what it is proved
        // with later is; another's go with it.
        self.forgetting = None;
        self.keys_used(cx);
        self.stop_preview();
        self.selected.clear();
        self.found.clear();
        self.search.update(cx, |s, cx| s.set_text("", 0..0, cx));
        self.more_at = None;
        self.look_at_signature(cx);
        cx.notify();
    }

    /// Shows the archive at `source`, being read with `keys`: what `heard`
    /// is told it found shows as it is, and what is done with it waits for
    /// [`ArchivePage::read_through`].
    pub fn start_reading(
        &mut self,
        source: Source,
        keys: Keys,
        heard: Heard,
        cx: &mut Context<Self>,
    ) {
        self.show(Shown::Open(Box::new(Archive::being_read(source, keys))), cx);
        let hearing = cx.spawn(async move |this, cx| {
            let mut wait = HEAR_EVERY;
            loop {
                cx.background_executor().timer(wait).await;
                let Ok(took) = this.update(cx, |this, cx| {
                    let at = Instant::now();
                    this.hear(cx);
                    at.elapsed()
                }) else {
                    return;
                };
                // A folder of many takes a while to show again: it waits
                // longer, so that the window still answers.
                wait = HEAR_EVERY.max(took * 4);
            }
        });
        self.reading = Some(Reading {
            heard,
            _hearing: hearing,
        });
    }

    /// Shows what the reading found since it last did.
    fn hear(&mut self, cx: &mut Context<Self>) {
        let Some(reading) = &self.reading else {
            return;
        };
        let found = std::mem::take(
            &mut *reading
                .heard
                .found
                .lock()
                .unwrap_or_else(PoisonError::into_inner),
        );
        cx.notify();
        if found.is_empty() {
            return;
        }
        self.keep_selection(cx, move |a| {
            let Some(listing) = &mut a.listing else {
                return;
            };
            let listing = Arc::make_mut(listing);
            let folder = a.folder.clone();
            let here = found
                .iter()
                .any(|e| e.path.len() > folder.len() && e.path.starts_with(&folder));
            for e in found.iter().filter(|e| !e.dir) {
                listing.files += 1;
                listing.unpacked += e.size;
            }
            listing.entries.extend(found);
            a.files = listing.files;
            a.unpacked = listing.unpacked;
            if here {
                a.show(folder);
            }
        });
    }

    /// The archive being read was read through, and is `archive`: the folder
    /// shown, what is selected and searched for stay as they are.
    pub fn read_through(&mut self, archive: Archive, cx: &mut Context<Self>) {
        if self.reading.take().is_none() || self.current().is_none() {
            self.show(Shown::Open(Box::new(archive)), cx);
            return;
        }
        let chosen = self.chosen.is_some();
        self.keep_selection(cx, move |a| {
            let folder = std::mem::take(&mut a.folder);
            *a = archive;
            // It comes with its top shown.
            if !folder.is_empty() {
                a.show(folder);
            }
        });
        self.keys_used(cx);
        if chosen {
            self.start_preview(cx);
        } else {
            self.chosen = self
                .current()
                .and_then(|a| a.entries.iter().position(|e| e.kind == Kind::Image));
        }
        cx.notify();
    }

    /// Changes the archive shown with `change`: what is selected and chosen
    /// in its folder stays so, by their names, and the search finds again.
    fn keep_selection(&mut self, cx: &mut Context<Self>, change: impl FnOnce(&mut Archive)) {
        let Shown::Open(a) = &mut self.shown else {
            return;
        };
        let name = |i: &usize| a.entries.get(*i).map(|e| e.name.clone());
        let selected: HashSet<String> = self.selected.iter().filter_map(name).collect();
        let chosen = self.chosen.as_ref().and_then(name);
        change(a);
        self.selected = a
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| !selected.is_empty() && selected.contains(&e.name))
            .map(|(i, _)| i)
            .collect();
        self.chosen = chosen.and_then(|n| a.entries.iter().position(|e| e.name == n));
        let query = self.search.read(cx).text().to_owned();
        if !query.trim().is_empty()
            && let Some(listing) = &a.listing
        {
            self.found = listing.search(&query);
        }
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
    pub fn open_menu_for_shot(&mut self, at: Point<Pixels>, cx: &mut Context<Self>) {
        self.menu_at = Some(at);
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
        // It waits for the archive to be read through.
        if self.reading.is_some() {
            return;
        }
        let Some(a) = self.current() else {
            return;
        };
        let Some(entry) = self.chosen.and_then(|i| a.entries.get(i)) else {
            return;
        };
        // A link's data is where it leads, which the preview says, if the
        // archive does.
        if entry.kind == Kind::Link {
            return;
        }
        let format = picture_format(&entry.name);
        // A script is a text; a program that is not says it is not.
        if format.is_none() && !matches!(entry.kind, Kind::Text | Kind::Other | Kind::Program) {
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
        self.keys_used(cx);
        let cancel = Cancel::new();
        let control = Control {
            progress: None,
            cancel: Some(cancel.clone()),
            on_damaged: None,
            on_unnamable: None,
            on_links_outside: None,
            on_gone: None,
        };
        let reading = cx.background_executor().spawn(async move {
            match preview_entry(&source, &path, &keys, MOST_PREVIEWED, &control) {
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
        self.current()
            .map(|a| {
                self.shown(a, cx)
                    .into_iter()
                    .map(|(_, e, _)| e.name)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Shows what the reading found, as the screenshots draw it.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn hear_for_shot(&mut self, cx: &mut Context<Self>) {
        self.hear(cx);
    }

    /// The names of the entries selected, for tests.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn selected_names(&self) -> Vec<String> {
        let Some(a) = self.current() else {
            return Vec::new();
        };
        self.selected
            .iter()
            .filter_map(|i| a.entries.get(*i))
            .map(|e| e.name.clone())
            .collect()
    }

    /// Where in the archive the list is, for tests.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn shown_folder(&self) -> Vec<String> {
        self.current().map(|a| a.folder.clone()).unwrap_or_default()
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

    /// Whether the preview has nothing to show of the file chosen but its tag,
    /// and is not reading it, for tests.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn previews_nothing(&self) -> bool {
        matches!(self.preview, Preview::Nothing)
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

    /// The archive open, if one is, read through: what is done with it waits
    /// for that.
    pub fn archive(&self) -> Option<&Archive> {
        self.current().filter(|_| self.reading.is_none())
    }

    /// The archive shown, read through or being read.
    fn current(&self) -> Option<&Archive> {
        match &self.shown {
            Shown::Open(a) => Some(a.as_ref()),
            _ => None,
        }
    }

    /// Whether the archive shown is still being read.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn is_reading(&self) -> bool {
        self.reading.is_some()
    }

    /// The archive open, told apart by where it is and its name, and the folder
    /// shown in it.
    pub fn at(&self) -> Option<(Option<Source>, String, Vec<String>)> {
        match &self.shown {
            Shown::Open(a) => Some((a.source.clone(), a.name.clone(), a.folder.clone())),
            _ => None,
        }
    }

    /// Shows `folder` of the open archive.
    pub fn enter(&mut self, folder: Vec<String>, cx: &mut Context<Self>) {
        if let Shown::Open(a) = &mut self.shown {
            a.show(folder);
            self.scroll.scroll_to_item(0, ScrollStrategy::Top);
            self.stop_preview();
            self.menu_at = None;
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
        if let Some(i) = self.chosen {
            self.scroll.scroll_to_item(i, ScrollStrategy::Center);
        }
        self.start_preview(cx);
        cx.notify();
    }

    /// What to list: the folder's entries; with something typed, what the
    /// search found, each with where it is.
    #[cfg(all(test, feature = "test-hooks"))]
    fn shown(&self, a: &Archive, cx: &gpui::App) -> Vec<Row> {
        let listed = self.listed(a, cx);
        (0..listed.len())
            .filter_map(|n| self.listed_at(a, &listed, n))
            .collect()
    }

    /// What the list holds, without making its rows.
    fn listed(&self, a: &Archive, cx: &gpui::App) -> Listed {
        let query = self.search.read(cx).text().trim().to_lowercase();
        if !query.is_empty() && a.listing.is_some() {
            return Listed::Found(self.found.len().min(MOST_FOUND));
        }
        if query.is_empty() {
            return Listed::Folder(a.entries.len());
        }
        Listed::Matching(
            a.entries
                .iter()
                .enumerate()
                .filter(|(_, e)| e.name.to_lowercase().contains(&query))
                .map(|(i, _)| i)
                .collect(),
        )
    }

    /// The `n`th the list holds: its index, among the folder's entries or
    /// what was found, the entry, and where it was found.
    fn listed_at(&self, a: &Archive, listed: &Listed, n: usize) -> Option<Row> {
        match listed {
            Listed::Found(_) => self
                .found
                .get(n)
                .map(|f| (n, Entry::of(&f.child), Some(f.path.clone()))),
            Listed::Folder(_) => a.entries.get(n).map(|e| (n, e.clone(), None)),
            Listed::Matching(at) => at
                .get(n)
                .and_then(|i| a.entries.get(*i).map(|e| (*i, e.clone(), None))),
        }
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
        // How far into its file the reading is, in thousandths, while it is read.
        let reading = self
            .reading
            .as_ref()
            .map(|r| r.heard.read.load(Ordering::Relaxed));
        // What needs it read through waits, its button there but dim.
        let waits = |b: gpui::Stateful<Div>, asked: fn() -> ArchiveEvent| {
            if reading.is_some() {
                b.opacity(0.5).cursor_default()
            } else {
                b.on_click(cx.listener(move |_, _, _, cx| cx.emit(asked())))
            }
        };
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
                            .child(match reading {
                                Some(0) => chip(tr!("Reading…"), c.accent, c.on_accent)
                                    .debug_selector(|| "archive-reading".to_owned()),
                                Some(thousandths) => chip(
                                    tr!("Reading… {percent}%", percent = thousandths / 10),
                                    c.accent,
                                    c.on_accent,
                                )
                                .debug_selector(|| "archive-reading".to_owned()),
                                None => chip(
                                    format!("{} · {}%", size(a.packed), a.ratio()),
                                    c.surface,
                                    c.on_tonal,
                                ),
                            })
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
                    .child(waits(
                        button("extract", tr!("Extract"), Fill::Accent, Size::Large).px(ui(28.)),
                        || ArchiveEvent::Extract,
                    ))
                    // A tar archive has to be written whole again, and
                    // packwing does not do that yet.
                    .when(
                        reading.is_none()
                            && !self.inside()
                            && a.listing
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
                    .child(waits(
                        button("test", tr!("Test"), Fill::Raised, Size::Large),
                        || ArchiveEvent::Test,
                    ))
                    .child(
                        icon_button("archive-more", "more", 48.)
                            .bg(rgb(c.surface))
                            .when(reading.is_some(), |d| d.opacity(0.5).cursor_default())
                            .when(reading.is_none(), |d| {
                                d.on_click(cx.listener(|this, e: &ClickEvent, _, cx| {
                                    this.more_at = Some(e.position());
                                    cx.notify();
                                }))
                            }),
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

    /// The archive's file, if it is on this computer.
    pub fn local_file(&self) -> Option<std::path::PathBuf> {
        let (storage, path) = self.archive()?.source.as_ref()?.place()?;
        storage.local_path(&path)
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
        // A copy is only read: what is made beside it would be lost with it.
        let inside = self.inside();
        let here = self
            .archive()
            .filter(|_| !inside)
            .and_then(|a| a.source.as_ref())
            .and_then(|s| s.place())
            .and_then(|(storage, path)| storage.local_path(&path));
        let cut = here
            .as_deref()
            .and_then(|h| h.file_name())
            .is_some_and(|n| crate::explorer::is_cut_part_name(&n.to_string_lossy()));
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
                        .when(!inside, |d| {
                            d.child(item("menu-sign", tr!("Sign…"), || ArchiveEvent::Sign))
                        })
                        .child(item("menu-checksums", tr!("Checksums…"), || {
                            ArchiveEvent::Checksums
                        }))
                        // Made beside its file, on this computer.
                        .when(here.is_some(), |d| {
                            d.child(item("menu-convert", tr!("Convert…"), || {
                                ArchiveEvent::Convert
                            }))
                            .child(item(
                                "menu-split",
                                tr!("Split into parts…"),
                                || ArchiveEvent::Split,
                            ))
                        })
                        .when(cut, |d| {
                            d.child(item("menu-join", tr!("Join the parts"), || {
                                ArchiveEvent::Join
                            }))
                        })
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

    /// Whether what is selected can be renamed or deleted: in a folder shown,
    /// not a search's, of an archive packwing changes in place.
    fn changeable(&self, cx: &gpui::App) -> bool {
        let Some(a) = self.archive() else {
            return false;
        };
        let searching = !self.search.read(cx).text().trim().is_empty() && a.listing.is_some();
        !searching
            && !self.inside()
            && !self.selected.is_empty()
            && a.listing
                .as_ref()
                .is_some_and(|l| packwing_core::can_add(l))
    }

    /// Whether what is selected can be renamed: one entry, of an archive that
    /// can be changed.
    fn renamable(&self, cx: &gpui::App) -> bool {
        self.changeable(cx)
            && self.selected.len() == 1
            && self
                .archive()
                .and_then(|a| a.listing.as_ref())
                .is_some_and(|l| packwing_core::can_rename(l))
    }

    /// The entry selected, when there is one alone.
    fn one_selected(&self) -> Option<&Entry> {
        match (self.current(), self.selected.len()) {
            (Some(a), 1) => self.selected.first().and_then(|i| a.entries.get(*i)),
            _ => None,
        }
    }

    /// The right-click menu of what is selected, at `at`.
    fn entry_menu(&self, at: Point<Pixels>, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let c = colors();
        let item =
            |id: &'static str, label: String, key: &'static str, asked: fn() -> ArchiveEvent| {
                menu_item(id, label, key).on_click(cx.listener(move |this, _, _, cx| {
                    this.menu_at = None;
                    cx.emit(asked());
                    cx.notify();
                }))
            };
        let line = || div().my(ui(4.)).mx(ui(10.)).h(ui(1.)).bg(rgb(c.field));
        let folder = self.current().map(|a| a.folder.clone()).unwrap_or_default();
        // One file opens as a double click does; one folder is gone into.
        // A link has nothing of its own to open.
        let open = self
            .one_selected()
            .filter(|e| e.kind != Kind::Link)
            .map(|e| {
                let mut path = folder.clone();
                path.push(e.name.clone());
                (path, e.kind)
            });
        let changeable = self.changeable(cx);
        let renamable = self.renamable(cx);
        deferred(
            anchored()
                .position(at)
                .snap_to_window_with_margin(px(8.))
                .child(
                    menu_panel("entry-menu")
                        .min_w(ui(200.))
                        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                        .on_mouse_down_out(cx.listener(|this, e: &MouseDownEvent, _, cx| {
                            // A right click opens the next one.
                            if e.button != MouseButton::Right {
                                this.menu_at = None;
                                cx.notify();
                            }
                        }))
                        .when_some(open, |d, (path, kind)| {
                            let dir = kind == Kind::Folder;
                            let (app, inside) = (path.clone(), path.clone());
                            let ask = |id: &'static str,
                                       label: String,
                                       path: Vec<String>,
                                       asked: fn(Vec<String>) -> ArchiveEvent| {
                                menu_item(id, label, "").on_click(cx.listener(
                                    move |this, _, _, cx| {
                                        this.menu_at = None;
                                        cx.emit(asked(path.clone()));
                                        cx.notify();
                                    },
                                ))
                            };
                            d.child(
                                menu_item("menu-open", tr!("Open"), "").on_click(cx.listener(
                                    move |this, _, _, cx| {
                                        this.menu_at = None;
                                        if dir {
                                            this.enter(path.clone(), cx);
                                        } else {
                                            cx.emit(ArchiveEvent::OpenEntry(path.clone()));
                                        }
                                        cx.notify();
                                    },
                                )),
                            )
                            .when(!dir, |d| {
                                d.child(ask(
                                    "menu-app",
                                    tr!("Open with default app"),
                                    app,
                                    ArchiveEvent::OpenEntryWithApp,
                                ))
                            })
                            // What Open opens in packwing already is not offered twice.
                            .when(!dir && kind != Kind::Archive, |d| {
                                d.child(ask(
                                    "menu-as-archive",
                                    tr!("Open as archive"),
                                    inside,
                                    ArchiveEvent::OpenEntryAsArchive,
                                ))
                            })
                        })
                        .child(item("menu-extract", tr!("Extract…"), "", || {
                            ArchiveEvent::Extract
                        }))
                        .when(changeable, |d| {
                            d.child(line())
                                .when(renamable, |d| {
                                    d.child(item("menu-rename", tr!("Rename"), "F2", || {
                                        ArchiveEvent::Rename
                                    }))
                                })
                                .child(
                                    item("menu-delete", tr!("Delete"), "Del", || {
                                        ArchiveEvent::Delete
                                    })
                                    .text_color(rgb(c.bad_text)),
                                )
                        }),
                ),
        )
        .with_priority(1)
    }

    /// The list's keys, as the browser's: F2 renames, Delete deletes, Escape
    /// closes the menu.
    fn on_key(&mut self, e: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        let key = e.keystroke.key.as_str();
        match key {
            "escape" if self.menu_at.is_some() => {
                self.menu_at = None;
                cx.notify();
            }
            "f2" if self.renamable(cx) => cx.emit(ArchiveEvent::Rename),
            "delete" if self.changeable(cx) => cx.emit(ArchiveEvent::Delete),
            "up" | "down" | "pageup" | "pagedown" | "home" | "end" if self.menu_at.is_none() => {
                self.move_choice(key, e.keystroke.modifiers.shift, cx)
            }
            "enter" if self.menu_at.is_none() => self.open_chosen(cx),
            "backspace" if self.menu_at.is_none() => self.go_up(cx),
            _ => return,
        }
        cx.stop_propagation();
    }

    /// Chooses the row `key` moves to (see [`list_keys`]), showing it in the
    /// preview; with `range`, all from the one chosen before to it.
    fn move_choice(&mut self, key: &str, range: bool, cx: &mut Context<Self>) {
        let Some(a) = self.archive() else {
            return;
        };
        let listed = self.listed(a, cx);
        let index_at = |n| self.listed_at(a, &listed, n).map(|(i, _, _)| i);
        let len = listed.len();
        let from = self
            .chosen
            .and_then(|c| (0..len).find(|&n| index_at(n) == Some(c)));
        let page = list_keys::rows_per_page(&self.scroll);
        let Some(to) = list_keys::moved(key, from, len, page) else {
            return;
        };
        let Some(i) = index_at(to) else {
            return;
        };
        self.select(i, false, range);
        self.scroll.scroll_to_item(to, ScrollStrategy::Nearest);
        self.start_preview(cx);
        cx.notify();
    }

    /// Opens the one row chosen, as a double click does: a folder gone into, a
    /// file opened.
    fn open_chosen(&mut self, cx: &mut Context<Self>) {
        if self.selected.len() != 1 {
            return;
        }
        let Some(a) = self.archive() else {
            return;
        };
        let listed = self.listed(a, cx);
        let row = (0..listed.len())
            .filter_map(|n| self.listed_at(a, &listed, n))
            .find(|(i, _, _)| Some(*i) == self.chosen);
        let Some((_, entry, found)) = row else {
            return;
        };
        let path = found.unwrap_or_else(|| {
            let mut path = a.folder.clone();
            path.push(entry.name.clone());
            path
        });
        match entry.kind {
            Kind::Folder => self.reveal(&path, true, cx),
            Kind::Link => {}
            _ => cx.emit(ArchiveEvent::OpenEntry(path)),
        }
    }

    /// Shows the folder the one shown is in; from its top, the archive it was
    /// opened from, if it was.
    fn go_up(&mut self, cx: &mut Context<Self>) {
        let Some(mut folder) = self.archive().map(|a| a.folder.clone()) else {
            return;
        };
        if folder.pop().is_some() {
            self.enter(folder, cx);
        } else if let Some(level) = self.outer.len().checked_sub(1) {
            cx.emit(ArchiveEvent::Back(level));
        }
    }

    /// A row of the list, of the entry at `i`, or what was found there.
    fn row(
        &self,
        a: &Archive,
        (i, e, found): Row,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let c = colors();
        let searching = found.is_some();
        {
            let on = !searching && self.selected.contains(&i);
            let file_path =
                (!matches!(e.kind, Kind::Folder | Kind::Link) && found.is_none()).then(|| {
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
            let inside = (e.kind == Kind::Folder).then(|| {
                let mut folder = a.folder.clone();
                folder.push(e.name.clone());
                folder
            });
            div()
                .debug_selector(|| format!("entry-{i}"))
                .id(("entry", i))
                .w_full()
                .flex()
                .items_center()
                .gap(ui(14.))
                .h(ui(ROW_H))
                .px(ui(14.))
                .py(ui(6.))
                .rounded(ui(18.))
                .cursor_pointer()
                .when(on, |d| d.bg(rgb(c.chosen)))
                .when(!on, |d| d.hover(|s| s.bg(rgb(c.hover))))
                // What is selected, or this alone, and its menu; none in a
                // search, nor while the archive is read.
                .when(found.is_none(), |d| {
                    d.on_mouse_down(
                        MouseButton::Right,
                        cx.listener(move |this, e: &MouseDownEvent, window, cx| {
                            if !this.selected.contains(&i) {
                                this.select(i, false, false);
                                this.start_preview(cx);
                            }
                            if this.reading.is_none() {
                                this.menu_at = Some(e.position);
                            }
                            window.focus(&this.focus, cx);
                            cx.stop_propagation();
                            cx.notify();
                        }),
                    )
                })
                // A click selects; a double click opens.
                .on_click(cx.listener(move |this, e: &gpui::ClickEvent, window, cx| {
                    window.focus(&this.focus, cx);
                    if let Some(path) = &found {
                        this.reveal(path, dir, cx);
                        return;
                    }
                    match (&inside, &file_path) {
                        (Some(folder), _) if e.click_count() >= 2 => this.enter(folder.clone(), cx),
                        (_, Some(path)) if e.click_count() >= 2 && this.reading.is_none() => {
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
                .child(kind_tile(e.kind, e.tag(), 40., 40., 14., 11.))
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
                        // A link's size is its target's length.
                        .when(e.kind != Kind::Link, |d| d.child(size(e.size))),
                )
        }
    }

    fn contents(&self, a: &Archive, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let crumb = self.breadcrumb(a, cx);
        let query = self.search.read(cx).text().trim().to_string();
        let more = self.found.len().saturating_sub(MOST_FOUND);
        let searching = !query.is_empty() && a.listing.is_some();
        let count = self.listed(a, cx).len();
        // Only the rows in sight are made: a folder can hold many thousands.
        let rows = uniform_list(
            "entries",
            count,
            cx.processor(|this, range: Range<usize>, _, cx| {
                let Some(a) = this.current() else {
                    return Vec::new();
                };
                let listed = this.listed(a, cx);
                range
                    .filter_map(|n| this.listed_at(a, &listed, n))
                    .map(|row| this.row(a, row, cx).into_any_element())
                    .collect::<Vec<_>>()
            }),
        )
        .track_scroll(&self.scroll)
        // As tall as its rows, a row's [`ROW_H`], while the panel has room:
        // a list that scrolls says nothing of its height itself.
        .h(ui(ROW_H * count as f32))
        .flex_shrink(1.)
        .min_h_0();
        let empty = count == 0;
        let reading = self.reading.is_some();
        panel()
            .track_focus(&self.focus)
            .on_key_down(cx.listener(Self::on_key))
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
                    .child(div().flex_1().min_w_0().child(crumb)),
            )
            .child(rows)
            .when(empty, |d| {
                d.child(
                    div()
                        .px(ui(14.))
                        .py(ui(12.))
                        .text_size(ui(14.))
                        .text_color(rgb(c.muted))
                        .child(if reading && query.is_empty() {
                            tr!("Reading what’s inside…").to_owned()
                        } else if searching {
                            tr!("Nothing in this archive matches “{query}”.", query = query)
                        } else if query.is_empty() {
                            tr!("This folder is empty.").to_owned()
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
            })
    }

    /// The folder shown, from the archive's top: each step leads back to
    /// it. The samples show theirs as text.
    fn breadcrumb(&self, a: &Archive, cx: &mut Context<Self>) -> Div {
        let c = colors();
        // The archives it was opened from first, each back to where it was left;
        // then its own top, and the folders down to the one shown.
        let mut steps: Vec<(String, Step)> = self
            .outer
            .iter()
            .enumerate()
            .map(|(level, o)| (o.archive.name.clone(), Step::Back(level)))
            .collect();
        if a.listing.is_some() {
            steps.push((a.stem().to_owned(), Step::Folder(0)));
        }
        let navigable = a.listing.is_some();
        steps.extend(
            a.folder
                .iter()
                .enumerate()
                .map(|(i, name)| (name.clone(), Step::Folder(i + 1))),
        );
        let last = steps.len().saturating_sub(1);
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
            .children(steps.into_iter().enumerate().map(move |(i, (step, to))| {
                let here = i == last;
                let goes = !here && (navigable || matches!(to, Step::Back(_)));
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
                            .when(goes, |d| {
                                d.cursor_pointer()
                                    .hover(|s| s.text_color(rgb(c.text)))
                                    .on_click(cx.listener(move |this, _, _, cx| match to {
                                        Step::Back(level) => cx.emit(ArchiveEvent::Back(level)),
                                        Step::Folder(depth) => {
                                            let folder = this
                                                .current()
                                                .map(|a| a.folder[..depth].to_vec())
                                                .unwrap_or_default();
                                            this.enter(folder, cx);
                                        }
                                    }))
                            })
                            // An archive it was opened from: a box before its name.
                            .when_some(
                                match to {
                                    Step::Back(level) => Some(level),
                                    Step::Folder(_) => None,
                                },
                                |d, level| {
                                    d.debug_selector(move || format!("archive-back-{level}"))
                                        .flex()
                                        .items_center()
                                        .gap(ui(4.))
                                        .child(icon("archive", 13., c.muted))
                                },
                            )
                            .child(step),
                    )
            }))
    }

    /// What the page shows with no archive open: an invitation, the file
    /// being read, or why it could not be.
    fn empty(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let (title, line, offer) = match &self.shown {
            Shown::Failed { name, why } => (name.clone(), why.clone(), true),
            _ => (
                tr!("Open an archive").to_owned(),
                tr!("ZIP, 7z and tar archives, disc images and packages, compressed or encrypted with age: .zip, .7z, .tar.gz, .tar.zst, .iso, .rpm, .pkg, .zst.age…")
                    .to_owned(),
                true,
            ),
        };
        let failed = matches!(self.shown, Shown::Failed { .. });
        // An archive opened from another that could not be: the way back.
        let back = self
            .outer
            .len()
            .checked_sub(1)
            .map(|level| (level, self.outer[level].archive.name.clone()));
        let open_fill = if back.is_some() {
            Fill::Raised
        } else {
            Fill::Accent
        };
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
            .when_some(back, |d, (level, name)| {
                d.child(
                    button(
                        "archive-back",
                        tr!("Back to {name}", name = name),
                        Fill::Accent,
                        Size::Large,
                    )
                    .mt(ui(8.))
                    .px(ui(28.))
                    .on_click(cx.listener(move |_, _, _, cx| cx.emit(ArchiveEvent::Back(level)))),
                )
            })
            .when(offer, |d| {
                d.child(
                    button(
                        "open-archive",
                        if failed {
                            tr!("Open another…")
                        } else {
                            tr!("Open an archive…")
                        },
                        open_fill,
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
            // Its corners follow the panel's, inside its padding.
            .rounded(ui(28. - crate::widgets::PANE_GAP))
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
                (Some(e), _) => d.child(kind_tile(e.kind, e.tag(), 64., 64., 20., 15.)),
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
            .p(ui(crate::widgets::PANE_GAP))
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
                        match (e.kind, e.modified.is_empty()) {
                            (Kind::Link, true) => tr!("Symbolic link"),
                            (Kind::Link, false) => format!("{} · {}", tr!("Symbolic link"), e.modified),
                            (_, true) => size(e.size),
                            (_, false) => format!("{} · {}", size(e.size), e.modified),
                        },
                    ))
                    .children(e.target.map(|target| {
                        div()
                            .debug_selector(|| "link-target".to_owned())
                            .mt(ui(8.))
                            .text_size(ui(13.))
                            .text_color(rgb(c.soft))
                            .child(tr!("Leads to {target}", target = target))
                    }))
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
        let page = div()
            .size_full()
            .flex()
            .flex_col()
            .gap(ui(crate::widgets::PANE_GAP));
        let Some(a) = self.current() else {
            return page.child(self.empty(cx));
        };
        let more = self.more_at.map(|at| self.more_menu(at, cx));
        let menu = self.menu_at.map(|at| self.entry_menu(at, cx));
        page.child(self.search_bar(cx))
            .child(self.header(a, cx))
            .children(more)
            .children(menu)
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .items_start()
                    .gap(ui(crate::widgets::PANE_GAP))
                    .child(self.contents(a, cx))
                    .child(self.preview(a)),
            )
    }
}
