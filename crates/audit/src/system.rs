//! Each record, written to the system's own log as well, where the
//! organization's policy says so (`AuditLogSystem`): the Event Log on Windows,
//! the unified log on macOS, journald elsewhere, or syslog where there is none. Collectors read those
//! already, and someone who can change the log's files cannot take a record
//! back out of them.
//!
//! It is a second copy: the day's file stays the record, chain and all. What
//! goes here is a line to read, the record whole, and a few of its fields to
//! filter on.

use std::io;

use serde_json::{Map, Value};

/// How a record reads in the system's log: what was done, how it went, and
/// what it was of.
pub(crate) fn summary(record: &Map<String, Value>) -> String {
    let text = |key: &str| record.get(key).and_then(Value::as_str);
    let action = text("action").unwrap_or("record");
    let mut line = match (text("phase"), text("result")) {
        (Some("start"), _) => format!("{action} started"),
        (Some("end"), Some(result)) => format!("{action} ended: {result}"),
        (Some("end"), None) => format!("{action} ended"),
        _ => action.to_owned(),
    };
    let subject = [
        "source",
        "archive",
        "file",
        "from",
        "destination",
        "folder",
        "part",
        "made",
    ]
    .iter()
    .find_map(|key| text(key));
    if let Some(subject) = subject {
        line += &format!(", {subject}");
    }
    if let Some(entry) = text("entry") {
        line += &format!(" › {entry}");
    }
    if let Some(error) = text("error") {
        line += &format!(" ({error})");
    }
    line
}

/// How much it matters, as syslog counts: a job that failed is a warning, one
/// that did not finish or a log that stops a notice, the rest information.
fn severity(record: &Map<String, Value>) -> u8 {
    let text = |key: &str| record.get(key).and_then(Value::as_str);
    match (text("result"), text("action")) {
        (Some("failed"), _) => 4,
        (Some("partial" | "cancelled"), _) | (_, Some("audit.stopped")) => 5,
        _ => 6,
    }
}

/// An event's id in the Event Log, by how its job went, for filters to take:
/// 1 started, 2 done, 3 partly done, 4 cancelled, 5 failed, 9 a record that is
/// no job's, as the log's own.
#[cfg_attr(not(windows), allow(dead_code))]
fn event_id(record: &Map<String, Value>) -> u32 {
    let text = |key: &str| record.get(key).and_then(Value::as_str);
    match (text("phase"), text("result")) {
        (Some("start"), _) => 1,
        (Some("end"), Some("partial")) => 3,
        (Some("end"), Some("cancelled")) => 4,
        (Some("end"), Some("failed")) => 5,
        (Some("end"), _) => 2,
        _ => 9,
    }
}

/// The system's log records go to, by its name: `journald`, `syslog`, the
/// unified log or the Event Log.
pub fn name() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "the unified log"
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if std::path::Path::new(unix::JOURNAL).exists() {
            "journald"
        } else {
            "syslog"
        }
    }
    #[cfg(not(unix))]
    {
        "the Event Log"
    }
}

/// Writes the record whose `line` it is (its JSON, without the line's end),
/// written by `app`, whose hash is `hash`.
pub(crate) fn send(
    app: &str,
    line: &str,
    hash: &str,
    record: &Map<String, Value>,
) -> io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        let _ = app;
        to_unified_log(line, hash, record)
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        unix::send(app, line, hash, record)
    }
    #[cfg(windows)]
    {
        let _ = app;
        to_event_log(line, hash, record)
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = (app, line, hash, record);
        Ok(())
    }
}

/// The record to the unified log: its line to read, then the record whole on
/// the next line. One longer than the log keeps whole goes as its line and
/// hash, which the day's file holds whole. The app's and pkw's records alike
/// are the subsystem's; which wrote it is in the record, and the process's.
#[cfg(target_os = "macos")]
fn to_unified_log(line: &str, hash: &str, record: &Map<String, Value>) -> io::Result<()> {
    use crate::macos::{MOST_BYTES, SUBSYSTEM, write};

    let read = summary(record);
    let mut message = format!("{read}\n{line}");
    if message.len() > MOST_BYTES {
        message = format!("{read}\nThe record is too long for the log: hash {hash}.");
    }
    write(SUBSYSTEM, severity(record) <= 4, &message)
}

