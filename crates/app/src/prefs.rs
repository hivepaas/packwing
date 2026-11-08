//! What the user chose, kept between runs: a file of `key=value` lines in the
//! system's folder for settings, and the archives opened lately. Secrets never
//! go in it. A line it does not know, or cannot read, is left at its default,
//! so that a file from another version still opens.

use crate::updates::UpdateState;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use gpui::{App, Global};
use packwing_core::{
    Algorithm, COMPRESSED_EXTENSIONS, ExtractOptions, Level, OnLinksOutside, Owners, PublicKey,
    Values, read_public_key,
};

use crate::dialogs::Format;
use crate::pages::settings::Appearance;
use crate::placement::WindowPlace;

/// How many archives Recent remembers.
const RECENT: usize = 30;

/// The most an archive may unpack to, as Settings offers it: none for no limit.
pub const LIMITS: [(&str, Option<u64>); 4] = [
    ("10 GB", Some(10_000_000_000)),
    ("100 GB", Some(100_000_000_000)),
    ("1 TB", Some(1_000_000_000_000)),
    ("No limit", None),
];

/// An archive opened before.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Opened {
    pub path: PathBuf,
    /// When, in seconds since 1970.
    pub at: i64,
}

/// Where a storage in the list is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeptPlace {
    /// A folder on this computer.
    Local(PathBuf),
    /// One of a kind this build adds, its secrets kept under `id`.
    Other {
        kind: String,
        id: String,
        values: Values,
    },
}

/// A storage in the list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeptStorage {
    pub name: String,
    pub place: KeptPlace,
}

/// A key whose signatures the user trusts: what they call it, and its public key's
/// line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrustedKey {
    pub name: String,
    pub line: String,
}

/// The setting of a storage that asks that it is the user before it opens.
/// With its kind's settings in the file, where a file without it means no.
pub const ASKS_FIRST: &str = "asks_first";

impl KeptStorage {
    pub fn asks_first(&self) -> bool {
        matches!(&self.place, KeptPlace::Other { values, .. } if values.contains_key(ASKS_FIRST))
    }
}

/// `text` on one line of the file, with no tab in it.
fn escaped(text: &str) -> String {
    text.replace('%', "%25")
        .replace('\t', "%09")
        .replace('\n', "%0A")
}

fn unescaped(text: &str) -> String {
    text.replace("%0A", "\n")
        .replace("%09", "\t")
        .replace("%25", "%")
}

/// Where Extract starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExtractTo {
    /// Right here if the archive holds one thing, else a new folder.
    Smart,
    /// A new folder beside the archive, named after it.
    NewFolder,
    /// The archive's own folder.
    Here,
}

