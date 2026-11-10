//! The audit log's records, a line a job and the latest first, with what each
//! was of and how it went, and whether the chain holds: a tab of Browse, there
//! while the log is kept (docs/audit.md). It reads the log and never changes it.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use gpui::{
    AnyElement, Context, Entity, FontWeight, Render, Subscription, UniformListScrollHandle, Window,
    div, prelude::*, relative, rgb, uniform_list,
};
use packwing_core::audit::{self, Report};
use packwing_i18n::{tr, tr_n};
use serde_json::{Map, Value};

use super::panel;
use crate::colors::colors;
use crate::input::{Changed, TextInput, said};
use crate::model::date;
use crate::text_size::ui;
use crate::widgets::{
    Chosen, Fill, Select, SelectLook, Size, Tone, button, chip, icon, icon_button, note,
};

/// The most records the page reads: past them, the oldest are left out, and
/// the page says so.
pub const MOST_RECORDS: usize = 50_000;

/// How many days back the page reads, as chosen: 0 for all.
const DAYS: [u32; 4] = [1, 7, 30, 0];

/// What the page reads at first, of [`DAYS`]: the last 7 days.
const FIRST_DAYS: usize = 1;

fn days_names() -> [String; 4] {
    [
        tr!("Today"),
        tr!("Last 7 days"),
        tr!("Last 30 days"),
        tr!("All days"),
    ]
}

fn results_names() -> [String; 3] {
    [tr!("All results"), tr!("Done"), tr!("Not done")]
}

/// How a job went, or that a record is no job.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Ok,
    /// Done, with some left out or damaged.
    Partial,
    Failed,
    Cancelled,
    /// Started, and no end recorded: packwing stopped with it, or it runs.
    Unfinished,
    /// A record of its own, as the log started.
    Said,
}

impl Outcome {
    /// What the records' `result` says: `ok`, `partial`…
    pub fn code(self) -> &'static str {
        match self {
            Outcome::Ok => "ok",
            Outcome::Partial => "partial",
            Outcome::Failed => "failed",
            Outcome::Cancelled => "cancelled",
            Outcome::Unfinished => "unfinished",
            Outcome::Said => "",
        }
    }

    fn label(self) -> String {
        match self {
            Outcome::Ok => tr!("Done"),
            Outcome::Partial => tr!("Partly done"),
            Outcome::Failed => tr!("Failed"),
            Outcome::Cancelled => tr!("Cancelled"),
            Outcome::Unfinished => tr!("No end recorded"),
            Outcome::Said => String::new(),
        }
    }

    /// Whether it went as asked.
    pub fn done(self) -> bool {
        matches!(self, Outcome::Ok | Outcome::Said)
    }
}

/// A job, its start and its end, or a record of its own.
#[derive(Clone, Debug)]
pub struct Line {
    /// When it started, as the record says: RFC 3339 in UTC.
    pub time: String,
    pub action: String,
    pub outcome: Outcome,
    /// What it was of: the archive, the file, or where to.
    pub subject: String,
    pub start: Option<Map<String, Value>>,
    pub end: Option<Map<String, Value>>,
    /// All its text, in lower case, for the search.
    text: String,
}

fn text_of(r: Option<&Map<String, Value>>, key: &str) -> Option<String> {
    match r?.get(key)? {
        Value::String(s) => Some(s.clone()),
        Value::Null => None,
        other => Some(other.to_string()),
    }
}

/// Every string in `value`, for the search.
fn words(value: &Value, into: &mut String) {
    match value {
        Value::String(s) => {
            into.push_str(&s.to_lowercase());
            into.push('\n');
        }
        Value::Array(all) => all.iter().for_each(|v| words(v, into)),
        Value::Object(map) => map.values().for_each(|v| words(v, into)),
        _ => {}
    }
}

impl Line {
    fn of(start: Option<Map<String, Value>>, end: Option<Map<String, Value>>, job: bool) -> Line {
        let first = start.as_ref().or(end.as_ref());
        let action = text_of(first, "action").unwrap_or_default();
        let outcome = if !job {
            Outcome::Said
        } else {
            match text_of(end.as_ref(), "result").as_deref() {
                None => Outcome::Unfinished,
                Some("ok") => Outcome::Ok,
                Some("partial") => Outcome::Partial,
                Some("cancelled") => Outcome::Cancelled,
                Some(_) => Outcome::Failed,
            }
        };
        let subject = [
            "source",
            "archive",
            "file",
            "from",
            "destination",
            "folder",
            "part",
        ]
        .iter()
        .find_map(|k| text_of(start.as_ref(), k).or_else(|| text_of(end.as_ref(), k)))
        .unwrap_or_default();
        let subject = match text_of(start.as_ref(), "entry") {
            Some(entry) => format!("{subject} › {entry}"),
            None => subject,
        };
        let mut text = String::new();
        for r in [&start, &end].into_iter().flatten() {
            r.values().for_each(|v| words(v, &mut text));
        }
        Line {
            time: text_of(first, "time").unwrap_or_default(),
            action,
            outcome,
            subject,
            start,
            end,
            text,
        }
    }

