//! What the App Sandbox lets packwing reach, in the Mac App Store's build, and asking
//! the user for the rest (packwing-sandbox). A file opened from Finder, chosen or
//! dropped can be read while packwing runs; a folder beside it, where an archive's
//! files are extracted or its other parts are, is asked for in an open panel on it
//! ([`ensure`]) before a job or a reading needs it. What the user gave is kept as
//! security-scoped bookmarks in `access`, beside the settings, and reached again as
//! packwing starts: the folders, and the archives of Recent.
//!
//! Outside the sandbox, nothing is asked or kept, and every function here lets
//! everything through.

use std::path::{Path, PathBuf};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use gpui::{App, AsyncApp, Task};
use packwing_core::{Source, Storage};
use packwing_i18n::tr;
pub use packwing_sandbox::Need;

use crate::dialogs::{Job, Saving};

/// The files kept, the latest first: as many as Recent shows.
const MOST_FILES: usize = 30;
/// The folders kept.
const MOST_FOLDERS: usize = 100;

/// What packwing was given, and keeps.
#[derive(Clone, Debug, Default)]
pub struct Access {
    grants: Vec<Grant>,
    /// Where they are kept; none outside the sandbox.
    file: Option<PathBuf>,
}

impl gpui::Global for Access {}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Grant {
    folder: bool,
    path: PathBuf,
    bookmark: Vec<u8>,
}

/// Whether packwing runs in the sandbox, which asks: or a test says it does.
pub fn asks() -> bool {
    #[cfg(all(test, feature = "test-hooks"))]
    if fake::on() {
        return true;
    }
    packwing_sandbox::active()
}

/// Whether packwing can do `need` with `path` now.
pub fn allows(path: &Path, need: Need) -> bool {
    #[cfg(all(test, feature = "test-hooks"))]
    if fake::on() {
        return fake::allows(path);
    }
    packwing_sandbox::allows(path, need)
}

/// The user's home: in the sandbox, not the container's.
pub fn home() -> Option<PathBuf> {
    if packwing_sandbox::active() {
        return packwing_sandbox::home();
    }
    dirs::home_dir()
}

/// The user's Desktop: in the sandbox, not the container's.
pub fn desktop() -> Option<PathBuf> {
    if packwing_sandbox::active() {
        return Some(packwing_sandbox::home()?.join("Desktop"));
    }
    dirs::desktop_dir()
}

/// The user's Downloads: in the sandbox, not the container's.
pub fn downloads() -> Option<PathBuf> {
    if packwing_sandbox::active() {
        return Some(packwing_sandbox::home()?.join("Downloads"));
    }
    dirs::download_dir()
}

impl Access {
    /// What was kept, reached again; nothing outside the sandbox.
    pub fn load() -> Access {
        if !packwing_sandbox::active() {
            return Access::default();
        }
        let file = crate::prefs::folder().map(|f| f.join("access"));
        let text = file
            .as_ref()
            .and_then(|f| std::fs::read_to_string(f).ok())
            .unwrap_or_default();
        let mut access = Access {
            grants: Vec::new(),
            file,
        };
        let mut changed = false;
        for kept in parse(&text) {
            match packwing_sandbox::open_bookmark(&kept.bookmark) {
                Ok(opened) => {
                    let bookmark = if opened.stale || opened.path != kept.path {
                        changed = true;
                        packwing_sandbox::bookmark(&opened.path).unwrap_or(kept.bookmark)
                    } else {
                        kept.bookmark
                    };
                    access.grants.push(Grant {
                        folder: kept.folder,
                        path: opened.path,
                        bookmark,
                    });
                }
                // Gone, or the user's to give again.
                Err(e) => {
                    eprintln!(
                        "packwing: {} is no longer reached: {e}",
                        kept.path.display()
                    );
                    changed = true;
                }
            }
        }
        if changed {
            access.save();
        }
        access
    }

    /// Keeps `path`, which packwing can reach now, for the next runs: a file first among
    /// the files, a folder among the folders. One a folder kept holds needs nothing.
    fn keep(&mut self, path: &Path) {
        if self.file.is_none()
            || self
                .grants
                .iter()
                .any(|g| g.folder && path.starts_with(&g.path))
        {
            return;
        }
        let folder = path.is_dir();
        let bookmark = match packwing_sandbox::bookmark(path) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("packwing: cannot keep {}: {e}", path.display());
                return;
            }
        };
        self.grants.retain(|g| g.path != path);
        if folder {
            // What it holds needs no more of its own.
            self.grants.retain(|g| !g.path.starts_with(path));
        }
        self.grants.insert(
            0,
            Grant {
                folder,
                path: path.to_path_buf(),
                bookmark,
            },
        );
        let (mut files, mut folders) = (0, 0);
        self.grants.retain(|g| {
            let count = if g.folder { &mut folders } else { &mut files };
            *count += 1;
            *count <= if g.folder { MOST_FOLDERS } else { MOST_FILES }
        });
        self.save();
    }

    fn save(&self) {
        let Some(file) = &self.file else {
            return;
        };
        if let Err(e) = crate::prefs::write_replacing(file, &text(&self.grants)) {
            eprintln!("packwing: cannot keep what it was given: {e}");
        }
    }
}

