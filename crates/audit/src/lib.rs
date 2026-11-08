//! packwing's audit log: a record of what packwing did, kept on this computer.
//!
//! Records are JSON objects, one a line, in a file a day (`2026-11-08.jsonl`, by
//! the UTC date), so that what is past its time goes a file at a time, and any
//! log collector reads them as they are. The app and pkw write to the same
//! folder: each record is appended under a lock, and none is ever rewritten.
//!
//! Each record holds `prev`, the SHA-256 of the line before it, the first of a
//! day's file the last of the file before. [`verify`] follows the chain: a record
//! changed, taken out or put in shows where. It does not stop someone who can
//! write the files from writing a whole new chain, or from taking the last
//! records away: the log tells what packwing did, it is no lock against the
//! user (docs/audit.md). Records leave the computer only as the organization's
//! own tools take them.
//!
//! Nothing secret is written here: no password, passphrase, key or file's
//! content. What a record holds besides the fields every one has is its
//! writer's to say ([`Log::append`]).

use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

#[cfg(target_os = "macos")]
mod macos;
mod system;
#[cfg(windows)]
mod windows;

pub use system::name as system_log_name;

/// The version of what a record holds, its `v`.
pub const SCHEMA: u64 = 1;

/// The extension of a day's file.
const EXTENSION: &str = "jsonl";

/// The file the writers take turns by.
const LOCK: &str = ".lock";

/// How the log is kept.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    /// Where its files are.
    pub folder: PathBuf,
    /// How many days of records are kept, today's among them: 0 keeps them all.
    pub retention_days: u32,
    /// The most its files may take together, in bytes: the oldest go first,
    /// never today's. 0 for no limit.
    pub max_bytes: u64,
    /// Records name the files in an archive where they are at hand, not only
    /// count them.
    pub entries: bool,
    /// Each record goes to the system's own log as well (`AuditLogSystem`).
    pub system: bool,
    /// A job whose start cannot be written does not start
    /// (`AuditLogRequired`): written to the day's file, and to the system's
    /// log where it goes there too.
    pub required: bool,
    /// What writes: `packwing` or `pkw`.
    pub app: String,
    /// Its version.
    pub version: String,
}

/// The log, written by this process.
#[derive(Debug)]
pub struct Log {
    config: Config,
    /// What every record of this process holds: who, where, what wrote it.
    base: Map<String, Value>,
}

