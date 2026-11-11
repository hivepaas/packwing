//! Browsing a storage, as a system's file manager does: a folder's items in
//! columns, a page at a time, chosen with the mouse or the keys, with a right-click
//! menu. It knows a storage only as a [`Storage`]; what it cannot do itself, such as
//! opening an archive, it says through [`ExplorerEvent`].
//!
//! A click chooses, with Ctrl or Cmd for more and Shift for a range; a double
//! click or Enter opens; Backspace goes up; F2 renames, Delete deletes, Space shows
//! a file, F5 refreshes. Items are kept by name, so that what is chosen stays chosen
//! however they are sorted or filtered.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gpui::{
    App, ClipboardItem, Context, Div, ElementId, Entity, EventEmitter, FocusHandle, Focusable,
    FontWeight, KeyDownEvent, MouseButton, MouseDownEvent, Pixels, Point, Render, ScrollStrategy,
    Stateful, Subscription, Task, UniformListScrollHandle, Window, anchored, canvas, deferred, div,
    prelude::*, px, rgb, uniform_list,
};
use packwing_core::{
    Cancel, Counted, Item, Lister, Signed, Source, Storage, Stored, bad_name, count, folded_query,
    is_signature_name, name_holds, signature_name,
};
use packwing_i18n::{tr, tr_n};

use crate::colors::colors;
use crate::editors;
use crate::file_manager;
use crate::input::{Changed, TextInput};
use crate::list_keys;
use crate::model::{Kind, date, tag_of};
use crate::prefs;
use crate::signatures::{self, Looking};
use crate::text_size::ui;
use crate::widgets::{Fill, Size, button, icon, icon_button, kind_tile, menu_item, menu_panel};

/// How many items are asked for at a time.
pub const PAGE: usize = 1000;

/// The letters on a storage's tile, for a folder on this computer: a tag, as a
/// file's are, not words to translate.
pub const LOCAL_TAG: &str = "HD";

/// How tall a row is. All are, for the list to draw only those in sight.
const ROW: f32 = 48.;

/// What the explorer needs someone else to do.
pub enum ExplorerEvent {
    /// Open this archive.
    OpenArchive(Source),
    /// Open this archive, and offer to extract it.
    Extract(Source),
    /// Make a new archive of these.
    Compress(Vec<PathBuf>),
    /// Show this file, by its name in the folder.
    View(Item),
    /// Ask whether to delete these, by their names in the folder.
    AskDelete(Vec<String>),
    /// Ask where to copy or move what is chosen.
    Transfer { moving: bool },
    /// Ask how to sign this file, by its name in the folder.
    Sign(String),
    /// Show this file's checksums, by its name in the folder.
    Checksums(String),
    /// Ask how to cut this file into parts, by its name in the folder.
    Split(String),
    /// Join the parts this one is of, by its name in the folder.
    Join(String),
    /// Ask how to make an archive of what this one holds, in another format.
    Convert(String),
}

/// Whether `item` is a part of a file cut as 7-Zip cuts one: `name.001`.
pub fn is_cut_part(item: &Item) -> bool {
    !item.dir && is_cut_part_name(&item.name)
}

/// Whether `name` is a part's, as [`is_cut_part`] says.
pub fn is_cut_part_name(name: &str) -> bool {
    name.rsplit_once('.').is_some_and(|(base, n)| {
        !base.is_empty() && n.len() >= 3 && n.bytes().all(|b| b.is_ascii_digit())
    })
}

/// A storage, by what the user calls it.
#[derive(Clone)]
pub struct Place {
    pub name: String,
    /// The letters on its tile.
    pub tag: &'static str,
    pub storage: Arc<dyn Storage>,
    /// It asks for this computer's password before it opens.
    pub asks_first: bool,
}

impl Place {
    /// A folder on this computer.
    pub fn local(name: String, path: &std::path::Path) -> Place {
        Place {
            name,
            tag: LOCAL_TAG,
            storage: Arc::new(packwing_core::LocalFolder::new(path)),
            asks_first: false,
        }
    }

    /// The tile's tint: folders on this computer are plain.
    pub fn tint(&self, c: &crate::colors::Palette) -> crate::colors::Tint {
        if self.tag == LOCAL_TAG {
            c.plain_tag
        } else {
            c.blue_tag
        }
    }
}

/// A column the items sort by.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Column {
    Name,
    Size,
    Modified,
}

/// Where the listing has got to.
#[derive(Clone, Debug, PartialEq, Eq)]
enum State {
    Idle,
    /// Nothing to show yet.
    Opening,
    /// More on their way, or the same ones again.
    Loading,
    Failed(String),
}

/// A name being typed.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Naming {
    NewFolder,
    Rename(String),
}

/// What a right click was on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MenuFor {
    Selected,
    Folder,
}

/// A count of what a folder holds, going on or done.
struct Count {
    name: String,
    cancel: Cancel,
    now: Arc<Mutex<Counted>>,
    /// How many folders could not be read, once it is done.
    done: Option<u64>,
    _task: Task<()>,
}

/// What is known of the signature of the one file chosen.
struct SignatureOf {
    name: String,
    /// None while it is looked at.
    signed: Option<Signed>,
    _looking: Option<Looking>,
}

/// A count as the info panel shows it.
pub struct CountShown {
    pub counted: Counted,
    pub done: bool,
    pub unread: u64,
}

pub struct Explorer {
    focus: FocusHandle,
    /// Choosing a folder to put things in: only folders show, a click goes into
    /// one, and there is no menu or search.
    picking: bool,
    /// Choosing an archive: they show beside the folders, and a click opens one.
    archives: bool,
    place: Option<Place>,
    folder: Vec<String>,
    items: Vec<Item>,
    /// Indexes in `items` of those shown, in the order shown.
    shown: Vec<usize>,
    lister: Option<Arc<Mutex<Box<dyn Lister>>>>,
    more: bool,
    state: State,
    /// Told apart from one begun later: what ends after is not shown.
    era: u64,
    selected: BTreeSet<String>,
    /// Where a Shift click's range starts.
    anchor: Option<String>,
    sort: Column,
    ascending: bool,
    show_hidden: bool,
    search: Entity<TextInput>,
    query: String,
    scroll: UniformListScrollHandle,
    menu: Option<(Point<Pixels>, MenuFor)>,
    naming: Option<Naming>,
    name_field: Entity<TextInput>,
    /// What went wrong with the last thing asked.
    problem: Option<String>,
    count: Option<Count>,
    signature: Option<SignatureOf>,
    /// The columns its width leaves room for, as it was last drawn.
    columns: std::rc::Rc<std::cell::Cell<Columns>>,
    _typing: Subscription,
}

/// The columns beside a name, as the width there is allows: Modified goes
/// first, then Size, so that a name always has room.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Columns {
    #[default]
    All,
    Size,
    Name,
}

impl Columns {
    /// The columns for a list `width` wide, in the units of [`ui`]: a name
    /// keeps at least 140 of them beside its tag, the rows' padding and gaps.
    fn for_width(width: f32) -> Columns {
        // Padding 28, the tag 36, a gap 12, and the name's 140.
        const NAME: f32 = 28. + 36. + 12. + 140.;
        // A gap and Size's 90; a gap and Modified's 100.
        const SIZE: f32 = 12. + 90.;
        const MODIFIED: f32 = 12. + 100.;
        if width >= NAME + SIZE + MODIFIED {
            Columns::All
        } else if width >= NAME + SIZE {
            Columns::Size
        } else {
            Columns::Name
        }
    }

    fn size(self) -> bool {
        self != Columns::Name
    }

    fn modified(self) -> bool {
        self == Columns::All
    }
}

impl EventEmitter<ExplorerEvent> for Explorer {}

