//! UI scenarios: the real workspace in GPUI's test window, driven by clicks and typing
//! as a user would, with real archives and folders on disk. They read back what a user
//! would get: the files made, the page shown, the dialog open. A click finds its element
//! by the `.debug_selector(..)` that the widgets give their ids ([`Ui::click`]).
//!
//! Built only with the `ui-tests` feature. Locally `make test-ui`, or `make test-ui
//! FILTER=create::`; on CI on Linux for every pull request (`.github/workflows/ci.yml`),
//! and by hand and daily on all three systems (`.github/workflows/ui-tests.yml`).
//!
//! Each scenario runs in a child process of the test binary with its own home and config
//! folder, so that none reads or writes the user's settings or keychain, and each has the
//! organization's policy to itself. [`scenario!`] writes the pair of tests: `run`, which
//! starts the child, and `scenario`, ignored, which the child runs.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use gpui::{AppContext as _, TestAppContext, VisualTestContext, Window, WindowHandle};

use crate::right_click::{Choice, RightClickMenu, Status};
use crate::workspace::Workspace;

mod archive;
mod browse;
mod create;
mod edit;
mod far;
mod jobs;
mod policy;
mod settings;
mod storages;

/// How long a step may take before the scenario fails: CI machines are slow.
const STEP: Duration = Duration::from_secs(60);

