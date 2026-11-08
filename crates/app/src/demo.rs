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
        target: None,
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
        folder: vec!["Design".into(), "Construction docs".into()],
        entries: vec![
            entry("Site photos", Kind::Folder, 86_200_000, "2026-10-15"),
            entry(
                "Floor plan, level 1.pdf",
                Kind::Pdf,
                14_800_000,
                "2026-10-12",
            ),
            entry("Front elevation.jpg", Kind::Image, 6_400_000, "2026-10-15"),
            entry(
                "Construction contract – signed.docx",
                Kind::Document,
                2_100_000,
                "2026-10-09",
            ),
            entry("Cost estimate.xlsx", Kind::Sheet, 880_000, "2026-10-14"),
            entry("Meeting notes 14-10.txt", Kind::Text, 12_000, "2026-10-14"),
            entry("setup_tool.exe", Kind::Program, 3_200_000, "2026-09-30"),
        ],
        marked: vec!["setup_tool.exe".into()],
        listing: None,
        keys: packwing_core::Keys::default(),
    }
}

const PROJECT_NAME: &str = "Project files 2026.7z";
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
        name: "Tax report Q3.part1.rar",
        location: "Desktop",
        size: 2_300_000_000,
        opened: "Yesterday, 17:40",
    },
    Recent {
        name: "Wedding photos – originals.zip",
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
        paths: vec!["/home/you/Documents/Quarterly report Q3".into()],
        files: 6,
        bytes: 1_800_000_000,
        ..Default::default()
    }
}

/// More chosen than the list of them shows at once.
pub fn many_chosen() -> crate::model::Selection {
    let names = [
        "Quarterly report Q3",
        "Construction contract – signed.docx",
        "Front elevation.jpg",
        "Cost estimate.xlsx",
        "Meeting notes 14-10.txt",
        "setup_tool.exe",
        "Floor plan, level 1.pdf",
        "Site photos",
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

/// The audit log the screenshots show: a morning's jobs, the latest last.
#[cfg(all(test, feature = "test-hooks"))]
pub fn audit_records() -> Vec<serde_json::Map<String, serde_json::Value>> {
    use serde_json::json;
    let who = json!({"user": "minh", "host": "studio", "app": "packwing", "version": "0.9.0"});
    [
        json!({"time": "2026-11-08T08:58:12.204Z", "action": "audit.started", "by": "startup",
               "retention_days": 90, "max_bytes": 104_857_600,
               "folder": "/home/minh/.local/state/packwing/audit"}),
        json!({"time": "2026-11-08T09:02:40.918Z", "action": "extract", "phase": "start", "job": "a1-1",
               "source": "/home/minh/Downloads/brand-assets-2026.zip",
               "destination": "/home/minh/Projects/brand-assets-2026", "overwrite": "ask",
               "keep_mark": true, "links_outside": "skip", "owners": "ignore"}),
        json!({"time": "2026-11-08T09:02:43.511Z", "action": "extract", "phase": "end", "job": "a1-1",
               "result": "ok", "format": "zip", "files": 214, "directories": 18,
               "bytes": 486_201_344_u64, "duration_ms": 2593}),
        json!({"time": "2026-11-08T09:10:05.077Z", "action": "read_entry", "phase": "start", "job": "a1-2",
               "source": "/home/minh/Downloads/brand-assets-2026.zip", "entry": "README.md", "via": "preview"}),
        json!({"time": "2026-11-08T09:10:05.102Z", "action": "read_entry", "phase": "end", "job": "a1-2",
               "result": "ok", "bytes": 2048, "duration_ms": 25}),
        json!({"time": "2026-11-08T09:31:19.640Z", "action": "test", "phase": "start", "job": "a1-3",
               "source": "/home/minh/Backups/photos-2025.7z"}),
        json!({"time": "2026-11-08T09:31:21.002Z", "action": "test", "phase": "end", "job": "a1-3",
               "result": "failed", "error": "The password is not this archive's.", "duration_ms": 1362}),
        json!({"time": "2026-11-08T09:45:52.330Z", "action": "create", "phase": "start", "job": "a1-4",
               "destination": "/home/minh/Projects/q4-report.zip", "kind": "zip", "level": 6,
               "encryption": "zip-aes256", "paths": 3,
               "path_names": ["/home/minh/Projects/q4/report.pdf", "/home/minh/Projects/q4/figures",
                              "/home/minh/Projects/q4/data.csv"]}),
        json!({"time": "2026-11-08T09:45:55.871Z", "action": "create", "phase": "end", "job": "a1-4",
               "result": "ok", "made": "/home/minh/Projects/q4-report.zip", "duration_ms": 3541}),
        json!({"time": "2026-11-08T10:05:08.415Z", "action": "copy", "phase": "start", "job": "a1-5",
               "from": "/home/minh/Projects/q4-report.zip", "to": "s3://studio-archive/2026/",
               "names": ["q4-report.zip"]}),
        json!({"time": "2026-11-08T10:05:31.009Z", "action": "copy", "phase": "end", "job": "a1-5",
               "result": "ok", "files": 1, "bytes": 31_457_280, "duration_ms": 22_594}),
        json!({"time": "2026-11-08T10:12:44.180Z", "action": "convert", "phase": "start", "job": "a1-6",
               "source": "/home/minh/Downloads/old-site.rar",
               "destination": "/home/minh/Archive/old-site.tar.zst", "kind": "tar.zst"}),
    ]
    .into_iter()
    .filter_map(|mut r| {
        let r = r.as_object_mut()?;
        for (k, v) in who.as_object()? {
            r.insert(k.clone(), v.clone());
        }
        Some(r.clone())
    })
    .collect()
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
