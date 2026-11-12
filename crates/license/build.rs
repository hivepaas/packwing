//! The day this build's line, its major and minor version, was first released, for
//! [`line_released`](crate::line_released): a license bought outright covers a line
//! released before its updates end, with every fix of that line after.
//!
//! It is the earliest `releaseDate` of the line in `release.json`, the list of
//! releases. The first release of a line is not in the list yet when it is built, so
//! then it is the day it is built (`SOURCE_DATE_EPOCH`, if set, as a reproducible
//! build says).

use std::path::Path;

fn main() {
    let list = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../release.json");
    println!("cargo:rerun-if-changed={}", list.display());
    println!("cargo:rerun-if-env-changed=SOURCE_DATE_EPOCH");
    let version = std::env::var("CARGO_PKG_VERSION").expect("a version");
    let day = std::fs::read(&list)
        .ok()
        .and_then(|text| first_of_line(&text, &version))
        .unwrap_or_else(today);
    println!("cargo:rustc-env=PACKWING_LINE_RELEASED={day}");
}

/// `major.minor.` of `version`, and whether it is a pre-release.
fn line(version: &str) -> (String, bool) {
    let version = version.trim_start_matches('v');
    let mut parts = version.splitn(3, '.');
    let (major, minor) = (parts.next().unwrap_or(""), parts.next().unwrap_or(""));
    (format!("{major}.{minor}."), version.contains('-'))
}

/// The earliest release date of `version`'s line in the list: of its releases, or of
/// its pre-releases if this is one and the line has no release yet.
fn first_of_line(text: &[u8], version: &str) -> Option<String> {
    let list: serde_json::Value = serde_json::from_slice(text).ok()?;
    let (want, pre) = line(version);
    let dates = |with_pre: bool| {
        list["releases"]
            .as_object()
            .into_iter()
            .flatten()
            .filter(|(name, _)| {
                let (of, is_pre) = line(name);
                of == want && (with_pre || !is_pre)
            })
            .filter_map(|(_, release)| release["releaseDate"].as_str())
            .filter(|d| d.len() == 10)
            .map(str::to_owned)
            .min()
    };
    dates(false).or_else(|| if pre { dates(true) } else { None })
}

fn today() -> String {
    let seconds = std::env::var("SOURCE_DATE_EPOCH")
        .ok()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_secs() as i64)
        });
    let date = time::OffsetDateTime::from_unix_timestamp(seconds)
        .unwrap_or(time::OffsetDateTime::UNIX_EPOCH)
        .date();
    format!(
        "{:04}-{:02}-{:02}",
        date.year(),
        u8::from(date.month()),
        date.day()
    )
}
