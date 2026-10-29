//! packwing in the system's right-click menu, through context-menu-rs: its
//! items, what each runs, and putting those chosen in the menu.
//!
//! The menu as installed is the setting. context-menu-rs keeps its
//! definition for the user, which says which items are in it and whether
//! they are in a submenu, so packwing keeps nothing else for it; with no
//! definition, packwing is not in the menu. Items chosen while it is not are
//! kept until packwing quits.
//!
//! Each item runs packwing with its option and the paths chosen,
//! `packwing --extract-here a.zip b.zip`: a [`Request`].

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use context_menu::{Count, Item, Menu};
use gpui::{AppContext as _, Context, Task};

/// The menu's id, where the system keeps it.
const ID: &str = "packwing";

/// What the archive items show for: the types packwing opens, by extension,
/// and the short names of compressed tars. `gz` takes in `tar.gz`, and the
/// others alike.
const ARCHIVES: [&str; 12] = [
    "zip", "rar", "7z", "tar", "gz", "tgz", "xz", "txz", "zst", "tzst", "cbz", "cbr",
];

/// One of packwing's items.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    ExtractHere,
    ExtractToFolder,
    Extract,
    Open,
    CompressZip,
    Compress,
}

impl Action {
    /// In the menu's order.
    pub const ALL: [Action; 6] = [
        Action::ExtractHere,
        Action::ExtractToFolder,
        Action::Extract,
        Action::Open,
        Action::CompressZip,
        Action::Compress,
    ];

    /// Its id in the menu, and its option on packwing's command line.
    fn id(self) -> &'static str {
        match self {
            Action::ExtractHere => "extract-here",
            Action::ExtractToFolder => "extract-to-folder",
            Action::Extract => "extract",
            Action::Open => "open",
            Action::CompressZip => "compress-zip",
            Action::Compress => "compress",
        }
    }

    /// What the menu says. One label for every archive: the menu cannot
    /// name the one chosen.
    pub fn label(self) -> &'static str {
        match self {
            Action::ExtractHere => "Extract here",
            Action::ExtractToFolder => "Extract to new folder",
            Action::Extract => "Extract…",
            Action::Open => "Open with packwing",
            Action::CompressZip => "Compress to ZIP",
            Action::Compress => "Compress…",
        }
    }

    /// Its option on packwing's command line, which the item runs with the
    /// paths chosen after it.
    pub fn option(self) -> String {
        format!("--{}", self.id())
    }

    /// The action whose option `arg` is.
    pub fn of_option(arg: &str) -> Option<Action> {
        let id = arg.strip_prefix("--")?;
        Action::ALL.into_iter().find(|a| a.id() == id)
    }

    /// Its item in the menu, running `program`.
    fn item(self, program: &Path) -> Item {
        let item = Item::new(self.id(), self.label(), program).arg(self.option());
        match self {
            Action::ExtractHere | Action::ExtractToFolder => item.extensions(ARCHIVES),
            // A dialog, or a window, for one archive.
            Action::Extract | Action::Open => item.extensions(ARCHIVES).count(Count::One),
            Action::CompressZip | Action::Compress => item.folders(true),
        }
    }
}

/// An item chosen in the menu: what to do, and the files and folders chosen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub action: Action,
    pub paths: Vec<PathBuf>,
}

impl Request {
    /// From packwing's command line, when `arg` is an item's option: every
    /// argument after it is a path, whatever it looks like.
    pub fn parse(arg: &str, rest: impl IntoIterator<Item = OsString>) -> Option<Request> {
        Some(Request {
            action: Action::of_option(arg)?,
            paths: rest.into_iter().map(PathBuf::from).collect(),
        })
    }
}

/// Which items are in the menu, in its order, and whether in a submenu of
/// packwing's own.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Choice {
    pub items: Vec<Action>,
    pub submenu: bool,
}

impl Default for Choice {
    /// What the first run puts in the menu: every item but Extract…, which
    /// the others and Open with packwing cover.
    fn default() -> Self {
        Choice {
            items: Action::ALL
                .into_iter()
                .filter(|a| *a != Action::Extract)
                .collect(),
            submenu: false,
        }
    }
}

impl Choice {
    pub fn has(&self, a: Action) -> bool {
        self.items.contains(&a)
    }

    /// Takes `a` out, or puts it in, in its place.
    pub fn toggle(&mut self, a: Action) {
        let on = !self.has(a);
        self.items = Action::ALL
            .into_iter()
            .filter(|&b| if b == a { on } else { self.has(b) })
            .collect();
    }