/// The record to the Event Log: its line to read and the record whole, as
/// Event Viewer shows it, then its action, phase, result, job and hash, for
/// collectors. One too long for an event's string goes as its line and hash,
/// which the day's file holds whole.
#[cfg(windows)]
fn to_event_log(line: &str, hash: &str, record: &Map<String, Value>) -> io::Result<()> {
    use windows_sys::Win32::System::EventLog::{EVENTLOG_INFORMATION_TYPE, EVENTLOG_WARNING_TYPE};

    use crate::windows::{MOST_UNITS, SOURCE, report};

    let read = summary(record);
    let mut shown = format!("{read}\r\n\r\n{line}");
    if shown.encode_utf16().count() > MOST_UNITS {
        shown = format!("{read}\r\n\r\nThe record is too long for an event: hash {hash}.");
    }
    let field = |key: &str| match record.get(key) {
        Some(Value::String(s)) => s.clone(),
        Some(other) => other.to_string(),
        None => String::new(),
    };
    let strings = [
        shown,
        field("action"),
        field("phase"),
        field("result"),
        field("job"),
        hash.to_owned(),
    ];
    let kind = if severity(record) <= 4 {
        EVENTLOG_WARNING_TYPE
    } else {
        EVENTLOG_INFORMATION_TYPE
    };
    report(SOURCE, kind, event_id(record), &strings)
}

// Elsewhere than on macOS, which has its own (crate::macos).
#[cfg(all(unix, not(target_os = "macos")))]
mod unix {
    use std::io;
    use std::os::unix::net::UnixDatagram;
    use std::path::Path;

    use serde_json::{Map, Value};

    use super::{severity, summary};

    /// Where journald takes what is sent to it.
    pub(super) const JOURNAL: &str = "/run/systemd/journal/socket";

    /// Where syslog does, by the system: Linux and the BSDs, the BSDs.
    const SYSLOG: [&str; 2] = ["/dev/log", "/var/run/log"];

    /// The facility of what users' programs say.
    const USER: u8 = 1;

    pub(super) fn send(
        app: &str,
        line: &str,
        hash: &str,
        record: &Map<String, Value>,
    ) -> io::Result<()> {
        if Path::new(JOURNAL).exists() {
            return to_journal(Path::new(JOURNAL), &fields(app, line, hash, record));
        }
        match SYSLOG.iter().map(Path::new).find(|p| p.exists()) {
            Some(socket) => to_syslog(socket, app, line, hash, record),
            None => Err(io::Error::new(
                io::ErrorKind::NotFound,
                "this system has neither journald nor syslog",
            )),
        }
    }

