//! Extracting: where the files go, what happens to a file already there, how
//! old archives' names are read, and how big an archive may unpack.

use gpui::{Context, Entity, Render, Window, div, prelude::*, px, rgb};

use super::{group, heading, rows, section, setting};
use crate::colors::colors;
use crate::dialogs::ENCODINGS;
use crate::widgets::{Select, SelectLook, option_card, switch, titled};

const EXISTING: [&str; 4] = ["Ask me", "Keep both", "Replace it", "Skip it"];

/// The most an archive may unpack to before extracting stops: what stops an
/// archive bomb, a small file that unpacks to fill the disk.
const LIMITS: [&str; 4] = ["10 GB", "100 GB", "1 TB", "No limit"];

pub struct Extracting {
    /// Into a new folder named after the archive, rather than beside it.
    new_folder: bool,
    existing: Entity<Select>,
    names: Entity<Select>,
    limit: Entity<Select>,
    keep_folders: bool,
    keep_dates: bool,
}

impl Extracting {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Extracting {
            new_folder: true,
            existing: cx.new(|_| Select::new(&EXISTING, 0, SelectLook::Pill)),
            names: cx.new(|_| Select::new(&ENCODINGS, 0, SelectLook::Pill)),
            limit: cx.new(|_| Select::new(&LIMITS, 1, SelectLook::Pill)),
            keep_folders: true,
            keep_dates: true,
        }
    }
}

impl Render for Extracting {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let place =
            |id: &'static str, new_folder: bool, title: &'static str, detail: &'static str| {
                let on = self.new_folder == new_folder;
                option_card(id, on)
                    .px(px(14.))
                    .py(px(12.))
                    .rounded(px(18.))
                    .when(!on, |d| d.bg(rgb(c.surface)))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.new_folder = new_folder;
                        cx.notify();
                    }))
                    .child(titled(title, detail, on))
            };
        section()
            .child(
                group()
                    .child(heading(
                        "Where the files go",
                        "When you extract without choosing, from the right-click menu or a \
                         double-click.",
                    ))
                    .child(
                        div()
                            .grid()
                            .grid_cols(2)
                            .gap(px(8.))
                            .child(place(
                                "into-new-folder",
                                true,
                                "A new folder next to it",
                                "Named after the archive",
                            ))
                            .child(place(
                                "beside",
                                false,
                                "Next to the archive",
                                "Straight into its folder",
                            )),
                    ),
            )
            .child(rows([
                setting(
                    "When a file is already there",
                    "A file of the same name in the folder the files go to",
                    self.existing.clone(),
                ),
                setting(
                    "File names in older archives",
                    "When an archive doesn’t say how its names are written",
                    self.names.clone(),
                ),
                setting(
                    "Largest an archive may unpack to",
                    "Stops archive bombs: small files that unpack to fill the disk",
                    self.limit.clone(),
                ),
            ]))
            .child(rows([
                setting(
                    "Keep the folders inside the archive",
                    "Off puts every file straight into one folder",
                    switch("keep-folders", self.keep_folders).on_click(cx.listener(
                        |this, _, _, cx| {
                            this.keep_folders = !this.keep_folders;
                            cx.notify();
                        },
                    )),
                ),
                setting(
                    "Keep the files’ dates",
                    "The dates they had when they were packed, not today’s",
                    switch("keep-dates", self.keep_dates).on_click(cx.listener(
                        |this, _, _, cx| {
                            this.keep_dates = !this.keep_dates;
                            cx.notify();
                        },
                    )),
                ),
            ]))
    }
}
