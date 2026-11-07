//! New archive: what goes in it and where it goes, its name and format, and
//! under Advanced, how hard it is squeezed, whether it is split into parts,
//! and a password.
//!
//! Convert is the same dialog, for an archive made of what another holds: the
//! one read is all that goes in, and the new one is made on this computer.

use packwing_i18n::{tr, tr_n};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use gpui::{
    App, Context, Div, Entity, EventEmitter, ExternalPaths, FontWeight, PathPromptOptions, Render,
    ScrollHandle, Stateful, Subscription, Task, UniformListScrollHandle, Window, div, prelude::*,
    relative, rgb,
};
use packwing_core::{Frames, Kind, NewArchive, Route, Source};

pub(crate) use packwing_core::Level;

use super::chosen::{self, Choosing};
use super::{
    Converting, Creating, DialogEvent, Job, Saving, SavingOn, Verb, footer, header, places,
};
use crate::colors::colors;
use crate::explorer::{Explorer, Place};
use crate::input::{Changed, TextInput};
use crate::model::{Selection, count, size};
use crate::pages::storages::StoragesPage;
use crate::text_size::ui;
use crate::widgets::{
    Chosen, Fill, Select, SelectLook, Size, Tone, button, check_row, icon, icon_button, legend,
    note, segment, segments, switch, tonal_tint,
};

/// A kind of archive New archive makes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Format {
    Zip,
    SevenZip,
    TarZst,
    TarGz,
    TarXz,
    TarBz2,
    TarLz4,
    Tar,
    Gz,
    Zst,
    Xz,
    Bz2,
    Lz4,
}

impl Format {
    /// Those for any files, most used first.
    pub const ALL: [Format; 8] = [
        Format::Zip,
        Format::SevenZip,
        Format::TarZst,
        Format::TarGz,
        Format::TarXz,
        Format::TarBz2,
        Format::TarLz4,
        Format::Tar,
    ];

    /// Those for any files, then those for one file by itself: an option's index
    /// is its place here, whichever are offered.
    pub const WITH_ONE_FILE: [Format; 13] = [
        Format::Zip,
        Format::SevenZip,
        Format::TarZst,
        Format::TarGz,
        Format::TarXz,
        Format::TarBz2,
        Format::TarLz4,
        Format::Tar,
        Format::Gz,
        Format::Zst,
        Format::Xz,
        Format::Bz2,
        Format::Lz4,
    ];

    /// What follows the archive's name, and what a [`Select`] of formats lists.
    pub fn ext(self) -> &'static str {
        match self {
            Format::Zip => ".zip",
            Format::SevenZip => ".7z",
            Format::Tar => ".tar",
            Format::TarGz => ".tar.gz",
            Format::TarXz => ".tar.xz",
            Format::TarZst => ".tar.zst",
            Format::TarBz2 => ".tar.bz2",
            Format::TarLz4 => ".tar.lz4",
            Format::Gz => ".gz",
            Format::Zst => ".zst",
            Format::Xz => ".xz",
            Format::Bz2 => ".bz2",
            Format::Lz4 => ".lz4",
        }
    }

    /// The format of a [`Select`]'s option.
    pub fn of_option(i: usize) -> Format {
        Format::WITH_ONE_FILE.get(i).copied().unwrap_or(Format::Zip)
    }

    /// Whether it holds one file by itself.
    pub fn one_file(self) -> bool {
        self.kind().one_file()
    }

    /// Where its archives may not open without an archiver app, said under
    /// the name: none for ZIP, which opens on every phone and computer.
    pub fn caveat(self) -> Option<String> {
        match self {
            Format::Zip => None,
            Format::SevenZip => Some(tr!(
                "Windows 11 and Linux open 7z on their own; macOS, phones and older Windows \
                 may need an archiver app. To share with anyone, pick .zip."
            )),
            Format::Tar | Format::TarGz | Format::TarXz | Format::TarBz2 => Some(tr!(
                "Linux, macOS and Windows 11 open it on their own; phones and older Windows \
                 may need an archiver app. To share with anyone, pick .zip."
            )),
            Format::TarZst => Some(tr!(
                "Linux and Windows 11 open it on their own; macOS, phones and older Windows \
                 may need an archiver app. To share with anyone, pick .zip."
            )),
            Format::TarLz4 => Some(tr!(
                "Most computers need an archiver app or the lz4 tool to open it. To share with \
                 anyone, pick .zip."
            )),
            Format::Gz | Format::Zst | Format::Xz | Format::Bz2 | Format::Lz4 => Some(tr!(
                "It holds this one file. Command-line tools open it; Windows and macOS may need \
                 an archiver app. To share with anyone, pick .zip."
            )),
        }
    }

    /// What packwing-core makes of it.
    fn kind(self) -> Kind {
        match self {
            Format::Zip => Kind::Zip,
            Format::SevenZip => Kind::SevenZip,
            Format::Tar => Kind::Tar,
            Format::TarGz => Kind::TarGz,
            Format::TarXz => Kind::TarXz,
            Format::TarZst => Kind::TarZst,
            Format::TarBz2 => Kind::TarBz2,
            Format::TarLz4 => Kind::TarLz4,
            Format::Gz => Kind::Gz,
            Format::Zst => Kind::Zst,
            Format::Xz => Kind::Xz,
            Format::Bz2 => Kind::Bz2,
            Format::Lz4 => Kind::Lz4,
        }
    }

    /// Whether its files can go in as they are, with [`Level::Store`]: a tar
    /// format has `.tar` for that.
    pub fn stores(self) -> bool {
        matches!(self, Format::Zip | Format::SevenZip)
    }

    /// Whether it is protected with age, around it, whatever is chosen: ZIP and 7z
    /// have their own AES-256, and age only if it is chosen.
    pub fn uses_age(self) -> bool {
        !matches!(self, Format::Zip | Format::SevenZip)
    }
}

/// The formats a [`Select`] offers: with those for one file by itself, if `one_file`.
fn format_options(one_file: bool) -> Vec<&'static str> {
    let formats: &[Format] = if one_file {
        &Format::WITH_ONE_FILE
    } else {
        &Format::ALL
    };
    formats.iter().map(|f| f.ext()).collect()
}

/// The levels, by the names they are kept under.
pub(crate) const LEVELS: [(Level, &str); 6] = [
    (Level::Store, "Store"),
    (Level::Lowest, "Lowest"),
    (Level::Low, "Low"),
    (Level::Medium, "Medium"),
    (Level::High, "High"),
    (Level::Highest, "Highest"),
];

/// A level as the user reads it.
pub(crate) fn level_name(level: Level) -> String {
    level.name()
}

/// How the archive is split: the last takes a size typed in.
fn splits() -> [String; 4] {
    [
        tr!("Don’t split"),
        tr!("25 MB (email)"),
        tr!("4 GB (USB stick)"),
        tr!("Custom size…"),
    ]
}
const CUSTOM: usize = 3;

/// The units a part's size is typed in, as the app counts sizes (`model::size`).
pub(super) const UNITS: [(&str, u64); 2] = [("MB", 1_000_000), ("GB", 1_000_000_000)];

/// The smallest part: a ZIP's parts are 64 KB or more, and parts this small
/// help nobody.
const SMALLEST_PART: u64 = 1_000_000;

/// The size of a part typed in, in bytes: a number of `unit`s, a comma
/// working as the decimal point, of [`SMALLEST_PART`] or more.
pub(super) fn part_size(typed: &str, unit: u64) -> Option<u64> {
    let n: f64 = typed.trim().replace(',', ".").parse().ok()?;
    let bytes = n * unit as f64;
    (bytes.is_finite() && bytes >= SMALLEST_PART as f64).then(|| bytes.round() as u64)
}

