//! Where an extraction writes when there is no folder to write into.
//!
//! A phone's document tree is reached by handles, not by paths: nothing can
//! be written into it by name, so an app had to extract into a folder of its
//! own and copy what it wrote across, writing every byte twice. A
//! [`Destination`] is given the entries themselves, as hivepack's `Sink` is
//! when an archive is written into another.

use std::io::{self, Read};
use std::sync::Arc;

use hivepack::extract::{Entry, Select, Sink};
use serde_json::json;

use crate::audit;
use crate::job::{Taker, WriteTo, extract_into, extracted, extracting_into};
use crate::source::Source;
use crate::{Control, Error, ExtractOptions, Keys, Summary};

/// How much of a file is read at once where its data comes as a stream.
const PIECE: usize = 64 << 10;

/// What takes an archive's entries in place of a folder.
///
/// They come in the order a folder's extraction writes them, each once: a
/// file as [`start_file`](Destination::start_file), its data in pieces of
/// [`write`](Destination::write), then
/// [`finish_file`](Destination::finish_file) — or
/// [`discard_file`](Destination::discard_file) when its data failed, after
/// which the extraction stops. Symbolic links come after the rest, and a hard
/// link after the file it names. A folder on the way to an entry comes only
/// if the archive holds an entry for it, so a destination that writes by name
/// makes the ones between itself.
///
/// What hivepack checks of a folder — that nothing is written outside it, nor
/// over what is already there — means nothing here: the paths are checked
/// (`..` and the like are gone before they arrive), but what to do about a
/// name already taken is the destination's own, and
/// [`ExtractOptions::overwrite`] is for it to honour.
pub trait Destination {
    /// A folder.
    fn directory(&mut self, path: &[String]) -> io::Result<()>;

    /// A file, whose data follows; `size` is what the archive says it is,
    /// where it says before the data.
    fn start_file(&mut self, path: &[String], size: Option<u64>) -> io::Result<()>;

    /// A piece of the file's data.
    fn write(&mut self, data: &[u8]) -> io::Result<()>;

    /// The end of the file, all of its data given.
    fn finish_file(&mut self) -> io::Result<()>;

    /// The end of a file whose data failed: what was given of it is not to be
    /// kept.
    fn discard_file(&mut self);

    /// A symbolic link to `target`, relative to the link's folder. By
    /// default a file holding where it points, which is what packwing writes
    /// on Windows, where most accounts cannot make one.
    fn symlink(&mut self, path: &[String], target: &str) -> io::Result<()> {
        self.start_file(path, Some(target.len() as u64))?;
        self.write(target.as_bytes())?;
        self.finish_file()
    }

    /// Another name for the file at `target`, which came before. Nothing by
    /// default: a destination that cannot make one copies that file, or
    /// leaves this out.
    fn hard_link(&mut self, path: &[String], target: &[String]) -> io::Result<()> {
        let _ = (path, target);
        Ok(())
    }
}

/// A [`Destination`] as what an extraction writes into.
struct Taking<'a>(&'a mut dyn Destination);

impl Sink for Taking<'_> {
    fn directory(&mut self, entry: &Entry<'_>) -> io::Result<()> {
        self.0.directory(entry.path)
    }

    fn start_file(&mut self, entry: &Entry<'_>) -> io::Result<()> {
        self.0.start_file(entry.path, entry.size)
    }

    fn write(&mut self, data: &[u8]) -> io::Result<()> {
        self.0.write(data)
    }

    fn finish_file(&mut self) -> io::Result<()> {
        self.0.finish_file()
    }

    fn discard_file(&mut self) {
        self.0.discard_file();
    }

    fn symlink(&mut self, entry: &Entry<'_>, target: &str) -> io::Result<()> {
        self.0.symlink(entry.path, target)
    }

    fn hard_link(&mut self, entry: &Entry<'_>, target: &[String]) -> io::Result<()> {
        self.0.hard_link(entry.path, target)
    }
}

impl Taker for Taking<'_> {
    /// The one file that is under layers alone, as `a.sql.gz` is: it has no
    /// entry of its own, only the name the layers leave.
    fn one_file(
        &mut self,
        name: &str,
        data: &mut dyn Read,
        control: &Control,
    ) -> Result<u64, Error> {
        let path = [name.to_owned()];
        self.0.start_file(&path, None)?;
        let mut buffer = vec![0u8; PIECE];
        let mut written = 0u64;
        loop {
            if let Err(e) = control.check() {
                self.0.discard_file();
                return Err(e);
            }
            match data.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => match self.0.write(&buffer[..n]) {
                    Ok(()) => written += n as u64,
                    Err(e) => {
                        self.0.discard_file();
                        return Err(e.into());
                    }
                },
                Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => {
                    self.0.discard_file();
                    return Err(e.into());
                }
            }
        }
        self.0.finish_file()?;
        Ok(written)
    }
}

/// Extracts the archive of `source` into `destination`: all of it, or only
/// `picked` (paths in it, each with what is under it). What comes out keeps
/// the path it has in the archive, unlike
/// [`extract_selected`](crate::extract_selected), which takes each picked
/// entry out of its folders.
pub fn extract_to(
    source: impl Into<Source>,
    destination: &mut dyn Destination,
    picked: &[Vec<String>],
    keys: &Keys,
    how: &ExtractOptions,
    control: &Control,
) -> Result<Summary, Error> {
    let source = source.into();
    let mut fields = extracting_into(&source, json!("what takes its entries"), how);
    if !picked.is_empty() {
        fields.insert("picked".into(), json!(picked.len()));
        fields.extend(audit::fields([(
            "picked_names",
            audit::list(picked.iter().map(|p| p.join("/"))),
        )]));
    }
    let job = audit::begin("extract", fields)?;
    let chosen: Vec<Vec<String>> = picked.to_vec();
    let select: Option<Select> = if chosen.is_empty() {
        None
    } else {
        Some(Arc::new(move |entry: &[String]| {
            chosen.iter().any(|p| entry.starts_with(p))
        }))
    };
    let mut taking = Taking(destination);
    let result = extract_into(
        &source,
        WriteTo::Taker(&mut taking),
        keys,
        how,
        select,
        control,
    );
    audit::end(job, &result, extracted);
    result
}