impl Focusable for Explorer {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

/// Whether opening a file with its default app could run it. Such a file is only
/// shown or edited here: a double click must not start a program.
pub fn runs_when_opened(item: &Item) -> bool {
    const ALSO: [&str; 13] = [
        "desktop", "command", "scr", "com", "vbs", "wsf", "hta", "appimage", "run", "bin", "py",
        "pl", "rb",
    ];
    let ext = item
        .name
        .rsplit_once('.')
        .map(|(_, e)| e.to_ascii_lowercase())
        .unwrap_or_default();
    Kind::of(&item.name, false) == Kind::Program
        || ALSO.contains(&ext.as_str())
        || item.mode.is_some_and(|m| m & 0o111 != 0)
}

/// Whether `item` is a file that can have a signature of its own.
pub fn can_be_signed(item: &Item) -> bool {
    !item.dir && !item.link && !is_signature_name(&item.name)
}

/// Whether `item` is opened as an archive, going by its name.
pub fn is_archive(item: &Item) -> bool {
    !item.dir && Kind::of(&item.name, false) == Kind::Archive
}

impl Explorer {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| TextInput::said(|| tr!("Search this folder"), cx));
        let typing = cx.subscribe(&search, |this: &mut Self, field, _: &Changed, cx| {
            this.query = folded_query(field.read(cx).text());
            this.rebuild();
            cx.notify();
        });
        let name_field = cx.new(|cx| TextInput::said(|| tr!("Name"), cx));
        Explorer {
            focus: cx.focus_handle(),
            picking: false,
            archives: false,
            place: None,
            folder: Vec::new(),
            items: Vec::new(),
            shown: Vec::new(),
            lister: None,
            more: false,
            state: State::Idle,
            era: 0,
            selected: BTreeSet::new(),
            anchor: None,
            sort: Column::Name,
            ascending: true,
            show_hidden: prefs::prefs(cx).show_hidden,
            search,
            query: String::new(),
            scroll: UniformListScrollHandle::new(),
            menu: None,
            naming: None,
            name_field,
            problem: None,
            count: None,
            signature: None,
            columns: Default::default(),
            _typing: typing,
        }
    }

    /// One for choosing a folder.
    pub fn picker(cx: &mut Context<Self>) -> Self {
        Explorer {
            picking: true,
            ..Explorer::new(cx)
        }
    }

    /// One for choosing an archive to open.
    pub fn archive_picker(cx: &mut Context<Self>) -> Self {
        Explorer {
            archives: true,
            ..Explorer::picker(cx)
        }
    }

    /// The folder shown, from the top of the storage.
    pub fn folder(&self) -> &[String] {
        &self.folder
    }

    /// Says `what` under the list.
    pub fn say(&mut self, what: Option<String>, cx: &mut Context<Self>) {
        self.problem = what;
        cx.notify();
    }

    /// Asks for what is chosen to be copied or moved.
    pub fn ask_transfer(&mut self, moving: bool, cx: &mut Context<Self>) {
        if !self.selected.is_empty() {
            self.menu = None;
            cx.emit(ExplorerEvent::Transfer { moving });
        }
    }

    /// Asks for the one file chosen to be signed.
    pub fn ask_sign(&mut self, cx: &mut Context<Self>) {
        if let [one] = self.selection().as_slice()
            && can_be_signed(one)
        {
            self.menu = None;
            cx.emit(ExplorerEvent::Sign(one.name.clone()));
        }
    }

    /// Shows the checksums of the one file chosen.
    pub fn ask_checksums(&mut self, cx: &mut Context<Self>) {
        if let [one] = self.selection().as_slice()
            && !one.dir
            && !one.link
        {
            self.menu = None;
            cx.emit(ExplorerEvent::Checksums(one.name.clone()));
        }
    }

    /// What the signature of the file `name` says so far: an outer none if it is
    /// not known of that file, an inner none while it is looked at.
    pub fn signature_of(&self, name: &str) -> Option<Option<&Signed>> {
        self.signature
            .as_ref()
            .filter(|s| s.name == name)
            .map(|s| s.signed.as_ref())
    }

    /// Looks at the signature of the file chosen again, as it or the keys trusted
    /// may have changed.
    pub fn forget_signature(&mut self, cx: &mut Context<Self>) {
        self.signature = None;
        cx.notify();
    }

    /// Reads all of the file chosen to check its signature: it is not done by itself
    /// across a network.
    pub fn check_signature(&mut self, cx: &mut Context<Self>) {
        if let Some(name) = self.signature.as_ref().map(|s| s.name.clone()) {
            self.look_at_signature(name, true, cx);
        }
    }

    /// Keeps up with what is chosen: called as the list is drawn, as what is chosen
    /// changes in many places.
    fn follow_signature(&mut self, cx: &mut Context<Self>) {
        if self.picking {
            return;
        }
        let wanted = match self.selected.iter().next() {
            Some(name) if self.selected.len() == 1 => self
                .items
                .iter()
                .find(|i| &i.name == name)
                .filter(|i| can_be_signed(i))
                .map(|i| i.name.clone()),
            _ => None,
        };
        match wanted {
            None => self.signature = None,
            Some(name) if self.signature.as_ref().is_none_or(|s| s.name != name) => {
                self.look_at_signature(name, self.is_local(), cx);
            }
            Some(_) => {}
        }
    }

    fn look_at_signature(&mut self, name: String, read_all: bool, cx: &mut Context<Self>) {
        let Some(place) = self.place.clone() else {
            return;
        };
        let beside = signature_name(&name);
        // What is listed says, with no need to ask a storage across a network.
        if !self.more && !self.items.iter().any(|i| i.name == beside) {
            self.signature = Some(SignatureOf {
                name,
                signed: Some(Signed::No),
                _looking: None,
            });
            return;
        }
        let of = name.clone();
        let looking = signatures::start(
            place.storage,
            self.path_of(&name),
            read_all,
            cx,
            move |this: &mut Explorer, signed, cx| {
                if let Some(s) = this.signature.as_mut().filter(|s| s.name == of) {
                    s.signed = Some(signed);
                    s._looking = None;
                    cx.notify();
                }
            },
        );
        self.signature = Some(SignatureOf {
            name,
            signed: None,
            _looking: Some(looking),
        });
        cx.notify();
    }

    pub fn new_folder(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.start_naming(Naming::NewFolder, window, cx);
    }

    /// The storage shown, if any.
    pub fn place(&self) -> Option<&Place> {
        self.place.as_ref()
    }

    /// Shows the top of `place`; none for nothing.
    pub fn show_place(&mut self, place: Option<Place>, cx: &mut Context<Self>) {
        self.place = place;
        self.open_folder(Vec::new(), cx);
    }

    /// Shows `folder` of the storage, from its first page.
    pub fn open_folder(&mut self, folder: Vec<String>, cx: &mut Context<Self>) {
        self.folder = folder;
        self.items.clear();
        self.shown.clear();
        self.selected.clear();
        self.anchor = None;
        self.naming = None;
        self.menu = None;
        self.problem = None;
        self.stop_count();
        self.search.update(cx, |s, cx| s.set_text("", 0..0, cx));
        self.query.clear();
        self.scroll.scroll_to_item(0, ScrollStrategy::Top);
        self.list_anew(PAGE, cx);
    }

    /// Reads the folder again, as many items as were there, keeping what is
    /// chosen. What is shown stays until the new list comes.
    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        self.list_anew(self.items.len().max(PAGE), cx);
    }

    /// Lists the folder from its start, `want` items for now.
    fn list_anew(&mut self, want: usize, cx: &mut Context<Self>) {
        self.era += 1;
        let era = self.era;
        self.lister = None;
        self.more = false;
        let Some(place) = self.place.clone() else {
            self.state = State::Idle;
            return cx.notify();
        };
        self.state = if self.items.is_empty() {
            State::Opening
        } else {
            State::Loading
        };
        let folder = self.folder.clone();
        let work = cx.background_spawn(async move {
            let mut lister = place.storage.list(&folder)?;
            let items = take(lister.as_mut(), want)?;
            std::io::Result::Ok((items, lister))
        });
        cx.spawn(async move |this, cx| {
            let result = work.await;
            let _ = this.update(cx, |this, cx| {
                if this.era != era {
                    return;
                }
                match result {
                    Ok((items, lister)) => {
                        this.more = !lister.done();
                        this.lister = Some(Arc::new(Mutex::new(lister)));
                        this.items = items;
                        let names: BTreeSet<&str> =
                            this.items.iter().map(|i| i.name.as_str()).collect();
                        this.selected.retain(|n| names.contains(n.as_str()));
                        // It may have been signed, or changed, since.
                        this.signature = None;
                        this.state = State::Idle;
                        this.rebuild();
                    }
                    Err(e) => {
                        this.items.clear();
                        this.shown.clear();
                        this.state =
                            State::Failed(tr!("This folder could not be read: {why}", why = e));
                    }
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    /// Asks for `want` more items, or all there are.
    pub fn load_more(&mut self, want: usize, cx: &mut Context<Self>) {
        if self.state != State::Idle || !self.more {
            return;
        }
        let Some(lister) = self.lister.clone() else {
            return;
        };
        let era = self.era;
        self.state = State::Loading;
        let work = cx.background_spawn(async move {
            let mut lister = lister.lock().unwrap_or_else(|e| e.into_inner());
            let items = take(lister.as_mut(), want)?;
            std::io::Result::Ok((items, lister.done()))
        });
        cx.spawn(async move |this, cx| {
            let result = work.await;
            let _ = this.update(cx, |this, cx| {
                if this.era != era {
                    return;
                }
                this.state = State::Idle;
                match result {
                    Ok((items, done)) => {
                        this.items.extend(items);
                        this.more = !done;
                        this.rebuild();
                    }
                    Err(e) => {
                        this.problem = Some(tr!("Could not load more: {why}", why = e));
                    }
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    /// Works out what is shown and in what order. Sorted by name, the order is
    /// the one the storage gave; the other columns sort what has been loaded.
    fn rebuild(&mut self) {
        let items = &self.items;
        let mut shown: Vec<usize> = (0..items.len())
            .filter(|&i| {
                let item = &items[i];
                (self.show_hidden || !item.hidden)
                    && (!self.picking || item.dir || (self.archives && is_archive(item)))
                    && (self.query.is_empty() || name_holds(&item.name, &self.query))
            })
            .collect();
        match self.sort {
            Column::Name => {}
            Column::Size => shown.sort_by_key(|&i| (!items[i].dir, items[i].size)),
            Column::Modified => shown.sort_by_key(|&i| (!items[i].dir, items[i].modified)),
        }
        if !self.ascending {
            // Folders stay first.
            let folders = shown.iter().take_while(|&&i| items[i].dir).count();
            shown[..folders].reverse();
            shown[folders..].reverse();
        }
        self.shown = shown;
    }

    /// Sorts by `column`, the other way round if it is sorted by it already.
    fn sort_by(&mut self, column: Column, cx: &mut Context<Self>) {
        if self.sort == column {
            self.ascending = !self.ascending;
        } else {
            self.sort = column;
            self.ascending = true;
        }
        self.rebuild();
        cx.notify();
    }

    pub fn set_show_hidden(&mut self, show: bool, cx: &mut Context<Self>) {
        self.show_hidden = show;
        prefs::change(cx, |p| p.show_hidden = show);
        self.rebuild();
        cx.notify();
    }

    fn at(&self, i: usize) -> Option<&Item> {
        self.shown.get(i).and_then(|&at| self.items.get(at))
    }

    /// The items chosen, in the order shown.
    pub fn selection(&self) -> Vec<Item> {
        self.shown
            .iter()
            .filter_map(|&i| self.items.get(i))
            .filter(|i| self.selected.contains(&i.name))
            .cloned()
            .collect()
    }

    /// The path of `name` from the top of the storage.
    fn path_of(&self, name: &str) -> Vec<String> {
        let mut path = self.folder.clone();
        path.push(name.to_owned());
        path
    }

    /// The path of `name` from the top of the storage.
    pub fn path_of_name(&self, name: &str) -> Vec<String> {
        self.path_of(name)
    }

    /// Whether the storage shown is on this computer: other programs and the
    /// archive jobs work only on what is.
    pub fn is_local(&self) -> bool {
        self.place
            .as_ref()
            .is_some_and(|p| p.storage.local_path(&[]).is_some())
    }

    /// Where the folder shown is on this computer, if it is.
    fn local_folder(&self) -> Option<PathBuf> {
        self.place.as_ref()?.storage.local_path(&self.folder)
    }

    /// Where `name` is on this computer, if it is.
    pub fn local_path_of(&self, name: &str) -> Option<PathBuf> {
        self.place.as_ref()?.storage.local_path(&self.path_of(name))
    }

    /// The path of `name` as the user would write it.
    pub fn shown_path_of(&self, name: &str) -> String {
        self.place
            .as_ref()
            .map(|p| p.storage.show_path(&self.path_of(name)))
            .unwrap_or_default()
    }

    /// `item` as an archive to read: its file here, or by ranges from the storage.
    fn source_of(&self, item: &Item) -> Option<Source> {
        let place = self.place.as_ref()?;
        let path = self.path_of(&item.name);
        Some(match place.storage.local_path(&path) {
            Some(file) => Source::File(file),
            None => Source::Stored(Stored::new(place.storage.clone(), path, item.size)),
        })
    }

    /// A click on the row at `i`.
    fn select(&mut self, i: usize, more: bool, range: bool, cx: &mut Context<Self>) {
        let Some(name) = self.at(i).map(|item| item.name.clone()) else {
            return;
        };
        let anchor_at = self.anchor.as_ref().and_then(|anchor| {
            self.shown
                .iter()
                .position(|&s| self.items.get(s).is_some_and(|it| &it.name == anchor))
        });
        match (range, anchor_at) {
            (true, Some(from)) => {
                let names: Vec<String> = (from.min(i)..=from.max(i))
                    .filter_map(|r| self.at(r).map(|it| it.name.clone()))
                    .collect();
                self.selected = names.into_iter().collect();
            }
            _ if more => {
                if !self.selected.remove(&name) {
                    self.selected.insert(name.clone());
                }
                self.anchor = Some(name);
            }
            _ => {
                self.selected.clear();
                self.selected.insert(name.clone());
                self.anchor = Some(name);
            }
        }
        self.problem = None;
        cx.notify();
    }

    /// Chooses only the item called `name`, for tests and after a rename.
    pub fn choose(&mut self, name: &str, cx: &mut Context<Self>) {
        self.selected.clear();
        self.selected.insert(name.to_owned());
        self.anchor = Some(name.to_owned());
        cx.notify();
    }

    /// Opens `item`: a folder shows, an archive opens, a file is shown.
    pub fn open(&mut self, item: &Item, cx: &mut Context<Self>) {
        if item.dir {
            let path = self.path_of(&item.name);
            self.open_folder(path, cx);
        } else if is_archive(item) {
            if let Some(source) = self.source_of(item) {
                cx.emit(ExplorerEvent::OpenArchive(source));
            }
        } else {
            cx.emit(ExplorerEvent::View(item.clone()));
        }
    }

    /// The files the list shows, in its order, for the viewer.
    pub fn viewer_files(&self) -> Vec<crate::viewer::ViewFile> {
        let local = self.is_local();
        self.shown
            .iter()
            .filter_map(|&i| self.items.get(i))
            .filter(|item| !item.dir)
            .map(|item| crate::viewer::ViewFile {
                path: self.path_of(&item.name),
                name: item.name.clone(),
                size: item.size,
                app: local && !runs_when_opened(item),
                edit: local,
            })
            .collect()
    }

    /// The item named `name` in the folder shown, if it is there.
    pub fn item_named(&self, name: &str) -> Option<Item> {
        self.items.iter().find(|i| i.name == name).cloned()
    }

    /// Chooses `name`, brings it into sight, and takes the keys.
    pub fn choose_and_show(&mut self, name: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.choose(name, cx);
        let at = self
            .shown
            .iter()
            .position(|&i| self.items.get(i).is_some_and(|it| it.name == name));
        if let Some(at) = at {
            self.scroll.scroll_to_item(at, ScrollStrategy::Center);
        }
        window.focus(&self.focus, cx);
    }

    /// Opens `item` in packwing as an archive, whatever its name says: a
    /// package, a document or a book may be one.
    pub fn open_as_archive(&mut self, item: &Item, cx: &mut Context<Self>) {
        if let Some(source) = self.source_of(item) {
            cx.emit(ExplorerEvent::OpenArchive(source));
        }
    }

    pub fn extract(&mut self, item: &Item, cx: &mut Context<Self>) {
        if let Some(source) = self.source_of(item) {
            cx.emit(ExplorerEvent::Extract(source));
        }
    }

    /// Makes a new archive of what is chosen.
    pub fn compress(&mut self, cx: &mut Context<Self>) {
        let paths: Vec<PathBuf> = self
            .selection()
            .iter()
            .filter_map(|i| self.local_path_of(&i.name))
            .collect();
        if !paths.is_empty() {
            cx.emit(ExplorerEvent::Compress(paths));
        }
    }

    pub fn ask_delete(&mut self, cx: &mut Context<Self>) {
        let names: Vec<String> = self.selection().into_iter().map(|i| i.name).collect();
        if !names.is_empty() {
            cx.emit(ExplorerEvent::AskDelete(names));
        }
    }

    /// Deletes these, as was asked, then lists the folder again.
    pub fn delete(&mut self, names: Vec<String>, cx: &mut Context<Self>) {
        let folder = self.folder.clone();
        self.run(
            move |storage| storage.delete(&folder, &names),
            |why| tr!("Could not delete: {why}", why = why),
            cx,
        );
    }

    /// Does `work` on the storage in the background, then lists the folder again.
    fn run(
        &mut self,
        work: impl FnOnce(&dyn Storage) -> std::io::Result<()> + Send + 'static,
        said: impl FnOnce(std::io::Error) -> String + 'static,
        cx: &mut Context<Self>,
    ) {
        let Some(place) = self.place.clone() else {
            return;
        };
        let era = self.era;
        let task = cx.background_spawn(async move { work(place.storage.as_ref()) });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                if this.era != era {
                    return;
                }
                match result {
                    Ok(()) => {
                        this.problem = None;
                        this.selected.clear();
                    }
                    Err(e) => this.problem = Some(said(e)),
                }
                this.refresh(cx);
            });
        })
        .detach();
    }

    fn start_naming(&mut self, naming: Naming, window: &mut Window, cx: &mut Context<Self>) {
        let (text, chosen) = match &naming {
            Naming::NewFolder => (String::new(), 0..0),
            Naming::Rename(name) => {
                // The name without its extension is chosen, as file managers do.
                let dir = self.items.iter().any(|i| i.name == *name && i.dir);
                let stem = match name.rfind('.') {
                    Some(i) if i > 0 && !dir => i,
                    _ => name.len(),
                };
                (name.clone(), 0..stem)
            }
        };
        self.name_field
            .update(cx, |f, cx| f.set_text(&text, chosen, cx));
        if let Naming::Rename(name) = &naming
            && let Some(at) = self
                .shown
                .iter()
                .position(|&i| self.items.get(i).is_some_and(|it| &it.name == name))
        {
            self.scroll.scroll_to_item(at, ScrollStrategy::Nearest);
        }
        if naming == Naming::NewFolder {
            self.scroll.scroll_to_item(0, ScrollStrategy::Top);
        }
        self.naming = Some(naming);
        self.problem = None;
        self.menu = None;
        window.focus(&self.name_field.focus_handle(cx), cx);
        cx.notify();
    }

    fn start_rename(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let [one] = self.selection().as_slice() {
            self.start_naming(Naming::Rename(one.name.clone()), window, cx);
        }
    }

    fn cancel_naming(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.naming = None;
        window.focus(&self.focus, cx);
        cx.notify();
    }

    /// Makes the folder or renames the item, then shows it chosen.
    fn commit_naming(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let (Some(naming), Some(place)) = (self.naming.clone(), self.place.clone()) else {
            return;
        };
        let name = self.name_field.read(cx).text().trim().to_owned();
        if naming == Naming::Rename(name.clone()) {
            return self.cancel_naming(window, cx);
        }
        if let Some(why) = bad_name(&name) {
            self.problem = Some(why);
            return cx.notify();
        }
        let folder = self.folder.clone();
        let (to, era) = (name.clone(), self.era);
        let task = cx.background_spawn(async move {
            match naming {
                Naming::NewFolder => place.storage.make_folder(&folder, &to),
                Naming::Rename(old) => place.storage.rename(&folder, &old, &to),
            }
        });
        // In the window: the keys go back to the list once it worked.
        cx.spawn_in(window, async move |this, cx| {
            let result = task.await;
            let _ = this.update_in(cx, |this, window, cx| {
                if this.era != era {
                    return;
                }
                match result {
                    Ok(()) => {
                        this.naming = None;
                        this.problem = None;
                        this.choose(&name, cx);
                        window.focus(&this.focus, cx);
                        this.refresh(cx);
                    }
                    Err(e) => {
                        // Still typing, to try another.
                        this.problem = Some(e.to_string());
                        cx.notify();
                    }
                }
            });
        })
        .detach();
    }

    /// Opens `item` in the editor chosen in Settings, or the system's.
    pub fn edit(&mut self, item: &Item, cx: &mut Context<Self>) {
        let Some(path) = self.local_path_of(&item.name) else {
            return;
        };
        let cmd = prefs::prefs(cx).editor;
        let started = match &cmd {
            Some(cmd) => editors::open_with(cmd, &path),
            None => editors::open_text(&path),
        };
        self.problem = started.err().map(|e| {
            let who = cmd
                .as_deref()
                .map_or(tr!("the text editor"), editors::name_of);
            tr!("Cannot start {who}: {why}", who = who, why = e)
        });
        cx.notify();
    }

    /// What Edit is called: after the editor chosen, if one is.
    pub fn edit_label(cx: &App) -> String {
        prefs::prefs(cx).editor.as_deref().map_or_else(
            || tr!("Edit in text editor"),
            |cmd| tr!("Edit in {editor}", editor = editors::name_of(cmd)),
        )
    }

    /// Opens `item` with the app the system has for its kind. A file that could
    /// run is not offered this, and refused here too.
    pub fn open_with_system(&mut self, item: &Item, cx: &mut Context<Self>) {
        if runs_when_opened(item) {
            return;
        }
        if let Some(path) = self.local_path_of(&item.name) {
            cx.open_with_system(&path);
        }
    }

    fn stop_count(&mut self) {
        if let Some(count) = self.count.take() {
            count.cancel.cancel();
        }
    }

    /// Counts what the folder `name` holds, in the background.
    pub fn count_folder(&mut self, name: &str, cx: &mut Context<Self>) {
        self.stop_count();
        let Some(place) = self.place.clone() else {
            return;
        };
        let (path, cancel) = (self.path_of(name), Cancel::new());
        let now = Arc::new(Mutex::new(Counted::default()));
        let finished = Arc::new(AtomicBool::new(false));
        let work = {
            let (cancel, now, finished) = (cancel.clone(), now.clone(), finished.clone());
            cx.background_spawn(async move {
                let result = count(place.storage.as_ref(), &path, &cancel, |c| {
                    *now.lock().unwrap_or_else(|e| e.into_inner()) = c;
                });
                if let Ok((c, _)) = &result {
                    *now.lock().unwrap_or_else(|e| e.into_inner()) = *c;
                }
                finished.store(true, Ordering::Relaxed);
                result.map(|(_, unread)| unread)
            })
        };
        let task = cx.spawn(async move |this, cx| {
            // What it has come to shows as it goes.
            while !finished.load(Ordering::Relaxed) {
                cx.background_executor()
                    .timer(Duration::from_millis(200))
                    .await;
                if this.update(cx, |_, cx| cx.notify()).is_err() {
                    return;
                }
            }
            let result = work.await;
            let _ = this.update(cx, |this, cx| {
                if let (Some(count), Ok(unread)) = (&mut this.count, result) {
                    count.done = Some(unread);
                }
                cx.notify();
            });
        });
        self.count = Some(Count {
            name: name.to_owned(),
            cancel,
            now,
            done: None,
            _task: task,
        });
        cx.notify();
    }

    /// The count of the folder `name`, if there is one.
    pub fn count_of(&self, name: &str) -> Option<CountShown> {
        let count = self.count.as_ref().filter(|c| c.name == name)?;
        Some(CountShown {
            counted: *count.now.lock().unwrap_or_else(|e| e.into_inner()),
            done: count.done.is_some(),
            unread: count.done.unwrap_or(0),
        })
    }

    fn on_key(&mut self, e: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let (key, m) = (e.keystroke.key.as_str(), &e.keystroke.modifiers);
        if self.menu.is_some() && key == "escape" {
            self.menu = None;
            cx.notify();
            return cx.stop_propagation();
        }
        // Typing a name: the field has the other keys.
        if self.naming.is_some() {
            match key {
                "enter" => self.commit_naming(window, cx),
                "escape" => self.cancel_naming(window, cx),
                _ => return,
            }
            return cx.stop_propagation();
        }
        if self.picking {
            return;
        }
        if self.search.focus_handle(cx).is_focused(window) {
            if key == "escape" {
                self.search.update(cx, |s, cx| s.set_text("", 0..0, cx));
                window.focus(&self.focus, cx);
                cx.stop_propagation();
            }
            return;
        }
        let one = match self.selection().as_slice() {
            [one] => Some(one.clone()),
            _ => None,
        };
        match key {
            "enter" => {
                if let Some(item) = one {
                    self.open(&item, cx);
                }
            }
            "backspace" => self.go_up(cx),
            "f2" => self.start_rename(window, cx),
            "f5" => self.refresh(cx),
            "delete" => self.ask_delete(cx),
            "space" => {
                if let Some(item) = one.filter(|i| !i.dir && !is_archive(i)) {
                    cx.emit(ExplorerEvent::View(item));
                }
            }
            "a" if m.secondary() => {
                self.selected = self
                    .shown
                    .iter()
                    .filter_map(|&i| self.items.get(i).map(|it| it.name.clone()))
                    .collect();
                cx.notify();
            }
            "up" | "down" | "pageup" | "pagedown" | "home" | "end" => {
                self.move_choice(key, m.shift, cx)
            }
            _ => return,
        }
        cx.stop_propagation();
    }

    /// Chooses the row `key` moves to from the anchor (see [`list_keys`]);
    /// with `range`, all up to it.
    fn move_choice(&mut self, key: &str, range: bool, cx: &mut Context<Self>) {
        let from = self.anchor.as_ref().and_then(|anchor| {
            self.shown
                .iter()
                .position(|&s| self.items.get(s).is_some_and(|it| &it.name == anchor))
        });
        let page = list_keys::rows_per_page(&self.scroll, self.shown.len());
        let Some(to) = list_keys::moved(key, from, self.shown.len(), page) else {
            return;
        };
        let keep = self.anchor.clone();
        self.select(to, false, range, cx);
        if range {
            self.anchor = keep;
        }
        self.scroll.scroll_to_item(to, ScrollStrategy::Nearest);
    }

    pub fn go_up(&mut self, cx: &mut Context<Self>) {
        if !self.folder.is_empty() {
            let mut up = self.folder.clone();
            up.pop();
            self.open_folder(up, cx);
        }
    }

    fn toolbar(&self, cx: &mut Context<Self>) -> Div {
        let c = colors();
        let name = self
            .place
            .as_ref()
            .map_or(String::new(), |p| p.name.clone());
        let steps: Vec<String> = std::iter::once(name)
            .chain(self.folder.iter().cloned())
            .collect();
        let last = steps.len().saturating_sub(1);
        let crumbs = steps.into_iter().enumerate().map(|(i, step)| {
            div()
                .flex()
                .items_center()
                .gap_x(ui(6.))
                .when(i > 0, |d| d.child("›"))
                .child(
                    div()
                        .id(("crumb", i))
                        .when(i == last, |d| {
                            d.font_weight(FontWeight::SEMIBOLD).text_color(rgb(c.text))
                        })
                        .when(i != last, |d| {
                            d.cursor_pointer()
                                .hover(|s| s.text_color(rgb(c.text)))
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    let to = this.folder[..i].to_vec();
                                    this.open_folder(to, cx);
                                }))
                        })
                        .child(step),
                )
        });
        div()
            .flex()
            .flex_wrap()
            .flex_shrink_0()
            .items_center()
            .gap(ui(10.))
            .px(ui(8.))
            .pb(ui(12.))
            .child(
                icon_button("up", "arrow-up", 40.)
                    .on_click(cx.listener(|this, _, _, cx| this.go_up(cx))),
            )
            .child(
                icon_button("refresh", "refresh", 40.)
                    .on_click(cx.listener(|this, _, _, cx| this.refresh(cx))),
            )
            .child(
                div()
                    .flex_1()
                    .min_w(ui(160.))
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_x(ui(6.))
                    .text_size(ui(14.))
                    .text_color(rgb(c.muted))
                    .children(crumbs),
            )
            .when(!self.picking, |d| {
                d.child(
                    div()
                        .w(ui(220.))
                        .h(ui(40.))
                        .flex()
                        .flex_shrink_0()
                        .items_center()
                        .gap(ui(8.))
                        .px(ui(16.))
                        .rounded_full()
                        .bg(rgb(c.field))
                        .text_size(ui(14.))
                        .child(icon("search", 16., c.muted))
                        .child(self.search.clone()),
                )
            })
    }

    /// The column heads: a click sorts by one.
    fn head(&self, cx: &mut Context<Self>) -> Div {
        let c = colors();
        let columns = self.columns.get();
        let arrow = if self.ascending { " ↑" } else { " ↓" };
        let head = |id: &'static str, label: String, column: Column, cx: &mut Context<Self>| {
            let on = self.sort == column;
            div()
                .id(id)
                .cursor_pointer()
                .hover(|s| s.text_color(rgb(c.text)))
                .when(on, |d| d.text_color(rgb(c.text)))
                .on_click(cx.listener(move |this, _, _, cx| this.sort_by(column, cx)))
                .child(if on { format!("{label}{arrow}") } else { label })
        };
        div()
            .flex()
            .flex_shrink_0()
            .items_center()
            .gap(ui(12.))
            .px(ui(14.))
            .pb(ui(8.))
            .text_size(ui(12.))
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(rgb(c.muted))
            .child(div().w(ui(36.)).flex_shrink_0())
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(head("sort-name", tr!("Name"), Column::Name, cx)),
            )
            .when(columns.size(), |d| {
                d.child(
                    div()
                        .w(ui(90.))
                        .flex_shrink_0()
                        .flex()
                        .justify_end()
                        .child(head("sort-size", tr!("Size"), Column::Size, cx)),
                )
            })
            .when(columns.modified(), |d| {
                d.child(div().w(ui(100.)).flex_shrink_0().child(head(
                    "sort-modified",
                    tr!("Modified"),
                    Column::Modified,
                    cx,
                )))
            })
    }

    /// A row's frame, with the columns as an item's are laid out.
    fn cells(
        columns: Columns,
        id: impl Into<ElementId>,
        kind: Kind,
        tag: String,
        name: impl IntoElement,
        size: String,
        modified: String,
    ) -> Stateful<Div> {
        let c = colors();
        div()
            .id(id)
            .w_full()
            .h(ui(ROW))
            .flex()
            .items_center()
            .gap(ui(12.))
            .px(ui(14.))
            .rounded(ui(16.))
            .child(kind_tile(kind, tag, 36., 32., 11., 10.))
            .child(div().flex_1().min_w_0().child(name))
            .when(columns.size(), |d| {
                d.child(
                    div()
                        .w(ui(90.))
                        .flex_shrink_0()
                        .flex()
                        .justify_end()
                        .text_size(ui(13.))
                        .text_color(rgb(c.muted))
                        .child(size),
                )
            })
            .when(columns.modified(), |d| {
                d.child(
                    div()
                        .w(ui(100.))
                        .flex_shrink_0()
                        .text_size(ui(13.))
                        .text_color(rgb(c.muted))
                        .child(modified),
                )
            })
    }

    /// The field a name is typed in, where the name goes in its row.
    fn name_box(&self) -> Div {
        let c = colors();
        div()
            .h(ui(36.))
            .flex()
            .items_center()
            .px(ui(10.))
            .rounded(ui(10.))
            .border_2()
            .border_color(rgb(c.accent))
            .bg(rgb(c.surface))
            .text_size(ui(15.))
            .child(self.name_field.clone())
    }

    fn row(&self, i: usize, cx: &mut Context<Self>) -> gpui::AnyElement {
        let c = colors();
        let typing_new = self.naming == Some(Naming::NewFolder);
        if typing_new && i == 0 {
            return Self::cells(
                self.columns.get(),
                "new-folder",
                Kind::Folder,
                "DIR".into(),
                self.name_box(),
                String::new(),
                String::new(),
            )
            .bg(rgb(c.chosen))
            .into_any_element();
        }
        let at = i - usize::from(typing_new);
        let Some(item) = self.at(at).cloned() else {
            return self.last_row(cx);
        };
        let on = self.selected.contains(&item.name);
        let kind = Kind::of(&item.name, item.dir);
        let renaming = self.naming == Some(Naming::Rename(item.name.clone()));
        let name: gpui::AnyElement = if renaming {
            self.name_box().into_any_element()
        } else {
            div()
                .truncate()
                .text_size(ui(15.))
                .font_weight(if on {
                    FontWeight::SEMIBOLD
                } else {
                    FontWeight::MEDIUM
                })
                .child(item.name.clone())
                .into_any_element()
        };
        let size = if item.dir {
            String::from("—")
        } else {
            crate::model::size(item.size)
        };
        let modified = item.modified.map_or(String::from("—"), date);
        let row_item = item.clone();
        if self.picking {
            return Self::cells(
                self.columns.get(),
                ("row", at),
                kind,
                tag_of(&item.name, kind),
                name,
                if item.dir { String::new() } else { size },
                modified,
            )
            .debug_selector({
                let name = item.name.clone();
                move || format!("pick: {name}")
            })
            .cursor_pointer()
            .hover(|s| s.bg(rgb(c.well_hover)))
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_click(cx.listener(move |this, _, _, cx| {
                if this.naming.is_none() {
                    this.open(&item, cx);
                }
            }))
            .into_any_element();
        }
        Self::cells(
            self.columns.get(),
            ("row", at),
            kind,
            tag_of(&item.name, kind),
            name,
            size,
            modified,
        )
        .debug_selector({
            let name = item.name.clone();
            move || format!("row: {name}")
        })
        .cursor_pointer()
        .when(on, |d| d.bg(rgb(c.chosen)))
        .when(!on, |d| d.hover(|s| s.bg(rgb(c.hover))))
        // A press on a row is the row's, not the list's under it.
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .on_mouse_down(
            MouseButton::Right,
            cx.listener(move |this, e: &MouseDownEvent, window, cx| {
                if !this.selection().iter().any(|s| s.name == row_item.name) {
                    this.select(at, false, false, cx);
                }
                this.menu = Some((e.position, MenuFor::Selected));
                window.focus(&this.focus, cx);
                cx.stop_propagation();
                cx.notify();
            }),
        )
        .on_click(cx.listener(move |this, e: &gpui::ClickEvent, window, cx| {
            if this.naming.is_some() {
                return;
            }
            window.focus(&this.focus, cx);
            let keys = e.modifiers();
            if e.click_count() >= 2 && !keys.secondary() && !keys.shift {
                this.open(&item, cx);
            } else {
                this.select(at, keys.secondary(), keys.shift, cx);
            }
        }))
        .into_any_element()
    }

    /// The row after the last item: for more, or for a search of all there is.
    fn last_row(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        let c = colors();
        let row = div()
            .id("last-row")
            .w_full()
            .h(ui(ROW))
            .flex()
            .items_center()
            .justify_center()
            .gap(ui(12.))
            .text_size(ui(13.))
            .text_color(rgb(c.muted));
        if self.state == State::Loading {
            return row.child(tr!("Loading…")).into_any_element();
        }
        if !self.query.is_empty() {
            return row
                .child(tr!("Only what is loaded was searched."))
                .child(
                    button(
                        "search-all",
                        tr!("Search the whole folder"),
                        Fill::Text,
                        Size::Medium,
                    )
                    .on_click(cx.listener(|this, _, _, cx| this.load_more(usize::MAX, cx))),
                )
                .into_any_element();
        }
        row.child(
            button(
                "load-more",
                tr!("Load {n} more", n = crate::model::count(PAGE as u64)),
                Fill::Tonal,
                Size::Medium,
            )
            .on_click(cx.listener(|this, _, _, cx| this.load_more(PAGE, cx))),
        )
        .into_any_element()
    }

    /// The right-click menu, at `at`.
    fn menu(&self, at: Point<Pixels>, target: MenuFor, cx: &mut Context<Self>) -> gpui::AnyElement {
        let c = colors();
        let item = |id: &'static str, label: String, key: &'static str, bad: bool| {
            menu_item(id, label, key).when(bad, |d| d.text_color(rgb(c.bad_text)))
        };
        let line = || div().my(ui(4.)).mx(ui(10.)).h(ui(1.)).bg(rgb(c.field));
        let mut menu = menu_panel("explorer-menu")
            .min_w(ui(230.))
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_mouse_down_out(cx.listener(|this, e: &MouseDownEvent, _, cx| {
                // A right click opens the next one.
                if e.button != MouseButton::Right {
                    this.menu = None;
                    cx.notify();
                }
            }));
        let selection = self.selection();
        match (target, selection.as_slice()) {
            (MenuFor::Folder, _) => {
                let hidden = self.show_hidden;
                menu = menu
                    .child(
                        item("menu-new-folder", tr!("New folder"), "", false).on_click(
                            cx.listener(|this, _, window, cx| {
                                this.start_naming(Naming::NewFolder, window, cx)
                            }),
                        ),
                    )
                    .child(
                        item("menu-refresh", tr!("Refresh"), "F5", false).on_click(cx.listener(
                            |this, _, _, cx| {
                                this.menu = None;
                                this.refresh(cx);
                            },
                        )),
                    )
                    .when_some(self.local_folder(), |d, folder| {
                        d.child(
                            item("menu-file-manager", file_manager::open_label(), "", false)
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.menu = None;
                                    file_manager::open(&folder, cx);
                                    cx.notify();
                                })),
                        )
                    })
                    .child(line())
                    .child(
                        item(
                            "menu-hidden",
                            if hidden {
                                tr!("Hide hidden files")
                            } else {
                                tr!("Show hidden files")
                            },
                            "",
                            false,
                        )
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.menu = None;
                            this.set_show_hidden(!hidden, cx);
                        })),
                    );
            }
            (MenuFor::Selected, [one]) => {
                let (open, edit, app, extract) =
                    (one.clone(), one.clone(), one.clone(), one.clone());
                let archive = is_archive(one);
                menu = menu.child(
                    item(
                        "menu-open",
                        if one.dir || archive {
                            tr!("Open")
                        } else {
                            tr!("View")
                        },
                        "Enter",
                        false,
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.menu = None;
                        this.open(&open, cx);
                    })),
                );
                let local = self.is_local();
                if archive {
                    menu = menu.child(item("menu-extract", tr!("Extract…"), "", false).on_click(
                        cx.listener(move |this, _, _, cx| {
                            this.menu = None;
                            this.extract(&extract, cx);
                        }),
                    ));
                }
                if !one.dir && !archive && local {
                    menu = menu.child(item("menu-edit", Self::edit_label(cx), "", false).on_click(
                        cx.listener(move |this, _, _, cx| {
                            this.menu = None;
                            this.edit(&edit, cx);
                        }),
                    ));
                    if !runs_when_opened(one) {
                        menu = menu.child(
                            item("menu-app", tr!("Open with default app"), "", false).on_click(
                                cx.listener(move |this, _, _, cx| {
                                    this.menu = None;
                                    this.open_with_system(&app, cx);
                                }),
                            ),
                        );
                    }
                }
                // What is an archive by another name: a package, a document, a
                // book. Archives open so already.
                if !one.dir && !archive {
                    let inside = one.clone();
                    menu = menu.child(
                        item("menu-as-archive", tr!("Open as archive"), "", false).on_click(
                            cx.listener(move |this, _, _, cx| {
                                this.menu = None;
                                this.open_as_archive(&inside, cx);
                            }),
                        ),
                    );
                }
                let path = self.shown_path_of(&one.name);
                if local {
                    menu = menu.child(item("menu-compress", tr!("Compress…"), "", false).on_click(
                        cx.listener(|this, _, _, cx| {
                            this.menu = None;
                            this.compress(cx);
                        }),
                    ));
                }
                menu =
                    menu.child(line())
                        .child(
                            item("menu-copy-to", tr!("Copy to…"), "", false).on_click(
                                cx.listener(|this, _, _, cx| this.ask_transfer(false, cx)),
                            ),
                        )
                        .child(
                            item("menu-move-to", tr!("Move to…"), "", false).on_click(
                                cx.listener(|this, _, _, cx| this.ask_transfer(true, cx)),
                            ),
                        )
                        .when(can_be_signed(one), |d| {
                            d.child(
                                item("menu-sign", tr!("Sign…"), "", false)
                                    .on_click(cx.listener(|this, _, _, cx| this.ask_sign(cx))),
                            )
                        })
                        .when(!one.dir && !one.link, |d| {
                            d.child(
                                item("menu-checksums", tr!("Checksums…"), "", false)
                                    .on_click(cx.listener(|this, _, _, cx| this.ask_checksums(cx))),
                            )
                        })
                        .when(local && !one.dir && !one.link, |d| {
                            let (split, join, convert) =
                                (one.name.clone(), one.name.clone(), one.name.clone());
                            d.when(archive && !is_cut_part(one), |d| {
                                d.child(item("menu-convert", tr!("Convert…"), "", false).on_click(
                                    cx.listener(move |this, _, _, cx| {
                                        this.menu = None;
                                        cx.emit(ExplorerEvent::Convert(convert.clone()));
                                    }),
                                ))
                            })
                            .child(
                                item("menu-split", tr!("Split into parts…"), "", false).on_click(
                                    cx.listener(move |this, _, _, cx| {
                                        this.menu = None;
                                        cx.emit(ExplorerEvent::Split(split.clone()));
                                    }),
                                ),
                            )
                            .when(is_cut_part(one), |d| {
                                d.child(
                                    item("menu-join", tr!("Join the parts"), "", false).on_click(
                                        cx.listener(move |this, _, _, cx| {
                                            this.menu = None;
                                            cx.emit(ExplorerEvent::Join(join.clone()));
                                        }),
                                    ),
                                )
                            })
                        })
                        .child(line())
                        .child(item("menu-rename", tr!("Rename"), "F2", false).on_click(
                            cx.listener(|this, _, window, cx| {
                                this.menu = None;
                                this.start_rename(window, cx);
                            }),
                        ))
                        .child(item("menu-delete", tr!("Delete"), "Del", true).on_click(
                            cx.listener(|this, _, _, cx| {
                                this.menu = None;
                                this.ask_delete(cx);
                            }),
                        ))
                        .child(line())
                        .when_some(self.local_path_of(&one.name), |d, here| {
                            d.child(
                                item("menu-file-manager", file_manager::show_label(), "", false)
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.menu = None;
                                        file_manager::show(&here, cx);
                                        cx.notify();
                                    })),
                            )
                        })
                        .child(
                            item("menu-copy-path", tr!("Copy path"), "", false).on_click(
                                cx.listener(move |this, _, _, cx| {
                                    this.menu = None;
                                    cx.write_to_clipboard(ClipboardItem::new_string(path.clone()));
                                    cx.notify();
                                }),
                            ),
                        );
            }
            (MenuFor::Selected, many) => {
                menu = menu
                    .child(
                        item(
                            "menu-compress",
                            tr_n!(
                                many.len() as u64,
                                "Compress {n} item…",
                                "Compress {n} items…"
                            ),
                            "",
                            false,
                        )
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.menu = None;
                            this.compress(cx);
                        })),
                    )
                    .child(line())
                    .child(
                        item("menu-copy-to", tr!("Copy to…"), "", false)
                            .on_click(cx.listener(|this, _, _, cx| this.ask_transfer(false, cx))),
                    )
                    .child(
                        item("menu-move-to", tr!("Move to…"), "", false)
                            .on_click(cx.listener(|this, _, _, cx| this.ask_transfer(true, cx))),
                    )
                    .child(line())
                    .child(
                        item("menu-delete", tr!("Delete"), "Del", true).on_click(cx.listener(
                            |this, _, _, cx| {
                                this.menu = None;
                                this.ask_delete(cx);
                            },
                        )),
                    );
            }
        }
        deferred(
            anchored()
                .position(at)
                .snap_to_window_with_margin(px(8.))
                .child(menu),
        )
        .with_priority(1)
        .into_any_element()
    }

    /// The line under the list.
    fn status(&self) -> String {
        if self.state == State::Opening {
            return tr!("Reading…");
        }
        let loaded = self.items.len() as u64;
        let what = if self.more {
            tr!(
                "{n} loaded, more in this folder",
                n = crate::model::count(loaded)
            )
        } else {
            tr_n!(self.shown.len() as u64, "{n} item", "{n} items")
        };
        let extra = if self.more && self.sort != Column::Name {
            format!(" · {}", tr!("Sorted among those loaded"))
        } else {
            String::new()
        };
        match self.selected.len() as u64 {
            0 => format!("{what}{extra}"),
            n => format!("{} · {what}{extra}", tr!("{n} selected", n = n)),
        }
    }
}

