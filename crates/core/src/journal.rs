//! Where packwing is extracting a tar archive into a folder, noted while it
//! does. hivepack reads such an archive once, into a private folder it makes
//! in the destination, and moves the entries into place at the end. A
//! process that ends without removing that folder (a crash, the power) leaves
//! it there: hivepack removes it itself when it extracts into the same place
//! again, and [`clean_left_extractions`] removes it the next time packwing
//! starts, wherever it was.
//!
//! The journal is a file of JSON lines in packwing's settings folder, one for
//! each extraction going on, locked while it is changed: the app and `pkw`
//! may run at once.

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Value, json};

/// The folder the journal is in, once the app or `pkw` has said.
static FOLDER: Mutex<Option<PathBuf>> = Mutex::new(None);

/// What tells this process's lines apart.
static NEXT: AtomicU64 = AtomicU64::new(0);

/// The journal's file, in its folder.
const FILE: &str = "extracting";

/// What hivepack names its private folders, before a number of their own.
const PRIVATE: &str = ".hivepack-extracting-";

/// What packwing names its own, before a number of their own: "Extract
/// selected" writes into one, and moves what was picked out of it.
const OWN: &str = ".packwing-extracting-";

/// The file in a private folder that its extraction keeps locked.
const OWNER: &str = "owner";

/// How old one of packwing's own private folders without an `owner` (made
/// before packwing kept one) is, when it is taken for left behind.
const UNOWNED_FOR: std::time::Duration = std::time::Duration::from_secs(24 * 60 * 60);

/// One of packwing's own private folders, `root`: what is extracted goes into
/// `files`, beside an `owner` file that is locked while it is used, which the
/// system lets go of however the process ends. Removed when dropped.
pub(crate) struct Scratch {
    pub(crate) files: PathBuf,
    root: PathBuf,
    owner: Option<File>,
}

