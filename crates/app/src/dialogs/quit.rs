//! Closing the window while jobs run: they go on, or stop and the window closes once
//! they have.

use gpui::{Context, EventEmitter, Render, Window, div, prelude::*, rgb};
use packwing_i18n::{tr, tr_n};

use super::{DialogEvent, footer, header};
use crate::colors::colors;
use crate::text_size::ui;
use crate::widgets::{Fill, Size, button, tonal_tint};

pub struct QuitDialog {
    /// How many jobs run, or wait for their turn.
    jobs: usize,
}

impl EventEmitter<DialogEvent> for QuitDialog {}

impl QuitDialog {
    pub fn new(jobs: usize) -> Self {
        QuitDialog { jobs }
    }
}

impl Render for QuitDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        super::card()
            .w(ui(520.))
            .child(header(
                "warning",
                tonal_tint(),
                tr_n!(
                    self.jobs as u64,
                    "{n} job is still running",
                    "{n} jobs are still running"
                ),
                tr!("Closing packwing stops them"),
                None,
            ))
            .child(div().text_size(ui(14.)).text_color(rgb(c.muted)).child(tr!(
                "What they have not finished stays unfinished: an archive being made is \
                 removed, and an extraction stays as far as it got."
            )))
            .child(
                footer()
                    .justify_end()
                    .child(
                        button("quit-keep", tr!("Keep running"), Fill::Text, Size::Medium)
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close))),
                    )
                    .child(
                        button(
                            "quit-stop",
                            tr!("Stop them and quit"),
                            Fill::Accent,
                            Size::Medium,
                        )
                        .px(ui(28.))
                        .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Quit))),
                    ),
            )
    }
}