    /// The menu of these items, each running `program`: none without items.
    fn menu(&self, program: &Path) -> Option<Menu> {
        if self.items.is_empty() {
            return None;
        }
        let menu = Menu::new(ID, "packwing").submenu(self.submenu);
        Some(
            self.items
                .iter()
                .fold(menu, |menu, a| menu.item(a.item(program))),
        )
    }

    /// What an installed menu holds: the items of packwing's it knows.
    fn of(menu: &Menu) -> Choice {
        Choice {
            items: Action::ALL
                .into_iter()
                .filter(|a| menu.items.iter().any(|i| i.id == a.id()))
                .collect(),
            submenu: menu.submenu,
        }
    }
}

/// What the last change to the menu came to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    /// Nothing was changed since packwing started.
    Idle,
    /// Being changed.
    Working,
    /// Changed: what the user may need to know, from context-menu-rs, such
    /// as where a file manager puts the items.
    Done(Vec<String>),
    /// It could not be.
    Failed(String),
}

/// packwing's items in the system's menu, which the welcome and Settings
/// change: one for the app.
pub struct RightClickMenu {
    /// packwing is in the menu.
    shown: bool,
    /// The items, kept while packwing is not in the menu too.
    choice: Choice,
    status: Status,
    /// Changes the system's menu: not in tests, which change nothing.
    system: bool,
    /// Counts the changes: only the last one's end says how it went.
    changes: u64,
    /// The last change, which the next one waits for.
    job: Option<Task<()>>,
}

impl RightClickMenu {
    /// The menu as installed for the user. When it is, it is brought up to
    /// date in the background: packwing may have moved, or been updated.
    pub fn installed(cx: &mut Context<Self>) -> Self {
        let mut this = RightClickMenu {
            shown: false,
            choice: Choice::default(),
            status: Status::Idle,
            system: true,
            changes: 0,
            job: None,
        };
        match read() {
            Ok(Some(menu)) => {
                this.shown = true;
                this.choice = Choice::of(&menu);
                this.apply(true, cx);
            }
            Ok(None) => {}
            Err(e) => this.status = Status::Failed(e.to_string()),
        }
        this
    }

    /// One that changes nothing on the system, for the screenshots.
    #[cfg(all(test, feature = "shots"))]
    pub fn offline(shown: bool, choice: Choice, status: Status) -> Self {
        RightClickMenu {
            shown,
            choice,
            status,
            system: false,
            changes: 0,
            job: None,
        }
    }

    pub fn shown(&self) -> bool {
        self.shown
    }

    pub fn choice(&self) -> &Choice {
        &self.choice
    }

    pub fn status(&self) -> &Status {
        &self.status
    }

    /// Puts packwing in the menu, or takes it out. With no items chosen,
    /// it goes in with the first run's.
    pub fn show(&mut self, shown: bool, cx: &mut Context<Self>) {
        if shown && self.choice.items.is_empty() {
            self.choice = Choice::default();
        }
        self.shown = shown;
        self.apply(false, cx);
    }

    /// Chooses the items. With none, packwing leaves the menu.
    pub fn choose(&mut self, choice: Choice, cx: &mut Context<Self>) {
        self.choice = choice;
        if self.shown {
            self.shown = !self.choice.items.is_empty();
            self.apply(false, cx);
        }
        cx.notify();
    }

    #[cfg(all(test, feature = "shots"))]
    pub fn set_status(&mut self, status: Status, cx: &mut Context<Self>) {
        self.status = status;
        cx.notify();
    }

    /// Brings the system's menu to what is chosen, once the change before
    /// is done. `if_stale`: only when it is not installed so already.
    fn apply(&mut self, if_stale: bool, cx: &mut Context<Self>) {
        cx.notify();
        if !self.system {
            return;
        }
        let menu = if self.shown {
            match std::env::current_exe() {
                Ok(program) => self.choice.menu(&program),
                Err(e) => {
                    self.status = Status::Failed(format!("packwing cannot find its program: {e}"));
                    return;
                }
            }
        } else {
            None
        };
        self.changes += 1;
        let change = self.changes;
        if !if_stale {
            self.status = Status::Working;
        }
        let before = self.job.take();
        self.job = Some(cx.spawn(async move |this, cx| {
            if let Some(before) = before {
                before.await;
            }
            let came = cx
                .background_spawn(async move { change_menu(menu.as_ref(), if_stale) })
                .await;
            let _ = this.update(cx, |this, cx| {
                if this.changes == change
                    && let Some(status) = came
                {
                    this.status = status;
                    cx.notify();
                }
            });
        }));
    }
}

