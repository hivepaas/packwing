//! The license this computer holds: the organization's, set by policy, or the one the
//! user entered. No license is Free, which does everything but keep more than one storage
//! across a network. It is read here, and nothing about it leaves this computer.

use std::path::PathBuf;
use std::time::SystemTime;

use gpui::{App, Global};
use packwing_license::{License, Plan};

/// Where the user's key is kept, beside the settings.
fn file() -> Option<PathBuf> {
    Some(crate::prefs::folder()?.join("license.key"))
}

/// The license held.
#[derive(Clone, Default)]
pub struct Licensed {
    pub license: Option<License>,
    /// Why a key that is there is not read.
    pub problem: Option<String>,
    /// The organization set it: the user can't change it here.
    pub by_policy: bool,
}

impl Global for Licensed {}

impl Licensed {
    /// What the policy says, or else what the user entered.
    pub fn load() -> Licensed {
        if let packwing_policy::Setting::Set(key) = &packwing_policy::current().license_key {
            let (license, problem) = match packwing_license::read(key) {
                Ok(license) => (Some(license), None),
                Err(why) => (None, Some(why)),
            };
            return Licensed {
                license,
                problem,
                by_policy: true,
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

/// What this computer's license gives today.
pub fn plan(cx: &App) -> Plan {
    let license = cx.try_global::<Licensed>().and_then(|l| l.license.as_ref());
    license.map_or(Plan::Free, |l| l.plan(SystemTime::now().into()))
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
    Ok(())
}

/// Forgets the user's key.
pub fn remove(cx: &mut App) {
    if let Some(file) = file() {
        let _ = std::fs::remove_file(file);
    }
    cx.set_global(Licensed::default());
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
