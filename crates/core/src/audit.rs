//! The audit log, as packwing's jobs write to it: a record when one starts, and
//! one when it ends, how it went (docs/audit.md). Whether it is kept, and how,
//! is the app's and pkw's to [`set`], from the organization's policy, the plan
//! and the user's choice ([`settled`]); until then, nothing is written.
//!
//! A job that runs inside another, as converting extracts, is the outer job's:
//! one record of each.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError, RwLock};
use std::time::Instant;

use packwing_audit::Log;
pub use packwing_audit::{Break, Config, Report, day_files, default_folder, read, verify};
use packwing_policy::Policy;
use serde_json::{Map, Value, json};

use crate::{Error, Source, Storage};

/// How many days the user's own log is kept unless they choose otherwise.
pub const PERSONAL_RETENTION_DAYS: u32 = 90;

/// What the user may choose of how long, in days: 0 keeps all.
pub const RETENTION_CHOICES: [u32; 4] = [30, 90, 365, 0];

/// The most the user's own log takes: 100 MiB.
pub const PERSONAL_MAX_BYTES: u64 = 100 << 20;

/// The most names of an archive's files a record holds: past it, their count.
const MOST_NAMES: usize = 1000;

/// What the user chose, where it is theirs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chosen {
    pub on: bool,
    pub retention_days: u32,
}

impl Default for Chosen {
    fn default() -> Chosen {
        Chosen {
            on: false,
            retention_days: PERSONAL_RETENTION_DAYS,
        }
    }
}

/// How the log is kept, if it is: the organization's word where it says one,
/// whatever the license; else the user's choice, where their plan has the log.
/// `app` is `packwing` or `pkw`.
pub fn settled(policy: &Policy, plan_has_it: bool, chosen: Chosen, app: &str) -> Option<Config> {
    if !policy.keeps_audit_log(chosen.on && plan_has_it) {
        return None;
    }
    Some(Config {
        folder: folder(policy)?,
        retention_days: policy.audit_log_retention_days(chosen.retention_days),
        max_bytes: policy.audit_log_max_bytes(PERSONAL_MAX_BYTES),
        entries: policy.audit_log_names_entries(),
        app: app.to_owned(),
        version: env!("CARGO_PKG_VERSION").to_owned(),
    })
}

/// `path` with `{user}` and `{host}` in it given, as a policy writes one folder
/// for every person.
fn placed(path: &str) -> String {
    let safe = |s: String| s.replace(['/', '\\', ':'], "_");
    path.replace("{user}", &safe(packwing_audit::user().0))
        .replace("{host}", &safe(packwing_audit::host()))
}

/// The log this process writes to, if any.
fn log() -> &'static RwLock<Option<Arc<Log>>> {
    static LOG: RwLock<Option<Arc<Log>>> = RwLock::new(None);
    &LOG
}

/// Why the last record could not be written, if it could not.
fn failure() -> &'static Mutex<Option<String>> {
    static FAILURE: Mutex<Option<String>> = Mutex::new(None);
    &FAILURE
}

/// Keeps the log as `config` says, from now on, or none, and takes away what
/// is past its time: a folder's listing, and the few files it takes.
pub fn set(config: Option<Config>) {
    let log = {
        let mut held = log().write().unwrap_or_else(PoisonError::into_inner);
        if held.as_ref().map(|l| l.config()) == config.as_ref() {
            return;
        }
        *held = config.map(|c| Arc::new(Log::new(c)));
        held.clone()
    };
    if let Some(log) = log
        && let Err(e) = log.prune()
    {
        failed(&e.to_string());
    }
}

/// Where the log is, or would be: the organization's folder, else the system's.
pub fn folder(policy: &Policy) -> Option<PathBuf> {
    match policy.audit_log_path() {
        Some(path) => Some(PathBuf::from(placed(path))),
        None => default_folder(),
    }
}

/// How the log is kept now, if it is.
pub fn config() -> Option<Config> {
    current().map(|l| l.config().clone())
}

