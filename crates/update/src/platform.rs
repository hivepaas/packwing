//! What a release has for each platform, and where it is downloaded from.
//!
//! | platform | file |
//! |---|---|
//! | `linux_x86_64`, `linux_aarch64` | `packwing_0.2.0_linux_x86_64.AppImage` |
//! | `macos_aarch64`, `macos_x86_64` | `packwing_0.2.0_macos_aarch64.app.tar.gz` (`Packwing.app`, tarred) |
//! | `windows_x86_64`, `windows_aarch64` | `packwing_0.2.0_windows_x86_64-setup.exe` (an installer, quiet with `/S`) |

use crate::version::Version;

pub const PLATFORMS: [&str; 6] = [
    "linux_x86_64",
    "linux_aarch64",
    "macos_aarch64",
    "macos_x86_64",
    "windows_x86_64",
    "windows_aarch64",
];

/// This build's platform, if releases are made for it.
pub fn current() -> Option<&'static str> {
    let os = match std::env::consts::OS {
        "linux" => "linux",
        "macos" => "macos",
        "windows" => "windows",
        _ => return None,
    };
    let arch = match std::env::consts::ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        _ => return None,
    };
    let name = format!("{os}_{arch}");
    PLATFORMS.into_iter().find(|p| *p == name)
}

/// The file a release has for `platform`.
pub fn archive_file(v: &Version, platform: &str) -> Option<String> {
    if !PLATFORMS.contains(&platform) {
        return None;
    }
    let plain = v.to_string();
    let plain = plain.trim_start_matches('v');
    let os = platform.split('_').next()?;
    Some(match os {
        "linux" => format!("packwing_{plain}_{platform}.AppImage"),
        "macos" => format!("packwing_{plain}_{platform}.app.tar.gz"),
        _ => format!("packwing_{plain}_{platform}-setup.exe"),
    })
}

/// The system packages a release has, by name in the list's `packages`:
/// `deb_x86_64` is `packwing_0.2.0_amd64.deb`.
pub const PACKAGES: [&str; 2] = ["deb_x86_64", "deb_aarch64"];

/// This build's system package, by its name in the list.
pub fn package_key() -> String {
    format!("deb_{}", std::env::consts::ARCH)
}

/// The file a release has for the system package `key`.
pub fn package_file(v: &Version, key: &str) -> Option<String> {
    let plain = v.to_string();
    let plain = plain.trim_start_matches('v');
    Some(match key {
        "deb_x86_64" => format!("packwing_{plain}_amd64.deb"),
        "deb_aarch64" => format!("packwing_{plain}_arm64.deb"),
        _ => return None,
    })
}

/// Where releases' files are: a release's assets on GitHub.
const DOWNLOADS: &str = "https://github.com/hivepaas/packwing/releases/download";

/// Where a release's file is downloaded from.
pub fn download_url(v: &Version, file: &str) -> String {
    format!("{DOWNLOADS}/{v}/{file}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_platform_has_its_file() {
        let v = Version::parse("v0.3.0-beta2").expect("version");
        assert_eq!(
            archive_file(&v, "linux_x86_64").as_deref(),
            Some("packwing_0.3.0-beta2_linux_x86_64.AppImage")
        );
        assert_eq!(
            archive_file(&v, "macos_aarch64").as_deref(),
            Some("packwing_0.3.0-beta2_macos_aarch64.app.tar.gz")
        );
        assert_eq!(
            archive_file(&v, "windows_x86_64").as_deref(),
            Some("packwing_0.3.0-beta2_windows_x86_64-setup.exe")
        );
        assert_eq!(archive_file(&v, "plan9_mips"), None);
        for key in PACKAGES {
            assert!(package_file(&v, key).is_some(), "{key}");
        }
        assert_eq!(
            package_file(&v, "deb_aarch64").as_deref(),
            Some("packwing_0.3.0-beta2_arm64.deb")
        );
        assert_eq!(package_file(&v, "snap_x86_64"), None);
        assert_eq!(
            download_url(&v, "x.AppImage"),
            "https://github.com/hivepaas/packwing/releases/download/v0.3.0-beta2/x.AppImage"
        );
        assert!(current().is_some_and(|p| PLATFORMS.contains(&p)));
    }
}