    /// What was done, in words.
    pub fn label(&self) -> String {
        let preview = text_of(self.start.as_ref(), "via").as_deref() == Some("preview");
        action_label(&self.action, preview)
    }

    pub fn error(&self) -> Option<String> {
        text_of(self.end.as_ref(), "error")
    }

    fn duration_ms(&self) -> Option<u64> {
        self.end.as_ref()?.get("duration_ms")?.as_u64()
    }

    fn says(&self, key: &str) -> String {
        text_of(self.start.as_ref(), key)
            .or_else(|| text_of(self.end.as_ref(), key))
            .unwrap_or_default()
    }
}

/// `action` in words; a file read to be shown beside its archive is a preview.
pub fn action_label(action: &str, preview: bool) -> String {
    match action {
        "extract" => tr!("Extract"),
        "create" => tr!("Make an archive"),
        "convert" => tr!("Convert"),
        "test" => tr!("Test"),
        "open_entry" => tr!("Open a file from an archive"),
        "read_entry" if preview => tr!("Preview a file"),
        "read_entry" => tr!("Read a file from an archive"),
        "add" => tr!("Add to an archive"),
        "delete" => tr!("Delete from an archive"),
        "rename" => tr!("Rename in an archive"),
        "copy" => tr!("Copy"),
        "move" => tr!("Move"),
        "sign" => tr!("Sign"),
        "verify" => tr!("Check a signature"),
        "keygen" => tr!("Make a signing key"),
        "split" => tr!("Split into parts"),
        "join" => tr!("Join parts"),
        "checksum" => tr!("Checksums"),
        "audit.started" => tr!("Audit log started"),
        "audit.stopped" => tr!("Audit log stopped"),
        "audit.changed" => tr!("Audit log changed"),
        "log.pruned" => tr!("Old records taken away"),
        other => other.to_owned(),
    }
}

/// The jobs of `records`, oldest first as written, a line each: a start and
/// its end are one line, the latest first.
pub fn lines(records: Vec<Map<String, Value>>) -> Vec<Line> {
    struct Held {
        start: Option<Map<String, Value>>,
        end: Option<Map<String, Value>>,
        job: bool,
    }
    let mut held: Vec<Held> = Vec::new();
    let mut of_job: HashMap<String, usize> = HashMap::new();
    for r in records {
        let end = r.get("phase").and_then(Value::as_str) == Some("end");
        match r.get("job").and_then(Value::as_str).map(str::to_owned) {
            Some(job) => match of_job.get(&job) {
                Some(&i) if end && held[i].end.is_none() => held[i].end = Some(r),
                _ => {
                    of_job.insert(job, held.len());
                    held.push(if end {
                        Held {
                            start: None,
                            end: Some(r),
                            job: true,
                        }
                    } else {
                        Held {
                            start: Some(r),
                            end: None,
                            job: true,
                        }
                    });
                }
            },
            None => held.push(Held {
                start: Some(r),
                end: None,
                job: false,
            }),
        }
    }
    held.into_iter()
        .rev()
        .map(|h| Line::of(h.start, h.end, h.job))
        .collect()
}

/// What was read of the log.
#[derive(Default)]
pub struct Loaded {
    pub lines: Vec<Line>,
    pub records: usize,
    /// Older records there are, left out past [`MOST_RECORDS`].
    pub cut: bool,
}

