//! The main window: the menu down its left side and the page chosen in it,
//! the first run's welcome in their place, and a dialog over them all.

use std::path::{Path, PathBuf};

use gpui::{
    AnyView, Context, Entity, EventEmitter, FocusHandle, Focusable, FontWeight, KeyDownEvent,
    PathPromptOptions, Render, Subscription, Window, WindowAppearance, div, prelude::*, px, rgb,
    rgba,
};
use packwing_core::{Asked, Keys, NewArchive};

use crate::chrome;
use crate::colors::{self, colors};
use crate::demo;
use crate::dialogs::{
    CreateDialog, Creating, DamagedDialog, DialogEvent, ExtractDialog, Job, PasswordDialog,
    PasswordEvent, ProgressDialog, Verb, asked_for,
};
use crate::model::{Archive, Selection, stem};
use crate::pages::archive::{ArchiveEvent, ArchivePage, Shown};
use crate::pages::onboarding::{Done, Onboarding};
use crate::pages::recent::{OpenRecent, RecentPage};
use crate::pages::settings::{Appearance, AppearanceChosen, SettingsPage};
use crate::right_click::{Action, Request, RightClickMenu};
use crate::widgets::{accent_tint, icon, logo, tile};

/// The pages the menu leads to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Archive,
    Recent,
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
    /// Reading the archive at `path`, read with `keys` so far; with
    /// `then_extract`, Extract opens over it once it is read.
    Read {
        path: PathBuf,
        keys: Keys,
        then_extract: bool,
    },
    /// Running a job again: an item of the right-click menu, which reads
    /// no listing first.
    Run(Job),
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
            Then::Read { path, .. } => path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default(),
            Then::Run(job) => job.name.to_string(),
        }
    }
}

/// The dialog open over the window.
struct Open {
    view: AnyView,
    /// Escape closes it: not a job's progress, which has its own buttons.
    dismissable: bool,
    _events: Subscription,
}

pub struct Workspace {
    focus: FocusHandle,
    page: Page,
    archive: Entity<ArchivePage>,
    recent: Entity<RecentPage>,
    settings: Entity<SettingsPage>,
    /// packwing's items in the system's right-click menu.
    right_click: Entity<RightClickMenu>,
    /// The first run's welcome, shown instead of the pages.
    welcome: Option<(Entity<Onboarding>, Subscription)>,
    /// The file being read: a reading that ends after another began is
    /// not shown.
    reading: Option<PathBuf>,
    dialog: Option<Open>,
    appearance: Appearance,
    /// The system's own appearance is dark.
    system_dark: bool,
    _subscriptions: Vec<Subscription>,
}

fn looks_dark(window: &Window) -> bool {
    matches!(
        window.appearance(),
        WindowAppearance::Dark | WindowAppearance::VibrantDark
    )
}

impl Workspace {
    /// `managed`: run as an organization would, with its policies.
    pub fn new(
        first_run: bool,
        managed: bool,
        right_click: Entity<RightClickMenu>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let appearance = Appearance::System;
        let archive = cx.new(ArchivePage::new);
        let recent = cx.new(|_| RecentPage::new(demo::RECENT));
        let settings = cx.new(|cx| {
            SettingsPage::new(
                appearance,
                managed.then_some(demo::MANAGED),
                right_click.clone(),
                cx,
            )
        });
        let subscriptions = vec![
            cx.subscribe_in(
                &archive,
                window,
                |this, _, e: &ArchiveEvent, window, cx| match e {
                    ArchiveEvent::Extract => this.open_extract(window, cx),
                    ArchiveEvent::AddFiles => this.add_files(cx),
                    ArchiveEvent::Test => this.test_archive(window, cx),
                    ArchiveEvent::Open => this.choose_archive(window, cx),
                },
            ),
            cx.subscribe_in(&recent, window, |this, _, e: &OpenRecent, window, cx| {
                this.open_recent(e.0, window, cx)
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
            recent,
            settings,
            right_click,
            welcome: None,
            reading: None,
            dialog: None,
            appearance,
            system_dark: looks_dark(window),
            _subscriptions: subscriptions,
        };
        if first_run {
            ws.show_welcome(window, cx);
        }
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

    pub fn set_appearance(&mut self, a: Appearance, window: &mut Window, cx: &mut Context<Self>) {
        self.appearance = a;
        self.settings.update(cx, |s, cx| s.show_appearance(a, cx));
        self.apply_appearance(window, cx);
    }

    pub fn show_page(&mut self, page: Page, cx: &mut Context<Self>) {
        self.page = page;
        cx.notify();
    }

    #[cfg(all(test, feature = "shots"))]
    pub fn show_settings(
        &mut self,
        section: crate::pages::settings::Section,
        cx: &mut Context<Self>,
    ) {
        self.settings
            .update(cx, |s, cx| s.show_section(section, cx));
        self.show_page(Page::Settings, cx);
    }

    #[cfg(all(test, feature = "shots"))]
    pub fn right_click(&self) -> &Entity<RightClickMenu> {
        &self.right_click
    }

    pub fn show_welcome(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let welcome = cx.new(Onboarding::new);
        let done = cx.subscribe_in(&welcome, window, |this, _, done: &Done, _, cx| {
            this.right_click
                .update(cx, |m, cx| m.show(done.right_click, cx));
            this.welcome = None;
            cx.notify();
        });
        self.welcome = Some((welcome, done));
        cx.notify();
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
                    this.open_progress(job.clone(), window, cx);
                }
                DialogEvent::Ask(job, asked, again) => {
                    let ask = Ask {
                        asked: *asked,
                        then: Then::Run(job.clone()),
                    };
                    this.ask(ask, again.clone(), window, cx);
                }
            },
        );
        self.dialog = Some(Open {
            view: view.into(),
            dismissable,
            _events: events,
        });
        cx.notify();
    }

