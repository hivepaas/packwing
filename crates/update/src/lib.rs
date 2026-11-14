//! Telling that a newer packwing is out: a signed list of releases
//! (`release.signed.json`, on the repository's `release` branch, made by
//! `tools/releasesign`) says each channel's newest; the app on
//! stable is offered stable releases only, the app on beta the newer of the
//! betas and stable (a release supersedes the betas before it). The list is
//! believed only when its signatures verify against the keys built in.
//!
//! A copy the app can replace (an AppImage, an `.app`, one its installer put
//! on Windows) downloads the release's file for its platform, checks it
//! against the list's SHA-256, and installs it as it quits (`apply`). A copy
//! from the release's `.deb` or `.rpm` is given the commands that install the
//! next one. Any other copy is told where the release is, or what updates it.

pub mod apply;
pub mod download;
pub mod envelope;
pub mod install;
pub mod license;
pub mod manifest;
pub mod platform;
pub mod version;

use std::time::Duration;

pub use envelope::{PublicKeys, SignatureError};
pub use install::{Install, PackageTool, install_of};
pub use manifest::{Manifest, ManifestError};
pub use version::{Channel, Version};

/// What the list is signed for. The keys are shellwing's too, under its own context: a
/// signature made with them for another purpose or product is never taken for this.
pub const CONTEXT: &str = "packwing-release-v1";

/// Where the signed list is read: the release branch, which moves to a
/// release once it is checked.
pub const LIST_URL: &str =
    "https://raw.githubusercontent.com/hivepaas/packwing/release/release.signed.json";

/// The list is small; anything bigger is not it.
const MOST_LIST: usize = 1 << 20;
const FETCH_TIMEOUT: Duration = Duration::from_secs(10);

/// The page a release is downloaded from.
pub fn release_page(v: &Version) -> String {
    format!("https://github.com/hivepaas/packwing/releases/tag/{v}")
}

/// The version running.
pub fn current() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).unwrap_or(Version {
        major: 0,
        minor: 0,
        patch: 0,
        pre: None,
    })
}

// KEYS: every keys/<id>.pub.pem, by id (build.rs).
include!(concat!(env!("OUT_DIR"), "/keys.rs"));

/// Built for the Mac App Store, which updates it (the `app-store` feature): it never
/// looks for a release, and never downloads nor installs one, as the App Store's
/// rules ask (App Review Guidelines 2.4.5).
pub const STORE_BUILD: bool = cfg!(feature = "app-store");

/// What a store build says when asked to download or install a release.
const STORE_UPDATES_IT: &str = "the App Store updates this copy";

/// This build has release keys: it can tell of updates. One without (a
/// build before the keys were made) does not look, nor does a store build.
pub fn has_keys() -> bool {
    !STORE_BUILD && !KEYS.is_empty()
}

/// The release keys this app trusts (keys/README.md), or the test's.
pub fn trusted_keys() -> Result<PublicKeys, SignatureError> {
    #[cfg(feature = "test-source")]
    if let Some((_, keys)) = test_source() {
        let pems: Vec<(&str, &str)> = keys.iter().map(|(i, p)| (i.as_str(), p.as_str())).collect();
        return envelope::parse_keys(&pems);
    }
    envelope::parse_keys(KEYS)
}

/// Where the signed list is read: [`LIST_URL`], or the test's.
pub fn list_url() -> String {
    #[cfg(feature = "test-source")]
    if let Some((url, _)) = test_source() {
        return url;
    }
    LIST_URL.to_string()
}

/// How this copy was installed, in place of what its path says: for the app's
/// UI tests.
#[cfg(feature = "test-source")]
static TEST_INSTALL: std::sync::Mutex<Option<Install>> = std::sync::Mutex::new(None);

#[cfg(feature = "test-source")]
pub fn set_test_install(install: Install) {
    if let Ok(mut i) = TEST_INSTALL.lock() {
        *i = Some(install);
    }
}

#[cfg(feature = "test-source")]
pub fn test_install() -> Option<Install> {
    TEST_INSTALL.lock().ok()?.clone()
}

