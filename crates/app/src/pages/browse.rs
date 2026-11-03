//! This computer's folders, from where it was left, and the archives opened before.

use std::path::{Component, Path, PathBuf};

use gpui::{
    Context, Div, Entity, FontWeight, Render, SharedString, Stateful, Subscription, Window, div,
    prelude::*, rgb,
};
use packwing_i18n::tr;

use super::recent::RecentPage;
use super::{explorer_info, panel};
use crate::colors::colors;
use crate::explorer::{Explorer, Place};
use crate::prefs;
use crate::text_size::ui;
use crate::widgets::icon;

/// What the page shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    Browse,
    Recent,
}

/// What `path` starts from: `/`, or on Windows its drive's top.
fn top_of(path: &Path) -> PathBuf {
    path.components()
        .take_while(|c| matches!(c, Component::Prefix(_) | Component::RootDir))
        .collect()
}

/// `path` as a folder from `top`, if it is under it and each name can be shown.
fn folder_from(top: &Path, path: &Path) -> Option<Vec<String>> {
    path.strip_prefix(top)
        .ok()?
        .components()
        .map(|c| match c {
            Component::Normal(name) => name.to_str().map(str::to_owned),
            _ => None,
        })
        .collect()
}

pub struct BrowsePage {
    tab: Tab,
    explorer: Entity<Explorer>,
    recent: Entity<RecentPage>,
    /// All of the disk it shows: from here it can go anywhere on it.
    top: PathBuf,
    _explorer_changed: Subscription,
}

impl BrowsePage {
    pub fn new(recent: Entity<RecentPage>, cx: &mut Context<Self>) -> Self {
        let kept = prefs::prefs(cx).browse_folder;
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        // Where it was left, if that is still there.
        let start = kept.filter(|f| f.is_dir()).unwrap_or_else(|| home.clone());
        let top = top_of(&start);
        let place = Place::local(tr!("This computer"), &top);
        let folder = folder_from(&top, &start).unwrap_or_default();
        let explorer = cx.new(|cx| {
            let mut explorer = Explorer::new(cx);
            explorer.show_place(Some(place), cx);
            explorer.open_folder(folder, cx);
            explorer
        });
        let changed = cx.observe(&explorer, |this: &mut Self, _, cx| {
            this.keep_folder(cx);
            cx.notify();
        });
        BrowsePage {
            tab: Tab::Browse,
            explorer,
            recent,
            top,
            _explorer_changed: changed,
        }
    }

    pub fn explorer(&self) -> &Entity<Explorer> {
        &self.explorer
    }

    pub fn show_tab(&mut self, tab: Tab, cx: &mut Context<Self>) {
        self.tab = tab;
        cx.notify();
    }

    /// The folder shown, on this computer.
    fn here(&self, cx: &Context<Self>) -> Option<PathBuf> {
        let explorer = self.explorer.read(cx);
        explorer.place()?.storage.local_path(explorer.folder())
    }

    /// Keeps the folder shown, for the next time the app opens.
    fn keep_folder(&mut self, cx: &mut Context<Self>) {
        let here = self.here(cx);
        if here.is_some() && prefs::prefs(cx).browse_folder != here {
            prefs::change(cx, |p| p.browse_folder = here);
        }
    }

    /// Shows the folder at `path`, wherever it is on this computer.
    pub fn go_to(&mut self, path: &Path, cx: &mut Context<Self>) {
        let top = top_of(path);
        // Another drive, on Windows.
        if top != self.top {
            self.top = top;
            let place = Place::local(tr!("This computer"), &self.top);
            self.explorer
                .update(cx, |e, cx| e.show_place(Some(place), cx));
        }
        if let Some(folder) = folder_from(&self.top, path) {
            self.explorer.update(cx, |e, cx| e.open_folder(folder, cx));
        }
        self.tab = Tab::Browse;
        cx.notify();
    }

    /// The folders a button leads to: those this system has.
    fn shortcuts() -> Vec<(&'static str, &'static str, String, PathBuf)> {
        let mut found = Vec::new();
        if let Some(path) = dirs::home_dir() {
            found.push(("go-home", "home", tr!("Home"), path));
        }
        if let Some(path) = dirs::desktop_dir().filter(|p| p.is_dir()) {
            found.push(("go-desktop", "desktop", tr!("Desktop"), path));
        }
        if let Some(path) = dirs::download_dir().filter(|p| p.is_dir()) {
            found.push(("go-downloads", "download", tr!("Downloads"), path));
        }
        found
    }
}