/// The records in `folder` from the day `since` on, the latest
/// [`MOST_RECORDS`] of them.
pub fn load(folder: &Path, since: Option<&str>) -> std::io::Result<Loaded> {
    if !folder.is_dir() {
        return Ok(Loaded::default());
    }
    let mut days: Vec<Vec<Map<String, Value>>> = Vec::new();
    let mut count = 0;
    let mut cut = false;
    for (name, path) in audit::day_files(folder)?.iter().rev() {
        if since.is_some_and(|s| name.as_str() < s) {
            break;
        }
        if count == MOST_RECORDS {
            cut = true;
            break;
        }
        let mut day = Vec::new();
        for line in BufReader::new(File::open(path)?).split(b'\n') {
            if let Ok(Value::Object(r)) = serde_json::from_slice(&line?) {
                day.push(r);
            }
        }
        if count + day.len() > MOST_RECORDS {
            day.drain(..count + day.len() - MOST_RECORDS);
            cut = true;
        }
        count += day.len();
        days.push(day);
        if cut {
            break;
        }
    }
    let records: Vec<_> = days.into_iter().rev().flatten().collect();
    Ok(Loaded {
        records: records.len(),
        lines: lines(records),
        cut,
    })
}

/// The first day of the last `days` to `now`, in UTC as the files are named;
/// none for all.
pub fn since(days: u32, now: SystemTime) -> Option<String> {
    let secs = now.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
    (days > 0).then(|| date(secs - i64::from(days - 1) * 86_400))
}

/// Which results are shown.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Results {
    #[default]
    All,
    Done,
    NotDone,
}

/// What the page shows of what it read.
#[derive(Clone, Debug, Default)]
pub struct Filter {
    pub action: Option<String>,
    pub results: Results,
    /// Words to find in its paths, names and error.
    pub query: String,
}

impl Filter {
    pub fn shows(&self, line: &Line) -> bool {
        self.action.as_ref().is_none_or(|a| *a == line.action)
            && match self.results {
                Results::All => true,
                Results::Done => line.outcome.done(),
                Results::NotDone => !line.outcome.done(),
            }
            && self
                .query
                .split_whitespace()
                .all(|w| line.text.contains(&w.to_lowercase()))
    }
}

/// `lines` as a spreadsheet: a job a row.
pub fn csv<'a>(lines: impl IntoIterator<Item = &'a Line>) -> String {
    let cell = |s: String| {
        if s.contains([',', '"', '\n', '\r']) {
            format!("\"{}\"", s.replace('"', "\"\""))
        } else {
            s
        }
    };
    let mut out = String::from("time,action,result,subject,error,duration_ms,user,host,app,job\n");
    for l in lines {
        let row = [
            l.time.clone(),
            l.action.clone(),
            l.outcome.code().to_owned(),
            l.subject.clone(),
            l.error().unwrap_or_default(),
            l.duration_ms().map(|d| d.to_string()).unwrap_or_default(),
            l.says("user"),
            l.says("host"),
            l.says("app"),
            l.says("job"),
        ];
        out.push_str(&row.map(cell).join(","));
        out.push('\n');
    }
    out
}

/// `2026-11-08T05:01:02.123Z` as `2026-11-08 05:01:02`.
fn shown_time(time: &str) -> String {
    time.get(..19).unwrap_or(time).replacen('T', " ", 1)
}

fn duration(ms: u64) -> String {
    match ms {
        0..1_000 => format!("{ms} ms"),
        1_000..60_000 => format!("{:.1} s", ms as f64 / 1000.),
        _ => format!("{} min {} s", ms / 60_000, ms / 1000 % 60),
    }
}

/// A field's value, to read: a list a name a line.
fn value_text(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Array(all) if all.iter().all(Value::is_string) => all
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join("\n"),
        other => other.to_string(),
    }
}

/// Whether the chain holds, once checked.
enum Chain {
    Checking,
    Checked(Report),
    Unread(String),
}

/// How the reading of the log went.
enum Read {
    Going,
    Done,
    Failed(String),
}

pub struct AuditPage {
    lines: Vec<Line>,
    records: usize,
    cut: bool,
    /// Those shown, by their place in `lines`.
    shown: Vec<usize>,
    /// The one whose records are shown beside, by its place in `lines`.
    chosen: Option<usize>,
    reading: Read,
    chain: Chain,
    /// Where the last CSV went, or why it did not.
    saved: Option<Result<PathBuf, String>>,
    /// The last reading asked: an older one's results are dropped.
    asked: u64,
    days: Entity<Select>,
    actions: Entity<Select>,
    /// The actions the second select offers after "All actions".
    action_codes: Vec<String>,
    results: Entity<Select>,
    search: Entity<TextInput>,
    scroll: UniformListScrollHandle,
    _chosen: Vec<Subscription>,
}