impl Log {
    pub fn new(config: Config) -> Log {
        let mut base = Map::new();
        base.insert("host".into(), json!(host()));
        let (user, id) = user();
        base.insert("user".into(), json!(user));
        if let Some(id) = id {
            base.insert("uid".into(), json!(id));
        }
        #[cfg(windows)]
        if let Some(sid) = windows::sid() {
            base.insert("sid".into(), json!(sid));
        }
        base.insert("app".into(), json!(config.app));
        base.insert("version".into(), json!(config.version));
        base.insert("pid".into(), json!(std::process::id()));
        base.insert("session".into(), json!(session()));
        Log { config, base }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    /// This process's session, as its records name it.
    pub fn session(&self) -> &str {
        self.base["session"].as_str().unwrap_or_default()
    }

    /// Writes `record`, with the fields every record has before its own: `v`,
    /// `time`, `prev`, and who and what wrote it. Its own take their place where
    /// they have the same names, but for `v`, `time` and `prev`. Where the
    /// system's log has it too, it goes there even when its file cannot be
    /// written, with no `prev`.
    pub fn append(&self, record: Map<String, Value>) -> io::Result<()> {
        self.append_at(SystemTime::now(), record)
    }

    fn append_at(&self, now: SystemTime, record: Map<String, Value>) -> io::Result<()> {
        let written = self.write(now, &record);
        if !self.config.system {
            return written.map(drop);
        }
        let line = match &written {
            Ok(line) => line.clone(),
            Err(_) => self.line(now, "", &record)?,
        };
        let sent = system::send(&self.config.app, &line, &hash(line.as_bytes()), &record);
        written?;
        sent.map_err(|e| io::Error::new(e.kind(), format!("{}: {e}", system::name())))
    }

    /// `record` as its line holds it, after `prev`, without the line's end.
    fn line(&self, now: SystemTime, prev: &str, record: &Map<String, Value>) -> io::Result<String> {
        let mut rest = Map::new();
        for (k, v) in self.base.iter().chain(record.iter()) {
            if !matches!(k.as_str(), "v" | "time" | "prev") {
                rest.insert(k.clone(), v.clone());
            }
        }
        // `v`, `time` and `prev` first, for a person reading the file; the rest
        // by name.
        let rest = serde_json::to_string(&Value::Object(rest)).map_err(io::Error::other)?;
        let head = format!(
            "{{\"v\":{SCHEMA},\"time\":{},\"prev\":{}",
            json!(rfc3339(now)),
            json!(prev)
        );
        Ok(match rest.strip_prefix('{').filter(|r| *r != "}") {
            Some(r) => format!("{head},{r}"),
            None => format!("{head}}}"),
        })
    }

    /// Writes `record` to its day's file, after the last: its line.
    fn write(&self, now: SystemTime, record: &Map<String, Value>) -> io::Result<String> {
        fs::create_dir_all(&self.config.folder)?;
        let _turn = Turn::take(&self.config.folder)?;
        let file = self.config.folder.join(file_name(now));
        let prev = match last_line(&file)? {
            Some(line) => hash(&line),
            None => newest_before(&self.config.folder, &file)?
                .map(|f| last_line(&f))
                .transpose()?
                .flatten()
                .map(|line| hash(&line))
                .unwrap_or_default(),
        };
        let line = self.line(now, &prev, record)?;
        let mut out = OpenOptions::new().create(true).append(true).open(&file)?;
        out.write_all(format!("{line}\n").as_bytes())?;
        out.sync_data()?;
        Ok(line)
    }

    /// Takes away what is past its time, or over the size, as the config says,
    /// and records it: the files taken, and the hash of the last record they
    /// held, where the chain of what is left now starts. The files taken.
    pub fn prune(&self) -> io::Result<Vec<String>> {
        self.prune_at(SystemTime::now())
    }

    fn prune_at(&self, now: SystemTime) -> io::Result<Vec<String>> {
        if !self.config.folder.is_dir() {
            return Ok(Vec::new());
        }
        let removed = {
            let _turn = Turn::take(&self.config.folder)?;
            let today = file_name(now);
            let mut files = day_files(&self.config.folder)?;
            let mut gone = Vec::new();
            if self.config.retention_days > 0 {
                let first = file_name(
                    now - Duration::from_secs(
                        86_400 * u64::from(self.config.retention_days.saturating_sub(1)),
                    ),
                );
                while files.first().is_some_and(|(name, _)| *name < first) {
                    gone.push(files.remove(0));
                }
            }
            if self.config.max_bytes > 0 {
                let size = |f: &Path| fs::metadata(f).map_or(0, |m| m.len());
                let mut total: u64 = files.iter().map(|(_, f)| size(f)).sum();
                while total > self.config.max_bytes
                    && files.first().is_some_and(|(name, _)| *name != today)
                {
                    let (name, f) = files.remove(0);
                    total -= size(&f);
                    gone.push((name, f));
                }
            }
            let last = match gone.last() {
                Some((_, f)) => last_line(f)?.map(|l| hash(&l)),
                None => None,
            };
            for (_, f) in &gone {
                fs::remove_file(f)?;
            }
            (gone.into_iter().map(|(n, _)| n).collect::<Vec<_>>(), last)
        };
        let (names, last) = removed;
        if !names.is_empty() {
            let mut record = Map::new();
            record.insert("action".into(), json!("log.pruned"));
            record.insert("removed".into(), json!(names));
            record.insert("last_removed".into(), json!(last.unwrap_or_default()));
            record.insert("retention_days".into(), json!(self.config.retention_days));
            record.insert("max_bytes".into(), json!(self.config.max_bytes));
            self.append_at(now, record)?;
        }
        Ok(names)
    }
}

/// A turn at the log's files: one process at a time writes or takes them away.
struct Turn(File);

impl Turn {
    fn take(folder: &Path) -> io::Result<Turn> {
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(folder.join(LOCK))?;
        file.lock()?;
        Ok(Turn(file))
    }
}

impl Drop for Turn {
    fn drop(&mut self) {
        let _ = self.0.unlock();
    }
}

/// The system's folder for what a program keeps of its state, and packwing's
/// audit folder in it: `~/.local/state/packwing/audit` on Linux (or
/// `$XDG_STATE_HOME`), `~/Library/Logs/packwing/audit` on macOS,
/// `%LOCALAPPDATA%\HivePaaS\packwing\audit` on Windows.
pub fn default_folder() -> Option<PathBuf> {
    let env = |name: &str| {
        std::env::var_os(name)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    };
    let folder = if cfg!(windows) {
        env("LOCALAPPDATA")?.join("HivePaaS").join("packwing")
    } else if cfg!(target_os = "macos") {
        env("HOME")?.join("Library/Logs/packwing")
    } else {
        env("XDG_STATE_HOME")
            .or_else(|| Some(env("HOME")?.join(".local/state")))?
            .join("packwing")
    };
    Some(folder.join("audit"))
}

/// The day files in `folder`, the oldest first, by name.
pub fn day_files(folder: &Path) -> io::Result<Vec<(String, PathBuf)>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(folder)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if is_day_file(&name) && entry.file_type()?.is_file() {
            files.push((name, entry.path()));
        }
    }
    files.sort();
    Ok(files)
}

