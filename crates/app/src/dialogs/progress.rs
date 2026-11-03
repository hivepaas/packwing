//! A job under way: how far it is, how long is left, and how it ended if it
//! failed. One that takes long can go on behind, on the Jobs page, without its
//! dialog: it is the same view, kept there, and shown again when asked.
//!
//! A job with work runs it in the background through packwing-core, and the
//! dialog shows what it reports. One without (the screenshots'
//! sample) counts up at a made-up speed, so the screen can be
//! seen moving, and closes itself at the end.

use packwing_i18n::{tr, tr_n};
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use gpui::{
    Context, EventEmitter, FontWeight, Render, Task, Window, div, prelude::*, relative, rgb,
};
use packwing_core::{Cancel, Control, Error, Progress};

use super::{
    DialogEvent, Editing, Job, Saving, SavingOn, Signing, Transferring, Verb, asked_for, card,
    footer, header,
};
use crate::colors::colors;
use crate::model::{count, size};
use crate::text_size::ui;
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
    /// The files and bytes a test has read.
    tested: Arc<Mutex<(u64, u64)>>,
    _work: Task<()>,
}

/// Where a job stands.
#[derive(Clone, Debug, PartialEq)]
pub enum State {
    Running,
    /// It did what it was for. Where what it made is, if it is on this computer: a
    /// new archive, or the folder extracted to.
    Done(Option<PathBuf>),
    /// Stopped when asked.
    Cancelled,
    Failed(String),
    /// A test that read everything: the files and bytes.
    Passed(u64, u64),
}

pub struct ProgressDialog {
    job: Job,
    /// How much is done, 0 to 1.
    done: f64,
    paused: bool,
    open_folder: bool,
    /// A new archive was made, and is going to its storage.
    uploading: bool,
    running: Option<Running>,
    /// Why it stopped, if it failed.
    failed: Option<String>,
    /// The files and bytes read, if it was a test and all of them were sound.
    passed: Option<(u64, u64)>,
    /// It ended, not failing: done, and where what it made is, or stopped.
    ended: Option<State>,
    _ticking: Task<()>,
}

impl EventEmitter<DialogEvent> for ProgressDialog {}

impl ProgressDialog {
    pub fn new(job: Job, cx: &mut Context<Self>) -> Self {
        let runs = !job.work.is_empty()
            || job.create.is_some()
            || job.add.is_some()
            || job.edit.is_some()
            || job.transfer.is_some()
            || job.sign.is_some();
        Self::with(job, runs, cx)
    }

