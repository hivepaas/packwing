//! The main window: the menu down its left side and the page chosen in it,
//! the first run's welcome in their place, and a dialog over them all.

use packwing_i18n::{tr, tr_n};
use std::path::{Path, PathBuf};
use std::time::Duration;

use gpui::{
    AnyView, Context, Entity, EventEmitter, ExternalPaths, FocusHandle, Focusable, FontWeight,
    KeyDownEvent, PathPromptOptions, Render, Subscription, WeakEntity, Window, WindowAppearance,
    div, prelude::*, rgb, rgba,
};
use packwing_core::{Asked, Keys, NewArchive, Source};

use crate::chrome;
use crate::colors::{self, colors};
#[cfg(all(test, feature = "test-hooks"))]
use crate::demo;
use crate::dialogs::{
    AddDialog, AddStorageDialog, ChecksumDialog, CreateDialog, Creating, DeleteDialog, DialogEvent,
    EditDialog, ExtractDialog, Job, OpenFileDialog, OpenStoredDialog, PasswordDialog,
    PasswordEvent, ProgressDialog, QuitDialog, RemoveStorageDialog, Saving, SignDialog,
    SplitDialog, State, TransferDialog, Verb, ViewDialog, asked_for,
};
use crate::explorer::ExplorerEvent;
use crate::model::{Archive, Kind, Selection, stem};
use crate::pages::archive::{ArchiveEvent, ArchivePage, Shown};
use crate::pages::browse::BrowsePage;
use crate::pages::jobs::{JobsEvent, JobsPage, Summary};
use crate::pages::onboarding::{Done, Onboarding};
use crate::pages::recent::{RecentEvent, RecentPage, Row};
use crate::pages::settings::{Appearance, AppearanceChosen, SettingsPage};
use crate::pages::storages::{AddStorageAsked, EditStorageAsked, RemoveStorageAsked, StoragesPage};
use crate::placement::{self, WindowPlace};
use crate::prefs::{self, TrustedKey};
use crate::right_click::{Action, Request, RightClickMenu};
use crate::text_size::ui;
use crate::updates::{Badge, Updates};
use crate::widgets::{accent_tint, icon, logo, tile};

/// The pages the menu leads to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Archive,
    Storages,
    Browse,
    /// What runs in the background: in the menu only while there is some.
    Jobs,
    Settings,
}

/// An archive that needs a passphrase or a password: what is asked, and
/// what to do again with it.
#[derive(Clone)]
struct Ask {
    asked: Asked,
    then: Then,
}

/// What a passphrase or password typed is for.
#[derive(Clone)]
enum Then {
    /// Reading the archive at `source`, read with `keys` so far; with
    /// `then_extract`, Extract opens over it once it is read.
    Read {
        source: Source,
        keys: Keys,
        then_extract: bool,
    },
    /// Running a job again: an item of the right-click menu, which reads
    /// no listing first.
    Run(Box<Job>),
}

impl Then {
    /// What it was tried with.
    fn keys(&self) -> &Keys {
        match self {
            Then::Read { keys, .. } => keys,
            Then::Run(job) => &job.keys,
        }
    }

    /// The archive's name, as the dialog shows it.
    fn name(&self) -> String {
        match self {
            Then::Read { source, .. } => source.name(),
            Then::Run(job) => job.name.to_string(),
        }
    }
}

/// The dialog open over the window.
struct Open {
    view: AnyView,
    /// Escape closes it: not a job's progress, which has its own buttons.
    dismissable: bool,
    /// What it says: none for a job's dialog shown from the Jobs page, which keeps it.
    events: Option<Subscription>,
}

pub struct Workspace {
    focus: FocusHandle,
    page: Page,
    archive: Entity<ArchivePage>,
    storages: Entity<StoragesPage>,
    browse: Entity<BrowsePage>,
    recent: Entity<RecentPage>,
    settings: Entity<SettingsPage>,
    /// The jobs sent to run in the background.
    jobs: Entity<JobsPage>,
    /// Whether a newer version is out.
    updates: Entity<Updates>,
    /// packwing's items in the system's right-click menu.
    right_click: Entity<RightClickMenu>,
    /// The first run's welcome, shown instead of the pages.
    welcome: Option<(Entity<Onboarding>, Subscription)>,
    /// The file being read: a reading that ends after another began is
    /// not shown.
    reading: Option<Source>,
    /// The folder to show when what is being read is read: the one files
    /// were just added to.
    folder_after_reading: Option<Vec<String>>,
    dialog: Option<Open>,
    appearance: Appearance,
    /// The window's place, waiting to be kept, and which change it is.
    unsaved_place: Option<WindowPlace>,
    place_change: u64,
    /// The system's own appearance is dark.
    system_dark: bool,
    /// The keys trusted when signatures were last looked at: Settings may change them.
    trusted_seen: Vec<TrustedKey>,
    /// Jobs waiting for their turn, the first first.
    queue: Vec<WeakEntity<ProgressDialog>>,
    /// The window closes once the jobs have stopped.
    quitting: bool,
    /// Jobs go on in the background as they start, as Run in background sends one:
    /// a scenario's job ends before a click on it could.
    #[cfg(all(test, feature = "test-hooks"))]
    behind_at_once: bool,
    _subscriptions: Vec<Subscription>,
}

fn looks_dark(window: &Window) -> bool {
    matches!(
        window.appearance(),
        WindowAppearance::Dark | WindowAppearance::VibrantDark
    )
}

/// What files dropped on the window are for.
#[derive(Debug, PartialEq, Eq)]
enum Dropped {
    /// To add to the archive open.
    AddTo,
    /// An archive, to open.
    Open(PathBuf),
    /// To make a new archive of.
    New,
}

/// `takes_more`: an archive that can be added to is open, and they go in it, an
/// archive too, as the dialog says so before anything happens.
fn what_a_drop_means(paths: &[PathBuf], takes_more: bool) -> Dropped {
    if takes_more {
        return Dropped::AddTo;
    }
    match paths {
        [one]
            if !one.is_dir()
                && Kind::of(
                    &one.file_name().unwrap_or_default().to_string_lossy(),
                    false,
                ) == Kind::Archive =>
        {
            Dropped::Open(one.clone())
        }
        _ => Dropped::New,
    }
}

/// How long after the window last changed its place is kept: it changes all the
/// way through a drag.
const SAVE_BOUNDS: Duration = Duration::from_millis(400);

/// The size of the menu's labels, a little under body text's, in rems as it is: they
/// follow the system's size, or Settings' Text size.
const LABEL: f32 = 13.;
/// The width of the menu down the window's left side, padding and all: as much
/// narrower than for body text as its labels are smaller.
const MENU_W: f32 = 86.;
/// What the menu's labels have of it, between its padding.
const LABEL_ROOM: f32 = MENU_W - 8.;

/// How many jobs run at once: more wait for their turn, so that those that run
/// are not each slowed by the others on the same disk.
const MOST_RUNNING: usize = 2;

/// The mark on the menu's Jobs: how many run; else a dot for one that ended unseen,
/// in the warning's colors for one that waits for the user.
fn jobs_mark(s: Summary) -> Option<gpui::Div> {
    let c = colors();
    let at = || div().absolute().rounded_full();
    if s.waiting {
        return Some(
            at().top(ui(2.))
                .right(ui(8.))
                .size(ui(8.))
                .bg(rgb(c.warn_text)),
        );
    }
    if s.running > 0 {
        return Some(
            at().top(ui(-2.))
                .right(ui(4.))
                .min_w(ui(16.))
                .h(ui(16.))
                .px(ui(4.))
                .flex()
                .items_center()
                .justify_center()
                .bg(rgb(c.accent))
                .text_color(rgb(c.on_accent))
                .text_size(ui(11.))
                .font_weight(gpui::FontWeight::BOLD)
                .child(s.running.to_string()),
        );
    }
    s.unseen.then(|| {
        at().top(ui(2.))
            .right(ui(8.))
            .size(ui(8.))
            .bg(rgb(c.accent))
    })
}

/// The size the menu's `labels` are drawn at: [`LABEL`], unless the longest word of
/// one is wider than the menu, as in some languages; then as much smaller as it
/// takes for it to fit, for all of them alike.
fn label_size(labels: &[String], window: &Window) -> f32 {
    let rem = window.rem_size();
    let size = ui(LABEL).to_pixels(rem);
    let room = ui(LABEL_ROOM).to_pixels(rem);
    let font = gpui::Font {
        // The one chosen is drawn bolder: the widest it can be.
        weight: FontWeight::SEMIBOLD,
        ..crate::ui_font()
    };
    let widest = labels
        .iter()
        .flat_map(|l| l.split_whitespace())
        .map(|word| {
            let run = gpui::TextRun {
                len: word.len(),
                font: font.clone(),
                color: gpui::black(),
                background_color: None,
                underline: None,
                strikethrough: None,
            };
            window
                .text_system()
                .shape_line(word.to_owned().into(), size, &[run], None)
                .width
        })
        .fold(gpui::px(0.), gpui::Pixels::max);
    // A little to spare: laid out, a word just as wide as the room may break.
    let room = room * 0.96;
    if widest > room {
        LABEL * (room / widest)
    } else {
        LABEL
    }
}

