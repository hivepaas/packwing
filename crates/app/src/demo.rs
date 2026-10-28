//! Sample archives the screens show until packwing reads real ones through
//! hivepack. Everything here is made up; nothing is read from disk.
//!
//! The screens take what they show from these types alone, so reading an
//! archive for real fills the same [`Archive`] and [`Entry`] from hivepack.

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
}

/// A file or folder in an archive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    pub name: &'static str,
    pub kind: Kind,
    /// Unpacked, in bytes.
    pub size: u64,
    pub modified: &'static str,
}

impl Entry {
    /// The letters on its tag: DIR for a folder, else its extension.
    pub fn tag(&self) -> String {
        tag_of(self.name, self.kind)
    }
}

/// An open archive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Archive {
    pub name: &'static str,
    /// The folder it is in, as the user knows it.
    pub location: &'static str,
    /// Its own size, and what it holds unpacked, in bytes.
    pub packed: u64,
    pub unpacked: u64,
    pub files: u64,
    pub encryption: Option<&'static str>,
    /// The folder shown in it, from its top.
    pub folder: &'static [&'static str],
    pub entries: &'static [Entry],
    /// Programs in it that came from the Internet: they keep the mark.
    pub marked: &'static [&'static str],
}

impl Archive {
    /// Its name without the archive's extensions: the folder it unpacks to.
    pub fn stem(&self) -> &'static str {
        stem(self.name)
    }

    /// How much of its unpacked size it takes, in percent.
    pub fn ratio(&self) -> u64 {
        if self.unpacked == 0 {
            return 100;
        }
        (self.packed * 100 + self.unpacked / 2) / self.unpacked
    }
}

pub const PROJECT: Archive = Archive {
    name: "Tài liệu dự án 2026.7z",
    location: "Downloads",
    packed: 412_000_000,
    unpacked: 1_240_000_000,
    files: 1284,
    encryption: Some("AES-256"),
    folder: &["Thiết kế", "Hồ sơ thi công"],
    entries: &[
        Entry {
            name: "Ảnh hiện trạng",
            kind: Kind::Folder,
            size: 86_200_000,
            modified: "2026-10-15",
        },
        Entry {
            name: "Bản vẽ mặt bằng tầng 1.pdf",
            kind: Kind::Pdf,
            size: 14_800_000,
            modified: "2026-10-12",
        },
        Entry {
            name: "Phối cảnh mặt tiền.jpg",
            kind: Kind::Image,
            size: 6_400_000,
            modified: "2026-10-15",
        },
        Entry {
            name: "Hợp đồng thi công – bản ký.docx",
            kind: Kind::Document,
            size: 2_100_000,
            modified: "2026-10-09",
        },
        Entry {
            name: "Dự toán chi phí.xlsx",
            kind: Kind::Sheet,
            size: 880_000,
            modified: "2026-10-14",
        },
        Entry {
            name: "Ghi chú họp ngày 14-10.txt",
            kind: Kind::Text,
            size: 12_000,
            modified: "2026-10-14",
        },
        Entry {
            name: "setup_tool.exe",
            kind: Kind::Program,
            size: 3_200_000,
            modified: "2026-09-30",
        },
    ],
    marked: &["setup_tool.exe"],
};

/// An archive opened before.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Recent {
    pub name: &'static str,
    pub location: &'static str,
    pub size: u64,
    pub opened: &'static str,
    /// Part of it cannot be read: opening it shows [`DAMAGED`].
    pub damaged: bool,
}

impl Recent {
    pub fn tag(&self) -> String {
        tag_of(self.name, Kind::Archive)
    }
}

pub const RECENT: &[Recent] = &[
    Recent {
        name: PROJECT.name,
        location: PROJECT.location,
        size: PROJECT.packed,
        opened: "Today, 09:12",
        damaged: false,
    },
    Recent {
        name: "Báo cáo thuế Q3.part1.rar",
        location: "Desktop",
        size: 2_300_000_000,
        opened: "Yesterday, 17:40",
        damaged: true,
    },
    Recent {
        name: "Ảnh cưới – bản gốc.zip",
        location: "Pictures",
        size: 3_800_000_000,
        opened: "2026-10-20",
        damaged: false,
    },
    Recent {
        name: "backup-2026-10.tar.zst",
        location: "Backups",
        size: 940_000_000,
        opened: "2026-10-14",
        damaged: false,
    },
];