    fn close_dialog(&mut self, cx: &mut Context<Self>) {
        self.dialog = None;
        cx.notify();
    }

    /// Asks for `asked`, as an archive that needs it does; with `again`,
    /// as after a wrong one.
    #[cfg(all(test, feature = "shots"))]
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
                path: PathBuf::from("/home/user/Downloads/Tài liệu dự án 2026.7z"),
                keys: Keys::default(),
                then_extract: false,
            },
        };
        self.ask(ask, again, window, cx);
    }

    #[cfg(all(test, feature = "shots"))]
    pub fn show_archive(&mut self, shown: Shown, cx: &mut Context<Self>) {
        self.archive.update(cx, |p, cx| p.show(shown, cx));
    }

    /// Shows the sample archive, as the screenshots draw it.
    pub fn show_sample(&mut self, cx: &mut Context<Self>) {
        self.reading = None;
        self.archive
            .update(cx, |p, cx| p.show(Shown::Open(demo::project()), cx));
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
        self.read_archive(path, Keys::default(), then_extract, window, cx);
    }

    /// Reads the archive at `path` with `keys`. One that turns out to need
    /// a passphrase or a password asks for it, and is read again with it;
    /// a wrong one asks again, saying so.
    fn read_archive(
        &mut self,
        path: PathBuf,
        keys: Keys,
        then_extract: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        self.archive
            .update(cx, |p, cx| p.show(Shown::Opening(name.clone()), cx));
        self.show_page(Page::Archive, cx);
        self.reading = Some(path.clone());
        let (read, with) = (path.clone(), keys.clone());
        let task = cx.background_executor().spawn(async move {
            let listing = packwing_core::list(&read, &with);
            listing.map(|l| (l, packwing_core::came_from_internet(&read)))
        });
        cx.spawn_in(window, async move |this, cx| {
            let result = task.await;
            let _ = this.update_in(cx, |this, window, cx| {
                if this.reading.as_ref() != Some(&path) {
                    return;
                }
                this.reading = None;
                let shown = match result {
                    Ok((listing, marked)) => {
                        Shown::Open(Archive::read(path, listing, keys, marked))
                    }
                    Err(e) => match asked_for(&e, &keys) {
                        Some((asked, again)) => {
                            let then = Then::Read {
                                path,
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
                this.archive.update(cx, |p, cx| p.show(shown, cx));
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
                                path, then_extract, ..
                            } => this.read_archive(path.clone(), keys, *then_extract, window, cx),
                            Then::Run(job) => {
                                let mut job = job.clone();
                                job.keys = keys;
                                this.open_progress(job, window, cx);
                            }
                        }
                    }
                    (PasswordEvent::Cancelled, Then::Read { .. }) => {
                        let why = match ask.asked {
                            Asked::Passphrase => "This file is encrypted with a passphrase.",
                            Asked::Password => "This archive needs its password.",
                        };
                        let shown = Shown::Failed {
                            name: name.clone(),
                            why: why.into(),
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
            _events: events,
        });
        cx.notify();
    }

    /// Asks which archive to open, and opens it.
    fn choose_archive(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let picked = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Open".into()),
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

    pub fn open_extract(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(archive) = self.archive.read(cx).archive().cloned() else {
            return;
        };
        let view = cx.new(|cx| ExtractDialog::new(archive, cx));
        self.show(view, true, window, cx);
    }

    /// Reads every file of the archive open through, its checks made.
    fn test_archive(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(a) = self.archive.read(cx).archive().cloned() else {
            return;
        };
        let mut job = Job::shown(Verb::Test, a.name.clone().into(), None, a.files, a.unpacked);
        if let Some(path) = a.path {
            job.work = vec![(path, None)];
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
        let view = cx.new(|cx| CreateDialog::new(Selection::default(), cx));
        if !paths.is_empty() {
            view.update(cx, |d, cx| d.add(paths, cx));
        }
        self.show(view, true, window, cx);
    }

    /// New archive, holding the screenshots' sample.
    #[cfg(all(test, feature = "shots"))]
    pub fn open_create_sample(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<CreateDialog> {
        let view = cx.new(|cx| CreateDialog::new(demo::selection(), cx));
        self.show(view.clone(), true, window, cx);
        view
    }

    pub fn open_damaged(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let view = cx.new(|_| DamagedDialog::new(demo::DAMAGED));
        self.show(view, true, window, cx);
    }

    pub fn open_progress(
        &mut self,
        job: Job,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<ProgressDialog> {
        let view = cx.new(|cx| ProgressDialog::new(job, cx));
        self.show(view.clone(), false, window, cx);
        view
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
                    format!("{} archives", r.paths.len())
                };
                let to = match r.action {
                    Action::ExtractHere => folder.display().to_string(),
                    _ if one => folder.join(stem(&archive)).display().to_string(),
                    _ => format!("{}, a folder for each", folder.display()),
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
                        (p.clone(), Some(to))
                    })
                    .collect();
                let mut job = Job::shown(Verb::Extract, archive.into(), Some(to.into()), 0, 0);
                job.work = work;
                self.open_progress(job, window, cx);
            }
            Action::Extract => self.open_archive(first.clone(), true, window, cx),
            Action::Open => self.open_archive(first.clone(), false, window, cx),
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
                    archive: folder.join(archive),
                    paths: selection.paths,
                    how: NewArchive::default(),
                });
                self.open_progress(job, window, cx);
            }
            Action::Compress => self.open_create(r.paths, window, cx),
        }
    }

    /// The Recent page's samples: the damaged one's dialog, or the sample
    /// archive, until packwing keeps its own list.
    fn open_recent(&mut self, i: usize, window: &mut Window, cx: &mut Context<Self>) {
        if demo::RECENT.get(i).is_some_and(|r| r.damaged) {
            self.open_damaged(window, cx);
        } else {
            self.show_sample(cx);
            self.show_page(Page::Archive, cx);
        }
    }

    fn add_files(&mut self, cx: &mut Context<Self>) {
        // Nothing is done with them yet: adding means writing the archive.
        let picked = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: true,
            multiple: true,
            prompt: Some("Add".into()),
        });
        cx.spawn(async move |_, _| {
            let _ = picked.await;
        })
        .detach();
    }

    fn on_key(&mut self, e: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        if e.keystroke.key == "escape" && self.dialog.as_ref().is_some_and(|d| d.dismissable) {
            self.close_dialog(cx);
            cx.stop_propagation();
        }
    }

    /// The menu down the window's left side.
    fn menu(&self, top: f32, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let item = |id: &'static str, icon_name: &'static str, label: &'static str, on: bool| {
            let ink = if on { c.on_tonal } else { c.muted };
            div()
                .id(id)
                .group(id)
                .flex()
                .flex_col()
                .items_center()
                .gap(px(4.))
                .cursor_pointer()
                .text_size(px(12.))
                .font_weight(if on {
                    FontWeight::SEMIBOLD
                } else {
                    FontWeight::MEDIUM
                })
                .text_color(rgb(ink))
                .child(
                    div()
                        .w(px(56.))
                        .h(px(32.))
                        .rounded_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .when(on, |d| d.bg(rgb(c.tonal)))
                        .when(!on, |d| d.group_hover(id, |s| s.bg(rgb(c.field))))
                        .child(icon(icon_name, 20., ink)),
                )
                .child(label)
        };
        let page = self.page;
        div()
            .w(px(88.))
            .flex_shrink_0()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(10.))
            .px(px(12.))
            .pt(px(top))
            .pb(px(20.))
            .child(
                tile(48., 16., accent_tint())
                    .mb(px(12.))
                    .child(logo(24., accent_tint())),
            )
            .child(
                item("nav-archive", "archive", "Archive", page == Page::Archive)
                    .on_click(cx.listener(|this, _, _, cx| this.show_page(Page::Archive, cx))),
            )
            .child(
                item("nav-recent", "recent", "Recent", page == Page::Recent)
                    .on_click(cx.listener(|this, _, _, cx| this.show_page(Page::Recent, cx))),
            )
            .child(item("nav-new", "plus", "New", false).on_click(cx.listener(
                |this, _, window, cx| {
                    this.open_create(Vec::new(), window, cx);
                },
            )))
            .child(
                item(
                    "nav-settings",
                    "sliders",
                    "Settings",
                    page == Page::Settings,
                )
                .on_click(cx.listener(|this, _, _, cx| this.show_page(Page::Settings, cx))),
            )
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
                    .p(px(24.))
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
                    Page::Recent => self.recent.clone().into(),
                    Page::Settings => self.settings.clone().into(),
                };
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .child(self.menu(top, cx))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .pt(px(top))
                            .pr(px(24.))
                            .pb(px(24.))
                            .pl(px(8.))
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
        let framed = chrome::window_chrome(
            div().flex_1().min_h_0().flex().flex_col().child(body),
            rgb(c.page).into(),
            switch,
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
            .font_family(crate::FONT)
            .text_size(px(15.))
            .text_color(rgb(c.text))
            .child(framed)
            .when_some(dialog, |d, view| d.child(self.overlay(view, window)))
    }
}