impl Workspace {
    pub fn new(
        first_run: bool,
        right_click: Entity<RightClickMenu>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let kept = prefs::prefs(cx);
        let appearance = kept.appearance;
        let archive = cx.new(ArchivePage::new);
        let storages = cx.new(StoragesPage::new);
        let explorer = storages.read(cx).explorer().clone();
        let recent = cx.new(|cx| RecentPage::new(kept.recent.iter().map(Row::of).collect(), cx));
        let browse = cx.new({
            let recent = recent.clone();
            move |cx| BrowsePage::new(recent, cx)
        });
        let browse_explorer = browse.read(cx).explorer().clone();
        let updates = cx.new(Updates::new);
        let settings =
            cx.new(|cx| SettingsPage::new(appearance, right_click.clone(), updates.clone(), cx));
        let jobs = cx.new(|_| JobsPage::new());
        let subscriptions = vec![
            cx.observe(&updates, |_, _, cx| cx.notify()),
            cx.subscribe_in(
                &jobs,
                window,
                |this, _, e: &JobsEvent, window, cx| match e {
                    JobsEvent::Show(view) => this.show_job(view.clone(), cx),
                    JobsEvent::Ask(job, asked, again) => {
                        let ask = Ask {
                            asked: *asked,
                            then: Then::Run(job.clone()),
                        };
                        this.ask(ask, again.clone(), window, cx);
                    }
                    JobsEvent::Changed => cx.notify(),
                },
            ),
            cx.subscribe_in(
                &archive,
                window,
                |this, _, e: &ArchiveEvent, window, cx| match e {
                    ArchiveEvent::Extract => this.open_extract(window, cx),
                    ArchiveEvent::AddFiles => this.add_files(window, cx),
                    ArchiveEvent::Test => this.test_archive(window, cx),
                    ArchiveEvent::Sign => this.open_sign_archive(window, cx),
                    ArchiveEvent::Checksums => this.open_checksums_archive(window, cx),
                    ArchiveEvent::Convert => this.archive_file_then(Self::open_convert, window, cx),
                    ArchiveEvent::Split => this.archive_file_then(Self::open_split, window, cx),
                    ArchiveEvent::Join => this.archive_file_then(Self::start_join, window, cx),
                    ArchiveEvent::Open => this.choose_archive(window, cx),
                    ArchiveEvent::OpenStored => this.choose_stored_archive(window, cx),
                    ArchiveEvent::Delete => this.open_edit(false, window, cx),
                    ArchiveEvent::Rename => this.open_edit(true, window, cx),
                    ArchiveEvent::OpenEntry(entry) => this.open_entry(entry.clone(), window, cx),
                },
            ),
            cx.subscribe_in(&explorer, window, Self::explorer_event),
            cx.subscribe_in(&browse_explorer, window, Self::explorer_event),
            cx.subscribe_in(
                &storages,
                window,
                |this, page, _: &AddStorageAsked, window, cx| {
                    let page = page.clone();
                    let view = cx.new(|cx| AddStorageDialog::new(page, window, cx));
                    this.show(view, true, window, cx);
                },
            ),
            cx.subscribe_in(
                &storages,
                window,
                |this, page, e: &EditStorageAsked, window, cx| {
                    this.open_edit_storage(page.clone(), e.0, window, cx);
                },
            ),
            cx.subscribe_in(
                &storages,
                window,
                |this, page, e: &RemoveStorageAsked, window, cx| {
                    if let Some(dialog) = RemoveStorageDialog::new(page.clone(), e.0, cx) {
                        let view = cx.new(|_| dialog);
                        this.show(view, true, window, cx);
                    }
                },
            ),
            cx.subscribe_in(&recent, window, |this, _, e: &RecentEvent, window, cx| {
                this.on_recent(e, window, cx)
            }),
            cx.subscribe_in(
                &settings,
                window,
                |this, _, e: &AppearanceChosen, window, cx| this.set_appearance(e.0, window, cx),
            ),
            // The desktop may move the window's buttons to the other end.
            cx.observe_button_layout_changed(window, |_, _, cx| cx.notify()),
            cx.observe_window_appearance(window, |this, window, cx| {
                this.system_dark = looks_dark(window);
                this.apply_appearance(window, cx);
            }),
        ];
        let mut ws = Workspace {
            focus: cx.focus_handle(),
            page: Page::Archive,
            archive,
            storages,
            browse,
            recent,
            settings,
            jobs,
            updates,
            right_click,
            welcome: None,
            reading: None,
            folder_after_reading: None,
            unsaved_place: None,
            place_change: 0,
            dialog: None,
            appearance,
            system_dark: looks_dark(window),
            trusted_seen: kept.trusted_keys.clone(),
            queue: Vec::new(),
            quitting: false,
            #[cfg(all(test, feature = "test-hooks"))]
            behind_at_once: false,
            _subscriptions: subscriptions,
        };
        if first_run {
            ws.show_welcome(window, cx);
        }
        // Closing the window asks first while jobs run: from its own button, or the
        // system's.
        let me = cx.entity().downgrade();
        let may_close: chrome::MayClose = std::rc::Rc::new(move |window, cx| {
            me.update(cx, |ws, cx| ws.may_close(window, cx))
                .unwrap_or(true)
        });
        cx.set_global(chrome::Closing(may_close.clone()));
        window.on_window_should_close(cx, move |window, cx| may_close(window, cx));
        // A notification of a job that ended brings its page.
        let me = cx.entity().downgrade();
        cx.on_system_notification_response(move |_, cx| {
            cx.activate(true);
            let _ = me.update(cx, |ws, cx| ws.show_page(Page::Jobs, cx));
        });
        ws.apply_appearance(window, cx);
        ws
    }

    fn apply_appearance(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let dark = match self.appearance {
            Appearance::Light => false,
            Appearance::Dark => true,
            Appearance::System => self.system_dark,
        };
        colors::set_dark(dark);
        window.refresh();
        cx.notify();
    }

    /// Keeps the window's size and place for the next start: shortly after they
    /// change, and when the window closes.
    pub fn track_bounds(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.place_changed(window, cx);
        cx.observe_window_bounds(window, Self::place_changed)
            .detach();
        cx.on_release(|ws, cx| ws.save_place(cx)).detach();
        cx.on_app_quit(|ws, cx| {
            ws.save_place(cx);
            std::future::ready(())
        })
        .detach();
    }

    /// The looks for a newer version, from now on.
    pub fn start_updates(&mut self, cx: &mut Context<Self>) {
        self.updates.update(cx, |u, cx| u.start(cx));
    }

    #[cfg(all(test, feature = "test-hooks"))]
    pub fn settings(&self) -> &Entity<SettingsPage> {
        &self.settings
    }

    #[cfg(all(test, feature = "test-hooks"))]
    pub fn updates(&self) -> &Entity<Updates> {
        &self.updates
    }

    #[cfg(all(test, feature = "test-hooks"))]
    pub fn settings_section(&self, cx: &gpui::App) -> crate::pages::settings::Section {
        self.settings.read(cx).section()
    }

    #[cfg(all(test, feature = "test-hooks"))]
    pub fn page(&self) -> Page {
        self.page
    }

    /// About's Updates: where the title bar's badge leads.
    pub fn show_updates(&mut self, cx: &mut Context<Self>) {
        self.settings.update(cx, |s, cx| s.show_updates(cx));
        self.show_page(Page::Settings, cx);
    }

    /// The title bar's badge, if a newer version is out.
    fn update_badge(&self, cx: &mut Context<Self>) -> Option<gpui::Stateful<gpui::Div>> {
        let (said, tooltip) = match self.updates.read(cx).badge()? {
            Badge::Out(v) => (
                tr!("Update available"),
                tr!("packwing {version} is out", version = v.to_string()),
            ),
            Badge::Ready(v) => (
                tr!("Restart to update"),
                tr!("packwing {version} is ready", version = v.to_string()),
            ),
        };
        Some(
            chrome::update_badge(said, tooltip)
                .on_click(cx.listener(|this, _, _, cx| this.show_updates(cx))),
        )
    }

