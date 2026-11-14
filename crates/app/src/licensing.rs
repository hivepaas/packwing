//! The license this computer holds: the organization's, set by policy, or the one the
//! user entered. No license is Free, which does everything but keep more than one storage
//! across a network. It is read here, and nothing about it leaves this computer.

use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use gpui::{App, AppContext as _, Global};
use packwing_license::{License, Plan};

/// Where the user's key is kept, beside the settings.
fn file() -> Option<PathBuf> {
    Some(crate::prefs::folder()?.join("license.key"))
}

/// Where the newer key of the organization's that the portal answered last is kept,
/// where its `LicenseRefresh` policy asks for one.
fn refreshed_file() -> Option<PathBuf> {
    Some(crate::prefs::folder()?.join("license.refreshed.key"))
}

/// When the portal was last asked, in seconds since 1970.
fn asked_file() -> Option<PathBuf> {
    Some(crate::prefs::folder()?.join("license.asked"))
}

/// The first ask waits for the app to settle; then it is looked at this often, and
/// asked once a day.
const FIRST_AFTER: Duration = Duration::from_secs(30);
const WAKE_EVERY: Duration = Duration::from_secs(60 * 60);
const ASK_EVERY: Duration = Duration::from_secs(24 * 60 * 60);

/// The license held.
#[derive(Clone, Default)]
pub struct Licensed {
    pub license: Option<License>,
    /// Why a key that is there is not read.
    pub problem: Option<String>,
    /// The organization set it: the user can't change it here.
    pub by_policy: bool,
    /// The organization's policy asks the portal for newer keys of it.
    pub refreshes: bool,
}

impl Global for Licensed {}

impl Licensed {
    /// What the policy says, or else what the user entered.
    pub fn load() -> Licensed {
        let policy = packwing_policy::current();
        if let packwing_policy::Setting::Set(key) = &policy.license_key {
            let refreshes = matches!(policy.license_refresh, packwing_policy::Setting::Set(true));
            let (license, problem) = match packwing_license::read(key) {
                Ok(license) => {
                    let refreshed = refreshes
                        .then(|| refreshed_file().and_then(|f| std::fs::read_to_string(f).ok()))
                        .flatten();
                    (
                        Some(packwing_license::newest(license, refreshed.as_deref())),
                        None,
                    )
                }
                Err(why) => (None, Some(why)),
            };
            return Licensed {
                license,
                problem,
                by_policy: true,
                refreshes,
            };
        }
        let text = file().and_then(|f| std::fs::read_to_string(f).ok());
        let Some(text) = text else {
            return Licensed::default();
        };
        match packwing_license::read(&text) {
            Ok(license) => Licensed {
                license: Some(license),
                ..Licensed::default()
            },
            Err(why) => Licensed {
                problem: Some(why),
                ..Licensed::default()
            },
        }
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// Asks the portal for a newer key of the organization's, once a day, where its
/// `LicenseRefresh` policy says to; and for nothing anywhere else.
pub fn start_refreshing(cx: &mut App) {
    if !cx.try_global::<Licensed>().is_some_and(|l| l.refreshes) {
        return;
    }
    cx.spawn(async move |cx| {
        cx.background_executor().timer(FIRST_AFTER).await;
        loop {
            let asked = asked_file()
                .and_then(|f| std::fs::read_to_string(f).ok())
                .and_then(|t| t.trim().parse::<u64>().ok());
            if asked.is_none_or(|t| now_secs().saturating_sub(t) >= ASK_EVERY.as_secs()) {
                let held = cx
                    .update(|cx| cx.try_global::<Licensed>().cloned())
                    .and_then(|l| l.license);
                if let Some(license) = held {
                    ask(license, cx).await;
                }
            }
            cx.background_executor().timer(WAKE_EVERY).await;
        }
    })
    .detach();
}

/// Asks the portal once for a newer key of `held`'s license, keeping one that is.
async fn ask(held: License, cx: &mut gpui::AsyncApp) {
    let Some(text) = key_text_of(&held) else {
        return;
    };
    let answer = cx
        .background_spawn(async move {
            packwing_update::license::refresh_blocking(packwing_update::license::PORTAL, &text)
        })
        .await;
    if let Some(f) = asked_file() {
        let _ = std::fs::write(f, now_secs().to_string());
    }
    let Ok(packwing_update::license::Answer::Newer(newer_text)) = answer else {
        return;
    };
    let Ok(newer) = packwing_license::read(&newer_text) else {
        return;
    };
    if !newer.replaces(&held) {
        return;
    }
    if let Some(file) = refreshed_file()
        && write_private(&file, newer_text.trim()).is_ok()
    {
        cx.update(|cx| {
            cx.set_global(Licensed::load());
            crate::audit_log::apply(cx, crate::audit_log::By::License);
        });
    }
}

/// The text of the key `held` was read from: the organization's, or the newer one kept.
fn key_text_of(held: &License) -> Option<String> {
    let refreshed = refreshed_file().and_then(|f| std::fs::read_to_string(f).ok());
    if let Some(text) = refreshed.filter(|t| packwing_license::read(t).as_ref() == Ok(held)) {
        return Some(text);
    }
    match &packwing_policy::current().license_key {
        packwing_policy::Setting::Set(key) => Some(key.clone()),
        _ => None,
    }
}

/// What this computer's license gives today: Pro at least, once bought in the Mac App
/// Store.
pub fn plan(cx: &App) -> Plan {
    let license = cx.try_global::<Licensed>().and_then(|l| l.license.as_ref());
    let plan = license.map_or(Plan::Free, |l| {
        l.plan(SystemTime::now().into(), packwing_license::line_released())
    });
    if plan == Plan::Free && crate::store::holds(cx) {
        return Plan::Pro;
    }
    plan
}

/// Takes `text` as the user's key, keeping it if it is one.
pub fn enter(text: &str, cx: &mut App) -> Result<(), String> {
    let license = packwing_license::read(text)?;
    let file =
        file().ok_or_else(|| packwing_i18n::tr!("This system has no folder for settings."))?;
    if let Some(folder) = file.parent() {
        std::fs::create_dir_all(folder).map_err(|e| e.to_string())?;
    }
    write_private(&file, text.trim()).map_err(|e| e.to_string())?;
    cx.set_global(Licensed {
        license: Some(license),
        ..Licensed::default()
    });
    crate::audit_log::apply(cx, crate::audit_log::By::License);
    Ok(())
}

/// Forgets the user's key.
pub fn remove(cx: &mut App) {
    if let Some(file) = file() {
        let _ = std::fs::remove_file(file);
    }
    cx.set_global(Licensed::default());
    crate::audit_log::apply(cx, crate::audit_log::By::License);
}

/// Writes the key for this user's eyes: it holds their name and email.
#[cfg(unix)]
fn write_private(file: &std::path::Path, text: &str) -> std::io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(file)?
        .write_all(text.as_bytes())
}

#[cfg(not(unix))]
fn write_private(file: &std::path::Path, text: &str) -> std::io::Result<()> {
    std::fs::write(file, text)
}
