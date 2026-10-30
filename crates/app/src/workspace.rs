//! The main window: the menu down its left side and the page chosen in it,
//! the first run's welcome in their place, and a dialog over them all.

use gpui::{
    AnyView, Context, Entity, EventEmitter, FocusHandle, Focusable, FontWeight, KeyDownEvent,
    PathPromptOptions, Render, Subscription, Window, WindowAppearance, div, prelude::*, px, rgb,
    rgba,
};

use crate::chrome;
use crate::colors::{self, colors};
use crate::demo;
use crate::dialogs::{
    CreateDialog, DamagedDialog, DialogEvent, ExtractDialog, Job, ProgressDialog, Verb,
};
use crate::pages::archive::{ArchiveEvent, ArchivePage};
use crate::pages::onboarding::{Done, Onboarding};
use crate::pages::recent::{OpenRecent, RecentPage};
use crate::pages::settings::{Appearance, AppearanceChosen, SettingsPage};
use crate::widgets::{accent_tint, icon, logo, tile};

/// The pages the menu leads to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Archive,
    Recent,
    Settings,
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
    /// The first run's welcome, shown instead of the pages.
    welcome: Option<(Entity<Onboarding>, Subscription)>,
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
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let appearance = Appearance::System;
        let archive = cx.new(|cx| ArchivePage::new(demo::PROJECT, cx));
        let recent = cx.new(|_| RecentPage::new(demo::RECENT));
        let settings =
            cx.new(|cx| SettingsPage::new(appearance, managed.then_some(demo::MANAGED), cx));
        let subscriptions = vec![
            cx.subscribe_in(
                &archive,
                window,
                |this, _, e: &ArchiveEvent, window, cx| match e {
                    ArchiveEvent::Extract => this.open_extract(window, cx),
                    ArchiveEvent::AddFiles => this.add_files(cx),
                    ArchiveEvent::Test => {
                        let a = demo::PROJECT;
                        this.open_progress(
                            Job {
                                verb: Verb::Test,
                                name: a.name.into(),
                                to: None,
                                files: a.files,
                                bytes: a.unpacked,
                            },
                            window,
                            cx,
                        );
                    }
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
            welcome: None,
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

    pub fn show_welcome(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let welcome = cx.new(Onboarding::new);
        let done = cx.subscribe_in(&welcome, window, |this, _, _: &Done, _, cx| {
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

    pub fn open_extract(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let view = cx.new(|cx| ExtractDialog::new(demo::PROJECT, cx));
        self.show(view, true, window, cx);
    }

    pub fn open_create(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<CreateDialog> {
        let view = cx.new(|cx| CreateDialog::new(demo::SELECTION, cx));
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

    fn open_recent(&mut self, i: usize, window: &mut Window, cx: &mut Context<Self>) {
        if demo::RECENT.get(i).is_some_and(|r| r.damaged) {
            self.open_damaged(window, cx);
        } else {
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
                    this.open_create(window, cx);
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
        let framed = chrome::window_chrome(
            div().flex_1().min_h_0().flex().flex_col().child(body),
            rgb(c.page).into(),
            window,
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