    fn place_changed(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.unsaved_place = Some(placement::place(window.window_bounds()));
        self.place_change += 1;
        let change = self.place_change;
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(SAVE_BOUNDS).await;
            let _ = this.update(cx, |ws, cx| {
                if ws.place_change == change {
                    ws.save_place(cx);
                }
            });
        })
        .detach();
    }

    fn save_place(&mut self, cx: &mut gpui::App) {
        if let Some(place) = self.unsaved_place.take() {
            prefs::change(cx, |p| p.window = Some(place));
        }
    }

    pub fn set_appearance(&mut self, a: Appearance, window: &mut Window, cx: &mut Context<Self>) {
        self.appearance = a;
        prefs::change(cx, |p| p.appearance = a);
        self.settings.update(cx, |s, cx| s.show_appearance(a, cx));
        self.apply_appearance(window, cx);
    }

    pub fn show_page(&mut self, page: Page, cx: &mut Context<Self>) {
        self.page = page;
        self.jobs
            .update(cx, |j, cx| j.set_shown(page == Page::Jobs, cx));
        let trusted = prefs::prefs(cx).trusted_keys;
        if trusted != self.trusted_seen {
            self.trusted_seen = trusted;
            self.look_at_signatures_again(cx);
        }
        cx.notify();
    }

    /// What was said of signatures may not hold: a file was signed, or the keys
    /// trusted changed.
    fn look_at_signatures_again(&mut self, cx: &mut Context<Self>) {
        self.archive.update(cx, |p, cx| p.look_at_signature(cx));
        let storages = self.storages.read(cx).explorer().clone();
        let browse = self.browse.read(cx).explorer().clone();
        for explorer in [storages, browse] {
            explorer.update(cx, |e, cx| e.forget_signature(cx));
        }
    }

    /// Asks how to sign the file `name` of the storage shown, for tests.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn open_sign_of(
        &mut self,
        name: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<SignDialog> {
        let explorer = self.storages.read(cx).explorer().read(cx);
        let storage = explorer
            .place()
            .expect("a storage is shown")
            .storage
            .clone();
        let path = explorer.path_of_name(name);
        let view = cx.new(|cx| SignDialog::new(storage, path, window, cx));
        self.show(view.clone(), true, window, cx);
        view
    }

    /// Shows the checksums of the file `name` of the storage shown, for tests.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn open_checksums_of(
        &mut self,
        name: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<ChecksumDialog> {
        let explorer = self.storages.read(cx).explorer().read(cx);
        let storage = explorer
            .place()
            .expect("a storage is shown")
            .storage
            .clone();
        let path = explorer.path_of_name(name);
        let view = cx.new(|cx| ChecksumDialog::new(storage, path, cx));
        self.show(view.clone(), true, window, cx);
        view
    }

    #[cfg(all(test, feature = "test-hooks"))]
    pub fn refresh_explorers_for_test(&mut self, cx: &mut Context<Self>) {
        self.refresh_explorers(cx);
    }

    /// Asks how to sign the file of the archive open.
    fn open_sign_archive(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let place = self
            .archive
            .read(cx)
            .archive()
            .and_then(|a| a.source.as_ref())
            .and_then(|s| s.place());
        if let Some((storage, path)) = place {
            let view = cx.new(|cx| SignDialog::new(storage, path, window, cx));
            self.show(view, true, window, cx);
        }
    }

    /// Shows the checksums of the file of the archive open.
    /// Split into parts…, for the file at `path`.
    fn open_split(&mut self, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        let bytes = std::fs::metadata(&path)
            .map(|m| m.len())
            .unwrap_or_default();
        let folder = path
            .parent()
            .map_or_else(String::new, |p| p.display().to_string());
        let view = cx.new(|cx| SplitDialog::new(path, bytes, folder, cx));
        self.show(view, true, window, cx);
    }

    /// Joins the parts that the one at `path` is of, beside them.
    fn start_join(&mut self, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        let named = |p: &Path| {
            p.file_name()
                .map_or_else(String::new, |n| n.to_string_lossy().into_owned())
        };
        let folder = path
            .parent()
            .map_or_else(String::new, |p| p.display().to_string());
        let whole = named(&path.with_extension(""));
        let mut job = Job::shown(Verb::Join, whole.into(), Some(folder.into()), 0, 0);
        job.join = Some(path);
        self.open_progress(job, window, cx);
    }

    /// Convert…, for the archive at `path`.
    fn open_convert(&mut self, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        let view = cx.new(|cx| CreateDialog::converting(path, cx));
        self.show(view, true, window, cx);
    }

    /// One of these, for the archive open, if its file is on this computer.
    fn archive_file_then(
        &mut self,
        then: fn(&mut Self, PathBuf, &mut Window, &mut Context<Self>),
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(path) = self.archive.read(cx).local_file() {
            then(self, path, window, cx);
        }
    }

    fn open_checksums_archive(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let place = self
            .archive
            .read(cx)
            .archive()
            .and_then(|a| a.source.as_ref())
            .and_then(|s| s.place());
        if let Some((storage, path)) = place {
            let view = cx.new(|cx| ChecksumDialog::new(storage, path, cx));
            self.show(view, true, window, cx);
        }
    }

    #[cfg(all(test, feature = "test-hooks"))]
    pub fn show_settings(
        &mut self,
        section: crate::pages::settings::Section,
        cx: &mut Context<Self>,
    ) {
        self.settings
            .update(cx, |s, cx| s.show_section(section, cx));
        self.show_page(Page::Settings, cx);
    }

    #[cfg(all(test, feature = "test-hooks"))]
    pub fn right_click(&self) -> &Entity<RightClickMenu> {
        &self.right_click
    }

    pub fn show_welcome(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let welcome = cx.new(|_| Onboarding::new());
        let done = cx.subscribe_in(&welcome, window, |this, _, done: &Done, _, cx| {
            this.right_click
                .update(cx, |m, cx| m.show(done.right_click, cx));
            this.welcome = None;
            cx.notify();
        });
        self.welcome = Some((welcome, done));
        cx.notify();
    }

    /// What an explorer of a page asked for.
    fn explorer_event(
        this: &mut Self,
        explorer: &Entity<crate::explorer::Explorer>,
        e: &ExplorerEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match e {
            ExplorerEvent::OpenArchive(source) => {
                this.read_archive(source.clone(), Keys::default(), false, window, cx)
            }
            ExplorerEvent::Extract(source) => {
                this.read_archive(source.clone(), Keys::default(), true, window, cx)
            }
            ExplorerEvent::Compress(paths) => this.open_create(paths.clone(), window, cx),
            ExplorerEvent::View(item) => {
                let (item, explorer) = (item.clone(), explorer.clone());
                let view = cx.new(|cx| ViewDialog::new(item, explorer, cx));
                this.show(view, true, window, cx);
            }
            ExplorerEvent::Transfer { moving } => {
                this.open_transfer(explorer, *moving, window, cx);
            }
            ExplorerEvent::Sign(name) => {
                let shown = explorer.read(cx);
                if let Some(place) = shown.place() {
                    let (storage, path) = (place.storage.clone(), shown.path_of_name(name));
                    let view = cx.new(|cx| SignDialog::new(storage, path, window, cx));
                    this.show(view, true, window, cx);
                }
            }
            ExplorerEvent::Checksums(name) => {
                let shown = explorer.read(cx);
                if let Some(place) = shown.place() {
                    let (storage, path) = (place.storage.clone(), shown.path_of_name(name));
                    let view = cx.new(|cx| ChecksumDialog::new(storage, path, cx));
                    this.show(view, true, window, cx);
                }
            }
            ExplorerEvent::Split(name) => {
                if let Some(path) = explorer.read(cx).local_path_of(name) {
                    this.open_split(path, window, cx);
                }
            }
            ExplorerEvent::Join(name) => {
                if let Some(path) = explorer.read(cx).local_path_of(name) {
                    this.start_join(path, window, cx);
                }
            }
            ExplorerEvent::Convert(name) => {
                if let Some(path) = explorer.read(cx).local_path_of(name) {
                    this.open_convert(path, window, cx);
                }
            }
            ExplorerEvent::AskDelete(names) => {
                let (names, explorer) = (names.clone(), explorer.clone());
                let view = cx.new(|cx| DeleteDialog::new(names, explorer, cx));
                this.show(view, true, window, cx);
            }
        }
    }

    /// Reads the folders shown again, as a job may have changed them.
    fn refresh_explorers(&mut self, cx: &mut Context<Self>) {
        let storages = self.storages.read(cx).explorer().clone();
        let browse = self.browse.read(cx).explorer().clone();
        for explorer in [storages, browse] {
            explorer.update(cx, |e, cx| e.refresh(cx));
        }
    }

    /// The explorer of the page shown; of Storages on any page without one.
    fn shown_explorer(&self, cx: &gpui::App) -> Entity<crate::explorer::Explorer> {
        match self.page {
            Page::Browse => self.browse.read(cx).explorer().clone(),
            _ => self.storages.read(cx).explorer().clone(),
        }
    }

    /// Opens `view` over the window, until it says it is done.
    fn show<V: Render + EventEmitter<DialogEvent>>(
        &mut self,
        view: Entity<V>,
        dismissable: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let events = cx.subscribe_in(
            &view,
            window,
            |this, _, e: &DialogEvent, window, cx| match e {
                DialogEvent::Close => this.close_dialog(cx),
                DialogEvent::Start(job) => {
                    // Said in the dialog, which stays for another name or folder.
                    let refused = this.jobs.read(cx).clash(job, cx).and_then(|why| {
                        let view = this.dialog.as_ref()?.view.clone();
                        if let Ok(d) = view.clone().downcast::<CreateDialog>() {
                            d.update(cx, |d, cx| d.refuse(why, cx));
                        } else if let Ok(d) = view.downcast::<ExtractDialog>() {
                            d.update(cx, |d, cx| d.refuse(why, cx));
                        } else {
                            return None;
                        }
                        Some(())
                    });
                    if refused.is_none() {
                        this.open_progress(job.clone(), window, cx);
                    }
                }
                DialogEvent::Transferred(went) => {
                    this.close_dialog(cx);
                    let mut said = Vec::new();
                    if went.skipped > 0 {
                        said.push(tr_n!(
                            went.skipped,
                            "{n} file was skipped, as its name was taken.",
                            "{n} files were skipped, as their names were taken."
                        ));
                    }
                    if went.links_left > 0 {
                        said.push(tr_n!(
                            went.links_left,
                            "{n} link stayed where it was.",
                            "{n} links stayed where they were."
                        ));
                    }
                    let said = (!said.is_empty()).then(|| said.join(" "));
                    this.refresh_explorers(cx);
                    this.shown_explorer(cx).update(cx, |e, cx| e.say(said, cx));
                }
                DialogEvent::Open(source) => {
                    this.close_dialog(cx);
                    this.read_archive(source.clone(), Keys::default(), false, window, cx);
                }
                DialogEvent::ShowLicense => {
                    this.close_dialog(cx);
                    this.settings.update(cx, |s, cx| {
                        s.show_section(crate::pages::settings::Section::License, cx)
                    });
                    this.show_page(Page::Settings, cx);
                }
                DialogEvent::Saved => this.refresh_explorers(cx),
                DialogEvent::Quit => this.stop_and_quit(window, cx),
                // Only a job's dialog says these, and they are heard in job_event.
                DialogEvent::Behind | DialogEvent::Ended | DialogEvent::Attention => {}
                DialogEvent::Signed => {
                    this.close_dialog(cx);
                    this.trusted_seen = prefs::prefs(cx).trusted_keys;
                    this.look_at_signatures_again(cx);
                    this.refresh_explorers(cx);
                }
                DialogEvent::Stored {
                    place,
                    folder,
                    show,
                } => {
                    this.close_dialog(cx);
                    if *show {
                        this.show_page(Page::Storages, cx);
                        this.storages
                            .update(cx, |p, cx| p.show(place, folder.clone(), cx));
                    }
                    this.refresh_explorers(cx);
                }
                DialogEvent::Added(path, folder, keys) => {
                    this.close_dialog(cx);
                    this.folder_after_reading = Some(folder.clone());
                    this.read_archive(path.into(), keys.clone(), false, window, cx);
                }
                DialogEvent::Ask(job, asked, again) => {
                    let ask = Ask {
                        asked: *asked,
                        then: Then::Run(Box::new(job.clone())),
                    };
                    this.ask(ask, again.clone(), window, cx);
                }
            },
        );
        self.dialog = Some(Open {
            view: view.into(),
            dismissable,
            events: Some(events),
        });
        cx.notify();
    }

    fn close_dialog(&mut self, cx: &mut Context<Self>) {
        self.dialog = None;
        cx.notify();
    }

    /// Asks for `asked`, as an archive that needs it does; with `again`,
    /// as after a wrong one.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn show_ask(
        &mut self,
        asked: Asked,
        again: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let ask = Ask {
            asked,
            then: Then::Read {
                source: PathBuf::from("/home/user/Downloads/Tài liệu dự án 2026.7z").into(),
                keys: Keys::default(),
                then_extract: false,
            },
        };
        self.ask(ask, again, window, cx);
    }

    #[cfg(all(test, feature = "test-hooks"))]
    pub fn show_archive(&mut self, shown: Shown, cx: &mut Context<Self>) {
        self.archive.update(cx, |p, cx| p.show(shown, cx));
    }

    /// The Storages page, browsing the folder at `path`.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn show_storage(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        self.storages.update(cx, |p, cx| p.add(path, cx));
        self.show_page(Page::Storages, cx);
    }

    /// Copies or moves what is chosen to the storage at `to`, as its dialog would.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn start_transfer_to(
        &mut self,
        moving: bool,
        to: &Path,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let page = self.storages.read(cx);
        let explorer = page.explorer().read(cx);
        let from = explorer.place().cloned().expect("a storage is shown");
        let t = packwing_core::Transfer {
            from: from.storage,
            from_folder: explorer.folder().to_vec(),
            names: explorer.selection().into_iter().map(|i| i.name).collect(),
            to: std::sync::Arc::new(packwing_core::LocalFolder::new(to)),
            to_folder: Vec::new(),
            moving,
            on_existing: packwing_core::OnExisting::Stop,
            keep_mark: prefs::prefs(cx).keep_mark,
        };
        let verb = if moving { Verb::Move } else { Verb::Copy };
        let mut job = Job::shown(verb, "test".into(), None, 0, 0);
        job.transfer = Some(crate::dialogs::Transferring(t, Default::default()));
        self.open_progress(job, window, cx);
    }

    /// Edit, for the storage at `at` in the list.
    fn open_edit_storage(
        &mut self,
        page: Entity<StoragesPage>,
        at: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let view = cx.new(|cx| AddStorageDialog::editing(page, at, window, cx));
        self.show(view, true, window, cx);
    }

    #[cfg(all(test, feature = "test-hooks"))]
    pub fn edit_storage(&mut self, at: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.open_edit_storage(self.storages.clone(), at, window, cx);
    }

    /// Add storage, as a build with more kinds than a folder asks it.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn open_add_storage(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let page = self.storages.clone();
        let view = cx.new(|cx| AddStorageDialog::new(page, window, cx));
        self.show(view, true, window, cx);
    }

    #[cfg(all(test, feature = "test-hooks"))]
    pub fn page_shown(&self) -> Page {
        self.page
    }

    #[cfg(all(test, feature = "test-hooks"))]
    pub fn storages_page(&self) -> Entity<StoragesPage> {
        self.storages.clone()
    }

    #[cfg(all(test, feature = "test-hooks"))]
    pub fn explorer(&self, cx: &gpui::App) -> Entity<crate::explorer::Explorer> {
        self.storages.read(cx).explorer().clone()
    }

    /// Shows the sample archive, as the screenshots draw it.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn show_sample(&mut self, cx: &mut Context<Self>) {
        self.reading = None;
        self.archive.update(cx, |p, cx| {
            p.show(Shown::Open(Box::new(demo::project())), cx)
        });
    }

    /// Reads the archive at `path` in the background and shows it; with
    /// `then_extract`, the Extract dialog opens over it once it is read.
    pub fn open_archive(
        &mut self,
        path: PathBuf,
        then_extract: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.read_archive(path.into(), Keys::default(), then_extract, window, cx);
    }

    /// Reads the archive at `source` with `keys`. One that turns out to need
    /// a passphrase or a password asks for it, and is read again with it;
    /// a wrong one asks again, saying so.
    fn read_archive(
        &mut self,
        source: Source,
        keys: Keys,
        then_extract: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let name = source.name();
        self.archive
            .update(cx, |p, cx| p.show(Shown::Opening(name.clone()), cx));
        self.show_page(Page::Archive, cx);
        self.reading = Some(source.clone());
        let (read, with) = (source.clone(), keys.clone());
        let task = cx.background_executor().spawn(async move {
            let listing = packwing_core::list(&read, &with);
            let marked = packwing_core::came_from_internet(&read);
            listing.map(|l| (l, marked))
        });
        cx.spawn_in(window, async move |this, cx| {
            let result = task.await;
            let _ = this.update_in(cx, |this, window, cx| {
                if this.reading.as_ref() != Some(&source) {
                    return;
                }
                this.reading = None;
                let shown = match result {
                    Ok((listing, marked)) => {
                        // Only what is on this computer: Recent opens it by its path.
                        if let Some(path) = source.path() {
                            prefs::change(cx, |p| p.opened(path));
                        }
                        let rows = prefs::prefs(cx).recent.iter().map(Row::of).collect();
                        this.recent.update(cx, |r, cx| r.show(rows, cx));
                        Shown::Open(Box::new(Archive::read(source, listing, keys, marked)))
                    }
                    Err(e) => match asked_for(&e, &keys) {
                        Some((asked, again)) => {
                            let then = Then::Read {
                                source,
                                keys,
                                then_extract,
                            };
                            let ask = Ask { asked, then };
                            this.ask(ask, again.then(|| e.to_string()), window, cx);
                            return;
                        }
                        None => Shown::Failed {
                            name,
                            why: e.to_string(),
                        },
                    },
                };
                let opened = matches!(shown, Shown::Open(_));
                let folder = this.folder_after_reading.take();
                this.archive.update(cx, |p, cx| {
                    p.show(shown, cx);
                    if let Some(folder) = folder {
                        p.enter(folder, cx);
                    }
                });
                if opened && then_extract {
                    this.open_extract(window, cx);
                }
            });
        })
        .detach();
    }

    /// Asks for what `ask` needs; `again` says why the one typed before
    /// did not open it. Typed, what was stopped goes again with it;
    /// cancelled, a reading's page says what it needs, and a job stops.
    fn ask(
        &mut self,
        ask: Ask,
        again: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let name = ask.then.name();
        let view = cx.new(|cx| PasswordDialog::new(ask.asked, name.clone(), again, window, cx));
        let events = cx.subscribe_in(
            &view,
            window,
            move |this, _, e: &PasswordEvent, window, cx| {
                this.close_dialog(cx);
                match (e, &ask.then) {
                    (PasswordEvent::Typed(secret), then) => {
                        let keys = then.keys().with(ask.asked, secret.clone());
                        match then {
                            Then::Read {
                                source,
                                then_extract,
                                ..
                            } => this.read_archive(source.clone(), keys, *then_extract, window, cx),
                            Then::Run(job) => {
                                let mut job = job.as_ref().clone();
                                job.keys = keys;
                                this.open_progress(job, window, cx);
                            }
                        }
                    }
                    (PasswordEvent::Cancelled, Then::Read { .. }) => {
                        let why = match ask.asked {
                            Asked::Passphrase => tr!("This file is encrypted with a passphrase."),
                            Asked::Password => tr!("This archive needs its password."),
                            Asked::KeyFile => tr!("This file is encrypted to a key."),
                            Asked::KeyPassphrase => tr!("This key file needs its passphrase."),
                        };
                        let shown = Shown::Failed {
                            name: name.clone(),
                            why,
                        };
                        this.archive.update(cx, |p, cx| p.show(shown, cx));
                    }
                    (PasswordEvent::Cancelled, Then::Run(_)) => {}
                }
                // The window takes the keys again, for Escape.
                window.focus(&this.focus, cx);
            },
        );
        self.dialog = Some(Open {
            view: view.into(),
            // Escape goes to the dialog, which gives up as Cancel does.
            dismissable: false,
            events: Some(events),
        });
        cx.notify();
    }

    /// Asks which archive to open, and opens it.
    fn choose_archive(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let picked = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some(tr!("Open").into()),
        });
        cx.spawn_in(window, async move |this, cx| {
            if let Ok(Ok(Some(paths))) = picked.await
                && let Some(path) = paths.into_iter().next()
            {
                let _ = this.update_in(cx, |this, window, cx| {
                    this.open_archive(path, false, window, cx)
                });
            }
        })
        .detach();
    }

    /// Asks which archive on a storage to open, and opens it.
    pub fn choose_stored_archive(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let (places, chosen) = self.storages.read(cx).places(cx);
        let storages = self.storages.clone();
        let view = cx.new(|cx| OpenStoredDialog::new(places, chosen, storages, cx));
        self.show(view, true, window, cx);
    }

    pub fn open_extract(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(archive) = self.archive.read(cx).archive().cloned() else {
            return;
        };
        let picked = self.archive.read(cx).picked();
        let view = cx.new(|cx| ExtractDialog::new(archive, picked, cx));
        self.show(view, true, window, cx);
    }

    /// Asks where to copy or move what is chosen in the storage shown.
    fn open_transfer(
        &mut self,
        explorer: &Entity<crate::explorer::Explorer>,
        moving: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let shown = explorer.read(cx);
        let Some(from) = shown.place().cloned() else {
            return;
        };
        let (folder, names): (Vec<String>, Vec<String>) = (
            shown.folder().to_vec(),
            shown.selection().into_iter().map(|i| i.name).collect(),
        );
        let (mut places, mut chosen) = self.storages.read(cx).places(cx);
        // Browse is not a storage: it starts from itself, with the storages after it.
        if explorer == self.browse.read(cx).explorer() {
            places.insert(0, from.clone());
            chosen = 0;
        }
        let storages = self.storages.clone();
        let view = cx.new(|cx| {
            TransferDialog::new(from, folder, names, moving, (places, chosen), storages, cx)
        });
        self.show(view, true, window, cx);
    }

    /// Asks how to change what is selected in the archive open.
    pub fn open_edit(&mut self, rename: bool, window: &mut Window, cx: &mut Context<Self>) {
        let page = self.archive.read(cx);
        let picked = page.picked();
        let Some(a) = page.archive().cloned() else {
            return;
        };
        let Some(path) = a.path else {
            return;
        };
        let about = (path, a.name, a.folder, a.keys);
        let view = cx.new(|cx| EditDialog::new(about, picked, rename, window, cx));
        self.show(view, true, window, cx);
    }

    /// Opens a file of the archive open, a copy of it: asked first, as
    /// Settings says, and always for a program.
    pub fn open_entry(&mut self, entry: Vec<String>, window: &mut Window, cx: &mut Context<Self>) {
        let Some(a) = self.archive.read(cx).archive().cloned() else {
            return;
        };
        let (Some(source), Some(name)) = (a.source, entry.last().cloned()) else {
            return;
        };
        let mut job = Job::shown(Verb::Open, a.name.into(), None, 1, 0);
        job.work = vec![(source, None)];
        job.keys = a.keys;
        job.picked = vec![entry];
        let program = Kind::of(&name, false) == Kind::Program;
        if program || prefs::prefs(cx).ask_before_open {
            let view = cx.new(|_| OpenFileDialog::new(job, name, program));
            self.show(view, true, window, cx);
        } else {
            self.open_progress(job, window, cx);
        }
    }

    /// Reads every file of the archive open through, its checks made.
    fn test_archive(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(a) = self.archive.read(cx).archive().cloned() else {
            return;
        };
        let mut job = Job::shown(Verb::Test, a.name.clone().into(), None, a.files, a.unpacked);
        if let Some(source) = a.source {
            job.work = vec![(source, None)];
            job.keys = a.keys;
        }
        self.open_progress(job, window, cx);
    }

    /// New archive, holding `paths` so far.
    pub fn open_create(
        &mut self,
        paths: Vec<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.open_create_holding(paths, window, cx);
    }

    /// [`Self::open_create`], for what is to be asked of the dialog.
    fn open_create_holding(
        &mut self,
        paths: Vec<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<CreateDialog> {
        let (places, chosen) = self.storages.read(cx).places(cx);
        let storages = self.storages.clone();
        let view = cx.new(|cx| {
            CreateDialog::new(Selection::default(), cx).with_places(places, chosen, storages, cx)
        });
        if !paths.is_empty() {
            view.update(cx, |d, cx| d.add(paths, cx));
        }
        self.show(view.clone(), true, window, cx);
        view
    }

    /// New archive holding `paths`, for tests.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn open_create_of(
        &mut self,
        paths: Vec<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<CreateDialog> {
        self.open_create_holding(paths, window, cx)
    }

    /// Add files, holding the screenshots' sample.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn open_add_sample(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let archive = self.archive.read(cx).archive().cloned();
        let dialog = archive.map(|a| AddDialog::sample(&a, cx));
        if let Some(dialog) = dialog {
            let view = cx.new(|_| dialog);
            self.show(view, true, window, cx);
        }
    }

    /// New archive, holding more than its list shows at once, the list open.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn open_create_many(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<CreateDialog> {
        let view = cx.new(|cx| CreateDialog::new(demo::many_chosen(), cx));
        view.update(cx, |d, cx| d.open_list(cx));
        self.show(view.clone(), true, window, cx);
        view
    }

    /// New archive, holding the screenshots' sample.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn open_create_sample(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<CreateDialog> {
        let view = cx.new(|cx| CreateDialog::new(demo::selection(), cx));
        self.show(view.clone(), true, window, cx);
        view
    }

    pub fn open_progress(
        &mut self,
        job: Job,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<ProgressDialog> {
        // A job that would write where one in the background does says why it did not
        // start, as one that failed; one more than may run at once waits its turn.
        let view = match self.jobs.read(cx).clash(&job, cx) {
            Some(why) => cx.new(|cx| ProgressDialog::refused(job, why, cx)),
            None if self.running(cx) >= MOST_RUNNING => {
                let view = cx.new(|cx| ProgressDialog::queued(job, cx));
                self.queue.push(view.downgrade());
                view
            }
            None => cx.new(|cx| ProgressDialog::new(job, cx)),
        };
        let events = cx.subscribe_in(&view, window, Self::job_event);
        self.dialog = Some(Open {
            view: view.clone().into(),
            dismissable: false,
            events: Some(events),
        });
        cx.notify();
        #[cfg(all(test, feature = "test-hooks"))]
        if self.behind_at_once && view.read(cx).can_go_behind() {
            view.update(cx, |_, cx| cx.emit(DialogEvent::Behind));
        }
        view
    }

    /// Sends each job that starts to the background at once.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn send_jobs_behind_for_test(&mut self) {
        self.behind_at_once = true;
    }

    /// The Jobs page with a job of each kind, as its picture shows it.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn show_sample_jobs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let samples = [
            (
                Verb::Extract,
                "Tài liệu dự án 2026.7z",
                "~/Downloads/Tài liệu dự án 2026",
                State::Running,
            ),
            (
                Verb::Extract,
                "Hợp đồng 2025.zip",
                "~/Documents/Hợp đồng 2025",
                State::Running,
            ),
            (Verb::Test, "photos-2026.7z", "", State::Queued),
            (
                Verb::Create,
                "site-2026-10.zip",
                "~/Backups",
                State::Done(Some(PathBuf::from("/home/me/Backups/site-2026-10.zip"))),
            ),
            (
                Verb::Copy,
                "photos.7z",
                "S3 · backups",
                State::Failed("The connection was reset.".to_owned()),
            ),
        ];
        for (i, (verb, name, to, state)) in samples.into_iter().enumerate() {
            let to = (!to.is_empty()).then(|| to.into());
            let job = Job::shown(verb, name.into(), to, 1_284, 412_000_000);
            let view = cx.new(|cx| ProgressDialog::new(job.clone(), cx));
            view.update(cx, |p, cx| {
                p.set_done(0.45, cx);
                p.set_state_for_test(state, cx);
            });
            let events = cx.subscribe_in(&view, window, Self::job_event);
            self.jobs
                .update(cx, |j, cx| j.add(view.clone(), events, cx));
            if i == 1 {
                let asked = (job, Asked::Password, None);
                self.jobs.update(cx, |j, cx| j.wait_for(&view, asked, cx));
            }
        }
        self.show_page(Page::Jobs, cx);
    }

    /// `job` in the background, never ending: what it writes stays taken.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn hold_job_for_test(&mut self, job: Job, window: &mut Window, cx: &mut Context<Self>) {
        let view = cx.new(|cx| ProgressDialog::held(job, cx));
        let events = cx.subscribe_in(&view, window, Self::job_event);
        self.jobs.update(cx, |j, cx| j.add(view, events, cx));
    }

    /// Asks whether to close the window, as its close button does.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn ask_to_close_for_test(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.may_close(window, cx);
    }

    /// Where each job of the Jobs page stands.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn job_states(&self, cx: &gpui::App) -> Vec<State> {
        self.jobs.read(cx).states(cx)
    }

    /// What the menu says of the jobs in the background.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn jobs_summary(&self, cx: &gpui::App) -> Summary {
        self.jobs.read(cx).summary()
    }

    /// The job whose dialog is shown, if it is not one of the Jobs page.
    fn shown_job(&self, cx: &gpui::App) -> Option<Entity<ProgressDialog>> {
        let view = self
            .dialog
            .as_ref()?
            .view
            .clone()
            .downcast::<ProgressDialog>()
            .ok()?;
        (!self.jobs.read(cx).holds(&view)).then_some(view)
    }

    /// How many jobs run now, not those waiting for their turn.
    fn running(&self, cx: &gpui::App) -> usize {
        let shown = self
            .shown_job(cx)
            .is_some_and(|v| v.read(cx).state() == State::Running);
        self.jobs.read(cx).running(cx) + usize::from(shown)
    }

    /// The jobs that run or wait for their turn.
    fn going(&self, cx: &gpui::App) -> Vec<Entity<ProgressDialog>> {
        let mut going = self.jobs.read(cx).going(cx);
        going.extend(
            self.shown_job(cx)
                .filter(|v| matches!(v.read(cx).state(), State::Running | State::Queued)),
        );
        going
    }

    /// Starts those waiting for their turn, as far as there is room.
    fn start_queued(&mut self, cx: &mut Context<Self>) {
        while self.running(cx) < MOST_RUNNING && !self.queue.is_empty() {
            let next = self.queue.remove(0);
            if let Some(view) = next.upgrade()
                && view.read(cx).state() == State::Queued
            {
                view.update(cx, |v, cx| v.begin(cx));
            }
        }
    }

    /// Whether the window may close: not while jobs run, until the user says.
    fn may_close(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let going = self.going(cx).len();
        if self.quitting || going == 0 {
            return true;
        }
        // A job shown goes on in the background, behind what is asked.
        if let Some(view) = self.shown_job(cx)
            && let Some(Open {
                events: Some(events),
                ..
            }) = self.dialog.take()
        {
            self.jobs.update(cx, |j, cx| j.add(view, events, cx));
        }
        let view = cx.new(|_| QuitDialog::new(going));
        self.show(view, true, window, cx);
        false
    }

    /// Stops every job, and closes the window once they have stopped.
    fn stop_and_quit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.quitting = true;
        self.close_dialog(cx);
        for view in self.going(cx) {
            view.update(cx, |v, cx| v.cancel(cx));
        }
        self.close_if_quitting(window, cx);
    }

    fn close_if_quitting(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.quitting && self.going(cx).is_empty() {
            window.remove_window();
        }
    }

    /// Tells the system a job in the background ended, when the window is not the one
    /// the user is in.
    fn tell_ended(
        &self,
        view: &Entity<ProgressDialog>,
        shown: bool,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        if shown || window.is_window_active() || self.quitting {
            return;
        }
        let waits = self.jobs.read(cx).waits(view);
        let job = view.read(cx);
        if job.state() == State::Cancelled {
            return;
        }
        cx.show_system_notification(gpui::SystemNotification {
            tag: format!("job-{}", view.entity_id()).into(),
            title: crate::pages::jobs::headline(job, waits).into(),
            body: job.detail().into(),
            actions: Vec::new(),
        });
    }

    /// Whether `view` is the dialog shown.
    fn shows(&self, view: &Entity<ProgressDialog>) -> bool {
        self.dialog
            .as_ref()
            .is_some_and(|d| d.view.entity_id() == view.entity_id())
    }

    /// Shows the dialog of a job of the Jobs page, over what is there if nothing is.
    fn show_job(&mut self, view: Entity<ProgressDialog>, cx: &mut Context<Self>) {
        if self.dialog.is_some() {
            return;
        }
        // Escape sends it back, as Run in background does.
        let dismissable = view.read(cx).can_go_behind();
        self.dialog = Some(Open {
            view: view.into(),
            dismissable,
            events: None,
        });
        cx.notify();
    }

    /// What a job said: as its dialog, if it is shown, else quietly, on the Jobs page.
    fn job_event(
        &mut self,
        view: &Entity<ProgressDialog>,
        e: &DialogEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let shown = self.shows(view);
        // A job over while its dialog is shown leaves with it.
        let over = |this: &mut Self, cx: &mut Context<Self>| {
            if shown {
                this.close_dialog(cx);
                this.jobs.update(cx, |j, cx| j.remove(view, cx));
            }
        };
        match e {
            DialogEvent::Attention if !shown => self.show_job(view.clone(), cx),
            DialogEvent::Behind => {
                if shown && let Some(open) = self.dialog.take() {
                    // One shown from the page is on it still.
                    if let Some(events) = open.events {
                        self.jobs
                            .update(cx, |j, cx| j.add(view.clone(), events, cx));
                    }
                    cx.notify();
                }
            }
            DialogEvent::Close => {
                over(self, cx);
                if view.read(cx).state() == State::Cancelled {
                    self.jobs.update(cx, |j, cx| j.remove(view, cx));
                }
            }
            DialogEvent::Transferred(went) => {
                over(self, cx);
                let mut said = Vec::new();
                if went.skipped > 0 {
                    said.push(tr_n!(
                        went.skipped,
                        "{n} file was skipped, as its name was taken.",
                        "{n} files were skipped, as their names were taken."
                    ));
                }
                if went.links_left > 0 {
                    said.push(tr_n!(
                        went.links_left,
                        "{n} link stayed where it was.",
                        "{n} links stayed where they were."
                    ));
                }
                let said = (!said.is_empty()).then(|| said.join(" "));
                self.refresh_explorers(cx);
                self.shown_explorer(cx).update(cx, |e, cx| e.say(said, cx));
            }
            DialogEvent::Signed => {
                over(self, cx);
                self.trusted_seen = prefs::prefs(cx).trusted_keys;
                self.look_at_signatures_again(cx);
                self.refresh_explorers(cx);
            }
            DialogEvent::Stored {
                place,
                folder,
                show,
            } => {
                // One that ends in the background does not take the user away.
                if *show && shown {
                    self.show_page(Page::Storages, cx);
                    self.storages
                        .update(cx, |p, cx| p.show(place, folder.clone(), cx));
                }
                over(self, cx);
                self.refresh_explorers(cx);
            }
            DialogEvent::Added(path, folder, keys) => {
                over(self, cx);
                self.folder_after_reading = Some(folder.clone());
                self.read_archive(path.into(), keys.clone(), false, window, cx);
            }
            DialogEvent::Ask(job, asked, again) => {
                if shown {
                    self.jobs.update(cx, |j, cx| j.remove(view, cx));
                    let ask = Ask {
                        asked: *asked,
                        then: Then::Run(Box::new(job.clone())),
                    };
                    self.ask(ask, again.clone(), window, cx);
                } else {
                    let asked = (job.clone(), *asked, again.clone());
                    self.jobs.update(cx, |j, cx| j.wait_for(view, asked, cx));
                }
            }
            DialogEvent::Ended => self.tell_ended(view, shown, window, cx),
            DialogEvent::Start(_)
            | DialogEvent::Open(_)
            | DialogEvent::ShowLicense
            | DialogEvent::Saved
            | DialogEvent::Attention
            | DialogEvent::Quit => {}
        }
        // What ended makes room for one waiting, or lets the window close.
        self.start_queued(cx);
        self.close_if_quitting(window, cx);
    }

    /// Does what an item of the right-click menu asked.
    pub fn start(&mut self, r: Request, window: &mut Window, cx: &mut Context<Self>) {
        let Some(first) = r.paths.first() else {
            return;
        };
        let name = |p: &Path| {
            p.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default()
        };
        let folder = first.parent().unwrap_or(Path::new(""));
        let one = r.paths.len() == 1;
        match r.action {
            Action::ExtractHere | Action::ExtractToFolder => {
                let archive = if one {
                    name(first)
                } else {
                    tr!("{paths} archives", paths = r.paths.len())
                };
                let to = match r.action {
                    Action::ExtractHere => folder.display().to_string(),
                    _ if one => folder.join(stem(&archive)).display().to_string(),
                    _ => tr!("{folder}, a folder for each", folder = folder.display()),
                };
                // Each archive's files go beside it, or in a folder of their
                // own there.
                let work = r
                    .paths
                    .iter()
                    .map(|p| {
                        let here = p.parent().unwrap_or(Path::new("")).to_path_buf();
                        let to = match r.action {
                            Action::ExtractHere => here,
                            _ => here.join(stem(&name(p))),
                        };
                        (Source::from(p), Some(to))
                    })
                    .collect();
                let mut job = Job::shown(Verb::Extract, archive.into(), Some(to.into()), 0, 0);
                job.work = work;
                job.how = prefs::prefs(cx).extract_options();
                self.open_progress(job, window, cx);
            }
            Action::Extract => self.open_archive(first.clone(), true, window, cx),
            Action::Open => self.open_archive(first.clone(), false, window, cx),
            // Encrypted, as it has to be, takes asking how.
            Action::CompressZip if crate::managed::policy(cx).requires_encryption() => {
                self.open_create(r.paths, window, cx)
            }
            Action::CompressZip => {
                let selection = Selection {
                    paths: r.paths.clone(),
                    ..Selection::default()
                };
                let archive = format!("{}.zip", selection.suggested_name());
                let mut job = Job::shown(
                    Verb::Create,
                    archive.clone().into(),
                    Some(folder.display().to_string().into()),
                    0,
                    0,
                );
                job.create = Some(Creating {
                    to: Saving::Here(folder.join(archive)),
                    paths: selection.paths,
                    how: NewArchive {
                        level: prefs::prefs(cx).level,
                        skip_mac_files: prefs::prefs(cx).skip_mac_files,
                        stored_extensions: prefs::stored_extensions(cx),
                        ..NewArchive::default()
                    },
                });
                self.open_progress(job, window, cx);
            }
            Action::Compress => self.open_create(r.paths, window, cx),
        }
    }

    /// What was asked of an archive of the Recent page.
    fn on_recent(&mut self, e: &RecentEvent, window: &mut Window, cx: &mut Context<Self>) {
        // The folder it is in, as a storage, and its name there.
        let place = |path: &std::path::Path| {
            let folder = path.parent().unwrap_or(std::path::Path::new("."));
            let name = path.file_name().map(|n| n.to_string_lossy().into_owned());
            let storage: std::sync::Arc<dyn packwing_core::Storage> =
                std::sync::Arc::new(packwing_core::LocalFolder::new(folder));
            (storage, name.into_iter().collect::<Vec<_>>())
        };
        match e {
            RecentEvent::Open(path) => self.open_archive(path.clone(), false, window, cx),
            RecentEvent::Extract(path) => self.open_archive(path.clone(), true, window, cx),
            RecentEvent::Sign(path) => {
                let (storage, path) = place(path);
                let view = cx.new(|cx| SignDialog::new(storage, path, window, cx));
                self.show(view, true, window, cx);
            }
            RecentEvent::Checksums(path) => {
                let (storage, path) = place(path);
                let view = cx.new(|cx| ChecksumDialog::new(storage, path, cx));
                self.show(view, true, window, cx);
            }
            RecentEvent::Convert(path) => self.open_convert(path.clone(), window, cx),
            RecentEvent::Split(path) => self.open_split(path.clone(), window, cx),
            RecentEvent::Join(path) => self.start_join(path.clone(), window, cx),
            RecentEvent::Forget(path) => {
                prefs::change(cx, |p| p.forget_opened(path));
                let rows = prefs::prefs(cx).recent.iter().map(Row::of).collect();
                self.recent.update(cx, |r, cx| r.show(rows, cx));
            }
        }
    }

    /// Selects entries of the archive open, as the screenshots draw it.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn select_in_archive(&mut self, at: &[usize], cx: &mut Context<Self>) {
        self.archive.update(cx, |p, cx| p.select_for_shot(at, cx));
    }

    /// The page of the archive, for tests.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn archive_page(&self) -> Entity<ArchivePage> {
        self.archive.clone()
    }

    /// Fills the preview of the archive open, as the screenshots draw it.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn preview_in_archive(
        &mut self,
        picture: Option<Vec<u8>>,
        text: &str,
        cx: &mut Context<Self>,
    ) {
        self.archive
            .update(cx, |p, cx| p.preview_for_shot(picture, text, cx));
    }

    /// Searches the archive open, as the screenshots draw it.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn search_archive(&mut self, query: &str, cx: &mut Context<Self>) {
        self.archive.update(cx, |p, cx| p.search_for(query, cx));
    }

    /// Browse, showing what was opened before.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn show_recent_tab(&mut self, cx: &mut Context<Self>) {
        self.browse.update(cx, |b, cx| {
            b.show_tab(crate::pages::browse::Tab::Recent, cx)
        });
        self.show_page(Page::Browse, cx);
    }

    #[cfg(all(test, feature = "test-hooks"))]
    pub fn browse_page(&self) -> Entity<BrowsePage> {
        self.browse.clone()
    }

    /// The menu of the row at `i` of Recent, as the screenshots draw it.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn open_recent_menu_for_shot(
        &mut self,
        at: gpui::Point<gpui::Pixels>,
        i: usize,
        cx: &mut Context<Self>,
    ) {
        self.recent
            .update(cx, |p, cx| p.open_menu_for_shot(at, i, cx));
    }

    /// The Recent page as the screenshots draw it.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn show_recent_samples(&mut self, cx: &mut Context<Self>) {
        let rows = demo::RECENT
            .iter()
            .map(|r| Row {
                path: std::path::Path::new(r.location).join(r.name),
                name: r.name.to_owned(),
                location: r.location.to_owned(),
                size: Some(r.size),
                opened: r.opened.to_owned(),
            })
            .collect();
        self.recent.update(cx, |p, cx| p.show(rows, cx));
        self.show_recent_tab(cx);
    }

    /// Whether a dialog is open, for tests.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn dialog_open(&self) -> bool {
        self.dialog.is_some()
    }

    /// Why the job in the dialog open failed, if it did, for tests.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn job_failed(&self, cx: &gpui::App) -> Option<String> {
        self.dialog
            .as_ref()
            .and_then(|d| d.view.clone().downcast::<ProgressDialog>().ok())
            .and_then(|view| view.read(cx).failed().map(str::to_owned))
    }

    /// What New archive holds, if it is the dialog open, for tests.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn create_dialog_paths(&self, cx: &gpui::App) -> Vec<PathBuf> {
        self.dialog
            .as_ref()
            .and_then(|d| d.view.clone().downcast::<CreateDialog>().ok())
            .map(|view| view.read(cx).selection_paths())
            .unwrap_or_default()
    }

    /// Files were dropped on the window, with no dialog there to take them.
    pub fn dropped(&mut self, paths: Vec<PathBuf>, window: &mut Window, cx: &mut Context<Self>) {
        if self.dialog.is_some() || self.welcome.is_some() {
            return;
        }
        let takes_more = self.page == Page::Archive
            && self
                .archive
                .read(cx)
                .archive()
                .filter(|a| a.path.is_some())
                .and_then(|a| a.listing.as_ref())
                .is_some_and(|l| packwing_core::can_add(l));
        match what_a_drop_means(&paths, takes_more) {
            Dropped::AddTo => {
                let archive = self.archive.read(cx).archive().cloned();
                let dialog = archive.and_then(|a| AddDialog::new(&a, cx));
                if let Some(dialog) = dialog {
                    let view = cx.new(|_| dialog);
                    view.update(cx, |d, cx| d.add(paths, cx));
                    self.show(view, true, window, cx);
                }
            }
            Dropped::Open(path) => self.open_archive(path, false, window, cx),
            Dropped::New => self.open_create(paths, window, cx),
        }
    }

    /// Asks what to add to the archive open; the samples have no file to add to.
    pub fn add_files(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let archive = self.archive.read(cx).archive().cloned();
        let dialog = archive.and_then(|a| AddDialog::new(&a, cx));
        if let Some(dialog) = dialog {
            let view = cx.new(|_| dialog);
            self.show(view, true, window, cx);
        }
    }

    fn on_key(&mut self, e: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        if e.keystroke.key == "escape" && self.dialog.as_ref().is_some_and(|d| d.dismissable) {
            self.close_dialog(cx);
            cx.stop_propagation();
        }
    }

    /// The menu down the window's left side, [`MENU_W`] wide.
    fn menu(&self, top: f32, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let size = label_size(
            &[
                tr!("Archive"),
                tr!("Browse"),
                tr!("New"),
                tr!("Storages"),
                tr!("Jobs"),
                tr!("Settings"),
            ],
            window,
        );
        let item = |id: &'static str, icon_name: &'static str, label: String, on: bool| {
            let ink = if on { c.on_tonal } else { c.muted };
            div()
                .debug_selector(|| id.to_owned())
                .id(id)
                .group(id)
                .flex()
                .flex_col()
                .items_center()
                .gap(ui(4.))
                .cursor_pointer()
                .text_size(ui(size))
                .font_weight(if on {
                    FontWeight::SEMIBOLD
                } else {
                    FontWeight::MEDIUM
                })
                .text_color(rgb(ink))
                .child(
                    div()
                        .w(ui(56.))
                        .h(ui(32.))
                        .rounded_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .when(on, |d| d.bg(rgb(c.tonal)))
                        .when(!on, |d| d.group_hover(id, |s| s.bg(rgb(c.field))))
                        .child(icon(icon_name, 20., ink)),
                )
                .child(div().text_center().child(label))
        };
        let page = self.page;
        let jobs = self.jobs.read(cx).summary();
        // Where the system draws the title bar, the badge has no place: a dot says it.
        let dot = !chrome::draws_title_bar(window) && self.updates.read(cx).badge().is_some();
        div()
            .w(ui(MENU_W))
            .flex_shrink_0()
            .flex()
            .flex_col()
            .items_center()
            .gap(ui(10.))
            .px(ui(4.))
            .pt(ui(top))
            .pb(ui(20.))
            .child(
                tile(48., 16., accent_tint())
                    .mb(ui(12.))
                    .child(logo(24., accent_tint())),
            )
            .child(
                item(
                    "nav-archive",
                    "archive",
                    tr!("Archive"),
                    page == Page::Archive,
                )
                .on_click(cx.listener(|this, _, _, cx| this.show_page(Page::Archive, cx))),
            )
            .child(
                item("nav-browse", "folder", tr!("Browse"), page == Page::Browse)
                    .on_click(cx.listener(|this, _, _, cx| this.show_page(Page::Browse, cx))),
            )
            .child(
                item("nav-new", "plus", tr!("New"), false).on_click(cx.listener(
                    |this, _, window, cx| {
                        this.open_create(Vec::new(), window, cx);
                    },
                )),
            )
            .child(
                item(
                    "nav-storages",
                    "storages",
                    tr!("Storages"),
                    page == Page::Storages,
                )
                .on_click(cx.listener(|this, _, _, cx| this.show_page(Page::Storages, cx))),
            )
            .child(
                item(
                    "nav-settings",
                    "sliders",
                    tr!("Settings"),
                    page == Page::Settings,
                )
                .relative()
                .when(dot, |d| {
                    d.child(
                        div()
                            .absolute()
                            .top(ui(2.))
                            .right(ui(8.))
                            .size(ui(8.))
                            .rounded_full()
                            .bg(rgb(c.accent)),
                    )
                })
                .on_click(cx.listener(|this, _, _, cx| this.show_page(Page::Settings, cx))),
            )
            // At the foot, where it is seen as it comes and goes.
            .when(jobs.jobs > 0 || page == Page::Jobs, |d| {
                d.child(div().flex_1()).child(
                    item("nav-jobs", "jobs", tr!("Jobs"), page == Page::Jobs)
                        .relative()
                        .children(jobs_mark(jobs))
                        .on_click(cx.listener(|this, _, _, cx| this.show_page(Page::Jobs, cx))),
                )
            })
    }

    /// The shade over the window, with the dialog on it.
    fn overlay(&self, view: AnyView, window: &Window) -> impl IntoElement {
        // It covers the whole surface, the margin around the window too: the
        // shade stops at the window's edge and follows its corners.
        let (m, round) = (chrome::margins(window), chrome::corners(window));
        div()
            .id("dialog")
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            // Nothing under it takes a click.
            .occlude()
            .pt(m.top)
            .pr(m.right)
            .pb(m.bottom)
            .pl(m.left)
            .child(
                div()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .p(ui(24.))
                    .rounded_tl(round.top_left)
                    .rounded_tr(round.top_right)
                    .rounded_bl(round.bottom_left)
                    .rounded_br(round.bottom_right)
                    .bg(rgba(colors().scrim))
                    .child(view),
            )
    }
}