/// `YYYY-MM-DD.jsonl`.
fn is_day_file(name: &str) -> bool {
    let Some(date) = name.strip_suffix(&format!(".{EXTENSION}")) else {
        return false;
    };
    let b = date.as_bytes();
    b.len() == 10
        && b.iter().enumerate().all(|(i, c)| {
            if i == 4 || i == 7 {
                *c == b'-'
            } else {
                c.is_ascii_digit()
            }
        })
}

/// The newest day file older than `file`, if any.
fn newest_before(folder: &Path, file: &Path) -> io::Result<Option<PathBuf>> {
    let name = file
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    Ok(day_files(folder)?
        .into_iter()
        .rev()
        .find(|(n, _)| *n < name)
        .map(|(_, f)| f))
}

/// The last line of `file`, without its end, if it has one: none where the file
/// is not there or empty.
fn last_line(file: &Path) -> io::Result<Option<Vec<u8>>> {
    let mut f = match File::open(file) {
        Ok(f) => f,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };
    let len = f.metadata()?.len();
    let mut end = len;
    // A last line cut short, its newline missing, is a line all the same.
    let mut tail = Vec::new();
    const STEP: u64 = 64 << 10;
    loop {
        let start = end.saturating_sub(STEP);
        let mut chunk = vec![0; (end - start) as usize];
        f.seek(SeekFrom::Start(start))?;
        f.read_exact(&mut chunk)?;
        chunk.extend_from_slice(&tail);
        tail = chunk;
        let body = tail.strip_suffix(b"\n").unwrap_or(&tail);
        if let Some(at) = body.iter().rposition(|&b| b == b'\n') {
            return Ok(Some(body[at + 1..].to_vec()));
        }
        if start == 0 {
            return Ok((!body.is_empty()).then(|| body.to_vec()));
        }
        end = start;
    }
}

/// A line's SHA-256, in hex.
pub fn hash(line: &[u8]) -> String {
    Sha256::digest(line)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// What [`verify`] found.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Report {
    pub files: usize,
    pub records: usize,
    /// The `prev` of the oldest record kept: empty where the log starts there,
    /// else the last record of what is gone.
    pub anchor: String,
    /// A `log.pruned` record says the anchor is where its files ended.
    pub anchor_explained: bool,
    /// Where the chain does not hold.
    pub breaks: Vec<Break>,
}

/// A place where the chain does not hold: a line changed, taken out, put in, or
/// not a record at all.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Break {
    pub file: String,
    /// From 1.
    pub line: usize,
    pub why: String,
}

/// Follows the chain through every day file in `folder`, the oldest first.
pub fn verify(folder: &Path) -> io::Result<Report> {
    let mut report = Report::default();
    let mut expected: Option<String> = None;
    let mut pruned_at = Vec::new();
    for (name, path) in day_files(folder)? {
        report.files += 1;
        let reader = BufReader::new(File::open(&path)?);
        for (i, line) in reader.split(b'\n').enumerate() {
            let line = line?;
            if line.is_empty() {
                continue;
            }
            report.records += 1;
            let at = |why: String| Break {
                file: name.clone(),
                line: i + 1,
                why,
            };
            let record: Value = match serde_json::from_slice(&line) {
                Ok(Value::Object(record)) => Value::Object(record),
                _ => {
                    report.breaks.push(at("not a record".into()));
                    expected = Some(hash(&line));
                    continue;
                }
            };
            let prev = record["prev"].as_str().unwrap_or_default().to_owned();
            match &expected {
                None => report.anchor = prev,
                Some(want) if *want != prev => report.breaks.push(at(
                    "its prev is not the hash of the record before it: a record \
                     changed, taken out or put in before here"
                        .into(),
                )),
                Some(_) => {}
            }
            if record["action"] == "log.pruned"
                && let Some(last) = record["last_removed"].as_str()
            {
                pruned_at.push(last.to_owned());
            }
            expected = Some(hash(&line));
        }
    }
    report.anchor_explained = report.anchor.is_empty() || pruned_at.contains(&report.anchor);
    Ok(report)
}

