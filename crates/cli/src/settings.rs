//! What the app keeps that pkw uses too, read from its settings file as its
//! `prefs.rs` writes it: the keys it trusts, the storages added to it, and how
//! many threads it takes. pkw never writes it.

use std::path::PathBuf;
use std::sync::Arc;

use packwing_core::{LocalFolder, PublicKey, Storage, StorageKind, Values, read_public_key};

/// The app's settings file, where `prefs.rs` keeps it.
fn file() -> Option<PathBuf> {
    let env = |name: &str| {
        std::env::var_os(name)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    };
    let folder = if cfg!(windows) {
        env("APPDATA")?
    } else if cfg!(target_os = "macos") {
        env("HOME")?.join("Library/Application Support")
    } else {
        env("XDG_CONFIG_HOME").or_else(|| Some(env("HOME")?.join(".config")))?
    };
    Some(folder.join("packwing").join("settings"))
}

/// What the user chose of the audit log in the app: `audit_log=true`, and
/// `audit_retention_days=N`.
pub fn audit(settings: &str) -> packwing_core::audit::Chosen {
    let mut chosen = packwing_core::audit::Chosen::default();
    for line in settings.lines() {
        if let Some(on) = line.strip_prefix("audit_log=") {
            chosen.on = on == "true";
        } else if let Some(days) = line.strip_prefix("audit_retention_days=") {
            chosen.retention_days = days.parse().unwrap_or(chosen.retention_days);
        }
    }
    chosen
}

/// The plan of the license this computer holds: the organization's, set by
/// policy, or the one entered in the app, kept beside its settings. Free without.
pub fn plan() -> packwing_license::Plan {
    let text = match &packwing_policy::current().license_key {
        packwing_policy::Setting::Set(key) => Some(key.clone()),
        _ => file().and_then(|f| std::fs::read_to_string(f.with_file_name("license.key")).ok()),
    };
    text.and_then(|t| packwing_license::read(&t).ok())
        .map_or(packwing_license::Plan::Free, |l| {
            l.plan(std::time::SystemTime::now().into())
        })
}

/// The settings' text, or none.
pub fn text() -> String {
    file()
        .and_then(|f| std::fs::read_to_string(f).ok())
        .unwrap_or_default()
}

/// A name or a value as the settings keep it, with what would end a line or a
/// field escaped.
fn unescaped(text: &str) -> String {
    text.replace("%0A", "\n")
        .replace("%09", "\t")
        .replace("%25", "%")
}

/// How many threads the app may take: `threads=N`, 0 for all, as it is
/// without the line.
pub fn threads(settings: &str) -> u32 {
    settings
        .lines()
        .filter_map(|l| l.strip_prefix("threads="))
        .find_map(|v| v.parse().ok())
        .unwrap_or(0)
}

/// The keys the app trusts, by the names it gives them: `trusted_key=NAME<TAB>LINE`.
pub fn trusted(settings: &str) -> Vec<(String, PublicKey)> {
    settings
        .lines()
        .filter_map(|l| l.strip_prefix("trusted_key="))
        .filter_map(|v| v.split_once('\t'))
        .filter_map(|(name, line)| Some((unescaped(name), read_public_key(line).ok()?)))
        .collect()
}

/// A storage added to the app.
pub struct Kept {
    pub name: String,
    /// What kind, as the user calls it, and where it is.
    pub kind: String,
    pub describe: String,
    /// It asks for this computer's password before it opens, in the app.
    pub asks_first: bool,
    /// None when this build has no kind for it.
    pub storage: Option<Arc<dyn Storage>>,
}

/// Why a storage of a kind this build has none for is not opened: said after
/// "pkw cannot open".
pub const WITHOUT: &str = if cfg!(feature = "remote") {
    ""
} else {
    ": this pkw is built without S3, SFTP and WebDAV (the remote feature)"
};

/// The kinds of storage this build opens, beside a folder here: none
/// without the `remote` feature.
pub fn kinds() -> Vec<Arc<dyn StorageKind>> {
    #[cfg(feature = "remote")]
    return packwing_remote::kinds();
    #[cfg(not(feature = "remote"))]
    return Vec::new();
}

/// The storages added to the app, in its order: `storage=local NAME<TAB>PATH`,
/// and `storage=other KIND<TAB>NAME<TAB>ID<TAB>K=V…`.
pub fn storages(settings: &str, kinds: &[Arc<dyn StorageKind>]) -> Vec<Kept> {
    let mut kept = Vec::new();
    for value in settings.lines().filter_map(|l| l.strip_prefix("storage=")) {
        if let Some(rest) = value.strip_prefix("local ")
            && let Some((name, path)) = rest.split_once('\t')
        {
            kept.push(Kept {
                name: name.to_owned(),
                kind: "Folder".into(),
                describe: path.to_owned(),
                asks_first: false,
                storage: Some(Arc::new(LocalFolder::new(path))),
            });
        } else if let Some(rest) = value.strip_prefix("other ") {
            let mut parts = rest.split('\t');
            let (Some(kind), Some(name), Some(id)) = (parts.next(), parts.next(), parts.next())
            else {
                continue;
            };
            let values: Values = parts
                .filter_map(|v| v.split_once('='))
                .map(|(k, v)| (unescaped(k), unescaped(v)))
                .collect();
            let known = kinds.iter().find(|k| k.id() == kind);
            kept.push(Kept {
                name: unescaped(name),
                kind: known.map_or_else(|| kind.to_owned(), |k| k.name()),
                describe: known.map(|k| k.describe(&values)).unwrap_or_default(),
                asks_first: values.contains_key("asks_first"),
                storage: known.map(|k| k.open(id, &values, None)),
            });
        }
    }
    kept
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_app_s_storages_and_keys_are_read_from_its_settings() {
        let dir = std::env::temp_dir().join(format!("pkw-settings-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a folder");
        let (key, _) = packwing_core::make_signing_key(&dir, "khoá".as_bytes()).expect("a key");
        let settings = format!(
            "language=vi\n\
             trusted_key=Máy%09ở nhà\t{}\n\
             trusted_key=broken\tnot a key\n\
             storage=local Tải về\t/home/u/Downloads\n\
             storage=other s3\tSao lưu\ts3-1\tbucket=b\tasks_first=yes\n\
             storage=other gone\tCũ\tg-1\n",
            key.public()
        );
        let found = trusted(&settings);
        assert_eq!(found.len(), 1, "the broken one is left out");
        assert_eq!(found[0].0, "Máy\tở nhà");
        assert_eq!(found[0].1.fingerprint(), key.public().fingerprint());
        let kept = storages(&settings, &[]);
        let names: Vec<&str> = kept.iter().map(|k| k.name.as_str()).collect();
        assert_eq!(names, ["Tải về", "Sao lưu", "Cũ"]);
        assert!(kept[0].storage.is_some(), "a folder here needs no kind");
        assert!(kept[1].asks_first);
        assert!(kept[2].storage.is_none(), "no kind for it in this build");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn the_app_s_threads_are_read_and_none_is_all() {
        assert_eq!(threads("language=vi\nthreads=4\n"), 4);
        assert_eq!(threads("language=vi\n"), 0);
        assert_eq!(threads("threads=many\n"), 0);
    }
}