#[cfg(all(test, feature = "test-hooks"))]
impl Explorer {
    /// The names shown, in order.
    pub fn shown_names(&self) -> Vec<String> {
        self.shown
            .iter()
            .filter_map(|&i| self.items.get(i))
            .map(|i| i.name.clone())
            .collect()
    }

    pub fn has_more(&self) -> bool {
        self.more
    }

    pub fn problem(&self) -> Option<&str> {
        self.problem.as_deref()
    }

    /// Lists the folder at once, for a screenshot to draw what follows.
    pub fn list_now(&mut self, cx: &mut Context<Self>) {
        self.era += 1;
        if let Some(place) = &self.place
            && let Ok(mut lister) = place.storage.list(&self.folder)
            && let Ok(items) = take(lister.as_mut(), PAGE)
        {
            self.more = !lister.done();
            self.items = items;
            self.state = State::Idle;
            self.rebuild();
        }
        cx.notify();
    }

    /// The right-click menu, for what is chosen or for the folder.
    pub fn show_menu(&mut self, x: f32, y: f32, on_folder: bool, cx: &mut Context<Self>) {
        let target = if on_folder {
            MenuFor::Folder
        } else {
            MenuFor::Selected
        };
        self.menu = Some((gpui::point(px(x), px(y)), target));
        cx.notify();
    }

