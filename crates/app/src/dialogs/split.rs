//! Split into parts: a file cut beside itself as 7-Zip cuts one, `name.001`,
//! `name.002` and on, each of the size chosen but the last.

use packwing_i18n::tr;
use std::path::PathBuf;

use gpui::{Context, Entity, EventEmitter, Render, Subscription, Window, div, prelude::*, rgb};

use super::create::{UNITS, part_size};
use super::{DialogEvent, Job, Verb, footer, header};
use crate::colors::colors;
use crate::input::{Changed, TextInput};
use crate::model::{count, size};
use crate::text_size::ui;
use crate::widgets::{
    Chosen, Fill, Select, SelectLook, Size, Tone, button, icon_button, note, tonal_tint,
};

/// The sizes offered: the last takes one typed in.
fn sizes() -> [String; 3] {
    [
        tr!("25 MB (email)"),
        tr!("4 GB (USB stick)"),
        tr!("Custom size…"),
    ]
}
const CUSTOM: usize = 2;

pub struct SplitDialog {
    path: PathBuf,
    name: String,
    /// The file's size, and where it is, as the user knows it.
    bytes: u64,
    shown: String,
    sizes: Entity<Select>,
    part: Entity<TextInput>,
    unit: Entity<Select>,
    /// Split was pressed with a size that will not do.
    checked: bool,
    _follow: Vec<Subscription>,
}

impl EventEmitter<DialogEvent> for SplitDialog {}

impl SplitDialog {
    /// For the file at `path`, of `bytes`, in the folder the user knows as `shown`.
    pub fn new(path: PathBuf, bytes: u64, shown: String, cx: &mut Context<Self>) -> Self {
        let sizes = cx.new(|_| Select::new(&sizes(), 0, SelectLook::Pill).marked("part-size"));
        let part = cx.new(|cx| TextInput::new(tr!("Size"), cx));
        let unit =
            cx.new(|_| Select::new(&UNITS.map(|(u, _)| u), 0, SelectLook::Pill).marked("unit"));
        let follow = vec![
            cx.subscribe(&sizes, |_, _, _: &Chosen, cx| cx.notify()),
            cx.subscribe(&part, |_, _, _: &Changed, cx| cx.notify()),
            cx.subscribe(&unit, |_, _, _: &Chosen, cx| cx.notify()),
        ];
        let name = path
            .file_name()
            .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
        SplitDialog {
            path,
            name,
            bytes,
            shown,
            sizes,
            part,
            unit,
            checked: false,
            _follow: follow,
        }
    }

    fn custom(&self, cx: &gpui::App) -> bool {
        self.sizes.read(cx).chosen() == CUSTOM
    }

    /// Each part's size, if one that will do is chosen.
    fn part(&self, cx: &gpui::App) -> Option<u64> {
        match self.sizes.read(cx).chosen() {
            0 => Some(25_000_000),
            1 => Some(4_000_000_000),
            _ => {
                let unit = UNITS[self.unit.read(cx).chosen().min(UNITS.len() - 1)].1;
                part_size(self.part.read(cx).text(), unit)
            }
        }
    }

    fn start(&mut self, cx: &mut Context<Self>) {
        let Some(part) = self.part(cx) else {
            self.checked = true;
            cx.notify();
            return;
        };
        let parts = self.bytes.div_ceil(part).max(1);
        let mut job = Job::shown(
            Verb::Split,
            self.name.clone().into(),
            Some(self.shown.clone().into()),
            parts,
            self.bytes,
        );
        job.split = Some((self.path.clone(), part));
        cx.emit(DialogEvent::Start(job));
    }
}

impl Render for SplitDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let custom = self.custom(cx);
        let typed = !self.part.read(cx).text().trim().is_empty();
        let chosen = self.part(cx);
        let bad = custom && chosen.is_none() && (typed || self.checked);
        let close = icon_button("close", "x", 40.)
            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close)));
        super::card()
            .w(ui(560.))
            .child(header(
                "archive",
                tonal_tint(),
                tr!("Split into parts"),
                format!("{} · {}", self.name, size(self.bytes)),
                Some(close),
            ))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(ui(12.))
                    .px(ui(16.))
                    .py(ui(14.))
                    .rounded(ui(20.))
                    .bg(rgb(c.well))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(ui(12.))
                            .child(div().flex_1().text_size(ui(14.)).child(tr!("Each part")))
                            .child(self.sizes.clone()),
                    )
                    .when(custom, |d| {
                        d.child(
                            div()
                                .flex()
                                .items_center()
                                .justify_end()
                                .gap(ui(8.))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .w(ui(120.))
                                        .h(ui(40.))
                                        .px(ui(14.))
                                        .rounded_full()
                                        .border_1()
                                        .border_color(rgb(if bad {
                                            c.warn_text
                                        } else {
                                            c.outline
                                        }))
                                        .bg(rgb(c.surface))
                                        .text_size(ui(14.))
                                        .child(self.part.clone()),
                                )
                                .child(self.unit.clone()),
                        )
                    })
                    .when_some(chosen, |d, part| {
                        let parts = self.bytes.div_ceil(part).max(1);
                        d.child(div().text_size(ui(13.)).text_color(rgb(c.muted)).child(tr!(
                            "{count} parts beside it, {first} and on. Join puts them back together, and packwing and 7-Zip open an archive cut so as it is.",
                            count = count(parts),
                            first = format!("{}.001", self.name)
                        )))
                    }),
            )
            .when(bad, |d| {
                d.child(
                    note(Tone::Warn)
                        .child(tr!("A part takes a size of 1 MB or more, such as 100 MB.")),
                )
            })
            .child(
                footer()
                    .justify_end()
                    .child(
                        button("cancel", tr!("Cancel"), Fill::Outline, Size::Medium)
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close))),
                    )
                    .child(
                        button("split", tr!("Split"), Fill::Accent, Size::Medium)
                            .px(ui(28.))
                            .on_click(cx.listener(|this, _, _, cx| this.start(cx))),
                    ),
            )
    }
}
