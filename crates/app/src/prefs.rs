//! What the user chose, kept between runs: a file of `key=value` lines in the
//! system's folder for settings, and the archives opened lately. Secrets never
//! go in it. A line it does not know, or cannot read, is left at its default,
//! so that a file from another version still opens.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use gpui::{App, Global};
use packwing_core::{ExtractOptions, Level};

use crate::dialogs::Format;
use crate::pages::settings::Appearance;

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prefs {
    /// Where they are kept: none while they are only for this run, as in tests.
    file: Option<PathBuf>,
    pub appearance: Appearance,
    pub format: Format,
    pub level: Level,
    pub hide_names: bool,
    pub skip_mac_files: bool,
    /// Extract starts on a new folder, rather than the archive's own.
    pub new_folder: bool,
    pub overwrite: bool,
    /// An index in [`LIMITS`].
    pub limit: usize,
    pub keep_mark: bool,
    /// The latest first.
    pub recent: Vec<Opened>,
}

impl Global for Prefs {}

impl Default for Prefs {
    fn default() -> Prefs {
        Prefs {
            file: None,
            appearance: Appearance::System,
            format: Format::Zip,
            level: Level::Medium,
            hide_names: true,
            skip_mac_files: true,
            new_folder: true,
            overwrite: false,
            limit: 1,
            keep_mark: true,
            recent: Vec::new(),
        }
    }
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
                "new_folder" => p.new_folder = on(value, p.new_folder),
                "overwrite" => p.overwrite = on(value, p.overwrite),
                "keep_mark" => p.keep_mark = on(value, p.keep_mark),
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
            "appearance={appearance}\nformat={}\nlevel={level}\nhide_names={}\n\
             skip_mac_files={}\nnew_folder={}\noverwrite={}\nlimit={limit}\nkeep_mark={}\n",
            self.format.ext(),
            self.hide_names,
            self.skip_mac_files,
            self.new_folder,
            self.overwrite,
            self.keep_mark,
        );
        for opened in &self.recent {
            if let Some(path) = opened.path.to_str().filter(|p| !p.contains('\n')) {
                text.push_str(&format!("recent={} {path}\n", opened.at));
            }
        }
        text
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
    fn what_is_kept_comes_back() {
        let mut p = Prefs {
            appearance: Appearance::Dark,
            format: Format::TarZst,
            level: Level::Highest,
            hide_names: false,
            skip_mac_files: false,
            new_folder: false,
            overwrite: true,
            limit: 3,
            keep_mark: false,
            ..Prefs::default()
        };
        p.recent.push(Opened {
            path: PathBuf::from("/tài liệu/a=b c.zip"),
            at: 1_790_000_000,
        });
        assert_eq!(Prefs::parse(&p.text()), p);
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
