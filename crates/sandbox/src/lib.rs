//! The App Sandbox, which the Mac App Store's packwing runs in. There, packwing reads
//! and writes only what the user gave it: a file opened from Finder, chosen in an open
//! panel or dropped on the window, for as long as it runs, and its own container. A
//! folder beside an archive, where its files are extracted or its other parts are, takes
//! the user's permission, asked in an open panel ([`ask_folder`]); a security-scoped
//! bookmark ([`bookmark`]) keeps it for the next runs ([`open_bookmark`]).
//!
//! Outside the sandbox (every other build, and every other system) nothing is asked:
//! [`active`] is false, and everything can be read and written as the user can.

use std::path::{Path, PathBuf};

use futures_channel::oneshot;

#[cfg(target_os = "macos")]
mod macos;

/// packwing runs in the App Sandbox.
pub fn active() -> bool {
    cfg!(target_os = "macos") && std::env::var_os("APP_SANDBOX_CONTAINER_ID").is_some()
}

/// The user's home, where it is on the disk: in the sandbox, `HOME` is the container's.
pub fn home() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    if let Some(home) = macos::home() {
        return Some(home);
    }
    std::env::home_dir()
}

/// What the sandbox lets packwing do with a folder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Need {
    /// List it and read what is in it.
    Read,
    /// Make, change and remove what is in it, and read it.
    Write,
}

/// Whether the sandbox lets packwing do `need` with `path`; true outside it.
pub fn allows(path: &Path, need: Need) -> bool {
    if !active() {
        return true;
    }
    #[cfg(target_os = "macos")]
    {
        macos::allows(path, need)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (path, need);
        true
    }
}

/// A security-scoped bookmark of `path`, which packwing can reach now: what gives it
/// back on a later run.
pub fn bookmark(path: &Path) -> Result<Vec<u8>, String> {
    #[cfg(target_os = "macos")]
    {
        macos::bookmark(path)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
        Err("no sandbox here".to_owned())
    }
}

/// What a bookmark gives back.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Opened {
    /// Where it is now: it may have moved, or been renamed.
    pub path: PathBuf,
    /// The bookmark should be made again, from `path`.
    pub stale: bool,
}

/// Reaches what `data` is a bookmark of, for as long as packwing runs.
pub fn open_bookmark(data: &[u8]) -> Result<Opened, String> {
    #[cfg(target_os = "macos")]
    {
        macos::open_bookmark(data)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = data;
        Err("no sandbox here".to_owned())
    }
}

/// Asks the user, in an open panel on `at`, for a folder: `message` says why, `prompt`
/// is its button. The folder chosen, which packwing can reach while it runs; none if
/// the panel was cancelled, or cannot be shown. Call it on the window's thread.
pub fn ask_folder(at: &Path, message: &str, prompt: &str) -> oneshot::Receiver<Option<PathBuf>> {
    let (answer, answered) = oneshot::channel();
    #[cfg(target_os = "macos")]
    macos::ask_folder(at, message, prompt, answer);
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (at, message, prompt);
        let _ = answer.send(None);
    }
    answered
}

/// The folder a write to `path` needs: `path` itself if it is there, or the nearest
/// folder above it that is, as one that is not there yet is made in it. A folder the
/// sandbox keeps packwing from looking in counts as there.
pub fn existing(path: &Path) -> PathBuf {
    let mut at = path;
    loop {
        match at.try_exists() {
            Ok(false) => match at.parent() {
                Some(parent) if !parent.as_os_str().is_empty() => at = parent,
                _ => return at.to_path_buf(),
            },
            _ => return at.to_path_buf(),
        }
    }
}

/// Whether the archive at `path` has other parts beside it, which its folder holds:
/// `a.7z.001` and `a.7z.002`, `a.part1.rar` and `a.part2.rar`, `a.rar` and `a.r00`,
/// `a.zip` and `a.z01`. The sandbox lets packwing ask whether a file is there without
/// the folder.
pub fn has_parts(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    let lower = name.to_ascii_lowercase();
    let beside = |other: String| path.with_file_name(other).try_exists().unwrap_or(true);
    // `.001`: the second part is `.002`.
    if let Some((stem, n)) = lower.rsplit_once('.')
        && n.len() == 3
        && n.bytes().all(|b| b.is_ascii_digit())
    {
        let next = n.parse::<u32>().unwrap_or(0) + 1;
        return beside(format!("{}.{next:03}", &name[..stem.len()]));
    }
    // `.part1.rar`, `.part01.rar`: any part names the next one.
    if let Some(stem) = lower.strip_suffix(".rar")
        && let Some((base, n)) = stem.rsplit_once(".part")
        && !n.is_empty()
        && n.bytes().all(|b| b.is_ascii_digit())
    {
        let next = n.parse::<u32>().unwrap_or(0) + 1;
        let width = n.len();
        return beside(format!("{}.part{next:0width$}.rar", &name[..base.len()]));
    }
    let stem = &name[..name.len() - lower.rsplit_once('.').map_or(0, |(_, e)| e.len() + 1)];
    if lower.ends_with(".rar") {
        return beside(format!("{stem}.r00"));
    }
    if lower.ends_with(".zip") {
        return beside(format!("{stem}.z01"));
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outside_the_sandbox_everything_is_allowed() {
        if active() {
            return;
        }
        assert!(allows(Path::new("/"), Need::Write));
        assert!(home().is_some());
    }

    #[test]
    fn a_write_needs_the_nearest_folder_there() {
        let dir = tempfile::tempdir().expect("a folder");
        let deep = dir.path().join("a/b/c");
        assert_eq!(existing(&deep), dir.path());
        assert_eq!(existing(dir.path()), dir.path());
        std::fs::create_dir(dir.path().join("a")).expect("a");
        assert_eq!(existing(&deep), dir.path().join("a"));
    }

    #[test]
    fn an_archive_in_parts_is_told_by_its_next_part() {
        let dir = tempfile::tempdir().expect("a folder");
        let file = |name: &str| {
            let path = dir.path().join(name);
            std::fs::write(&path, b"").expect("a file");
            path
        };
        let alone = [
            file("one.zip"),
            file("one.rar"),
            file("one.7z"),
            file("x.part1.rar"),
        ];
        for path in alone {
            assert!(!has_parts(&path), "{}", path.display());
        }
        let first = [
            (file("a.7z.001"), "a.7z.002"),
            (file("b.part1.rar"), "b.part2.rar"),
            (file("c.part01.rar"), "c.part02.rar"),
            (file("d.rar"), "d.r00"),
            (file("E.ZIP"), "E.z01"),
            (file("f.tar.gz.009"), "f.tar.gz.010"),
        ];
        for (path, next) in first {
            file(next);
            assert!(has_parts(&path), "{}", path.display());
        }
    }
}