/// The records in `folder` from the day `since` (`YYYY-MM-DD`) on, or all, the
/// oldest first; a line that is not a record is left out.
pub fn read(folder: &Path, since: Option<&str>) -> io::Result<Vec<Map<String, Value>>> {
    let mut records = Vec::new();
    for (name, path) in day_files(folder)? {
        if since.is_some_and(|s| name.as_str() < s) {
            continue;
        }
        for line in BufReader::new(File::open(&path)?).split(b'\n') {
            if let Ok(Value::Object(record)) = serde_json::from_slice(&line?) {
                records.push(record);
            }
        }
    }
    Ok(records)
}

/// The day file a record of `now` goes to.
fn file_name(now: SystemTime) -> String {
    let (y, m, d, ..) = civil(now);
    format!("{y:04}-{m:02}-{d:02}.{EXTENSION}")
}

/// `now` as RFC 3339 in UTC, to the millisecond: `2026-11-08T05:01:02.123Z`.
pub fn rfc3339(now: SystemTime) -> String {
    let (y, mo, d, h, mi, s, ms) = civil(now);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}.{ms:03}Z")
}

/// The date and time in UTC of `now`: Howard Hinnant's civil_from_days.
fn civil(now: SystemTime) -> (i64, u32, u32, u32, u32, u32, u32) {
    let since = now.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = since.as_secs() as i64;
    let (days, rest) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = yoe + era * 400 + i64::from(m <= 2);
    (
        y,
        m,
        d,
        (rest / 3600) as u32,
        (rest % 3600 / 60) as u32,
        (rest % 60) as u32,
        since.subsec_millis(),
    )
}

/// This computer's name.
pub fn host() -> String {
    #[cfg(unix)]
    {
        rustix::system::uname()
            .nodename()
            .to_string_lossy()
            .into_owned()
    }
    #[cfg(not(unix))]
    {
        std::env::var("COMPUTERNAME").unwrap_or_default()
    }
}

/// Who runs packwing, as the system names them, and their number on Unix.
pub fn user() -> (String, Option<u32>) {
    let env = |name: &str| std::env::var(name).ok().filter(|v| !v.is_empty());
    #[cfg(unix)]
    {
        let name = env("USER").or_else(|| env("LOGNAME")).unwrap_or_default();
        (name, Some(rustix::process::getuid().as_raw()))
    }
    #[cfg(not(unix))]
    {
        let name = env("USERNAME").unwrap_or_default();
        let name = match env("USERDOMAIN") {
            Some(domain) if !name.is_empty() => format!("{domain}\\{name}"),
            _ => name,
        };
        (name, None)
    }
}

