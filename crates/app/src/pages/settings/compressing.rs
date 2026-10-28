//! Compressing: what new archives are for and how hard they are squeezed, as
//! New archive starts, and what is left out of them.

use gpui::{Context, FontWeight, Render, Window, div, prelude::*, px, rgb};

use super::{group, heading, rows, section, setting};
use crate::colors::colors;
use crate::dialogs::{LEVELS, Level, PURPOSES, Purpose};
use crate::widgets::{segment, segments, switch};

pub struct Compressing {
    purpose: Purpose,
    level: Level,
    /// Leave out what only a Mac uses: `.DS_Store`, `__MACOSX`, resource forks.
    skip_mac_files: bool,
    /// Write names in UTF-8 and mark them so.
    utf8_names: bool,
    hide_names: bool,
}

impl Compressing {
    pub fn new(_: &mut Context<Self>) -> Self {
        Compressing {
            purpose: Purpose::Share,
            level: Level::Balanced,
            skip_mac_files: true,
            utf8_names: true,
            hide_names: true,
        }
    }
}

impl Render for Compressing {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let purposes = PURPOSES.iter().map(|p| {
            let on = p.purpose == self.purpose;
            let purpose = p.purpose;
            div()
                .id(p.label)
                .flex()
                .flex_col()
                .gap(px(4.))
                .p(px(14.))
                .rounded(px(18.))
                .border_2()
                .border_color(rgb(if on { c.accent } else { c.surface }))
                .bg(rgb(if on { c.tonal } else { c.surface }))
                .text_color(rgb(if on { c.on_tonal } else { c.text }))
                .cursor_pointer()
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.purpose = purpose;
                    cx.notify();
                }))
                .child(
                    div()
                        .text_size(px(15.))
                        .font_weight(FontWeight::BOLD)
                        .child(p.label),
                )
                .child(
                    div()
                        .text_size(px(12.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .opacity(0.85)
                        .child(p.format),
                )
        });
        let levels = LEVELS.iter().map(|&(level, name)| {
            segment(name, name, level == self.level).on_click(cx.listener(move |this, _, _, cx| {
                this.level = level;
                cx.notify();
            }))
        });
        let toggle = |id: &'static str, on: bool, flip: fn(&mut Compressing)| {
            switch(id, on).on_click(cx.listener(move |this, _, _, cx| {
                flip(this);
                cx.notify();
            }))
        };
        section()
            .child(
                group()
                    .child(heading(
                        "What new archives are for",
                        "New archive starts with it; you can pick another there.",
                    ))
                    .child(div().grid().grid_cols(3).gap(px(8.)).children(purposes)),
            )
            .child(
                group()
                    .child(heading(
                        "Compression",
                        "Smaller takes longer to make, not to open.",
                    ))
                    .child(segments(levels)),
            )
            .child(rows([
                setting(
                    "Leave out Mac-only files",
                    ".DS_Store, __MACOSX and resource forks, clutter anywhere but a Mac",
                    toggle("skip-mac-files", self.skip_mac_files, |s| {
                        s.skip_mac_files = !s.skip_mac_files
                    }),
                ),
                setting(
                    "Write names that read right everywhere",
                    "UTF-8, so Vietnamese, Chinese or Japanese names open on any computer",
                    toggle("utf8-names", self.utf8_names, |s| {
                        s.utf8_names = !s.utf8_names
                    }),
                ),
                setting(
                    "Hide file names behind the password",
                    "For 7z: nobody sees what is inside without it",
                    toggle("hide-names", self.hide_names, |s| {
                        s.hide_names = !s.hide_names
                    }),
                ),
            ]))
    }
}
