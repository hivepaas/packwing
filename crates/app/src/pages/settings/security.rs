//! Security: what packwing checks and keeps when it extracts.

use gpui::{Context, Render, Window, prelude::*};
use packwing_i18n::tr;

use super::{rows, section, setting};
use crate::widgets::switch;

/// What a switch sets.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Sets {
    KeepMark,
    AskOpen,
}

struct Toggle {
    sets: Sets,
    label: String,
    detail: String,
    on: bool,
}

pub struct Security {
    /// The settings the user may change.
    toggles: Vec<Toggle>,
}

impl Security {
    pub fn new(keep_mark: bool, ask_open: bool) -> Self {
        let toggles = vec![
            Toggle {
                sets: Sets::KeepMark,
                label: tr!("Keep the “downloaded from the Internet” mark"),
                detail: tr!(
                    "The system warns before running files that came from a downloaded \
                         archive"
                ),
                on: keep_mark,
            },
            Toggle {
                sets: Sets::AskOpen,
                label: tr!("Ask before opening files without extracting"),
                detail: tr!(
                    "Opening a file inside an archive puts a copy in a temporary folder. \
                         A program always asks."
                ),
                on: ask_open,
            },
        ];
        Security { toggles }
    }
}

impl Render for Security {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let open = self.toggles.iter().enumerate().map(|(i, t)| {
            setting(
                t.label.clone(),
                t.detail.clone(),
                switch(("security", i), t.on).on_click(cx.listener(move |this, _, _, cx| {
                    this.toggles[i].on = !this.toggles[i].on;
                    if this.toggles[i].sets == Sets::KeepMark {
                        let on = this.toggles[i].on;
                        crate::prefs::change(cx, |p| p.keep_mark = on);
                    }
                    if this.toggles[i].sets == Sets::AskOpen {
                        let on = this.toggles[i].on;
                        crate::prefs::change(cx, |p| p.ask_before_open = on);
                    }
                    cx.notify();
                })),
            )
        });
        section().child(rows(open))
    }
}
