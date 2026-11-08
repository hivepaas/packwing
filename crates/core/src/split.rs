//! Cutting a file into parts as 7-Zip cuts it (`name.001`, `name.002`, …, each
//! of the same size but the last), and joining such parts into the file they
//! were. packwing opens an archive's parts as they are; these are for any file,
//! and for what does not read parts.
//!
//! Parts are written beside the file, never over what is there, and a job that
//! fails or is stopped takes away what it wrote.

use crate::audit;
use packwing_i18n::tr;
use serde_json::json;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use crate::create::free_name;
use crate::joined::part_paths;
use crate::{Control, Error, Progress};

/// How much is read at a time.
const PIECE: usize = 1 << 20;

/// The part numbered `n`, from 1, of the file at `base`: `name.001`.
fn part_path(base: &Path, n: u64) -> PathBuf {
    let mut name = base.as_os_str().to_owned();
    name.push(format!(".{n:03}"));
    PathBuf::from(name)
}

/// Cuts the file at `path` into parts of `size` bytes beside it, `name.001` and
/// on, the last of what is left: the parts, in order. One of them there already
/// stops it before anything is written.
pub fn split_file(path: &Path, size: u64, control: &Control) -> Result<Vec<PathBuf>, Error> {
    let job = audit::begin(
        "split",
        audit::fields([
            ("file", json!(audit::path_text(path))),
            ("part_size", json!(size)),
        ]),
    );
    let result = split_into(path, size, control);
    audit::end(job, &result, |parts| {
        audit::fields([("parts", json!(parts.len()))])
    });
    result
}

fn split_into(path: &Path, size: u64, control: &Control) -> Result<Vec<PathBuf>, Error> {
    if size == 0 {
        return Err(Error::Failed(tr!("A part has to hold at least one byte.")));
    }
    let read = |e: io::Error| {
        Error::Failed(tr!(
            "Could not read {path}: {e}",
            path = path.display(),
            e = e
        ))
    };
    let mut from = File::open(path).map_err(read)?;
    let metadata = from.metadata().map_err(read)?;
    let total = metadata.len();
    if !metadata.is_file() {
        return Err(Error::Failed(tr!(
            "{path} is not a file.",
            path = path.display()
        )));
    }
    let count = total.div_ceil(size).max(1);
    if count > 999_999 {
        return Err(Error::Failed(tr!(
            "That would make more parts than can be numbered: make them bigger."
        )));
    }
    if let Some(there) = (1..=count).map(|n| part_path(path, n)).find(|p| p.exists()) {
        return Err(Error::Failed(tr!(
            "{part} is there already.",
            part = there.display()
        )));
    }
    let mut made = Vec::new();
    let written = (|| {
        let mut progress = Progress::default();
        progress.total_bytes = total;
        progress.total_entries = count;
        control.report(&progress);
        let mut buf = vec![0; PIECE];
        for n in 1..=count {
            let part = part_path(path, n);
            let mut out = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&part)
                .map_err(|e| written_error(&part, e))?;
            made.push(part.clone());
            let mut left = size.min(total - progress.bytes);
            while left > 0 {
                control.check()?;
                let wanted = left.min(buf.len() as u64) as usize;
                let got = match from.read(&mut buf[..wanted]) {
                    Ok(0) => {
                        return Err(Error::Failed(tr!(
                            "{path} got shorter while it was being read.",
                            path = path.display()
                        )));
                    }
                    Ok(got) => got,
                    Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(e) => return Err(read(e)),
                };
                out.write_all(&buf[..got])
                    .map_err(|e| written_error(&part, e))?;
                left -= got as u64;
                progress.bytes += got as u64;
                control.report(&progress);
            }
            out.sync_all().map_err(|e| written_error(&part, e))?;
            progress.entries += 1;
            control.report(&progress);
        }
        Ok(())
    })();
    if let Err(e) = written {
        for part in &made {
            let _ = fs::remove_file(part);
        }
        return Err(e);
    }
    Ok(made)
}

fn written_error(path: &Path, e: io::Error) -> Error {
    Error::Failed(tr!(
        "Could not write {path}: {e}",
        path = path.display(),
        e = e
    ))
}

