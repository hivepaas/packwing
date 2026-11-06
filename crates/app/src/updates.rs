//! Telling that a newer packwing is out, and installing it (see `packwing_update`):
//! a look once a day while it is on, or when asked in Settings › About. What is offered
//! shows as a badge on the title bar, which leads to About's Updates; it never takes
//! the keyboard.
//!
//! A copy the app can replace downloads the release when told to, checks it against the
//! signed list, and installs it as it quits: at *Restart now*, or whenever the user
//! quits. A copy from the release's `.deb` is given the commands that install the next
//! one. Any other copy opens the release's page, or says what updates it.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use gpui::{App, AppContext as _, Context, Task};
use packwing_i18n::{tr, tr_n};
use packwing_update::{CheckError, Install, Version};

use crate::prefs;

/// How often a look is due, and how often whether one is due is asked.
const CHECK_EVERY: Duration = Duration::from_secs(24 * 60 * 60);
const WAKE_EVERY: Duration = Duration::from_secs(60 * 60);
/// The first look waits for the app to settle.
const FIRST_AFTER: Duration = Duration::from_secs(20);
/// How often a download's progress is shown.
const PROGRESS_EVERY: Duration = Duration::from_millis(250);

/// What is remembered between runs, with the preferences.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpdateState {
    /// Whether to look once a day.
    pub check: bool,
    /// When the last look succeeded, in seconds since 1970.
    pub checked_at: Option<u64>,
    /// The release the last look found, if it found a newer one.
    pub offered: Option<String>,
    /// The release not to offer again.
    pub skipped: Option<String>,
    /// The offered release's `.deb` or `.rpm`: its file and SHA-256.
    pub package: Option<(String, String)>,
    /// A release downloaded and checked: its version, and where it is.
    pub staged: Option<(String, PathBuf)>,
}

impl Default for UpdateState {
    fn default() -> UpdateState {
        UpdateState {
            check: true,
            checked_at: None,
            offered: None,
            skipped: None,
            package: None,
            staged: None,
        }
    }
}

/// Getting a release in.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Stage {
    #[default]
    Idle,
    /// Bytes so far, and of how many when known.
    Downloading(u64, Option<u64>),
    /// Downloaded and checked: installed as the app quits.
    Ready(Version),
    Failed(String),
}

/// What the title bar's badge says.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Badge {
    Out(Version),
    Ready(Version),
}

/// How looking for updates stands.
pub struct Updates {
    state: UpdateState,
    checking: bool,
    /// What went wrong the last time, if it did.
    problem: Option<String>,
    stage: Stage,
    /// Quitting to install the update starts the app again.
    relaunch: bool,
    _looks: Option<Task<()>>,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Where a release's file is downloaded.
fn staging_dir() -> Option<PathBuf> {
    dirs::cache_dir().map(|d| d.join("packwing").join("updates"))
}

fn install() -> Install {
    #[cfg(all(test, feature = "test-hooks"))]
    if let Some(how) = packwing_update::test_install() {
        return how;
    }
    let exe = std::env::current_exe().unwrap_or_default();
    packwing_update::install_of(&exe, |k| std::env::var(k).ok())
}

/// The release a list offers, and its package's file and SHA-256.
type Found = (Option<Version>, Option<(String, String)>);

/// What a signed list offers this copy, and that release's package for it.
fn found_in(list: &[u8]) -> Result<Found, CheckError> {
    let offered = packwing_update::offer(list, &packwing_update::current())?;
    let package = match (&offered, install()) {
        (Some(v), Install::Package(tool)) => packwing_update::package_file(list, v, tool)?,
        _ => None,
    };
    Ok((offered, package))
}

/// What updates this copy, in the language in use: names stay as they are, the others
/// are said.
pub fn by_said(by: &str) -> String {
    match by {
        "the system's package manager" => tr!("the system's package manager"),
        "the App Store" => tr!("the App Store"),
        "the Microsoft Store" => tr!("the Microsoft Store"),
        name => name.to_owned(),
    }
}

/// "12.3 MB".
fn megabytes(n: u64) -> String {
    format!("{:.1} MB", n as f64 / 1_000_000.)
}

/// Said as a download goes: "12.3 MB of 45.6 MB".
pub fn download_said(got: u64, of: Option<u64>) -> String {
    match of {
        Some(of) => tr!(
            "Downloading… {got} of {total}",
            got = megabytes(got),
            total = megabytes(of)
        ),
        None => tr!("Downloading… {got}", got = megabytes(got)),
    }
}

/// How long ago `then` was, in seconds since 1970, at `now`.
pub fn ago(then: u64, now: u64) -> String {
    let minutes = now.saturating_sub(then) / 60;
    match minutes {
        0 => tr!("just now"),
        1..=59 => tr_n!(minutes, "{n} minute ago", "{n} minutes ago"),
        60..=1439 => tr_n!(minutes / 60, "{n} hour ago", "{n} hours ago"),
        _ => tr_n!(minutes / 1440, "{n} day ago", "{n} days ago"),
    }
}

impl Updates {
    pub fn new(cx: &mut Context<Self>) -> Updates {
        let mut updates = Updates {
            state: prefs::prefs(cx).update,
            checking: false,
            problem: None,
            stage: Stage::Idle,
            relaunch: false,
            _looks: None,
        };
        updates.restore_staged(cx);
        updates
    }