impl Focusable for Workspace {
    fn focus_handle(&self, _: &gpui::App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for Workspace {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Every size in rems follows the body size: the system's, or Settings'.
        window.set_rem_size(crate::text_size::rem_for(crate::text_size::body(cx)));
        let c = colors();
        // Under the app's title bar the page starts at once; under the
        // system's it keeps the design's margin.
        let top = if chrome::draws_title_bar(window) {
            4.
        } else {
            20.
        };
        let body = match &self.welcome {
            Some((welcome, _)) => div().flex_1().min_h_0().flex().child(welcome.clone()),
            None => {
                let page: AnyView = match self.page {
                    Page::Archive => self.archive.clone().into(),
                    Page::Storages => self.storages.clone().into(),
                    Page::Browse => self.browse.clone().into(),
                    Page::Jobs => self.jobs.clone().into(),
                    Page::Settings => self.settings.clone().into(),
                };
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .child(self.menu(top, window, cx))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .pt(ui(top))
                            .pr(ui(24.))
                            .pb(ui(24.))
                            .pl(ui(8.))
                            .child(page),
                    )
            }
        };
        let switch = chrome::appearance_switch(self.appearance).on_click(cx.listener(
            |this, _, window, cx| {
                let next = this.appearance.next();
                this.set_appearance(next, window, cx);
            },
        ));
        let badge = self.update_badge(cx);
        let framed = chrome::window_chrome(
            div().flex_1().min_h_0().flex().flex_col().child(body),
            rgb(c.page).into(),
            switch,
            badge,
            window,
            cx,
        );
        let dialog = self.dialog.as_ref().map(|d| d.view.clone());
        div()
            .id("workspace")
            .relative()
            .size_full()
            .track_focus(&self.focus)
            .on_key_down(cx.listener(Self::on_key))
            .drag_over::<ExternalPaths>(|style, _, _, _| style.opacity(0.85))
            .on_drop(cx.listener(|this, dropped: &ExternalPaths, window, cx| {
                this.dropped(dropped.paths().to_vec(), window, cx)
            }))
            .font(crate::ui_font())
            .text_size(ui(15.))
            .text_color(rgb(c.text))
            .child(framed)
            .when_some(dialog, |d, view| d.child(self.overlay(view, window)))
    }
}

#[cfg(test)]
mod drop_tests {
    use super::*;

    #[test]
    fn a_drop_goes_where_it_makes_sense() {
        let path = |p: &str| vec![PathBuf::from(p)];
        assert_eq!(
            what_a_drop_means(&path("/a/b.zip"), false),
            Dropped::Open(PathBuf::from("/a/b.zip"))
        );
        assert_eq!(
            what_a_drop_means(&path("/a/b.TAR.ZST"), false),
            Dropped::Open(PathBuf::from("/a/b.TAR.ZST"))
        );
        assert_eq!(what_a_drop_means(&path("/a/b.zip"), true), Dropped::AddTo);
        assert_eq!(what_a_drop_means(&path("/a/b.txt"), true), Dropped::AddTo);
        assert_eq!(what_a_drop_means(&path("/a/b.txt"), false), Dropped::New);
        let two = [PathBuf::from("/a/b.zip"), PathBuf::from("/a/c.zip")];
        assert_eq!(what_a_drop_means(&two, false), Dropped::New);
    }
}
