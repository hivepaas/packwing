//! Copying and moving items, within a storage or from one to another. Nothing
//! appears under its name until all of it is written, and a moved file's original
//! goes only once its copy has been read back and matches.

use std::io::{self, Read};
use std::path::Path;
use std::sync::Arc;

use hivepack_deflate::crc32;
use packwing_i18n::tr;

use crate::create::OnExisting;
use crate::job::Control;
use crate::storage::{Item, Storage};
use crate::{Error, Progress};

/// How many items are asked for at a time.
const PAGE: usize = 1000;

/// What to copy or move, and where to.
#[derive(Clone)]
pub struct Transfer {
    pub from: Arc<dyn Storage>,
    pub from_folder: Vec<String>,
    pub names: Vec<String>,
    pub to: Arc<dyn Storage>,
    pub to_folder: Vec<String>,
    pub moving: bool,
    pub on_existing: OnExisting,
}

/// How a transfer went.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Transferred {
    pub files: u64,
    pub bytes: u64,
    /// Skipped, as a name was taken.
    pub skipped: u64,
    /// Links left where they were: one may lead anywhere.
    pub links_left: u64,
}

/// A thing to do, in the order of doing it: a folder before what it holds.
struct Step {
    /// From the top of what was picked: its name, then those under it.
    path: Vec<String>,
    item: Item,
}

fn failed(e: impl std::fmt::Display) -> Error {
    Error::Failed(e.to_string())
}

fn joined(folder: &[String], rest: &[String]) -> Vec<String> {
    folder.iter().chain(rest).cloned().collect()
}

/// The item at `path`, if there is one.
fn find(storage: &dyn Storage, path: &[String]) -> io::Result<Option<Item>> {
    let Some((name, folder)) = path.split_last() else {
        return Ok(None);
    };
    storage.stat(folder, name)
}

/// All there is to do, walking each folder's pages.
fn plan(t: &Transfer, control: &Control, done: &mut Transferred) -> Result<Vec<Step>, Error> {
    let mut steps = Vec::new();
    for name in &t.names {
        let item = t
            .from
            .stat(&t.from_folder, name)
            .map_err(failed)?
            .ok_or_else(|| failed(tr!("{name} is not there any more.", name = name)))?;
        let mut waiting = vec![Step {
            path: vec![name.clone()],
            item,
        }];
        while let Some(step) = waiting.pop() {
            control.check()?;
            if step.item.link {
                done.links_left += 1;
                continue;
            }
            if step.item.dir {
                let mut lister = t
                    .from
                    .list(&joined(&t.from_folder, &step.path))
                    .map_err(failed)?;
                let mut under = Vec::new();
                while !lister.done() {
                    control.check()?;
                    for item in lister.next(PAGE).map_err(failed)? {
                        let mut path = step.path.clone();
                        path.push(item.name.clone());
                        under.push(Step { path, item });
                    }
                }
                // Popped from the end: in their order.
                waiting.extend(under.into_iter().rev());
            }
            steps.push(step);
        }
    }
    Ok(steps)
}

/// Whether `inner` is `outer`, or under it.
fn is_under(inner: &Path, outer: &Path) -> bool {
    let real = |p: &Path| p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
    real(inner).starts_with(real(outer))
}

/// Whether `from` and `to` are one place on this computer, however it is reached:
/// writing one over the other, then removing the first, would lose the file.
fn same_place(t: &Transfer, from: &[String], to: &[String]) -> bool {
    match (t.from.local_path(from), t.to.local_path(to)) {
        (Some(a), Some(b)) => match (a.canonicalize(), b.canonicalize()) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        },
        _ => false,
    }
}