/// A line a grant: `folder` or `file`, its bookmark in base64, and its path last, as it
/// may hold anything but a line's end.
fn text(grants: &[Grant]) -> String {
    let mut text = String::new();
    for g in grants {
        let path = g.path.to_string_lossy();
        if path.contains('\n') {
            continue;
        }
        let kind = if g.folder { "folder" } else { "file" };
        text.push_str(&format!(
            "{kind}\t{}\t{path}\n",
            STANDARD.encode(&g.bookmark)
        ));
    }
    text
}

fn parse(text: &str) -> Vec<Grant> {
    text.lines()
        .filter_map(|line| {
            let mut parts = line.splitn(3, '\t');
            let folder = match parts.next()? {
                "folder" => true,
                "file" => false,
                _ => return None,
            };
            let bookmark = STANDARD.decode(parts.next()?).ok()?;
            let path = PathBuf::from(parts.next()?);
            Some(Grant {
                folder,
                path,
                bookmark,
            })
        })
        .collect()
}

/// Keeps what the user gave packwing (an archive opened, a folder chosen), for the next
/// runs: in the sandbox only.
pub fn remember(cx: &mut App, paths: &[PathBuf]) {
    if !packwing_sandbox::active() {
        return;
    }
    let access = cx.default_global::<Access>();
    for path in paths {
        access.keep(path);
    }
}

/// What reading the archive at `path` needs that packwing cannot do yet: its folder,
/// when the archive itself is out of reach (one of Recent from an earlier run that was
/// not kept), or holds its other parts.
pub fn to_read(path: &Path) -> Vec<(PathBuf, Need)> {
    if !asks() {
        return Vec::new();
    }
    let Some(folder) = path.parent().filter(|p| !p.as_os_str().is_empty()) else {
        return Vec::new();
    };
    let out_of_reach = !allows(path, Need::Read);
    if (out_of_reach || packwing_sandbox::has_parts(path)) && !allows(folder, Need::Read) {
        return vec![(folder.to_path_buf(), Need::Read)];
    }
    Vec::new()
}

/// The folders a job writes in, or reads beside its archive, that packwing cannot yet.
pub fn of_job(job: &Job) -> Vec<(PathBuf, Need)> {
    if !asks() {
        return Vec::new();
    }
    let mut needs: Vec<(PathBuf, Need)> = Vec::new();
    let write = |path: &Path| (packwing_sandbox::existing(path), Need::Write);
    let beside = |path: &Path| path.parent().map(write);
    let local = |storage: &dyn Storage, folder: &[String]| storage.local_path(folder);
    for (source, to) in &job.work {
        if let Source::File(path) = source {
            needs.extend(to_read(path));
        }
        needs.extend(to.as_deref().map(write));
    }
    if let Some(creating) = &job.create {
        match &creating.to {
            Saving::Here(path) => needs.extend(beside(path)),
            Saving::On(on) => {
                needs.extend(local(on.place.storage.as_ref(), &on.folder).map(|p| write(&p)))
            }
        }
    }
    if let Some(adding) = &job.add {
        needs.extend(beside(&adding.archive));
    }
    if let Some((archive, _, _)) = &job.edit {
        needs.extend(beside(archive));
    }
    if let Some(t) = &job.transfer {
        let t = &t.0;
        needs.extend(local(t.to.as_ref(), &t.to_folder).map(|p| write(&p)));
        if let Some(from) = local(t.from.as_ref(), &t.from_folder) {
            needs.push((from, if t.moving { Need::Write } else { Need::Read }));
        }
    }
    if let Some(signing) = &job.sign
        && let Some((_, folder)) = signing.1.split_last()
    {
        needs.extend(local(signing.0.as_ref(), folder).map(|p| write(&p)));
    }
    if let Some((path, _)) = &job.split {
        needs.extend(beside(path));
    }
    if let Some(part) = &job.join {
        needs.extend(beside(part));
    }
    if let Some(converting) = &job.convert {
        if let Source::File(path) = &converting.from {
            needs.extend(to_read(path));
        }
        needs.extend(beside(&converting.to));
    }
    lacking(needs)
}