/// Writes a scenario: `run` (a test) runs `scenario` (ignored) in a child process with
/// a scratch home; `scenario` gets a fresh [`Ui`].
macro_rules! scenario {
    ($(#[doc = $doc:literal])* $name:ident, $body:expr) => {
        $(#[doc = $doc])*
        mod $name {
            #[allow(unused_imports)]
            use super::*;

            #[test]
            fn run() {
                $crate::ui_tests::in_child(module_path!());
            }

            #[test]
            #[ignore = "run in its own process by `run`"]
            fn scenario() {
                let body: fn(&mut $crate::ui_tests::Ui) = $body;
                let mut ui = $crate::ui_tests::Ui::new();
                body(&mut ui);
            }
        }
    };
    // With what the organization has fixed, as this process's policy.
    ($(#[doc = $doc:literal])* $name:ident, policy: $policy:expr, $body:expr) => {
        $(#[doc = $doc])*
        mod $name {
            #[allow(unused_imports)]
            use super::*;

            #[test]
            fn run() {
                $crate::ui_tests::in_child(module_path!());
            }

            #[test]
            #[ignore = "run in its own process by `run`"]
            fn scenario() {
                let found: packwing_policy::Found = $policy;
                assert!(packwing_policy::install(packwing_policy::Policy::of(found)));
                let body: fn(&mut $crate::ui_tests::Ui) = $body;
                let mut ui = $crate::ui_tests::Ui::new();
                body(&mut ui);
            }
        }
    };
}
pub(crate) use scenario;

/// Runs the scenario of module `module` (a `module_path!()`) in a child process whose
/// home and config folders are in a fresh scratch directory.
pub fn in_child(module: &str) {
    // The test's name, as the harness knows it: without the crate's name.
    let module = module.split_once("::").map_or(module, |(_, rest)| rest);
    let name = format!("{module}::scenario");
    let dir = std::env::temp_dir().join(format!(
        "packwing-ui-{}-{}",
        module.replace("::", "-"),
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("work")).expect("scratch folder");
    let out = Command::new(std::env::current_exe().expect("test binary"))
        .args([
            "--exact",
            &name,
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ])
        .env("PACKWING_UI_TEST_DIR", &dir)
        .env("HOME", &dir)
        .env("USERPROFILE", &dir)
        .env("APPDATA", dir.join("appdata"))
        .env("LOCALAPPDATA", dir.join("localappdata"))
        .env("XDG_CONFIG_HOME", dir.join("xdg/config"))
        .env("XDG_DATA_HOME", dir.join("xdg/data"))
        .env("XDG_CACHE_HOME", dir.join("xdg/cache"))
        // No session bus: nothing can reach the desktop's keychain.
        .env_remove("DBUS_SESSION_BUS_ADDRESS")
        .output()
        .expect("run the scenario");
    let (stdout, stderr) = (
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    assert!(
        out.status.success() && stdout.contains("1 passed"),
        "scenario {name} failed:\n{stdout}\n{stderr}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// One of the archives the core's tests read.
pub fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../core/tests/data")
        .join(name)
}

/// A scenario's app: one window with the workspace, as the program opens it.
pub struct Ui {
    pub cx: TestAppContext,
    /// Where the scenario keeps its files.
    pub work: PathBuf,
    window: Option<WindowHandle<Workspace>>,
}

impl Ui {
    pub fn new() -> Ui {
        let dir = std::env::var_os("PACKWING_UI_TEST_DIR")
            .map(PathBuf::from)
            .expect("run through in_child");
        let cx = TestAppContext::single();
        cx.executor().allow_parking();
        let mut ui = Ui {
            cx,
            work: dir.join("work"),
            window: None,
        };
        ui.start();
        ui
    }

    /// What the program does as it starts: what was kept is read, and the window opens.
    pub fn start(&mut self) {
        self.cx.update(|cx| {
            cx.set_global(crate::Extensions {
                storage_kinds: far::kinds(),
            });
            let kept = crate::prefs::Prefs::load();
            packwing_i18n::set_language(kept.language.as_deref().or(Some("en")));
            cx.set_global(kept);
            cx.set_global(crate::licensing::Licensed::load());
        });
        let window = self.cx.add_window(|window, cx| {
            // Nothing is put in the system's menu.
            let menu = cx.new(|_| RightClickMenu::offline(true, Choice::default(), Status::Idle));
            Workspace::new(false, menu, window, cx)
        });
        self.window = Some(window);
        self.cx.run_until_parked();
    }

    fn window(&self) -> WindowHandle<Workspace> {
        self.window.expect("the window is open")
    }

    /// Acts on the workspace.
    pub fn ws<R>(
        &mut self,
        f: impl FnOnce(&mut Workspace, &mut Window, &mut gpui::Context<Workspace>) -> R,
    ) -> R {
        let out = self
            .window()
            .update(&mut self.cx, f)
            .expect("the window is open");
        self.cx.run_until_parked();
        out
    }

    /// A file made for the scenario.
    pub fn file(&self, name: &str, text: &str) -> PathBuf {
        let path = self.work.join(name);
        if let Some(folder) = path.parent() {
            std::fs::create_dir_all(folder).expect("its folder");
        }
        std::fs::write(&path, text).expect("the file");
        path
    }

    /// A copy of a fixture in the scenario's folder, to be changed or extracted beside.
    pub fn copy_of(&self, name: &str) -> PathBuf {
        let path = self.work.join(name);
        std::fs::copy(fixture(name), &path).expect("the fixture");
        path
    }

    /// Lets the app run for a moment, as a job on another thread finishes.
    pub fn wait_until(&mut self, what: &str, mut done: impl FnMut(&mut Ui) -> bool) {
        let start = Instant::now();
        loop {
            self.cx.run_until_parked();
            if done(self) {
                return;
            }
            assert!(start.elapsed() < STEP, "waited {STEP:?} for {what}");
            std::thread::sleep(Duration::from_millis(15));
        }
    }

    pub fn draw(&mut self) {
        let window = self.window();
        gpui::AppContext::update_window(&mut self.cx, window.into(), |_, w, cx| {
            w.refresh();
            w.draw(cx).clear(cx);
        })
        .expect("the window is open");
    }

    /// The element marked `selector`, if the window draws it.
    pub fn find(&mut self, selector: &str) -> Option<gpui::Bounds<gpui::Pixels>> {
        self.draw();
        // GPUI looks selectors up by a 'static str; a scenario names a few.
        let selector: &'static str = Box::leak(selector.to_string().into_boxed_str());
        let mut vcx = VisualTestContext::from_window(self.window().into(), &self.cx);
        vcx.debug_bounds(selector)
    }

    pub fn shows(&mut self, selector: &str) -> bool {
        self.find(selector).is_some()
    }

    /// Waits until the window draws the element marked `selector`.
    pub fn wait_for(&mut self, selector: &str) {
        self.wait_until(selector, |ui| ui.shows(selector));
    }

    /// Clicks the middle of the element marked `selector`, as the mouse does.
    pub fn click(&mut self, selector: &str) {
        let at = self
            .find(selector)
            .unwrap_or_else(|| panic!("nothing marked {selector:?} is drawn"))
            .center();
        let mut vcx = VisualTestContext::from_window(self.window().into(), &self.cx);
        vcx.simulate_click(at, gpui::Modifiers::none());
        self.cx.run_until_parked();
    }

    /// Presses the right button on the middle of the element marked `selector`.
    pub fn right_click(&mut self, selector: &str) {
        let at = self
            .find(selector)
            .unwrap_or_else(|| panic!("nothing marked {selector:?} is drawn"))
            .center();
        let mut vcx = VisualTestContext::from_window(self.window().into(), &self.cx);
        let none = gpui::Modifiers::none();
        vcx.simulate_mouse_down(at, gpui::MouseButton::Right, none);
        vcx.simulate_mouse_up(at, gpui::MouseButton::Right, none);
        self.cx.run_until_parked();
    }

    /// Opens the select marked `mark` and clicks its option `option`, as the mouse does.
    pub fn choose(&mut self, mark: &str, option: &str) {
        self.click(&format!("select: {mark}"));
        let option = format!("option: {option}");
        // A long menu scrolls: the wheel brings the option into it.
        for _ in 0..20 {
            let menu = self.find("menu").expect("the menu is open");
            let at = self
                .find(&option)
                .unwrap_or_else(|| panic!("the menu has no {option:?}"));
            if menu.contains(&at.center()) {
                break;
            }
            let down = if at.center().y > menu.center().y {
                -1.
            } else {
                1.
            };
            let mut vcx = VisualTestContext::from_window(self.window().into(), &self.cx);
            vcx.simulate_event(gpui::ScrollWheelEvent {
                position: menu.center(),
                delta: gpui::ScrollDelta::Pixels(gpui::point(gpui::px(0.), gpui::px(down * 40.))),
                modifiers: gpui::Modifiers::none(),
                touch_phase: gpui::TouchPhase::Moved,
            });
            self.cx.run_until_parked();
        }
        self.click(&option);
    }

    /// Clicks the field whose hint is `hint`, and types `text` in it.
    pub fn type_in(&mut self, hint: &str, text: &str) {
        self.click(&format!("field: {hint}"));
        let window = self.window();
        self.cx.simulate_input(window.into(), text);
        self.cx.run_until_parked();
    }

    /// Clicks the field whose hint is `hint`, and types `text` over what it holds. Its hint
    /// is its selector whatever it holds, so the same click finds it.
    pub fn replace_in(&mut self, hint: &str, text: &str) {
        self.click(&format!("field: {hint}"));
        let select_all = if cfg!(target_os = "macos") {
            "cmd-a"
        } else {
            "ctrl-a"
        };
        self.keys(select_all);
        let window = self.window();
        self.cx.simulate_input(window.into(), text);
        self.cx.run_until_parked();
    }

    /// Presses keys, GPUI's way.
    pub fn keys(&mut self, keys: &str) {
        let window = self.window();
        self.cx.simulate_keystrokes(window.into(), keys);
        self.cx.run_until_parked();
    }

    pub fn dialog_open(&mut self) -> bool {
        self.ws(|ws, _, _| ws.dialog_open())
    }

    /// Waits until no dialog is open: what was asked is done, or failed.
    pub fn wait_for_no_dialog(&mut self) {
        self.wait_until("no dialog", |ui| {
            if let Some(why) = ui.ws(|ws, _, cx| ws.job_failed(cx)) {
                panic!("the job failed: {why}");
            }
            !ui.dialog_open()
        });
    }

    /// Opens the archive at `path`, as the program does with one it is given.
    pub fn open(&mut self, path: &Path) {
        let path = path.to_path_buf();
        self.ws(|ws, window, cx| ws.open_archive(path, false, window, cx));
        self.wait_until("the archive read", |ui| {
            ui.ws(|ws, _, cx| {
                let page = ws.archive_page();
                let page = page.read(cx);
                page.archive().is_some() || page.shown_failed().is_some() || ws.dialog_open()
            })
        });
    }

    /// The storage kept at `at` in the Storages page shows `folder`, and the page opens.
    pub fn storage(&mut self, folder: &Path) {
        let folder = folder.to_path_buf();
        std::fs::create_dir_all(&folder).expect("the folder");
        self.ws(|ws, _, cx| ws.show_storage(folder, cx));
    }

    /// The names the explorer of the Storages page shows, listed now.
    pub fn explorer_names(&mut self) -> Vec<String> {
        self.ws(|ws, _, cx| {
            let explorer = ws.explorer(cx);
            explorer.update(cx, |e, cx| e.list_now(cx));
            explorer.read(cx).shown_names()
        })
    }

    /// Opens Settings on section number `i`, by the menus as a user does.
    pub fn settings(&mut self, i: usize) {
        self.click("nav-settings");
        self.click(&format!("section-{i}"));
    }

    /// The plan this computer is on.
    pub fn plan(&mut self) -> packwing_license::Plan {
        self.ws(|_, _, cx| crate::licensing::plan(cx))
    }

    /// How many storages are across a network.
    pub fn remote_storages(&mut self) -> usize {
        self.ws(|ws, _, cx| ws.storages_page().read(cx).remote_count())
    }

    /// The names in the folder the Archive page shows.
    pub fn archive_names(&mut self) -> Vec<String> {
        self.ws(|ws, _, cx| {
            let page = ws.archive_page();
            let page = page.read(cx);
            page.archive().map(|a| a.top_names()).unwrap_or_default()
        })
    }

    /// The names the Archive page lists now: the folder's, or what the search found.
    pub fn shown_in_archive(&mut self) -> Vec<String> {
        self.ws(|ws, _, cx| ws.archive_page().read(cx).shown_names(cx))
    }

    /// The folder in the archive the Archive page shows.
    pub fn archive_folder(&mut self) -> Vec<String> {
        self.ws(|ws, _, cx| ws.archive_page().read(cx).shown_folder())
    }

    /// The folder on this computer the Browse page shows.
    pub fn browsed(&mut self) -> Option<PathBuf> {
        self.ws(|ws, _, cx| {
            let explorer = ws.browse_page().read(cx).explorer().read(cx);
            explorer.place()?.storage.local_path(explorer.folder())
        })
    }

    /// Why the archive could not be read, if it could not.
    pub fn archive_failed(&mut self) -> Option<String> {
        self.ws(|ws, _, cx| ws.archive_page().read(cx).shown_failed())
    }
}

/// What an archive holds, read by the core, not by the window.
pub fn listed(path: &Path, password: Option<&str>) -> Vec<String> {
    let keys = packwing_core::Keys {
        password: password.map(|p| p.as_bytes().to_vec()),
        passphrase: password.map(|p| p.as_bytes().to_vec()),
        ..packwing_core::Keys::default()
    };
    packwing_core::list(packwing_core::Source::from(path), &keys)
        .unwrap_or_else(|e| panic!("{path:?} cannot be read: {e}"))
        .entries
        .iter()
        .map(|e| e.path.join("/"))
        .collect()
}
