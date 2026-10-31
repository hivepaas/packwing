//! Samples: the archive and dialogs the screenshots show, and the Recent
//! page until packwing keeps its own. Everything here is made up; nothing is
//! read from disk. The archive fills the same [`Archive`] and [`Entry`] that
//! an archive read through hivepack fills (`model.rs`).

use crate::model::{Archive, Entry, Kind, tag_of};

/// The archive the screenshots open.
pub fn project() -> Archive {
    let entry = |name: &str, kind, size, modified: &str| Entry {
        name: name.into(),
        kind,
        size,
        modified: modified.into(),
    };
    Archive {
        name: PROJECT_NAME.into(),
        location: PROJECT_LOCATION.into(),
        path: None,
        packed: PROJECT_PACKED,
        unpacked: 1_240_000_000,
        files: 1284,
        encryption: Some("AES-256".into()),
        folder: vec!["Thiết kế".into(), "Hồ sơ thi công".into()],
        entries: vec![
            entry("Ảnh hiện trạng", Kind::Folder, 86_200_000, "2026-10-15"),
            entry(
                "Bản vẽ mặt bằng tầng 1.pdf",
                Kind::Pdf,
                14_800_000,
                "2026-10-12",
            ),
            entry(
                "Phối cảnh mặt tiền.jpg",
                Kind::Image,
                6_400_000,
                "2026-10-15",
            ),
            entry(
                "Hợp đồng thi công – bản ký.docx",
                Kind::Document,
                2_100_000,
                "2026-10-09",
            ),
            entry("Dự toán chi phí.xlsx", Kind::Sheet, 880_000, "2026-10-14"),
            entry(
                "Ghi chú họp ngày 14-10.txt",
                Kind::Text,
                12_000,
                "2026-10-14",
            ),
            entry("setup_tool.exe", Kind::Program, 3_200_000, "2026-09-30"),
        ],
        marked: vec!["setup_tool.exe".into()],
        listing: None,
        keys: packwing_core::Keys::default(),
    }
}

const PROJECT_NAME: &str = "Tài liệu dự án 2026.7z";
const PROJECT_LOCATION: &str = "Downloads";
const PROJECT_PACKED: u64 = 412_000_000;

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
        name: PROJECT_NAME,
        location: PROJECT_LOCATION,
        size: PROJECT_PACKED,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn samples_read_as_the_design_shows_them() {
        let p = project();
        assert_eq!(p.entries[0].tag(), "DIR");
        assert_eq!(p.entries[1].tag(), "PDF");
        assert_eq!(p.entries[3].tag(), "DOCX");
        assert_eq!(RECENT[3].tag(), "ZST");
        assert_eq!(p.ratio(), 33);
        assert_eq!(DAMAGED.missing_number(), "3");
    }
}
