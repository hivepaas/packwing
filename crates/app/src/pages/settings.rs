//! Settings: a menu of sections and the one open, each section a view of
//! its own. What they choose is kept (`prefs.rs`) and acted on.

mod about;
mod compressing;
mod extracting;
mod general;
mod license;
mod right_click;
mod security;

pub use general::{Appearance, AppearanceChosen};

use gpui::{
    AnyView, Context, Div, Entity, EventEmitter, FontWeight, IntoElement, Render, ScrollHandle,
    SharedString, Subscription, Window, div, point, prelude::*, px, rgb,
};
use packwing_i18n::tr;

use super::{panel, title};
use crate::colors::colors;
use crate::right_click::RightClickMenu;
use crate::text_size::ui;
use crate::updates::Updates;

/// The sections, in the menu's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
    General,
    Extracting,
    Compressing,
    Security,
    RightClick,
    License,
    About,
}

impl Section {
    const ALL: [Section; 7] = [
        Section::General,
        Section::Extracting,
        Section::Compressing,
        Section::Security,
        Section::RightClick,
        Section::License,
        Section::About,
    ];

    fn name(self) -> String {
        match self {
            Section::General => tr!("General"),
            Section::Extracting => tr!("Extracting"),
            Section::Compressing => tr!("Compressing"),
            Section::Security => tr!("Security"),
            Section::RightClick => tr!("Right-click menu"),
            Section::License => tr!("License"),
            Section::About => tr!("About"),
        }
    }
}

pub struct SettingsPage {
    section: Section,
    general: Entity<general::General>,
    /// Each section's view, in [`Section::ALL`]'s order.
    views: Vec<AnyView>,
    /// How far the open section is scrolled.
    scroll: ScrollHandle,
    _appearance: Subscription,
}

impl EventEmitter<AppearanceChosen> for SettingsPage {}

impl SettingsPage {
    pub fn new(
        appearance: Appearance,
        menu: Entity<RightClickMenu>,
        updates: Entity<Updates>,
        cx: &mut Context<Self>,
    ) -> Self {
        let general = cx.new(|cx| general::General::new(appearance, menu.clone(), cx));
        let appearance = cx.subscribe(&general, |_, _, e: &AppearanceChosen, cx| {
            cx.emit(AppearanceChosen(e.0))
        });
        let views = vec![
            general.clone().into(),
            cx.new(extracting::Extracting::new).into(),
            cx.new(compressing::Compressing::new).into(),
            {
                let kept = crate::prefs::prefs(cx);
                let (mark, ask) = (kept.keep_mark, kept.ask_before_open);
                cx.new(|cx| security::Security::new(mark, ask, cx)).into()
            },
            cx.new(|cx| right_click::RightClick::new(menu, cx)).into(),
            cx.new(license::LicenseSection::new).into(),
            cx.new(|cx| about::About::new(updates, cx)).into(),
        ];
        SettingsPage {
            section: Section::General,
            general,
            views,
            scroll: ScrollHandle::new(),
            _appearance: appearance,
        }
    }

    pub fn show_section(&mut self, section: Section, cx: &mut Context<Self>) {
        self.section = section;
        cx.notify();
    }

    /// Scrolls down, for a picture of what is further down.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn scroll_for_test(&mut self, by: f32, cx: &mut Context<Self>) {
        self.scroll.set_offset(point(px(0.), px(-by)));
        cx.notify();
    }

    #[cfg(all(test, feature = "test-hooks"))]
    pub fn section(&self) -> Section {
        self.section
    }

    /// Shows About from its top, where its Updates group is.
    pub fn show_updates(&mut self, cx: &mut Context<Self>) {
        self.scroll.set_offset(point(px(0.), px(0.)));
        self.show_section(Section::About, cx);
    }

    /// Shows `a` as chosen, when it was chosen elsewhere.
    pub fn show_appearance(&mut self, a: Appearance, cx: &mut Context<Self>) {
        self.general.update(cx, |g, cx| g.show_appearance(a, cx));
    }

    fn menu(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let items = Section::ALL.into_iter().enumerate().map(|(i, s)| {
            let on = s == self.section;
            div()
                .debug_selector(|| format!("section-{i}"))
                .id(("section", i))
                .flex()
                .items_center()
                .h(ui(52.))
                .px(ui(18.))
                .rounded_full()
                .text_size(ui(15.))
                .cursor_pointer()
                .when(on, |d| {
                    d.bg(rgb(c.tonal))
                        .text_color(rgb(c.on_tonal))
                        .font_weight(FontWeight::BOLD)
                })
                .when(!on, |d| {
                    d.font_weight(FontWeight::MEDIUM)
                        .hover(|st| st.bg(rgb(c.hover)))
                })
                .on_click(cx.listener(move |this, _, _, cx| this.show_section(s, cx)))
                .child(s.name())
        });
        div()
            .w(ui(240.))
            .flex_shrink_0()
            .flex()
            .flex_col()
            .gap(ui(4.))
            .px(ui(8.))
            .py(ui(12.))
            .child(title(tr!("Settings"), 24.).px(ui(16.)).pb(ui(16.)))
            .children(items)
    }
}

impl Render for SettingsPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let at = Section::ALL
            .iter()
            .position(|s| *s == self.section)
            .unwrap_or(0);
        let name: SharedString = self.section.name().into();
        div()
            .size_full()
            .flex()
            .gap(ui(8.))
            .child(self.menu(cx))
            .child(
                panel()
                    .id("settings-content")
                    .flex_1()
                    .min_w_0()
                    .max_h_full()
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll)
                    .flex()
                    .flex_col()
                    .gap(ui(14.))
                    .px(ui(24.))
                    .py(ui(20.))
                    .child(title(name, 26.).pb(ui(4.)))
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .child(self.views[at].clone()),
                    ),
            )
    }
}

/// What a section's view is drawn in: its settings one under another.
fn section() -> Div {
    div().flex_1().flex().flex_col().gap(ui(14.))
}

/// A group of settings on the well's tint.
fn group() -> Div {
    div()
        .flex()
        .flex_col()
        .gap(ui(10.))
        .px(ui(18.))
        .py(ui(16.))
        .rounded(ui(22.))
        .bg(rgb(colors().well))
}

/// A setting's name.
fn label(text: impl Into<SharedString>) -> Div {
    div()
        .text_size(ui(15.))
        .font_weight(FontWeight::SEMIBOLD)
        .child(text.into())
}

/// What a setting does, under its name.
fn explain(text: impl Into<SharedString>) -> Div {
    div()
        .text_size(ui(13.))
        .text_color(rgb(colors().muted))
        .child(text.into())
}

/// A group's name and what it is for, close together.
fn heading(name: impl Into<SharedString>, does: impl Into<SharedString>) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(ui(2.))
        .child(label(name))
        .child(explain(does))
}

/// A setting in a row of its own: its name and what it does, and what
/// changes it at the right.
fn setting(
    name: impl Into<SharedString>,
    does: impl Into<SharedString>,
    control: impl IntoElement,
) -> Div {
    div()
        .flex()
        .items_center()
        .gap(ui(16.))
        .min_h(ui(60.))
        .px(ui(18.))
        .py(ui(8.))
        .rounded(ui(18.))
        .bg(rgb(colors().well))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .child(label(name))
                .child(explain(does)),
        )
        .child(div().flex_shrink_0().child(control))
}

/// Settings one under another, close together.
fn rows(items: impl IntoIterator<Item = impl IntoElement>) -> Div {
    div().flex().flex_col().gap(ui(6.)).children(items)
}