    /// The looks to come, and the update to install as the app quits.
    pub fn start(&mut self, cx: &mut Context<Self>) {
        cx.on_app_quit(|this, cx| {
            this.install_staged(cx);
            std::future::ready(())
        })
        .detach();
        // A build without release keys could believe no list: it does not look.
        if !packwing_update::has_keys() {
            return;
        }
        self._looks = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(FIRST_AFTER).await;
            loop {
                let went_on = this.update(cx, |this, cx| {
                    let due = this
                        .state
                        .checked_at
                        .is_none_or(|t| now_secs().saturating_sub(t) >= CHECK_EVERY.as_secs());
                    if this.state.check && due {
                        this.check(cx);
                    }
                });
                if went_on.is_err() {
                    break;
                }
                cx.background_executor().timer(WAKE_EVERY).await;
            }
        }));
    }

    fn keep(&self, cx: &mut App) {
        let state = self.state.clone();
        prefs::change(cx, |p| p.update = state);
    }

    /// A release downloaded by an earlier run: ready still, unless what runs is it (or
    /// newer), whose file then goes.
    fn restore_staged(&mut self, cx: &mut App) {
        let Some((version, path)) = self.state.staged.clone() else {
            return;
        };
        match Version::parse(&version) {
            Some(v) if v > packwing_update::current() && path.exists() => {
                self.stage = Stage::Ready(v);
            }
            _ => {
                let _ = std::fs::remove_file(&path);
                self.state.staged = None;
                self.keep(cx);
            }
        }
    }

    /// Whether this build can believe a list at all.
    pub fn has_keys(&self) -> bool {
        packwing_update::has_keys()
    }

    pub fn checking(&self) -> bool {
        self.checking
    }

    pub fn checks(&self) -> bool {
        self.state.check
    }

    pub fn checked_at(&self) -> Option<u64> {
        self.state.checked_at
    }

    pub fn problem(&self) -> Option<&str> {
        self.problem.as_deref()
    }

    pub fn stage(&self) -> &Stage {
        &self.stage
    }

    /// How this copy was installed.
    pub fn install(&self) -> Install {
        install()
    }

    /// Turns the daily look on or off.
    pub fn set_checks(&mut self, on: bool, cx: &mut Context<Self>) {
        self.state.check = on;
        self.keep(cx);
        cx.notify();
    }

    /// Reads the signed list now, and says what it offers.
    pub fn check(&mut self, cx: &mut Context<Self>) {
        if self.checking || !packwing_update::has_keys() {
            return;
        }
        self.checking = true;
        cx.notify();
        let url = packwing_update::list_url();
        let work = cx.background_spawn(async move {
            match packwing_update::fetch_blocking(&url) {
                Ok(list) => found_in(&list).map_err(|e| e.to_string()),
                // Before the first release: up to date.
                Err(CheckError::NoList) => Ok((None, None)),
                Err(e) => Err(e.to_string()),
            }
        });
        cx.spawn(async move |this, cx| {
            let found = work.await;
            let _ = this.update(cx, |this, cx| {
                this.checking = false;
                match found {
                    Ok((offered, package)) => {
                        this.problem = None;
                        this.state.checked_at = Some(now_secs());
                        this.state.offered = offered.map(|v| v.to_string());
                        this.state.package = package;
                    }
                    Err(e) => this.problem = Some(e),
                }
                this.keep(cx);
                cx.notify();
            });
        })
        .detach();
    }

    /// The release offered, unless skipped or already running.
    pub fn offered(&self) -> Option<Version> {
        let s = &self.state;
        let v = Version::parse(s.offered.as_deref()?)?;
        (v > packwing_update::current() && s.skipped.as_deref() != s.offered.as_deref())
            .then_some(v)
    }

    /// What the title bar says, if anything: a release that is ready, or, while looking
    /// for updates is on, one that is out. Nothing while one downloads: About shows
    /// that, and it was asked for.
    pub fn badge(&self) -> Option<Badge> {
        match &self.stage {
            Stage::Ready(v) => Some(Badge::Ready(v.clone())),
            Stage::Downloading(..) => None,
            _ if self.state.check => self.offered().map(Badge::Out),
            _ => None,
        }
    }

    /// The offered release: downloaded to install, when this copy can take it; else its
    /// page.
    pub fn get(&mut self, cx: &mut Context<Self>) {
        let Some(v) = self.offered() else {
            return;
        };
        if install().updates_itself() {
            self.download(v, cx);
        } else {
            cx.open_url(&packwing_update::release_page(&v));
        }
    }

    /// The commands that install the offered release's package, when this copy came from
    /// one and the list names it.
    pub fn steps(&self) -> Option<Vec<String>> {
        let Install::Package(tool) = install() else {
            return None;
        };
        let v = self.offered()?;
        let (file, sha256) = self.state.package.as_ref()?;
        Some(packwing_update::package_steps(&v, tool, file, sha256))
    }

    /// Downloads `v`'s file for this platform, as the signed list (read again, and
    /// checked again) names it.
    fn download(&mut self, v: Version, cx: &mut Context<Self>) {
        if matches!(self.stage, Stage::Downloading(..) | Stage::Ready(_)) {
            return;
        }
        let Some(dir) = staging_dir() else {
            self.stage = Stage::Failed(tr!("No folder to download to."));
            return cx.notify();
        };
        let (got, total) = (Arc::new(AtomicU64::new(0)), Arc::new(AtomicU64::new(0)));
        let (job_got, job_total, wanted) = (got.clone(), total.clone(), v.clone());
        let work = cx.background_spawn(async move {
            let list = packwing_update::fetch_blocking(&packwing_update::list_url())?;
            let file = packwing_update::release_file(&list, &wanted)?;
            let dest = dir.join(&file.file);
            packwing_update::download_blocking(&file, &dest, |n, of| {
                job_got.store(n, Ordering::Relaxed);
                job_total.store(of.unwrap_or(0), Ordering::Relaxed);
            })?;
            Ok::<PathBuf, CheckError>(dest)
        });
        self.stage = Stage::Downloading(0, None);
        cx.notify();
        // The progress, shown as it comes, until the download ends.
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(PROGRESS_EVERY).await;
                let (n, of) = (got.load(Ordering::Relaxed), total.load(Ordering::Relaxed));
                let going = this.update(cx, |this, cx| {
                    let going = matches!(this.stage, Stage::Downloading(..));
                    if going {
                        this.stage = Stage::Downloading(n, (of > 0).then_some(of));
                        cx.notify();
                    }
                    going
                });
                if !going.unwrap_or(false) {
                    break;
                }
            }
        })
        .detach();
        cx.spawn(async move |this, cx| {
            let done = work.await;
            let _ = this.update(cx, |this, cx| {
                match done {
                    Ok(path) => {
                        this.state.staged = Some((v.to_string(), path));
                        this.keep(cx);
                        this.stage = Stage::Ready(v);
                    }
                    Err(e) => this.stage = Stage::Failed(e.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// Quits to install the downloaded release, and starts it.
    pub fn restart(&mut self, cx: &mut Context<Self>) {
        if !matches!(self.stage, Stage::Ready(_)) {
            return;
        }
        self.relaunch = true;
        cx.quit();
    }

    /// As the app quits: the downloaded release in place of this copy, and started again
    /// after a Restart.
    fn install_staged(&mut self, cx: &mut App) {
        let Stage::Ready(v) = &self.stage else {
            return;
        };
        let Some((_, file)) = self.state.staged.clone() else {
            return;
        };
        let how = install();
        match packwing_update::apply::apply(&how, &file, self.relaunch) {
            Ok(()) => {
                // The installer on Windows runs once this app has gone: its file stays
                // until the next run sees the new version.
                if how != Install::WindowsInstaller {
                    let _ = std::fs::remove_file(&file);
                    self.state.staged = None;
                }
            }
            Err(e) => {
                eprintln!("packwing: cannot install {v}: {e}");
                // Not tried again on every quit: the next look offers it anew.
                let _ = std::fs::remove_file(&file);
                self.state.staged = None;
            }
        }
        self.keep(cx);
    }

    /// Not offered again until a newer one comes.
    pub fn skip(&mut self, cx: &mut Context<Self>) {
        self.state.skipped = self.state.offered.clone();
        self.keep(cx);
        cx.notify();
    }

    /// As if the offered release had been downloaded, for the pictures.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn ready_for_test(&mut self, cx: &mut Context<Self>) {
        self.stage = self.offered().map_or(Stage::Idle, Stage::Ready);
        cx.notify();
    }

    /// As if a look had just offered `version` with `package`, for the pictures.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn offer_for_test(
        &mut self,
        version: &str,
        package: Option<(String, String)>,
        cx: &mut Context<Self>,
    ) {
        self.state.offered = Some(version.to_owned());
        self.state.package = package;
        cx.notify();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn how_long_ago_is_said_in_the_largest_unit() {
        assert_eq!(ago(1_000, 1_030), "just now");
        assert_eq!(ago(1_000, 1_060), "1 minute ago");
        assert_eq!(ago(0, 2 * 3_600), "2 hours ago");
        assert_eq!(ago(0, 3 * 86_400), "3 days ago");
        // A clock that went back is not an error.
        assert_eq!(ago(2_000, 1_000), "just now");
    }

    #[test]
    fn a_download_is_said_with_its_size_when_known() {
        assert_eq!(
            download_said(12_300_000, Some(45_600_000)),
            "Downloading… 12.3 MB of 45.6 MB"
        );
        assert_eq!(download_said(1_000_000, None), "Downloading… 1.0 MB");
    }
}
