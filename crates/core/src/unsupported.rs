//! What a file packwing does not open is, when it is a kind of archive that
//! packwing knows of but does not read yet: the window says so, rather than
//! that the file is no archive at all.

use std::io::{self, Read, Seek, SeekFrom};

use crate::Error;
use crate::source::Source;

/// A kind of archive packwing does not read yet, told by its bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unsupported {
    /// An ar archive that is not a Debian package, which packwing opens: a
    /// static library (`.a`), most often.
    Ar,
    /// A macOS disk image (`.dmg`): UDIF, whose last 512 bytes start `koly`.
    Dmg,
}

/// `result`, or, when it failed as reading a file of a kind packwing does
/// not read fails, that kind: no archive, or a layer whose data goes on as
/// none would (a disk image's first block is a bzip2 stream).
pub(crate) fn named<T>(source: &Source, result: Result<T, Error>) -> Result<T, Error> {
    match result {
        Err(Error::NotAnArchive | Error::Failed(_)) => match what_it_is(source) {
            Ok(Some(what)) => Err(Error::Unsupported(what)),
            _ => result,
        },
        result => result,
    }
}

fn what_it_is(source: &Source) -> io::Result<Option<Unsupported>> {
    let mut r = source.reader()?;
    let mut head = [0u8; 24];
    let got = read_up_to(&mut r, &mut head)?;
    if let Some(found) = by_head(&head[..got]) {
        return Ok(Some(found));
    }
    // UDIF's trailer: what is before it may be anything.
    let size = source.size()?;
    if size < 512 {
        return Ok(None);
    }
    r.seek(SeekFrom::Start(size - 512))?;
    let mut tail = [0u8; 4];
    let got = read_up_to(&mut r, &mut tail)?;
    Ok((&tail[..got] == b"koly").then_some(Unsupported::Dmg))
}

/// What the first bytes of a file say it is.
/// A Debian package, which hivepack::open tells too, is one packwing reads:
/// one that fails, fails as itself.
fn by_head(head: &[u8]) -> Option<Unsupported> {
    let member = head.strip_prefix(b"!<arch>\n")?;
    (!member.starts_with(b"debian-binary")).then_some(Unsupported::Ar)
}

fn read_up_to(r: &mut impl Read, buf: &mut [u8]) -> io::Result<usize> {
    let mut got = 0;
    while got < buf.len() {
        match r.read(&mut buf[got..])? {
            0 => break,
            n => got += n,
        }
    }
    Ok(got)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn told_by_their_first_bytes() {
        assert_eq!(
            by_head(b"!<arch>\nfoo.o/          0"),
            Some(Unsupported::Ar)
        );
        // What packwing reads: a Debian package, a SquashFS image, an AppImage.
        assert_eq!(by_head(b"!<arch>\ndebian-binary   1342"), None);
        assert_eq!(by_head(b"hsqs\x05\x00\x00\x00"), None);
        assert_eq!(by_head(b"\x7fELF\x02\x01\x01\x00AI\x02\x00\x00\x00"), None);
        // A program that is only one.
        assert_eq!(by_head(b"\x7fELF\x02\x01\x01\x00\x00\x00\x00\x00"), None);
        assert_eq!(by_head(b"\xFF\xD8\xFF\xE0"), None);
        assert_eq!(by_head(b""), None);
    }
}
