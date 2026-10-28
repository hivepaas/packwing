//! Settings: a menu of sections and the one open. General is drawn; the
//! others come with what they set.

use gpui::{
    Context, Entity, EventEmitter, FontWeight, Render, SharedString, Window, div, prelude::*, px,
    rgb,
};

use super::{panel, title};
use crate::colors::colors;
use crate::widgets::{
    Fill, Select, SelectLook, Size, button, chip, option_card, segment, segments, switch, titled,
};

/// Light, dark, or as the system is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Appearance {
    Light,
    Dark,
    #[default]
    System,
}

/// Another appearance was chosen.
pub struct AppearanceChosen(pub Appearance);

const SECTIONS: [&str; 7] = [
    "General",
    "Extracting",
    "Compressing",
    "Security",
    "Right-click menu",
    "Updates",
    "About",
];

pub const LANGUAGES: [&str; 3] = ["Same as system (English)", "English", "Tiếng Việt"];

/// The archive types packwing can open.
const TYPES: [&str; 9] = ["ZIP", "RAR", "7z", "TAR", "GZ", "XZ", "ZST", "CBZ", "CBR"];

struct Toggle {
    label: &'static str,
    detail: &'static str,
    on: bool,
}

pub struct SettingsPage {
    section: usize,
    appearance: Appearance,
    language: Entity<Select>,
    /// Double-click extracts at once, rather than opening the archive.
    extract_on_open: bool,
    toggles: Vec<Toggle>,
}

impl EventEmitter<AppearanceChosen> for SettingsPage {}

impl SettingsPage {
    pub fn new(appearance: Appearance, cx: &mut Context<Self>) -> Self {
        SettingsPage {
            section: 0,
            appearance,
            language: cx.new(|_| Select::new(&LANGUAGES, 0, SelectLook::Field)),
            extract_on_open: false,
            toggles: vec![
                Toggle {
                    label: "Show packwing in the right-click menu",
                    detail: "Extract and compress from any folder",
                    on: true,
                },
                Toggle {
                    label: "Open the folder when extraction is done",
                    detail: "So you can find your files right away",
                    on: true,
                },
                Toggle {
                    label: "Delete the archive after extracting",
                    detail: "Only when every file extracted without errors",
                    on: false,
                },
                Toggle {
                    label: "Send anonymous crash reports",
                    detail: "No file names or contents are ever sent",
                    on: false,
                },
            ],
        }
    }

    /// Shows `a` as chosen, when it was chosen elsewhere.
    pub fn show_appearance(&mut self, a: Appearance, cx: &mut Context<Self>) {
        self.appearance = a;
        cx.notify();
    }

    fn menu(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let items = SECTIONS.iter().enumerate().map(|(i, label)| {
            let on = i == self.section;
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
                        .hover(|s| s.bg(rgb(c.hover)))
                })
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.section = i;
                    cx.notify();
                }))
                .child(*label)
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

    fn general(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let well = || {
            div()
                .flex()
                .flex_col()
                .gap(px(10.))
                .px(px(18.))
                .py(px(16.))
                .rounded(px(22.))
                .bg(rgb(c.well))
        };
        let label = |text: &'static str| {
            div()
                .text_size(px(15.))
                .font_weight(FontWeight::SEMIBOLD)
                .child(text)
        };
        let looks = [
            (Appearance::Light, "Light"),
            (Appearance::Dark, "Dark"),
            (Appearance::System, "System"),
        ]
        .into_iter()
        .map(|(a, name)| {
            segment(name, name, a == self.appearance).on_click(cx.listener(
                move |this, _, _, cx| {
                    this.appearance = a;
                    cx.emit(AppearanceChosen(a));
                    cx.notify();
                },
            ))
        });
        let open = |id: &'static str, extract: bool, title: &'static str, detail: &'static str| {
            let on = self.extract_on_open == extract;
            option_card(id, on)
                .px(px(14.))
                .py(px(12.))
                .rounded(px(18.))
                .when(!on, |d| d.bg(rgb(c.surface)))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.extract_on_open = extract;
                    cx.notify();
                }))
                .child(titled(title, detail, on))
        };
        let toggles = self.toggles.iter().enumerate().map(|(i, t)| {
            div()
                .flex()
                .items_center()
                .gap(px(16.))
                .min_h(px(60.))
                .px(px(18.))
                .py(px(8.))
                .rounded(px(18.))
                .bg(rgb(c.well))
                .child(
                    div().flex_1().min_w_0().child(label(t.label)).child(
                        div()
                            .text_size(px(13.))
                            .text_color(rgb(c.muted))
                            .child(t.detail),
                    ),
                )
                .child(
                    switch(("toggle", i), t.on).on_click(cx.listener(move |this, _, _, cx| {
                        this.toggles[i].on = !this.toggles[i].on;
                        cx.notify();
                    })),
                )
        });
        div()
            .flex()
            .flex_col()
            .gap(px(14.))
            .child(
                div()
                    .grid()
                    .grid_cols(2)
                    .gap(px(12.))
                    .child(well().child(label("Language")).child(self.language.clone()))
                    .child(well().child(label("Appearance")).child(segments(looks))),
            )
            .child(
                well().child(label("When I double-click an archive")).child(
                    div()
                        .grid()
                        .grid_cols(2)
                        .gap(px(8.))
                        .child(open(
                            "look-inside",
                            false,
                            "Look inside first",
                            "Preview files, then decide",
                        ))
                        .child(open(
                            "extract-at-once",
                            true,
                            "Extract right away",
                            "Into a new folder next to it",
                        )),
                ),
            )
            .child(div().flex().flex_col().gap(px(6.)).children(toggles))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .px(px(18.))
                    .py(px(14.))
                    .rounded(px(22.))
                    .bg(rgb(c.well))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .gap(px(8.))
                            .child(label("Open these with packwing"))
                            .child(
                                div()
                                    .flex()
                                    .flex_wrap()
                                    .gap(px(6.))
                                    .children(TYPES.iter().map(|t| chip(*t, c.surface, c.text))),
                            ),
                    )
                    .child(button("types", "Choose types…", Fill::Outline, Size::Small)),
            )
    }
}

impl Render for SettingsPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let name: SharedString = SECTIONS[self.section].into();
        let body = if self.section == 0 {
            self.general(cx).into_any_element()
        } else {
            div()
                .text_size(px(15.))
                .text_color(rgb(c.muted))
                .child("Nothing to set here yet.")
                .into_any_element()
        };
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
                    .child(body),
            )
    }
}