/// Whether `name` can name a file in a folder: nothing that leads elsewhere.
fn name_fits(name: &str) -> bool {
    !name.is_empty() && !name.contains(['/', '\\']) && name != "." && name != ".."
}

/// How hard a password is to guess, roughly: its length and the kinds of
/// characters in it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Strength {
    Weak,
    Fair,
    Strong,
}

fn strength(password: &str) -> Strength {
    let n = password.chars().count();
    let kinds = [
        password.chars().any(|c| c.is_lowercase()),
        password.chars().any(|c| c.is_uppercase()),
        password.chars().any(|c| c.is_numeric()),
        password.chars().any(|c| !c.is_alphanumeric()),
    ]
    .iter()
    .filter(|k| **k)
    .count();
    if n >= 16 || (n >= 12 && kinds >= 3) {
        Strength::Strong
    } else if n >= 8 {
        Strength::Fair
    } else {
        Strength::Weak
    }
}

/// The storages shown to choose where the archive goes.
struct Picking {
    chosen: usize,
    picker: Entity<Explorer>,
    _picker_changed: Subscription,
}

/// Keys read from a file, for age to encrypt to.
struct KeysFile {
    name: String,
    keys: Vec<String>,
    left_out: usize,
}

/// A format and level, and what was said of the memory they take.
type MemorySaid = ((Kind, Level, Frames), Option<String>);

pub struct CreateDialog {
    selection: Selection,
    /// It makes an archive of what the one archive chosen holds, not of the file.
    converting: bool,
    /// The list of what goes in is open.
    list_shown: bool,
    list_scroll: UniformListScrollHandle,
    /// Where the dialog's body has scrolled to.
    body_scroll: ScrollHandle,
    /// Where the archive goes.
    folder: Option<PathBuf>,
    /// Or this folder of a storage, if one was chosen.
    on: Option<(Place, Vec<String>)>,
    /// The storages it can go on, and the one to show first.
    places: Vec<Place>,
    first_place: usize,
    /// Where a storage that asks first is opened, and its watch.
    storages: Option<(Entity<StoragesPage>, Subscription)>,
    picking: Option<Picking>,
    /// The name it was given last: while the field still says it, more
    /// files change it.
    suggested: String,
    /// What the files are counted to hold, being counted.
    _counting: Option<Task<()>>,
    name: Entity<TextInput>,
    format: Entity<Select>,
    level: Level,
    /// How a zstd stream is cut, if the user chose: as its size says if not.
    frames: Option<Frames>,
    advanced: bool,
    split: Entity<Select>,
    part: Entity<TextInput>,
    unit: Entity<Select>,
    protect: bool,
    /// Whether the field holds a passphrase made here, shown as it is.
    passphrase_made: bool,
    password: Entity<TextInput>,
    /// With age: to keys, not to a password.
    to_keys: bool,
    /// A ZIP or 7z archive encrypted whole with age, not with its own AES-256.
    zip_age: bool,
    /// The key typed for age to encrypt to.
    key: Entity<TextInput>,
    /// Or the keys of a file, what it is called and how many it left out.
    keys_file: Option<KeysFile>,
    /// Why the file chosen gave no keys.
    keys_file_problem: Option<String>,
    hide_names: bool,
    /// Create was pressed with something missing: the notes say what.
    checked: bool,
    /// Why it did not start: another job makes an archive of that name there now.
    busy: Option<String>,
    /// What [`packwing_core::memory_warning`] said of a format and level, kept
    /// so that the memory is not measured each time the dialog is drawn.
    memory_said: std::cell::RefCell<Option<MemorySaid>>,
    _follow: Vec<Subscription>,
}

impl EventEmitter<DialogEvent> for CreateDialog {}

impl Choosing for CreateDialog {
    fn selection(&self) -> &Selection {
        &self.selection
    }

    fn list_shown(&self) -> bool {
        self.list_shown
    }

    fn list_scroll(&self) -> &UniformListScrollHandle {
        &self.list_scroll
    }

    fn show_list(&mut self, shown: bool) {
        self.list_shown = shown;
    }

    fn pick(&mut self, folders: bool, cx: &mut Context<Self>) {
        self.pick_paths(folders, cx);
    }

    fn remove(&mut self, path: &Path, cx: &mut Context<Self>) {
        self.selection.remove(path);
        self.list_shown &= !self.selection.paths.is_empty();
        self.changed_what_goes_in(cx);
    }
}

impl CreateDialog {
    pub fn new(selection: Selection, cx: &mut Context<Self>) -> Self {
        let kept = crate::prefs::prefs(cx);
        let chosen = Format::ALL
            .iter()
            .position(|f| *f == kept.format)
            .unwrap_or(0);
        let suggested = if selection.paths.is_empty() {
            String::new()
        } else {
            selection.suggested_name()
        };
        let name = cx.new(|cx| {
            let mut field = TextInput::new(tr!("Name"), cx);
            field.set_text(&suggested, suggested.len()..suggested.len(), cx);
            field
        });
        let format = cx.new(|_| {
            Select::new(
                &format_options(selection.one_file()),
                chosen,
                SelectLook::Pill,
            )
            .marked("format")
        });
        let split = cx.new(|_| Select::new(&splits(), 0, SelectLook::Pill).marked("split"));
        let part = cx.new(|cx| TextInput::new(tr!("Size"), cx));
        let unit =
            cx.new(|_| Select::new(&UNITS.map(|(u, _)| u), 0, SelectLook::Pill).marked("unit"));
        let password = cx.new(|cx| TextInput::new(tr!("Password"), cx).masked());
        let key = cx.new(|cx| TextInput::new(tr!("age1…, age1pq1… or ssh-ed25519 …"), cx));
        // What is drawn follows what is typed and chosen: the extension, the
        // notes, the password's strength.
        let follow = vec![
            cx.subscribe(&format, |this: &mut Self, _, _: &Chosen, cx| {
                this.follow_name(cx);
                cx.notify();
            }),
            cx.subscribe(&split, |_, _, _: &Chosen, cx| cx.notify()),
            cx.subscribe(&part, |_, _, _: &Changed, cx| cx.notify()),
            cx.subscribe(&unit, |_, _, _: &Chosen, cx| cx.notify()),
            cx.subscribe(&password, |_, _, _: &Changed, cx| cx.notify()),
            cx.subscribe(&key, |_, _, _: &Changed, cx| cx.notify()),
        ];
        CreateDialog {
            converting: false,
            folder: selection.folder(),
            on: None,
            places: Vec::new(),
            first_place: 0,
            storages: None,
            picking: None,
            selection,
            list_shown: false,
            list_scroll: UniformListScrollHandle::new(),
            body_scroll: ScrollHandle::new(),
            suggested,
            _counting: None,
            name,
            format,
            level: kept.level,
            frames: None,
            // What has to be typed is not hidden.
            advanced: crate::managed::policy(cx).requires_encryption(),
            split,
            part,
            unit,
            protect: crate::managed::policy(cx).requires_encryption(),
            passphrase_made: false,
            password,
            to_keys: false,
            zip_age: false,
            key,
            keys_file: None,
            keys_file_problem: None,
            hide_names: kept.hide_names,
            checked: false,
            busy: None,
            memory_said: std::cell::RefCell::new(None),
            _follow: follow,
        }
    }

    /// With `places` to save on, the one at `first` shown first: those that ask
    /// first open through `storages`.
    pub fn with_places(
        mut self,
        places: Vec<Place>,
        first: usize,
        storages: Entity<StoragesPage>,
        cx: &mut Context<Self>,
    ) -> Self {
        self.places = places;
        self.first_place = first;
        let watch = cx.observe(&storages, |this: &mut Self, storages, cx| {
            if let Some(picking) = &this.picking {
                places::follow_lock(&this.places, picking.chosen, &picking.picker, &storages, cx);
            }
            cx.notify();
        });
        self.storages = Some((storages, watch));
        self
    }

