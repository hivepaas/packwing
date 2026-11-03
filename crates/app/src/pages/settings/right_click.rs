//! Right-click menu: whether packwing is in the system's menu, which of its
//! items, and whether they sit in a submenu of their own; and what came of
//! the last change.

use gpui::{Context, Entity, Render, Window, prelude::*};
use packwing_i18n::tr;

use super::{explain, group, heading, label, section, setting};
use crate::right_click::{Action, RightClickMenu, Status};
use crate::text_size::ui;
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
                .text_size(ui(15.))
                .py(ui(6.))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.menu.update(cx, |m, cx| {
                        let mut c = m.choice().clone();
                        c.toggle(a);
                        m.choose(c, cx);
                    });
                }))
        });
        let items_are = if shown {
            tr!("They show when you right-click files, folders and archives.")
        } else {
            tr!("packwing is not in the menu now: these go in when it is.")
        };
        let came = match status {
            Status::Idle => None,
            Status::Working => Some(explain(tr!("Changing the menu…")).into_any_element()),
            Status::Done(notes) if notes.is_empty() => None,
            Status::Done(notes) => Some(
                group()
                    .child(label(tr!("Good to know")))
                    .children(notes.into_iter().map(explain))
                    .into_any_element(),
            ),
            Status::Failed(why) => Some(
                note(Tone::Warn)
                    .child(tr!("The menu could not be changed: {why}", why = why))
                    .into_any_element(),
            ),
        };
        section()
            .child(setting(
                tr!("Show packwing in the right-click menu"),
                tr!("Extract and compress from any folder"),
                switch("shown", shown).on_click(cx.listener(move |this, _, _, cx| {
                    this.menu.update(cx, |m, cx| m.show(!shown, cx));
                })),
            ))
            .child(
                group()
                    .child(heading(tr!("Items in the menu"), items_are))
                    .children(items),
            )
            .child(setting(
                tr!("Put them in a submenu"),
                tr!("One packwing item, with these under it"),
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
