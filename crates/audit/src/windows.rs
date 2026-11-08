//! What Windows is asked by hand: who runs packwing, by their SID, and the
//! Event Log.
//!
//! Records go to the Application log, from the source `packwing`, the app's and
//! pkw's alike. An administrator registers the source once a computer
//! (docs/audit.md): its message file is .NET's `EventLogMessages.dll`, whose
//! every event reads as its first string, so Event Viewer shows what packwing
//! wrote. Unregistered, the event is written all the same, and Event Viewer
//! says its description cannot be found before showing it. packwing installs
//! for the user, with no administrator's rights, so it cannot register the
//! source itself, and a message file in a user's folder would not be found by
//! another's Event Viewer.
#![allow(unsafe_code)]

use std::io;
use std::ptr::{null, null_mut};
use std::sync::OnceLock;

use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, LocalFree};
use windows_sys::Win32::Security::Authorization::ConvertSidToStringSidW;
use windows_sys::Win32::Security::{GetTokenInformation, PSID, TOKEN_QUERY, TOKEN_USER, TokenUser};
use windows_sys::Win32::System::EventLog::{
    DeregisterEventSource, RegisterEventSourceW, ReportEventW,
};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows_sys::core::{PCWSTR, PWSTR};

/// The source records are written from.
pub(crate) const SOURCE: &str = "packwing";

/// The longest string an event takes, in UTF-16 units.
pub(crate) const MOST_UNITS: usize = 31_839;

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}

/// The process's user as its token has it, the SID within: kept, as an
/// event takes the SID's address.
fn token_user() -> Option<&'static [u64]> {
    static USER: OnceLock<Option<Vec<u64>>> = OnceLock::new();
    USER.get_or_init(|| {
        let mut token: HANDLE = null_mut();
        // SAFETY: the process's own token, closed below; the buffer is as
        // long as Windows says, and aligned for TOKEN_USER.
        unsafe {
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
                return None;
            }
            let mut size = 0u32;
            GetTokenInformation(token, TokenUser, null_mut(), 0, &mut size);
            let mut buffer = vec![0u64; (size as usize).div_ceil(8)];
            let ok = GetTokenInformation(
                token,
                TokenUser,
                buffer.as_mut_ptr().cast(),
                size,
                &mut size,
            );
            CloseHandle(token);
            (ok != 0).then_some(buffer)
        }
    })
    .as_deref()
}

fn sid_address() -> PSID {
    match token_user() {
        // SAFETY: Windows wrote a TOKEN_USER at the buffer's start.
        Some(buffer) => unsafe { (*buffer.as_ptr().cast::<TOKEN_USER>()).User.Sid },
        None => null_mut(),
    }
}

/// Who runs packwing, as Windows knows them whatever their name:
/// `S-1-5-21-…`.
pub(crate) fn sid() -> Option<String> {
    let sid = sid_address();
    if sid.is_null() {
        return None;
    }
    let mut text: PWSTR = null_mut();
    // SAFETY: a SID Windows gave; the text it writes is ended by a 0, and
    // freed with LocalFree.
    unsafe {
        if ConvertSidToStringSidW(sid, &mut text) == 0 {
            return None;
        }
        let length = (0..).take_while(|&i| *text.add(i) != 0).count();
        let found = String::from_utf16_lossy(std::slice::from_raw_parts(text, length));
        LocalFree(text.cast());
        Some(found)
    }
}

/// Writes an event of `kind` (information, warning) and `id`, from `source`,
/// holding `strings`, as the user who runs packwing.
pub(crate) fn report(source: &str, kind: u16, id: u32, strings: &[String]) -> io::Result<()> {
    let name = wide(source);
    let texts: Vec<Vec<u16>> = strings.iter().map(|s| wide(s)).collect();
    let pointers: Vec<PCWSTR> = texts.iter().map(|t| t.as_ptr()).collect();
    // SAFETY: each string is ended by a 0 and lives past the call; the
    // handle is let go before returning.
    unsafe {
        let log = RegisterEventSourceW(null(), name.as_ptr());
        if log.is_null() {
            return Err(io::Error::last_os_error());
        }
        let ok = ReportEventW(
            log,
            kind,
            0,
            id,
            sid_address(),
            pointers.len() as u16,
            0,
            pointers.as_ptr(),
            null(),
        );
        let why = io::Error::last_os_error();
        DeregisterEventSource(log);
        if ok == 0 { Err(why) } else { Ok(()) }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use windows_sys::Win32::System::EventLog::{
        CloseEventLog, EVENTLOG_SEQUENTIAL_READ, EVENTLOG_WARNING_TYPE, EVENTLOGRECORD,
        OpenEventLogW, ReadEventLogW,
    };

    use super::*;

    const EVENTLOG_BACKWARDS_READ: u32 = 8;

    /// The text at `at` in `buffer`, to its 0.
    fn text_at(buffer: &[u8], at: usize) -> (String, usize) {
        let units: Vec<u16> = buffer[at..]
            .as_chunks::<2>()
            .0
            .iter()
            .map(|&c| u16::from_le_bytes(c))
            .take_while(|&u| u != 0)
            .collect();
        (String::from_utf16_lossy(&units), at + 2 * (units.len() + 1))
    }

    /// The latest events of the Application log, from `source`: their id,
    /// kind and strings.
    fn latest(source: &str) -> Vec<(u32, u16, Vec<String>)> {
        let mut found = Vec::new();
        let mut buffer = vec![0u8; 1 << 16];
        unsafe {
            let log = OpenEventLogW(null(), wide("Application").as_ptr());
            assert!(!log.is_null(), "{}", io::Error::last_os_error());
            for _ in 0..20 {
                let (mut read, mut needed) = (0u32, 0u32);
                let ok = ReadEventLogW(
                    log,
                    EVENTLOG_SEQUENTIAL_READ | EVENTLOG_BACKWARDS_READ,
                    0,
                    buffer.as_mut_ptr().cast(),
                    buffer.len() as u32,
                    &mut read,
                    &mut needed,
                );
                if ok == 0 {
                    break;
                }
                let mut at = 0usize;
                while at < read as usize {
                    let record = &*buffer.as_ptr().add(at).cast::<EVENTLOGRECORD>();
                    let (name, _) = text_at(&buffer, at + size_of::<EVENTLOGRECORD>());
                    if name == source {
                        let mut strings = Vec::new();
                        let mut next = at + record.StringOffset as usize;
                        for _ in 0..record.NumStrings {
                            let (text, after) = text_at(&buffer, next);
                            strings.push(text);
                            next = after;
                        }
                        found.push((record.EventID & 0xffff, record.EventType, strings));
                    }
                    at += record.Length as usize;
                }
            }
            CloseEventLog(log);
        }
        found
    }

    #[test]
    fn the_user_is_named_by_their_sid() {
        assert!(sid().unwrap().starts_with("S-1-"));
    }

    #[test]
    fn an_event_is_written_and_read_back() {
        let marker = format!("packwing test {}", std::process::id());
        let strings = vec![marker.clone(), "extract".to_owned()];
        report("packwing-test", EVENTLOG_WARNING_TYPE, 5, &strings).unwrap();
        let events = latest("packwing-test");
        assert!(
            events
                .iter()
                .any(|(id, kind, s)| *id == 5 && *kind == EVENTLOG_WARNING_TYPE && *s == strings),
            "{events:?}"
        );
    }
}
