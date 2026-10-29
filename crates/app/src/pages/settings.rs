//! Settings: a menu of sections and the one open, each section a view of
//! its own. Nothing is saved yet: each keeps what is chosen while it runs,
//! but for the right-click menu, which is the system's.

mod about;
mod compressing;
mod extracting;
mod general;
mod right_click;
mod security;
mod updates;

pub use general::{Appearance, AppearanceChosen, LANGUAGES};

use gpui::{
    AnyView, Context, Div, Entity, EventEmitter, FontWeight, IntoElement, Render, SharedString,
    Subscription, Window, div, prelude::*, px, rgb,
};

use super::{panel, title};
use crate::colors::colors;
use crate::demo::Managed;
use crate::right_click::RightClickMenu;

/// The sections, in the menu's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
    General,
    Extracting,
    Compressing,
    Security,
    RightClick,
    Updates,
    About,
}

impl Section {
    const ALL: [Section; 7] = [
        Section::General,
        Section::Extracting,
        Section::Compressing,
        Section::Security,
        Section::RightClick,
        Section::Updates,
        Section::About,
    ];

    fn name(self) -> &'static str {
        match self {
            Section::General => "General",
            Section::Extracting => "Extracting",
            Section::Compressing => "Compressing",
            Section::Security => "Security",
            Section::RightClick => "Right-click menu",
            Section::Updates => "Updates",
            Section::About => "About",
        }
    }
}

pub struct SettingsPage {
    section: Section,
    general: Entity<general::General>,
    /// Each section's view, in [`Section::ALL`]'s order.
    views: Vec<AnyView>,
    _appearance: Subscription,
}

impl EventEmitter<AppearanceChosen> for SettingsPage {}

impl SettingsPage {
    /// `managed`: the organization's policies, when it runs packwing.
    pub fn new(
        appearance: Appearance,
        managed: Option<Managed>,
        menu: Entity<RightClickMenu>,
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
            cx.new(|_| security::Security::new(managed)).into(),
            cx.new(|cx| right_click::RightClick::new(menu, cx)).into(),
            cx.new(|_| updates::Updates::new()).into(),
            cx.new(|_| about::About).into(),
        ];
        SettingsPage {
            section: Section::General,
            general,
            views,
            _appearance: appearance,
        }
    }

    pub fn show_section(&mut self, section: Section, cx: &mut Context<Self>) {
        self.section = section;
        cx.notify();
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
                .id(("section", i))
                .flex()
                .items_center()
                .h(px(52.))
                .px(px(18.))
                .rounded_full()
                .text_size(px(15.))
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
            .w(px(240.))
            .flex_shrink_0()
            .flex()
            .flex_col()
            .gap(px(4.))
            .px(px(8.))
            .py(px(12.))
            .child(title("Settings", 24.).px(px(16.)).pb(px(16.)))
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
            .gap(px(8.))
            .child(self.menu(cx))
            .child(
                panel()
                    .id("settings-content")
                    .flex_1()
                    .min_w_0()
                    .max_h_full()
                    .overflow_y_scroll()
                    .flex()
                    .flex_col()
                    .gap(px(14.))
                    .px(px(24.))
                    .py(px(20.))
                    .child(title(name, 26.).pb(px(4.)))
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
    div().flex_1().flex().flex_col().gap(px(14.))
}

/// A group of settings on the well's tint.
fn group() -> Div {
    div()
        .flex()
        .flex_col()
        .gap(px(10.))
        .px(px(18.))
        .py(px(16.))
        .rounded(px(22.))
        .bg(rgb(colors().well))
}

/// A setting's name.
fn label(text: impl Into<SharedString>) -> Div {
    div()
        .text_size(px(15.))
        .font_weight(FontWeight::SEMIBOLD)
        .child(text.into())
}

/// What a setting does, under its name.
fn explain(text: impl Into<SharedString>) -> Div {
    div()
        .text_size(px(13.))
        .text_color(rgb(colors().muted))
        .child(text.into())
}

/// A group's name and what it is for, close together.
fn heading(name: impl Into<SharedString>, does: impl Into<SharedString>) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(px(2.))
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
        .gap(px(16.))
        .min_h(px(60.))
        .px(px(18.))
        .py(px(8.))
        .rounded(px(18.))
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
    div().flex().flex_col().gap(px(6.)).children(items)
}
