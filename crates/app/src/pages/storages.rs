//! The storages: their list, the one chosen browsed in an [`Explorer`], and what
//! is known about what is chosen in it. A local folder is a storage: Add storage
//! asks which.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use gpui::{
    Context, Div, Entity, FontWeight, MouseButton, PathPromptOptions, Pixels, Point, Render,
    Subscription, Window, anchored, deferred, div, prelude::*, px, rgb,
};
use packwing_core::{Item, LocalFolder};
use packwing_i18n::{tr, tr_n};

use super::{panel, title};
use crate::colors::colors;
use crate::explorer::{Explorer, LOCAL_TAG, Place, is_archive, runs_when_opened};
use crate::model::{Kind, date, size, tag_of};
use crate::prefs::{self, KeptStorage};
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
        path: home,
    }];
    if downloads.is_dir() {
        kept.push(KeptStorage {
            name: tr!("Downloads"),
            path: downloads,
        });
    }
    kept
}

pub struct StoragesPage {
    storages: Vec<KeptStorage>,
    chosen: Option<usize>,
    explorer: Entity<Explorer>,
    /// The menu of the storage at this index.
    menu: Option<(Point<Pixels>, usize)>,
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
        let mut page = StoragesPage {
            chosen: None,
            storages,
            explorer,
            menu: None,
            _explorer_changed: changed,
        };
        page.choose(0, cx);
        page
    }

    /// The storages, and the index of the one chosen.
    pub fn places(&self) -> (Vec<Place>, usize) {
        let places = self
            .storages
            .iter()
            .map(|s| Place {
                name: s.name.clone(),
                storage: Arc::new(LocalFolder::new(&s.path)),
            })
            .collect();
        (places, self.chosen.unwrap_or(0))
    }

    pub fn explorer(&self) -> &Entity<Explorer> {
        &self.explorer
    }

    /// Browses the storage at `i`, if there is one.
    pub fn choose(&mut self, i: usize, cx: &mut Context<Self>) {
        let place = self.storages.get(i).map(|s| Place {
            name: s.name.clone(),
            storage: Arc::new(LocalFolder::new(&s.path)),
        });
        self.chosen = place.is_some().then_some(i);
        self.explorer.update(cx, |e, cx| e.show_place(place, cx));
        cx.notify();
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
        let at = match self.storages.iter().position(|s| s.path == path) {
            Some(at) => at,
            None => {
                self.storages.push(KeptStorage {
                    name: named(&path),
                    path,
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
        self.storages.remove(i);
        self.keep(cx);
        self.choose(0, cx);
    }

    /// Asks which folder to add.
    fn pick_folder(&mut self, cx: &mut Context<Self>) {
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
                .on_click(cx.listener(move |this, _, _, cx| this.choose(i, cx)))
                .on_mouse_down(
                    MouseButton::Right,
                    cx.listener(move |this, e: &gpui::MouseDownEvent, _, cx| {
                        this.menu = Some((e.position, i));
                        cx.notify();
                    }),
                )
                .child(
                    tile(40., 14., c.plain_tag)
                        .text_size(px(11.))
                        .font_weight(FontWeight::BOLD)
                        .child(LOCAL_TAG),
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
                                .child(s.path.display().to_string()),
                        ),
                )
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
                    .on_click(cx.listener(|this, _, _, cx| this.pick_folder(cx))),
            )
    }

    /// The buttons that copy or move what is chosen.
    fn transfers(&self) -> Div {
        let (copy, moved) = (self.explorer.clone(), self.explorer.clone());
        div()
            .flex()
            .gap(px(8.))
            .child(
                button("info-copy", tr!("Copy to…"), Fill::Outline, Size::Small)
                    .flex_1()
                    .on_click(move |_, _, cx| copy.update(cx, |e, cx| e.ask_transfer(false, cx))),
            )
            .child(
                button("info-move", tr!("Move to…"), Fill::Outline, Size::Small)
                    .flex_1()
                    .on_click(move |_, _, cx| moved.update(cx, |e, cx| e.ask_transfer(true, cx))),
            )
    }

    /// What is known about what is chosen, and what to do with it.
    fn info(&self, window: &Window, cx: &mut Context<Self>) -> Div {
        let c = colors();
        let selection = self.explorer.read(cx).selection();
        let body = panel()
            // A quarter of the window, as it grows; its buttons need the least.
            .w(window.viewport_size().width * 0.25)
            .min_w(px(280.))
            .flex_shrink_0()
            .flex()
            .flex_col()
            .gap(px(14.))
            .p(px(18.));
        match selection.as_slice() {
            [] => body.child(
                div()
                    .m_auto()
                    .p(px(16.))
                    .text_center()
                    .text_size(px(13.))
                    .text_color(rgb(c.muted))
                    .child(tr!("Choose a file to see its information here")),
            ),
            [one] => self.info_of(one, body, cx),
            many => {
                let explorer = self.explorer.clone();
                let delete = self.explorer.clone();
                let bytes: u64 = many.iter().map(|i| i.size).sum();
                body.child(title(
                    tr_n!(many.len() as u64, "{n} selected", "{n} selected"),
                    18.,
                ))
                .child(
                    div()
                        .text_size(px(13.))
                        .text_color(rgb(c.muted))
                        .child(tr!("Files in them come to {size}", size = size(bytes))),
                )
                .child(
                    button("info-compress", tr!("Compress…"), Fill::Tonal, Size::Small)
                        .on_click(move |_, _, cx| explorer.update(cx, |e, cx| e.compress(cx))),
                )
                .child(
                    button("info-delete", tr!("Delete…"), Fill::Outline, Size::Small)
                        .on_click(move |_, _, cx| delete.update(cx, |e, cx| e.ask_delete(cx))),
                )
                .child(self.transfers())
            }
        }
    }

    fn info_of(&self, item: &Item, body: Div, cx: &mut Context<Self>) -> Div {
        let c = colors();
        let explorer = self.explorer.read(cx);
        let kind = Kind::of(&item.name, item.dir);
        let row = |key: String, value: String| {
            div()
                .flex()
                .gap(px(12.))
                .text_size(px(13.))
                .child(
                    div()
                        .w(px(84.))
                        .flex_shrink_0()
                        .text_color(rgb(c.muted))
                        .child(key),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .font_weight(FontWeight::MEDIUM)
                        .child(value),
                )
        };
        let what = match () {
            _ if item.link && item.dir => tr!("Link to a folder"),
            _ if item.link => tr!("Link"),
            _ if item.dir => tr!("Folder"),
            _ => tr!("{tag} file", tag = tag_of(&item.name, kind)),
        };
        let counted = item.dir.then(|| explorer.count_of(&item.name)).flatten();
        let count_rows = counted.as_ref().map(|shown| {
            let n = shown.counted;
            let holds = tr!(
                "{files}, {folders}",
                files = tr_n!(n.files, "{n} file", "{n} files"),
                folders = tr_n!(n.folders, "{n} folder", "{n} folders")
            );
            let bytes = if shown.done {
                size(n.bytes)
            } else {
                tr!("{size} so far", size = size(n.bytes))
            };
            (bytes, holds, shown.unread)
        });
        let mut rows = vec![row(tr!("Kind"), what)];
        if !item.dir {
            rows.push(row(tr!("Size"), size(item.size)));
        }
        if let Some((bytes, holds, _)) = &count_rows {
            rows.push(row(tr!("Size"), bytes.clone()));
            rows.push(row(tr!("Holds"), holds.clone()));
        }
        if let Some(when) = item.modified {
            rows.push(row(tr!("Modified"), date(when)));
        }
        if let Some(mode) = item.mode {
            rows.push(row(tr!("Permissions"), permissions(mode)));
        }
        rows.push(row(tr!("Path"), explorer.shown_path_of(&item.name)));
        let unread = count_rows.as_ref().map_or(0, |(_, _, unread)| *unread);
        let handle = self.explorer.clone();
        let on = |f: fn(&mut Explorer, &Item, &mut Context<Explorer>)| {
            let (handle, item) = (handle.clone(), item.clone());
            move |_: &gpui::ClickEvent, _: &mut Window, cx: &mut gpui::App| {
                handle.update(cx, |e, cx| f(e, &item, cx));
            }
        };
        let open_label = if item.dir || is_archive(item) {
            tr!("Open")
        } else {
            tr!("View")
        };
        let mut actions = div().flex().flex_col().gap(px(8.)).child(
            button("info-open", open_label, Fill::Accent, Size::Small)
                .on_click(on(|e, i, cx| e.open(i, cx))),
        );
        if is_archive(item) {
            actions = actions.child(
                button("info-extract", tr!("Extract…"), Fill::Tonal, Size::Small)
                    .on_click(on(|e, i, cx| e.extract(i, cx))),
            );
        }
        if !item.dir && !is_archive(item) {
            actions = actions.child(
                button(
                    "info-edit",
                    Explorer::edit_label(cx),
                    Fill::Tonal,
                    Size::Small,
                )
                .on_click(on(|e, i, cx| e.edit(i, cx))),
            );
            if !runs_when_opened(item) {
                actions = actions.child(
                    button(
                        "info-app",
                        tr!("Open with default app"),
                        Fill::Outline,
                        Size::Small,
                    )
                    .on_click(on(|e, i, cx| e.open_with_system(i, cx))),
                );
            }
        }
        if item.dir && counted.is_none() {
            let (handle, name) = (self.explorer.clone(), item.name.clone());
            actions = actions.child(
                button(
                    "info-count",
                    tr!("Count size and files"),
                    Fill::Outline,
                    Size::Small,
                )
                .on_click(move |_, _, cx| {
                    handle.update(cx, |e, cx| e.count_folder(&name, cx));
                }),
            );
        }
        body.child(
            div()
                .flex()
                .items_center()
                .gap(px(12.))
                .child(
                    tile(48., 16., kind.tint(&c))
                        .text_size(px(12.))
                        .font_weight(FontWeight::BOLD)
                        .child(tag_of(&item.name, kind)),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .text_size(px(16.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(item.name.clone()),
                ),
        )
        .child(div().flex().flex_col().gap(px(7.)).children(rows))
        .when(unread > 0, |d| {
            d.child(
                div()
                    .text_size(px(13.))
                    .text_color(rgb(c.muted))
                    .child(tr_n!(
                        unread,
                        "{n} folder in it could not be read.",
                        "{n} folders in it could not be read."
                    )),
            )
        })
        .child(actions.child(self.transfers()))
    }

    /// The menu of a storage in the list.
    fn storage_menu(
        &self,
        at: Point<Pixels>,
        i: usize,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let c = colors();
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
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.menu = None;
                                    this.remove(i, cx);
                                })),
                        ),
                ),
        )
        .with_priority(1)
    }
}

/// Unix permissions as `ls` writes them.
fn permissions(mode: u32) -> String {
    (0..9)
        .map(|bit| {
            if mode & (0o400 >> bit) == 0 {
                '-'
            } else {
                ['r', 'w', 'x'][bit % 3]
            }
        })
        .collect()
}

impl Render for StoragesPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let list = self.list(cx);
        let info = self.info(window, cx);
        let menu = self.menu.map(|(at, i)| self.storage_menu(at, i, cx));
        div()
            .size_full()
            .flex()
            .gap(px(18.))
            .child(list)
            .child(
                panel()
                    .flex_1()
                    .min_w_0()
                    .p(px(10.))
                    .pt(px(14.))
                    .child(self.explorer.clone()),
            )
            .child(info)
            .children(menu)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permissions_read_as_ls_writes_them() {
        assert_eq!(permissions(0o644), "rw-r--r--");
        assert_eq!(permissions(0o755), "rwxr-xr-x");
        assert_eq!(permissions(0o000), "---------");
    }
}
