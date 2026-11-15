//! Self-extracting archives: a program with an archive after it. Windows runs
//! it to unpack itself; everywhere else it is still an archive, and packwing
//! reads it as one.
//!
//! hivepack tells a file by its first bytes, and a program's first bytes are a
//! program's, so one of these arrives as [`Container::Data`] — not an archive.
//! What is looked for here is what comes after the program, as UnRAR looks for
//! a RAR archive after one (`MAXSFXSIZE`): a RAR archive, which
//! `hivepack::rar` then finds by itself, or a ZIP or 7z archive, which is read
//! from where it begins through [`At`].
//!
//! Only a program is read through this way. An unknown file is not, so that
//! opening one costs no more than it does now.

use std::io::{self, Read, Seek, SeekFrom};

use hivepack::open::{Container, Layer};
use hivepack::zip::ReadAt;

use crate::source::Source;

/// How far in an archive after a program is looked for: UnRAR's `MAXSFXSIZE`,
/// which is as far as it looks for a RAR archive. A self-extracting
/// program is far smaller than this — 7-Zip's is under 200 KiB, WinRAR's
/// under 600 KiB — and past it the cost of looking is not worth paying.
const MOST: u64 = 0x40_0000;

/// A ZIP archive's end of central directory record, and the most a comment
/// after it can be.
const EOCD: usize = 22;
const MOST_COMMENT: u64 = 0xFFFF;

/// The first bytes of each archive that can come after a program.
const SEVEN_ZIP: &[u8] = &[b'7', b'z', 0xBC, 0xAF, 0x27, 0x1C];
const RAR: &[u8] = b"Rar!\x1a\x07";

/// What a program carries after it, and where that begins. `at` is 0 for a
/// RAR archive, which `hivepack::rar` finds after the program itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Carried {
    pub(crate) container: Container,
    pub(crate) at: u64,
}

/// What `source` is read as, when hivepack found a program's bytes and no
/// layer over them: the archive it carries after it, if it carries one, and
/// otherwise what hivepack said. `at` is where the archive begins.
pub(crate) fn of(source: &Source, container: Container, layers: &[Layer]) -> io::Result<Carried> {
    let plain = Carried { container, at: 0 };
    if container != Container::Data || !layers.is_empty() {
        return Ok(plain);
    }
    let size = source.size()?;
    Ok(carried(&mut source.reader()?, size)?.unwrap_or(plain))
}

/// Whether `head` is a program: an MZ executable (DOS, and Windows after it)
/// or an ELF one. A type 2 AppImage is ELF with a SquashFS image after it, and
/// hivepack tells one by the mark in its header before it gets here.
pub(crate) fn is_program(head: &[u8]) -> bool {
    head.starts_with(b"MZ") || head.starts_with(b"\x7fELF")
}

/// What the program in `r`, of `size` bytes, carries after it, if anything.
/// Reads `r` where it looks, and leaves it wherever that was.
pub(crate) fn carried<R: Read + Seek>(r: &mut R, size: u64) -> io::Result<Option<Carried>> {
    let mut head = [0u8; 4];
    r.seek(SeekFrom::Start(0))?;
    if fill(r, &mut head)? < head.len() || !is_program(&head) {
        return Ok(None);
    }
    // A ZIP archive is told from its end, which costs one read; the others are
    // told from one pass over the program, which costs more.
    if let Some(at) = zip_at(r, size)? {
        return Ok(Some(Carried {
            container: Container::Zip,
            at,
        }));
    }
    scanned(r, size)
}