/// A split archive with a part missing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Damaged {
    pub name: &'static str,
    pub location: &'static str,
    /// Unpacked, in bytes.
    pub unpacked: u64,
    pub files: u64,
    /// Those that read well.
    pub fine: u64,
    /// The folder the others are in.
    pub folder: &'static str,
    pub missing_part: &'static str,
    /// It carries recovery data, so repair can be tried.
    pub recoverable: bool,
}

pub const DAMAGED: Damaged = Damaged {
    name: "Báo cáo thuế Q3.part1.rar",
    location: "Desktop",
    unpacked: 2_900_000_000,
    files: 1284,
    fine: 1212,
    folder: "Hóa đơn",
    missing_part: "Báo cáo thuế Q3.part3.rar",
    recoverable: true,
};

impl Damaged {
    /// The missing part's number, as people count them: `3`.
    pub fn missing_number(&self) -> &'static str {
        self.missing_part
            .rsplit_once(".part")
            .and_then(|(_, rest)| rest.split('.').next())
            .unwrap_or("")
    }
}

/// A setting the user's organization has fixed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Policy {
    pub label: &'static str,
    pub detail: &'static str,
    /// What it is fixed at, as the user reads it.
    pub value: &'static str,
    /// Its name in the policy file, for IT.
    pub key: &'static str,
}

/// packwing as an organization runs it: who set it up, and what they fixed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Managed {
    pub by: &'static str,
    pub contact: &'static str,
    pub organization: &'static str,
    pub seats: u64,
    pub channel: &'static str,
    pub policies: &'static [Policy],
}

pub const MANAGED: Managed = Managed {
    by: "IT department – Example Co.",
    contact: "it@example.com",
    organization: "Example Co.",
    seats: 250,
    channel: "LTS",
    policies: &[
        Policy {
            label: "Keep the “downloaded from the Internet” mark",
            detail: "The system warns before running files that came from a downloaded archive",
            value: "Always on",
            key: "PropagateMotW",
        },
        Policy {
            label: "Require a password for new archives",
            detail: "Archives can’t be created without encryption",
            value: "AES-256",
            key: "RequireEncryption",
        },
        Policy {
            label: "Minimum password length",
            detail: "Applies to every new archive",
            value: "12 characters",
            key: "MinPasswordLength",
        },
        Policy {
            label: "FIPS cryptography",
            detail: "Only validated algorithms; old ZipCrypto is blocked",
            value: "On",
            key: "FipsMode",
        },
        Policy {
            label: "Password escrow",
            detail: "IT can recover the password if you lose it",
            value: "On",
            key: "PasswordEscrow",
        },
        Policy {
            label: "Activity log",
            detail: "Compress and extract actions are written to the system log",
            value: "On",
            key: "AuditLog",
        },
        Policy {
            label: "Block programs from Internet archives",
            detail: ".exe, .msi, .js and .lnk files from downloaded archives",
            value: "Blocked",
            key: "BlockInternetExecutables",
        },
    ],
};

/// What New archive starts with: the files chosen to pack.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Selection {
    pub name: &'static str,
    pub items: u64,
    pub size: u64,
}

pub const SELECTION: Selection = Selection {
    name: "Báo cáo quý 3",
    items: 6,
    size: 1_800_000_000,
};

/// The letters on a file's tag: DIR for a folder, else its last extension in
/// capitals (`tar.zst` gives ZST), at most four.
fn tag_of(name: &str, kind: Kind) -> String {
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
/// to (`Báo cáo.part1.rar` → `Báo cáo`, `backup.tar.zst` → `backup`).
pub fn stem(name: &str) -> &str {
    let mut s = name;
    for ext in [".zst", ".gz", ".xz", ".bz2", ".tar", ".7z", ".zip", ".rar"] {
        if let Some(rest) = s.strip_suffix(ext) {
            s = rest;
        }
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
        assert_eq!(stem("notes.txt"), "notes.txt");
    }

    #[test]
    fn tags_come_from_the_extension() {
        assert_eq!(PROJECT.entries[0].tag(), "DIR");
        assert_eq!(PROJECT.entries[1].tag(), "PDF");
        assert_eq!(PROJECT.entries[3].tag(), "DOCX");
        assert_eq!(RECENT[3].tag(), "ZST");
        assert_eq!(PROJECT.ratio(), 33);
        assert_eq!(DAMAGED.missing_number(), "3");
    }
}
