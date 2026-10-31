//! What the screens show of an archive, and how sizes, counts and names
//! read. An archive read from disk fills [`Archive`] from what
//! packwing-core lists; the samples of `demo.rs` fill the same.

use std::path::PathBuf;
use std::sync::Arc;

use packwing_core::{Child, Keys, Listing};

use crate::colors::{Palette, Tint};

/// What a file in an archive is, as far as its tag and color go.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Folder,
    Pdf,
    Image,
    Document,
    Sheet,
    Text,
    Program,
    Archive,
}

impl Kind {
    pub fn tint(self, c: &Palette) -> Tint {
        match self {
            Kind::Folder => c.folder_tag,
            Kind::Pdf => c.red_tag,
            Kind::Image | Kind::Document => c.blue_tag,
            Kind::Sheet | Kind::Archive => c.green_tag,
            Kind::Text => c.plain_tag,
            Kind::Program => c.orange_tag,
        }
    }

    /// What a file is, from its extension.
    pub fn of(name: &str, dir: bool) -> Kind {
        if dir {
            return Kind::Folder;
        }
        let ext = name
            .rsplit_once('.')
            .map(|(_, e)| e.to_ascii_lowercase())
            .unwrap_or_default();
        match ext.as_str() {
            "pdf" => Kind::Pdf,
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "heic" | "bmp" | "tif" | "tiff" | "svg" => {
                Kind::Image
            }
            "doc" | "docx" | "odt" | "rtf" | "pages" | "ppt" | "pptx" | "odp" | "key" => {
                Kind::Document
            }
            "xls" | "xlsx" | "ods" | "csv" | "numbers" => Kind::Sheet,
            "exe" | "msi" | "dll" | "app" | "dmg" | "pkg" | "deb" | "rpm" | "sh" | "bat"
            | "cmd" | "ps1" | "js" | "lnk" | "jar" | "apk" => Kind::Program,
            "zip" | "tar" | "gz" | "tgz" | "zst" | "xz" | "bz2" | "lz" | "lz4" | "lzma" | "7z"
            | "rar" | "age" | "iso" | "cpio" | "xar" => Kind::Archive,
            _ => Kind::Text,
        }
    }
}

/// A file or folder in an archive.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub kind: Kind,
    /// Unpacked, in bytes: all a folder holds.
    pub size: u64,
    /// The day it was last changed, `2026-10-15`, or nothing if the archive
    /// does not say.
    pub modified: String,
}

impl Entry {
    /// The letters on its tag: DIR for a folder, else its extension.
    pub fn tag(&self) -> String {
        tag_of(&self.name, self.kind)
    }

    fn of(child: &Child) -> Entry {
        Entry {
            name: child.name.clone(),
            kind: Kind::of(&child.name, child.dir),
            size: child.size,
            modified: child.modified.map(date).unwrap_or_default(),
        }
    }
}

/// An open archive, and the folder of it shown.
#[derive(Clone, Debug, PartialEq)]
pub struct Archive {
    pub name: String,
    /// The folder it is in, as the user knows it.
    pub location: String,
    /// Its file: none for the samples.
    pub path: Option<PathBuf>,
    /// Its own size, and what it holds unpacked, in bytes.
    pub packed: u64,
    pub unpacked: u64,
    pub files: u64,
    pub encryption: Option<String>,
    /// The folder shown in it, from its top.
    pub folder: Vec<String>,
    /// What that folder holds.
    pub entries: Vec<Entry>,
    /// Programs in it that came from the Internet: they keep the mark.
    pub marked: Vec<String>,
    /// All it holds, to show its other folders: none for the samples.
    pub listing: Option<Arc<Listing>>,
    /// The passphrase and password it was opened with, which extracting
    /// and testing it take again: kept in memory while it is open, never
    /// written down.
    pub keys: Keys,
}