    /// What journald is told of a record: its own names, and packwing's.
    pub(super) fn fields(
        app: &str,
        line: &str,
        hash: &str,
        record: &Map<String, Value>,
    ) -> Vec<(&'static str, String)> {
        let mut fields = vec![
            ("MESSAGE", summary(record)),
            ("PRIORITY", severity(record).to_string()),
            ("SYSLOG_IDENTIFIER", app.to_owned()),
            ("SYSLOG_FACILITY", USER.to_string()),
        ];
        for (name, key) in [
            ("PACKWING_ACTION", "action"),
            ("PACKWING_PHASE", "phase"),
            ("PACKWING_RESULT", "result"),
            ("PACKWING_JOB", "job"),
        ] {
            if let Some(value) = record.get(key) {
                let value = match value {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                fields.push((name, value));
            }
        }
        fields.push(("PACKWING_HASH", hash.to_owned()));
        fields.push(("PACKWING_RECORD", line.to_owned()));
        fields
    }

    /// `fields` as journald's native protocol has them: `NAME=value` a line,
    /// or, for a value with a line's end in it, the name, its length and it.
    pub(super) fn encode(fields: &[(&str, String)]) -> Vec<u8> {
        let mut out = Vec::new();
        for (name, value) in fields {
            out.extend_from_slice(name.as_bytes());
            if value.contains('\n') {
                out.push(b'\n');
                out.extend_from_slice(&(value.len() as u64).to_le_bytes());
            } else {
                out.push(b'=');
            }
            out.extend_from_slice(value.as_bytes());
            out.push(b'\n');
        }
        out
    }

    pub(super) fn to_journal(socket: &Path, fields: &[(&str, String)]) -> io::Result<()> {
        let data = encode(fields);
        let out = UnixDatagram::unbound()?;
        match out.send_to(&data, socket) {
            Ok(_) => Ok(()),
            // Too long for one datagram: in a sealed memory file, as journald
            // takes those.
            Err(e) if is_too_long(&e) => in_memory_file(&out, socket, &data),
            Err(e) => Err(e),
        }
    }

    fn is_too_long(e: &io::Error) -> bool {
        e.raw_os_error() == Some(rustix::io::Errno::MSGSIZE.raw_os_error())
            || e.raw_os_error() == Some(rustix::io::Errno::NOBUFS.raw_os_error())
    }

    #[cfg(any(target_os = "linux", target_os = "android"))]
    fn in_memory_file(out: &UnixDatagram, socket: &Path, data: &[u8]) -> io::Result<()> {
        use std::io::Write;
        use std::mem::MaybeUninit;
        use std::os::fd::AsFd;

        use rustix::fs::{MemfdFlags, SealFlags};
        use rustix::net::{SendAncillaryBuffer, SendAncillaryMessage, SendFlags, SocketAddrUnix};

        let fd = rustix::fs::memfd_create(
            "packwing-audit",
            MemfdFlags::CLOEXEC | MemfdFlags::ALLOW_SEALING,
        )?;
        let mut file = std::fs::File::from(fd);
        file.write_all(data)?;
        rustix::fs::fcntl_add_seals(
            &file,
            SealFlags::SHRINK | SealFlags::GROW | SealFlags::WRITE | SealFlags::SEAL,
        )?;
        let mut space = [MaybeUninit::uninit(); rustix::cmsg_space!(ScmRights(1))];
        let mut control = SendAncillaryBuffer::new(&mut space);
        let fds = [file.as_fd()];
        control.push(SendAncillaryMessage::ScmRights(&fds));
        rustix::net::sendmsg_addr(
            out,
            &SocketAddrUnix::new(socket)?,
            &[],
            &mut control,
            SendFlags::empty(),
        )?;
        Ok(())
    }

    #[cfg(not(any(target_os = "linux", target_os = "android")))]
    fn in_memory_file(_: &UnixDatagram, _: &Path, _: &[u8]) -> io::Result<()> {
        Err(io::Error::other("the record is too long for journald"))
    }

    /// The record to syslog: its line to read, then the record whole. One too
    /// long for a datagram goes as its line and hash, which the day's file
    /// holds whole.
    fn to_syslog(
        socket: &Path,
        app: &str,
        line: &str,
        hash: &str,
        record: &Map<String, Value>,
    ) -> io::Result<()> {
        let head = format!(
            "<{}>{app}[{}]: {}",
            USER * 8 + severity(record),
            std::process::id(),
            summary(record).replace('\n', " "),
        );
        let out = UnixDatagram::unbound()?;
        match out.send_to(format!("{head} {line}").as_bytes(), socket) {
            Ok(_) => Ok(()),
            Err(e) if is_too_long(&e) => out
                .send_to(format!("{head} (hash {hash})").as_bytes(), socket)
                .map(drop),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use serde_json::json;

    use super::*;

    fn record(value: Value) -> Map<String, Value> {
        match value {
            Value::Object(map) => map,
            _ => unreachable!(),
        }
    }

    #[test]
    fn a_record_reads_as_what_was_done_and_how_it_went() {
        let start = record(json!({"action": "extract", "phase": "start", "source": "/a/b.zip"}));
        assert_eq!(summary(&start), "extract started, /a/b.zip");
        assert_eq!(severity(&start), 6);
        let failed = record(json!({
            "action": "read_entry", "phase": "end", "result": "failed",
            "source": "/a/b.zip", "entry": "c.txt", "error": "wrong password",
        }));
        assert_eq!(
            summary(&failed),
            "read_entry ended: failed, /a/b.zip › c.txt (wrong password)"
        );
        assert_eq!(severity(&failed), 4);
        let made = record(
            json!({"action": "create", "phase": "end", "result": "ok", "made": "/a/new.zip"}),
        );
        assert_eq!(summary(&made), "create ended: ok, /a/new.zip");
        let stopped = record(json!({"action": "audit.stopped", "folder": "/var/audit"}));
        assert_eq!(summary(&stopped), "audit.stopped, /var/audit");
        assert_eq!(severity(&stopped), 5);
        assert_eq!([&start, &failed, &stopped].map(event_id), [1, 5, 9]);
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    mod unix {
        use std::io::{IoSliceMut, Read, Seek, SeekFrom};
        use std::mem::MaybeUninit;
        use std::os::unix::net::UnixDatagram;
        use std::path::PathBuf;

        use rustix::net::{RecvAncillaryBuffer, RecvAncillaryMessage, RecvFlags};

        use super::super::unix::{encode, fields, to_journal};
        use super::*;

        struct Socket(PathBuf, UnixDatagram);

        impl Socket {
            fn bind(name: &str) -> Socket {
                let path = std::env::temp_dir()
                    .join(format!("packwing-journal-{name}-{}", std::process::id()));
                let _ = std::fs::remove_file(&path);
                let socket = UnixDatagram::bind(&path).unwrap();
                Socket(path, socket)
            }

            /// What was sent, in a datagram or a memory file.
            fn take(&self) -> Vec<u8> {
                let mut data = vec![0u8; 1 << 16];
                let mut space = [MaybeUninit::uninit(); rustix::cmsg_space!(ScmRights(1))];
                let mut control = RecvAncillaryBuffer::new(&mut space);
                let got = rustix::net::recvmsg(
                    &self.1,
                    &mut [IoSliceMut::new(&mut data)],
                    &mut control,
                    RecvFlags::empty(),
                )
                .unwrap();
                data.truncate(got.bytes);
                for message in control.drain() {
                    if let RecvAncillaryMessage::ScmRights(fds) = message {
                        for fd in fds {
                            // From its start, as journald reads it: the
                            // writer's offset is shared.
                            let mut file = std::fs::File::from(fd);
                            file.seek(SeekFrom::Start(0)).unwrap();
                            file.read_to_end(&mut data).unwrap();
                        }
                    }
                }
                data
            }
        }

        impl Drop for Socket {
            fn drop(&mut self) {
                let _ = std::fs::remove_file(&self.0);
            }
        }

        #[test]
        fn journald_is_told_the_record_and_its_fields() {
            let socket = Socket::bind("fields");
            let r = record(json!({
                "action": "create", "phase": "end", "result": "ok",
                "job": 3, "destination": "/a/new\nline.zip",
            }));
            let line = serde_json::to_string(&r).unwrap();
            let told = fields("pkw", &line, "abc", &r);
            to_journal(&socket.0, &told).unwrap();
            let got = socket.take();
            assert_eq!(got, encode(&told));
            let text = String::from_utf8_lossy(&got);
            assert!(text.contains("PRIORITY=6\n"), "{text}");
            assert!(text.contains("SYSLOG_IDENTIFIER=pkw\n"), "{text}");
            assert!(text.contains("PACKWING_ACTION=create\n"), "{text}");
            assert!(text.contains("PACKWING_JOB=3\n"), "{text}");
            assert!(text.contains("PACKWING_HASH=abc\n"), "{text}");
            assert!(
                text.contains(&format!("PACKWING_RECORD={line}\n")),
                "{text}"
            );
            // A line's end in the message: its length, and it whole.
            let message = "create ended: ok, /a/new\nline.zip";
            let mut binary = b"MESSAGE\n".to_vec();
            binary.extend_from_slice(&(message.len() as u64).to_le_bytes());
            binary.extend_from_slice(message.as_bytes());
            assert!(got.windows(binary.len()).any(|w| w == binary.as_slice()));
        }

        #[cfg(target_os = "linux")]
        #[test]
        fn a_record_too_long_for_a_datagram_goes_in_a_memory_file() {
            let socket = Socket::bind("long");
            let r = record(json!({"action": "extract", "names": "x".repeat(4 << 20)}));
            let line = serde_json::to_string(&r).unwrap();
            let told = fields("packwing", &line, "abc", &r);
            to_journal(&socket.0, &told).unwrap();
            assert!(socket.take() == encode(&told), "not what was sent");
        }
    }
}
