//! Right-click menu: whether packwing is in the system's menu, which of its
//! items, and whether they sit in a submenu of their own; and what came of
//! the last change.

use gpui::{Context, Entity, Render, Window, prelude::*, px};

use super::{explain, group, heading, label, section, setting};
use crate::right_click::{Action, RightClickMenu, Status};
use crate::widgets::{Tone, check_row, note, switch};

pub struct RightClick {
    menu: Entity<RightClickMenu>,
}

impl RightClick {
    pub fn new(menu: Entity<RightClickMenu>, cx: &mut Context<Self>) -> Self {
        cx.observe(&menu, |_, _, cx| cx.notify()).detach();
        RightClick { menu }
    }
}

impl Render for RightClick {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let m = self.menu.read(cx);
        let (shown, choice, status) = (m.shown(), m.choice().clone(), m.status().clone());
        let items = Action::ALL.into_iter().enumerate().map(|(i, a)| {
            check_row(("item", i), choice.has(a), a.label(choice.submenu))
                .text_size(px(15.))
                .py(px(6.))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.menu.update(cx, |m, cx| {
                        let mut c = m.choice().clone();
                        c.toggle(a);
                        m.choose(c, cx);
                    });
                }))
        });
        let items_are = if shown {
            "They show when you right-click files, folders and archives."
        } else {
            "packwing is not in the menu now: these go in when it is."
        };
        let came = match status {
            Status::Idle => None,
            Status::Working => Some(explain("Changing the menu…").into_any_element()),
            Status::Done(notes) if notes.is_empty() => None,
            Status::Done(notes) => Some(
                group()
                    .child(label("Good to know"))
                    .children(notes.into_iter().map(explain))
                    .into_any_element(),
            ),
            Status::Failed(why) => Some(
                note(Tone::Warn)
                    .child(format!("The menu could not be changed: {why}"))
                    .into_any_element(),
            ),
        };
        section()
            .child(setting(
                "Show packwing in the right-click menu",
                "Extract and compress from any folder",
                switch("shown", shown).on_click(cx.listener(move |this, _, _, cx| {
                    this.menu.update(cx, |m, cx| m.show(!shown, cx));
                })),
            ))
            .child(
                group()
                    .child(heading("Items in the menu", items_are))
                    .children(items),
            )
            .child(setting(
                "Put them in a submenu",
                "One packwing item, with these under it",
                switch("submenu", choice.submenu).on_click(cx.listener(|this, _, _, cx| {
                    this.menu.update(cx, |m, cx| {
                        let mut c = m.choice().clone();
                        c.submenu = !c.submenu;
                        m.choose(c, cx);
                    });
                })),
            ))
            .children(came)
    }
}