    /// The place at `i`, to browse: none while it asks first and has not.
    fn open_place(&self, i: usize, cx: &App) -> Option<Place> {
        match &self.storages {
            Some((storages, _)) => places::open_place(&self.places, i, storages, cx),
            None => self.places.get(i).cloned(),
        }
    }

    /// Chooses several zstd frames, or one, as a user would.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn choose_frames(&mut self, several: bool, cx: &mut Context<Self>) {
        self.frames = Some(if several {
            Frames::Several
        } else {
            Frames::One
        });
        cx.notify();
    }

    /// How the archive would be cut into frames, for tests.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn frames_chosen(&self, cx: &App) -> Frames {
        self.frames(cx)
    }

    #[cfg(all(test, feature = "test-hooks"))]
    pub fn show_advanced(&mut self, cx: &mut Context<Self>) {
        self.advanced = true;
        cx.notify();
    }

    /// Fills the dialog as a user might: `format`, split into parts of a size
    /// typed in (`part`, maybe nothing yet), with a password asked for.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn fill(&mut self, format: Format, part: &str, cx: &mut Context<Self>) {
        self.choose_format(format, cx);
        self.split.update(cx, |s, cx| s.choose(CUSTOM, cx));
        self.part
            .update(cx, |t, cx| t.set_text(part, part.len()..part.len(), cx));
        self.protect = true;
        cx.notify();
    }

    /// Encrypts a ZIP or 7z archive whole with age, as pressing age would, for tests.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn choose_age(&mut self, cx: &mut Context<Self>) {
        self.zip_age = true;
        cx.notify();
    }

    /// Chooses `format`, as pressing it would.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn choose_format(&mut self, format: Format, cx: &mut Context<Self>) {
        let i = Format::WITH_ONE_FILE
            .iter()
            .position(|f| *f == format)
            .unwrap_or(0);
        self.format.update(cx, |s, cx| s.choose(i, cx));
    }

    /// Opens the menu of formats, for tests.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn open_formats(&mut self, cx: &mut Context<Self>) {
        self.format.update(cx, |s, cx| s.open_menu(cx));
    }

    /// Scrolls to the end of the dialog, for tests.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn scroll_to_end(&mut self, cx: &mut Context<Self>) {
        self.body_scroll.scroll_to_bottom();
        cx.notify();
    }

    /// Asks for encryption to `key`, typed, for tests.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn encrypt_to_key(&mut self, key: &str, cx: &mut Context<Self>) {
        self.protect = true;
        self.to_keys = true;
        self.advanced = true;
        self.key
            .update(cx, |t, cx| t.set_text(key, key.len()..key.len(), cx));
        cx.notify();
    }

    /// The name typed, for tests.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn name_typed(&self, cx: &App) -> String {
        self.name.read(cx).text().to_owned()
    }

    #[cfg(all(test, feature = "test-hooks"))]
    pub fn open_list(&mut self, cx: &mut Context<Self>) {
        self.list_shown = true;
        cx.notify();
    }

    #[cfg(all(test, feature = "test-hooks"))]
    pub fn list_is_open(&self) -> bool {
        self.list_shown
    }

    /// How far the list and the dialog's body have scrolled, for tests.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn scrolled(&self) -> (gpui::Pixels, gpui::Pixels) {
        (
            self.list_scroll.0.borrow().base_handle.offset().y,
            self.body_scroll.offset().y,
        )
    }

    #[cfg(all(test, feature = "test-hooks"))]
    pub fn selection_paths(&self) -> Vec<PathBuf> {
        self.selection.paths.clone()
    }

    #[cfg(all(test, feature = "test-hooks"))]
    pub fn format_chosen(&self, cx: &App) -> Format {
        self.format(cx)
    }

    /// What the files come to, as if they had been counted, for tests.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn set_counted(&mut self, bytes: u64, cx: &mut Context<Self>) {
        self.selection.bytes = bytes;
        cx.notify();
    }

    /// Whether it is choosing a storage, for tests.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn is_picking(&self) -> bool {
        self.picking.is_some()
    }

    /// Chooses the storage at `i` and the folder it opens on, as pressing them would.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn press_storage(&mut self, i: usize, cx: &mut Context<Self>) {
        self.pick_storage(cx);
        self.pick_place(i, cx);
    }

    #[cfg(all(test, feature = "test-hooks"))]
    pub fn press_save_here(&mut self, cx: &mut Context<Self>) {
        self.save_here(cx);
    }

    #[cfg(all(test, feature = "test-hooks"))]
    pub fn press_create(&mut self, cx: &mut Context<Self>) {
        self.start(cx);
    }

    /// Adds `paths` to what goes in the archive, and counts what is in them.
    /// Convert: an archive of what the archive at `path` holds, beside it.
    pub fn converting(path: PathBuf, cx: &mut Context<Self>) -> Self {
        let mut dialog = CreateDialog::new(Selection::default(), cx);
        dialog.converting = true;
        dialog.add(vec![path], cx);
        dialog
    }

    pub fn add(&mut self, paths: Vec<PathBuf>, cx: &mut Context<Self>) {
        // What is converted is the one archive.
        if self.converting && !self.selection.paths.is_empty() {
            return;
        }
        self.selection.add(paths);
        if self.folder.is_none() {
            self.folder = self.selection.folder();
        }
        self.changed_what_goes_in(cx);
    }

    /// Offers the formats that fit what goes in. One for a single file that no longer
    /// fits gives way to the one new archives start with.
    fn follow_formats(&mut self, cx: &mut Context<Self>) {
        // What an archive holds is not one file, as far as is known.
        let one_file = self.selection.one_file() && !self.converting;
        let format = self.format(cx);
        let format = if format.one_file() && !one_file {
            crate::prefs::prefs(cx).format
        } else {
            format
        };
        let chosen = Format::WITH_ONE_FILE
            .iter()
            .position(|f| *f == format)
            .unwrap_or(0);
        self.format.update(cx, |s, cx| {
            s.set_options(&format_options(one_file), chosen, cx)
        });
    }

    /// Follows what goes in and the format: the name, until one is typed. A file by
    /// itself keeps all its name, as `dump.sql.zst`.
    fn follow_name(&mut self, cx: &mut Context<Self>) {
        let suggested = if self.selection.paths.is_empty() {
            String::new()
        } else if self.converting {
            packwing_core::stem(&self.selection.whole_name()).to_owned()
        } else if self.format(cx).one_file() {
            self.selection.whole_name()
        } else {
            self.selection.suggested_name()
        };
        if self.name.read(cx).text() == self.suggested {
            self.name.update(cx, |f, cx| {
                f.set_text(&suggested, suggested.len()..suggested.len(), cx)
            });
            self.suggested = suggested;
        }
    }

    /// Follows what goes in: the formats, the name and the count.
    fn changed_what_goes_in(&mut self, cx: &mut Context<Self>) {
        self.follow_formats(cx);
        self.follow_name(cx);
        let paths = self.selection.paths.clone();
        let counting = cx
            .background_executor()
            .spawn(async move { packwing_core::measure(&paths) });
        self._counting = Some(cx.spawn(async move |this, cx| {
            // One that cannot be read says so when the archive is made.
            let measured = counting.await.unwrap_or_default();
            let _ = this.update(cx, |this, cx| {
                this.selection.files = measured.files;
                this.selection.bytes = measured.bytes;
                this.selection.not_portable = measured.not_portable;
                cx.notify();
            });
        }));
        cx.notify();
    }

    /// Asks for files, or for a folder, to add.
    fn pick_paths(&mut self, folders: bool, cx: &mut Context<Self>) {
        let picked = cx.prompt_for_paths(PathPromptOptions {
            files: !folders,
            directories: folders,
            multiple: true,
            prompt: Some(tr!("Add").into()),
        });
        cx.spawn(async move |this, cx| {
            if let Ok(Ok(Some(paths))) = picked.await {
                let _ = this.update(cx, |this, cx| this.add(paths, cx));
            }
        })
        .detach();
    }

    /// Asks for the folder the archive goes in.
    fn pick_folder(&mut self, cx: &mut Context<Self>) {
        let picked = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some(tr!("Save here").into()),
        });
        cx.spawn(async move |this, cx| {
            if let Ok(Ok(Some(mut paths))) = picked.await
                && let Some(folder) = paths.pop()
            {
                let _ = this.update(cx, |this, cx| {
                    this.folder = Some(folder);
                    this.on = None;
                    cx.notify();
                });
            }
        })
        .detach();
    }

    /// Shows the storages, to choose where the archive goes.
    fn pick_storage(&mut self, cx: &mut Context<Self>) {
        let (chosen, folder) = match &self.on {
            Some((place, folder)) => (
                self.places
                    .iter()
                    .position(|p| p.name == place.name && p.tag == place.tag)
                    .unwrap_or(self.first_place),
                folder.clone(),
            ),
            None => (self.first_place, Vec::new()),
        };
        let start = self.open_place(chosen, cx);
        let picker = cx.new(|cx| {
            let mut picker = Explorer::picker(cx);
            picker.show_place(start, cx);
            picker.open_folder(folder, cx);
            picker
        });
        let changed = cx.observe(&picker, |_, _, cx| cx.notify());
        self.picking = Some(Picking {
            chosen,
            picker,
            _picker_changed: changed,
        });
        cx.notify();
    }

    fn pick_place(&mut self, i: usize, cx: &mut Context<Self>) {
        let place = self.open_place(i, cx);
        if let Some(picking) = &mut self.picking {
            picking.chosen = i;
            picking.picker.update(cx, |p, cx| p.show_place(place, cx));
        }
        cx.notify();
    }

    /// The archive goes in the folder the storages show.
    /// Nothing, while the storage chosen asks first.
    fn save_here(&mut self, cx: &mut Context<Self>) {
        let shown = self.picking.as_ref().and_then(|picking| {
            let picker = picking.picker.read(cx);
            Some((picker.place().cloned()?, picker.folder().to_vec()))
        });
        if let Some(on) = shown {
            self.on = Some(on);
            self.picking = None;
        }
        cx.notify();
    }

    fn format(&self, cx: &App) -> Format {
        Format::of_option(self.format.read(cx).chosen())
    }

    fn custom_split(&self, cx: &App) -> bool {
        self.split.read(cx).chosen() == CUSTOM
    }

    /// The size typed for a part, if it is one.
    fn part_size(&self, cx: &App) -> Option<u64> {
        let (_, unit) = UNITS[self.unit.read(cx).chosen().min(UNITS.len() - 1)];
        part_size(self.part.read(cx).text(), unit)
    }

    /// Whether it is to be protected.
    fn protected(&self) -> bool {
        self.protect
    }

    /// The level it is made at: Store, where the format has none, is its lowest.
    fn level(&self, cx: &App) -> Level {
        if self.level == Level::Store && !self.format(cx).stores() {
            Level::Lowest
        } else {
            self.level
        }
    }

    /// Whether the choice of frames is shown: for zstd, of more than a frame.
    fn frames_shown(&self, cx: &App) -> bool {
        matches!(self.format(cx).kind(), Kind::TarZst | Kind::Zst)
            && self.selection.bytes > packwing_core::FRAME_SIZE
    }

    /// Whether the frames shown are several: as chosen, or as the size says.
    fn several_frames(&self) -> bool {
        self.frames
            .unwrap_or_default()
            .several(Some(self.selection.bytes))
    }

    /// How it is cut into frames: as chosen where the choice is shown, as the
    /// size says otherwise, which is then counted again.
    fn frames(&self, cx: &App) -> Frames {
        match self.frames {
            Some(frames) if self.frames_shown(cx) => frames,
            _ => Frames::Auto,
        }
    }

    /// Whether what it is protected with is age: always, but for a ZIP or 7z
    /// archive left to its own AES-256.
    fn with_age(&self, cx: &App) -> bool {
        self.format(cx).uses_age() || self.zip_age
    }

    /// Whether it is to be encrypted to keys with age.
    fn encrypted_to_keys(&self, cx: &App) -> bool {
        self.protect && self.to_keys && self.with_age(cx)
    }

    /// The keys age is to encrypt to: those of the file, or the one typed if it is one.
    fn age_keys(&self, cx: &App) -> Vec<String> {
        if !self.encrypted_to_keys(cx) {
            return Vec::new();
        }
        if let Some(file) = &self.keys_file {
            return file.keys.clone();
        }
        let typed = self.key.read(cx).text().trim();
        if packwing_core::age_key_problem(typed).is_none() {
            vec![typed.to_owned()]
        } else {
            Vec::new()
        }
    }

    /// Asks for a file of keys for age to encrypt to, and reads it.
    fn pick_keys_file(&mut self, cx: &mut Context<Self>) {
        let picked = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some(tr!("Use these keys").into()),
        });
        cx.spawn(async move |this, cx| {
            let Ok(Ok(Some(mut paths))) = picked.await else {
                return;
            };
            let Some(path) = paths.pop() else {
                return;
            };
            let read = cx
                .background_executor()
                .spawn(async move {
                    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
                    let name = path
                        .file_name()
                        .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
                    let (keys, left_out) = packwing_core::age_keys_in(&bytes)?;
                    Ok::<_, String>(KeysFile {
                        name,
                        keys,
                        left_out,
                    })
                })
                .await;
            let _ = this.update(cx, |this, cx| this.use_keys_file(read, cx));
        })
        .detach();
    }

    fn use_keys_file(&mut self, read: Result<KeysFile, String>, cx: &mut Context<Self>) {
        match read {
            Ok(file) => {
                self.keys_file = Some(file);
                self.keys_file_problem = None;
            }
            Err(why) => {
                self.keys_file = None;
                self.keys_file_problem = Some(why);
            }
        }
        cx.notify();
    }

    /// Whatever Advanced asks for is there: a part's size when one is to be
    /// typed, a password or a key when one is asked for.
    fn advanced_ready(&self, cx: &App) -> bool {
        let part = !self.custom_split(cx) || self.part_size(cx).is_some();
        let secret = if self.encrypted_to_keys(cx) {
            !self.age_keys(cx).is_empty()
        } else {
            !self.protected() || !self.password.read(cx).text().is_empty()
        };
        part && secret
    }

    /// Puts a passphrase of random words in the field, for the user to read.
    fn suggest(&mut self, cx: &mut Context<Self>) {
        let least = crate::managed::policy(cx)
            .least_password_length()
            .unwrap_or(0);
        let Ok(made) = packwing_core::suggest_passphrase(least as usize) else {
            return;
        };
        self.password.update(cx, |field, cx| {
            field.set_masked(false, cx);
            field.set_text(&made, made.len()..made.len(), cx);
        });
        self.passphrase_made = true;
        cx.notify();
    }

    /// What to say of the memory the format and level chosen take, if even
    /// one thread takes more than is free: measured again only when they
    /// change.
    fn memory_warning(&self, cx: &App) -> Option<String> {
        let chosen = (self.format(cx).kind(), self.level(cx), self.frames(cx));
        let mut said = self.memory_said.borrow_mut();
        if let Some((was, warning)) = said.as_ref()
            && *was == chosen
        {
            return warning.clone();
        }
        let warning = packwing_core::memory_warning(&NewArchive {
            kind: chosen.0,
            level: chosen.1,
            frames: chosen.2,
            ..NewArchive::default()
        });
        *said = Some((chosen, warning.clone()));
        warning
    }

    /// How encrypted the archive would be, for what a policy says of it.
    fn encryption(&self, cx: &App) -> NewArchive {
        let age_keys = self.age_keys(cx);
        let password = self.protected() && age_keys.is_empty();
        NewArchive {
            kind: self.format(cx).kind(),
            password: password.then(|| self.password.read(cx).text().as_bytes().to_vec()),
            age_keys,
            whole_with_age: self.zip_age,
            ..NewArchive::default()
        }
    }

    /// Why the organization's policy stops this, as it stands.
    fn refused(&self, cx: &App) -> Option<String> {
        packwing_core::refused_by(crate::managed::policy(cx), &self.encryption(cx))
            .err()
            .map(|why| why.to_string())
    }

    /// How big each part is, if it is split.
    fn parts(&self, cx: &App) -> Option<u64> {
        match self.split.read(cx).chosen() {
            0 => None,
            1 => Some(25_000_000),
            2 => Some(4_000_000_000),
            _ => self.part_size(cx),
        }
    }

    /// It did not start, for `why`: said over its buttons, until it is started again.
    pub fn refuse(&mut self, why: String, cx: &mut Context<Self>) {
        self.busy = Some(why);
        cx.notify();
    }

    fn start(&mut self, cx: &mut Context<Self>) {
        self.busy = None;
        cx.notify();
        let format = self.format(cx);
        let typed = self.name.read(cx).text().trim().to_owned();
        let chosen_place = self.on.is_some() || self.folder.is_some();
        if self.selection.paths.is_empty()
            || !chosen_place
            || !name_fits(&typed)
            || !self.advanced_ready(cx)
            || self.refused(cx).is_some()
        {
            self.checked = true;
            // What is missing may be under Advanced, which may be hidden.
            self.advanced |= !self.advanced_ready(cx) || self.refused(cx).is_some();
            cx.notify();
            return;
        }
        let name = self.file_name(cx);
        let age_keys = self.age_keys(cx);
        let password = self.protected() && age_keys.is_empty();
        let (shown, to) = match (&self.on, &self.folder) {
            (Some((place, folder)), _) => (
                places::shown(&place.name, folder),
                Saving::On(SavingOn {
                    place: place.clone(),
                    folder: folder.clone(),
                    name: name.clone(),
                    uploading: Arc::default(),
                }),
            ),
            (None, Some(folder)) => (
                folder.display().to_string(),
                Saving::Here(folder.join(&name)),
            ),
            (None, None) => return,
        };
        let mut job = Job::shown(
            if self.converting {
                Verb::Convert
            } else {
                Verb::Create
            },
            name.into(),
            Some(shown.into()),
            self.selection.files,
            self.selection.bytes,
        );
        job.create = Some(Creating {
            paths: self.selection.paths.clone(),
            to,
            how: NewArchive {
                kind: format.kind(),
                level: self.level(cx),
                password: password.then(|| self.password.read(cx).text().as_bytes().to_vec()),
                age_keys,
                whole_with_age: self.zip_age,
                hide_names: self.hide_names && !self.with_age(cx),
                stored_extensions: crate::prefs::stored_extensions(cx),
                skip_mac_files: crate::prefs::prefs(cx).skip_mac_files,
                part_size: self.parts(cx),
                stream_above: Some(crate::prefs::prefs(cx).stream_above),
                frames: self.frames(cx),
            },
        });
        if self.converting
            && let Some(Creating {
                paths,
                to: Saving::Here(to),
                how,
            }) = job.create.take()
            && let Some(from) = paths.into_iter().next()
        {
            job.convert = Some(Converting {
                from: Source::from(from.as_path()),
                to,
                how,
            });
        }
        cx.emit(DialogEvent::Start(job));
    }

    /// What Advanced has changed, said beside it while it is hidden: the
    /// level, the parts, a password.
    fn changed(&self, cx: &App) -> Vec<String> {
        let mut said = Vec::new();
        match self.level(cx) {
            Level::Medium => {}
            Level::Store => said.push(tr!("No compression")),
            level => said.push(tr!("{name} compression", name = level_name(level))),
        }
        // The frames, if chosen other than the size says.
        if let Some(chosen) = self.frames
            && self.frames_shown(cx)
            && (chosen == Frames::Several) != Frames::Auto.several(Some(self.selection.bytes))
        {
            said.push(if chosen == Frames::Several {
                tr!("Several frames")
            } else {
                tr!("One frame")
            });
        }
        match self.split.read(cx).chosen() {
            0 => {}
            1 => said.push(tr!("Parts of 25 MB")),
            2 => said.push(tr!("Parts of 4 GB")),
            _ => said.push(match self.part_size(cx) {
                Some(bytes) => tr!("Parts of {size}", size = size(bytes)),
                None => tr!("Parts of a size to type"),
            }),
        }
        if self.encrypted_to_keys(cx) {
            said.push(tr!("Public key"));
        } else if self.protected() {
            said.push(tr!("Password"));
        }
        said
    }

    /// What goes in the archive, and where it goes.
    fn contents(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let shown = match &self.on {
            Some((place, folder)) => places::shown(&place.name, folder),
            None => self
                .folder
                .as_ref()
                .map_or_else(|| tr!("Beside the files"), |f| f.display().to_string()),
        };
        let about_storage = self.about_storage(cx);
        div()
            .flex()
            .flex_col()
            .gap(ui(10.))
            .when(!self.converting, |d| {
                d.child(chosen::section(self, self.checked, cx))
            })
            .when(self.converting, |d| {
                d.child(
                    div()
                        .flex()
                        .flex_col()
                        .child(legend(tr!("What it holds, from")))
                        .child(
                            div()
                                .truncate()
                                .text_size(ui(13.))
                                .text_color(rgb(c.muted))
                                .child(self.selection.whole_name()),
                        ),
                )
            })
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(ui(8.))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .child(legend(tr!("Save in")))
                            .child(
                                div()
                                    .truncate()
                                    .text_size(ui(13.))
                                    .text_color(rgb(c.muted))
                                    .child(shown),
                            ),
                    )
                    .child(
                        button("change-folder", tr!("Folder…"), Fill::Outline, Size::Medium)
                            .on_click(cx.listener(|this, _, _, cx| this.pick_folder(cx))),
                    )
                    // With no storage of the user's, Folder… is all there is.
                    .when(
                        !self.places.is_empty() && crate::prefs::has_storage(cx),
                        |d| {
                            d.child(
                                button(
                                    "change-storage",
                                    tr!("Storage…"),
                                    Fill::Outline,
                                    Size::Medium,
                                )
                                .on_click(cx.listener(|this, _, _, cx| this.pick_storage(cx))),
                            )
                        },
                    ),
            )
            .children(about_storage)
    }

    /// What going to the storage chosen means for this archive, if it matters:
    /// more than it takes to go straight there, or more than it can take at all.
    fn about_storage(&self, cx: &App) -> Vec<gpui::Div> {
        let Some((place, folder)) = &self.on else {
            return Vec::new();
        };
        let bytes = self.selection.bytes;
        let prefs = crate::prefs::prefs(cx);
        let format = self.format(cx);
        let how = NewArchive {
            kind: format.kind(),
            part_size: self.parts(cx),
            stream_above: Some(prefs.stream_above),
            ..NewArchive::default()
        };
        let storage = place.storage.as_ref();
        let mut said = Vec::new();
        if bytes > prefs.stream_above
            && packwing_core::route(storage, folder, &how, bytes) == Route::Temporary
        {
            let text = if how.part_size.is_some() {
                tr!(
                    "Parts are made in this computer’s temporary folder first, then uploaded, so they need room there."
                )
            } else if storage.sends_as_read() {
                tr!(
                    "{ext} is made in this computer’s temporary folder first, then uploaded, so it needs room there. .zip and tar formats go straight to this storage.",
                    ext = format.ext()
                )
            } else {
                tr!(
                    "It is made in this computer’s temporary folder first, then uploaded, so it needs room there."
                )
            };
            said.push(note(Tone::Warn).child(text));
        }
        if let Some(most) = storage.most_file().filter(|most| bytes > *most) {
            said.push(note(Tone::Warn).child(tr!(
                "{place} takes files of up to {most}. An archive larger than that fails when it starts to upload.",
                place = place.name.clone(),
                most = size(most)
            )));
        }
        said
    }

    /// The storages and their folders, in place of the form, as a list that scrolls
    /// has no place in a body that does.
    fn storages_shown(&self, picking: &Picking, cx: &mut Context<Self>) -> Stateful<Div> {
        let c = colors();
        let back = |this: &mut Self, cx: &mut Context<Self>| {
            this.picking = None;
            cx.notify();
        };
        let close = icon_button("close", "x", 40.)
            .on_click(cx.listener(move |this, _, _, cx| back(this, cx)));
        let picker = picking.picker.clone();
        super::card()
            .id("save-in")
            .w(ui(900.))
            .child(header(
                "storages",
                tonal_tint(),
                tr!("Save in"),
                tr!("Choose a storage, and a folder in it"),
                Some(close),
            ))
            .child(
                div()
                    .flex()
                    .gap(ui(8.))
                    .h(ui(340.))
                    .flex_shrink_0()
                    .p(ui(10.))
                    .rounded(ui(22.))
                    .bg(rgb(c.well))
                    .child(places::list(
                        &self.places,
                        picking.chosen,
                        Self::pick_place,
                        cx,
                    ))
                    .child(div().flex_1().min_w_0().child(match &self.storages {
                        Some((storages, _)) => places::beside(
                            &self.places,
                            picking.chosen,
                            &picking.picker,
                            storages,
                            cx,
                        ),
                        None => picking.picker.clone().into_any_element(),
                    })),
            )
            .child(
                footer()
                    .child(
                        button("new-folder", tr!("New folder"), Fill::Text, Size::Medium).on_click(
                            move |_, window, cx| {
                                picker.update(cx, |p, cx| p.new_folder(window, cx));
                            },
                        ),
                    )
                    .child(div().flex_1())
                    .child(
                        button("cancel", tr!("Cancel"), Fill::Outline, Size::Medium)
                            .on_click(cx.listener(move |this, _, _, cx| back(this, cx))),
                    )
                    .child(
                        button("save-here", tr!("Save here"), Fill::Accent, Size::Medium)
                            .on_click(cx.listener(|this, _, _, cx| this.save_here(cx))),
                    ),
            )
    }

    /// The new archive's file name: what is typed, and what the format and Advanced
    /// add to it.
    fn file_name(&self, cx: &App) -> String {
        let typed = self.name.read(cx).text().trim();
        let encrypted = self.protected() && self.with_age(cx);
        format!(
            "{typed}{}{}",
            self.format(cx).ext(),
            if encrypted { ".age" } else { "" }
        )
    }

    /// The name it will get instead, if a file of its name is in the folder chosen on
    /// this computer already.
    fn renamed(&self, cx: &App) -> Option<String> {
        let folder = self.folder.as_ref().filter(|_| self.on.is_none())?;
        if !name_fits(self.name.read(cx).text().trim()) {
            return None;
        }
        let wanted = folder.join(self.file_name(cx));
        let free = packwing_core::free_archive_path(&wanted, self.parts(cx).is_some());
        if free == wanted {
            return None;
        }
        Some(free.file_name()?.to_string_lossy().into_owned())
    }

    fn name_field(&self, cx: &App) -> impl IntoElement {
        let c = colors();
        div()
            .flex()
            .flex_col()
            .gap(ui(6.))
            .child(legend(tr!("Name")))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(ui(8.))
                    .h(ui(40.))
                    .pl(ui(18.))
                    .pr(ui(6.))
                    .rounded(ui(16.))
                    .bg(rgb(c.field))
                    .text_size(ui(16.))
                    .child(self.name.clone())
                    .child(self.format.clone()),
            )
            .when(
                self.checked && !name_fits(self.name.read(cx).text().trim()),
                |d| {
                    d.child(
                        note(Tone::Warn)
                            .mt(ui(4.))
                            .child(tr!("Give it a name, without / or \\ in it.")),
                    )
                },
            )
            .children(self.renamed(cx).map(|name| {
                note(Tone::Warn)
                    .mt(ui(4.))
                    .debug_selector(|| "renamed".to_owned())
                    .child(tr!(
                        "A file of that name is there already: this one will be {name}.",
                        name = name
                    ))
            }))
            .children(
                self.format(cx)
                    .caveat()
                    .map(|caveat| note(Tone::Warn).mt(ui(4.)).child(caveat)),
            )
            .children(self.selection.not_portable.first().map(|name| {
                // What it holds is no reason not to make it: said, not refused.
                note(Tone::Warn)
                    .mt(ui(4.))
                    .debug_selector(|| "not-portable".to_owned())
                    .child(tr!(
                        "Some names can’t be used on Windows or macOS, such as “{name}”. There, those files may not extract as they are.",
                        name = name.trim_end_matches('/')
                    ))
            }))
    }

    fn advanced_toggle(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let changed = if self.advanced {
            Vec::new()
        } else {
            self.changed(cx)
        };
        div()
            .flex()
            .items_center()
            .gap(ui(10.))
            .child(
                div()
                    .debug_selector(|| "advanced".to_owned())
                    .id("advanced")
                    .flex_shrink_0()
                    .px(ui(4.))
                    .py(ui(8.))
                    .text_size(ui(14.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(c.accent))
                    .cursor_pointer()
                    .hover(|s| s.underline())
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.advanced = !this.advanced;
                        cx.notify();
                    }))
                    .child(if self.advanced {
                        tr!("Hide advanced")
                    } else {
                        tr!("Advanced options")
                    }),
            )
            .when(!changed.is_empty(), |d| {
                d.child(
                    div()
                        .min_w_0()
                        .truncate()
                        .text_size(ui(13.))
                        .text_color(rgb(c.muted))
                        .child(changed.join(" · ")),
                )
            })
    }

    fn compression(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let stores = self.format(cx).stores();
        let now = self.level(cx);
        let levels = LEVELS
            .iter()
            .filter(|(level, _)| stores || *level != Level::Store)
            .map(|&(level, label)| {
                segment(label, level_name(level), level == now).on_click(cx.listener(
                    move |this, _, _, cx| {
                        this.level = level;
                        cx.notify();
                    },
                ))
            });
        let several = self.several_frames();
        let frames = [(true, "frames-several"), (false, "frames-one")].map(|(many, id)| {
            let label = if many {
                tr!("Several frames")
            } else {
                tr!("One frame")
            };
            segment(id, label, many == several).on_click(cx.listener(move |this, _, _, cx| {
                this.frames = Some(if many { Frames::Several } else { Frames::One });
                cx.notify();
            }))
        });
        let custom = self.custom_split(cx);
        let typed = !self.part.read(cx).text().trim().is_empty();
        let bad_part = custom && self.part_size(cx).is_none() && (typed || self.checked);
        div()
            .flex()
            .flex_col()
            .gap(ui(12.))
            .px(ui(16.))
            .py(ui(14.))
            .rounded(ui(20.))
            .bg(rgb(c.well))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(ui(6.))
                    .child(
                        div()
                            .text_size(ui(13.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(tr!("Compression level")),
                    )
                    .child(segments(levels))
                    .child(div().text_size(ui(13.)).text_color(rgb(c.muted)).child(
                        if now == Level::Store {
                            tr!("Store puts the files in as they are: the quickest to make, and the largest.")
                        } else {
                            tr!("Higher makes a smaller archive, which takes longer to make, not to open.")
                        },
                    ))
                    .when_some(self.memory_warning(cx), |d, warning| {
                        d.child(
                            div()
                                .debug_selector(|| "create-memory".into())
                                .text_size(ui(13.))
                                .text_color(rgb(c.warn_text))
                                .child(warning),
                        )
                    }),
            )
            .when(self.frames_shown(cx), |d| {
                d.child(
                    div()
                        .debug_selector(|| "frames".into())
                        .flex()
                        .flex_col()
                        .gap(ui(6.))
                        .child(
                            div()
                                .text_size(ui(13.))
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(tr!("Frames")),
                        )
                        .child(segments(frames))
                        .child(div().text_size(ui(13.)).text_color(rgb(c.muted)).child(
                            if several {
                                tr!("Several frames open on every core, and make the archive a little larger: under half a percent.")
                            } else {
                                tr!("One frame makes the smallest archive, which opens on one core.")
                            },
                        )),
                )
            })
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(ui(12.))
                    .child(
                        div()
                            .flex_1()
                            .text_size(ui(14.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(tr!("Split into parts")),
                    )
                    .child(self.split.clone()),
            )
            .when(custom, |d| {
                d.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(ui(8.))
                        .child(div().flex_1().text_size(ui(14.)).child(tr!("Each part")))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .w(ui(120.))
                                .h(ui(40.))
                                .px(ui(14.))
                                .rounded_full()
                                .border_1()
                                .border_color(rgb(if bad_part { c.warn_text } else { c.outline }))
                                .bg(rgb(c.surface))
                                .text_size(ui(14.))
                                .child(self.part.clone()),
                        )
                        .child(self.unit.clone()),
                )
            })
            .when(self.split.read(cx).chosen() != 0, |d| {
                d.child(
                    div().text_size(ui(13.)).text_color(rgb(c.muted)).child(
                        tr!("Parts open in 7-Zip and archivers like it, not in the tools built into Windows and macOS."),
                    ),
                )
            })
            .when(bad_part, |d| {
                d.child(
                    note(Tone::Warn).child(tr!("A part takes a size of 1 MB or more, such as 100 MB.")),
                )
            })
    }

    fn protection(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let format = self.format(cx);
        let on = self.protected();
        let age = self.with_age(cx);
        let to_keys = self.encrypted_to_keys(cx);
        let typed = self.password.read(cx).text().to_string();
        let strength = (!typed.is_empty()).then(|| strength(&typed));
        let saved_as = format!("{}{}.age", self.name.read(cx).text().trim(), format.ext());
        let required = crate::managed::policy(cx).requires_encryption();
        let refused = self.refused(cx);
        div()
            .flex()
            .flex_col()
            .gap(ui(12.))
            .p(ui(16.))
            .rounded(ui(20.))
            .bg(rgb(c.well))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(ui(12.))
                    .child(icon("lock", 22., c.accent))
                    .child(
                        div()
                            .flex_1()
                            .text_size(ui(15.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(if age {
                                tr!("Encrypt with age")
                            } else {
                                tr!("Protect with a password")
                            }),
                    )
                    .child(if required {
                        icon("lock", 18., c.muted).into_any_element()
                    } else {
                        switch("protect", on)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.protect = !this.protect;
                                cx.notify();
                            }))
                            .into_any_element()
                    }),
            )
            // ZIP and 7z have their own AES-256, or age around them, as the others do.
            .when(on && !format.uses_age(), |d| {
                d.child(segments([
                    segment(
                        "own-aes",
                        format!("{} AES-256", format.ext().trim_start_matches('.').to_uppercase()),
                        !self.zip_age,
                    )
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.zip_age = false;
                        cx.notify();
                    })),
                    segment("whole-age", "age", self.zip_age).on_click(cx.listener(
                        |this, _, _, cx| {
                            this.zip_age = true;
                            cx.notify();
                        },
                    )),
                ]))
            })
            .when(on && age, |d| {
                d.child(segments([
                    segment("age-password", tr!("Password"), !self.to_keys).on_click(cx.listener(
                        |this, _, _, cx| {
                            this.to_keys = false;
                            cx.notify();
                        },
                    )),
                    segment("age-key", tr!("Public key"), self.to_keys).on_click(cx.listener(
                        |this, _, _, cx| {
                            this.to_keys = true;
                            cx.notify();
                        },
                    )),
                ]))
            })
            .when(on && !to_keys, |d| {
                d.child(
                    div()
                        .flex()
                        .items_center()
                        .h(ui(40.))
                        .px(ui(16.))
                        .rounded(ui(14.))
                        .bg(rgb(c.surface))
                        .text_size(ui(16.))
                        .child(div().flex_1().min_w_0().child(self.password.clone()))
                        .child(
                            button("suggest-password", tr!("Suggest"), Fill::Outline, Size::Medium)
                                .on_click(cx.listener(|this, _, _, cx| this.suggest(cx))),
                        ),
                )
                .when(self.passphrase_made, |d| {
                    d.child(note(Tone::Warn).child(tr!(
                        "Keep this somewhere safe: without it, this archive can never be opened."
                    )))
                })
                .when_some(strength, |d, s| d.child(strength_bar(s)))
                .when(typed.is_empty() && self.checked, |d| {
                    d.child(note(Tone::Warn).child(tr!("Type a password, or turn this off.")))
                })
                .when(format == Format::SevenZip && !age, |d| {
                    d.child(
                        check_row("hide-names", self.hide_names, tr!("Hide file names too")).on_click(
                            cx.listener(|this, _, _, cx| {
                                this.hide_names = !this.hide_names;
                                cx.notify();
                            }),
                        ),
                    )
                })
                .when(format == Format::Zip && !age, |d| {
                    d.child(note(Tone::Warn).child(
                        tr!("With a password, the built-in Windows and macOS tools can’t open this ZIP. \
                         The recipient needs an archiver app. File names also stay visible; \
                         pick .7z to hide them."),
                    ))
                })
            })
            .when(to_keys, |d| d.children(self.keys_part(cx)))
            .when_some(refused, |d, why| d.child(note(Tone::Warn).child(why)))
            .when(on && age, |d| {
                d.child(note(Tone::Warn).child(tr!(
                    "Saved as {name}. The names inside are encrypted too. Windows and macOS may need the age tool or an archiver app to open it.",
                    name = saved_as
                )))
            })
    }

    /// What is asked for encrypting to keys.
    fn keys_part(&self, cx: &mut Context<Self>) -> Vec<gpui::AnyElement> {
        let c = colors();
        let mut part = Vec::new();
        match &self.keys_file {
            Some(file) => {
                let said = tr_n!(
                    file.keys.len() as u64,
                    "{n} key from {name}",
                    "{n} keys from {name}",
                    name = file.name.clone()
                );
                part.push(
                    div()
                        .flex()
                        .items_center()
                        .gap(ui(8.))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .truncate()
                                .text_size(ui(14.))
                                .child(said),
                        )
                        .child(
                            button("keys-clear", tr!("Remove"), Fill::Outline, Size::Medium)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.keys_file = None;
                                    cx.notify();
                                })),
                        )
                        .into_any_element(),
                );
                if file.left_out > 0 {
                    part.push(
                        note(Tone::Warn)
                            .child(tr_n!(
                                file.left_out as u64,
                                "{n} key in it was left out, of a kind age does not encrypt to.",
                                "{n} keys in it were left out, of a kind age does not encrypt to."
                            ))
                            .into_any_element(),
                    );
                }
            }
            None => {
                part.push(
                    div()
                        .flex()
                        .items_center()
                        .gap(ui(8.))
                        .child(
                            div()
                                .flex()
                                .flex_1()
                                .min_w_0()
                                .items_center()
                                .h(ui(40.))
                                .px(ui(16.))
                                .rounded(ui(14.))
                                .bg(rgb(c.surface))
                                .text_size(ui(14.))
                                .child(self.key.clone()),
                        )
                        .child(
                            button(
                                "keys-file",
                                tr!("From a file…"),
                                Fill::Outline,
                                Size::Medium,
                            )
                            .on_click(cx.listener(|this, _, _, cx| this.pick_keys_file(cx))),
                        )
                        .into_any_element(),
                );
                let typed = self.key.read(cx).text().trim().to_owned();
                let problem = if typed.is_empty() {
                    self.checked
                        .then(|| tr!("Type a public key, or choose a file of them."))
                } else {
                    packwing_core::age_key_problem(&typed)
                };
                if let Some(problem) = problem {
                    part.push(note(Tone::Warn).child(problem).into_any_element());
                }
            }
        }
        if let Some(why) = &self.keys_file_problem {
            part.push(note(Tone::Warn).child(why.clone()).into_any_element());
        }
        if self
            .age_keys(cx)
            .iter()
            .any(|key| packwing_core::age_key_is_post_quantum(key))
        {
            part.push(
                note(Tone::Warn)
                    .child(tr!(
                        "A post-quantum key: to open this, the age tool must be version 1.3 or newer."
                    ))
                    .into_any_element(),
            );
        }
        part.push(
            note(Tone::Warn)
                .child(tr!(
                    "Only the private key that goes with it opens this, and nothing can if that key is lost."
                ))
                .into_any_element(),
        );
        part
    }
}

