//! Compressing: the format new archives get and how hard they are squeezed,
//! as New archive starts, and what is left out of them.

use gpui::{Context, Entity, Render, Window, prelude::*};

use super::{group, heading, rows, section, setting};
use crate::dialogs::{Format, LEVELS, Level};
use crate::widgets::{Select, SelectLook, segment, segments, switch};

pub struct Compressing {
    format: Entity<Select>,
    level: Level,
    /// Leave out what only a Mac uses: `.DS_Store`, `__MACOSX`, resource forks.
    skip_mac_files: bool,
    /// Write names in UTF-8 and mark them so.
    utf8_names: bool,
    hide_names: bool,
}

impl Compressing {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Compressing {
            format: cx.new(|_| Select::new(&Format::ALL.map(Format::ext), 0, SelectLook::Pill)),
            level: Level::Medium,
            skip_mac_files: true,
            utf8_names: true,
            hide_names: true,
        }
    }
}

impl Render for Compressing {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
            .child(setting(
                "Format of new archives",
                "New archive starts with it; you can pick another there.",
                self.format.clone(),
            ))
            .child(
                group()
                    .child(heading(
                        "Compression level",
                        "Higher makes smaller archives, which take longer to make, not to open.",
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
