//! Jobs: those sent to run in the background, without their dialog: how far each is,
//! and how it ended. A job that runs shows its dialog again from its row; one that
//! stopped for a password asks for it when the user says; one that ended stays until
//! it is cleared. The menu has the page only while it has jobs.

use gpui::{
    Context, Entity, EventEmitter, FontWeight, Render, Subscription, Window, div, prelude::*,
    relative, rgb,
};
use packwing_core::Asked;
use packwing_i18n::{tr, tr_n};

use super::{panel, title};
use crate::colors::colors;
use crate::dialogs::{Job, ProgressDialog, State, Verb};
use crate::file_manager;
use crate::model::{count, size};
use crate::text_size::ui;
use crate::widgets::{Fill, Size, Tone, button, icon, icon_button, note, tile, tonal_tint};

/// A job on the page.
struct Behind {
    view: Entity<ProgressDialog>,
    /// What it stopped for, to be asked when the user is ready, and why again.
    asked: Option<(Job, Asked, Option<String>)>,
    /// How it ended has been on the page, shown.
    seen: bool,
    /// What it says, to the workspace.
    _events: Subscription,
    _observed: Subscription,
}

impl Behind {
    fn ended(&self, cx: &gpui::App) -> bool {
        self.asked.is_none() && self.view.read(cx).state() != State::Running
    }
}

/// What the menu says of the jobs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Summary {
    /// How many there are.
    pub jobs: usize,
    pub running: usize,
    /// One waits for the user: for a password, or with why it failed, not yet seen.
    pub waiting: bool,
    /// One ended that the page has not shown yet.
    pub unseen: bool,
}

pub enum JobsEvent {
    /// Show this job's dialog.
    Show(Entity<ProgressDialog>),
    /// Ask for what a job stopped for, and run it again with it.
    Ask(Box<Job>, Asked, Option<String>),
    /// What the menu says has changed.
    Changed,
}

pub struct JobsPage {
    jobs: Vec<Behind>,
    summary: Summary,
    /// The page is shown: what ends on it is seen.
    shown: bool,
}

impl EventEmitter<JobsEvent> for JobsPage {}

/// What a job that did what it was for did.
fn done_title(verb: Verb) -> String {
    match verb {
        Verb::Extract => tr!("Extracted"),
        Verb::Create => tr!("Archive created"),
        Verb::Copy => tr!("Copied"),
        Verb::Move => tr!("Moved"),
        Verb::Sign => tr!("Signed"),
        // A test that ends has passed, or failed: neither is done.
        Verb::Test | Verb::Add | Verb::Edit | Verb::Open => tr!("Done"),
    }
}

impl JobsPage {
    pub fn new() -> Self {
        JobsPage {
            jobs: Vec::new(),
            summary: Summary::default(),
            shown: false,
        }
    }

    pub fn summary(&self) -> Summary {
        self.summary
    }

    /// Why `job` cannot start while those of the page run, if it cannot.
    pub fn clash(&self, job: &Job, cx: &gpui::App) -> Option<String> {
        self.jobs
            .iter()
            .filter(|b| b.asked.is_none())
            .map(|b| b.view.read(cx))
            .filter(|v| v.state() == State::Running)
            .find_map(|v| job.clash(v.job()))
    }

    /// Whether `view` is a job of the page.
    pub fn holds(&self, view: &Entity<ProgressDialog>) -> bool {
        self.jobs.iter().any(|b| &b.view == view)
    }

    /// Takes `view`, which goes on in the background, and `events`, what it says.
    pub fn add(
        &mut self,
        view: Entity<ProgressDialog>,
        events: Subscription,
        cx: &mut Context<Self>,
    ) {
        if self.holds(&view) {
            return;
        }
        let observed = cx.observe(&view, |this, _, cx| this.changed(cx));
        self.jobs.push(Behind {
            view,
            asked: None,
            seen: false,
            _events: events,
            _observed: observed,
        });
        self.changed(cx);
    }