/// A tab, or a button that leads to a folder: `on` where it is now.
fn pill(
    id: &'static str,
    icon_name: &'static str,
    label: impl Into<SharedString>,
    on: bool,
) -> Stateful<Div> {
    let c = colors();
    div()
        .debug_selector(|| id.to_owned())
        .id(id)
        .flex()
        .items_center()
        .gap(ui(8.))
        .h(ui(40.))
        .px(ui(14.))
        .rounded_full()
        .cursor_pointer()
        .text_size(ui(14.))
        .when(on, |d| {
            d.bg(rgb(c.tonal))
                .text_color(rgb(c.on_tonal))
                .font_weight(FontWeight::SEMIBOLD)
        })
        .when(!on, |d| d.hover(|s| s.bg(rgb(c.hover))))
        .child(icon(icon_name, 18., if on { c.on_tonal } else { c.muted }))
        .child(label.into())
}

/// Says that files can be dropped on the window: it takes them anywhere, and
/// this lights up while they are dragged over it.
fn drop_hint(dragging: bool) -> Div {
    let c = colors();
    let ink = if dragging { c.accent } else { c.muted };
    div()
        .debug_selector(|| "drop-hint".to_owned())
        .flex()
        .items_center()
        .gap(ui(8.))
        .h(ui(40.))
        .px(ui(14.))
        .ml(ui(4.))
        .rounded_full()
        .border_1()
        .border_dashed()
        .border_color(rgb(ink))
        .text_size(ui(14.))
        .text_color(rgb(ink))
        .child(icon("drop", 18., ink))
        .child(tr!("Drop to open"))
}

impl Render for BrowsePage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tab = self.tab;
        let here = self.here(cx);
        let tabs = div()
            .flex()
            .items_center()
            .gap(ui(4.))
            .child(
                pill("tab-browse", "folder", tr!("Browse"), tab == Tab::Browse)
                    .on_click(cx.listener(|this, _, _, cx| this.show_tab(Tab::Browse, cx))),
            )
            .child(
                pill("tab-recent", "recent", tr!("Recent"), tab == Tab::Recent)
                    .on_click(cx.listener(|this, _, _, cx| this.show_tab(Tab::Recent, cx))),
            )
            .child(drop_hint(cx.has_active_drag()));
        let shortcuts = Self::shortcuts()
            .into_iter()
            .map(|(id, icon_name, label, path)| {
                let on = here.as_deref() == Some(path.as_path());
                pill(id, icon_name, label, on)
                    .on_click(cx.listener(move |this, _, _, cx| this.go_to(&path, cx)))
            });
        let top = div()
            .flex()
            .flex_wrap()
            .flex_shrink_0()
            .items_center()
            .justify_between()
            .gap(ui(8.))
            .px(ui(8.))
            .pt(ui(8.))
            .child(tabs)
            .when(tab == Tab::Browse, |d| {
                d.child(div().flex().items_center().gap(ui(4.)).children(shortcuts))
            });
        let body = div().flex_1().min_h_0();
        let body = match tab {
            Tab::Browse => body
                .flex()
                .gap(ui(18.))
                .child(
                    panel()
                        .flex_1()
                        .min_w_0()
                        .p(ui(10.))
                        .pt(ui(14.))
                        .child(self.explorer.clone()),
                )
                .child(explorer_info::info(&self.explorer, window, cx)),
            Tab::Recent => body.child(self.recent.clone()),
        };
        div()
            .size_full()
            .flex()
            .flex_col()
            .gap(ui(14.))
            .child(top)
            .child(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_folder_is_named_from_the_top_of_its_disk() {
        let path = Path::new("/home/an/other");
        let top = top_of(path);
        assert_eq!(top, Path::new("/"));
        assert_eq!(
            folder_from(&top, path),
            Some(vec!["home".to_owned(), "an".to_owned(), "other".to_owned()])
        );
        assert_eq!(folder_from(&top, Path::new("/")), Some(Vec::new()));
        assert_eq!(folder_from(Path::new("/home"), Path::new("/etc")), None);
    }
}