/// Installs `menu`, or without one takes packwing's away; `if_stale`, only
/// when it is not installed so already. How it went, when anything was done.
fn change_menu(menu: Option<&Menu>, if_stale: bool) -> Option<Status> {
    let report = match menu {
        Some(menu) if if_stale => match context_menu::is_current(menu) {
            Ok(true) => return None,
            Ok(false) => context_menu::install(menu),
            Err(e) => Err(e),
        },
        Some(menu) => context_menu::install(menu),
        None => context_menu::uninstall(ID),
    };
    Some(match report {
        Ok(report) => Status::Done(report.notes),
        Err(e) => Status::Failed(e.to_string()),
    })
}

/// The menu as installed for the user, if it is.
fn read() -> Result<Option<Menu>, context_menu::Error> {
    let path = definition()?;
    match std::fs::read_to_string(&path) {
        Ok(text) => Menu::from_definition(&text).map(Some),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(context_menu::Error::Io { path, source }),
    }
}

/// Where context-menu-rs keeps the menu's definition for the user.
fn definition() -> Result<PathBuf, context_menu::Error> {
    #[cfg(target_os = "linux")]
    {
        use context_menu::linux;
        Ok(linux::definition_path(ID, &linux::data_home()?))
    }
    #[cfg(windows)]
    {
        use context_menu::windows;
        Ok(windows::definition_path(ID, &windows::data_home()?))
    }
    #[cfg(target_os = "macos")]
    {
        use context_menu::macos;
        Ok(macos::definition_path(
            ID,
            &macos::Root::current()?.data_home,
        ))
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
    {
        Err(context_menu::Error::Unsupported)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use context_menu::{Chosen, Files};

    fn program() -> PathBuf {
        std::env::current_exe().expect("the test has a program")
    }

    fn chosen(name: &str, folder: bool) -> Chosen<'_> {
        Chosen { name, folder }
    }

    #[test]
    fn each_item_has_its_option() {
        for a in Action::ALL {
            assert_eq!(Action::of_option(&a.option()), Some(a));
        }
        assert_eq!(Action::of_option("extract-here"), None);
        assert_eq!(Action::of_option("--managed"), None);
        assert_eq!(Action::of_option("--"), None);
    }

    #[test]
    fn a_request_takes_every_path_after_the_option() {
        let rest = ["/home/u/a b.zip", "--managed", "-x.zip"].map(OsString::from);
        let r = Request::parse("--extract-here", rest).expect("an item's option");
        assert_eq!(r.action, Action::ExtractHere);
        assert_eq!(
            r.paths,
            ["/home/u/a b.zip", "--managed", "-x.zip"].map(PathBuf::from)
        );
        assert_eq!(Request::parse("--first-run", []), None);
    }

    #[test]
    fn the_first_run_puts_all_but_extract_in() {
        let c = Choice::default();
        assert!(!c.has(Action::Extract));
        assert_eq!(c.items.len(), Action::ALL.len() - 1);
        assert!(!c.submenu);
    }

    #[test]
    fn toggling_keeps_the_menu_order() {
        let mut c = Choice {
            items: vec![Action::Compress],
            submenu: false,
        };
        c.toggle(Action::ExtractHere);
        c.toggle(Action::Open);
        assert_eq!(
            c.items,
            [Action::ExtractHere, Action::Open, Action::Compress]
        );
        c.toggle(Action::Open);
        assert_eq!(c.items, [Action::ExtractHere, Action::Compress]);
    }

    #[test]
    fn the_menu_runs_packwing_with_each_option() {
        let menu = Choice::default().menu(&program()).expect("items");
        menu.validate().expect("a valid menu");
        assert_eq!(menu.id, "packwing");
        assert!(!menu.submenu);
        let ids: Vec<&str> = menu.items.iter().map(|i| i.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "extract-here",
                "extract-to-folder",
                "open",
                "compress-zip",
                "compress"
            ]
        );
        for item in &menu.items {
            assert_eq!(item.program, program());
            assert_eq!(item.args, [format!("--{}", item.id)]);
        }
        let submenu = Choice {
            submenu: true,
            ..Choice::default()
        };
        assert!(submenu.menu(&program()).is_some_and(|m| m.submenu));
        let none = Choice {
            items: Vec::new(),
            submenu: true,
        };
        assert_eq!(none.menu(&program()), None);
    }

    #[test]
    fn each_item_shows_for_what_it_takes() {
        let item = |a: Action| a.item(&program()).applies_to;
        let zip = [chosen("Báo cáo.ZIP", false)];
        let two = [chosen("a.zip", false), chosen("b.tar.zst", false)];
        let folder = [chosen("Photos", true)];
        let mixed = [chosen("a.zip", false), chosen("notes.txt", false)];

        let here = item(Action::ExtractHere);
        assert!(here.accepts(&zip) && here.accepts(&two));
        assert!(!here.accepts(&folder) && !here.accepts(&mixed));
        assert!(item(Action::ExtractToFolder).accepts(&two));

        for one in [Action::Extract, Action::Open] {
            assert!(item(one).accepts(&zip), "{one:?}");
            assert!(!item(one).accepts(&two), "{one:?}");
        }

        for pack in [Action::CompressZip, Action::Compress] {
            let applies = item(pack);
            assert_eq!(applies.files, Files::Any);
            assert!(applies.accepts(&folder) && applies.accepts(&mixed));
            assert!(applies.accepts(&[chosen("Photos", true), chosen("a.zip", false)]));
        }
    }

    #[test]
    fn the_installed_menu_says_what_was_chosen() {
        let chosen = Choice {
            items: vec![Action::ExtractToFolder, Action::Extract, Action::Compress],
            submenu: true,
        };
        let menu = chosen.menu(&program()).expect("items");
        let installed = Menu::from_definition(&menu.to_definition()).expect("it reads back");
        assert_eq!(Choice::of(&installed), chosen);

        // An item of a later packwing's, unknown here, is left out.
        let later = menu.item(Item::new("convert", "Convert…", program()));
        assert_eq!(Choice::of(&later), chosen);
    }

    /// Set in the run of the test below that has a home of its own.
    #[cfg(target_os = "linux")]
    const SCRATCH: &str = "PACKWING_TEST_SCRATCH_HOME";

    /// Puts the menu in, reads it back and takes it out, as packwing does,
    /// with context-menu-rs, in a home of the test's own: the test runs
    /// again in it, as a running one cannot change where its home is. Linux
    /// only: Windows keeps the menu in the user's registry, and macOS some
    /// of it in the user's preferences, whatever the home.
    #[cfg(target_os = "linux")]
    #[test]
    fn the_menu_goes_in_and_comes_out() {
        if std::env::var_os(SCRATCH).is_some() {
            in_and_out();
            return;
        }
        let home =
            std::env::temp_dir().join(format!("packwing-right-click-{}", std::process::id()));
        let ran = std::process::Command::new(program())
            .args([
                "--exact",
                "right_click::tests::the_menu_goes_in_and_comes_out",
            ])
            .env(SCRATCH, "1")
            .env("HOME", &home)
            .env("XDG_DATA_HOME", home.join("data"))
            .env("XDG_CONFIG_HOME", home.join("config"))
            .env("XDG_DATA_DIRS", home.join("shared"))
            .env("XDG_CONFIG_DIRS", home.join("etc"))
            .output()
            .expect("the test runs again");
        let written = home.join("data/context-menu-rs/packwing.menu");
        let _ = std::fs::remove_dir_all(&home);
        assert!(
            ran.status.success() && String::from_utf8_lossy(&ran.stdout).contains("1 passed"),
            "{}\n{}\n(the definition: {})",
            String::from_utf8_lossy(&ran.stdout),
            String::from_utf8_lossy(&ran.stderr),
            written.display()
        );
    }

    #[cfg(target_os = "linux")]
    fn in_and_out() {
        assert_eq!(read().expect("nothing there to read"), None);
        let chosen = Choice {
            items: vec![Action::ExtractHere, Action::Compress],
            submenu: true,
        };
        let menu = chosen.menu(&program());
        let came = change_menu(menu.as_ref(), false);
        assert!(matches!(came, Some(Status::Done(_))), "{came:?}");

        let installed = read().expect("it reads").expect("it is in");
        assert_eq!(Choice::of(&installed), chosen);
        assert!(installed.items.iter().all(|i| i.program == program()));
        assert_eq!(change_menu(menu.as_ref(), true), None, "it is up to date");

        let came = change_menu(None, false);
        assert!(matches!(came, Some(Status::Done(_))), "{came:?}");
        assert_eq!(read().expect("nothing there to read"), None);
    }
}