    /// Lets `view` go: it is over, or it runs again as a new job.
    pub fn remove(&mut self, view: &Entity<ProgressDialog>, cx: &mut Context<Self>) {
        self.jobs.retain(|b| &b.view != view);
        self.changed(cx);
    }

    /// `view` stopped for `asked`: it waits on the page for the user.
    pub fn wait_for(
        &mut self,
        view: &Entity<ProgressDialog>,
        asked: (Job, Asked, Option<String>),
        cx: &mut Context<Self>,
    ) {
        if let Some(b) = self.jobs.iter_mut().find(|b| &b.view == view) {
            b.asked = Some(asked);
            b.seen = self.shown;
        }
        self.changed(cx);
    }

    /// The page is shown, or not: what ended is seen when it is.
    pub fn set_shown(&mut self, shown: bool, cx: &mut Context<Self>) {
        self.shown = shown;
        self.changed(cx);
    }

    fn clear_ended(&mut self, cx: &mut Context<Self>) {
        self.jobs.retain(|b| !b.ended(cx));
        self.changed(cx);
    }

    /// Something of a job moved: the page draws it, and the menu hears if what it
    /// says changed.
    fn changed(&mut self, cx: &mut Context<Self>) {
        let shown = self.shown;
        let mut summary = Summary {
            jobs: self.jobs.len(),
            ..Summary::default()
        };
        for b in &mut self.jobs {
            let state = b.view.read(cx).state();
            let running = b.asked.is_none() && state == State::Running;
            if running {
                summary.running += 1;
                continue;
            }
            if shown {
                b.seen = true;
            }
            if !b.seen {
                summary.unseen = true;
                summary.waiting |= b.asked.is_some() || matches!(state, State::Failed(_));
            }
        }
        if summary != self.summary {
            self.summary = summary;
            cx.emit(JobsEvent::Changed);
        }
        cx.notify();
    }