/// A new session's id: 16 hex digits, random.
fn session() -> String {
    let mut bytes = [0u8; 8];
    if getrandom::fill(&mut bytes).is_err() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        bytes = (nanos ^ (u64::from(std::process::id()) << 32)).to_le_bytes();
    }
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    const DAY: u64 = 86_400;

    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Scratch {
            let p =
                std::env::temp_dir().join(format!("packwing-audit-{name}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&p);
            Scratch(p)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn log(folder: &Path) -> Log {
        Log::new(Config {
            folder: folder.to_owned(),
            retention_days: 3,
            max_bytes: 0,
            entries: false,
            system: false,
            required: false,
            app: "pkw".into(),
            version: "0.0.0".into(),
        })
    }

    fn at(days: u64) -> SystemTime {
        // 2026-11-08T00:00:00Z, and days after.
        UNIX_EPOCH + Duration::from_secs(1_794_096_000 + days * DAY + 3600)
    }

    fn record(action: &str) -> Map<String, Value> {
        let mut r = Map::new();
        r.insert("action".into(), json!(action));
        r
    }

    #[test]
    fn times_and_days_are_utc() {
        assert_eq!(rfc3339(at(0)), "2026-11-08T01:00:00.000Z");
        assert_eq!(file_name(at(0)), "2026-11-08.jsonl");
        assert_eq!(file_name(at(54)), "2027-01-01.jsonl");
        assert_eq!(rfc3339(UNIX_EPOCH), "1970-01-01T00:00:00.000Z");
    }

    #[test]
    fn records_chain_across_days_and_verify() {
        let s = Scratch::new("chain");
        let log = log(&s.0);
        log.append_at(at(0), record("extract")).unwrap();
        log.append_at(at(0), record("create")).unwrap();
        log.append_at(at(1), record("test")).unwrap();
        let report = verify(&s.0).unwrap();
        assert_eq!((report.files, report.records), (2, 3));
        assert!(report.breaks.is_empty(), "{report:?}");
        assert_eq!(report.anchor, "");
        assert!(report.anchor_explained);
        let records = read(&s.0, None).unwrap();
        assert_eq!(records[0]["app"], "pkw");
        assert_eq!(records[0]["v"], SCHEMA);
        assert_eq!(records[2]["action"], "test");
        assert_eq!(read(&s.0, Some("2026-11-09")).unwrap().len(), 1);
    }

    #[test]
    fn a_record_changed_or_taken_out_breaks_the_chain() {
        let s = Scratch::new("tamper");
        let log = log(&s.0);
        for action in ["a", "b", "c", "d"] {
            log.append_at(at(0), record(action)).unwrap();
        }
        let file = s.0.join(file_name(at(0)));
        let text = fs::read_to_string(&file).unwrap();
        fs::write(&file, text.replacen("\"b\"", "\"x\"", 1)).unwrap();
        let report = verify(&s.0).unwrap();
        assert_eq!(report.breaks.len(), 1, "{report:?}");
        assert_eq!(report.breaks[0].line, 3);
        let lines: Vec<&str> = text.lines().collect();
        fs::write(&file, format!("{}\n{}\n{}\n", lines[0], lines[1], lines[3])).unwrap();
        let report = verify(&s.0).unwrap();
        assert_eq!(report.breaks.len(), 1, "{report:?}");
        assert_eq!(report.breaks[0].line, 3);
    }

    #[test]
    fn old_days_go_and_the_chain_left_is_anchored_by_the_record_of_it() {
        let s = Scratch::new("prune");
        let log = log(&s.0);
        for day in 0..5 {
            log.append_at(at(day), record("extract")).unwrap();
        }
        // Three days kept: the 10th, 11th and 12th.
        let gone = log.prune_at(at(4)).unwrap();
        assert_eq!(gone, ["2026-11-08.jsonl", "2026-11-09.jsonl"]);
        let report = verify(&s.0).unwrap();
        assert_eq!(report.files, 3);
        assert!(report.breaks.is_empty(), "{report:?}");
        assert!(!report.anchor.is_empty());
        assert!(report.anchor_explained, "{report:?}");
        let last = read(&s.0, None).unwrap().pop().unwrap();
        assert_eq!(last["action"], "log.pruned");
        assert_eq!(last["removed"], json!(gone));
        // Nothing more to take: nothing recorded.
        assert!(log.prune_at(at(4)).unwrap().is_empty());
    }

    #[test]
    fn over_the_size_the_oldest_go_never_today() {
        let s = Scratch::new("size");
        let mut log = log(&s.0);
        log.config.retention_days = 0;
        for day in 0..3 {
            log.append_at(at(day), record("extract")).unwrap();
        }
        log.config.max_bytes = 1;
        let gone = log.prune_at(at(2)).unwrap();
        assert_eq!(gone, ["2026-11-08.jsonl", "2026-11-09.jsonl"]);
        assert_eq!(day_files(&s.0).unwrap().len(), 1);
    }

    #[test]
    fn writers_at_once_keep_one_chain() {
        let s = Scratch::new("writers");
        std::thread::scope(|scope| {
            for _ in 0..4 {
                scope.spawn(|| {
                    let log = log(&s.0);
                    for _ in 0..25 {
                        log.append(record("extract")).unwrap();
                    }
                });
            }
        });
        let report = verify(&s.0).unwrap();
        assert_eq!(report.records, 100);
        assert!(report.breaks.is_empty(), "{report:?}");
    }

    #[test]
    fn a_last_line_is_found_however_long() {
        let s = Scratch::new("long");
        fs::create_dir_all(&s.0).unwrap();
        let f = s.0.join("x");
        let long = "y".repeat(200_000);
        fs::write(&f, format!("first\n{long}\n")).unwrap();
        assert_eq!(last_line(&f).unwrap().unwrap().len(), 200_000);
        fs::write(&f, "only").unwrap();
        assert_eq!(last_line(&f).unwrap().unwrap(), b"only");
        fs::write(&f, "").unwrap();
        assert_eq!(last_line(&f).unwrap(), None);
    }
}
