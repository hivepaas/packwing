//! The parts of an archive cut as 7-Zip cuts them (`name.7z.001`,
//! `name.7z.002`, …), read as the one file they were. A file with no number
//! in its name is its own only part.

use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

/// `path`, or if its name ends in a number of three digits or more, the parts
/// numbered from `.001` that are there and follow on. Which of them was opened
/// does not matter; without a `.001`, it is a file like any other.
pub(crate) fn part_paths(path: &Path) -> Vec<PathBuf> {
    let text = path.as_os_str().to_string_lossy();
    let base = text.rsplit_once('.').and_then(|(base, number)| {
        (number.len() >= 3 && number.bytes().all(|b| b.is_ascii_digit())).then_some(base)
    });
    let Some(base) = base else {
        return vec![path.to_path_buf()];
    };
    let parts: Vec<PathBuf> = (1u64..)
        .map(|n| PathBuf::from(format!("{base}.{n:03}")))
        .take_while(|part| part.is_file())
        .collect();
    if parts.is_empty() {
        return vec![path.to_path_buf()];
    }
    parts
}

/// How many bytes the parts of `path` hold together.
pub(crate) fn joined_size(path: &Path) -> io::Result<u64> {
    let mut size = 0;
    for part in part_paths(path) {
        size += fs::metadata(part)?.len();
    }
    Ok(size)
}

/// The parts of a file, read as one. One is open at a time: there may be many.
pub(crate) struct Joined {
    parts: Vec<(PathBuf, u64)>,
    length: u64,
    position: u64,
    /// The part that is open, and where in it the file is.
    open: Option<(usize, File, u64)>,
}

impl Joined {
    pub(crate) fn open(path: &Path) -> io::Result<Joined> {
        let mut parts = Vec::new();
        let mut length = 0;
        for part in part_paths(path) {
            let size = fs::metadata(&part)?.len();
            length += size;
            parts.push((part, size));
        }
        // Now rather than at the first read, as File::open would say it.
        File::open(&parts[0].0)?;
        Ok(Joined {
            parts,
            length,
            position: 0,
            open: None,
        })
    }
}

impl Read for Joined {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut start = 0;
        for (index, (path, size)) in self.parts.iter().enumerate() {
            if self.position >= start + size {
                start += size;
                continue;
            }
            let within = self.position - start;
            let open = match &mut self.open {
                Some(open) if open.0 == index => open,
                slot => slot.insert((index, File::open(path)?, 0)),
            };
            if open.2 != within {
                open.1.seek(SeekFrom::Start(within))?;
            }
            let wanted = (size - within).min(buf.len() as u64) as usize;
            let n = open.1.read(&mut buf[..wanted])?;
            open.2 = within + n as u64;
            self.position += n as u64;
            return Ok(n);
        }
        Ok(0)
    }
}

impl Seek for Joined {
    fn seek(&mut self, to: SeekFrom) -> io::Result<u64> {
        let position = match to {
            SeekFrom::Start(p) => Some(p),
            SeekFrom::Current(d) => self.position.checked_add_signed(d),
            SeekFrom::End(d) => self.length.checked_add_signed(d),
        };
        self.position =
            position.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "seek before 0"))?;
        Ok(self.position)
    }
}
