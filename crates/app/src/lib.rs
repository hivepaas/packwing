//! packwing: a file archiver that opens archives safely.
//!
//! Usage: `packwing [--first-run] [--policies] [--<item> PATH…]`. Archives
//! are read, extracted and tested through packwing-core and hivepack
//! (`--open PATH` opens one), and made; the screenshots show made-up ones
//! (`demo.rs`). What the user chooses is kept (`prefs.rs`). `--first-run`
//! opens on the welcome. An item of the right-click menu runs packwing with its
//! option and the paths chosen (`right_click.rs`).

// The hooks the screenshots look through are not all the scenarios', nor theirs all
// the screenshots'.
#![cfg_attr(
    all(test, any(feature = "ui-tests", feature = "shots")),
    allow(dead_code)
)]

mod chrome;
mod colors;
mod confirm;
// Most of it only the screenshots draw.
#[cfg_attr(not(all(test, feature = "test-hooks")), allow(dead_code))]
mod demo;
mod dialogs;
mod editors;
mod explorer;
mod input;
mod licensing;
mod managed;
mod model;
mod open_with;
mod pages;
mod placement;
mod prefs;
mod right_click;
mod signatures;
mod updates;
mod widgets;
mod workspace;

#[cfg(all(test, feature = "shots", target_os = "linux"))]
mod shots;
#[cfg(all(test, feature = "ui-tests"))]
mod ui_tests;

use std::borrow::Cow;
use std::sync::Arc;

use gpui::{
    App, AppContext as _, Focusable as _, TitlebarOptions, WindowDecorations, WindowOptions, px,
    size,
};

use right_click::{Action, Request, RightClickMenu};

/// What a build adds to the app as published.
#[derive(Clone, Default)]
pub struct Extensions {
    /// Kinds of storage besides a folder on this computer.
    pub storage_kinds: Vec<Arc<dyn packwing_core::StorageKind>>,
}

impl gpui::Global for Extensions {}

/// The UI's typeface, drawn into the binary. The design asks for Figtree,
/// which has no Vietnamese letters; Nunito has every one of them, and its
/// rounded ends suit the Soft tonal design. Its four weights are static
/// instances of google/fonts' variable `ofl/nunito/Nunito[wght].ttf`, made
/// with fontTools' `varLib.instancer --static --update-name-table`.
pub const FONT: &str = "Nunito";

/// The system's fonts for Chinese, Japanese and Korean, which Nunito has none of, the
/// language's own first, as a character shared between them is drawn differently in each.
/// Those not installed are passed over.
fn cjk_fonts() -> Vec<String> {
    let japanese = ["Hiragino Sans", "Yu Gothic UI", "Noto Sans CJK JP"];
    let korean = ["Apple SD Gothic Neo", "Malgun Gothic", "Noto Sans CJK KR"];
    let simplified = ["PingFang SC", "Microsoft YaHei UI", "Noto Sans CJK SC"];
    let traditional = ["PingFang TC", "Microsoft JhengHei UI", "Noto Sans CJK TC"];
    let order = match packwing_i18n::language() {
        "ko" => [korean, japanese, simplified, traditional],
        "zh" => [simplified, traditional, japanese, korean],
        "zh-Hant" => [traditional, simplified, japanese, korean],
        _ => [japanese, simplified, traditional, korean],
    };
    order.iter().flatten().map(|f| (*f).to_owned()).collect()
}

/// The window's font: [`FONT`], and for what it lacks [`cjk_fonts`].
pub fn ui_font() -> gpui::Font {
    gpui::Font {
        fallbacks: Some(gpui::FontFallbacks::from_fonts(cjk_fonts())),
        ..gpui::font(FONT)
    }
}

const FONTS: [&[u8]; 4] = [
    include_bytes!("../assets/fonts/Nunito-Regular.ttf"),
    include_bytes!("../assets/fonts/Nunito-Medium.ttf"),
    include_bytes!("../assets/fonts/Nunito-SemiBold.ttf"),
    include_bytes!("../assets/fonts/Nunito-Bold.ttf"),
];

/// Hands [`FONTS`] to GPUI; without them the system's font draws the UI.
pub fn load_fonts(cx: &mut App) {
    let fonts = FONTS.iter().map(|f| Cow::Borrowed(*f)).collect();
    if let Err(e) = cx.text_system().add_fonts(fonts) {
        eprintln!("packwing: cannot load the UI font: {e}");
    }
}

/// Icons drawn into the binary: the app has no other files.
pub struct Assets;

macro_rules! icons {
    ($($name:literal),* $(,)?) => {
        &[$((
            concat!("icons/", $name, ".svg"),
            include_bytes!(concat!("../assets/icons/", $name, ".svg")),
        )),*]
    };
}

const ICONS: &[(&str, &[u8])] = icons![
    "appearance-dark",
    "appearance-light",
    "appearance-system",
    "archive",
    "arrow-up",
    "building",
    "check",
    "chevron-down",
    "desktop",
    "download",
    "drop",
    "extract",
    "eye",
    "eye-off",
    "folder",
    "home",
    "image",
    "lock",
    "logo",
    "logo-feather",
    "more",
    "plus",
    "recent",
    "refresh",
    "search",
    "shellwing",
    "shield",
    "sliders",
    "storages",
    "warning",
    "x",
    // The window's buttons, as GNOME and Windows draw them.
    "minimize",
    "maximize",
    "close",
    "win-minimize",
    "win-maximize",
    "win-restore",
    "win-close",
];