/// Refuses a folder going into itself, and an item onto itself.
fn check_places(t: &Transfer) -> Result<(), Error> {
    for name in &t.names {
        let (mut from, mut to) = (t.from_folder.clone(), t.to_folder.clone());
        from.push(name.clone());
        to.push(name.clone());
        if same_place(t, &from, &to) {
            return Err(failed(tr!("{name} is already there.", name = name)));
        }
    }
    let Some(to) = t.to.local_path(&t.to_folder) else {
        return Ok(());
    };
    for name in &t.names {
        let mut path = t.from_folder.clone();
        path.push(name.clone());
        if let Some(from) = t.from.local_path(&path)
            && is_under(&to, &from)
        {
            return Err(failed(tr!("{name} cannot go inside itself.", name = name)));
        }
    }
    Ok(())
}

/// The file at `path` of `storage` read through, as a checksum.
fn checksum(storage: &dyn Storage, path: &[String], control: &Control) -> Result<u32, Error> {
    let mut reader = storage.open(path).map_err(failed)?;
    let mut buffer = vec![0; 256 << 10];
    let mut crc = 0;
    loop {
        control.check()?;
        match reader.read(&mut buffer).map_err(failed)? {
            0 => return Ok(crc),
            n => crc = crc32(crc, &buffer[..n]),
        }
    }
}

/// What reads from a file on its way: its checksum, how far it has come, and
/// whether to stop.
pub(crate) struct Watched<'a> {
    pub(crate) inner: Box<dyn Read + Send>,
    pub(crate) crc: u32,
    pub(crate) control: &'a Control,
    pub(crate) progress: &'a mut Progress,
}

impl Read for Watched<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.control.check().is_err() {
            // Not `Interrupted`, which a copy tries again for ever.
            return Err(io::Error::other(tr!("Stopped.")));
        }
        let n = self.inner.read(buf)?;
        self.crc = crc32(self.crc, &buf[..n]);
        self.progress.bytes += n as u64;
        if let Some(say) = &self.control.progress {
            say(self.progress);
        }
        Ok(n)
    }
}

fn say(control: &Control, progress: &Progress) {
    if let Some(say) = &control.progress {
        say(progress);
    }
}

/// Moves or copies `t`'s items. With [`OnExisting::Stop`], a name taken stops it
/// before anything is changed. A folder that is there takes what goes in it.
pub fn transfer(t: &Transfer, control: &Control) -> Result<Transferred, Error> {
    check_places(t)?;
    let mut done = Transferred::default();
    if t.moving && move_whole(t, control, &mut done)? {
        return Ok(done);
    }
    let steps = plan(t, control, &mut done)?;
    let mut progress = Progress::default();
    progress.total_entries = steps.len() as u64;
    progress.total_bytes = steps.iter().map(|s| s.item.size).sum();
    say(control, &progress);
    // What is there already, found before anything changes.
    let mut there: Vec<Option<Item>> = Vec::with_capacity(steps.len());
    for step in &steps {
        control.check()?;
        let at = find(t.to.as_ref(), &joined(&t.to_folder, &step.path)).map_err(failed)?;
        if let Some(found) = &at {
            let shown = step.path.join("/");
            if found.dir != step.item.dir {
                return Err(failed(tr!(
                    "{name} is a file in one place and a folder in the other.",
                    name = shown
                )));
            }
            if !found.dir && t.on_existing == OnExisting::Stop {
                return Err(failed(tr!("{name} is already there.", name = shown)));
            }
        }
        there.push(at);
    }
    // Folders emptied by a move, last: the deepest first.
    let mut emptied = Vec::new();
    for (step, found) in steps.iter().zip(&there) {
        control.check()?;
        let from = joined(&t.from_folder, &step.path);
        let to = joined(&t.to_folder, &step.path);
        if step.item.dir {
            if found.is_none() {
                let (name, folder) = to.split_last().expect("a step has a name");
                t.to.make_folder(folder, name).map_err(failed)?;
            }
            emptied.push(from);
        } else if found.is_some() && t.on_existing == OnExisting::Skip {
            done.skipped += 1;
            progress.bytes += step.item.size;
        } else {
            // Reached through a link in the destination.
            if same_place(t, &from, &to) {
                return Err(failed(tr!(
                    "{name} is already there.",
                    name = step.path.join("/")
                )));
            }
            let mut reader = Watched {
                inner: t.from.open(&from).map_err(failed)?,
                crc: 0,
                control,
                progress: &mut progress,
            };
            let written = t.to.put(&to, &mut reader, &step.item, found.is_some());
            let crc = reader.crc;
            match written {
                Ok(_) => {}
                Err(_) if control.check().is_err() => return Err(Error::Cancelled),
                Err(e) => return Err(failed(e)),
            }
            if t.moving {
                if checksum(t.to.as_ref(), &to, control)? != crc {
                    return Err(failed(tr!(
                        "The copy of {name} does not match. The original was kept.",
                        name = step.path.join("/")
                    )));
                }
                t.from.remove(&from).map_err(failed)?;
            }
            done.files += 1;
            done.bytes += step.item.size;
        }
        progress.entries += 1;
        say(control, &progress);
    }
    if t.moving {
        // One still holding a skipped file or a link stays.
        for folder in emptied.iter().rev() {
            let _ = t.from.remove(folder);
        }
    }
    Ok(done)
}

