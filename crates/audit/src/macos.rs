//! macOS: the unified log, by os_log, as the subsystem `com.hivepaas.packwing`
//! and the category `audit`. os_log is a macro, whose format the log keeps
//! apart from the message: src/macos/os_log.c writes it, the record a public
//! string.
//!
//! A record goes at the default level, which the log keeps on disk (info and
//! debug it keeps in memory only), a job that failed as an error.
#![allow(unsafe_code)]

use std::ffi::{CString, c_char, c_void};
use std::io;
use std::sync::OnceLock;

/// The subsystem records are written as.
pub(crate) const SUBSYSTEM: &str = "com.hivepaas.packwing";

/// The category, within it.
const CATEGORY: &str = "audit";

/// The longest message the log keeps whole, in bytes. Past about 1013 it
/// cuts it, between characters, and ends it with `<…>` (macOS 15).
pub(crate) const MOST_BYTES: usize = 1000;

const OS_LOG_TYPE_DEFAULT: u8 = 0x00;
const OS_LOG_TYPE_ERROR: u8 = 0x10;

unsafe extern "C" {
    fn os_log_create(subsystem: *const c_char, category: *const c_char) -> *mut c_void;
    fn packwing_os_log(log: *mut c_void, kind: u8, message: *const c_char);
}

/// A log of the system's: safe to write to from any thread, and never let go.
struct Log(*mut c_void);

// SAFETY: os_log_t is thread-safe, as Apple's documentation says.
unsafe impl Send for Log {}
// SAFETY: as above.
unsafe impl Sync for Log {}

fn create(subsystem: &str) -> io::Result<Log> {
    let subsystem = CString::new(subsystem).map_err(io::Error::other)?;
    let category = CString::new(CATEGORY).map_err(io::Error::other)?;
    // SAFETY: both strings are ended by a 0 and live past the call; what it
    // gives is never null, and is kept.
    Ok(Log(unsafe {
        os_log_create(subsystem.as_ptr(), category.as_ptr())
    }))
}

/// Writes `message`, as an error where `failed`, to `subsystem`.
pub(crate) fn write(subsystem: &str, failed: bool, message: &str) -> io::Result<()> {
    static OURS: OnceLock<Log> = OnceLock::new();
    let made;
    let log = if subsystem == SUBSYSTEM {
        match OURS.get() {
            Some(log) => log,
            None => {
                let log = create(subsystem)?;
                OURS.get_or_init(|| log)
            }
        }
    } else {
        made = create(subsystem)?;
        &made
    };
    let text = CString::new(message.replace('\0', "")).map_err(io::Error::other)?;
    let kind = if failed {
        OS_LOG_TYPE_ERROR
    } else {
        OS_LOG_TYPE_DEFAULT
    };
    // SAFETY: a log os_log_create gave, and a string ended by a 0.
    unsafe { packwing_os_log(log.0, kind, text.as_ptr()) };
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use std::process::Command;
    use std::time::Duration;

    use super::*;

    /// The messages the log has of `subsystem` in the last minutes, as it
    /// shows them, waited for a while as it may take a moment to have them.
    fn shown(subsystem: &str, marker: &str) -> Vec<(String, String)> {
        for _ in 0..30 {
            let out = Command::new("log")
                .args(["show", "--last", "5m", "--style", "ndjson", "--predicate"])
                .arg(format!("subsystem == \"{subsystem}\""))
                .output()
                .unwrap();
            let found: Vec<(String, String)> = String::from_utf8_lossy(&out.stdout)
                .lines()
                .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
                .filter_map(|v| {
                    let message = v["eventMessage"].as_str()?.to_owned();
                    let kind = v["messageType"].as_str().unwrap_or_default().to_owned();
                    message.contains(marker).then_some((kind, message))
                })
                .collect();
            if !found.is_empty() {
                return found;
            }
            std::thread::sleep(Duration::from_millis(500));
        }
        Vec::new()
    }

    #[test]
    fn a_record_is_written_and_read_back() {
        let subsystem = format!("{SUBSYSTEM}.test");
        let marker = format!("packwing test {}", std::process::id());
        let message = format!("{marker}\n{{\"action\":\"test\"}}");
        write(&subsystem, true, &message).unwrap();
        // As long as is kept, in characters of two bytes, which it cuts between.
        let mut long = format!("{marker} long ");
        while long.len() + 2 <= MOST_BYTES {
            long.push('é');
        }
        write(&subsystem, false, &long).unwrap();
        let found = shown(&subsystem, &marker);
        assert!(
            found.contains(&("Error".to_owned(), message.clone())),
            "{found:?}"
        );
        assert!(
            found
                .iter()
                .any(|(kind, m)| kind == "Default" && *m == long),
            "the longest kept whole: {:?}",
            found.iter().map(|(k, m)| (k, m.len())).collect::<Vec<_>>()
        );
    }
}
