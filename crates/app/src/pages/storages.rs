//! The storages: their list, the one chosen browsed in an [`Explorer`], and what
//! is known about what is chosen in it. A local folder is a storage: Add storage
//! asks which.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use gpui::{
    App, Context, Div, Entity, EventEmitter, Focusable as _, FontWeight, KeyDownEvent, MouseButton,
    PathPromptOptions, Pixels, Point, Render, Subscription, Task, Window, anchored, deferred, div,
    prelude::*, px, rgb,
};
use packwing_core::Values;
use packwing_i18n::tr;

use super::{explorer_info, panel, title};
use crate::Extensions;
use crate::colors::colors;
use crate::confirm::{How, confirmer};
use crate::explorer::{Explorer, Place};
use crate::input::TextInput;
use crate::prefs::{self, KeptPlace, KeptStorage};
use crate::widgets::{Fill, Size, button, float_shadow, icon, tile};

/// A storage's name, from its folder.
fn named(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    )
}

/// The storages there are before the user changes them: the home folder, and
/// the Downloads folder if there is one.
fn at_first() -> Vec<KeptStorage> {
    let home = std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" })
        .filter(|h| !h.is_empty())
        .map(PathBuf::from);
    let Some(home) = home else {
        return Vec::new();
    };
    let downloads = home.join("Downloads");
    let mut kept = vec![KeptStorage {
        name: tr!("Home"),
        place: KeptPlace::Local(home),
    }];
    if downloads.is_dir() {
        kept.push(KeptStorage {
            name: tr!("Downloads"),
            place: KeptPlace::Local(downloads),
        });
    }
    kept
}

/// The storage kinds and the like this build adds.
fn extensions(cx: &App) -> Extensions {
    cx.try_global::<Extensions>().cloned().unwrap_or_default()
}

fn kind_named(kind: &str, cx: &App) -> Option<std::sync::Arc<dyn packwing_core::StorageKind>> {
    extensions(cx)
        .storage_kinds
        .into_iter()
        .find(|k| k.id() == kind)
}

/// Add storage was pressed where there is more to choose from than a folder.
pub struct AddStorageAsked;

impl EventEmitter<AddStorageAsked> for StoragesPage {}

/// Edit was pressed for the storage at this index.
pub struct EditStorageAsked(pub usize);

impl EventEmitter<EditStorageAsked> for StoragesPage {}

/// With no use for this long, a storage that asks first asks again.
pub const ASKS_AGAIN_AFTER: Duration = Duration::from_secs(5 * 60);

/// What waits for it to be the user.
#[derive(Clone, Copy)]
enum Then {
    Edit(usize),
    Remove(usize),
}

pub struct StoragesPage {
    storages: Vec<KeptStorage>,
    chosen: Option<usize>,
    explorer: Entity<Explorer>,
    /// The menu of the storage at this index.
    menu: Option<(Point<Pixels>, usize)>,
    /// It was the user, for every storage that asks first, until they leave it
    /// [`ASKS_AGAIN_AFTER`]. A guard for a screen left unlocked, no more: the
    /// secrets stay in the keychain, for the user's other programs as before.
    confirmed: bool,
    last_used: Instant,
    /// A prompt or a check is under way.
    confirming: bool,
    /// Why it was not.
    refused: Option<String>,
    /// The login password, where packwing is the one to ask for it.
    password: Entity<TextInput>,
    then: Option<Then>,
    _watching: Option<Task<()>>,
    _explorer_changed: Subscription,
}