    fn with(job: Job, runs: bool, cx: &mut Context<Self>) -> Self {
        let ticking = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(TICK).await;
                match this.update(cx, |this, cx| this.tick(cx)) {
                    Ok(false) => {}
                    _ => break,
                }
            }
        });
        let running = runs.then(|| Self::run(&job, cx));
        ProgressDialog {
            job,
            done: 0.,
            paused: false,
            // A scenario has no file manager to open.
            open_folder: !cfg!(all(test, feature = "ui-tests")),
            uploading: false,
            running,
            failed: None,
            passed: None,
            ended: None,
            _ticking: ticking,
        }
    }

    /// A job that never ends, as if it ran on: what it would write is taken.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn held(job: Job, cx: &mut Context<Self>) -> Self {
        Self::with(job, false, cx)
    }

    /// A job that does not start, for `why`: it says so, as one that failed.
    pub fn refused(job: Job, why: String, cx: &mut Context<Self>) -> Self {
        let mut dialog = Self::with(job, false, cx);
        dialog.failed = Some(why);
        dialog
    }

    /// Where it stands.
    pub fn state(&self) -> State {
        if let Some(why) = &self.failed {
            return State::Failed(why.clone());
        }
        if let Some((files, bytes)) = self.passed {
            return State::Passed(files, bytes);
        }
        self.ended.clone().unwrap_or(State::Running)
    }

    /// Whether it can go on behind, without its dialog: one that runs, and does not
    /// change the archive open or open a file when it is done.
    pub fn can_go_behind(&self) -> bool {
        self.running.is_some()
            && self.state() == State::Running
            && matches!(
                self.job.verb,
                Verb::Extract | Verb::Create | Verb::Test | Verb::Copy | Verb::Move | Verb::Sign
            )
    }

    pub fn job(&self) -> &Job {
        &self.job
    }

    /// How much is done, 0 to 1.
    pub fn done(&self) -> f64 {
        self.done
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
        let tested = Arc::new(Mutex::new((0, 0)));
        let tested_in_task = tested.clone();
        let (keys, how, picked) = (job.keys.clone(), job.how, job.picked.clone());
        let (verb, work, create, add) = (
            job.verb,
            job.work.clone(),
            job.create.clone(),
            job.add.clone(),
        );
        let edit = job.edit.clone();
        let transfer = job.transfer.clone();
        let sign = job.sign.clone();
        let task = cx.background_executor().spawn(async move {
            if let Some(Signing(storage, path, key)) = sign {
                packwing_core::sign_file(storage.as_ref(), &path, &key, &control)?;
                return Ok(None);
            }
            if let Some(Transferring(t, went)) = transfer {
                let done = packwing_core::transfer(&t, &control)?;
                *went.lock().unwrap_or_else(PoisonError::into_inner) = done;
                return Ok(None);
            }
            if let Some(c) = create {
                return match &c.to {
                    Saving::Here(archive) => {
                        packwing_core::create(&c.paths, archive, &c.how, &control).map(Some)
                    }
                    Saving::On(on) => packwing_core::create_on(
                        &c.paths,
                        on.place.storage.as_ref(),
                        &on.folder,
                        &on.name,
                        &c.how,
                        &control,
                        &|| on.uploading.store(true, Ordering::Relaxed),
                    ),
                };
            }
            if let Some(a) = add {
                packwing_core::add_with(
                    &a.paths,
                    &a.archive,
                    &a.folder,
                    a.on_existing,
                    &a.stored_extensions,
                    &keys,
                    &control,
                )?;
                return Ok(None);
            }
            if let Some((archive, _, edit)) = edit {
                match edit {
                    Editing::Delete(picked) => packwing_core::delete(&archive, &picked, &keys)?,
                    Editing::Rename { from, to } => {
                        packwing_core::rename(&archive, &from, &to, &keys)?
                    }
                }
                return Ok(None);
            }
            for (source, destination) in &work {
                match (verb, destination) {
                    (Verb::Open, _) => {
                        let entry = picked.first().map_or(&[][..], Vec::as_slice);
                        let copy = packwing_core::extract_to_open(source, entry, &keys, &control)?;
                        return Ok(Some(copy));
                    }
                    (Verb::Extract, Some(to)) if !picked.is_empty() => {
                        packwing_core::extract_selected(
                            source, to, &picked, &keys, &how, &control,
                        )?;
                    }
                    (Verb::Extract, Some(to)) => {
                        packwing_core::extract_with(source, to, &keys, &how, &control)?;
                    }
                    _ => {
                        let read = packwing_core::test(source, &keys, &control)?;
                        let mut tested = tested_in_task
                            .lock()
                            .unwrap_or_else(PoisonError::into_inner);
                        tested.0 += read.files;
                        tested.1 += read.bytes;
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
            tested,
            _work: work,
        }
    }

    /// The job ended: done, cancelled, or failed.
    fn finish(&mut self, result: Result<Option<PathBuf>, Error>, cx: &mut Context<Self>) {
        self.ended = match &result {
            Ok(made) => Some(State::Done(self.made_at(made.clone()))),
            Err(Error::Cancelled) => Some(State::Cancelled),
            Err(_) => None,
        };
        // Whoever looks at it, the Jobs page among them, sees it ended.
        cx.notify();
        match result {
            Ok(_) if self.job.add.is_some() => {
                if let Some(a) = self.job.add.clone() {
                    cx.emit(DialogEvent::Added(
                        a.archive,
                        a.folder,
                        self.job.keys.clone(),
                    ));
                }
            }
            Ok(_) if self.job.edit.is_some() => {
                if let Some((archive, folder, _)) = self.job.edit.clone() {
                    cx.emit(DialogEvent::Added(archive, folder, self.job.keys.clone()));
                }
            }
            Ok(_) if self.job.transfer.is_some() => {
                if let Some(Transferring(_, went)) = &self.job.transfer {
                    let went = *went.lock().unwrap_or_else(PoisonError::into_inner);
                    cx.emit(DialogEvent::Transferred(went));
                }
            }
            Ok(_) if self.job.sign.is_some() => cx.emit(DialogEvent::Signed),
            Ok(Some(copy)) if self.job.verb == Verb::Open => {
                cx.open_with_system(&copy);
                cx.emit(DialogEvent::Close);
            }
            // A test is done for what it says: it stays to say it.
            Ok(_) if self.job.verb == Verb::Test => {
                self.passed = self
                    .running
                    .as_ref()
                    .map(|r| *r.tested.lock().unwrap_or_else(PoisonError::into_inner));
                cx.notify();
            }
            Ok(made) if let Some(on) = self.saving_on().cloned() => {
                let show = self.open_folder && made.is_none();
                if let Some(made) = made.filter(|_| self.open_folder) {
                    cx.reveal_path(&made);
                }
                cx.emit(DialogEvent::Stored {
                    place: on.place,
                    folder: on.folder,
                    show,
                });
            }
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
                if (self.job.work.len() == 1 || self.job.add.is_some())
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

    /// Where what it made is, on this computer: the new archive, or the folder it
    /// extracted to.
    fn made_at(&self, made: Option<PathBuf>) -> Option<PathBuf> {
        if self.saving_on().is_some() {
            return None;
        }
        made.or_else(|| match (self.job.verb, self.job.work.first()) {
            (Verb::Extract, Some((_, Some(to)))) => Some(to.clone()),
            _ => None,
        })
    }

    /// The new archive's place on a storage, if that is where it goes.
    fn saving_on(&self) -> Option<&SavingOn> {
        match self.job.create.as_ref().map(|c| &c.to) {
            Some(Saving::On(on)) => Some(on),
            _ => None,
        }
    }

    /// Puts the count at `done` (0 to 1), as if it got there.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn set_done(&mut self, done: f64, cx: &mut Context<Self>) {
        self.done = done.clamp(0., 1.);
        cx.notify();
    }

    /// Shows the job as failed, for `why`.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn set_failed(&mut self, why: String, cx: &mut Context<Self>) {
        self.failed = Some(why);
        cx.notify();
    }

    /// Puts it where `state` says, as the Jobs page's picture shows it.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn set_state_for_test(&mut self, state: State, cx: &mut Context<Self>) {
        match state {
            State::Failed(why) => self.failed = Some(why),
            State::Passed(files, bytes) => self.passed = Some((files, bytes)),
            State::Running => self.ended = None,
            ended => self.ended = Some(ended),
        }
        cx.notify();
    }

    /// Shows the test as passed, having read these.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn set_passed(&mut self, files: u64, bytes: u64, cx: &mut Context<Self>) {
        self.passed = Some((files, bytes));
        cx.notify();
    }

    /// The files and bytes read, if it was a test that passed.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn passed(&self) -> Option<(u64, u64)> {
        self.passed
    }

    /// Why it failed, if it did.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn failed(&self) -> Option<&str> {
        self.failed.as_deref()
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
        if self.failed.is_some() || self.passed.is_some() || self.ended.is_some() {
            return true;
        }
        if !self.uploading
            && self
                .saving_on()
                .is_some_and(|on| on.uploading.load(Ordering::Relaxed))
        {
            self.uploading = true;
            // A new count, and a new speed.
            if let Some(r) = &mut self.running {
                r.started = Instant::now();
            }
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

    pub fn cancel(&mut self, cx: &mut Context<Self>) {
        match &self.running {
            // It stops at its next step, and closes the dialog then.
            Some(r) if self.failed.is_none() => r.cancel.cancel(),
            _ => cx.emit(DialogEvent::Close),
        }
    }

    pub fn title(&self) -> String {
        match (self.job.verb, self.failed.is_some()) {
            (Verb::Extract, false) => tr!("Extracting…"),
            (Verb::Create, false) if self.uploading => tr!("Uploading…"),
            (Verb::Create, false) => tr!("Creating…"),
            (Verb::Add, false) => tr!("Adding…"),
            (Verb::Edit, false) => tr!("Changing the archive…"),
            (Verb::Open, false) => tr!("Opening…"),
            (Verb::Test, false) => tr!("Testing…"),
            (Verb::Copy, false) => tr!("Copying…"),
            (Verb::Move, false) => tr!("Moving…"),
            (Verb::Sign, false) => tr!("Signing…"),
            (Verb::Sign, true) => tr!("Couldn’t sign"),
            (Verb::Copy, true) => tr!("Couldn’t copy"),
            (Verb::Move, true) => tr!("Couldn’t move"),
            (Verb::Extract, true) => tr!("Couldn’t extract"),
            (Verb::Create, true) => tr!("Couldn’t create the archive"),
            (Verb::Add, true) => tr!("Couldn’t add the files"),
            (Verb::Edit, true) => tr!("Couldn’t change the archive"),
            (Verb::Open, true) => tr!("Couldn’t open the file"),
            (Verb::Test, true) => tr!("The test failed"),
        }
    }

    pub fn left(&self) -> String {
        if self.paused {
            return tr!("Paused");
        }
        let speed = self.speed();
        if speed == 0 {
            return tr!("Starting…");
        }
        let (_, _, bytes, total) = self.counts();
        let secs = (total.saturating_sub(bytes) as f64 / speed as f64).round() as u64;
        match secs {
            0..=4 => tr!("Almost done"),
            5..=59 => tr!("About {secs} seconds left", secs = secs),
            60..=119 => tr!("About a minute left"),
            _ => tr!(
                "About {div_ceil} minutes left",
                div_ceil = secs.div_ceil(60)
            ),
        }
    }
}

impl ProgressDialog {
    /// What it reads and where it goes, in a line.
    pub fn detail(&self) -> String {
        let j = &self.job;
        match &j.to {
            Some(to) => format!("{} → {}", j.name, to),
            None => tr!("{name} · checking that every file reads", name = j.name),
        }
    }

    pub fn icon_name(&self) -> &'static str {
        match self.job.verb {
            Verb::Extract | Verb::Open => "extract",
            Verb::Create | Verb::Add | Verb::Edit => "logo",
            Verb::Test => "shield",
            Verb::Copy | Verb::Move => "storages",
            Verb::Sign => "shield",
        }
    }
}