impl ExtractTo {
    fn name(self) -> &'static str {
        match self {
            ExtractTo::Smart => "smart",
            ExtractTo::NewFolder => "new-folder",
            ExtractTo::Here => "here",
        }
    }

    fn named(name: &str) -> Option<ExtractTo> {
        [ExtractTo::Smart, ExtractTo::NewFolder, ExtractTo::Here]
            .into_iter()
            .find(|to| to.name() == name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prefs {
    /// Where they are kept: none while they are only for this run, as in tests.
    file: Option<PathBuf>,
    /// A language's tag, or none for the system's.
    pub language: Option<String>,
    /// Body text's size in px, or none for the system's (`text_size`).
    pub text_size: Option<u8>,
    /// How the window was last left; none before it has been.
    pub window: Option<WindowPlace>,
    pub appearance: Appearance,
    pub format: Format,
    pub level: Level,
    pub hide_names: bool,
    pub skip_mac_files: bool,
    /// ZIP stores what is compressed already as it is: see
    /// [`packwing_core::NewArchive::stored_extensions`].
    pub store_compressed: bool,
    /// The extensions it knows that by, if changed from
    /// [`COMPRESSED_EXTENSIONS`]: none follows the list as it grows.
    pub stored_extensions: Option<Vec<String>>,
    /// In bytes: files that come to more go straight to a storage that can take
    /// them that way, and the rest through a temporary file.
    pub stream_above: u64,
    /// Where Extract starts.
    pub extract_to: ExtractTo,
    pub overwrite: bool,
    /// An index in [`LIMITS`].
    pub limit: usize,
    pub keep_mark: bool,
    /// Extracts only names every system can use; otherwise those this one can.
    pub portable_names: bool,
    /// Whose the files extracted are, where a tar archive kept their owners for
    /// a backup: the user's (the default), or the archive's by name or by number.
    pub restore_owners: Owners,
    /// Gives the files extracted the extended attributes a tar archive kept.
    pub restore_xattrs: bool,
    /// Asks before a file in an archive is opened. A program always asks.
    pub ask_before_open: bool,
    /// The latest first.
    pub recent: Vec<Opened>,
    /// The storages; before the user changes them, `storages_kept` is off and
    /// the page starts with the home folder.
    pub storages: Vec<KeptStorage>,
    pub storages_kept: bool,
    pub show_hidden: bool,
    /// The folder Browse showed last.
    pub browse_folder: Option<PathBuf>,
    /// The command that starts the text editor, or none for the system's.
    pub editor: Option<String>,
    /// Whether to look for updates, and what the last look found.
    pub update: UpdateState,
    /// The keys whose signatures are taken as proof.
    pub trusted_keys: Vec<TrustedKey>,
    /// The key file signed with last. Its contents never go in this file.
    pub signing_key: Option<PathBuf>,
    /// The checksums Checksums shows, in [`Algorithm::ALL`]'s order; never none.
    pub checksums: Vec<Algorithm>,
    /// How many threads compressing and decompressing may take: 0 for as many
    /// as the computer has. pkw reads it too.
    pub threads: u32,
}

impl Global for Prefs {}

impl Default for Prefs {
    fn default() -> Prefs {
        Prefs {
            file: None,
            language: None,
            text_size: None,
            window: None,
            appearance: Appearance::System,
            format: Format::Zip,
            level: Level::Medium,
            hide_names: true,
            skip_mac_files: true,
            store_compressed: true,
            stored_extensions: None,
            stream_above: 500_000_000,
            extract_to: ExtractTo::Smart,
            overwrite: false,
            limit: 1,
            keep_mark: true,
            portable_names: false,
            restore_owners: Owners::Ignore,
            restore_xattrs: false,
            ask_before_open: true,
            recent: Vec::new(),
            storages: Vec::new(),
            storages_kept: false,
            show_hidden: false,
            browse_folder: None,
            editor: None,
            update: UpdateState::default(),
            trusted_keys: Vec::new(),
            signing_key: None,
            checksums: vec![Algorithm::Sha256, Algorithm::Sha1],
            threads: 0,
        }
    }
}

/// packwing's folder in the system's folder for settings.
pub fn folder() -> Option<PathBuf> {
    default_file()?.parent().map(PathBuf::from)
}

/// The system's folder for a program's settings, and packwing's file in it.
fn default_file() -> Option<PathBuf> {
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

impl Prefs {
    /// What was kept, or the defaults; kept in the system's folder from now on.
    pub fn load() -> Prefs {
        let file = default_file();
        let text = file.as_ref().and_then(|f| fs::read_to_string(f).ok());
        let mut prefs = text.as_deref().map(Prefs::parse).unwrap_or_default();
        prefs.file = file;
        prefs
    }

    /// The preferences in `text`.
    fn parse(text: &str) -> Prefs {
        let mut p = Prefs::default();
        let on = |value: &str, default: bool| match value {
            "true" => true,
            "false" => false,
            _ => default,
        };
        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            match key {
                "language" => p.language = Some(value.to_owned()).filter(|v| !v.is_empty()),
                "text_size" => {
                    p.text_size = value
                        .parse()
                        .ok()
                        .filter(|s| crate::text_size::CHOICES.contains(s));
                }
                "window" => p.window = WindowPlace::parse(value),
                "appearance" => {
                    p.appearance = match value {
                        "light" => Appearance::Light,
                        "dark" => Appearance::Dark,
                        _ => Appearance::System,
                    }
                }
                "format" => {
                    if let Some(f) = Format::ALL.into_iter().find(|f| f.ext() == value) {
                        p.format = f;
                    }
                }
                "level" => {
                    if let Some((l, _)) = crate::dialogs::LEVELS
                        .iter()
                        .find(|(_, name)| *name == value)
                    {
                        p.level = *l;
                    }
                }
                "hide_names" => p.hide_names = on(value, p.hide_names),
                "skip_mac_files" => p.skip_mac_files = on(value, p.skip_mac_files),
                "store_compressed" => p.store_compressed = on(value, p.store_compressed),
                "stored_extensions" => p.stored_extensions = Some(extensions_in(value)),
                "stream_above" => p.stream_above = value.parse().unwrap_or(p.stream_above),
                "extract_to" => p.extract_to = ExtractTo::named(value).unwrap_or(p.extract_to),
                // What it was before there was Smart: a new folder was the
                // default then, as Smart is now.
                "new_folder" if value == "false" => p.extract_to = ExtractTo::Here,
                "overwrite" => p.overwrite = on(value, p.overwrite),
                "keep_mark" => p.keep_mark = on(value, p.keep_mark),
                "portable_names" => p.portable_names = on(value, p.portable_names),
                "restore_owners" => {
                    p.restore_owners = match value {
                        "name" => Owners::ByName,
                        "number" => Owners::ByNumber,
                        _ => Owners::Ignore,
                    }
                }
                "restore_xattrs" => p.restore_xattrs = on(value, p.restore_xattrs),
                "ask_before_open" => p.ask_before_open = on(value, p.ask_before_open),
                "threads" => p.threads = value.parse().unwrap_or(p.threads),
                "limit" => {
                    if let Some(i) = LIMITS.iter().position(|(name, _)| *name == value) {
                        p.limit = i;
                    }
                }
                // A path may hold anything but a line's end, which is refused
                // when it is written.
                "recent" => {
                    if let Some((at, path)) = value.split_once(' ')
                        && let Ok(at) = at.parse()
                        && p.recent.len() < RECENT
                    {
                        p.recent.push(Opened {
                            path: PathBuf::from(path),
                            at,
                        });
                    }
                }
                "storages_kept" => p.storages_kept = on(value, p.storages_kept),
                "show_hidden" => p.show_hidden = on(value, p.show_hidden),
                "browse_folder" => p.browse_folder = Some(PathBuf::from(value)),
                "editor" => p.editor = Some(value.to_owned()).filter(|v| !v.trim().is_empty()),
                "update_check" => p.update.check = on(value, p.update.check),
                "update_checked_at" => p.update.checked_at = value.parse().ok(),
                "update_offered" => p.update.offered = Some(value.to_owned()),
                "update_skipped" => p.update.skipped = Some(value.to_owned()),
                "update_package" => {
                    p.update.package = value
                        .split_once('\t')
                        .map(|(file, sha)| (file.to_owned(), sha.to_owned()));
                }
                "update_staged" => {
                    p.update.staged = value
                        .split_once('\t')
                        .map(|(version, path)| (version.to_owned(), PathBuf::from(path)));
                }
                "signing_key" => p.signing_key = Some(PathBuf::from(value)),
                "checksums" => {
                    let named: Vec<_> = value.split(' ').filter_map(Algorithm::keyed).collect();
                    if !named.is_empty() {
                        p.checksums = named;
                    }
                }
                // What it is called, a tab, and its public key's line.
                "trusted_key" => {
                    if let Some((name, line)) = value.split_once('\t') {
                        p.trusted_keys.push(TrustedKey {
                            name: unescaped(name),
                            line: line.to_owned(),
                        });
                    }
                }
                // A local folder: its name, a tab, and its path. Another kind: its
                // kind, then tabs between its name, its id and its settings.
                "storage" => {
                    if let Some(rest) = value.strip_prefix("local ")
                        && let Some((name, path)) = rest.split_once('\t')
                    {
                        p.storages.push(KeptStorage {
                            name: name.to_owned(),
                            place: KeptPlace::Local(PathBuf::from(path)),
                        });
                    } else if let Some(rest) = value.strip_prefix("other ") {
                        let mut parts = rest.split('\t');
                        if let (Some(kind), Some(name), Some(id)) =
                            (parts.next(), parts.next(), parts.next())
                        {
                            let values = parts
                                .filter_map(|v| v.split_once('='))
                                .map(|(k, v)| (unescaped(k), unescaped(v)))
                                .collect();
                            p.storages.push(KeptStorage {
                                name: unescaped(name),
                                place: KeptPlace::Other {
                                    kind: kind.to_owned(),
                                    id: id.to_owned(),
                                    values,
                                },
                            });
                        }
                    }
                }
                _ => {}
            }
        }
        p
    }

    fn text(&self) -> String {
        let appearance = match self.appearance {
            Appearance::Light => "light",
            Appearance::Dark => "dark",
            Appearance::System => "system",
        };
        let level = crate::dialogs::LEVELS
            .iter()
            .find(|(l, _)| *l == self.level)
            .map_or("Medium", |(_, name)| name);
        let limit = LIMITS.get(self.limit).map_or(LIMITS[1].0, |(name, _)| name);
        let mut text = format!(
            "language={}\n",
            self.language.as_deref().unwrap_or_default()
        );
        if let Some(size) = self.text_size {
            text += &format!("text_size={size}\n");
        }
        if let Some(window) = self.window {
            text += &format!("window={}\n", window.to_text());
        }
        text += &format!(
            "appearance={appearance}\nformat={}\nlevel={level}\nhide_names={}\n\
             skip_mac_files={}\nstore_compressed={}\nstream_above={}\nextract_to={}\noverwrite={}\nlimit={limit}\nkeep_mark={}\n\
             portable_names={}\nrestore_owners={}\nrestore_xattrs={}\nask_before_open={}\nthreads={}\n",
            self.format.ext(),
            self.hide_names,
            self.skip_mac_files,
            self.store_compressed,
            self.stream_above,
            self.extract_to.name(),
            self.overwrite,
            self.keep_mark,
            self.portable_names,
            match self.restore_owners {
                Owners::ByName => "name",
                Owners::ByNumber => "number",
                _ => "none",
            },
            self.restore_xattrs,
            self.ask_before_open,
            self.threads,
        );
        if let Some(extensions) = &self.stored_extensions {
            text += &format!("stored_extensions={}\n", extensions.join(" "));
        }
        for opened in &self.recent {
            if let Some(path) = opened.path.to_str().filter(|p| !p.contains('\n')) {
                text.push_str(&format!("recent={} {path}\n", opened.at));
            }
        }
        text.push_str(&format!(
            "storages_kept={}\nshow_hidden={}\neditor={}\n",
            self.storages_kept,
            self.show_hidden,
            self.editor
                .as_deref()
                .unwrap_or_default()
                .replace('\n', " "),
        ));
        if let Some(folder) = self
            .browse_folder
            .as_ref()
            .and_then(|f| f.to_str())
            .filter(|f| !f.contains('\n'))
        {
            text.push_str(&format!("browse_folder={folder}\n"));
        }
        let u = &self.update;
        text.push_str(&format!("update_check={}\n", u.check));
        if let Some(at) = u.checked_at {
            text.push_str(&format!("update_checked_at={at}\n"));
        }
        if let Some(offered) = &u.offered {
            text.push_str(&format!("update_offered={offered}\n"));
        }
        if let Some(skipped) = &u.skipped {
            text.push_str(&format!("update_skipped={skipped}\n"));
        }
        if let Some((file, sha)) = &u.package {
            text.push_str(&format!("update_package={file}\t{sha}\n"));
        }
        if let Some((version, path)) = &u.staged
            && let Some(path) = path.to_str().filter(|p| !p.contains('\n'))
        {
            text.push_str(&format!("update_staged={version}\t{path}\n"));
        }
        if let Some(key) = self
            .signing_key
            .as_ref()
            .and_then(|f| f.to_str())
            .filter(|f| !f.contains('\n'))
        {
            text.push_str(&format!("signing_key={key}\n"));
        }
        let checksums: Vec<_> = self.checksums.iter().map(|a| a.key()).collect();
        text.push_str(&format!("checksums={}\n", checksums.join(" ")));
        for key in &self.trusted_keys {
            text.push_str(&format!(
                "trusted_key={}\t{}\n",
                escaped(&key.name),
                key.line
            ));
        }
        for s in &self.storages {
            let name = s.name.replace(['\n', '\t'], " ");
            match &s.place {
                KeptPlace::Local(path) => {
                    if let Some(path) = path.to_str().filter(|p| !p.contains('\n')) {
                        text.push_str(&format!("storage=local {name}\t{path}\n"));
                    }
                }
                KeptPlace::Other { kind, id, values } => {
                    text.push_str(&format!("storage=other {kind}\t{}\t{id}", escaped(&name)));
                    for (k, v) in values {
                        text.push_str(&format!("\t{}={}", escaped(k), escaped(v)));
                    }
                    text.push('\n');
                }
            }
        }
        text
    }

    #[cfg(all(test, feature = "test-hooks"))]
    pub fn text_for_test(&self) -> String {
        self.text()
    }

    /// Writes them beside the file, then over it: a crash leaves the old one.
    fn save(&self) -> io::Result<()> {
        let Some(file) = &self.file else {
            return Ok(());
        };
        write_replacing(file, &self.text())
    }

    /// `path` was opened just now: it goes first, once.
    pub fn opened(&mut self, path: &Path) {
        let at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs() as i64);
        self.recent.retain(|o| o.path != path);
        self.recent.insert(
            0,
            Opened {
                path: path.to_path_buf(),
                at,
            },
        );
        self.recent.truncate(RECENT);
    }

    /// Takes `path` out of the archives opened before; the file stays.
    pub fn forget_opened(&mut self, path: &Path) {
        self.recent.retain(|o| o.path != path);
    }

    /// Shows `algorithm`'s checksum, or not: the last one shown stays.
    pub fn show_checksum(&mut self, algorithm: Algorithm, on: bool) {
        let shown: Vec<_> = Algorithm::ALL
            .into_iter()
            .filter(|a| {
                if *a == algorithm {
                    on
                } else {
                    self.checksums.contains(a)
                }
            })
            .collect();
        if !shown.is_empty() {
            self.checksums = shown;
        }
    }

    /// The keys trusted, those that read: one edited by hand may not.
    pub fn trusted_public_keys(&self) -> Vec<PublicKey> {
        self.trusted_keys
            .iter()
            .filter_map(|k| read_public_key(&k.line).ok())
            .collect()
    }

    /// What the user calls the key trusted with this fingerprint.
    pub fn trusted_key_name(&self, fingerprint: &str) -> Option<String> {
        self.trusted_keys
            .iter()
            .find(|k| read_public_key(&k.line).is_ok_and(|key| key.fingerprint() == fingerprint))
            .map(|k| k.name.clone())
    }

    /// Trusts `key`, under `name`, or its short fingerprint if that says nothing. One
    /// trusted already keeps the name it has.
    pub fn trust(&mut self, name: String, key: &PublicKey) {
        let line = key.to_string();
        if self.trusted_keys.iter().any(|k| k.line == line) {
            return;
        }
        let name = name.replace('\n', " ");
        let name = if name.trim().is_empty() {
            key.fingerprint().chars().take(19).collect()
        } else {
            name
        };
        self.trusted_keys.push(TrustedKey { name, line });
    }

    /// How an archive is extracted, as Settings says.
    pub fn extract_options(&self) -> ExtractOptions {
        ExtractOptions {
            overwrite: self.overwrite,
            limit: LIMITS
                .get(self.limit)
                .map_or(LIMITS[1].1, |(_, limit)| *limit),
            keep_mark: self.keep_mark,
            portable_names: self.portable_names,
            // Asked about while extracting; left out where nothing asks.
            links_outside: OnLinksOutside::Skip,
            owners: self.restore_owners,
            xattrs: self.restore_xattrs,
        }
    }
}

fn write_replacing(file: &Path, text: &str) -> io::Result<()> {
    if let Some(folder) = file.parent() {
        fs::create_dir_all(folder)?;
    }
    let beside = file.with_extension("new");
    fs::write(&beside, text)?;
    fs::rename(&beside, file)
}

/// The preferences, the defaults if none were loaded.
pub fn prefs(cx: &App) -> Prefs {
    cx.try_global::<Prefs>().cloned().unwrap_or_default()
}

/// Whether the user has storages of their own: one they added, here or across a
/// network, says they mean to use them. The home and Downloads folders the page
/// starts with, before any change, are not.
pub fn has_storage(cx: &App) -> bool {
    cx.try_global::<Prefs>()
        .is_some_and(|p| p.storages_kept && !p.storages.is_empty())
}

/// The extensions ZIP stores as they are: none if that is off.
pub fn stored_extensions(cx: &App) -> Vec<String> {
    cx.try_global::<Prefs>()
        .filter(|p| p.store_compressed)
        .map(|p| {
            p.stored_extensions
                .clone()
                .unwrap_or_else(default_extensions)
        })
        .unwrap_or_default()
}

/// [`COMPRESSED_EXTENSIONS`], as the preferences keep them.
pub fn default_extensions() -> Vec<String> {
    COMPRESSED_EXTENSIONS
        .iter()
        .map(|e| e.to_string())
        .collect()
}

/// The extensions in `text`, as typed: apart by spaces or commas, with or
/// without their dot (or `*.`), each once, in lower case.
pub fn extensions_in(text: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for word in text.split(|c: char| c.is_whitespace() || c == ',' || c == ';') {
        let ext = word
            .trim_start_matches('*')
            .trim_start_matches('.')
            .to_lowercase();
        if !ext.is_empty() && !found.contains(&ext) {
            found.push(ext);
        }
    }
    found
}

/// Changes the preferences and keeps them. One that cannot be written lasts
/// for this run, and says so where a user of a terminal can see it.
pub fn change(cx: &mut App, f: impl FnOnce(&mut Prefs)) {
    let prefs = cx.default_global::<Prefs>();
    f(prefs);
    packwing_core::use_threads(prefs.threads);
    if let Err(e) = prefs.save() {
        eprintln!("packwing: cannot keep the settings: {e}");
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn extensions_are_read_as_typed_and_none_kept_is_the_known_list() {
        assert_eq!(
            extensions_in(" .JPG, png;*.Raw  raw\nheic "),
            ["jpg", "png", "raw", "heic"]
        );
        assert!(extensions_in(" , . ").is_empty());
        let p = Prefs::parse("language=en\n");
        assert!(p.store_compressed && p.stored_extensions.is_none());
        let p = Prefs::parse("stored_extensions=\n");
        assert_eq!(p.stored_extensions, Some(Vec::new()));
    }

    #[test]
    fn the_threads_chosen_come_back_and_none_kept_is_all() {
        assert_eq!(Prefs::parse("language=en\n").threads, 0);
        assert_eq!(Prefs::parse("threads=lots\n").threads, 0);
        let p = Prefs {
            threads: 4,
            ..Prefs::default()
        };
        assert_eq!(Prefs::parse(&p.text()).threads, 4);
    }

    #[test]
    fn what_a_backup_gives_back_comes_back_and_none_kept_is_nothing() {
        let unset = Prefs::parse("language=en\n");
        assert_eq!(
            (unset.restore_owners, unset.restore_xattrs),
            (Owners::Ignore, false)
        );
        assert_eq!(
            Prefs::parse("restore_owners=whoever\n").restore_owners,
            Owners::Ignore
        );
        for owners in [Owners::Ignore, Owners::ByName, Owners::ByNumber] {
            let p = Prefs {
                restore_owners: owners,
                restore_xattrs: true,
                ..Prefs::default()
            };
            let back = Prefs::parse(&p.text());
            assert_eq!((back.restore_owners, back.restore_xattrs), (owners, true));
            let how = back.extract_options();
            assert_eq!((how.owners, how.xattrs), (owners, true));
        }
    }

    #[test]
    fn what_a_look_for_updates_found_comes_back() {
        let mut p = Prefs::default();
        assert!(p.update.check);
        p.update = UpdateState {
            check: false,
            checked_at: Some(1_800_000_000),
            offered: Some("v0.3.0".into()),
            skipped: Some("v0.2.0".into()),
            package: Some(("packwing_0.3.0_amd64.deb".into(), "abc".into())),
            staged: Some(("v0.3.0".into(), PathBuf::from("/home/me/.cache/a b/x"))),
        };
        assert_eq!(Prefs::parse(&p.text()).update, p.update);
        // What older settings did not have is on by default.
        assert!(Prefs::parse("language=en\n").update.check);
    }

    #[test]
    fn a_storage_that_asks_first_comes_back_asking() {
        let other = |values: Values| KeptStorage {
            name: "Far".into(),
            place: KeptPlace::Other {
                kind: "s3".into(),
                id: "1".into(),
                values,
            },
        };
        let p = Prefs {
            storages: vec![
                other(Values::from([(ASKS_FIRST.to_owned(), "yes".to_owned())])),
                other(Values::new()),
            ],
            ..Prefs::default()
        };
        let back = Prefs::parse(&p.text());
        assert!(back.storages[0].asks_first());
        assert!(!back.storages[1].asks_first());
        assert!(
            !KeptStorage {
                name: "Here".into(),
                place: KeptPlace::Local("/here".into()),
            }
            .asks_first()
        );
    }

    #[test]
    fn storages_and_the_editor_come_back() {
        let mut p = Prefs {
            storages_kept: true,
            show_hidden: true,
            browse_folder: Some(PathBuf::from("/home/a b/c")),
            editor: Some("code -n".to_owned()),
            ..Prefs::default()
        };
        p.storages.push(KeptStorage {
            name: "Tài liệu\tcũ".to_owned(),
            place: KeptPlace::Local(PathBuf::from("/home/me/Tài liệu cũ")),
        });
        p.storages.push(KeptStorage {
            name: "Sao lưu".to_owned(),
            place: KeptPlace::Other {
                kind: "s3".to_owned(),
                id: "abc".to_owned(),
                values: [("bucket".to_owned(), "a=b%\tc".to_owned())].into(),
            },
        });
        let back = Prefs::parse(&p.text());
        assert_eq!(back.storages[0].name, "Tài liệu cũ");
        assert_eq!(back.storages[0].place, p.storages[0].place);
        assert_eq!(back.storages[1], p.storages[1]);
        assert!(back.storages_kept && back.show_hidden);
        assert_eq!(back.browse_folder, Some(PathBuf::from("/home/a b/c")));
        assert_eq!(back.editor.as_deref(), Some("code -n"));
        assert!(!Prefs::parse("").storages_kept);
        assert_eq!(Prefs::parse("editor=\n").editor, None);
    }

    #[test]
    fn what_is_kept_comes_back() {
        let mut p = Prefs {
            language: Some("en".to_owned()),
            window: Some(WindowPlace {
                x: 10,
                y: 20,
                width: 1400,
                height: 900,
                mode: crate::placement::WindowMode::Maximized,
            }),
            appearance: Appearance::Dark,
            format: Format::TarZst,
            level: Level::Highest,
            hide_names: false,
            skip_mac_files: false,
            store_compressed: false,
            stored_extensions: Some(vec!["raw".to_owned(), "cr2".to_owned()]),
            stream_above: 1_000_000_000,
            extract_to: ExtractTo::Here,
            overwrite: true,
            limit: 3,
            keep_mark: false,
            ask_before_open: false,
            signing_key: Some(PathBuf::from("/home/me/khoá của tôi")),
            trusted_keys: vec![TrustedKey {
                name: "Tôi\tở nhà".to_owned(),
                line: "ssh-ed25519 AAAA".to_owned(),
            }],
            checksums: vec![Algorithm::Sha512, Algorithm::Blake3],
            ..Prefs::default()
        };
        p.recent.push(Opened {
            path: PathBuf::from("/tài liệu/a=b c.zip"),
            at: 1_790_000_000,
        });
        assert_eq!(Prefs::parse(&p.text()), p);
    }

    #[test]
    fn a_key_trusted_once_keeps_its_name_and_one_that_does_not_read_is_left_out() {
        let dir = std::env::temp_dir().join(format!("packwing-trust-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let (key, _) = packwing_core::make_signing_key(&dir, b"a passphrase").unwrap();
        let _ = fs::remove_dir_all(&dir);
        let mut p = Prefs::default();
        p.trust("Mine".to_owned(), key.public());
        p.trust("Another".to_owned(), key.public());
        assert_eq!(p.trusted_keys.len(), 1);
        assert_eq!(p.trusted_public_keys(), vec![key.public().clone()]);
        let fingerprint = key.public().fingerprint();
        assert_eq!(p.trusted_key_name(&fingerprint).as_deref(), Some("Mine"));
        p.trusted_keys.push(TrustedKey {
            name: "Broken".to_owned(),
            line: "ssh-ed25519 nonsense".to_owned(),
        });
        assert_eq!(p.trusted_public_keys().len(), 1);
    }

    #[test]
    fn where_it_extracted_before_there_was_smart() {
        assert_eq!(
            Prefs::parse("new_folder=false\n").extract_to,
            ExtractTo::Here
        );
        assert_eq!(
            Prefs::parse("new_folder=true\n").extract_to,
            ExtractTo::Smart
        );
    }

    #[test]
    fn what_cannot_be_read_stays_at_its_default() {
        let p = Prefs::parse(
            "format=.rar\nlevel=9\nhide_names=maybe\nnew=1\nnonsense\nrecent=x y\nchecksums=crc\n",
        );
        assert_eq!(p, Prefs::default());
    }

    #[test]
    fn opening_again_moves_it_first_and_the_list_has_an_end() {
        let mut p = Prefs::default();
        for i in 0..RECENT + 5 {
            p.opened(Path::new(&format!("/a/{i}.zip")));
        }
        assert_eq!(p.recent.len(), RECENT);
        p.opened(Path::new("/a/10.zip"));
        assert_eq!(p.recent[0].path, Path::new("/a/10.zip"));
        assert_eq!(p.recent.len(), RECENT);
        assert_eq!(
            p.recent
                .iter()
                .filter(|o| o.path == Path::new("/a/10.zip"))
                .count(),
            1
        );
    }

    #[test]
    fn it_is_written_where_it_is_read() {
        let folder = std::env::temp_dir().join(format!("packwing-prefs-{}", std::process::id()));
        let _ = fs::remove_dir_all(&folder);
        let file = folder.join("deep/settings");
        let p = Prefs {
            file: Some(file.clone()),
            level: Level::Low,
            ..Prefs::default()
        };
        p.save().unwrap();
        p.save().unwrap();
        assert_eq!(
            Prefs::parse(&fs::read_to_string(&file).unwrap()).level,
            Level::Low
        );
        let _ = fs::remove_dir_all(&folder);
    }

    #[test]
    fn extracting_follows_the_settings() {
        let p = Prefs {
            overwrite: true,
            limit: 3,
            keep_mark: false,
            ..Prefs::default()
        };
        assert_eq!(
            p.extract_options(),
            ExtractOptions {
                overwrite: true,
                limit: None,
                keep_mark: false,
                portable_names: false,
                links_outside: OnLinksOutside::Skip,
                owners: Owners::Ignore,
                xattrs: false,
            }
        );
    }

    #[test]
    fn a_text_size_is_kept_when_it_is_one_offered() {
        let p = Prefs::parse("text_size=12\n");
        assert_eq!(p.text_size, Some(12));
        assert_eq!(Prefs::parse(&p.text()).text_size, Some(12));
        assert_eq!(Prefs::parse("text_size=7\n").text_size, None);
        assert_eq!(Prefs::parse("text_size=big\n").text_size, None);
        assert!(!Prefs::default().text().contains("text_size"));
    }

    #[test]
    fn checksums_stay_in_their_order_and_one_is_always_shown() {
        let mut p = Prefs::default();
        assert_eq!(p.checksums, [Algorithm::Sha256, Algorithm::Sha1]);
        p.show_checksum(Algorithm::Md5, true);
        p.show_checksum(Algorithm::Sha512, true);
        assert_eq!(
            p.checksums,
            [
                Algorithm::Sha256,
                Algorithm::Sha512,
                Algorithm::Sha1,
                Algorithm::Md5
            ]
        );
        for a in Algorithm::ALL {
            p.show_checksum(a, false);
        }
        assert_eq!(p.checksums, [Algorithm::Md5]);
        assert_eq!(
            Prefs::parse("checksums=md5 nonsense\n").checksums,
            [Algorithm::Md5]
        );
    }
}