/// A list and keys of a test's own: the url, and each key's id and PEM.
#[cfg(feature = "test-source")]
type Source = (String, Vec<(String, String)>);

#[cfg(feature = "test-source")]
static TEST_SOURCE: std::sync::Mutex<Option<Source>> = std::sync::Mutex::new(None);

#[cfg(feature = "test-source")]
fn test_source() -> Option<Source> {
    TEST_SOURCE.lock().ok()?.clone()
}

/// Reads the signed list at `url` and checks it with `keys` (id, PEM), in
/// place of the release's: for the app's UI tests.
#[cfg(feature = "test-source")]
pub fn set_test_source(url: &str, keys: &[(&str, &str)]) {
    if let Ok(mut s) = TEST_SOURCE.lock() {
        *s = Some((
            url.to_string(),
            keys.iter()
                .map(|(i, p)| (i.to_string(), p.to_string()))
                .collect(),
        ));
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CheckError {
    #[error("{0}")]
    Fetch(String),
    /// No list is published yet (before the first release): nothing to offer.
    #[error("no release is published yet")]
    NoList,
    #[error(transparent)]
    Signature(#[from] SignatureError),
    #[error(transparent)]
    Manifest(#[from] ManifestError),
    #[error("the download is not the file the list of releases names: its SHA-256 differs")]
    Checksum,
    #[error("{0}")]
    Io(String),
    #[error("the release has no file for this platform")]
    NoFile,
}

/// A release's file for this platform, as the signed list names it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReleaseFile {
    pub version: Version,
    pub file: String,
    pub sha256: String,
    pub url: String,
}

/// The file `version` has for this platform, from a signed list.
pub fn release_file(envelope: &[u8], version: &Version) -> Result<ReleaseFile, CheckError> {
    let data = envelope::open(&trusted_keys()?, envelope, CONTEXT)?;
    let m = Manifest::parse(&data)?;
    let release = m
        .releases
        .get(&version.to_string())
        .ok_or(CheckError::NoFile)?;
    let a = platform::current()
        .and_then(|p| release.archives.get(p))
        .ok_or(CheckError::NoFile)?;
    Ok(ReleaseFile {
        version: version.clone(),
        url: platform::download_url(version, &a.file),
        file: a.file.clone(),
        sha256: a.sha256.clone(),
    })
}

/// The system package `version` has for `tool` on this computer, from a
/// signed list: its file and SHA-256. `None` when the list has none.
pub fn package_file(
    envelope: &[u8],
    version: &Version,
    tool: PackageTool,
) -> Result<Option<(String, String)>, CheckError> {
    let data = envelope::open(&trusted_keys()?, envelope, CONTEXT)?;
    Ok(Manifest::parse(&data)?
        .releases
        .get(&version.to_string())
        .and_then(|r| r.packages.get(&platform::package_key(tool)))
        .map(|a| (a.file.clone(), a.sha256.clone())))
}

/// The commands that download a release's package, check it against the
/// signed list's SHA-256 and install it, each to follow the one before with
/// `&&`. They run the same in bash, zsh and fish.
///
/// `file` and `sha256` are what the list holds: [`Manifest::parse`] takes only
/// a file named as the release's and a SHA-256 in hex, so nothing in them means
/// anything to a shell.
pub fn package_steps(
    version: &Version,
    tool: PackageTool,
    file: &str,
    sha256: &str,
) -> Vec<String> {
    let install = match tool {
        PackageTool::Apt => format!("sudo apt install ./{file}"),
        PackageTool::Dnf => format!("sudo dnf install ./{file}"),
        // Not signed: the SHA-256 is what vouches for it, and zypper stops without this.
        PackageTool::Zypper => format!("sudo zypper --no-gpg-checks install ./{file}"),
    };
    vec![
        "cd \"$(mktemp -d)\"".to_string(),
        format!("curl -fLO {}", platform::download_url(version, file)),
        format!("echo '{sha256}  {file}' | sha256sum -c"),
        install,
    ]
}

/// The day `version`'s line was first released, from a signed list: what a license
/// bought outright is held to.
pub fn line_released(envelope: &[u8], version: &Version) -> Result<Option<String>, CheckError> {
    let data = envelope::open(&trusted_keys()?, envelope, CONTEXT)?;
    Ok(Manifest::parse(&data)?.line_released(version))
}

/// What a signed list offers the app at `current`: None when it is the
/// newest of its channel.
pub fn offer(envelope: &[u8], current: &Version) -> Result<Option<Version>, CheckError> {
    let data = envelope::open(&trusted_keys()?, envelope, CONTEXT)?;
    Ok(Manifest::parse(&data)?.offer(current))
}

/// `work` on a runtime of its own, for the thread that waits for it.
fn blocking<T>(work: impl Future<Output = Result<T, CheckError>>) -> Result<T, CheckError> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| CheckError::Io(e.to_string()))?
        .block_on(work)
}

/// Reads the signed list from `url`, and waits for it.
pub fn fetch_blocking(url: &str) -> Result<Vec<u8>, CheckError> {
    blocking(fetch(url))
}

/// Downloads a release's file to `dest`, as [`download::download`], and waits for it.
pub fn download_blocking(
    file: &ReleaseFile,
    dest: &std::path::Path,
    progress: impl Fn(u64, Option<u64>),
) -> Result<(), CheckError> {
    if STORE_BUILD {
        return Err(CheckError::Io(STORE_UPDATES_IT.to_string()));
    }
    blocking(download::download(&file.url, &file.sha256, dest, progress))
}

/// Reads the signed list from `url`.
pub async fn fetch(url: &str) -> Result<Vec<u8>, CheckError> {
    let fail = |e: &dyn std::fmt::Display| CheckError::Fetch(e.to_string());
    // reqwest on rustls with ring, as the sync places use it. Once is
    // enough; again is harmless (the first one installed stays).
    let _ = rustls::crypto::ring::default_provider().install_default();
    let client = reqwest::Client::builder()
        .timeout(FETCH_TIMEOUT)
        .user_agent(concat!("packwing/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| fail(&e))?;
    let res = client.get(url).send().await.map_err(|e| fail(&e))?;
    if res.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(CheckError::NoList);
    }
    if !res.status().is_success() {
        return Err(CheckError::Fetch(format!(
            "the list of releases: {}",
            res.status()
        )));
    }
    if res.content_length().is_some_and(|n| n as usize > MOST_LIST) {
        return Err(CheckError::Fetch(
            "the list of releases is too large".into(),
        ));
    }
    let body = res.bytes().await.map_err(|e| fail(&e))?;
    if body.len() > MOST_LIST {
        return Err(CheckError::Fetch(
            "the list of releases is too large".into(),
        ));
    }
    Ok(body.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// shellwing's published list opens with these keys under its own context, and
    /// not as packwing's: the same keys, and neither product's list is the other's.
    #[test]
    fn the_keys_are_shellwings_and_its_list_is_not_ours() {
        let keys = envelope::parse_keys(KEYS).expect("the app's keys");
        let signed = include_bytes!("../testdata/shellwing.signed.json");
        envelope::open(&keys, signed, "shellwing-release-v1").expect("signed by these keys");
        assert!(envelope::open(&keys, signed, CONTEXT).is_err());
    }

    #[test]
    fn the_steps_name_the_release_and_its_hash() {
        let v = Version::parse("v0.2.1").expect("version");
        assert_eq!(
            package_steps(&v, PackageTool::Apt, "packwing_0.2.1_amd64.deb", "abc"),
            [
                "cd \"$(mktemp -d)\"",
                "curl -fLO https://github.com/hivepaas/packwing/releases/download/v0.2.1/packwing_0.2.1_amd64.deb",
                "echo 'abc  packwing_0.2.1_amd64.deb' | sha256sum -c",
                "sudo apt install ./packwing_0.2.1_amd64.deb",
            ]
        );
        let last = |tool| {
            package_steps(&v, tool, "x.rpm", "abc")
                .pop()
                .expect("a step")
        };
        assert_eq!(last(PackageTool::Dnf), "sudo dnf install ./x.rpm");
        assert_eq!(
            last(PackageTool::Zypper),
            "sudo zypper --no-gpg-checks install ./x.rpm"
        );
    }
}