/// Where a ZIP archive after a program begins: its end of central directory
/// record says how large its central directory is and where it starts, both
/// counted from the archive's own beginning, so what is before the archive is
/// the difference. Checked against the central directory's own first bytes, so
/// that a program holding those four bytes by chance is not taken for one.
fn zip_at<R: Read + Seek>(r: &mut R, size: u64) -> io::Result<Option<u64>> {
    let want = (MOST_COMMENT + EOCD as u64).min(size);
    if want < EOCD as u64 {
        return Ok(None);
    }
    let from = size - want;
    let mut tail = vec![0u8; want as usize];
    r.seek(SeekFrom::Start(from))?;
    if fill(r, &mut tail)? < tail.len() {
        return Ok(None);
    }
    // The last one first: a ZIP archive holding another is its own, and the
    // outer one ends last.
    for i in (0..=tail.len() - EOCD).rev() {
        if &tail[i..i + 4] != b"PK\x05\x06" {
            continue;
        }
        let (Some(entries), Some(cdir_size), Some(cdir_at)) = (
            le16(&tail, i + 10),
            le32(&tail, i + 12),
            le32(&tail, i + 16),
        ) else {
            continue;
        };
        let at = if entries == u16::MAX || cdir_size == u32::MAX || cdir_at == u32::MAX {
            zip64_at(&tail, from, i)
        } else {
            let whole = u64::from(cdir_size) + u64::from(cdir_at);
            (from + i as u64)
                .checked_sub(whole)
                .map(|at| (at, u64::from(cdir_at), entries > 0))
        };
        let Some((at, cdir_at, has_entries)) = at else {
            continue;
        };
        if at == 0 || !begins(r, at + cdir_at, b"PK\x01\x02", has_entries)? {
            continue;
        }
        return Ok(Some(at));
    }
    Ok(None)
}

/// Where a ZIP64 archive begins: the locator before the end of central
/// directory record says where the ZIP64 record is, counted from the archive's
/// own beginning, and the record itself is found in `tail`, so what is before
/// the archive is the difference. `at` is where the 32-bit record is in
/// `tail`, which the locator comes before.
fn zip64_at(tail: &[u8], from: u64, at: usize) -> Option<(u64, u64, bool)> {
    const LOCATOR: usize = 20;
    let locator = at.checked_sub(LOCATOR)?;
    if tail.get(locator..locator + 4)? != b"PK\x06\x07" {
        return None;
    }
    let told = le64(tail, locator + 8)?;
    // The record it points at, in what was read: its own place says what is
    // before the archive.
    let found = tail[..locator]
        .windows(4)
        .rposition(|w| w == b"PK\x06\x06")?;
    let entries = le64(tail, found + 32)?;
    let cdir_at = le64(tail, found + 48)?;
    let before = (from + found as u64).checked_sub(told)?;
    Some((before, cdir_at, entries > 0))
}

/// Whether what is at `at` in `r` begins with `magic`. `must` is whether it
/// has to be there: an archive of nothing has no central directory to find.
fn begins<R: Read + Seek>(r: &mut R, at: u64, magic: &[u8], must: bool) -> io::Result<bool> {
    if !must {
        return Ok(true);
    }
    let mut found = vec![0u8; magic.len()];
    r.seek(SeekFrom::Start(at))?;
    Ok(fill(r, &mut found)? == found.len() && found == magic)
}

/// A 7z or RAR archive after the program, found by going over the program
/// once. 7z's first bytes are followed by what says how large it is, which is
/// checked against the file's own size; RAR's are followed by its kind.
fn scanned<R: Read + Seek>(r: &mut R, size: u64) -> io::Result<Option<Carried>> {
    /// Enough for either archive's first bytes and what is checked after them,
    /// kept from one chunk to the next so that an archive beginning across the
    /// two is still found.
    const KEEP: usize = 32;
    const CHUNK: usize = 64 * 1024;

    let look = MOST.min(size);
    let mut buffer = vec![0u8; CHUNK + KEEP];
    // Where `buffer` begins in the file, and how much of it was kept from the
    // chunk before.
    let mut start = 0u64;
    let mut kept = 0usize;
    r.seek(SeekFrom::Start(0))?;
    loop {
        // What is left to look at: the reader is at `start` plus what was
        // kept, and nothing past `look` is read.
        let left = look.saturating_sub(start + kept as u64) as usize;
        let want = (buffer.len() - kept).min(left);
        let read = fill(r, &mut buffer[kept..kept + want])?;
        let have = kept + read;
        let chunk = &buffer[..have];
        for i in 0..have {
            let from = &chunk[i..];
            if from.starts_with(SEVEN_ZIP) && seven_zip_fits(from, start + i as u64, size) {
                return Ok(Some(Carried {
                    container: Container::SevenZip,
                    at: start + i as u64,
                }));
            }
            if from.starts_with(RAR) && matches!(from.get(RAR.len()), Some(&(0 | 1))) {
                // Where it begins is hivepack::rar's to find, as UnRAR finds
                // it: the whole file is given to it.
                return Ok(Some(Carried {
                    container: Container::Rar,
                    at: 0,
                }));
            }
        }
        if read < want || have <= KEEP {
            break;
        }
        kept = KEEP;
        buffer.copy_within(have - kept..have, 0);
        start += (have - kept) as u64;
        if start >= look {
            break;
        }
    }
    Ok(None)
}