impl Scratch {
    /// The private folder `root`, just made, given its `owner`, locked, and
    /// its `files`.
    pub(crate) fn new(root: PathBuf) -> std::io::Result<Scratch> {
        let made = (|| {
            let owner = File::create_new(root.join(OWNER))?;
            owner
                .try_lock()
                .map_err(|_| std::io::Error::from(std::io::ErrorKind::WouldBlock))?;
            let files = root.join("files");
            std::fs::create_dir(&files)?;
            Ok((owner, files))
        })();
        match made {
            Ok((owner, files)) => Ok(Scratch {
                files,
                root,
                owner: Some(owner),
            }),
            Err(e) => {
                let _ = std::fs::remove_dir_all(&root);
                Err(e)
            }
        }
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        // Closed first: Windows removes no file that is open.
        self.owner = None;
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

/// Removes packwing's own private folders in `parent` that no one uses: whose
/// `owner` can be locked, or that have none and are more than a day old.
pub(crate) fn remove_stale_own(parent: &Path) {
    let Ok(entries) = std::fs::read_dir(parent) else {
        return;
    };
    for entry in entries.flatten() {
        let named = entry
            .file_name()
            .to_str()
            .is_some_and(|name| name.starts_with(OWN));
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if !named || !kind.is_dir() {
            continue;
        }
        let path = entry.path();
        let free = match OpenOptions::new().write(true).open(path.join(OWNER)) {
            Ok(owner) => owner.try_lock().is_ok(),
            Err(_) => entry
                .metadata()
                .and_then(|m| m.modified())
                .ok()
                .and_then(|at| at.elapsed().ok())
                .is_some_and(|age| age > UNOWNED_FOR),
        };
        if free {
            let _ = std::fs::remove_dir_all(&path);
        }
    }
}

/// Keeps the journal in `folder`, packwing's settings folder: before, nothing
/// is noted.
pub fn keep_journal_in(folder: PathBuf) {
    *FOLDER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(folder);
}

/// The folder the journal is kept in, if it is.
fn folder() -> Option<PathBuf> {
    FOLDER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
}

/// The journal in `folder`, open and locked.
fn open(folder: &Path) -> Option<File> {
    std::fs::create_dir_all(folder).ok()?;
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(folder.join(FILE))
        .ok()?;
    file.lock().ok()?;
    Some(file)
}

/// The lines of `file`, read from its start.
fn lines(file: &mut File) -> Vec<Value> {
    let mut text = String::new();
    if file.seek(SeekFrom::Start(0)).is_err() || file.read_to_string(&mut text).is_err() {
        return Vec::new();
    }
    text.lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect()
}

/// `file`, written again with `lines`, and synced.
fn write(file: &mut File, lines: &[Value]) {
    let mut text = String::new();
    for line in lines {
        text.push_str(&line.to_string());
        text.push('\n');
    }
    let _ = file
        .set_len(0)
        .and_then(|()| file.seek(SeekFrom::Start(0)))
        .and_then(|_| file.write_all(text.as_bytes()))
        .and_then(|()| file.sync_all());
}

/// An extraction into a folder, noted: its line goes when this is dropped,
/// however the extraction ended.
pub(crate) struct Noted {
    folder: PathBuf,
    id: String,
}

/// Notes that a tar archive is extracted into `destination`: none if the
/// journal is not kept, or the path is not text.
pub(crate) fn note(destination: &Path) -> Option<Noted> {
    note_in(&folder()?, destination)
}

/// [`note`], in the journal in `folder`.
fn note_in(folder: &Path, destination: &Path) -> Option<Noted> {
    let destination = std::path::absolute(destination).ok()?;
    let path = destination.to_str()?.to_owned();
    let id = format!(
        "{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    );
    let mut file = open(folder)?;
    let mut all = lines(&mut file);
    all.push(json!({ "id": id, "path": path }));
    write(&mut file, &all);
    Some(Noted {
        folder: folder.to_path_buf(),
        id,
    })
}

impl Drop for Noted {
    fn drop(&mut self) {
        let Some(mut file) = open(&self.folder) else {
            return;
        };
        let mut all = lines(&mut file);
        all.retain(|line| line["id"].as_str() != Some(&self.id));
        write(&mut file, &all);
    }
}

/// Removes the private folders that extractions packwing noted left behind,
/// their process having ended without removing them: what the app and `pkw`
/// call when they start. One being used, by a packwing running now, is left,
/// and so is its line.
pub fn clean_left_extractions() {
    if let Some(folder) = folder() {
        clean_in(&folder);
    }
}

/// [`clean_left_extractions`], of the journal in `folder`.
fn clean_in(folder: &Path) {
    let Some(mut file) = open(folder) else {
        return;
    };
    let mut all = lines(&mut file);
    let before = all.len();
    all.retain(|line| {
        let Some(path) = line["path"].as_str().map(Path::new) else {
            return false;
        };
        let _ = hivepack::extract::remove_stale(path);
        remove_stale_own(path);
        // Kept while a private folder is there still: in use.
        std::fs::read_dir(path).is_ok_and(|entries| {
            entries.flatten().any(|entry| {
                entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| name.starts_with(PRIVATE) || name.starts_with(OWN))
            })
        })
    });
    if all.len() != before {
        write(&mut file, &all);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn a_folder_left_is_removed_and_its_line_with_it() {
        let root = std::env::temp_dir().join(format!("packwing-journal-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let settings = root.join("settings");
        let out = root.join("out");
        let noted = note_in(&settings, &out).unwrap();
        // A private folder left by a process that died, no one holding it.
        let left = out.join(format!("{PRIVATE}dead"));
        std::fs::create_dir_all(left.join("data")).unwrap();
        std::fs::write(left.join("owner"), "").unwrap();
        std::mem::forget(noted);
        clean_in(&settings);
        assert!(!left.exists());
        let text = std::fs::read_to_string(settings.join(FILE)).unwrap();
        assert!(text.is_empty(), "{text}");
        // A line goes when its extraction ends.
        let noted = note_in(&settings, &out).unwrap();
        assert!(
            !std::fs::read_to_string(settings.join(FILE))
                .unwrap()
                .is_empty()
        );
        drop(noted);
        assert!(
            std::fs::read_to_string(settings.join(FILE))
                .unwrap()
                .is_empty()
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn packwing_s_own_private_folder_left_is_removed_and_one_in_use_is_not() {
        let root = std::env::temp_dir().join(format!("packwing-scratch-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let used = root.join(format!("{OWN}0"));
        std::fs::create_dir(&used).unwrap();
        let scratch = Scratch::new(used.clone()).unwrap();
        let left = root.join(format!("{OWN}1"));
        std::fs::create_dir_all(left.join("files")).unwrap();
        std::fs::write(left.join(OWNER), "").unwrap();
        remove_stale_own(&root);
        assert!(used.exists(), "in use");
        assert!(!left.exists(), "left behind");
        drop(scratch);
        assert!(!used.exists(), "removed when dropped");
        let _ = std::fs::remove_dir_all(&root);
    }
}
