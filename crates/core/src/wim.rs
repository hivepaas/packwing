//! WIM archives: the images one holds, and the tree of names in each.
//!
//! A WIM has two levels, as a disk image does. One archive holds several
//! **images** over one set of resources -- `install.wim` from a Windows
//! disc holds an image for each edition on it, every one of them a whole
//! tree of names -- so **an image is a folder**, named by its number and
//! the name the archive's XML gives it (`1-Windows 11 Pro`,
//! `2-Windows 11 Home`). The number is the one every tool counts images
//! by, so `2-Windows 11 Home` is what DISM's `/Index:2` is.
//!
//! An archive of **one** image is that image's tree with no folder in
//! front of it, as a disk with no partition table is one file system over
//! its whole length: a folder there would say the archive holds a level it
//! does not.
//!
//! Read only: packwing writes no WIM and changes none. The names an image
//! says share a file come out as **hard links**, which a Windows image is
//! full of, so an installation comes out once rather than a dozen times
//! over; a **reparse point** -- a symbolic link, a junction, a file kept
//! through Windows' overlay filter -- is refused rather than written
//! pointing at nothing, as NTFS's are. Every file is held to the SHA-1 the
//! image names it by, which is the only check a WIM carries (hivepack's
//! `docs/wim.md`).

use hivepack::wim::Image;

/// What an image is called in a listing: its number, and the name the
/// archive's XML gives it -- `1-Windows 11 Pro`.
///
/// An image with no name of its own is `image3`, as a partition with
/// nothing to say about it is `partition3`.
pub(crate) fn name_of(image: &Image) -> String {
    named(image.index, image.name.as_deref())
}

/// The name of image number `number`, which the XML calls `name`: what
/// [`name_of`] says, from the two things it says it from.
fn named(number: u32, name: Option<&str>) -> String {
    match name.map(one_part).filter(|name| !name.is_empty()) {
        Some(name) => format!("{number}-{name}"),
        None => format!("image{number}"),
    }
}

/// `name` as **one part** of a path, for the folder an image is.
///
/// An image's name is whatever was written into the archive's XML, which
/// is not a file name and was never held to being one: a name holding a
/// separator would put the folder somewhere else, a control character
/// would end it early, and the characters Windows keeps out of a name
/// (`:` among them) would leave nothing to write on. Each becomes `_`, and
/// what Windows strips from the end of a name -- a space, a dot -- goes.
/// The number in front means the part is never `.` or `..`, whatever the
/// name holds.
fn one_part(name: &str) -> String {
    let kept: String = name
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '<' | '>' | '"' | '|' | '?' | '*' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect();
    kept.trim().trim_end_matches(['.', ' ']).trim().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_image_is_named_by_its_number_and_the_xml_name() {
        assert_eq!(named(1, Some("Windows 11 Pro")), "1-Windows 11 Pro");
        assert_eq!(named(2, Some("Windows 11 Home")), "2-Windows 11 Home");
        assert_eq!(named(3, None), "image3");
        assert_eq!(named(4, Some("  ")), "image4");
    }

    #[test]
    fn a_name_that_is_no_file_name_is_one_part_all_the_same() {
        // A separator, a drive's colon and a control character: none of
        // them leaves the folder the image's own.
        assert_eq!(named(1, Some("Windows/11")), "1-Windows_11");
        assert_eq!(named(1, Some("C:\\boot")), "1-C__boot");
        assert_eq!(named(1, Some("one\ntwo")), "1-one_two");
        // Windows strips a trailing dot and space from a name, so a folder
        // named with them is not the one asked for.
        assert_eq!(named(2, Some("Pro. ")), "2-Pro");
        // Nothing of the name is left to write on: the number alone, as
        // for an image with no name at all.
        assert_eq!(named(2, Some("..")), "image2");
    }
}