impl StoragesPage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let kept = prefs::prefs(cx);
        let storages = if kept.storages_kept {
            kept.storages
        } else {
            at_first()
        };
        let explorer = cx.new(Explorer::new);
        let changed = cx.observe(&explorer, |_, _, cx| cx.notify());
        let password = cx.new(|cx| TextInput::said(|| tr!("Password"), cx).masked());
        let mut page = StoragesPage {
            chosen: None,
            storages,
            explorer,
            menu: None,
            confirmed: false,
            last_used: Instant::now(),
            confirming: false,
            refused: None,
            password,
            then: None,
            _watching: None,
            _explorer_changed: changed,
        };
        page.choose(0, cx);
        page
    }

    /// The storages this build can reach, and the index of the one chosen. One
    /// that asks first is not among them until it has asked.
    pub fn places(&self, cx: &App) -> (Vec<Place>, usize) {
        let mut chosen = 0;
        let mut places = Vec::new();
        for (i, s) in self.storages.iter().enumerate() {
            if self.locked(s) {
                continue;
            }
            if let Some(place) = Self::place_of(s, cx) {
                if self.chosen == Some(i) {
                    chosen = places.len();
                }
                places.push(place);
            }
        }
        (places, chosen)
    }

    pub fn explorer(&self) -> &Entity<Explorer> {
        &self.explorer
    }

    /// The storage kept as `s`, if this build has its kind; none if not, and it
    /// stays on the list for a build that has.
    fn place_of(s: &KeptStorage, cx: &App) -> Option<Place> {
        match &s.place {
            KeptPlace::Local(path) => Some(Place::local(s.name.clone(), path)),
            KeptPlace::Other { kind, id, values } => {
                let k = kind_named(kind, cx)?;
                Some(Place {
                    name: s.name.clone(),
                    tag: k.tag(),
                    storage: k.open(id, values, None),
                })
            }
        }
    }

    fn locked(&self, s: &KeptStorage) -> bool {
        s.asks_first() && !self.confirmed
    }

    /// The storage chosen, if it waits to be told it is the user.
    fn chosen_locked(&self) -> Option<&KeptStorage> {
        self.chosen
            .and_then(|i| self.storages.get(i))
            .filter(|s| self.locked(s))
    }

    /// Browses the storage at `i`, if there is one; if it asks first, nothing
    /// of it is shown until it has.
    pub fn choose(&mut self, i: usize, cx: &mut Context<Self>) {
        let storage = self.storages.get(i);
        let locked = storage.is_some_and(|s| self.locked(s));
        let place = storage.and_then(|s| Self::place_of(s, cx));
        self.chosen = place.is_some().then_some(i);
        self.refused = None;
        self.then = None;
        let place = place.filter(|_| !locked);
        self.explorer.update(cx, |e, cx| e.show_place(place, cx));
        cx.notify();
    }

    /// Asks that it is the user, as this system does, for the storage chosen.
    fn start_asking(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match confirmer(cx).how() {
            Some(How::System) => self.confirm(cx),
            Some(How::Password) => window.focus(&self.password.focus_handle(cx), cx),
            None => {
                self.refused = Some(tr!("This computer has no way to ask that it is you."));
                cx.notify();
            }
        }
    }

    /// Asks the system, with the password typed where packwing asks for it.
    fn confirm(&mut self, cx: &mut Context<Self>) {
        if self.confirming {
            return;
        }
        let confirmer = confirmer(cx);
        let how = confirmer.how();
        let typed = self.password.read(cx).text().to_owned();
        if how == Some(How::Password) && typed.is_empty() {
            return;
        }
        self.password.update(cx, |p, cx| p.set_text("", 0..0, cx));
        self.confirming = true;
        self.refused = None;
        let reason = tr!("open a storage");
        let description = tr!("Open a storage in packwing");
        let work = cx.background_spawn(async move {
            match how {
                Some(How::Password) => confirmer.check_password(&typed),
                _ => confirmer.ask(&reason, &description),
            }
        });
        cx.spawn(async move |this, cx| {
            let answer = work.await;
            let _ = this.update(cx, |this, cx| this.answered(answer, cx));
        })
        .detach();
        cx.notify();
    }

    fn answered(&mut self, answer: Result<bool, String>, cx: &mut Context<Self>) {
        self.confirming = false;
        match answer {
            Ok(true) => {
                self.confirmed = true;
                self.used();
                self.watch(cx);
                let then = self.then.take();
                if let Some(i) = self.chosen {
                    self.choose(i, cx);
                }
                match then {
                    Some(Then::Edit(i)) => cx.emit(EditStorageAsked(i)),
                    Some(Then::Remove(i)) => self.remove(i, cx),
                    None => {}
                }
            }
            Ok(false) => {
                self.refused = Some(tr!("That was not it. It stays locked."));
            }
            Err(why) => {
                self.refused = Some(tr!("The system could not ask: {why}", why = why));
            }
        }
        cx.notify();
    }

    fn used(&mut self) {
        self.last_used = Instant::now();
    }

    /// Asks again once it has been left unused.
    fn watch(&mut self, cx: &mut Context<Self>) {
        self._watching = Some(cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_secs(15))
                    .await;
                let locked = this.update(cx, |this, cx| this.lock_if_left(cx));
                if !matches!(locked, Ok(false)) {
                    return;
                }
            }
        }));
    }

    /// Whether it was left unused long enough, and is locked now.
    fn lock_if_left(&mut self, cx: &mut Context<Self>) -> bool {
        if self.last_used.elapsed() < ASKS_AGAIN_AFTER {
            return false;
        }
        self.confirmed = false;
        if let Some(i) = self.chosen
            && self.storages.get(i).is_some_and(KeptStorage::asks_first)
        {
            self.choose(i, cx);
        }
        cx.notify();
        true
    }

    /// Does `then` on the storage at `i`, once it has asked if it asks first.
    fn when_confirmed(
        &mut self,
        i: usize,
        then: Then,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.storages.get(i).is_some_and(|s| self.locked(s)) {
            match then {
                Then::Edit(i) => cx.emit(EditStorageAsked(i)),
                Then::Remove(i) => self.remove(i, cx),
            }
            return;
        }
        self.choose(i, cx);
        self.then = Some(then);
        self.start_asking(window, cx);
    }

    /// The storage at `at`, to edit.
    pub fn storage(&self, at: usize) -> Option<&KeptStorage> {
        self.storages.get(at)
    }

    /// Changes the name and folder of the local storage at `at`.
    pub fn change_local(&mut self, at: usize, name: String, path: PathBuf, cx: &mut Context<Self>) {
        if let Some(s) = self.storages.get_mut(at) {
            s.name = name;
            s.place = KeptPlace::Local(path);
        }
        self.changed(at, cx);
    }

    /// Changes the settings and secrets of the storage of another kind at `at`.
    /// Its secrets are kept first: if they cannot be, nothing changes.
    pub fn change_other(
        &mut self,
        at: usize,
        name: String,
        new_values: Values,
        secrets: &Values,
        cx: &mut Context<Self>,
    ) -> std::io::Result<()> {
        let Some(KeptStorage {
            place: KeptPlace::Other { kind, id, .. },
            ..
        }) = self.storages.get(at)
        else {
            return Ok(());
        };
        if let Some(k) = kind_named(kind, cx) {
            k.keep_secrets(id, secrets)?;
        }
        if let Some(s) = self.storages.get_mut(at) {
            s.name = name;
            if let KeptPlace::Other { values, .. } = &mut s.place {
                *values = new_values;
            }
        }
        self.changed(at, cx);
        Ok(())
    }

    fn changed(&mut self, at: usize, cx: &mut Context<Self>) {
        self.keep(cx);
        if self.chosen == Some(at) {
            self.choose(at, cx);
        }
        cx.notify();
    }

    /// Browses `folder` of the storage `place` is, if it is still on the list.
    pub fn show(&mut self, place: &Place, folder: Vec<String>, cx: &mut Context<Self>) {
        let at = self.storages.iter().position(|s| {
            s.name == place.name && Self::place_of(s, cx).is_some_and(|p| p.tag == place.tag)
        });
        if let Some(at) = at {
            self.choose(at, cx);
            self.explorer.update(cx, |e, cx| e.open_folder(folder, cx));
        }
    }

    fn keep(&self, cx: &mut Context<Self>) {
        let storages = self.storages.clone();
        prefs::change(cx, |p| {
            p.storages = storages;
            p.storages_kept = true;
        });
    }

    /// Adds the folder at `path` as a storage, and browses it.
    pub fn add(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        let at = match self
            .storages
            .iter()
            .position(|s| s.place == KeptPlace::Local(path.clone()))
        {
            Some(at) => at,
            None => {
                self.storages.push(KeptStorage {
                    name: named(&path),
                    place: KeptPlace::Local(path),
                });
                self.keep(cx);
                self.storages.len() - 1
            }
        };
        self.choose(at, cx);
    }

    /// Takes the storage at `i` off the list. Its files stay where they are.
    fn remove(&mut self, i: usize, cx: &mut Context<Self>) {
        if i >= self.storages.len() {
            return;
        }
        let gone = self.storages.remove(i);
        if let KeptPlace::Other { kind, id, .. } = &gone.place
            && let Some(k) = kind_named(kind, cx)
        {
            k.forget_secrets(id);
        }
        self.keep(cx);
        self.choose(0, cx);
    }

    /// Adds a storage of a kind this build has, and browses it. Its secrets are
    /// kept first: if they cannot be, it is not added, and that is said.
    pub fn add_other(
        &mut self,
        kind: &str,
        name: String,
        values: Values,
        secrets: &Values,
        cx: &mut Context<Self>,
    ) -> std::io::Result<()> {
        let id = format!(
            "{:x}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_nanos())
        );
        if let Some(k) = kind_named(kind, cx) {
            k.keep_secrets(&id, secrets)?;
        }
        self.storages.push(KeptStorage {
            name,
            place: KeptPlace::Other {
                kind: kind.to_owned(),
                id,
                values,
            },
        });
        self.keep(cx);
        self.choose(self.storages.len() - 1, cx);
        Ok(())
    }

    /// How many are across a network.
    pub fn remote_count(&self) -> usize {
        self.storages
            .iter()
            .filter(|s| !matches!(s.place, KeptPlace::Local(_)))
            .count()
    }

    #[cfg(all(test, feature = "shots"))]
    pub fn count(&self) -> usize {
        self.storages.len()
    }

    #[cfg(all(test, feature = "shots"))]
    pub fn leave_for_long(&mut self, cx: &mut Context<Self>) -> bool {
        self.last_used = Instant::now() - ASKS_AGAIN_AFTER;
        self.lock_if_left(cx)
    }

    #[cfg(all(test, feature = "shots"))]
    pub fn type_password(&mut self, text: &str, cx: &mut Context<Self>) {
        self.password.update(cx, |p, cx| p.set_text(text, 0..0, cx));
        self.confirm(cx);
    }

    #[cfg(all(test, feature = "shots"))]
    pub fn is_chosen_locked(&self) -> bool {
        self.chosen_locked().is_some()
    }

    #[cfg(all(test, feature = "shots"))]
    pub fn press_remove(&mut self, i: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.when_confirmed(i, Then::Remove(i), window, cx);
    }

    #[cfg(all(test, feature = "shots"))]
    pub fn remove_last(&mut self, cx: &mut Context<Self>) {
        self.remove(self.storages.len().saturating_sub(1), cx);
    }

    /// Asks which folder to add.
    pub fn pick_folder(&mut self, cx: &mut Context<Self>) {
        let picked = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some(tr!("Add").into()),
        });
        cx.spawn(async move |this, cx| {
            if let Ok(Ok(Some(paths))) = picked.await
                && let Some(path) = paths.into_iter().next()
            {
                let _ = this.update(cx, |this, cx| this.add(path, cx));
            }
        })
        .detach();
    }

    fn list(&self, cx: &mut Context<Self>) -> Div {
        let c = colors();
        let rows = self.storages.iter().enumerate().map(|(i, s)| {
            let on = self.chosen == Some(i);
            let locked = self.locked(s);
            let (tag, tile_tint, where_it_is) = match &s.place {
                KeptPlace::Local(path) => (
                    crate::explorer::LOCAL_TAG,
                    c.plain_tag,
                    path.display().to_string(),
                ),
                KeptPlace::Other { kind, values, .. } => match kind_named(kind, cx) {
                    Some(k) => (k.tag(), c.blue_tag, k.describe(values)),
                    None => (
                        "?",
                        c.plain_tag,
                        tr!("This copy of packwing cannot open it."),
                    ),
                },
            };
            div()
                .id(("storage", i))
                .flex()
                .items_center()
                .gap(px(12.))
                .min_h(px(60.))
                .px(px(12.))
                .py(px(8.))
                .rounded(px(18.))
                .cursor_pointer()
                .when(on, |d| d.bg(rgb(c.chosen)))
                .when(!on, |d| d.hover(|s| s.bg(rgb(c.hover))))
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.choose(i, cx);
                    if this.chosen_locked().is_some() {
                        this.start_asking(window, cx);
                    }
                }))
                .on_mouse_down(
                    MouseButton::Right,
                    cx.listener(move |this, e: &gpui::MouseDownEvent, _, cx| {
                        this.menu = Some((e.position, i));
                        cx.notify();
                    }),
                )
                .child(
                    tile(40., 14., tile_tint)
                        .text_size(px(11.))
                        .font_weight(FontWeight::BOLD)
                        .child(tag),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .child(
                            div()
                                .truncate()
                                .text_size(px(15.))
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(s.name.clone()),
                        )
                        .child(
                            div()
                                .truncate()
                                .text_size(px(12.))
                                .text_color(rgb(c.muted))
                                .child(where_it_is),
                        ),
                )
                .when(locked, |d| d.child(icon("lock", 16., c.muted)))
        });
        panel()
            .w(px(250.))
            .flex_shrink_0()
            .flex()
            .flex_col()
            .gap(px(4.))
            .pt(px(18.))
            .px(px(10.))
            .pb(px(10.))
            .child(title(tr!("Storages"), 20.).px(px(12.)).pb(px(10.)))
            .child(
                div()
                    .id("storage-rows")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .children(rows),
            )
            .child(
                button("add-storage", tr!("Add storage"), Fill::Tonal, Size::Small)
                    .mt(px(8.))
                    .child(icon("plus", 16., c.on_tonal))
                    .on_click(cx.listener(|this, _, _, cx| {
                        if extensions(cx).storage_kinds.is_empty() {
                            this.pick_folder(cx);
                        } else {
                            cx.emit(AddStorageAsked);
                        }
                    })),
            )
    }

    /// What is shown of a storage that asks first, until it has.
    fn locked_view(&self, name: String, cx: &mut Context<Self>) -> Div {
        let c = colors();
        let typed = confirmer(cx).how() == Some(How::Password);
        div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(14.))
            .px(px(24.))
            .child(icon("lock", 40., c.muted))
            .child(title(name, 20.))
            .child(
                div()
                    .text_size(px(14.))
                    .text_color(rgb(c.muted))
                    .child(if typed {
                        tr!("Type the password you log in to this computer with to open it.")
                    } else {
                        tr!("Show this computer it is you to open it.")
                    }),
            )
            .when(typed, |d| {
                d.child(
                    div()
                        .w(px(320.))
                        .h(px(44.))
                        .flex()
                        .items_center()
                        .px(px(16.))
                        .rounded(px(16.))
                        .bg(rgb(c.field))
                        .text_size(px(15.))
                        .text_color(rgb(c.text))
                        .on_key_down(cx.listener(|this, e: &KeyDownEvent, _, cx| {
                            if e.keystroke.key == "enter" {
                                this.confirm(cx);
                                cx.stop_propagation();
                            }
                        }))
                        .child(self.password.clone()),
                )
            })
            .children(self.refused.clone().map(|why| {
                div()
                    .text_size(px(13.))
                    .text_color(rgb(c.bad_text))
                    .child(why)
            }))
            .child(
                button(
                    "storage-unlock",
                    if self.confirming {
                        tr!("Checking…")
                    } else {
                        tr!("Open")
                    },
                    Fill::Accent,
                    Size::Medium,
                )
                .on_click(cx.listener(|this, _, _, cx| this.confirm(cx))),
            )
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(rgb(c.muted))
                    .child(tr!("It asks again after 5 minutes left unused.")),
            )
    }

    /// The buttons that copy or move what is chosen.
    /// The menu of a storage in the list.
    fn storage_menu(
        &self,
        at: Point<Pixels>,
        i: usize,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let c = colors();
        // One of a kind this build does not have cannot be understood to edit.
        let editable = self.storages.get(i).is_some_and(|s| match &s.place {
            KeptPlace::Local(_) => true,
            KeptPlace::Other { kind, .. } => kind_named(kind, cx).is_some(),
        });
        deferred(
            anchored()
                .position(at)
                .snap_to_window_with_margin(px(8.))
                .child(
                    div()
                        .id("storage-menu")
                        .occlude()
                        .p(px(6.))
                        .rounded(px(16.))
                        .bg(rgb(c.surface))
                        .border_1()
                        .border_color(rgb(c.outline))
                        .shadow(float_shadow(8., 24.))
                        .text_size(px(14.))
                        .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                            this.menu = None;
                            cx.notify();
                        }))
                        .children(editable.then(|| {
                            div()
                                .id("storage-edit")
                                .h(px(40.))
                                .px(px(12.))
                                .flex()
                                .items_center()
                                .whitespace_nowrap()
                                .rounded(px(12.))
                                .cursor_pointer()
                                .hover(|s| s.bg(rgb(c.hover)))
                                .child(tr!("Edit…"))
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.menu = None;
                                    this.when_confirmed(i, Then::Edit(i), window, cx);
                                }))
                        }))
                        .child(
                            div()
                                .id("storage-remove")
                                .h(px(40.))
                                .px(px(12.))
                                .flex()
                                .items_center()
                                .whitespace_nowrap()
                                .rounded(px(12.))
                                .cursor_pointer()
                                .hover(|s| s.bg(rgb(c.hover)))
                                .child(tr!("Remove from this list"))
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.menu = None;
                                    this.when_confirmed(i, Then::Remove(i), window, cx);
                                })),
                        ),
                ),
        )
        .with_priority(1)
    }
}

impl Render for StoragesPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let list = self.list(cx);
        let info = explorer_info::info(&self.explorer, window, cx);
        let locked = self.chosen_locked().map(|s| s.name.clone());
        let browsing = match locked {
            Some(name) => self.locked_view(name, cx).into_any_element(),
            None => self.explorer.clone().into_any_element(),
        };
        let page = div()
            .size_full()
            .flex()
            .gap(px(18.))
            .on_mouse_move(cx.listener(|this, _: &gpui::MouseMoveEvent, _, _| this.used()))
            .on_scroll_wheel(cx.listener(|this, _: &gpui::ScrollWheelEvent, _, _| this.used()))
            .capture_key_down(cx.listener(|this, _: &KeyDownEvent, _, _| this.used()))
            .child(list)
            .child(
                panel()
                    .flex_1()
                    .min_w_0()
                    .p(px(10.))
                    .pt(px(14.))
                    .child(browsing),
            )
            .child(info);
        let menu = self.menu.map(|(at, i)| self.storage_menu(at, i, cx));
        page.children(menu)
    }
}
