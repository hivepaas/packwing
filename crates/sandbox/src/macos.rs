//! The sandbox on macOS: AppKit's open panel, Foundation's security-scoped bookmarks,
//! and the C library's `access` and `getpwuid_r`, which no crate wraps as needed.
#![allow(unsafe_code)]

use std::cell::RefCell;
use std::ffi::{CStr, CString, OsStr};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use block2::RcBlock;
use futures_channel::oneshot;
use objc2::MainThreadMarker;
use objc2::runtime::Bool;
use objc2_app_kit::{NSModalResponse, NSModalResponseOK, NSOpenPanel};
use objc2_foundation::{
    NSData, NSString, NSURL, NSURLBookmarkCreationOptions, NSURLBookmarkResolutionOptions,
};

use crate::{Need, Opened};

/// The home the user database gives, as `HOME` is the container's.
pub fn home() -> Option<PathBuf> {
    let mut entry: libc::passwd = unsafe { std::mem::zeroed() };
    let mut found: *mut libc::passwd = std::ptr::null_mut();
    let mut buf = vec![0 as libc::c_char; 4096];
    // SAFETY: every pointer is to a value or buffer that outlives the call, and the
    // buffer's length is the one given.
    let status = unsafe {
        libc::getpwuid_r(
            libc::getuid(),
            &mut entry,
            buf.as_mut_ptr(),
            buf.len(),
            &mut found,
        )
    };
    if status != 0 || found.is_null() || entry.pw_dir.is_null() {
        return None;
    }
    // SAFETY: getpwuid_r succeeded, so pw_dir points to a C string in `buf`.
    let dir = unsafe { CStr::from_ptr(entry.pw_dir) };
    Some(PathBuf::from(OsStr::from_bytes(dir.to_bytes())))
}

/// What the sandbox answers `access` with: it says no to what it keeps packwing from.
pub fn allows(path: &Path, need: Need) -> bool {
    let Ok(c_path) = CString::new(path.as_os_str().as_bytes()) else {
        return false;
    };
    let mode = match need {
        Need::Read => libc::R_OK,
        Need::Write => libc::R_OK | libc::W_OK,
    };
    // SAFETY: c_path is a C string that outlives the call.
    unsafe { libc::access(c_path.as_ptr(), mode) == 0 }
}

fn url_of(path: &Path) -> Result<objc2::rc::Retained<NSURL>, String> {
    NSURL::from_file_path(path).ok_or_else(|| format!("{} is not a path", path.display()))
}

pub fn bookmark(path: &Path) -> Result<Vec<u8>, String> {
    let url = url_of(path)?;
    let data = url
        .bookmarkDataWithOptions_includingResourceValuesForKeys_relativeToURL_error(
            NSURLBookmarkCreationOptions::WithSecurityScope,
            None,
            None,
        )
        .map_err(|e| e.localizedDescription().to_string())?;
    Ok(data.to_vec())
}

pub fn open_bookmark(data: &[u8]) -> Result<Opened, String> {
    let data = NSData::with_bytes(data);
    let mut stale = Bool::NO;
    // SAFETY: `stale` is a valid pointer for the call.
    let url = unsafe {
        NSURL::URLByResolvingBookmarkData_options_relativeToURL_bookmarkDataIsStale_error(
            &data,
            NSURLBookmarkResolutionOptions::WithSecurityScope
                | NSURLBookmarkResolutionOptions::WithoutUI,
            None,
            &mut stale,
        )
    }
    .map_err(|e| e.localizedDescription().to_string())?;
    // Kept for as long as packwing runs: it is never stopped.
    // SAFETY: a URL resolved from a security-scoped bookmark, as the method asks.
    if !unsafe { url.startAccessingSecurityScopedResource() } {
        return Err("the system did not give it back".to_owned());
    }
    let path = url
        .to_file_path()
        .ok_or_else(|| "the bookmark is not of a file".to_owned())?;
    Ok(Opened {
        path,
        stale: stale.as_bool(),
    })
}

pub fn ask_folder(
    at: &Path,
    message: &str,
    prompt: &str,
    answer: oneshot::Sender<Option<PathBuf>>,
) {
    let Some(mtm) = MainThreadMarker::new() else {
        let _ = answer.send(None);
        return;
    };
    let panel = NSOpenPanel::openPanel(mtm);
    panel.setCanChooseFiles(false);
    panel.setCanChooseDirectories(true);
    panel.setAllowsMultipleSelection(false);
    panel.setCanCreateDirectories(true);
    panel.setMessage(Some(&NSString::from_str(message)));
    panel.setPrompt(Some(&NSString::from_str(prompt)));
    if let Some(url) = NSURL::from_directory_path(at) {
        panel.setDirectoryURL(Some(&url));
    }
    let answer = RefCell::new(Some(answer));
    let shown = panel.clone();
    let done = RcBlock::new(move |response: NSModalResponse| {
        let chosen = if response == NSModalResponseOK {
            shown.URL().and_then(|url| url.to_file_path())
        } else {
            None
        };
        if let Some(answer) = answer.borrow_mut().take() {
            let _ = answer.send(chosen);
        }
    });
    panel.beginWithCompletionHandler(&done);
}