/// The parts that the part at `path` is one of, from `.001`, checked to be all
/// there and whole as far as can be told: none missing between them or after,
/// and each the size of the first but the last, which is no bigger.
pub fn parts_of(path: &Path) -> Result<Vec<PathBuf>, Error> {
    let not_a_part = || {
        Error::Failed(tr!(
            "{path} is not a part of a file cut as 7-Zip cuts it, as name.001.",
            path = path.display()
        ))
    };
    let name = path.file_name().ok_or_else(not_a_part)?.to_string_lossy();
    let (base, number) = name.rsplit_once('.').ok_or_else(not_a_part)?;
    if number.len() < 3 || !number.bytes().all(|b| b.is_ascii_digit()) {
        return Err(not_a_part());
    }
    let first = path.with_file_name(format!("{base}.001"));
    if !first.is_file() {
        return Err(Error::Failed(tr!(
            "{part} is missing: the parts start from it.",
            part = first.display()
        )));
    }
    let parts = part_paths(path);
    // One after a gap means those in it are missing.
    let folder = path.parent().filter(|p| !p.as_os_str().is_empty());
    let last_there = fs::read_dir(folder.unwrap_or(Path::new(".")))
        .map(|entries| {
            entries
                .flatten()
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().into_owned();
                    let number = name.strip_prefix(base)?.strip_prefix('.')?;
                    (number.len() >= 3 && number.bytes().all(|b| b.is_ascii_digit()))
                        .then(|| number.parse::<u64>().ok())
                        .flatten()
                })
                .max()
                .unwrap_or(0)
        })
        .unwrap_or(0);
    let count = parts.len() as u64;
    if last_there > count {
        return Err(Error::Failed(tr!(
            "{part} is missing, and the parts after it are there.",
            part = path
                .with_file_name(format!("{base}.{:03}", count + 1))
                .display()
        )));
    }
    let sizes: Vec<u64> = parts
        .iter()
        .map(|p| fs::metadata(p).map(|m| m.len()))
        .collect::<io::Result<_>>()?;
    let size = sizes[0];
    for (part, &got) in parts.iter().zip(&sizes).take(parts.len() - 1) {
        if got != size {
            return Err(cut_short(part, got, size));
        }
    }
    if let (Some(last), Some(&got)) = (parts.last(), sizes.last())
        && got > size
    {
        return Err(cut_short(last, got, size));
    }
    Ok(parts)
}

fn cut_short(part: &Path, got: u64, size: u64) -> Error {
    Error::Failed(tr!(
        "{part} holds {got}, not {size} as the first part does: it may not be whole.",
        part = part.display(),
        got = crate::size(got),
        size = crate::size(size)
    ))
}

/// Joins the parts that the part at `path` is one of ([`parts_of`]) into the file
/// they were, beside them: named as they are without their number, or the first
/// name after it that is free. Where it went.
pub fn join_parts(path: &Path, control: &Control) -> Result<PathBuf, Error> {
    let job = audit::begin(
        "join",
        audit::fields([("part", json!(audit::path_text(path)))]),
    );
    let result = join_from(path, control);
    audit::end(job, &result, |made| {
        audit::fields([("made", json!(audit::path_text(made)))])
    });
    result
}

fn join_from(path: &Path, control: &Control) -> Result<PathBuf, Error> {
    let parts = parts_of(path)?;
    let first = &parts[0];
    let wanted = first.with_extension("");
    let (to, mut out) = free_name(&wanted, |to| {
        OpenOptions::new().write(true).create_new(true).open(to)
    })?;
    let joined = (|| {
        let mut progress = Progress::default();
        progress.total_entries = parts.len() as u64;
        for part in &parts {
            progress.total_bytes += fs::metadata(part)?.len();
        }
        control.report(&progress);
        let mut buf = vec![0; PIECE];
        for part in &parts {
            let mut from = File::open(part).map_err(|e| {
                Error::Failed(tr!(
                    "Could not read {path}: {e}",
                    path = part.display(),
                    e = e
                ))
            })?;
            loop {
                control.check()?;
                match from.read(&mut buf) {
                    Ok(0) => break,
                    Ok(got) => {
                        out.write_all(&buf[..got])
                            .map_err(|e| written_error(&to, e))?;
                        progress.bytes += got as u64;
                        control.report(&progress);
                    }
                    Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                    Err(e) => {
                        return Err(Error::Failed(tr!(
                            "Could not read {path}: {e}",
                            path = part.display(),
                            e = e
                        )));
                    }
                }
            }
            progress.entries += 1;
            control.report(&progress);
        }
        out.sync_all().map_err(|e| written_error(&to, e))?;
        Ok(())
    })();
    if let Err(e) = joined {
        drop(out);
        let _ = fs::remove_file(&to);
        return Err(e);
    }
    Ok(to)
}
