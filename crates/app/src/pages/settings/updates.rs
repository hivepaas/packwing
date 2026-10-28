//! Updates: the version running, how new ones are found, and when they go in.

use gpui::{Context, Render, Window, div, prelude::*, px};

use super::{rows, section, setting};
use crate::widgets::{Fill, Size, button, segment, segments, switch};

pub struct Updates {
    automatic: bool,
    beta: bool,
    on_quit: bool,
}

impl Updates {
    pub fn new() -> Self {
        Updates {
            automatic: true,
            beta: false,
            on_quit: true,
        }
    }
}

impl Render for Updates {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let channels = [(false, "Stable"), (true, "Beta")].map(|(beta, name)| {
            segment(name, name, beta == self.beta).on_click(cx.listener(move |this, _, _, cx| {
                this.beta = beta;
                cx.notify();
            }))
        });
        section()
            .child(setting(
                format!("packwing {}", env!("CARGO_PKG_VERSION")),
                // Made up: nothing checks yet.
                "Checked today at 09:12 · this is the latest",
                button("check-now", "Check now", Fill::Outline, Size::Small),
            ))
            .child(rows([
                setting(
                    "Check for updates automatically",
                    "Once a day. Nothing about you or your files is sent",
                    switch("automatic", self.automatic).on_click(cx.listener(|this, _, _, cx| {
                        this.automatic = !this.automatic;
                        cx.notify();
                    })),
                ),
                setting(
                    "Channel",
                    "Beta gets what is new first, with more bugs",
                    div().w(px(220.)).child(segments(channels)),
                ),
                setting(
                    "Install updates when packwing quits",
                    "Otherwise you are asked first",
                    switch("on-quit", self.on_quit).on_click(cx.listener(|this, _, _, cx| {
                        this.on_quit = !this.on_quit;
                        cx.notify();
                    })),
                ),
            ]))
    }
}
