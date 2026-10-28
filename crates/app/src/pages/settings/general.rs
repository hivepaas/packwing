//! General: the language, light or dark, what a double-click does, and the
//! types packwing opens.

use gpui::{Context, Entity, EventEmitter, Render, Window, div, prelude::*, px, rgb};

use super::{group, label, rows, section, setting};
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

pub const LANGUAGES: [&str; 3] = ["Same as system (English)", "English", "Tiếng Việt"];

/// The archive types packwing can open.
const TYPES: [&str; 9] = ["ZIP", "RAR", "7z", "TAR", "GZ", "XZ", "ZST", "CBZ", "CBR"];

struct Toggle {
    label: &'static str,
    detail: &'static str,
    on: bool,
}

pub struct General {
    appearance: Appearance,
    language: Entity<Select>,
    /// Double-click extracts at once, rather than opening the archive.
    extract_on_open: bool,
    toggles: Vec<Toggle>,
}

impl EventEmitter<AppearanceChosen> for General {}

impl General {
    pub fn new(appearance: Appearance, cx: &mut Context<Self>) -> Self {
        General {
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

    pub fn show_appearance(&mut self, a: Appearance, cx: &mut Context<Self>) {
        self.appearance = a;
        cx.notify();
    }
}

impl Render for General {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
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
            setting(
                t.label,
                t.detail,
                switch(("toggle", i), t.on).on_click(cx.listener(move |this, _, _, cx| {
                    this.toggles[i].on = !this.toggles[i].on;
                    cx.notify();
                })),
            )
        });
        section()
            .child(
                div()
                    .grid()
                    .grid_cols(2)
                    .gap(px(12.))
                    .child(
                        group()
                            .child(label("Language"))
                            .child(self.language.clone()),
                    )
                    .child(group().child(label("Appearance")).child(segments(looks))),
            )
            .child(
                group()
                    .child(label("When I double-click an archive"))
                    .child(
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
            .child(rows(toggles))
            .child(
                group()
                    .flex_row()
                    .items_center()
                    .gap(px(12.))
                    .py(px(14.))
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
