//! Compressing: the format new archives get and how hard they are squeezed,
//! as New archive starts, and what is left out of them.

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px, rgb};
use packwing_i18n::tr;

use super::{group, heading, rows, section, setting};
use crate::colors::colors;
use crate::dialogs::{Format, LEVELS, Level, level_name};
use crate::input::{Changed, TextInput};
use crate::model::size;
use crate::prefs::{change, default_extensions, extensions_in, prefs};
use crate::widgets::{Chosen, Fill, Select, SelectLook, Size, button, segment, segments, switch};

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
    store_compressed: bool,
    /// The extensions stored as they are, typed.
    extensions: Entity<TextInput>,
    /// Whether they are other than the known list, which Reset brings back.
    extensions_changed: bool,
    _extensions_typed: Subscription,
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
        let shown = kept.stored_extensions.clone();
        let extensions_changed = shown.is_some();
        let extensions = cx.new(|cx| {
            let mut field =
                TextInput::said(|| tr!("Extensions, apart by spaces"), cx).multiline(2, 6);
            let text = shown.unwrap_or_else(default_extensions).join(" ");
            field.set_text(&text, 0..0, cx);
            field
        });
        let extensions_typed =
            cx.subscribe(&extensions, |this: &mut Self, field, _: &Changed, cx| {
                let typed = extensions_in(field.read(cx).text());
                // The known list, typed back, follows it again as it grows.
                let kept = (typed != default_extensions()).then_some(typed);
                this.extensions_changed = kept.is_some();
                change(cx, |p| p.stored_extensions = kept);
                cx.notify();
            });
        Compressing {
            format,
            _format_chosen: format_chosen,
            stream_above,
            _stream_above_chosen: stream_above_chosen,
            level: kept.level,
            skip_mac_files: kept.skip_mac_files,
            hide_names: kept.hide_names,
            store_compressed: kept.store_compressed,
            extensions,
            extensions_changed,
            _extensions_typed: extensions_typed,
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
                let (skip, hide, store) =
                    (this.skip_mac_files, this.hide_names, this.store_compressed);
                change(cx, |p| {
                    p.skip_mac_files = skip;
                    p.hide_names = hide;
                    p.store_compressed = store;
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
                        if self.level == Level::Store {
                            tr!("Store puts the files of ZIP and 7z archives in as they are. tar formats take Lowest, as .tar alone has no compression.")
                        } else {
                            tr!("Higher makes smaller archives, which take longer to make, not to open.")
                        },
                    ))
                    .child(segments(levels)),
            )
            .child(setting(
                tr!("Write straight to a storage above"),
                tr!("Larger files take no room on this computer, but a lost connection means compressing again. S3 takes tar and ZIP this way, SFTP every format too."),
                self.stream_above.clone(),
            ))
            .child(setting(
                tr!("Don’t compress what already is"),
                tr!("For ZIP and 7z: pictures, video, sound and archives go in as they are, sooner and no larger"),
                toggle("store-compressed", self.store_compressed, |s| {
                    s.store_compressed = !s.store_compressed
                }),
            ))
            .when(self.store_compressed, |d| {
                d.child(
                    group()
                        .child(heading(
                            tr!("What goes in as it is"),
                            tr!("Files whose names end in these, whatever their case"),
                        ))
                        .child(
                            div()
                                .px(px(16.))
                                .py(px(10.))
                                .rounded(px(14.))
                                .bg(rgb(colors().field))
                                .text_size(px(14.))
                                .child(self.extensions.clone()),
                        )
                        .when(self.extensions_changed, |d| {
                            d.child(div().flex().justify_end().child(
                                button(
                                    "extensions-reset",
                                    tr!("Reset to default"),
                                    Fill::Outline,
                                    Size::Small,
                                )
                                .on_click(cx.listener(|this, _, _, cx| {
                                    let known = default_extensions().join(" ");
                                    this.extensions
                                        .update(cx, |f, cx| f.set_text(&known, 0..0, cx));
                                })),
                            ))
                        }),
                )
            })
            .child(rows([
                setting(
                    tr!("Leave out Mac-only files"),
                    tr!(".DS_Store, __MACOSX and resource forks, clutter anywhere but a Mac"),
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