impl Archive {
    /// The archive at `path`, as packwing-core listed it with `keys`; its
    /// top folder shown. `marked`: the file came from the Internet, and the
    /// programs in it will keep that mark.
    pub fn read(path: PathBuf, listing: Listing, keys: Keys, marked: bool) -> Archive {
        let location = listing
            .folder
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| listing.folder.display().to_string());
        let marked = if marked {
            listing
                .entries
                .iter()
                .filter(|e| !e.dir)
                .filter_map(|e| e.path.last())
                .filter(|name| Kind::of(name, false) == Kind::Program)
                .cloned()
                .collect()
        } else {
            Vec::new()
        };
        let mut archive = Archive {
            name: listing.name.clone(),
            location,
            path: Some(path),
            packed: listing.packed,
            unpacked: listing.unpacked,
            files: listing.files,
            encryption: listing.encryption.clone(),
            folder: Vec::new(),
            entries: Vec::new(),
            marked,
            listing: Some(Arc::new(listing)),
            keys,
        };
        archive.show(Vec::new());
        archive
    }

    /// Shows `folder`, from the archive's top: what it holds, its folders
    /// first. Nothing changes for a sample, which has one folder.
    pub fn show(&mut self, folder: Vec<String>) {
        let Some(listing) = &self.listing else {
            return;
        };
        self.entries = listing.children(&folder).iter().map(Entry::of).collect();
        self.folder = folder;
    }

    /// Its name without the archive's extensions: the folder it unpacks to.
    pub fn stem(&self) -> &str {
        stem(&self.name)
    }

    /// How much of its unpacked size it takes, in percent.
    pub fn ratio(&self) -> u64 {
        if self.unpacked == 0 {
            return 100;
        }
        (self.packed * 100 + self.unpacked / 2) / self.unpacked
    }
}

/// The day of `secs` since 1970, in UTC: `2026-10-15`.
pub fn date(secs: i64) -> String {
    // Howard Hinnant's civil_from_days.
    let z = secs.div_euclid(86_400) + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

/// The letters on a file's tag: DIR for a folder, else its last extension in
/// capitals (`tar.zst` gives ZST), at most four.
pub fn tag_of(name: &str, kind: Kind) -> String {
    if kind == Kind::Folder {
        return "DIR".into();
    }
    // A split archive's part number is not its type.
    match name.rsplit_once('.') {
        Some((_, ext)) if !ext.is_empty() && ext.len() <= 4 => ext.to_uppercase(),
        _ => "FILE".into(),
    }
}

/// `name` without its archive extensions and a part number: what it unpacks
/// to (`Báo cáo.part1.rar` → `Báo cáo`, `backup.tar.zst.age` → `backup`).
pub fn stem(name: &str) -> &str {
    const EXTENSIONS: [&str; 22] = [
        ".age", ".zst", ".gz", ".xz", ".bz2", ".lz4", ".lz", ".lzma", ".tgz", ".tbz2", ".txz",
        ".tzst", ".tlz", ".tar", ".7z", ".zip", ".rar", ".iso", ".cpio", ".rpm", ".xar", ".pkg",
    ];
    let mut s = name;
    while let Some(rest) = EXTENSIONS
        .iter()
        .find_map(|ext| s.strip_suffix(ext).filter(|r| !r.is_empty()))
    {
        s = rest;
    }
    if let Some((rest, part)) = s.rsplit_once(".part")
        && !part.is_empty()
        && part.bytes().all(|b| b.is_ascii_digit())
    {
        s = rest;
    }
    s
}

/// A size as people read it: three figures at most, no trailing zeros, in SI
/// units as macOS shows them (`880 KB`, `6.4 MB`, `412 MB`, `1.24 GB`).
pub fn size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["bytes", "KB", "MB", "GB", "TB"];
    if bytes < 1000 {
        return format!("{bytes} {}", if bytes == 1 { "byte" } else { "bytes" });
    }
    let mut v = bytes as f64;
    let mut unit = 0;
    while v >= 999.5 && unit < UNITS.len() - 1 {
        v /= 1000.;
        unit += 1;
    }
    let text = if unit == 1 || v >= 99.95 {
        format!("{v:.0}")
    } else if unit == 2 || v >= 9.995 {
        format!("{v:.1}")
    } else {
        format!("{v:.2}")
    };
    let text = match text.split_once('.') {
        Some(_) => text.trim_end_matches('0').trim_end_matches('.'),
        None => &text,
    };
    format!("{text} {}", UNITS[unit])
}

