//! A job under way: how far it is, how long is left, and how it ended if it
//! failed. One that takes long can go on behind, on the Jobs page, without its
//! dialog: it is the same view, kept there, and shown again when asked.
//!
//! A job with work runs it in the background through packwing-core, and the
//! dialog shows what it reports. One without (the screenshots'
//! sample) counts up at a made-up speed, so the screen can be
//! seen moving, and closes itself at the end.
//!
//! An extraction that meets a file whose data is damaged waits on its dialog:
//! to stop, to leave the file out, or to keep what was read of it, and go on.
//! One holding names this computer can't use asks once, before anything is
//! written, about all of them: to stop, to leave them out, or to rename them.
//! Once it ends, the dialog stays to say which files they were.

use packwing_i18n::{tr, tr_n};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use gpui::{
    ClipboardItem, Context, EventEmitter, FontWeight, Render, Task, Window, div, prelude::*,
    relative, rgb,
};
use packwing_core::{
    BadName, Cancel, Control, Damage, DamagedFn, Error, GoneFn, LinksOutsideFn, OnDamaged,
    OnLinksOutside, OnUnnamable, OutsideLink, Progress, Renaming, Summary, UnnamableFn, damages,
    left_out, renamings,
};

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
/// The most damaged files listed when it is done: the rest are counted, and
/// copied with the list.
const MOST_SHOWN: usize = 6;

/// A job that runs: what it has reported, and what stops it.
struct Running {
    progress: Arc<Mutex<Progress>>,
    cancel: Cancel,
    started: Instant,
    /// The files and bytes a test has read.
    tested: Arc<Mutex<(u64, u64)>>,
    damaged: Arc<Mutex<Damaged>>,
    _work: Task<()>,
}

/// What an extraction and its dialog share of the files whose data is damaged,
/// and of those whose names can't be used.
#[derive(Default)]
struct Damaged {
    /// The file it waits on an answer about, and where the answer goes.
    asked: Option<(Damage, Sender<OnDamaged>)>,
    /// What was answered for all the rest, once it is.
    for_rest: Option<OnDamaged>,
    /// The files whose names can't be used that it waits on an answer about,
    /// and where the answer goes.
    named: Option<(Vec<BadName>, Sender<OnUnnamable>)>,
    /// The links that lead outside the folder that it waits on an answer about,
    /// and where the answer goes.
    linked: Option<(Vec<OutsideLink>, Sender<OnLinksOutside>)>,
    /// What came of each archive extracted.
    found: Report,
}

/// What an extraction went on from: files whose data was damaged, and files left
/// out or renamed; or what a new archive went on from: files gone by the time
/// they were read, as another program's files may go while it runs.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Report {
    pub damaged: Vec<Damage>,
    pub left_out: Vec<BadName>,
    pub renamed: Vec<Renaming>,
    pub gone: Vec<String>,
}

impl Report {
    fn add(&mut self, summary: &Summary) {
        self.damaged.extend(damages(summary));
        self.left_out.extend(left_out(summary));
        self.renamed.extend(renamings(summary));
    }

    fn is_empty(&self) -> bool {
        self.damaged.is_empty()
            && self.left_out.is_empty()
            && self.renamed.is_empty()
            && self.gone.is_empty()
    }

    /// Each file, what became of it, and why, in the order said.
    fn rows(&self) -> Vec<(String, String, String)> {
        let damaged = self.damaged.iter().map(|d| {
            let kept = match d.kept {
                Some(bytes) => tr!("kept, {size}", size = size(bytes)),
                None => tr!("left out"),
            };
            (d.path.clone(), kept, d.why.clone())
        });
        let left = self
            .left_out
            .iter()
            .map(|b| (b.path.clone(), tr!("left out"), b.why.clone()));
        let renamed = self.renamed.iter().map(|r| {
            (
                r.path.clone(),
                tr!("renamed {to}", to = r.to.clone()),
                r.why.clone(),
            )
        });
        let gone = self.gone.iter().map(|path| {
            (
                path.clone(),
                tr!("left out"),
                tr!("It was gone by the time it was read."),
            )
        });
        damaged.chain(left).chain(renamed).chain(gone).collect()
    }
}

/// What notes each file or folder gone by the time a new archive read it.
fn noting_gone(damaged: &Arc<Mutex<Damaged>>) -> GoneFn {
    let damaged = damaged.clone();
    Arc::new(move |path: &Path| {
        damaged
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .found
            .gone
            .push(path.display().to_string());
    })
}