/// Whether what follows 7z's first bytes at `at` says an archive that fits in
/// a file of `size` bytes: its header is 32 bytes, and after them come the
/// bytes it says its own header takes. Six bytes come up by chance once in
/// many lifetimes of looking, and this is what tells those from an archive.
fn seven_zip_fits(found: &[u8], at: u64, size: u64) -> bool {
    // Its first bytes, the version it was made to, what says where its own
    // header is and how large that is: 32 bytes in all.
    if found.len() < 32 || found.get(6) != Some(&0) {
        return false;
    }
    let (Some(next_at), Some(next_size)) = (le64(found, 12), le64(found, 20)) else {
        return false;
    };
    32u64
        .checked_add(next_at)
        .and_then(|n| n.checked_add(next_size))
        .is_some_and(|whole| at.saturating_add(whole) <= size)
}

/// The little-endian number at `at` in `of`, if that many bytes are there.
fn le16(of: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(of.get(at..at + 2)?.try_into().ok()?))
}

fn le32(of: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(of.get(at..at + 4)?.try_into().ok()?))
}

fn le64(of: &[u8], at: usize) -> Option<u64> {
    Some(u64::from_le_bytes(of.get(at..at + 8)?.try_into().ok()?))
}

/// Reads until `buf` is full or there is no more, and says how much was read:
/// one read answers with what it has, which at the end of a file is less than
/// was asked for.
fn fill<R: Read>(r: &mut R, buf: &mut [u8]) -> io::Result<usize> {
    let mut done = 0;
    while done < buf.len() {
        match r.read(&mut buf[done..]) {
            Ok(0) => break,
            Ok(n) => done += n,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    Ok(done)
}

/// The archive after a program, as what reads it sees: from where it begins to
/// the end of the file, and nothing of the program before it.
pub(crate) struct At<R> {
    inner: R,
    from: u64,
    len: u64,
    at: u64,
}

impl<R> At<R> {
    /// What is from `from` to the end of a source of `size` bytes.
    pub(crate) fn new(inner: R, from: u64, size: u64) -> At<R> {
        At {
            inner,
            from,
            len: size.saturating_sub(from),
            at: 0,
        }
    }
}

impl<R: ReadAt> ReadAt for At<R> {
    fn size(&self) -> io::Result<u64> {
        Ok(self.len)
    }

    fn read_at(&self, offset: u64, buf: &mut [u8]) -> io::Result<usize> {
        let Some(left) = self.len.checked_sub(offset) else {
            return Ok(0);
        };
        let want = (buf.len() as u64).min(left) as usize;
        if want == 0 {
            return Ok(0);
        }
        self.inner.read_at(self.from + offset, &mut buf[..want])
    }
}

impl<R: Read + Seek> Read for At<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let Some(left) = self.len.checked_sub(self.at) else {
            return Ok(0);
        };
        let want = (buf.len() as u64).min(left) as usize;
        if want == 0 {
            return Ok(0);
        }
        // Where the archive is, each time: what reads it seeks as it likes,
        // and one seek costs nothing beside what a read costs.
        self.inner.seek(SeekFrom::Start(self.from + self.at))?;
        let read = self.inner.read(&mut buf[..want])?;
        self.at += read as u64;
        Ok(read)
    }
}

impl<R: Read + Seek> Seek for At<R> {
    fn seek(&mut self, to: SeekFrom) -> io::Result<u64> {
        let at = match to {
            SeekFrom::Start(at) => at as i128,
            SeekFrom::End(by) => self.len as i128 + i128::from(by),
            SeekFrom::Current(by) => self.at as i128 + i128::from(by),
        };
        if at < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "before the archive's own beginning",
            ));
        }
        self.at = at as u64;
        Ok(self.at)
    }
}