/// A count with thousands grouped: `1,284`.
pub fn count(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, d) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(d);
    }
    out
}

/// `n` and the noun, singular or plural: `1 file`, `1,284 files`.
pub fn counted(n: u64, one: &str, many: &str) -> String {
    format!("{} {}", count(n), if n == 1 { one } else { many })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_read_as_the_design_shows_them() {
        assert_eq!(size(0), "0 bytes");
        assert_eq!(size(1), "1 byte");
        assert_eq!(size(12_000), "12 KB");
        assert_eq!(size(880_000), "880 KB");
        assert_eq!(size(2_100_000), "2.1 MB");
        assert_eq!(size(86_200_000), "86.2 MB");
        assert_eq!(size(412_000_000), "412 MB");
        assert_eq!(size(1_240_000_000), "1.24 GB");
        assert_eq!(size(999_999), "1 MB");
        assert_eq!(size(1_800_000_000), "1.8 GB");
        assert_eq!(size(38_000_000), "38 MB");
        assert_eq!(size(18_400_000_000_000), "18.4 TB");
    }

    #[test]
    fn counts_group_thousands() {
        assert_eq!(count(7), "7");
        assert_eq!(count(1284), "1,284");
        assert_eq!(count(1_000_000), "1,000,000");
        assert_eq!(counted(1, "file", "files"), "1 file");
        assert_eq!(counted(1284, "file", "files"), "1,284 files");
    }

    #[test]
    fn stems_drop_archive_extensions_and_parts() {
        assert_eq!(stem("Tài liệu dự án 2026.7z"), "Tài liệu dự án 2026");
        assert_eq!(stem("Báo cáo thuế Q3.part1.rar"), "Báo cáo thuế Q3");
        assert_eq!(stem("backup-2026-10.tar.zst"), "backup-2026-10");
        assert_eq!(stem("backup.tar.zst.age"), "backup");
        assert_eq!(stem("logs.tgz"), "logs");
        assert_eq!(stem("ubuntu-26.04-desktop.iso"), "ubuntu-26.04-desktop");
        assert_eq!(stem("disc.iso.xz"), "disc");
        assert_eq!(stem("hello-1.0-1.x86_64.rpm"), "hello-1.0-1.x86_64");
        assert_eq!(stem("Install Example.pkg"), "Install Example");
        assert_eq!(stem("initramfs.cpio.zst"), "initramfs");
        assert_eq!(stem("notes.txt"), "notes.txt");
        assert_eq!(stem(".tar"), ".tar");
    }

    #[test]
    fn days_from_seconds() {
        assert_eq!(date(0), "1970-01-01");
        assert_eq!(date(1_500_000_000), "2017-07-14");
        assert_eq!(date(1_772_323_200), "2026-03-01");
        assert_eq!(date(951_782_400), "2000-02-29");
        assert_eq!(date(-86_400), "1969-12-31");
    }

    #[test]
    fn kinds_from_extensions() {
        assert_eq!(Kind::of("a.PDF", false), Kind::Pdf);
        assert_eq!(Kind::of("photo.jpeg", false), Kind::Image);
        assert_eq!(Kind::of("setup.exe", false), Kind::Program);
        assert_eq!(Kind::of("backup.tar.zst", false), Kind::Archive);
        assert_eq!(Kind::of("Makefile", false), Kind::Text);
        assert_eq!(Kind::of("docs", true), Kind::Folder);
    }

    #[test]
    fn tags_come_from_the_extension() {
        let e = |name: &str, kind| Entry {
            name: name.into(),
            kind,
            size: 0,
            modified: String::new(),
        };
        assert_eq!(e("Ảnh", Kind::Folder).tag(), "DIR");
        assert_eq!(e("a.pdf", Kind::Pdf).tag(), "PDF");
        assert_eq!(e("b.docx", Kind::Document).tag(), "DOCX");
        assert_eq!(tag_of("x.tar.zst", Kind::Archive), "ZST");
    }
}
