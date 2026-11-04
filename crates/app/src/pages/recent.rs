//! The archives opened before, the latest first: a tab of Browse.

use std::path::PathBuf;

use gpui::{
    ClipboardItem, Context, EventEmitter, FocusHandle, FontWeight, KeyDownEvent, MouseButton,
    MouseDownEvent, Pixels, Point, Render, Window, anchored, deferred, div, prelude::*, px, rgb,
};
use packwing_i18n::tr;

use super::panel;
use crate::colors::colors;
use crate::file_manager;
use crate::model::{Kind, date, size, tag_of};
use crate::prefs::Opened;
use crate::text_size::ui;
use crate::widgets::{menu_item, menu_panel, tile};

/// What was asked of one of them, by its path: what the page cannot do itself.
pub enum RecentEvent {
    Open(PathBuf),
    /// Open it, and offer to extract it.
    Extract(PathBuf),
    Sign(PathBuf),
    Checksums(PathBuf),
    /// Make an archive of what this one holds, in another format.
    Convert(PathBuf),
    /// Cut this one's file into parts.
    Split(PathBuf),
    /// Join the parts this one is of.
    Join(PathBuf),
    /// Take it out of the list; the file stays.
    Forget(PathBuf),
}

/// A row of the page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub path: PathBuf,
    pub name: String,
    pub location: String,
    /// Its size; none if it is not there any more.
    pub size: Option<u64>,
    pub opened: String,
}

impl Row {
    /// What `opened` is now, as its file says.
    pub fn of(opened: &Opened) -> Row {
        let path = &opened.path;
        Row {
            path: path.clone(),
            name: path.file_name().map_or_else(
                || path.display().to_string(),
                |n| n.to_string_lossy().into_owned(),
            ),
            location: path
                .parent()
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
            size: std::fs::metadata(path).ok().map(|m| m.len()),
            opened: date(opened.at),
        }
    }
}

pub struct RecentPage {
    archives: Vec<Row>,
    /// Where the right-click menu is open, and of which row.
    menu: Option<(Point<Pixels>, usize)>,
    /// The list's, for Escape to close the menu.
    focus: FocusHandle,
}

impl EventEmitter<RecentEvent> for RecentPage {}

impl RecentPage {
    pub fn new(archives: Vec<Row>, cx: &mut Context<Self>) -> Self {
        RecentPage {
            archives,
            menu: None,
            focus: cx.focus_handle(),
        }
    }

    fn on_key(&mut self, e: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        if e.keystroke.key == "escape" && self.menu.is_some() {
            self.menu = None;
            cx.notify();
            cx.stop_propagation();
        }
    }

    pub fn show(&mut self, archives: Vec<Row>, cx: &mut Context<Self>) {
        self.archives = archives;
        self.menu = None;
        cx.notify();
    }

    /// Opens the menu of the row at `i`, as the screenshots draw it.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn open_menu_for_shot(&mut self, at: Point<Pixels>, i: usize, cx: &mut Context<Self>) {
        self.menu = Some((at, i));
        cx.notify();
    }

    /// The right-click menu of the row at `i`, at `at`, as Browse's of an
    /// archive: what is asked of a file no longer there is only its path,
    /// and to take it out.
    fn menu(
        &self,
        at: Point<Pixels>,
        i: usize,
        cx: &mut Context<Self>,
    ) -> Option<gpui::AnyElement> {
        let row = self.archives.get(i)?;
        let c = colors();
        let line = || div().my(ui(4.)).mx(ui(10.)).h(ui(1.)).bg(rgb(c.field));
        let asking = |id: &'static str, label: String, asked: fn(PathBuf) -> RecentEvent| {
            let path = row.path.clone();
            menu_item(id, label, "").on_click(cx.listener(move |this, _, _, cx| {
                this.menu = None;
                cx.emit(asked(path.clone()));
                cx.notify();
            }))
        };
        let there = row.size.is_some();
        let cut = crate::explorer::is_cut_part_name(&row.name);
        let (shown, copied) = (row.path.clone(), row.path.display().to_string());
        let menu = menu_panel("recent-menu")
            .min_w(ui(230.))
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_mouse_down_out(cx.listener(|this, e: &MouseDownEvent, _, cx| {
                // A right click opens the next one.
                if e.button != MouseButton::Right {
                    this.menu = None;
                    cx.notify();
                }
            }))
            .when(there, |d| {
                d.child(asking("menu-open", tr!("Open"), RecentEvent::Open))
                    .child(asking(
                        "menu-extract",
                        tr!("Extract…"),
                        RecentEvent::Extract,
                    ))
                    .child(line())
                    .child(asking("menu-sign", tr!("Sign…"), RecentEvent::Sign))
                    .child(asking(
                        "menu-checksums",
                        tr!("Checksums…"),
                        RecentEvent::Checksums,
                    ))
                    .child(asking(
                        "menu-convert",
                        tr!("Convert…"),
                        RecentEvent::Convert,
                    ))
                    .child(asking(
                        "menu-split",
                        tr!("Split into parts…"),
                        RecentEvent::Split,
                    ))
                    .when(cut, |d| {
                        d.child(asking(
                            "menu-join",
                            tr!("Join the parts"),
                            RecentEvent::Join,
                        ))
                    })
                    .child(line())
                    .child(
                        menu_item("menu-file-manager", file_manager::show_label(), "").on_click(
                            cx.listener(move |this, _, _, cx| {
                                this.menu = None;
                                file_manager::show(&shown, cx);
                                cx.notify();
                            }),
                        ),
                    )
            })
            .child(
                menu_item("menu-copy-path", tr!("Copy path"), "").on_click(cx.listener(
                    move |this, _, _, cx| {
                        this.menu = None;
                        cx.write_to_clipboard(ClipboardItem::new_string(copied.clone()));
                        cx.notify();
                    },
                )),
            )
            .child(line())
            .child(asking(
                "menu-forget",
                tr!("Remove from Recent"),
                RecentEvent::Forget,
            ));
        Some(
            deferred(
                anchored()
                    .position(at)
                    .snap_to_window_with_margin(px(8.))
                    .child(menu),
            )
            .with_priority(1)
            .into_any_element(),
        )
    }
}

