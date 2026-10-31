//! A job under way: how far it is, how long is left, and how it ended if it
//! failed.
//!
//! A job with work runs it in the background through packwing-core, and the
//! dialog shows what it reports. One without (a repair, the screenshots'
//! sample) counts up at a made-up speed, so the screen can be
//! seen moving, and closes itself at the end.

use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use gpui::{
    Context, EventEmitter, FontWeight, Render, Task, Window, div, prelude::*, px, relative, rgb,
};
use packwing_core::{Cancel, Control, Error, Progress};

use super::{DialogEvent, Job, Verb, asked_for, card, footer, header};
use crate::colors::colors;
use crate::model::{count, size};
use crate::widgets::{Fill, Size, Tone, button, check_row, icon, note, tonal_tint};

/// The made-up speed, in bytes a second.
const SPEED: u64 = 38_000_000;
/// How often the count moves.
const TICK: Duration = Duration::from_millis(100);

/// A job that runs: what it has reported, and what stops it.
struct Running {
    progress: Arc<Mutex<Progress>>,
    cancel: Cancel,
    started: Instant,
    _work: Task<()>,
}

pub struct ProgressDialog {
    job: Job,
    /// How much is done, 0 to 1.
    done: f64,
    paused: bool,
    open_folder: bool,
    running: Option<Running>,
    /// Why it stopped, if it failed.
    failed: Option<String>,
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
        let running = (!job.work.is_empty() || job.create.is_some()).then(|| Self::run(&job, cx));
        ProgressDialog {
            job,
            done: 0.,
            paused: false,
            open_folder: true,
            running,
            failed: None,
            _ticking: ticking,
        }
    }

    /// Starts `job`'s work in the background: the new archive, or each archive
    /// in turn. It ends with the new archive's place, if it made one.
    fn run(job: &Job, cx: &mut Context<Self>) -> Running {
        let progress = Arc::new(Mutex::new(Progress::default()));
        let cancel = Cancel::new();
        let told = progress.clone();
        let control = Control {
            progress: Some(Arc::new(move |p: &Progress| {
                *told.lock().unwrap_or_else(PoisonError::into_inner) = *p;
            })),
            cancel: Some(cancel.clone()),
        };
        let keys = job.keys.clone();
        let (verb, work, create) = (job.verb, job.work.clone(), job.create.clone());
        let task = cx.background_executor().spawn(async move {
            if let Some(c) = create {
                return packwing_core::create(&c.paths, &c.archive, &c.how, &control).map(Some);
            }
            for (source, destination) in &work {
                match (verb, destination) {
                    (Verb::Extract, Some(to)) => {
                        packwing_core::extract(source, to, &keys, &control)?;
                    }
                    _ => {
                        packwing_core::test(source, &keys, &control)?;
                    }
                }
            }
            Ok::<Option<PathBuf>, Error>(None)
        });
        let work = cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| this.finish(result, cx));
        });
        Running {
            progress,
            cancel,
            started: Instant::now(),
            _work: work,
        }
    }

    /// The job ended: done, cancelled, or failed.
    fn finish(&mut self, result: Result<Option<PathBuf>, Error>, cx: &mut Context<Self>) {
        match result {
            Ok(made) => {
                if let Some(made) = made.filter(|_| self.open_folder) {
                    cx.reveal_path(&made);
                } else if self.open_folder
                    && self.job.verb == Verb::Extract
                    && let Some((_, Some(to))) = self.job.work.first()
                {
                    cx.open_with_system(to);
                }
                cx.emit(DialogEvent::Close);
            }
            Err(Error::Cancelled) => cx.emit(DialogEvent::Close),
            // One archive that needs a passphrase or password: asked for,
            // and run again. Several would extract those before it twice.
            Err(e)
                if self.job.work.len() == 1
                    && let Some((asked, again)) = asked_for(&e, &self.job.keys) =>
            {
                let again = again.then(|| e.to_string());
                cx.emit(DialogEvent::Ask(self.job.clone(), asked, again));
            }
            Err(e) => {
                self.failed = Some(e.to_string());
                cx.notify();
            }
        }
    }

    /// Puts the count at `done` (0 to 1), as if it got there.
    #[cfg(all(test, feature = "shots"))]
    pub fn set_done(&mut self, done: f64, cx: &mut Context<Self>) {
        self.done = done.clamp(0., 1.);
        cx.notify();
    }

    /// Shows the job as failed, for `why`.
    #[cfg(all(test, feature = "shots"))]
    pub fn set_failed(&mut self, why: String, cx: &mut Context<Self>) {
        self.failed = Some(why);
        cx.notify();
    }

    /// What the job has reported, if it runs.
    fn reported(&self) -> Option<Progress> {
        let r = self.running.as_ref()?;
        Some(*r.progress.lock().unwrap_or_else(PoisonError::into_inner))
    }

    /// Files done and in all, bytes done and in all: as reported, or as
    /// counted up. A total the job has not told yet is the job's own guess.
    fn counts(&self) -> (u64, u64, u64, u64) {
        let j = &self.job;
        match self.reported() {
            Some(p) => {
                let files = if p.total_entries > 0 {
                    p.total_entries
                } else {
                    j.files
                };
                let bytes = if p.total_bytes > 0 {
                    p.total_bytes
                } else {
                    j.bytes
                };
                (p.entries, files, p.bytes, bytes)
            }
            None => (
                (j.files as f64 * self.done).round() as u64,
                j.files,
                (j.bytes as f64 * self.done) as u64,
                j.bytes,
            ),
        }
    }

    /// Bytes a second: measured, or the made-up speed.
    fn speed(&self) -> u64 {
        match (&self.running, self.reported()) {
            (Some(r), Some(p)) => {
                let secs = r.started.elapsed().as_secs_f64();
                if secs < 0.5 {
                    0
                } else {
                    (p.bytes as f64 / secs) as u64
                }
            }
            _ => SPEED,
        }
    }

    /// One step: of the made-up job, or a look at the real one. True when
    /// the dialog has no more to count.
    fn tick(&mut self, cx: &mut Context<Self>) -> bool {
        if self.failed.is_some() {
            return true;
        }
        if self.running.is_some() {
            let (_, _, bytes, total) = self.counts();
            self.done = if total == 0 {
                0.
            } else {
                (bytes as f64 / total as f64).min(1.)
            };
            cx.notify();
            return false;
        }
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

    fn cancel(&mut self, cx: &mut Context<Self>) {
        match &self.running {
            // It stops at its next step, and closes the dialog then.
            Some(r) if self.failed.is_none() => r.cancel.cancel(),
            _ => cx.emit(DialogEvent::Close),
        }
    }

    fn title(&self) -> &'static str {
        match (self.job.verb, self.failed.is_some()) {
            (Verb::Extract, false) => "Extracting…",
            (Verb::Create, false) => "Creating…",
            (Verb::Repair, false) => "Repairing…",
            (Verb::Test, false) => "Testing…",
            (Verb::Extract, true) => "Couldn’t extract",
            (Verb::Create, true) => "Couldn’t create the archive",
            (Verb::Repair, true) => "Couldn’t repair",
            (Verb::Test, true) => "The test failed",
        }
    }

    fn left(&self) -> String {
        if self.paused {
            return "Paused".into();
        }
        let speed = self.speed();
        if speed == 0 {
            return "Starting…".into();
        }
        let (_, _, bytes, total) = self.counts();
        let secs = (total.saturating_sub(bytes) as f64 / speed as f64).round() as u64;
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
        let header = header(icon_name, tonal_tint(), self.title(), detail, None);
        if let Some(why) = &self.failed {
            return card()
                .gap(px(22.))
                .child(header)
                .child(note(Tone::Warn).child(why.clone()))
                .child(
                    footer().child(div().flex_1()).child(
                        button("close", "Close", Fill::Accent, Size::Medium)
                            .px(px(28.))
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close))),
                    ),
                );
        }
        let real = self.running.is_some();
        let (files, all_files, bytes, all_bytes) = self.counts();
        let percent = (self.done * 100.).floor() as u64;
        // The file it is on, made up from the count: only the samples have
        // one to show.
        let current = format!("Ảnh hiện trạng / IMG_{}.jpg", 2000 + files % 400);
        let speed = self.speed();
        card()
            .gap(px(22.))
            .child(header)
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
                                count(all_files),
                                size(bytes),
                                size(all_bytes)
                            ))
                            .child(if self.paused || speed == 0 {
                                "—".to_string()
                            } else {
                                format!("{}/s", size(speed))
                            }),
                    ),
            )
            .when(!real, |d| {
                d.child(
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
            })
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
                    // A job that runs cannot go on without its dialog, or
                    // pause, yet.
                    .when(!real, |d| {
                        d.child(
                            button("background", "Run in background", Fill::Tonal, Size::Medium)
                                .px(px(20.))
                                .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close))),
                        )
                    })
                    .child(div().flex_1())
                    .when(!real, |d| {
                        d.child(
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
                    })
                    .child(
                        button("cancel", "Cancel", Fill::Outline, Size::Medium)
                            .on_click(cx.listener(|this, _, _, cx| this.cancel(cx))),
                    ),
            )
    }
}
