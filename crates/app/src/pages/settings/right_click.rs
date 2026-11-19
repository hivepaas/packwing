//! Right-click menu: whether packwing is in the system's menu, which of its
//! items, and whether they sit in a submenu of their own; and what came of
//! the last change. In the Mac App Store's build, its items are macOS
//! services, which macOS puts in the menu and the user turns on and off in
//! System Settings: the section says where, and how.

use gpui::{
    AnyElement, Context, Div, Entity, FontWeight, Render, Window, div, prelude::*, relative, rgb,
};
use packwing_i18n::tr;

use super::{explain, group, heading, label, section, setting};
use crate::colors::{Tint, colors};
use crate::right_click::{Action, RightClickMenu, SERVICES, STORE_BUILD, Status};
use crate::text_size::ui;
use crate::widgets::{Fill, Size, Tone, button, check_row, icon, note, switch, tile};

/// System Settings' Keyboard, where its Keyboard Shortcuts hold the services.
const KEYBOARD_SETTINGS: &str = "x-apple.systempreferences:com.apple.Keyboard-Settings.extension";

/// A setting the system holds on, where a switch would be.
fn held_on() -> Div {
    let c = colors();
    div()
        .debug_selector(|| "submenu-held".to_owned())
        .flex()
        .items_center()
        .gap(ui(6.))
        .px(ui(12.))
        .py(ui(4.))
        .rounded_full()
        .bg(rgb(c.surface))
        .text_size(ui(13.))
        .font_weight(FontWeight::SEMIBOLD)
        .whitespace_nowrap()
        .child(icon("lock", 13., c.text))
        .child(tr!("On"))
}

pub struct RightClick {
    menu: Entity<RightClickMenu>,
}

impl RightClick {
    pub fn new(menu: Entity<RightClickMenu>, cx: &mut Context<Self>) -> Self {
        cx.observe(&menu, |_, _, cx| cx.notify()).detach();
        RightClick { menu }
    }
}

/// One of the steps to turn the services on and off: its number, then what to do.
fn step(n: usize, text: String) -> Div {
    let c = colors();
    div()
        .flex()
        .items_start()
        .gap(ui(12.))
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .size(ui(24.))
                .flex_shrink_0()
                .rounded_full()
                .bg(rgb(c.tonal))
                .text_color(rgb(c.on_tonal))
                .text_size(ui(13.))
                .font_weight(FontWeight::BOLD)
                .child(n.to_string()),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .pt(ui(2.))
                .text_size(ui(14.))
                .line_height(relative(1.45))
                .child(text),
        )
}

/// The Mac App Store's build: where macOS puts packwing's items, which they are, and
/// how to turn them on and off, which packwing may not do itself.
fn services() -> AnyElement {
    let c = colors();
    let about = div()
        .debug_selector(|| "services-note".to_owned())
        .flex()
        .items_center()
        .gap(ui(14.))
        .px(ui(18.))
        .py(ui(16.))
        .rounded(ui(22.))
        .bg(rgb(c.tonal))
        .text_color(rgb(c.on_tonal))
        .child(
            tile(
                44.,
                15.,
                Tint {
                    ground: c.surface,
                    ink: c.on_tonal,
                },
            )
            .flex()
            .items_center()
            .justify_center()
            .child(icon("sliders", 22., c.on_tonal)),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_size(ui(14.))
                .line_height(relative(1.5))
                .child(
                    div()
                        .text_size(ui(15.))
                        .font_weight(FontWeight::BOLD)
                        .child(tr!("packwing’s items are in Finder’s right-click menu")),
                )
                .child(tr!(
                    "macOS puts them there for an app from the App Store, at the bottom of the menu, under Quick Actions or Services. It shows them once packwing is installed, and you choose which ones stay in System Settings."
                )),
        );
    let items = SERVICES.iter().enumerate().map(|(i, s)| {
        div()
            .debug_selector(move || format!("service-{i}"))
            .flex()
            .items_center()
            .gap(ui(12.))
            .py(ui(4.))
            .child(icon(
                if s.archives { "extract" } else { "archive" },
                18.,
                c.muted,
            ))
            .child(div().flex_1().min_w_0().text_size(ui(15.)).child(s.title))
            .child(explain(if s.archives {
                tr!("For archives")
            } else {
                tr!("For files and folders")
            }))
    });
    let steps = [
        tr!("Open System Settings, then Keyboard: the button below takes you there."),
        tr!("Click Keyboard Shortcuts…, then Services in the list at the left."),
        tr!(
            "Open Files and Folders. Tick the packwing items you want in the menu, and untick the others. You can give each one a keyboard shortcut there too."
        ),
    ];
    section()
        .child(about)
        .child(
            group()
                .child(heading(
                    tr!("Items in the menu"),
                    tr!("As the menu names them, in English for now."),
                ))
                .children(items),
        )
        .child(
            group()
                .child(heading(
                    tr!("Turn items on or off"),
                    tr!("An app from the App Store may not change the menu itself: macOS keeps the choice."),
                ))
                .children(steps.into_iter().enumerate().map(|(i, t)| step(i + 1, t)))
                .child(
                    // In a row, so the button is as wide as its name.
                    div().pt(ui(4.)).flex().child(
                        button(
                            "services-settings",
                            tr!("Open Keyboard settings"),
                            Fill::Tonal,
                            Size::Medium,
                        )
                        .on_click(|_, _, cx| cx.open_url(KEYBOARD_SETTINGS)),
                    ),
                ),
        )
        .child(explain(tr!(
            "Items not in the menu yet? macOS finds them when packwing first opens: quit packwing and open it again, or log out and back in."
        )))
        .into_any_element()
}

impl Render for RightClick {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if STORE_BUILD {
            return services();
        }
        let m = self.menu.read(cx);
        let (shown, choice, status) = (m.shown(), m.choice().clone(), m.status().clone());
        let one_entry = m.one_entry();
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
                switch("shown", shown, tr!("Show packwing in the right-click menu")).on_click(cx.listener(move |this, _, _, cx| {
                    this.menu.update(cx, |m, cx| m.show(!shown, cx));
                })),
            ))
            .child(
                group()
                    .child(heading(tr!("Items in the menu"), items_are))
                    .children(items),
            )
            .child(if one_entry {
                // Windows groups a package's commands under one entry.
                setting(
                    tr!("Put them in a submenu"),
                    tr!("Windows 11 puts them under one Packwing item, in its menu and under Show more options"),
                    held_on(),
                )
            } else {
                setting(
                    tr!("Put them in a submenu"),
                    tr!("One packwing item, with these under it"),
                    switch("submenu", choice.submenu, tr!("Put them in a submenu")).on_click(cx.listener(|this, _, _, cx| {
                        this.menu.update(cx, |m, cx| {
                            let mut c = m.choice().clone();
                            c.submenu = !c.submenu;
                            m.choose(c, cx);
                        });
                    })),
                )
            })
            .children(came)
            .into_any_element()
    }
}