    /// The item shown under `name`.
    pub fn selection_of(&self, name: &str) -> Item {
        self.items
            .iter()
            .find(|i| i.name == name)
            .cloned()
            .expect("it is shown")
    }

    pub fn type_new_folder(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.start_naming(Naming::NewFolder, window, cx);
    }

    pub fn type_rename(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.start_rename(window, cx);
    }

    pub fn type_name(&mut self, name: &str, cx: &mut Context<Self>) {
        self.name_field
            .update(cx, |f, cx| f.set_text(name, name.len()..name.len(), cx));
    }

    pub fn commit_name(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.commit_naming(window, cx);
    }

    pub fn search_for(&mut self, text: &str, cx: &mut Context<Self>) {
        self.search
            .update(cx, |s, cx| s.set_text(text, text.len()..text.len(), cx));
    }
}

/// Up to `want` items from `lister`, a page at a time.
fn take(lister: &mut dyn Lister, want: usize) -> std::io::Result<Vec<Item>> {
    let mut items = Vec::new();
    while items.len() < want {
        let page = lister.next(PAGE.min(want - items.len()))?;
        let empty = page.is_empty();
        items.extend(page);
        if empty || lister.done() {
            break;
        }
    }
    Ok(items)
}

impl Render for Explorer {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.follow_signature(cx);
        let c = colors();
        let typing_new = self.naming == Some(Naming::NewFolder);
        let rows = usize::from(typing_new) + self.shown.len() + usize::from(self.more);
        let body = if let State::Failed(why) = &self.state {
            div()
                .flex_1()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(ui(14.))
                .p(ui(24.))
                .text_size(ui(14.))
                .text_color(rgb(c.muted))
                .child(div().text_center().child(why.clone()))
                .child(
                    button("try-again", tr!("Try again"), Fill::Tonal, Size::Medium)
                        .on_click(cx.listener(|this, _, _, cx| this.refresh(cx))),
                )
                .into_any_element()
        } else if rows == 0 {
            let text = match self.state {
                State::Opening => tr!("Reading…"),
                _ if !self.query.is_empty() => tr!("Nothing here matches that."),
                _ if self.place.is_none() => tr!("Add a storage to browse it."),
                _ if self.archives => tr!("No folders or archives here."),
                _ if self.picking => tr!("No folders here."),
                _ => tr!("This folder is empty."),
            };
            div()
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .text_size(ui(14.))
                .text_color(rgb(c.muted))
                .child(text)
                .into_any_element()
        } else {
            uniform_list(
                "explorer-rows",
                rows,
                cx.processor(|this, range: std::ops::Range<usize>, _, cx| {
                    range.map(|i| this.row(i, cx)).collect::<Vec<_>>()
                }),
            )
            .track_scroll(&self.scroll)
            .flex_1()
            .min_h_0()
            .into_any_element()
        };
        let menu = self.menu.map(|(at, target)| self.menu(at, target, cx));
        let toolbar = self.toolbar(cx);
        let head = self.head(cx);
        // Its width, once laid out: the columns it leaves room for change the
        // next frame, as the window or the panes beside it change.
        let columns = self.columns.clone();
        let measure = canvas(
            move |bounds, window, _| {
                let width = f32::from(bounds.size.width) * 16. / f32::from(window.rem_size());
                let fits = Columns::for_width(width);
                if columns.get() != fits {
                    columns.set(fits);
                    window.refresh();
                }
            },
            |_, _, _, _| {},
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full();
        div()
            .id("explorer")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(Self::on_key))
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .child(measure)
            .child(toolbar)
            .when(!self.picking, |d| d.child(head))
            .child(
                div()
                    .id("explorer-list")
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    // A press beside the rows chooses nothing.
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, window, cx| {
                            window.focus(&this.focus, cx);
                            this.selected.clear();
                            this.anchor = None;
                            cx.notify();
                        }),
                    )
                    .on_mouse_down(
                        MouseButton::Right,
                        cx.listener(|this, e: &MouseDownEvent, window, cx| {
                            if this.picking {
                                return;
                            }
                            window.focus(&this.focus, cx);
                            this.selected.clear();
                            this.menu = Some((e.position, MenuFor::Folder));
                            cx.notify();
                        }),
                    )
                    .child(body),
            )
            .children(self.problem.clone().map(|why| {
                div()
                    .mx(ui(4.))
                    .mt(ui(8.))
                    .px(ui(14.))
                    .py(ui(10.))
                    .rounded(ui(14.))
                    .bg(rgb(c.bad_bg))
                    .text_color(rgb(c.bad_text))
                    .text_size(ui(13.))
                    .child(why)
            }))
            .when(!self.picking, |d| {
                d.child(
                    div()
                        .flex_shrink_0()
                        .px(ui(14.))
                        .pt(ui(10.))
                        .pb(ui(4.))
                        .text_size(ui(12.))
                        .text_color(rgb(c.muted))
                        .child(self.status()),
                )
            })
            .children(menu)
    }
}

