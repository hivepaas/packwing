//! Asked before a file in an archive is opened: a copy of it goes in a
//! temporary folder, and a program in it would run. A program is always asked
//! about.

use gpui::{Context, EventEmitter, Render, SharedString, Window, div, prelude::*, rgb};
use packwing_i18n::tr;

use super::{DialogEvent, Job, footer, header};
use crate::colors::colors;
use crate::text_size::ui;
use crate::widgets::{Fill, Size, Tone, button, check_row, icon_button, note, tonal_tint};

pub struct OpenFileDialog {
    job: Job,
    name: SharedString,
    program: bool,
    /// Do not ask again, unless it is a program.
    dont_ask: bool,
}

impl EventEmitter<DialogEvent> for OpenFileDialog {}

impl OpenFileDialog {
    pub fn new(job: Job, name: String, program: bool) -> OpenFileDialog {
        OpenFileDialog {
            job,
            name: name.into(),
            program,
            dont_ask: false,
        }
    }
}

impl Render for OpenFileDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let close = icon_button("close", "x", 40.)
            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close)));
        super::card()
            .w(ui(520.))
            .child(header(
                "extract",
                tonal_tint(),
                tr!("Open {name}?", name = self.name),
                self.job.name.clone(),
                Some(close),
            ))
            .child(
                div()
                    .text_size(ui(14.))
                    .text_color(rgb(c.muted))
                    .child(tr!("packwing puts a copy in a temporary folder and opens that. Changes to it do not go back in the archive.")),
            )
            .when(self.program, |d| {
                d.child(note(Tone::Warn).child(
                    tr!("This is a program: opening it runs it. Only do so if you trust where the archive came from."),
                ))
            })
            .when(!self.program, |d| {
                d.child(
                    check_row("dont-ask", self.dont_ask, tr!("Don’t ask again")).on_click(cx.listener(
                        |this, _, _, cx| {
                            this.dont_ask = !this.dont_ask;
                            cx.notify();
                        },
                    )),
                )
            })
            .child(
                footer()
                    .justify_end()
                    .child(
                        button("cancel", tr!("Cancel"), Fill::Text, Size::Medium)
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close))),
                    )
                    .child(
                        button(
                            "open",
                            if self.program { tr!("Run it") } else { tr!("Open") },
                            Fill::Accent,
                            Size::Medium,
                        )
                        .px(ui(28.))
                        .on_click(cx.listener(|this, _, _, cx| {
                            if this.dont_ask {
                                crate::prefs::change(cx, |p| p.ask_before_open = false);
                            }
                            cx.emit(DialogEvent::Start(this.job.clone()));
                        })),
                    ),
            )
    }
}