impl gpui::AssetSource for Assets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        Ok(ICONS
            .iter()
            .find(|(p, _)| *p == path)
            .map(|(_, bytes)| Cow::Borrowed(*bytes)))
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<gpui::SharedString>> {
        Ok(ICONS
            .iter()
            .filter(|(p, _)| p.starts_with(path))
            .map(|(p, _)| (*p).into())
            .collect())
    }
}

fn usage() -> String {
    let items: Vec<String> = Action::ALL.iter().map(|a| a.option()).collect();
    format!(
        "usage: packwing [--first-run] [--policies] [--<item> PATH…]
  --first-run  open on the welcome, as on the first start
  --policies   say what an organization has fixed here, and any mistake in it, then stop
  --<item>     what an item of the right-click menu runs, with the paths chosen:
               {}",
        items.join(", ")
    )
}

/// Runs the app, with what this build adds to it.
pub fn run(extensions: Extensions) {
    let (mut first_run, mut request) = (false, None);
    let mut args = std::env::args_os().skip(1);
    while let Some(a) = args.next() {
        let text = a.to_string_lossy();
        match text.as_ref() {
            "-h" | "--help" => {
                println!("{}", usage());
                return;
            }
            "--first-run" => first_run = true,
            "--policies" => {
                let policy = packwing_policy::current();
                print!("{}", policy.report());
                let wrong =
                    policy.failure().is_some() || policy.rows().iter().any(|r| r.problem.is_some());
                std::process::exit(i32::from(wrong));
            }
            option => match Request::parse(option, args.by_ref()) {
                Some(r) => request = Some(r),
                None => {
                    eprintln!("packwing: unknown option {text}\n{}", usage());
                    std::process::exit(2);
                }
            },
        }
    }
    gpui_platform::application()
        .with_assets(Assets)
        .run(move |cx: &mut App| {
            load_fonts(cx);
            cx.set_global(extensions);
            let kept = prefs::Prefs::load();
            packwing_i18n::set_language(kept.language.as_deref());
            let window_bounds = placement::initial(kept.window, cx);
            cx.set_global(kept);
            cx.set_global(licensing::Licensed::load());
            packwing_core::clean_opened_files();
            // GNOME's Wayland compositor draws no window frame, so Linux gets
            // client-side decorations. macOS and Windows keep the system's
            // frame but hide its title bar, and the app draws its own.
            let client_side = cfg!(any(target_os = "linux", target_os = "freebsd"));
            let options = WindowOptions {
                window_bounds: Some(window_bounds),
                window_min_size: Some(size(px(placement::MIN_SIZE.0), px(placement::MIN_SIZE.1))),
                window_decorations: client_side.then_some(WindowDecorations::Client),
                titlebar: (!client_side).then(|| TitlebarOptions {
                    title: Some("packwing".into()),
                    appears_transparent: true,
                    traffic_light_position: Some(chrome::TRAFFIC_LIGHTS),
                }),
                app_id: Some("packwing".into()),
                ..Default::default()
            };
            let menu = cx.new(RightClickMenu::installed);
            open_with::keep_current(cx);
            let opened = cx.open_window(options, |window, cx| {
                let ws = cx.new(|cx| {
                    let mut ws = workspace::Workspace::new(first_run, menu, window, cx);
                    ws.track_bounds(window, cx);
                    ws.start_updates(cx);
                    if let Some(r) = request {
                        ws.start(r, window, cx);
                    }
                    ws
                });
                window.focus(&ws.read(cx).focus_handle(cx), cx);
                ws
            });
            if opened.is_err() {
                eprintln!("packwing: cannot open a window");
                cx.quit();
                return;
            }
            cx.activate(true);
            // One window: closing it quits, on macOS too.
            cx.on_window_closed(|cx, _| {
                if cx.windows().is_empty() {
                    cx.quit();
                }
            })
            .detach();
        });
}

#[cfg(test)]
mod tests {
    /// Words on the screen go through `tr!`, or a new language would miss them.
    /// This looks where they are most often written; `scripts/i18n.py` finds the
    /// rest once they are there.
    #[test]
    fn what_the_window_says_can_be_translated() {
        fn walk(folder: &std::path::Path, found: &mut Vec<std::path::PathBuf>) {
            for entry in std::fs::read_dir(folder).expect("the sources") {
                let path = entry.expect("an entry").path();
                if path.is_dir() {
                    walk(&path, found);
                } else if path.extension().is_some_and(|e| e == "rs") {
                    found.push(path);
                }
            }
        }
        let mut sources = Vec::new();
        let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        walk(&src, &mut sources);
        let said = [
            ".child(\"",
            "legend(\"",
            "label(\"",
            "title(\"",
            "setting(\"",
        ];
        for path in sources {
            let name = path.file_name().map(|n| n.to_string_lossy().into_owned());
            if matches!(name.as_deref(), Some("shots.rs" | "demo.rs" | "lib.rs")) {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("a source");
            for (i, line) in text.lines().enumerate() {
                for start in said {
                    if let Some(at) = line.find(start) {
                        let rest = &line[at + start.len()..];
                        assert!(
                            !rest.starts_with(|c: char| c.is_alphabetic())
                                || rest.starts_with("packwing\""),
                            "{}:{}: {line}",
                            path.display(),
                            i + 1
                        );
                    }
                }
            }
        }
    }

    use super::*;

    #[test]
    fn every_icon_is_drawn_into_the_binary() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons");
        for entry in std::fs::read_dir(dir).expect("assets/icons") {
            let name = entry.expect("an icon").file_name();
            let path = format!("icons/{}", name.to_string_lossy());
            assert!(
                ICONS.iter().any(|(p, _)| *p == path),
                "{path} is not in ICONS"
            );
        }
    }
}
