//! General: light or dark, and whether packwing is in the right-click menu.

use gpui::{Context, Entity, EventEmitter, Render, Window, prelude::*};

use super::{group, label, rows, section, setting};
use crate::right_click::RightClickMenu;
use crate::widgets::{segment, segments, switch};

/// Light, dark, or as the system is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Appearance {
    Light,
    Dark,
    #[default]
    System,
}

impl Appearance {
    pub fn name(self) -> &'static str {
        match self {
            Appearance::Light => "Light",
            Appearance::Dark => "Dark",
            Appearance::System => "System",
        }
    }

    /// The one after this, as the title bar's switch goes round them.
    pub fn next(self) -> Appearance {
        match self {
            Appearance::System => Appearance::Light,
            Appearance::Light => Appearance::Dark,
            Appearance::Dark => Appearance::System,
        }
    }
}

/// Another appearance was chosen.
pub struct AppearanceChosen(pub Appearance);

pub struct General {
    appearance: Appearance,
    /// The right-click menu: whether packwing is in it.
    menu: Entity<RightClickMenu>,
}

impl EventEmitter<AppearanceChosen> for General {}

impl General {
    pub fn new(
        appearance: Appearance,
        menu: Entity<RightClickMenu>,
        cx: &mut Context<Self>,
    ) -> Self {
        cx.observe(&menu, |_, _, cx| cx.notify()).detach();
        General { appearance, menu }
    }

    pub fn show_appearance(&mut self, a: Appearance, cx: &mut Context<Self>) {
        self.appearance = a;
        cx.notify();
    }
}

impl Render for General {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let looks = [Appearance::Light, Appearance::Dark, Appearance::System]
            .into_iter()
            .map(|a| {
                segment(a.name(), a.name(), a == self.appearance).on_click(cx.listener(
                    move |this, _, _, cx| {
                        this.appearance = a;
                        cx.emit(AppearanceChosen(a));
                        cx.notify();
                    },
                ))
            });
        let shown = self.menu.read(cx).shown();
        let menu = setting(
            "Show packwing in the right-click menu",
            "Extract and compress from any folder",
            switch("right-click", shown).on_click(cx.listener(move |this, _, _, cx| {
                this.menu.update(cx, |m, cx| m.show(!shown, cx));
            })),
        );
        section()
            .child(group().child(label("Appearance")).child(segments(looks)))
            .child(rows([menu]))
    }
}