/// Why the last record could not be written, if it could not: the app says so.
pub fn last_failure() -> Option<String> {
    failure()
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()
}

fn failed(why: &str) {
    *failure().lock().unwrap_or_else(PoisonError::into_inner) = Some(why.to_owned());
}

fn current() -> Option<Arc<Log>> {
    log().read().unwrap_or_else(PoisonError::into_inner).clone()
}

/// Whether records name an archive's files.
pub fn names_entries() -> bool {
    current().is_some_and(|l| l.config().entries)
}

/// Writes one record of `action`, which is no job: the log turned on or off,
/// how long it is kept.
pub fn record(action: &str, fields: Map<String, Value>) {
    if let Some(log) = current() {
        let mut record = Map::new();
        record.insert("action".into(), json!(action));
        record.extend(fields);
        write(&log, record);
    }
}

fn write(log: &Log, record: Map<String, Value>) {
    match log.append(record) {
        Ok(()) => *failure().lock().unwrap_or_else(PoisonError::into_inner) = None,
        Err(e) => failed(&e.to_string()),
    }
}

thread_local! {
    /// What the job running on this thread has noted for its end, if one runs.
    static NOTES: RefCell<Option<Map<String, Value>>> = const { RefCell::new(None) };
}

/// A job being recorded.
pub(crate) struct Job {
    log: Arc<Log>,
    id: String,
    action: &'static str,
    started: Instant,
}

/// Starts recording a job of `action`, if the log is kept and no job runs on
/// this thread already: its start is written now, with `fields`.
pub(crate) fn begin(action: &'static str, fields: Map<String, Value>) -> Option<Job> {
    let log = current()?;
    let outer = NOTES.with(|n| {
        let mut n = n.borrow_mut();
        let running = n.is_some();
        if !running {
            *n = Some(Map::new());
        }
        running
    });
    if outer {
        return None;
    }
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    let id = format!(
        "{}-{}",
        log.session(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    );
    let mut record = Map::new();
    record.insert("action".into(), json!(action));
    record.insert("phase".into(), json!("start"));
    record.insert("job".into(), json!(id));
    record.extend(fields);
    write(&log, record);
    Some(Job {
        log,
        id,
        action,
        started: Instant::now(),
    })
}

/// Notes `value` as `key` for the end of the job running on this thread, if one
/// is recorded: what is known only once it runs, as an archive's format.
pub(crate) fn note(key: &str, value: Value) {
    NOTES.with(|n| {
        if let Some(notes) = n.borrow_mut().as_mut() {
            notes.insert(key.to_owned(), value);
        }
    });
}

/// Ends recording `job`, if it was, with how `result` went: `ok`, `failed`,
/// `cancelled`, or what `more` says of a job done (`partial`, where some was
/// left out), with what else it says.
pub(crate) fn end<T>(
    job: Option<Job>,
    result: &Result<T, Error>,
    more: impl FnOnce(&T) -> Map<String, Value>,
) {
    let Some(job) = job else {
        return;
    };
    let notes = NOTES.with(|n| n.borrow_mut().take()).unwrap_or_default();
    let mut record = Map::new();
    record.insert("action".into(), json!(job.action));
    record.insert("phase".into(), json!("end"));
    record.insert("job".into(), json!(job.id));
    record.insert(
        "duration_ms".into(),
        json!(job.started.elapsed().as_millis() as u64),
    );
    match result {
        Ok(done) => {
            record.insert("result".into(), json!("ok"));
            record.extend(notes);
            record.extend(more(done));
        }
        Err(Error::Cancelled) => {
            record.insert("result".into(), json!("cancelled"));
            record.extend(notes);
        }
        Err(e) => {
            record.insert("result".into(), json!("failed"));
            record.insert("error".into(), json!(e.to_string()));
            record.extend(notes);
        }
    }
    write(&job.log, record);
}

/// Fields, from pairs.
pub(crate) fn fields<const N: usize>(pairs: [(&str, Value); N]) -> Map<String, Value> {
    pairs
        .into_iter()
        .filter(|(_, v)| !v.is_null())
        .map(|(k, v)| (k.to_owned(), v))
        .collect()
}

/// Where `source` is, as the user would write it.
pub(crate) fn source(source: &Source) -> Value {
    match source {
        Source::File(path) => json!(path_text(path)),
        Source::Stored(s) => {
            let (storage, path) = s.place();
            json!(place(storage.as_ref(), &path))
        }
    }
}

/// A path on this computer, as text: whole, as one given from where pkw runs
/// tells nothing later.
pub(crate) fn path_text(path: &Path) -> String {
    std::path::absolute(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .into_owned()
}

/// `path` on `storage`, as the user would write it: on this computer, its path.
pub(crate) fn place(storage: &dyn Storage, path: &[String]) -> String {
    storage
        .local_path(path)
        .map(|p| path_text(&p))
        .unwrap_or_else(|| storage.show_path(path))
}

/// What the user chose to work on, as files or names: the first [`MOST_NAMES`],
/// and how many there are. Always recorded, as it is what the job is of.
pub(crate) fn list<I, S>(all: I) -> Value
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    capped(all)
}

fn capped<I, S>(all: I) -> Value
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut kept = Vec::new();
    let mut count = 0usize;
    for name in all {
        if kept.len() < MOST_NAMES {
            kept.push(name.into());
        }
        count += 1;
    }
    if count > kept.len() {
        json!({ "names": kept, "count": count })
    } else {
        json!(kept)
    }
}