impl AuditPage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let days = cx
            .new(|_| Select::new(&days_names(), FIRST_DAYS, SelectLook::Pill).marked("audit-days"));
        let actions = cx.new(|_| {
            Select::new(&[tr!("All actions")], 0, SelectLook::Pill).marked("audit-action")
        });
        let results =
            cx.new(|_| Select::new(&results_names(), 0, SelectLook::Pill).marked("audit-result"));
        let search = said(cx, || tr!("Find a path, a name or an error"));
        let subscriptions = vec![
            cx.subscribe(&days, |this: &mut Self, _, _: &Chosen, cx| this.reload(cx)),
            cx.subscribe(&actions, |this: &mut Self, _, _: &Chosen, cx| {
                this.refilter(cx)
            }),
            cx.subscribe(&results, |this: &mut Self, _, _: &Chosen, cx| {
                this.refilter(cx)
            }),
            cx.subscribe(&search, |this: &mut Self, _, _: &Changed, cx| {
                this.refilter(cx)
            }),
        ];
        AuditPage {
            lines: Vec::new(),
            records: 0,
            cut: false,
            shown: Vec::new(),
            chosen: None,
            reading: Read::Done,
            chain: Chain::Checking,
            saved: None,
            asked: 0,
            days,
            actions,
            action_codes: Vec::new(),
            results,
            search,
            scroll: UniformListScrollHandle::new(),
            _chosen: subscriptions,
        }
    }

    /// Reads the log again, and checks its chain.
    pub fn reload(&mut self, cx: &mut Context<Self>) {
        let Some(folder) = audit::config().map(|c| c.folder) else {
            return;
        };
        self.asked += 1;
        let asked = self.asked;
        let days = DAYS[self.days.read(cx).chosen().min(DAYS.len() - 1)];
        let since = since(days, SystemTime::now());
        self.reading = Read::Going;
        self.chain = Chain::Checking;
        self.saved = None;
        cx.notify();
        let read = {
            let folder = folder.clone();
            cx.background_executor()
                .spawn(async move { load(&folder, since.as_deref()) })
        };
        let checked = cx
            .background_executor()
            .spawn(async move { audit::verify(&folder) });
        cx.spawn(async move |this, cx| {
            let read = read.await;
            let _ = this.update(cx, |this, cx| {
                if this.asked == asked {
                    this.show(read, cx);
                }
            });
            let checked = checked.await;
            let _ = this.update(cx, |this, cx| {
                if this.asked == asked {
                    this.chain = match checked {
                        Ok(report) => Chain::Checked(report),
                        Err(e) => Chain::Unread(e.to_string()),
                    };
                    cx.notify();
                }
            });
        })
        .detach();
    }

    fn show(&mut self, read: std::io::Result<Loaded>, cx: &mut Context<Self>) {
        match read {
            Ok(loaded) => {
                self.lines = loaded.lines;
                self.records = loaded.records;
                self.cut = loaded.cut;
                self.reading = Read::Done;
            }
            Err(e) => {
                self.lines.clear();
                self.records = 0;
                self.cut = false;
                self.reading = Read::Failed(e.to_string());
            }
        }
        self.chosen = None;
        // The actions there are, in the order of their names.
        let was = self.action_filter(cx);
        let mut codes: Vec<String> = self.lines.iter().map(|l| l.action.clone()).collect();
        codes.sort();
        codes.dedup();
        codes.sort_by_cached_key(|a| action_label(a, false));
        let names: Vec<String> = std::iter::once(tr!("All actions"))
            .chain(codes.iter().map(|a| action_label(a, false)))
            .collect();
        let chosen = was
            .and_then(|a| codes.iter().position(|c| *c == a))
            .map_or(0, |i| i + 1);
        self.action_codes = codes;
        self.actions
            .update(cx, |s, cx| s.set_options(&names, chosen, cx));
        self.refilter(cx);
    }

    fn action_filter(&self, cx: &Context<Self>) -> Option<String> {
        let i = self.actions.read(cx).chosen();
        i.checked_sub(1)
            .and_then(|i| self.action_codes.get(i))
            .cloned()
    }

    fn filter(&self, cx: &Context<Self>) -> Filter {
        Filter {
            action: self.action_filter(cx),
            results: match self.results.read(cx).chosen() {
                1 => Results::Done,
                2 => Results::NotDone,
                _ => Results::All,
            },
            query: self.search.read(cx).text().to_owned(),
        }
    }

    fn refilter(&mut self, cx: &mut Context<Self>) {
        let filter = self.filter(cx);
        self.shown = (0..self.lines.len())
            .filter(|i| filter.shows(&self.lines[*i]))
            .collect();
        if self.chosen.is_some_and(|c| !self.shown.contains(&c)) {
            self.chosen = None;
        }
        cx.notify();
    }

    /// Those shown, as the page shows them.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn shown(&self) -> Vec<&Line> {
        self.shown.iter().map(|i| &self.lines[*i]).collect()
    }

    /// Shows `lines`, as the screenshots draw them, with a chain that holds.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn show_samples(
        &mut self,
        lines: Vec<Line>,
        chosen: Option<usize>,
        cx: &mut Context<Self>,
    ) {
        self.asked += 1;
        let records = lines
            .iter()
            .map(|l| usize::from(l.start.is_some()) + usize::from(l.end.is_some()))
            .sum();
        self.show(
            Ok(Loaded {
                lines,
                records,
                cut: false,
            }),
            cx,
        );
        self.chain = Chain::Checked(Report {
            files: 3,
            records,
            anchor: String::new(),
            anchor_explained: true,
            breaks: Vec::new(),
        });
        self.chosen = chosen;
        cx.notify();
    }

    /// Asks where to save those shown as CSV, and saves them there.
    fn save_csv(&mut self, cx: &mut Context<Self>) {
        let text = csv(self.shown.iter().map(|i| &self.lines[*i]));
        let folder = dirs::download_dir()
            .filter(|d| d.is_dir())
            .or_else(dirs::home_dir)
            .unwrap_or_else(|| PathBuf::from("."));
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let name = format!("packwing-audit-{}.csv", date(secs));
        let picked = cx.prompt_for_new_path(&folder, Some(&name));
        cx.spawn(async move |this, cx| {
            let Ok(Ok(Some(path))) = picked.await else {
                return;
            };
            let written = {
                let path = path.clone();
                cx.background_executor()
                    .spawn(async move { std::fs::write(&path, text) })
                    .await
            };
            let _ = this.update(cx, |this, cx| {
                this.saved = Some(written.map(|()| path).map_err(|e| e.to_string()));
                cx.notify();
            });
        })
        .detach();
    }

    fn toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let c = colors();
        let search = div()
            .flex()
            .flex_1()
            .min_w(ui(220.))
            .items_center()
            .gap(ui(10.))
            .h(ui(40.))
            .px(ui(16.))
            .rounded_full()
            .bg(rgb(c.field))
            .text_size(ui(14.))
            .child(icon("search", 18., c.muted))
            .child(self.search.clone());
        div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(ui(8.))
            .child(self.days.clone())
            .child(self.actions.clone())
            .child(self.results.clone())
            .child(search)
            .child(
                icon_button("audit-refresh", "refresh", 40.)
                    .tooltip(crate::widgets::tip(tr!("Read again")))
                    .on_click(cx.listener(|this, _, _, cx| this.reload(cx))),
            )
            .child(
                button(
                    "audit-save",
                    tr!("Save as CSV…"),
                    Fill::Outline,
                    Size::Medium,
                )
                .on_click(cx.listener(|this, _, _, cx| this.save_csv(cx))),
            )
    }

    /// What is said of the log: whether its chain holds, what was left out,
    /// where the CSV went.
    fn notes(&self) -> Vec<AnyElement> {
        let c = colors();
        let mut notes = Vec::new();
        let chain = match &self.chain {
            Chain::Checking => div()
                .px(ui(12.))
                .text_size(ui(13.))
                .text_color(rgb(c.muted))
                .child(tr!("Checking that no record was changed…")),
            Chain::Unread(why) => note(Tone::Bad).child(tr!(
                "The audit log cannot be read: {why}",
                why = why.clone()
            )),
            Chain::Checked(report) if !report.breaks.is_empty() => {
                let first = &report.breaks[0];
                note(Tone::Bad).child(tr_n!(
                    report.breaks.len(),
                    "A record was changed, taken out or put in by hand: in {file}, line {line}.",
                    "Records were changed, taken out or put in by hand, in {n} places: the first in {file}, line {line}.",
                    file = first.file.clone(),
                    line = first.line,
                ))
            }
            Chain::Checked(report) if !report.anchor_explained => note(Tone::Warn).child(tr!(
                "The oldest record follows one that is gone, and no record says it was taken away as past its time."
            )),
            Chain::Checked(report) => note(Tone::Good).child(tr_n!(
                report.records,
                "{n} record, and none was changed or taken out.",
                "{n} records, and none was changed or taken out.",
            )),
        };
        notes.push(
            chain
                .debug_selector(|| "audit-chain".to_owned())
                .into_any_element(),
        );
        if self.cut {
            notes.push(
                note(Tone::Warn)
                    .child(tr!(
                        "Only the latest {most} records are shown: choose fewer days, or read all with pkw audit list.",
                        most = MOST_RECORDS
                    ))
                    .into_any_element(),
            );
        }
        match &self.saved {
            Some(Ok(path)) => notes.push(
                note(Tone::Good)
                    .child(tr!("Saved to {path}.", path = path.display().to_string()))
                    .into_any_element(),
            ),
            Some(Err(why)) => notes.push(
                note(Tone::Bad)
                    .child(tr!("Could not save: {why}", why = why.clone()))
                    .into_any_element(),
            ),
            None => {}
        }
        notes
    }

    fn row(&self, at: usize, cx: &mut Context<Self>) -> AnyElement {
        let c = colors();
        let Some(&i) = self.shown.get(at) else {
            return div().into_any_element();
        };
        let line = &self.lines[i];
        let (ground, ink, mark) = match line.outcome {
            Outcome::Ok => (c.tonal, c.on_tonal, "check"),
            Outcome::Failed => (c.bad_bg, c.bad_text, "warning"),
            Outcome::Partial | Outcome::Cancelled | Outcome::Unfinished => {
                (c.warn_bg, c.warn_text, "warning")
            }
            Outcome::Said => (c.well, c.muted, "audit"),
        };
        let how = match (line.outcome, line.duration_ms()) {
            (Outcome::Said, _) => String::new(),
            (o, Some(ms)) => format!("{} · {}", o.label(), duration(ms)),
            (o, None) => o.label(),
        };
        let on = self.chosen == Some(i);
        div()
            .debug_selector(move || format!("audit-row-{at}"))
            .id(("audit-row", at))
            .w_full()
            .flex()
            .items_center()
            .gap(ui(14.))
            .h(ui(64.))
            .px(ui(14.))
            .rounded(ui(18.))
            .cursor_pointer()
            .when(on, |d| d.bg(rgb(c.chosen)))
            .when(!on, |d| d.hover(|s| s.bg(rgb(c.hover))))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.chosen = if this.chosen == Some(i) {
                    None
                } else {
                    Some(i)
                };
                cx.notify();
            }))
            .child(
                div()
                    .size(ui(40.))
                    .flex_shrink_0()
                    .rounded(ui(14.))
                    .bg(rgb(ground))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(icon(mark, 20., ink)),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(
                        div()
                            .truncate()
                            .text_size(ui(15.))
                            .font_weight(FontWeight::MEDIUM)
                            .child(line.label()),
                    )
                    .child(
                        div()
                            .truncate()
                            .text_size(ui(13.))
                            .text_color(rgb(c.muted))
                            .child(line.subject.clone()),
                    ),
            )
            .child(
                div()
                    .flex_shrink_0()
                    .flex()
                    .flex_col()
                    .items_end()
                    .text_size(ui(13.))
                    .child(div().text_color(rgb(c.muted)).child(shown_time(&line.time)))
                    .child(
                        div()
                            .text_color(rgb(if line.outcome.done() { c.muted } else { ink }))
                            .child(how),
                    ),
            )
            .into_any_element()
    }

    /// The records of the line at `i`, beside the list.
    fn detail(&self, i: usize, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let c = colors();
        let line = &self.lines[i];
        let section = |title: String, r: &Map<String, Value>| {
            let fields = r
                .iter()
                .filter(|(k, _)| !matches!(k.as_str(), "v" | "prev" | "action" | "phase"))
                .map(|(k, v)| {
                    div()
                        .flex()
                        .flex_col()
                        .gap(ui(2.))
                        .child(
                            div()
                                .text_size(ui(12.))
                                .text_color(rgb(c.muted))
                                .child(k.clone()),
                        )
                        .child(
                            div()
                                .text_size(ui(13.))
                                .line_height(relative(1.4))
                                .child(value_text(v)),
                        )
                });
            div()
                .flex()
                .flex_col()
                .gap(ui(10.))
                .child(
                    div()
                        .text_size(ui(14.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(title),
                )
                .children(fields)
        };
        let (ground, ink) = match line.outcome {
            Outcome::Ok => (c.tonal, c.on_tonal),
            Outcome::Failed => (c.bad_bg, c.bad_text),
            Outcome::Said => (c.well, c.muted),
            _ => (c.warn_bg, c.warn_text),
        };
        let said_how =
            (line.outcome != Outcome::Said).then(|| chip(line.outcome.label(), ground, ink));
        let started = line.start.as_ref().map(|r| {
            let title = if line.outcome == Outcome::Said {
                tr!("Record")
            } else {
                tr!("Started")
            };
            section(title, r)
        });
        let ended = line.end.as_ref().map(|r| section(tr!("Ended"), r));
        panel()
            .id("audit-detail")
            .debug_selector(|| "audit-detail".to_owned())
            .w(relative(0.35))
            .min_w(ui(320.))
            .flex_shrink_0()
            .min_h_0()
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .gap(ui(16.))
            .p(ui(20.))
            .child(
                div()
                    .flex()
                    .items_start()
                    .justify_between()
                    .gap(ui(8.))
                    .child(
                        div()
                            .text_size(ui(18.))
                            .font_weight(FontWeight::BOLD)
                            .child(line.label()),
                    )
                    .child(
                        icon_button("audit-detail-close", "x", 32.).on_click(cx.listener(
                            |this, _, _, cx| {
                                this.chosen = None;
                                cx.notify();
                            },
                        )),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(ui(8.))
                    .children(said_how)
                    .child(
                        div()
                            .text_size(ui(13.))
                            .text_color(rgb(c.muted))
                            .child(format!("{} UTC", shown_time(&line.time))),
                    ),
            )
            .children(line.error().map(|e| note(Tone::Bad).child(e)))
            .children(started)
            .children(ended)
    }
}

impl Render for AuditPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let empty = |text: String| {
            div()
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .p(ui(16.))
                .text_size(ui(14.))
                .text_color(rgb(c.muted))
                .child(text)
                .into_any_element()
        };
        let list = match &self.reading {
            Read::Going if self.lines.is_empty() => empty(tr!("Reading the audit log…")),
            Read::Failed(why) => empty(tr!(
                "The audit log cannot be read: {why}",
                why = why.clone()
            )),
            _ if self.lines.is_empty() => empty(tr!("No records in these days.")),
            _ if self.shown.is_empty() => empty(tr!("No record is as chosen.")),
            _ => uniform_list(
                "audit-rows",
                self.shown.len(),
                cx.processor(|this, range: std::ops::Range<usize>, _, cx| {
                    range.map(|i| this.row(i, cx)).collect::<Vec<_>>()
                }),
            )
            .track_scroll(&self.scroll)
            .flex_1()
            .min_h_0()
            .into_any_element(),
        };
        let detail = self.chosen.map(|i| self.detail(i, cx));
        let toolbar = self.toolbar(cx);
        div()
            .size_full()
            .flex()
            .flex_col()
            .gap(ui(10.))
            .child(toolbar)
            .children(self.notes())
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .gap(ui(crate::widgets::PANE_GAP))
                    .child(
                        panel()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .p(ui(10.))
                            .child(list),
                    )
                    .children(detail),
            )
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use serde_json::json;

    fn record(v: Value) -> Map<String, Value> {
        match v {
            Value::Object(m) => m,
            _ => unreachable!(),
        }
    }

    fn samples() -> Vec<Map<String, Value>> {
        [
            json!({"time": "2026-11-08T05:00:00.000Z", "action": "audit.started", "by": "user", "user": "an"}),
            json!({"time": "2026-11-08T05:01:00.000Z", "action": "extract", "phase": "start", "job": "s-1", "source": "/home/an/Lương.zip", "destination": "/home/an/out", "user": "an"}),
            json!({"time": "2026-11-08T05:02:00.000Z", "action": "test", "phase": "start", "job": "s-2", "source": "/home/an/b.7z"}),
            json!({"time": "2026-11-08T05:02:01.000Z", "action": "test", "phase": "end", "job": "s-2", "result": "failed", "error": "Wrong password, \"b\"", "duration_ms": 1200}),
            json!({"time": "2026-11-08T05:03:00.000Z", "action": "extract", "phase": "end", "job": "s-1", "result": "ok", "files": 2, "duration_ms": 120_000}),
            json!({"time": "2026-11-08T05:04:00.000Z", "action": "read_entry", "phase": "start", "job": "s-3", "source": "/home/an/b.zip", "entry": "in/a.txt", "via": "preview"}),
        ]
        .into_iter()
        .map(record)
        .collect()
    }

    #[test]
    fn a_job_is_a_line_the_latest_first() {
        let lines = lines(samples());
        let actions: Vec<_> = lines.iter().map(|l| l.action.as_str()).collect();
        assert_eq!(actions, ["read_entry", "test", "extract", "audit.started"]);
        assert_eq!(lines[0].outcome, Outcome::Unfinished);
        assert_eq!(lines[0].subject, "/home/an/b.zip › in/a.txt");
        assert_eq!(lines[0].label(), "Preview a file");
        assert_eq!(lines[1].outcome, Outcome::Failed);
        assert_eq!(lines[1].error().as_deref(), Some("Wrong password, \"b\""));
        assert_eq!(lines[2].outcome, Outcome::Ok);
        assert_eq!(lines[2].time, "2026-11-08T05:01:00.000Z", "when it started");
        assert_eq!(lines[2].subject, "/home/an/Lương.zip");
        assert_eq!(lines[3].outcome, Outcome::Said);
    }

    #[test]
    fn what_is_shown_is_as_chosen() {
        let lines = lines(samples());
        let shown = |f: Filter| -> Vec<&str> {
            lines
                .iter()
                .filter(|l| f.shows(l))
                .map(|l| l.action.as_str())
                .collect()
        };
        assert_eq!(shown(Filter::default()).len(), 4);
        let only = |results| Filter {
            results,
            ..Filter::default()
        };
        assert_eq!(shown(only(Results::NotDone)), ["read_entry", "test"]);
        assert_eq!(shown(only(Results::Done)), ["extract", "audit.started"]);
        let found = |query: &str| Filter {
            query: query.into(),
            ..Filter::default()
        };
        assert_eq!(shown(found("lương")), ["extract"]);
        assert_eq!(shown(found("LƯƠNG out")), ["extract"]);
        assert_eq!(shown(found("wrong password")), ["test"]);
        assert!(shown(found("nowhere")).is_empty());
        let action = Filter {
            action: Some("test".into()),
            ..Filter::default()
        };
        assert_eq!(shown(action), ["test"]);
    }

    #[test]
    fn a_csv_row_is_a_job() {
        let text = csv(&lines(samples()));
        let rows: Vec<&str> = text.lines().collect();
        assert_eq!(
            rows[0],
            "time,action,result,subject,error,duration_ms,user,host,app,job"
        );
        assert_eq!(rows.len(), 5);
        assert_eq!(
            rows[2],
            "2026-11-08T05:02:00.000Z,test,failed,/home/an/b.7z,\"Wrong password, \"\"b\"\"\",1200,,,,s-2"
        );
        assert!(
            rows[3]
                .starts_with("2026-11-08T05:01:00.000Z,extract,ok,/home/an/Lương.zip,,120000,an,")
        );
    }

    #[test]
    fn days_are_counted_to_today() {
        let now = UNIX_EPOCH + std::time::Duration::from_secs(1_762_581_600); // 2025-11-08 06:00 UTC
        assert_eq!(since(1, now).as_deref(), Some("2025-11-08"));
        assert_eq!(since(7, now).as_deref(), Some("2025-11-02"));
        assert_eq!(since(0, now), None);
        assert_eq!(
            shown_time("2026-11-08T05:01:02.123Z"),
            "2026-11-08 05:01:02"
        );
        assert_eq!(duration(950), "950 ms");
        assert_eq!(duration(1_200), "1.2 s");
        assert_eq!(duration(125_000), "2 min 5 s");
    }

    #[test]
    fn the_latest_records_are_read_and_the_rest_said_left_out() {
        let dir = std::env::temp_dir().join(format!("packwing-audit-page-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let day = |name: &str, n: usize| {
            let text: String = (0..n)
                .map(|i| {
                    format!(
                        "{{\"time\":\"{name}T00:00:{i:02}.000Z\",\"action\":\"audit.changed\"}}\n"
                    )
                })
                .collect();
            std::fs::write(dir.join(format!("{name}.jsonl")), text).unwrap();
        };
        day("2026-11-06", 3);
        day("2026-11-07", 2);
        day("2026-11-08", 4);
        let all = load(&dir, None).unwrap();
        assert_eq!((all.records, all.cut), (9, false));
        let some = load(&dir, Some("2026-11-07")).unwrap();
        assert_eq!(some.records, 6);
        assert_eq!(some.lines[0].time, "2026-11-08T00:00:03.000Z");
        assert!(load(&dir.join("none"), None).unwrap().lines.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