/// Moves items whole where they can be renamed, as within a disk. False when
/// not all of them can: then nothing was moved, and they go a file at a time.
fn move_whole(t: &Transfer, control: &Control, done: &mut Transferred) -> Result<bool, Error> {
    for name in &t.names {
        control.check()?;
        let mut to = t.to_folder.clone();
        to.push(name.clone());
        let mut from = t.from_folder.clone();
        from.push(name.clone());
        let free = find(t.to.as_ref(), &to).map_err(failed)?.is_none();
        let can = free
            && t.from.local_path(&from).is_some()
            && t.to.local_path(&to).is_some()
            && same_disk(t, &from);
        if !can {
            return Ok(false);
        }
    }
    for name in &t.names {
        let (mut from, mut to) = (t.from_folder.clone(), t.to_folder.clone());
        from.push(name.clone());
        to.push(name.clone());
        let (Some(a), Some(b)) = (t.from.local_path(&from), t.to.local_path(&to)) else {
            return Ok(false);
        };
        std::fs::rename(a, b).map_err(failed)?;
        done.files += 1;
    }
    Ok(true)
}

#[cfg(unix)]
fn same_disk(t: &Transfer, from: &[String]) -> bool {
    use std::os::unix::fs::MetadataExt;
    let device = |p: Option<std::path::PathBuf>| {
        p.and_then(|p| std::fs::symlink_metadata(p).ok())
            .map(|m| m.dev())
    };
    let (a, b) = (
        device(t.from.local_path(from)),
        device(t.to.local_path(&t.to_folder)),
    );
    a.is_some() && a == b
}

