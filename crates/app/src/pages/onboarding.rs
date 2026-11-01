//! The first run: a welcome, and what packwing may set up on the computer,
//! each with a switch.

use gpui::{Context, EventEmitter, FontWeight, Render, Window, div, prelude::*, px, relative, rgb};

use super::{panel, title};
use crate::colors::{Tint, colors};
use crate::widgets::{Fill, Size, accent_tint, button, logo, switch, tile};

/// Get started was pressed.
pub struct Done {
    /// Put packwing in the right-click menu.
    pub right_click: bool,
}

struct Step {
    label: &'static str,
    detail: &'static str,
    on: bool,
}

pub struct Onboarding {
    steps: Vec<Step>,
}

impl EventEmitter<Done> for Onboarding {}

impl Onboarding {
    pub fn new() -> Self {
        Onboarding {
            steps: vec![
                // The first: Get started says whether it is on.
                Step {
                    label: "Add to the right-click menu",
                    detail: "Extract or compress from any folder.",
                    on: true,
                },
            ],
        }
    }
}

impl Render for Onboarding {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let welcome = div()
            .w(px(340.))
            .flex_shrink_0()
            .flex()
            .flex_col()
            .gap(px(18.))
            .px(px(32.))
            .py(px(36.))
            .rounded(px(28.))
            .bg(rgb(c.tonal))
            .text_color(rgb(c.on_tonal))
            .child(tile(72., 24., accent_tint()).child(logo(36., accent_tint())))
            .child(title("Welcome to packwing", 34.).line_height(relative(1.15)))
            .child(div().text_size(px(17.)).line_height(relative(1.5)).child(
                "Open any archive, make smaller files to share, and stay safe from \
                         harmful downloads.",
            ))
            .child(div().flex_1());
        let steps = self.steps.iter().enumerate().map(|(i, s)| {
            let tint = if s.on {
                Tint {
                    ground: c.tonal,
                    ink: c.on_tonal,
                }
            } else {
                Tint {
                    ground: c.field,
                    ink: c.muted,
                }
            };
            div()
                .flex()
                .items_start()
                .gap(px(14.))
                .px(px(16.))
                .py(px(14.))
                .rounded(px(20.))
                .bg(rgb(c.well))
                .child(
                    tile(40., 14., tint)
                        .text_size(px(15.))
                        .font_weight(FontWeight::BOLD)
                        .child((i + 1).to_string()),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .gap(px(3.))
                        .child(
                            div()
                                .text_size(px(16.))
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(s.label),
                        )
                        .child(
                            div()
                                .text_size(px(14.))
                                .line_height(relative(1.45))
                                .text_color(rgb(c.muted))
                                .child(s.detail),
                        ),
                )
                .child(
                    div()
                        .pt(px(4.))
                        .child(switch(("step", i), s.on).on_click(cx.listener(
                            move |this, _, _, cx| {
                                this.steps[i].on = !this.steps[i].on;
                                cx.notify();
                            },
                        ))),
                )
        });
        let setup = panel()
            .flex_1()
            .min_w_0()
            .flex()
            .flex_col()
            .gap(px(14.))
            .p(px(32.))
            .child(title("Set things up", 22.).pb(px(4.)))
            .children(steps)
            .child(div().flex_1())
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .child(
                        div()
                            .flex_1()
                            .text_size(px(13.))
                            .text_color(rgb(c.muted))
                            .child("You can change these later in Settings."),
                    )
                    .child(
                        button("get-started", "Get started", Fill::Accent, Size::Large)
                            .px(px(32.))
                            .on_click(cx.listener(|this, _, _, cx| {
                                cx.emit(Done {
                                    right_click: this.steps[0].on,
                                })
                            })),
                    ),
            );
        div().size_full().flex().justify_center().p(px(16.)).child(
            div()
                .w_full()
                .max_w(px(1100.))
                .flex()
                .gap(px(16.))
                .child(welcome)
                .child(setup),
        )
    }
}
