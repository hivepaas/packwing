//! What the user chose, kept between runs: a file of `key=value` lines in the
//! system's folder for settings, and the archives opened lately. Secrets never
//! go in it. A line it does not know, or cannot read, is left at its default,
//! so that a file from another version still opens.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use gpui::{App, Global};
use packwing_core::{ExtractOptions, Level, PublicKey, Values, read_public_key};

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
    /// How the window was last left; none before it has been.
    pub window: Option<WindowPlace>,
    pub appearance: Appearance,
    pub format: Format,
    pub level: Level,
    pub hide_names: bool,
    pub skip_mac_files: bool,
    /// In bytes: files that come to more go straight to a storage that can take
    /// them that way, and the rest through a temporary file.
    pub stream_above: u64,
    /// Where Extract starts.
    pub extract_to: ExtractTo,
    pub overwrite: bool,
    /// An index in [`LIMITS`].
    pub limit: usize,
    pub keep_mark: bool,
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
    /// The keys whose signatures are taken as proof.
    pub trusted_keys: Vec<TrustedKey>,
    /// The key file signed with last. Its contents never go in this file.
    pub signing_key: Option<PathBuf>,
}

impl Global for Prefs {}

impl Default for Prefs {
    fn default() -> Prefs {
        Prefs {
            file: None,
            language: None,
            window: None,
            appearance: Appearance::System,
            format: Format::Zip,
            level: Level::Medium,
            hide_names: true,
            skip_mac_files: true,
            stream_above: 500_000_000,
            extract_to: ExtractTo::Smart,
            overwrite: false,
            limit: 1,
            keep_mark: true,
            ask_before_open: true,
            recent: Vec::new(),
            storages: Vec::new(),
            storages_kept: false,
            show_hidden: false,
            browse_folder: None,
            editor: None,
            trusted_keys: Vec::new(),
            signing_key: None,
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
                "stream_above" => p.stream_above = value.parse().unwrap_or(p.stream_above),
                "extract_to" => p.extract_to = ExtractTo::named(value).unwrap_or(p.extract_to),
                // What it was before there was Smart: a new folder was the
                // default then, as Smart is now.
                "new_folder" if value == "false" => p.extract_to = ExtractTo::Here,
                "overwrite" => p.overwrite = on(value, p.overwrite),
                "keep_mark" => p.keep_mark = on(value, p.keep_mark),
                "ask_before_open" => p.ask_before_open = on(value, p.ask_before_open),
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
                "signing_key" => p.signing_key = Some(PathBuf::from(value)),
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
        if let Some(window) = self.window {
            text += &format!("window={}\n", window.to_text());
        }
        text += &format!(
            "appearance={appearance}\nformat={}\nlevel={level}\nhide_names={}\n\
             skip_mac_files={}\nstream_above={}\nextract_to={}\noverwrite={}\nlimit={limit}\nkeep_mark={}\n\
             ask_before_open={}\n",
            self.format.ext(),
            self.hide_names,
            self.skip_mac_files,
            self.stream_above,
            self.extract_to.name(),
            self.overwrite,
            self.keep_mark,
            self.ask_before_open,
        );
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
        if let Some(key) = self
            .signing_key
            .as_ref()
            .and_then(|f| f.to_str())
            .filter(|f| !f.contains('\n'))
        {
            text.push_str(&format!("signing_key={key}\n"));
        }
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

    #[cfg(all(test, feature = "shots"))]
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

/// Changes the preferences and keeps them. One that cannot be written lasts
/// for this run, and says so where a user of a terminal can see it.
pub fn change(cx: &mut App, f: impl FnOnce(&mut Prefs)) {
    let prefs = cx.default_global::<Prefs>();
    f(prefs);
    if let Err(e) = prefs.save() {
        eprintln!("packwing: cannot keep the settings: {e}");
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

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
        let p =
            Prefs::parse("format=.rar\nlevel=9\nhide_names=maybe\nnew=1\nnonsense\nrecent=x y\n");
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
            }
        );
    }
}
