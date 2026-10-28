//! Right-click menu: which of packwing's items the system's menu shows, and
//! whether they sit in a submenu of their own.

use gpui::{Context, Render, Window, prelude::*};

use super::{group, heading, section, setting};
use crate::widgets::{check_row, switch};

pub struct RightClick {
    /// Each item, and whether the menu shows it.
    items: Vec<(&'static str, bool)>,
    submenu: bool,
}

impl RightClick {
    pub fn new() -> Self {
        RightClick {
            items: vec![
                ("Extract here", true),
                ("Extract to “Archive name”", true),
                ("Extract…", false),
                ("Open with packwing", true),
                ("Compress to ZIP", true),
                ("Compress…", true),
            ],
            submenu: false,
        }
    }
}

impl Render for RightClick {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let items = self.items.iter().enumerate().map(|(i, &(name, on))| {
            check_row(("item", i), on, name)
                .text_size(gpui::px(15.))
                .py(gpui::px(6.))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.items[i].1 = !this.items[i].1;
                    cx.notify();
                }))
        });
        section()
            .child(
                group()
                    .child(heading(
                        "Items in the menu",
                        "They show when you right-click files, folders and archives. On macOS, \
                         System Settings asks you to allow them the first time.",
                    ))
                    .children(items),
            )
            .child(setting(
                "Put them in a submenu",
                "One packwing item, with these under it",
                switch("submenu", self.submenu).on_click(cx.listener(|this, _, _, cx| {
                    this.submenu = !this.submenu;
                    cx.notify();
                })),
            ))
    }
}
