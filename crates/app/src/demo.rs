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
        source: None,
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
    },
    Recent {
        name: "Báo cáo thuế Q3.part1.rar",
        location: "Desktop",
        size: 2_300_000_000,
        opened: "Yesterday, 17:40",
    },
    Recent {
        name: "Ảnh cưới – bản gốc.zip",
        location: "Pictures",
        size: 3_800_000_000,
        opened: "2026-10-20",
    },
    Recent {
        name: "backup-2026-10.tar.zst",
        location: "Backups",
        size: 940_000_000,
        opened: "2026-10-14",
    },
];

/// What New archive holds in the screenshots.
#[cfg(all(test, feature = "test-hooks"))]
pub fn selection() -> crate::model::Selection {
    crate::model::Selection {
        paths: vec!["/home/you/Documents/Báo cáo quý 3".into()],
        files: 6,
        bytes: 1_800_000_000,
        ..Default::default()
    }
}

/// More chosen than the list of them shows at once.
pub fn many_chosen() -> crate::model::Selection {
    let names = [
        "Báo cáo quý 3",
        "Hợp đồng thi công – bản ký.docx",
        "Phối cảnh mặt tiền.jpg",
        "Dự toán chi phí.xlsx",
        "Ghi chú họp ngày 14-10.txt",
        "setup_tool.exe",
        "Bản vẽ mặt bằng tầng 1.pdf",
        "Ảnh hiện trạng",
        "backup-2026-10.tar.zst",
    ];
    let paths: Vec<std::path::PathBuf> = names
        .iter()
        .map(|n| std::path::Path::new("/home/you/Documents").join(n))
        .collect();
    crate::model::Selection {
        folders: [paths[0].clone(), paths[7].clone()].into_iter().collect(),
        paths,
        files: 412,
        bytes: 1_800_000_000,
        not_portable: Vec::new(),
    }
}

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
    }
}