impl Render for ProgressDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let j = &self.job;
        let detail = self.detail();
        let icon_name = self.icon_name();
        if let Some((files, bytes)) = self.passed {
            return card()
                .gap(ui(22.))
                .child(header(
                    icon_name,
                    tonal_tint(),
                    tr!("The test passed"),
                    j.name.clone(),
                    None,
                ))
                .child(note(Tone::Good).child(tr_n!(
                    files,
                    "{count} file read through, {size}, with no errors.",
                    "{count} files read through, {size}, with no errors.",
                    count = count(files),
                    size = size(bytes)
                )))
                .child(
                    footer().child(div().flex_1()).child(
                        button("close", tr!("Close"), Fill::Accent, Size::Medium)
                            .px(ui(28.))
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close))),
                    ),
                );
        }
        let header = header(icon_name, tonal_tint(), self.title(), detail, None);
        if let Some(why) = &self.failed {
            return card()
                .gap(ui(22.))
                .child(header)
                .child(note(Tone::Warn).child(why.clone()))
                .child(
                    footer().child(div().flex_1()).child(
                        button("close", tr!("Close"), Fill::Accent, Size::Medium)
                            .px(ui(28.))
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
            .gap(ui(22.))
            .child(header)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(ui(10.))
                    .child(
                        div()
                            .flex()
                            .items_end()
                            .justify_between()
                            .gap(ui(12.))
                            .child(
                                div()
                                    .text_size(ui(40.))
                                    .line_height(relative(1.1))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(c.on_tonal))
                                    .child(format!("{percent}%")),
                            )
                            .child(
                                div()
                                    .pb(ui(6.))
                                    .text_size(ui(15.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(self.left()),
                            ),
                    )
                    .child(
                        div().h(ui(14.)).rounded_full().bg(rgb(c.field)).child(
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
                            .gap_x(ui(16.))
                            .gap_y(ui(4.))
                            .text_size(ui(14.))
                            .text_color(rgb(c.muted))
                            .child(tr!(
                                "{count} of {count2} files · {size} of {size2}",
                                count = count(files),
                                count2 = count(all_files),
                                size = size(bytes),
                                size2 = size(all_bytes)
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
                        .gap(ui(12.))
                        .px(ui(16.))
                        .py(ui(12.))
                        .rounded(ui(18.))
                        .bg(rgb(c.well))
                        .child(icon("image", 20., c.accent))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .truncate()
                                .text_size(ui(14.))
                                .child(current),
                        ),
                )
            })
            .when(
                !matches!(
                    j.verb,
                    Verb::Test
                        | Verb::Add
                        | Verb::Edit
                        | Verb::Open
                        | Verb::Copy
                        | Verb::Move
                        | Verb::Sign
                ),
                |d| {
                    d.child(
                        check_row(
                            "open-folder",
                            self.open_folder,
                            tr!("Open the folder when it’s done"),
                        )
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.open_folder = !this.open_folder;
                            cx.notify();
                        })),
                    )
                },
            )
            .child(
                footer()
                    // The samples' only goes away.
                    .when(!real || self.can_go_behind(), |d| {
                        d.child(
                            button(
                                "background",
                                tr!("Run in background"),
                                Fill::Tonal,
                                Size::Medium,
                            )
                            .px(ui(20.))
                            .on_click(cx.listener(
                                move |_, _, _, cx| {
                                    cx.emit(if real {
                                        DialogEvent::Behind
                                    } else {
                                        DialogEvent::Close
                                    })
                                },
                            )),
                        )
                    })
                    // A job that runs cannot pause yet.
                    .when(!real, |d| {
                        d.child(
                            button(
                                "background",
                                tr!("Run in background"),
                                Fill::Tonal,
                                Size::Medium,
                            )
                            .px(ui(20.))
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close))),
                        )
                    })
                    .child(div().flex_1())
                    .when(!real, |d| {
                        d.child(
                            button(
                                "pause",
                                if self.paused {
                                    tr!("Resume")
                                } else {
                                    tr!("Pause")
                                },
                                Fill::Text,
                                Size::Medium,
                            )
                            .px(ui(20.))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.paused = !this.paused;
                                cx.notify();
                            })),
                        )
                    })
                    .child(
                        button("cancel", tr!("Cancel"), Fill::Outline, Size::Medium)
                            .on_click(cx.listener(|this, _, _, cx| this.cancel(cx))),
                    ),
            )
    }
}