/// Names of an archive's files, if records name them: the first
/// [`MOST_NAMES`], and how many there are; else nothing.
pub(crate) fn names<I, S>(all: I) -> Value
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    if !names_entries() {
        return Value::Null;
    }
    capped(all)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use packwing_policy::{Found, Value as PolicyValue};

    fn policy(values: &[(&str, PolicyValue)]) -> Policy {
        Policy::of(Found {
            origin: "test".into(),
            values: values
                .iter()
                .map(|(k, v)| ((*k).to_owned(), v.clone()))
                .collect(),
            failure: None,
        })
    }

    #[test]
    fn free_keeps_no_log_unless_the_organization_says() {
        let none = policy(&[]);
        let on = Chosen {
            on: true,
            ..Chosen::default()
        };
        // Free: the user's choice has no effect.
        assert_eq!(settled(&none, false, on, "pkw"), None);
        // Pro: as chosen, off by default.
        assert_eq!(settled(&none, true, Chosen::default(), "pkw"), None);
        let kept = settled(&none, true, on, "pkw").unwrap();
        assert_eq!(kept.retention_days, PERSONAL_RETENTION_DAYS);
        assert_eq!(kept.max_bytes, PERSONAL_MAX_BYTES);
        assert!(!kept.entries);
        // The organization's word, whatever the plan and the choice.
        let managed = policy(&[(packwing_policy::AUDIT_LOG, PolicyValue::Bool(true))]);
        let kept = settled(&managed, false, Chosen::default(), "packwing").unwrap();
        assert_eq!(kept.retention_days, packwing_policy::MANAGED_RETENTION_DAYS);
        let off = policy(&[(packwing_policy::AUDIT_LOG, PolicyValue::Bool(false))]);
        assert_eq!(settled(&off, true, on, "packwing"), None);
    }

    #[test]
    fn a_folder_of_the_organization_names_its_user() {
        let p = policy(&[
            (packwing_policy::AUDIT_LOG, PolicyValue::Bool(true)),
            (
                packwing_policy::AUDIT_LOG_PATH,
                PolicyValue::Text("/srv/audit/{user}".into()),
            ),
        ]);
        let kept = settled(&p, false, Chosen::default(), "pkw").unwrap();
        let folder = path_text(&kept.folder);
        assert!(folder.starts_with("/srv/audit/"), "{folder}");
        assert!(!folder.contains("{user}"), "{folder}");
    }
}