/// Of `needs`, those packwing cannot do yet, each once: a folder needed to write in and
/// to read is asked for once, to write.
fn lacking(needs: Vec<(PathBuf, Need)>) -> Vec<(PathBuf, Need)> {
    let mut left: Vec<(PathBuf, Need)> = Vec::new();
    for (path, need) in needs {
        if allows(&path, need) {
            continue;
        }
        match left.iter_mut().find(|(p, _)| *p == path) {
            Some(had) => {
                if need == Need::Write {
                    had.1 = Need::Write;
                }
            }
            None => left.push((path, need)),
        }
    }
    left
}

/// Asks for each of `needs` in turn, in an open panel on the folder; what the user
/// chooses is kept. True once packwing can do them all; false as soon as one is
/// cancelled, or what was chosen does not hold the folder.
pub fn ensure(needs: Vec<(PathBuf, Need)>, cx: &mut App) -> Task<bool> {
    cx.spawn(async move |cx: &mut AsyncApp| {
        for (folder, need) in needs {
            if allows(&folder, need) {
                continue;
            }
            let message = match need {
                Need::Read => tr!(
                    "packwing needs your permission to read this folder. Choose Allow, or a folder that holds it."
                ),
                Need::Write => tr!(
                    "packwing needs your permission to write in this folder. Choose Allow, or a folder that holds it."
                ),
            };
            let answered = cx.update(|_| ask(&folder, &message, &tr!("Allow")));
            let Some(chosen) = answered.await.ok().flatten() else {
                return false;
            };
            cx.update(|cx| remember(cx, std::slice::from_ref(&chosen)));
            if !allows(&folder, need) {
                return false;
            }
        }
        true
    })
}

fn ask(
    at: &Path,
    message: &str,
    prompt: &str,
) -> futures_channel::oneshot::Receiver<Option<PathBuf>> {
    #[cfg(all(test, feature = "test-hooks"))]
    if fake::on() {
        return fake::ask(at);
    }
    packwing_sandbox::ask_folder(at, message, prompt)
}

/// A sandbox for the UI scenarios: the folders it keeps packwing from, and what the user
/// answers when asked.
#[cfg(all(test, feature = "test-hooks"))]
pub mod fake {
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;

    #[derive(Default)]
    struct Fake {
        denied: Vec<PathBuf>,
        granted: Vec<PathBuf>,
        answers: Vec<Option<PathBuf>>,
        asked: Vec<PathBuf>,
    }

    static FAKE: Mutex<Option<Fake>> = Mutex::new(None);

    fn with<R>(f: impl FnOnce(&mut Fake) -> R) -> Option<R> {
        let mut fake = FAKE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        fake.as_mut().map(f)
    }

    /// From now on, what is in `denied` is out of reach until it is asked for, but what
    /// is in `given` (a file opened from Finder, say).
    pub fn start(denied: Vec<PathBuf>, given: Vec<PathBuf>) {
        *FAKE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Fake {
            denied,
            granted: given,
            ..Fake::default()
        });
    }

    pub fn on() -> bool {
        with(|_| ()).is_some()
    }

    /// The next panel is answered with `chosen`, or cancelled with none.
    pub fn answer(chosen: Option<PathBuf>) {
        with(|f| f.answers.push(chosen));
    }

    /// The folders asked for, in turn.
    pub fn asked() -> Vec<PathBuf> {
        with(|f| f.asked.clone()).unwrap_or_default()
    }

    pub fn allows(path: &Path) -> bool {
        with(|f| {
            !f.denied.iter().any(|d| path.starts_with(d))
                || f.granted.iter().any(|g| path.starts_with(g))
        })
        .unwrap_or(true)
    }

    pub fn ask(at: &Path) -> futures_channel::oneshot::Receiver<Option<PathBuf>> {
        let (answer, answered) = futures_channel::oneshot::channel();
        let chosen = with(|f| {
            f.asked.push(at.to_path_buf());
            let chosen = if f.answers.is_empty() {
                None
            } else {
                f.answers.remove(0)
            };
            f.granted.extend(chosen.clone());
            chosen
        })
        .flatten();
        let _ = answer.send(chosen);
        answered
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grants_are_kept_as_lines_and_read_back() {
        let grants = vec![
            Grant {
                folder: true,
                path: PathBuf::from("/Users/a/Projects/with\ttab"),
                bookmark: vec![0, 1, 2, 255],
            },
            Grant {
                folder: false,
                path: PathBuf::from("/Users/a/Downloads/b.zip"),
                bookmark: b"book".to_vec(),
            },
        ];
        let text = text(&grants);
        assert_eq!(parse(&text), grants);
        // A line that is not one is left out, not the rest.
        assert_eq!(parse(&format!("nonsense\n{text}folder\t%%%\t/x\n")), grants);
    }

    #[test]
    fn outside_the_sandbox_nothing_is_needed() {
        if packwing_sandbox::active() {
            return;
        }
        assert!(to_read(Path::new("/nowhere/a.7z.001")).is_empty());
        assert!(lacking(vec![(PathBuf::from("/"), Need::Write)]).is_empty());
    }
}