/// Elsewhere it is not known: a file at a time always works.
#[cfg(not(unix))]
fn same_disk(_: &Transfer, _: &[String]) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::*;
    use crate::{Cancel, LocalFolder};

    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Scratch {
            let dir = std::env::temp_dir()
                .join(format!("packwing-transfer-{}-{name}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(dir.join("a/d/e")).expect("mkdir");
            fs::create_dir_all(dir.join("b")).expect("mkdir");
            fs::write(dir.join("a/d/one"), "11111").expect("write");
            fs::write(dir.join("a/d/e/two"), "222").expect("write");
            fs::write(dir.join("a/top"), "top").expect("write");
            Scratch(dir)
        }

        fn t(&self, names: &[&str], moving: bool, on_existing: OnExisting) -> Transfer {
            Transfer {
                from: Arc::new(LocalFolder::new(self.0.join("a"))),
                from_folder: Vec::new(),
                names: names.iter().map(|n| (*n).to_owned()).collect(),
                to: Arc::new(LocalFolder::new(self.0.join("b"))),
                to_folder: Vec::new(),
                moving,
                on_existing,
            }
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn none() -> Control {
        Control {
            progress: None,
            cancel: None,
        }
    }

    /// Any part file left behind, under `dir`.
    fn parts(dir: &Path) -> Vec<PathBuf> {
        let mut found = Vec::new();
        for entry in fs::read_dir(dir).expect("read").flatten() {
            let path = entry.path();
            if path.is_dir() {
                found.extend(parts(&path));
            } else if entry
                .file_name()
                .to_string_lossy()
                .contains("packwing-part")
            {
                found.push(path);
            }
        }
        found
    }

    #[test]
    fn a_copy_keeps_the_originals_and_the_details() {
        let s = Scratch::new("copy");
        let old = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_500_000_000);
        fs::File::options()
            .write(true)
            .open(s.0.join("a/top"))
            .expect("open")
            .set_modified(old)
            .expect("set");
        let done = transfer(&s.t(&["d", "top"], false, OnExisting::Stop), &none()).expect("copy");
        assert_eq!(done.files, 3);
        assert_eq!(done.bytes, 11);
        assert_eq!(fs::read(s.0.join("b/d/e/two")).expect("read"), b"222");
        assert_eq!(fs::read(s.0.join("a/d/e/two")).expect("read"), b"222");
        let copied = fs::metadata(s.0.join("b/top")).expect("meta");
        assert_eq!(copied.modified().expect("time"), old);
        assert!(parts(&s.0).is_empty());
    }

    #[test]
    fn a_name_taken_stops_it_before_anything_changes() {
        let s = Scratch::new("stop");
        fs::create_dir_all(s.0.join("b/d")).expect("mkdir");
        fs::write(s.0.join("b/d/one"), "old").expect("write");
        let e = transfer(&s.t(&["top", "d"], false, OnExisting::Stop), &none()).expect_err("taken");
        assert!(e.to_string().contains("d/one"), "{e}");
        assert!(!s.0.join("b/top").exists());
        assert_eq!(fs::read(s.0.join("b/d/one")).expect("read"), b"old");
    }

    #[test]
    fn a_name_taken_is_skipped_or_replaced_as_chosen() {
        let s = Scratch::new("choose");
        fs::create_dir_all(s.0.join("b/d")).expect("mkdir");
        fs::write(s.0.join("b/d/one"), "old").expect("write");
        let done = transfer(&s.t(&["d"], false, OnExisting::Skip), &none()).expect("skip");
        assert_eq!(done.skipped, 1);
        assert_eq!(fs::read(s.0.join("b/d/one")).expect("read"), b"old");
        assert_eq!(fs::read(s.0.join("b/d/e/two")).expect("read"), b"222");
        let done = transfer(&s.t(&["d"], false, OnExisting::Replace), &none()).expect("replace");
        assert_eq!(done.skipped, 0);
        assert_eq!(fs::read(s.0.join("b/d/one")).expect("read"), b"11111");
    }

    #[test]
    fn a_file_does_not_replace_a_folder() {
        let s = Scratch::new("clash");
        fs::create_dir_all(s.0.join("b/top")).expect("mkdir");
        assert!(transfer(&s.t(&["top"], false, OnExisting::Replace), &none()).is_err());
        assert!(s.0.join("b/top").is_dir());
    }

    #[test]
    fn a_move_leaves_nothing_behind() {
        let s = Scratch::new("move");
        transfer(&s.t(&["d", "top"], true, OnExisting::Stop), &none()).expect("move");
        assert!(!s.0.join("a/d").exists());
        assert!(!s.0.join("a/top").exists());
        assert_eq!(fs::read(s.0.join("b/d/one")).expect("read"), b"11111");
    }

    #[test]
    fn a_move_a_file_at_a_time_keeps_what_was_skipped() {
        let s = Scratch::new("move-skip");
        // A folder there already: it cannot go whole.
        fs::create_dir_all(s.0.join("b/d")).expect("mkdir");
        fs::write(s.0.join("b/d/one"), "old").expect("write");
        let done = transfer(&s.t(&["d"], true, OnExisting::Skip), &none()).expect("move");
        assert_eq!((done.files, done.skipped), (1, 1));
        assert_eq!(fs::read(s.0.join("a/d/one")).expect("read"), b"11111");
        assert!(!s.0.join("a/d/e").exists());
        assert_eq!(fs::read(s.0.join("b/d/e/two")).expect("read"), b"222");
    }

    #[test]
    fn a_folder_cannot_go_inside_itself() {
        let s = Scratch::new("inside");
        let mut t = s.t(&["d"], true, OnExisting::Stop);
        t.to = t.from.clone();
        t.to_folder = vec!["d".into(), "e".into()];
        assert!(transfer(&t, &none()).is_err());
        t.to_folder = vec!["d".into()];
        assert!(transfer(&t, &none()).is_err());
        assert!(s.0.join("a/d/one").exists());
    }

    #[test]
    fn copying_onto_itself_changes_nothing() {
        let s = Scratch::new("onto");
        let mut t = s.t(&["top"], false, OnExisting::Stop);
        t.to = t.from.clone();
        assert!(transfer(&t, &none()).is_err());
        t.on_existing = OnExisting::Replace;
        assert!(transfer(&t, &none()).is_err());
        assert_eq!(fs::read(s.0.join("a/top")).expect("read"), b"top");
    }

    #[test]
    fn moving_onto_itself_keeps_the_file() {
        let s = Scratch::new("move-onto");
        for on_existing in [OnExisting::Stop, OnExisting::Replace, OnExisting::Skip] {
            let mut t = s.t(&["top", "d"], true, on_existing);
            t.to = t.from.clone();
            assert!(transfer(&t, &none()).is_err(), "{on_existing:?}");
            assert_eq!(fs::read(s.0.join("a/top")).expect("read"), b"top");
            assert_eq!(fs::read(s.0.join("a/d/one")).expect("read"), b"11111");
        }
    }

    #[cfg(unix)]
    #[test]
    fn moving_through_a_link_back_to_itself_keeps_the_file() {
        let s = Scratch::new("move-link");
        fs::create_dir_all(s.0.join("b/d")).expect("mkdir");
        std::os::unix::fs::symlink(s.0.join("a/d/e"), s.0.join("b/d/e")).expect("link");
        let e = transfer(&s.t(&["d"], true, OnExisting::Replace), &none());
        assert!(e.is_err());
        assert_eq!(fs::read(s.0.join("a/d/e/two")).expect("read"), b"222");
    }

    #[test]
    fn cancelled_it_leaves_no_part_file() {
        let s = Scratch::new("cancel");
        let cancel = Cancel::new();
        let told = cancel.clone();
        let control = Control {
            // Stops as soon as it says it has begun to write.
            progress: Some(Arc::new(move |p: &Progress| {
                if p.bytes > 0 {
                    told.cancel();
                }
            })),
            cancel: Some(cancel),
        };
        let e = transfer(&s.t(&["d"], false, OnExisting::Stop), &control).expect_err("stopped");
        assert!(matches!(e, Error::Cancelled), "{e}");
        assert!(parts(&s.0).is_empty(), "{:?}", parts(&s.0));
    }

    #[cfg(unix)]
    #[test]
    fn links_stay_where_they_are() {
        let s = Scratch::new("links");
        std::os::unix::fs::symlink("/etc/hostname", s.0.join("a/d/out")).expect("link");
        // Whole, by a rename, a link goes along as it is; a copy leaves it.
        let done = transfer(&s.t(&["d"], false, OnExisting::Stop), &none()).expect("copy");
        assert_eq!(done.links_left, 1);
        assert!(fs::symlink_metadata(s.0.join("b/d/out")).is_err());
        assert!(fs::symlink_metadata(s.0.join("a/d/out")).is_ok());
    }
}