#[cfg(all(test, feature = "shots", target_os = "linux"))]
impl Explorer {
    /// Whether the Size and Modified columns show, as its width left them.
    pub(crate) fn columns_for_test(&self) -> (bool, bool) {
        let columns = self.columns.get();
        (columns.size(), columns.modified())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_narrow_list_drops_modified_then_size_and_keeps_the_name() {
        assert_eq!(Columns::for_width(800.), Columns::All);
        assert_eq!(Columns::for_width(430.), Columns::All);
        assert_eq!(Columns::for_width(429.), Columns::Size);
        assert_eq!(Columns::for_width(318.), Columns::Size);
        assert_eq!(Columns::for_width(317.), Columns::Name);
        assert_eq!(Columns::for_width(0.), Columns::Name);
    }

    fn file(name: &str, mode: Option<u32>) -> Item {
        Item {
            name: name.to_owned(),
            dir: false,
            link: false,
            hidden: false,
            size: 1,
            modified: None,
            mode,
        }
    }

    #[test]
    fn what_could_run_is_not_opened() {
        for name in ["a.exe", "b.sh", "c.desktop", "d.PY", "e.AppImage", "f.lnk"] {
            assert!(runs_when_opened(&file(name, Some(0o644))), "{name}");
        }
        assert!(runs_when_opened(&file("tool", Some(0o755))));
        assert!(!runs_when_opened(&file("notes.txt", Some(0o644))));
        assert!(!runs_when_opened(&file("photo.jpg", None)));
    }
}