/// How strong the password is: a bar and a word.
fn strength_bar(s: Strength) -> impl IntoElement {
    let c = colors();
    let (part, color, word) = match s {
        Strength::Weak => (0.33, c.bad_text, tr!("Weak")),
        Strength::Fair => (0.66, c.warn_text, tr!("Fair")),
        Strength::Strong => (1., c.accent, tr!("Strong")),
    };
    div()
        .flex()
        .items_center()
        .gap(ui(10.))
        .text_size(ui(13.))
        .child(
            div()
                .flex_1()
                .h(ui(6.))
                .rounded_full()
                .bg(rgb(c.field))
                .child(
                    div()
                        .h_full()
                        .w(relative(part))
                        .rounded_full()
                        .bg(rgb(color)),
                ),
        )
        .child(word)
}

impl Render for CreateDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(picking) = self.picking.take() {
            let shown = self.storages_shown(&picking, cx);
            self.picking = Some(picking);
            return shown;
        }
        let s = &self.selection;
        let detail = if s.paths.is_empty() {
            tr!("Choose what to put in it").to_owned()
        } else if self._counting.is_some() && s.files == 0 {
            tr_n!(
                s.paths.len() as u64,
                "{count} item",
                "{count} items",
                count = count(s.paths.len() as u64)
            )
        } else {
            format!(
                "{} · {}",
                tr_n!(
                    s.files,
                    "{count} file",
                    "{count} files",
                    count = count(s.files)
                ),
                size(s.bytes)
            )
        };
        let close = icon_button("close", "x", 40.)
            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close)));
        super::card()
            .id("create-dialog")
            .drag_over::<ExternalPaths>(|style, _, _, _| style.bg(rgb(colors().tonal)))
            .on_drop(cx.listener(|this, dropped: &ExternalPaths, _, cx| {
                if !this.converting {
                    this.add(dropped.paths().to_vec(), cx)
                }
            }))
            .p(ui(0.))
            .gap(ui(0.))
            .overflow_hidden()
            .child(div().px(ui(28.)).pt(ui(24.)).pb(ui(8.)).child(header(
                "logo",
                tonal_tint(),
                if self.converting {
                    tr!("Convert")
                } else {
                    tr!("New archive")
                },
                detail,
                Some(close),
            )))
            .child(
                div()
                    .id("create-body")
                    .track_scroll(&self.body_scroll)
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .px(ui(28.))
                    .pt(ui(12.))
                    .pb(ui(8.))
                    .flex()
                    .flex_col()
                    .gap(ui(14.))
                    .child(self.contents(cx))
                    .child(self.name_field(cx))
                    .child(self.advanced_toggle(cx))
                    .when(self.advanced, |d| {
                        d.child(self.compression(cx)).child(self.protection(cx))
                    })
                    .children(self.busy.clone().map(|why| {
                        note(Tone::Warn)
                            .debug_selector(|| "busy".to_owned())
                            .child(why)
                    })),
            )
            .child(
                footer()
                    .justify_end()
                    .px(ui(28.))
                    .pt(ui(16.))
                    .pb(ui(24.))
                    .child(
                        button("cancel", tr!("Cancel"), Fill::Text, Size::Medium)
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close))),
                    )
                    .child(
                        button(
                            "create",
                            if self.converting {
                                tr!("Convert")
                            } else {
                                tr!("Create")
                            },
                            Fill::Accent,
                            Size::Medium,
                        )
                        .px(ui(28.))
                        .on_click(cx.listener(|this, _, _, cx| this.start(cx))),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn longer_and_mixed_passwords_are_stronger() {
        assert_eq!(strength("abc"), Strength::Weak);
        assert_eq!(strength("abcdefgh"), Strength::Fair);
        assert_eq!(strength("Abcdefgh12!x"), Strength::Strong);
        assert_eq!(strength("correct horse battery"), Strength::Strong);
    }

    #[test]
    fn a_name_stays_in_its_folder() {
        assert!(name_fits("Báo cáo quý 3"));
        assert!(name_fits(".hidden"));
        for bad in ["", ".", "..", "a/b", "../a", "a\\b"] {
            assert!(!name_fits(bad), "{bad:?}");
        }
    }

    #[test]
    fn every_format_but_zip_says_where_it_may_not_open() {
        for f in Format::WITH_ONE_FILE {
            assert_eq!(f.caveat().is_none(), f == Format::Zip, "{f:?}");
        }
    }

    #[test]
    fn all_but_zip_and_7z_are_protected_with_age() {
        for f in Format::WITH_ONE_FILE {
            assert_eq!(
                f.uses_age(),
                !matches!(f, Format::Zip | Format::SevenZip),
                "{f:?}"
            );
        }
    }

    #[test]
    fn those_for_any_files_come_first_and_are_not_for_one_file() {
        assert_eq!(Format::WITH_ONE_FILE[..Format::ALL.len()], Format::ALL);
        for (i, f) in Format::WITH_ONE_FILE.into_iter().enumerate() {
            assert_eq!(f.one_file(), i >= Format::ALL.len(), "{f:?}");
        }
    }

    #[test]
    fn each_option_is_its_format() {
        for (i, f) in Format::WITH_ONE_FILE.into_iter().enumerate() {
            assert_eq!(Format::of_option(i), f);
        }
    }

    #[test]
    fn a_part_takes_a_size_of_a_megabyte_or_more() {
        let (mb, gb) = (UNITS[0].1, UNITS[1].1);
        assert_eq!(part_size("100", mb), Some(100_000_000));
        assert_eq!(part_size(" 1.5 ", gb), Some(1_500_000_000));
        assert_eq!(part_size("1,5", gb), Some(1_500_000_000));
        assert_eq!(part_size("1", mb), Some(1_000_000));
        for bad in ["", "0.5", "0", "-3", "abc", "NaN", "inf"] {
            assert_eq!(part_size(bad, mb), None, "{bad:?}");
        }
    }
}