/// What asks the dialog about each file whose data is damaged, and waits for its
/// answer: none, if nothing answers, stops.
fn asking(damaged: &Arc<Mutex<Damaged>>) -> DamagedFn {
    let damaged = damaged.clone();
    Arc::new(move |damage: &Damage| {
        let answered = {
            let mut d = damaged.lock().unwrap_or_else(PoisonError::into_inner);
            if let Some(chosen) = d.for_rest {
                return chosen;
            }
            let (answer, answered) = channel();
            d.asked = Some((damage.clone(), answer));
            answered
        };
        answered.recv().unwrap_or(OnDamaged::Stop)
    })
}

/// What asks the dialog, once, about the files whose names can't be used, and
/// waits for its answer: none, if nothing answers, stops.
fn asking_names(damaged: &Arc<Mutex<Damaged>>) -> UnnamableFn {
    let damaged = damaged.clone();
    Arc::new(move |bad: &[BadName]| {
        let answered = {
            let mut d = damaged.lock().unwrap_or_else(PoisonError::into_inner);
            let (answer, answered) = channel();
            d.named = Some((bad.to_vec(), answer));
            answered
        };
        answered.recv().unwrap_or(OnUnnamable::Stop)
    })
}

/// What asks the dialog, once, about the links that lead outside the folder, and
/// waits for its answer: none, if nothing answers, stops.
fn asking_links(damaged: &Arc<Mutex<Damaged>>) -> LinksOutsideFn {
    let damaged = damaged.clone();
    Arc::new(move |links: &[OutsideLink]| {
        let answered = {
            let mut d = damaged.lock().unwrap_or_else(PoisonError::into_inner);
            let (answer, answered) = channel();
            d.linked = Some((links.to_vec(), answer));
            answered
        };
        answered.recv().unwrap_or(OnLinksOutside::Stop)
    })
}

