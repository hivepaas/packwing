//! Compressing: the format new archives get and how hard they are squeezed,
//! as New archive starts, and what is left out of them.

use gpui::{Context, Entity, Render, Subscription, Window, prelude::*};
use packwing_i18n::tr;

use super::{group, heading, rows, section, setting};
use crate::dialogs::{Format, LEVELS, Level, level_name};
use crate::model::size;
use crate::prefs::{change, prefs};
use crate::widgets::{Chosen, Select, SelectLook, segment, segments, switch};

/// What can be chosen for going straight to a storage, in bytes.
const STREAM_ABOVE: [u64; 5] = [0, 100_000_000, 500_000_000, 1_000_000_000, 10_000_000_000];

fn stream_above_name(bytes: u64) -> String {
    match bytes {
        0 => tr!("Always"),
        bytes => size(bytes),
    }
}

pub struct Compressing {
    format: Entity<Select>,
    _format_chosen: Subscription,
    stream_above: Entity<Select>,
    _stream_above_chosen: Subscription,
    level: Level,
    /// Leave out what only a Mac uses: `.DS_Store`, `__MACOSX`, resource forks.
    skip_mac_files: bool,
    hide_names: bool,
}

impl Compressing {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let kept = prefs(cx);
        let chosen = Format::ALL
            .iter()
            .position(|f| *f == kept.format)
            .unwrap_or(0);
        let format =
            cx.new(|_| Select::new(&Format::ALL.map(Format::ext), chosen, SelectLook::Pill));
        let format_chosen = cx.subscribe(&format, |_, select, _: &Chosen, cx| {
            let format = Format::of_option(select.read(cx).chosen());
            change(cx, |p| p.format = format);
        });
        let chosen = STREAM_ABOVE
            .iter()
            .position(|b| *b == kept.stream_above)
            .unwrap_or(2);
        let stream_above = cx.new(|_| {
            Select::new(
                &STREAM_ABOVE.map(stream_above_name),
                chosen,
                SelectLook::Pill,
            )
        });
        let stream_above_chosen = cx.subscribe(&stream_above, |_, select, _: &Chosen, cx| {
            let bytes = STREAM_ABOVE[select.read(cx).chosen().min(STREAM_ABOVE.len() - 1)];
            change(cx, |p| p.stream_above = bytes);
        });
        Compressing {
            format,
            _format_chosen: format_chosen,
            stream_above,
            _stream_above_chosen: stream_above_chosen,
            level: kept.level,
            skip_mac_files: kept.skip_mac_files,
            hide_names: kept.hide_names,
        }
    }
}

impl Render for Compressing {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let levels = LEVELS.iter().map(|&(level, name)| {
            segment(name, level_name(level), level == self.level).on_click(cx.listener(
                move |this, _, _, cx| {
                    this.level = level;
                    change(cx, |p| p.level = level);
                    cx.notify();
                },
            ))
        });
        let toggle = |id: &'static str, on: bool, flip: fn(&mut Compressing)| {
            switch(id, on).on_click(cx.listener(move |this, _, _, cx| {
                flip(this);
                let (skip, hide) = (this.skip_mac_files, this.hide_names);
                change(cx, |p| {
                    p.skip_mac_files = skip;
                    p.hide_names = hide;
                });
                cx.notify();
            }))
        };
        section()
            .child(setting(
                tr!("Format of new archives"),
                tr!("New archive starts with it; you can pick another there."),
                self.format.clone(),
            ))
            .child(
                group()
                    .child(heading(
                        tr!("Compression level"),
                        tr!(
                            "Higher makes smaller archives, which take longer to make, not to open."
                        ),
                    ))
                    .child(segments(levels)),
            )
            .child(setting(
                tr!("Write straight to a storage above"),
                tr!("Larger files take no room on this computer, but a lost connection means compressing again. S3 takes tar this way, SFTP every format."),
                self.stream_above.clone(),
            ))
            .child(rows([
                setting(
                    tr!("Leave out Mac-only files"),
                    ".DS_Store, __MACOSX and resource forks, clutter anywhere but a Mac",
                    toggle("skip-mac-files", self.skip_mac_files, |s| {
                        s.skip_mac_files = !s.skip_mac_files
                    }),
                ),
                setting(
                    tr!("Hide file names behind the password"),
                    tr!("For 7z: nobody sees what is inside without it"),
                    toggle("hide-names", self.hide_names, |s| {
                        s.hide_names = !s.hide_names
                    }),
                ),
            ]))
    }
}
