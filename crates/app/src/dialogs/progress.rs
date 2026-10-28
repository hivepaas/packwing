//! A job under way: how far it is, how long is left, the file it is on.
//!
//! Nothing runs yet: the dialog counts up at a made-up speed, so the screen
//! can be seen moving, and closes itself at the end.

use std::time::Duration;

use gpui::{
    Context, EventEmitter, FontWeight, Render, Task, Window, div, prelude::*, px, relative, rgb,
};

use super::{DialogEvent, Job, Verb, card, footer, header};
use crate::colors::colors;
use crate::demo::{count, size};
use crate::widgets::{Fill, Size, button, check_row, icon, tonal_tint};

/// The made-up speed, in bytes a second.
const SPEED: u64 = 38_000_000;
/// How often the count moves.
const TICK: Duration = Duration::from_millis(100);

pub struct ProgressDialog {
    job: Job,
    /// How much is done, 0 to 1.
    done: f64,
    paused: bool,
    open_folder: bool,
    _ticking: Task<()>,
}

impl EventEmitter<DialogEvent> for ProgressDialog {}

impl ProgressDialog {
    pub fn new(job: Job, cx: &mut Context<Self>) -> Self {
        let ticking = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(TICK).await;
                match this.update(cx, |this, cx| this.tick(cx)) {
                    Ok(false) => {}
                    _ => break,
                }
            }
        });
        ProgressDialog {
            job,
            done: 0.,
            paused: false,
            open_folder: true,
            _ticking: ticking,
        }
    }

    /// Puts the count at `done` (0 to 1), as if it got there.
    #[cfg(all(test, feature = "shots"))]
    pub fn set_done(&mut self, done: f64, cx: &mut Context<Self>) {
        self.done = done.clamp(0., 1.);
        cx.notify();
    }

    /// One step of the made-up job; true when it is over.
    fn tick(&mut self, cx: &mut Context<Self>) -> bool {
        if self.paused {
            return false;
        }
        let step = (SPEED as f64 * TICK.as_secs_f64()) / self.job.bytes.max(1) as f64;
        self.done = (self.done + step).min(1.);
        if self.done >= 1. {
            cx.emit(DialogEvent::Close);
            return true;
        }
        cx.notify();
        false
    }

    fn title(&self) -> &'static str {
        match self.job.verb {
            Verb::Extract => "Extracting…",
            Verb::Create => "Creating…",
            Verb::Repair => "Repairing…",
            Verb::Test => "Testing…",
        }
    }

    fn left(&self) -> String {
        if self.paused {
            return "Paused".into();
        }
        let secs = ((1. - self.done) * self.job.bytes as f64 / SPEED as f64).round() as u64;
        match secs {
            0..=4 => "Almost done".into(),
            5..=59 => format!("About {secs} seconds left"),
            60..=119 => "About a minute left".into(),
            _ => format!("About {} minutes left", secs.div_ceil(60)),
        }
    }
}

impl Render for ProgressDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let j = &self.job;
        let detail = match &j.to {
            Some(to) => format!("{} → {}", j.name, to),
            None => format!("{} · checking that every file reads", j.name),
        };
        let icon_name = match j.verb {
            Verb::Extract => "extract",
            Verb::Create => "logo",
            Verb::Repair | Verb::Test => "shield",
        };
        let files = (j.files as f64 * self.done).round() as u64;
        let bytes = (j.bytes as f64 * self.done) as u64;
        let percent = (self.done * 100.).floor() as u64;
        // The file it is on: made up from the count.
        let current = format!("Ảnh hiện trạng / IMG_{}.jpg", 2000 + files % 400);
        card()
            .gap(px(22.))
            .child(header(icon_name, tonal_tint(), self.title(), detail, None))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(10.))
                    .child(
                        div()
                            .flex()
                            .items_end()
                            .justify_between()
                            .gap(px(12.))
                            .child(
                                div()
                                    .text_size(px(40.))
                                    .line_height(relative(1.1))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(c.on_tonal))
                                    .child(format!("{percent}%")),
                            )
                            .child(
                                div()
                                    .pb(px(6.))
                                    .text_size(px(15.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(self.left()),
                            ),
                    )
                    .child(
                        div().h(px(14.)).rounded_full().bg(rgb(c.field)).child(
                            div()
                                .h_full()
                                .w(relative(self.done as f32))
                                .rounded_full()
                                .bg(rgb(c.accent)),
                        ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .justify_between()
                            .gap_x(px(16.))
                            .gap_y(px(4.))
                            .text_size(px(14.))
                            .text_color(rgb(c.muted))
                            .child(format!(
                                "{} of {} files · {} of {}",
                                count(files),
                                count(j.files),
                                size(bytes),
                                size(j.bytes)
                            ))
                            .child(if self.paused {
                                "—".to_string()
                            } else {
                                format!("{}/s", size(SPEED))
                            }),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .px(px(16.))
                    .py(px(12.))
                    .rounded(px(18.))
                    .bg(rgb(c.well))
                    .child(icon("image", 20., c.accent))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .text_size(px(14.))
                            .child(current),
                    ),
            )
            .when(j.verb != Verb::Test, |d| {
                d.child(
                    check_row(
                        "open-folder",
                        self.open_folder,
                        "Open the folder when it’s done",
                    )
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.open_folder = !this.open_folder;
                        cx.notify();
                    })),
                )
            })
            .child(
                footer()
                    .child(
                        button("background", "Run in background", Fill::Tonal, Size::Medium)
                            .px(px(20.))
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close))),
                    )
                    .child(div().flex_1())
                    .child(
                        button(
                            "pause",
                            if self.paused { "Resume" } else { "Pause" },
                            Fill::Text,
                            Size::Medium,
                        )
                        .px(px(20.))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.paused = !this.paused;
                            cx.notify();
                        })),
                    )
                    .child(
                        button("cancel", "Cancel", Fill::Outline, Size::Medium)
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close))),
                    ),
            )
    }
}