/// Where a job stands.
#[derive(Clone, Debug, PartialEq)]
pub enum State {
    /// Waiting for one of the jobs that run to end: only so many run at once.
    Queued,
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
    /// It waits for its turn, and has not started.
    queued: bool,
    /// The answer about a damaged file is for all the rest too.
    same_for_rest: bool,
    /// It has said it waits on an answer, or has damaged files to show.
    called: bool,
    /// The files whose data was damaged, left out or renamed, once it is done
    /// with some.
    report: Option<Report>,
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
            || job.sign.is_some()
            || job.split.is_some()
            || job.join.is_some()
            || job.convert.is_some();
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
            queued: false,
            same_for_rest: false,
            called: false,
            report: None,
            _ticking: ticking,
        }
    }

    /// A job that never ends, as if it ran on: what it would write is taken.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn held(job: Job, cx: &mut Context<Self>) -> Self {
        Self::with(job, false, cx)
    }

    /// A job that waits for its turn: [`ProgressDialog::begin`] starts it.
    pub fn queued(job: Job, cx: &mut Context<Self>) -> Self {
        let mut dialog = Self::with(job, false, cx);
        dialog.queued = true;
        dialog
    }

    /// Starts a job that waited for its turn.
    pub fn begin(&mut self, cx: &mut Context<Self>) {
        if !self.queued {
            return;
        }
        self.queued = false;
        self.running = Some(Self::run(&self.job, cx));
        cx.notify();
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
        if let Some(ended) = &self.ended {
            return ended.clone();
        }
        if self.queued {
            return State::Queued;
        }
        State::Running
    }

    /// Whether it can go on behind, without its dialog: one that runs, and does not
    /// change the archive open or open a file when it is done.
    pub fn can_go_behind(&self) -> bool {
        (self.running.is_some() || self.queued)
            && matches!(self.state(), State::Running | State::Queued)
            && matches!(
                self.job.verb,
                Verb::Extract
                    | Verb::Create
                    | Verb::Test
                    | Verb::Copy
                    | Verb::Move
                    | Verb::Sign
                    | Verb::Split
                    | Verb::Join
                    | Verb::Convert
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
        let damaged = Arc::new(Mutex::new(Damaged::default()));
        let control = Control {
            progress: Some(Arc::new(move |p: &Progress| {
                *told.lock().unwrap_or_else(PoisonError::into_inner) = *p;
            })),
            cancel: Some(cancel.clone()),
            // Only what is extracted to stay: a file opened, or read, is whole.
            on_damaged: matches!(job.verb, Verb::Extract | Verb::Convert).then(|| asking(&damaged)),
            on_unnamable: matches!(job.verb, Verb::Extract | Verb::Convert)
                .then(|| asking_names(&damaged)),
            // Converting keeps every link: the new archive holds it.
            on_links_outside: (job.verb == Verb::Extract).then(|| asking_links(&damaged)),
            on_gone: job.create.is_some().then(|| noting_gone(&damaged)),
        };
        let found = damaged.clone();
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
        let (split, join, convert) = (job.split.clone(), job.join.clone(), job.convert.clone());
        let converted = damaged.clone();
        let job_work = move || {
            if let Some((path, size)) = split {
                let parts = packwing_core::split_file(&path, size, &control)?;
                return Ok(parts.into_iter().next());
            }
            if let Some(part) = join {
                return packwing_core::join_parts(&part, &control).map(Some);
            }
            if let Some(c) = convert {
                let done = packwing_core::convert(c.from, &keys, &c.to, &c.how, &how, &control)?;
                converted
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .found
                    .add(&done.summary);
                return Ok(Some(done.made));
            }
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
                    Editing::Delete(picked) => {
                        packwing_core::delete(&archive, &picked, &keys, &control)?
                    }
                    Editing::Rename { from, to } => {
                        packwing_core::rename(&archive, &from, &to, &keys, &control)?
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
                    (Verb::Extract, Some(to)) => {
                        let done = if picked.is_empty() {
                            packwing_core::extract_with(source, to, &keys, &how, &control)?
                        } else {
                            packwing_core::extract_selected(
                                source, to, &picked, &keys, &how, &control,
                            )?
                        };
                        found
                            .lock()
                            .unwrap_or_else(PoisonError::into_inner)
                            .found
                            .add(&done);
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
        };
        let (done, finished) = futures_channel::oneshot::channel();
        let run_it = move || {
            let _ = done.send(job_work());
        };
        if matches!(verb, Verb::Extract | Verb::Convert) {
            // An extraction may wait on its dialog's answer about a damaged file,
            // as long as that takes: on a thread of its own, not the executor's.
            // One that cannot start says so, as the job's sender is gone.
            let _ = std::thread::Builder::new()
                .name("packwing-extract".into())
                .spawn(run_it);
        } else {
            cx.background_executor()
                .spawn(async move { run_it() })
                .detach();
        }
        let work = cx.spawn(async move |this, cx| {
            let result = finished
                .await
                .unwrap_or_else(|_| Err(Error::Failed(tr!("The job stopped before it was done."))));
            let _ = this.update(cx, |this, cx| this.finish(result, cx));
        });
        Running {
            progress,
            cancel,
            started: Instant::now(),
            tested,
            damaged,
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
        // What opened its one archive opens it the next time, for a while.
        if result.is_ok()
            && let [(source, _)] = self.job.work.as_slice()
        {
            cx.emit(DialogEvent::Proved(source.clone(), self.job.keys.clone()));
        }
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
            Ok(Some(copy)) if self.job.verb == Verb::Open && self.job.inside => {
                cx.emit(DialogEvent::Inside(copy));
            }
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
            Ok(made) if !self.found().is_empty() => {
                if let Some(made) = made.filter(|_| self.open_folder) {
                    cx.reveal_path(&made);
                } else if self.open_folder
                    && let Some((_, Some(to))) = self.job.work.first()
                {
                    cx.open_with_system(to);
                }
                // It stays to say which, seen even if it went on behind.
                self.report = Some(self.found());
                cx.emit(DialogEvent::Attention);
                cx.notify();
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
                if (self.job.work.len() == 1
                    || self.job.add.is_some()
                    || self.job.convert.is_some())
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
        cx.emit(DialogEvent::Ended);
    }

    /// The files whose data was damaged, left out or renamed, so far.
    fn found(&self) -> Report {
        self.running.as_ref().map_or_else(Report::default, |r| {
            r.damaged
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .found
                .clone()
        })
    }

    /// The file it waits on an answer about, if it does.
    pub fn asked(&self) -> Option<Damage> {
        let r = self.running.as_ref()?;
        let d = r.damaged.lock().unwrap_or_else(PoisonError::into_inner);
        d.asked.as_ref().map(|(damage, _)| damage.clone())
    }

    /// Answers about the damaged file it waits on, and about the rest too if
    /// that box is ticked.
    fn answer(&mut self, chosen: OnDamaged, cx: &mut Context<Self>) {
        if let Some(r) = &self.running {
            let mut d = r.damaged.lock().unwrap_or_else(PoisonError::into_inner);
            if self.same_for_rest {
                d.for_rest = Some(chosen);
            }
            if let Some((_, answer)) = d.asked.take() {
                let _ = answer.send(chosen);
            }
        }
        self.called = false;
        cx.notify();
    }

    /// The files whose names can't be used that it waits on an answer about, if
    /// it does.
    pub fn asked_names(&self) -> Option<Vec<BadName>> {
        let r = self.running.as_ref()?;
        let d = r.damaged.lock().unwrap_or_else(PoisonError::into_inner);
        d.named.as_ref().map(|(bad, _)| bad.clone())
    }

    /// The links that lead outside the folder that it waits on an answer about,
    /// if it does.
    pub fn asked_links(&self) -> Option<Vec<OutsideLink>> {
        let r = self.running.as_ref()?;
        let d = r.damaged.lock().unwrap_or_else(PoisonError::into_inner);
        d.linked.as_ref().map(|(links, _)| links.clone())
    }

    /// Answers about the links that lead outside the folder.
    fn answer_links(&mut self, chosen: OnLinksOutside, cx: &mut Context<Self>) {
        if let Some(r) = &self.running
            && let Some((_, answer)) = r
                .damaged
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .linked
                .take()
        {
            let _ = answer.send(chosen);
        }
        self.called = false;
        cx.notify();
    }

    /// Answers about the files whose names can't be used.
    fn answer_names(&mut self, chosen: OnUnnamable, cx: &mut Context<Self>) {
        if let Some(r) = &self.running
            && let Some((_, answer)) = r
                .damaged
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .named
                .take()
        {
            let _ = answer.send(chosen);
        }
        self.called = false;
        cx.notify();
    }

    /// Puts it as waiting on an answer about `damage`, or as done with these
    /// damaged, as the pictures show it.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn set_damaged_for_test(
        &mut self,
        asked: Option<Damage>,
        report: Option<Report>,
        cx: &mut Context<Self>,
    ) {
        if let (Some(damage), Some(r)) = (asked, &self.running) {
            let (answer, _) = channel();
            r.damaged
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .asked = Some((damage, answer));
        }
        if report.is_some() {
            self.ended = Some(State::Done(None));
        }
        self.report = report;
        cx.notify();
    }

    /// Puts it as waiting on an answer about the files of `bad`, as the
    /// pictures show it.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn set_names_for_test(&mut self, bad: Vec<BadName>, cx: &mut Context<Self>) {
        if let Some(r) = &self.running {
            let (answer, _) = channel();
            r.damaged
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .named = Some((bad, answer));
        }
        cx.notify();
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
            State::Queued => self.queued = true,
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
        // Nothing moves until its turn: it is no sample to count up.
        if self.queued {
            return false;
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
            // Seen, even if it went on behind.
            if !self.called && (self.asked().is_some() || self.asked_names().is_some()) {
                self.called = true;
                cx.emit(DialogEvent::Attention);
            }
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
            // It stops at its next step, and closes the dialog then: one that
            // waits on an answer goes on to it, the file left out.
            Some(r) if self.state() == State::Running => {
                r.cancel.cancel();
                let mut d = r.damaged.lock().unwrap_or_else(PoisonError::into_inner);
                if let Some((_, answer)) = d.asked.take() {
                    let _ = answer.send(OnDamaged::Skip);
                }
                if let Some((_, answer)) = d.named.take() {
                    let _ = answer.send(OnUnnamable::Skip);
                }
                if let Some((_, answer)) = d.linked.take() {
                    let _ = answer.send(OnLinksOutside::Skip);
                }
            }
            _ => {
                // One that never ran, or waited for its turn, stops now.
                if matches!(self.state(), State::Running | State::Queued) {
                    self.queued = false;
                    self.ended = Some(State::Cancelled);
                    cx.notify();
                }
                cx.emit(DialogEvent::Close);
            }
        }
    }

    pub fn title(&self) -> String {
        if self.queued {
            return tr!("Waiting for other jobs…");
        }
        if self.asked().is_some() {
            return tr!("A file is damaged");
        }
        if self.asked_names().is_some() {
            return tr!("Some names can’t be used here");
        }
        if let Some(report) = &self.report {
            return match (self.job.verb, report.damaged.is_empty()) {
                (Verb::Convert, false) => tr!("Converted, but some files are damaged"),
                (_, false) => tr!("Extracted, but some files are damaged"),
                (Verb::Convert, true) => tr!("Converted, with some files left out or renamed"),
                (_, true) => tr!("Extracted, with some files left out or renamed"),
            };
        }
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
            (Verb::Split, false) => tr!("Splitting…"),
            (Verb::Join, false) => tr!("Joining…"),
            (Verb::Convert, false) => tr!("Converting…"),
            (Verb::Split, true) => tr!("Couldn’t split the file"),
            (Verb::Join, true) => tr!("Couldn’t join the parts"),
            (Verb::Convert, true) => tr!("Couldn’t convert the archive"),
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
        if self.queued {
            return tr!("Starts when one of the others ends");
        }
        if self.paused {
            return tr!("Paused");
        }
        let speed = self.speed();
        if speed == 0 {
            return tr!("Starting…");
        }
        let (_, _, bytes, total) = self.counts();
        // All is read, and what is left is the job's own: the last of the
        // compressing, a 7z's headers, the archive put in its place.
        if total > 0 && bytes >= total {
            return tr!("Finishing…");
        }
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
            Verb::Create | Verb::Add | Verb::Edit | Verb::Convert => "logo",
            Verb::Split | Verb::Join => "archive",
            Verb::Test => "shield",
            Verb::Copy | Verb::Move => "storages",
            Verb::Sign => "shield",
        }
    }
}

/// Files, a row each: the path, and what is said of it, muted; the first
/// [`MOST_SHOWN`], and how many more.
fn listing(rows: &[(String, String, String)]) -> gpui::Div {
    let c = colors();
    let shown = rows.iter().take(MOST_SHOWN).map(|(path, what, _)| {
        div()
            .flex()
            .gap(ui(12.))
            .text_size(ui(14.))
            .child(div().flex_1().min_w_0().truncate().child(path.clone()))
            .child(
                div()
                    .max_w(relative(0.5))
                    .truncate()
                    .text_color(rgb(c.muted))
                    .child(what.clone()),
            )
    });
    let more = rows.len().saturating_sub(MOST_SHOWN) as u64;
    div()
        .flex()
        .flex_col()
        .gap(ui(6.))
        .px(ui(16.))
        .py(ui(12.))
        .rounded(ui(18.))
        .bg(rgb(c.well))
        .children(shown)
        .when(more > 0, |d| {
            d.child(
                div()
                    .text_size(ui(14.))
                    .text_color(rgb(c.muted))
                    .child(tr_n!(
                        more,
                        "and {count} more",
                        "and {count} more",
                        count = count(more)
                    )),
            )
        })
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
        if let Some(damage) = self.asked() {
            return card()
                .gap(ui(22.))
                .child(header)
                .child(note(Tone::Warn).child(tr!(
                    "“{path}” is damaged: {why}. Stop, leave it out, or keep what could be read of it?",
                    path = damage.path,
                    why = damage.why
                )))
                .child(
                    check_row(
                        "same-for-rest",
                        self.same_for_rest,
                        tr!("Do the same for the other damaged files"),
                    )
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.same_for_rest = !this.same_for_rest;
                        cx.notify();
                    })),
                )
                .child(
                    footer()
                        .child(
                            button("stop", tr!("Stop"), Fill::Outline, Size::Medium).on_click(
                                cx.listener(|this, _, _, cx| this.answer(OnDamaged::Stop, cx)),
                            ),
                        )
                        .child(div().flex_1())
                        .child(
                            button("skip", tr!("Leave it out"), Fill::Tonal, Size::Medium)
                                .px(ui(20.))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.answer(OnDamaged::Skip, cx)
                                })),
                        )
                        .child(
                            button("keep", tr!("Keep what was read"), Fill::Accent, Size::Medium)
                                .px(ui(20.))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.answer(OnDamaged::Keep, cx)
                                })),
                        ),
                );
        }
        if let Some(bad) = self.asked_names() {
            let n = bad.len() as u64;
            let rows: Vec<(String, String, String)> = bad
                .iter()
                .map(|b| (b.path.clone(), b.why.clone(), String::new()))
                .collect();
            return card()
                .gap(ui(22.))
                .child(header)
                .child(note(Tone::Warn).child(tr_n!(
                    n,
                    "{count} file can’t be extracted under its name here. Nothing is written yet: stop, leave it out, or rename it?",
                    "{count} files can’t be extracted under their names here. Nothing is written yet: stop, leave them out, or rename them?",
                    count = count(n)
                )))
                .child(listing(&rows))
                .child(
                    footer()
                        .child(
                            button("stop", tr!("Stop"), Fill::Outline, Size::Medium).on_click(
                                cx.listener(|this, _, _, cx| {
                                    this.answer_names(OnUnnamable::Stop, cx)
                                }),
                            ),
                        )
                        .child(div().flex_1())
                        .child(
                            button("skip", tr!("Leave out"), Fill::Tonal, Size::Medium)
                                .px(ui(20.))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.answer_names(OnUnnamable::Skip, cx)
                                })),
                        )
                        .child(
                            button("rename", tr!("Rename"), Fill::Accent, Size::Medium)
                                .px(ui(20.))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.answer_names(OnUnnamable::Rename, cx)
                                })),
                        ),
                );
        }
        if let Some(links) = self.asked_links() {
            let n = links.len() as u64;
            let rows: Vec<(String, String, String)> = links
                .iter()
                .map(|l| (l.path.clone(), format!("→ {}", l.target), String::new()))
                .collect();
            return card()
                .gap(ui(22.))
                .child(header)
                .child(note(Tone::Warn).child(tr_n!(
                    n,
                    "{count} link leads outside the folder, elsewhere on this computer. Nothing is written yet: leave it out, or create it as it is if you trust this archive?",
                    "{count} links lead outside the folder, elsewhere on this computer. Nothing is written yet: leave them out, or create them as they are if you trust this archive?",
                    count = count(n)
                )))
                .child(listing(&rows))
                .child(
                    footer()
                        .child(
                            button("stop", tr!("Stop"), Fill::Outline, Size::Medium).on_click(
                                cx.listener(|this, _, _, cx| {
                                    this.answer_links(OnLinksOutside::Stop, cx)
                                }),
                            ),
                        )
                        .child(div().flex_1())
                        .child(
                            button("create-links", tr!("Create them"), Fill::Tonal, Size::Medium)
                                .px(ui(20.))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.answer_links(OnLinksOutside::Create, cx)
                                })),
                        )
                        .child(
                            button("skip", tr!("Leave out"), Fill::Accent, Size::Medium)
                                .px(ui(20.))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.answer_links(OnLinksOutside::Skip, cx)
                                })),
                        ),
                );
        }
        if let Some(report) = &self.report {
            let rows = report.rows();
            let listed: String = rows
                .iter()
                .map(|(path, what, why)| format!("{path}\t{what}\t{why}\n"))
                .collect();
            let mut said = Vec::new();
            let damaged = report.damaged.len() as u64;
            if damaged > 0 {
                said.push(tr_n!(
                    damaged,
                    "{count} file’s data was damaged.",
                    "{count} files’ data was damaged.",
                    count = count(damaged)
                ));
            }
            let left = report.left_out.len() as u64;
            if left > 0 {
                said.push(tr_n!(
                    left,
                    "{count} file was left out.",
                    "{count} files were left out.",
                    count = count(left)
                ));
            }
            let renamed = report.renamed.len() as u64;
            if renamed > 0 {
                said.push(tr_n!(
                    renamed,
                    "{count} file was renamed, as its name can’t be used here.",
                    "{count} files were renamed, as their names can’t be used here.",
                    count = count(renamed)
                ));
            }
            let gone = report.gone.len() as u64;
            if gone > 0 {
                said.push(tr_n!(
                    gone,
                    "{count} file was gone by the time it was read, and left out.",
                    "{count} files were gone by the time they were read, and left out.",
                    count = count(gone)
                ));
            }
            said.push(if self.job.create.is_some() {
                tr!("The rest is in the archive.")
            } else {
                tr!("The rest was extracted.")
            });
            let what = rows
                .iter()
                .map(|(path, what, _)| (path.clone(), what.clone(), String::new()))
                .collect::<Vec<_>>();
            return card()
                .gap(ui(22.))
                .child(header)
                .child(note(Tone::Warn).child(said.join(" ")))
                .child(listing(&what))
                .child(
                    footer()
                        .child(
                            button("copy-list", tr!("Copy the list"), Fill::Text, Size::Medium)
                                .px(ui(20.))
                                .on_click(cx.listener(move |_, _, _, cx| {
                                    cx.write_to_clipboard(ClipboardItem::new_string(listed.clone()))
                                })),
                        )
                        .child(div().flex_1())
                        .child(
                            button("close", tr!("Close"), Fill::Accent, Size::Medium)
                                .px(ui(28.))
                                .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close))),
                        ),
                );
        }
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
        // One waiting for its turn is a job that will run, not a sample.
        let real = self.running.is_some() || self.queued;
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
                            .child(if self.paused || self.queued || speed == 0 {
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