impl Render for RecentPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let menu = self.menu.and_then(|(at, i)| self.menu(at, i, cx));
        let menu_of = self.menu.map(|(_, i)| i);
        let rows = self.archives.iter().enumerate().map(|(i, r)| {
            let name = r.name.clone();
            let path = r.path.clone();
            div()
                .debug_selector(move || format!("recent: {name}"))
                .id(("recent", i))
                .flex()
                .items_center()
                .gap(ui(14.))
                .min_h(ui(64.))
                .px(ui(14.))
                .py(ui(8.))
                .rounded(ui(18.))
                .cursor_pointer()
                .when(menu_of == Some(i), |d| d.bg(rgb(c.chosen)))
                .when(menu_of != Some(i), |d| d.hover(|s| s.bg(rgb(c.hover))))
                .on_click(cx.listener(move |_, _, _, cx| cx.emit(RecentEvent::Open(path.clone()))))
                .on_mouse_down(
                    MouseButton::Right,
                    cx.listener(move |this, e: &MouseDownEvent, window, cx| {
                        // Whether it is still there, now: the menu offers what is.
                        if let Some(row) = this.archives.get_mut(i) {
                            row.size = std::fs::metadata(&row.path).ok().map(|m| m.len());
                        }
                        this.menu = Some((e.position, i));
                        window.focus(&this.focus, cx);
                        cx.stop_propagation();
                        cx.notify();
                    }),
                )
                .child(
                    tile(40., 14., Kind::Archive.tint(&c))
                        .text_size(ui(11.))
                        .font_weight(FontWeight::BOLD)
                        .child(tag_of(&r.name, Kind::Archive)),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .child(
                            div().flex().items_center().gap(ui(8.)).child(
                                div()
                                    .min_w_0()
                                    .truncate()
                                    .text_size(ui(15.))
                                    .font_weight(FontWeight::MEDIUM)
                                    .child(r.name.clone()),
                            ),
                        )
                        .child(div().text_size(ui(13.)).text_color(rgb(c.muted)).child(
                            match r.size {
                                Some(bytes) => format!("{} · {}", r.location, size(bytes)),
                                None => {
                                    tr!("{location} · not there any more", location = r.location)
                                }
                            },
                        )),
                )
                .child(
                    div()
                        .flex_shrink_0()
                        .text_size(ui(13.))
                        .text_color(rgb(c.muted))
                        .child(r.opened.clone()),
                )
        });
        div().size_full().flex().flex_col().children(menu).child(
            panel()
                .id("recent-list")
                .track_focus(&self.focus)
                .on_key_down(cx.listener(Self::on_key))
                .max_h_full()
                .min_h_0()
                .overflow_y_scroll()
                .flex()
                .flex_col()
                .p(ui(10.))
                .when(self.archives.is_empty(), |d| {
                    d.child(
                        div()
                            .p(ui(16.))
                            .text_size(ui(14.))
                            .text_color(rgb(c.muted))
                            .child(tr!("The archives you open will show here.")),
                    )
                })
                .children(rows),
        )
    }
}