    fn row(&self, i: usize, b: &Behind, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let c = colors();
        let job = b.view.read(cx);
        let state = job.state();
        let headline = match (&b.asked, &state) {
            (Some(_), _) => tr!("Waiting for a password or a key"),
            (None, State::Done(_)) => done_title(job.job().verb),
            (None, State::Passed(..)) => tr!("The test passed"),
            (None, _) => job.title(),
        };
        let percent = (job.done() * 100.).floor() as u64;
        // A new archive, under the name it got: the one asked for may have been taken.
        let detail = match (&state, job.job().verb, &job.job().to) {
            (State::Done(Some(made)), Verb::Create, Some(to)) => format!(
                "{} → {to}",
                made.file_name().map_or_else(
                    || made.display().to_string(),
                    |n| n.to_string_lossy().into_owned()
                )
            ),
            _ => job.detail(),
        };
        let below = match (&b.asked, &state) {
            (Some(_), _) => None,
            (None, State::Running) => Some(
                div()
                    .flex()
                    .flex_col()
                    .gap(ui(6.))
                    .child(
                        div().h(ui(8.)).rounded_full().bg(rgb(c.field)).child(
                            div()
                                .h_full()
                                .w(relative(job.done() as f32))
                                .rounded_full()
                                .bg(rgb(c.accent)),
                        ),
                    )
                    .child(
                        div()
                            .text_size(ui(13.))
                            .text_color(rgb(c.muted))
                            .child(format!("{percent}% · {}", job.left())),
                    )
                    .into_any_element(),
            ),
            (None, State::Failed(why)) => {
                Some(note(Tone::Warn).child(why.clone()).into_any_element())
            }
            (None, State::Passed(files, bytes)) => Some(
                div()
                    .text_size(ui(13.))
                    .text_color(rgb(c.muted))
                    .child(tr_n!(
                        *files,
                        "{count} file read through, {size}, with no errors.",
                        "{count} files read through, {size}, with no errors.",
                        count = count(*files),
                        size = size(*bytes)
                    ))
                    .into_any_element(),
            ),
            (None, _) => None,
        };
        let view = b.view.clone();
        let actions = div().flex().flex_shrink_0().items_center().gap(ui(8.));
        let actions = match (&b.asked, &state) {
            (Some(asked), _) => {
                let asked = asked.clone();
                let view = view.clone();
                actions.child(
                    button(("job-ask", i), tr!("Enter it…"), Fill::Accent, Size::Medium).on_click(
                        cx.listener(move |this, _, _, cx| {
                            let (job, asked, again) = asked.clone();
                            this.remove(&view, cx);
                            cx.emit(JobsEvent::Ask(Box::new(job), asked, again));
                        }),
                    ),
                )
            }
            (None, State::Running) => {
                let (show, stop) = (view.clone(), view.clone());
                actions
                    .child(
                        button(("job-show", i), tr!("Show"), Fill::Tonal, Size::Medium).on_click(
                            cx.listener(move |_, _, _, cx| cx.emit(JobsEvent::Show(show.clone()))),
                        ),
                    )
                    .child(
                        button(("job-stop", i), tr!("Stop"), Fill::Outline, Size::Medium).on_click(
                            cx.listener(move |_, _, _, cx| stop.update(cx, |j, cx| j.cancel(cx))),
                        ),
                    )
            }
            (None, State::Done(Some(made))) => {
                let made = made.clone();
                actions.child(
                    button(
                        ("job-file-manager", i),
                        file_manager::show_label(),
                        Fill::Tonal,
                        Size::Medium,
                    )
                    .on_click(move |_, _, cx| file_manager::show(&made, cx)),
                )
            }
            (None, _) => actions,
        };
        let running = b.asked.is_none() && state == State::Running;
        let actions = actions.when(!running, |d| {
            let view = view.clone();
            d.child(
                icon_button(("job-clear", i), "x", 40.)
                    .on_click(cx.listener(move |this, _, _, cx| this.remove(&view, cx))),
            )
        });
        div()
            .debug_selector(move || format!("job-{i}"))
            .flex()
            .gap(ui(14.))
            .px(ui(16.))
            .py(ui(14.))
            .rounded(ui(20.))
            .bg(rgb(c.well))
            .child(tile(40., 14., tonal_tint()).child(icon(job.icon_name(), 20., c.on_tonal)))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(ui(8.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(ui(12.))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .child(
                                        div()
                                            .text_size(ui(15.))
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .truncate()
                                            .child(headline),
                                    )
                                    .child(
                                        div()
                                            .text_size(ui(13.))
                                            .text_color(rgb(c.muted))
                                            .truncate()
                                            .child(detail),
                                    ),
                            )
                            .child(actions),
                    )
                    .children(below),
            )
    }
}

impl Render for JobsPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let any_ended = self.jobs.iter().any(|b| b.ended(cx));
        let rows: Vec<_> = self
            .jobs
            .iter()
            .enumerate()
            .map(|(i, b)| self.row(i, b, cx).into_any_element())
            .collect();
        div().size_full().flex().flex_col().child(
            panel()
                .id("jobs")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .flex()
                .flex_col()
                .gap(ui(8.))
                .px(ui(24.))
                .py(ui(20.))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .pb(ui(8.))
                        .child(title(tr!("Jobs"), 26.))
                        .when(any_ended, |d| {
                            d.child(
                                button(
                                    "jobs-clear",
                                    tr!("Clear finished"),
                                    Fill::Outline,
                                    Size::Medium,
                                )
                                .on_click(cx.listener(|this, _, _, cx| this.clear_ended(cx))),
                            )
                        }),
                )
                .when(rows.is_empty(), |d| {
                    d.child(
                        div()
                            .py(ui(24.))
                            .text_size(ui(14.))
                            .text_color(rgb(c.muted))
                            .child(tr!("Nothing runs in the background.")),
                    )
                })
                .children(rows),
        )
    }
}
