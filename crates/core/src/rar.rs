//! RAR archives, through hivepack::rar, which reads them and never writes them.
//! It is under UnRAR's license; packwing's own license allows that
//! (`GPL-ADDITIONAL-PERMISSION.md`).

use packwing_i18n::tr;
use std::fs::{self, File};
use std::io;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use hivepack::extract::{Progress, Summary};
use hivepack::rar::{Archive, ExitCode, Extract, HEAD_FILE, ReadAt, name};

use crate::job::{Control, copy_progress};
use crate::listing::{Entry, components};
use crate::{Error, Keys, PASSWORD};

/// The password of `keys`, as RAR takes it: nothing for none.
pub(crate) fn password(keys: &Keys) -> String {
    keys.password
        .as_deref()
        .map(|p| String::from_utf8_lossy(p).into_owned())
        .unwrap_or_default()
}

/// The archive at `path`, the next volumes found as UnRAR names them. `packed`
/// counts the bytes of those it opens.
pub(crate) fn open(
    path: &Path,
    password: &str,
    packed: Arc<AtomicU64>,
) -> Result<Archive<File>, hivepack::rar::Error> {
    let next = move |volume: &str| {
        let file = File::open(volume).ok()?;
        let size = fs::metadata(volume).map_or(0, |m| m.len());
        packed.fetch_add(size, Ordering::Relaxed);
        Some(file)
    };
    let name = path.to_string_lossy();
    Archive::open_volumes(File::open(path)?, &name, next, false, password)
}

/// What the archive holds, and whether anything in it is encrypted. A file
/// going on in the next volume is one entry.
pub(crate) fn entries<R: ReadAt>(
    mut arc: Archive<R>,
    keys: &Keys,
) -> Result<(Vec<Entry>, bool), Error> {
    let mut entries = Vec::new();
    let mut encrypted = false;
    // Through it, as it moves to the next volume where seeking does not.
    let mut extract = Extract::new();
    extract.headers_only = true;
    loop {
        let size = arc.read_header()?;
        if size != 0 && arc.cur_header_type == HEAD_FILE && !arc.file_head.split_before {
            let head = &arc.file_head;
            encrypted |= head.encrypted;
            let path = components(&name::to_string_lossy(&head.file_name));
            if !path.is_empty() {
                entries.push(Entry {
                    path,
                    dir: head.dir,
                    size: if head.dir {
                        0
                    } else {
                        u64::try_from(head.unp_size).unwrap_or(0)
                    },
                    modified: head.mtime.system_time().and_then(|t| {
                        t.duration_since(std::time::UNIX_EPOCH)
                            .ok()
                            .map(|d| d.as_secs() as i64)
                    }),
                });
            }
        }
        if !extract.current_file(&mut arc, size, &mut |_| Ok(()))? {
            break;
        }
        if extract.missing_volume {
            return Err(Error::Failed(tr!("A volume of this archive is not there.")));
        }
    }
    // Headers that do not decrypt read as none, which is no empty archive.
    if entries.is_empty() && arc.exit_code() != ExitCode::Success {
        return Err(failed(arc.exit_code(), keys));
    }
    Ok((entries, encrypted))
}

/// Reads every file through, its hash checked, nothing written.
pub(crate) fn test<R: ReadAt>(
    mut arc: Archive<R>,
    keys: &Keys,
    control: &Control,
    summary: &mut Summary,
) -> Result<(), Error> {
    let mut extract = Extract::new();
    let mut done = Progress::default();
    loop {
        control.check()?;
        let size = arc.read_header()?;
        let is_file = size != 0
            && arc.cur_header_type == HEAD_FILE
            && !arc.file_head.dir
            && !arc.file_head.split_before;
        let encrypted = is_file && arc.file_head.encrypted;
        let named = name::to_string_lossy(&arc.file_head.file_name);
        let go_on = extract
            .current_file(&mut arc, size, &mut |data| {
                control.check().map_err(|_| io::Error::other("cancelled"))?;
                copy_progress(control, &mut done, data.len());
                Ok(())
            })
            .map_err(|e| match control.check() {
                Err(cancelled) => cancelled,
                Ok(()) => Error::from(e),
            })?;
        if extract.missing_password {
            return Err(Error::Locked {
                needs: vec![PASSWORD.to_owned()],
            });
        }
        if extract.bad_password || (extract.broken_file && encrypted && keys.password.is_some()) {
            return Err(Error::WrongPassword);
        }
        if extract.broken_file {
            return Err(Error::Failed(tr!(
                "{named}: its data is damaged.",
                named = named
            )));
        }
        if extract.missing_volume {
            return Err(Error::Failed(tr!("A volume of this archive is not there.")));
        }
        if is_file {
            summary.files += 1;
            done.entries += 1;
            control.report(&done);
        }
        if !go_on {
            break;
        }
    }
    summary.bytes = done.bytes;
    match arc.exit_code() {
        ExitCode::Success => Ok(()),
        code => Err(failed(code, keys)),
    }
}

/// What UnRAR's `code` says: with a password given, a check that fails is that
/// password, which encrypted headers have nothing else to tell by.
fn failed(code: ExitCode, keys: &Keys) -> Error {
    match code {
        ExitCode::BadPassword => Error::WrongPassword,
        ExitCode::Crc if keys.password.is_some() => Error::WrongPassword,
        code => Error::Failed(tr!(
            "The archive is damaged: UnRAR would exit with {number}.",
            number = code.number()
        )),
    }
}
