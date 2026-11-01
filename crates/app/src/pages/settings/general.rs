//! General: the language, light or dark, and whether packwing is in the right-click menu.

use gpui::{Context, Entity, EventEmitter, Render, Subscription, Window, div, prelude::*, px, rgb};
use packwing_i18n::tr;

use super::{explain, group, heading, label, rows, section, setting};
use crate::colors::colors;
use crate::input::{Changed, TextInput};
use crate::right_click::RightClickMenu;
use crate::widgets::{Chosen, Select, SelectLook, segment, segments, switch};

/// Light, dark, or as the system is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Appearance {
    Light,
    Dark,
    #[default]
    System,
}

impl Appearance {
    pub fn name(self) -> String {
        match self {
            Appearance::Light => tr!("Light"),
            Appearance::Dark => tr!("Dark"),
            Appearance::System => tr!("System"),
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

/// The language options: the system's, then each one there is.
fn languages() -> Vec<String> {
    let system = packwing_i18n::system_language();
    let name = packwing_i18n::LANGUAGES
        .iter()
        .find(|(tag, _)| *tag == system)
        .map_or("English", |(_, name)| name);
    std::iter::once(tr!("Same as system ({language})", language = name))
        .chain(
            packwing_i18n::LANGUAGES
                .iter()
                .map(|(_, name)| (*name).to_owned()),
        )
        .collect()
}

/// The tag of a language option: none for the system's, as [`languages`] has it.
fn language_of_option(option: usize) -> Option<&'static str> {
    let at = option.checked_sub(1)?;
    packwing_i18n::LANGUAGES.get(at).map(|(tag, _)| *tag)
}

/// The editors offered: the system's, those found here, and a command typed.
fn editor_options(found: &[(String, String)]) -> Vec<String> {
    std::iter::once(tr!("The system’s"))
        .chain(found.iter().map(|(name, _)| name.clone()))
        .chain(std::iter::once(tr!("Custom…")))
        .collect()
}

/// Which of them `kept` is: a command that is none of those found is typed.
fn editor_option(kept: Option<&str>, found: &[(String, String)]) -> usize {
    match kept {
        None => 0,
        Some(cmd) => found
            .iter()
            .position(|(_, c)| c == cmd)
            .map_or(found.len() + 1, |at| at + 1),
    }
}

pub struct General {
    language: Entity<Select>,
    _language_chosen: Subscription,
    /// The editors found on this system: (name, command).
    editors: Vec<(String, String)>,
    editor: Entity<Select>,
    /// The command typed for Custom.
    editor_command: Entity<TextInput>,
    _editor_chosen: [Subscription; 2],
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
        let kept = crate::prefs::prefs(cx).language;
        let chosen = packwing_i18n::LANGUAGES
            .iter()
            .position(|(tag, _)| kept.as_deref() == Some(*tag))
            .map_or(0, |at| at + 1);
        let language = cx.new(|_| Select::new(&languages(), chosen, SelectLook::Field));
        let language_chosen = cx.subscribe(&language, |_, select, _: &Chosen, cx| {
            let tag = language_of_option(select.read(cx).chosen());
            packwing_i18n::set_language(tag);
            crate::prefs::change(cx, |p| p.language = tag.map(str::to_owned));
            // Words said when drawn change now; a menu's options, made once,
            // when packwing next starts.
            cx.refresh_windows();
        });
        let editors = crate::editors::installed();
        let kept_editor = crate::prefs::prefs(cx).editor;
        let chosen_editor = editor_option(kept_editor.as_deref(), &editors);
        let editor =
            cx.new(|_| Select::new(&editor_options(&editors), chosen_editor, SelectLook::Field));
        let editor_command = cx.new(|cx| {
            let mut field = TextInput::said(|| tr!("e.g. code -n, or /path/to/editor --flag"), cx);
            if chosen_editor == editors.len() + 1 {
                let typed = kept_editor.clone().unwrap_or_default();
                field.set_text(&typed, typed.len()..typed.len(), cx);
            }
            field
        });
        let editor_chosen = [
            cx.subscribe(&editor, |this: &mut Self, select, _: &Chosen, cx| {
                let at = select.read(cx).chosen();
                let cmd = match at {
                    0 => None,
                    at if at <= this.editors.len() => Some(this.editors[at - 1].1.clone()),
                    _ => Some(this.editor_command.read(cx).text().trim().to_owned())
                        .filter(|c| !c.is_empty()),
                };
                crate::prefs::change(cx, |p| p.editor = cmd);
                cx.notify();
            }),
            cx.subscribe(&editor_command, |_, field, _: &Changed, cx| {
                let typed = field.read(cx).text().trim().to_owned();
                crate::prefs::change(cx, |p| p.editor = Some(typed).filter(|c| !c.is_empty()));
            }),
        ];
        General {
            language,
            _language_chosen: language_chosen,
            editors,
            editor,
            editor_command,
            _editor_chosen: editor_chosen,
            appearance,
            menu,
        }
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
        let custom = self.editor.read(cx).chosen() == self.editors.len() + 1;
        let shown = self.menu.read(cx).shown();
        let menu = setting(
            tr!("Show packwing in the right-click menu"),
            tr!("Extract and compress from any folder"),
            switch("right-click", shown).on_click(cx.listener(move |this, _, _, cx| {
                this.menu.update(cx, |m, cx| m.show(!shown, cx));
            })),
        );
        section()
            .child(
                div()
                    .grid()
                    .grid_cols(2)
                    .gap(px(12.))
                    .child(
                        group()
                            .child(label(tr!("Language")))
                            .child(self.language.clone()),
                    )
                    .child(
                        group()
                            .child(label(tr!("Appearance")))
                            .child(segments(looks)),
                    ),
            )
            .child(
                group()
                    .child(heading(
                        tr!("Text editor"),
                        tr!("What Edit opens a file of a storage in"),
                    ))
                    .child(self.editor.clone())
                    .when(custom, |d| {
                        d.child(
                            div()
                                .h(px(44.))
                                .flex()
                                .items_center()
                                .px(px(16.))
                                .rounded_full()
                                .bg(rgb(colors().field))
                                .child(self.editor_command.clone()),
                        )
                        .child(explain(tr!("The file’s path is added at the end.")))
                    }),
            )
            .child(rows([menu]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_option_is_the_system_and_the_others_each_language() {
        let options = languages();
        assert_eq!(options.len(), packwing_i18n::LANGUAGES.len() + 1);
        assert_eq!(language_of_option(0), None);
        for (i, (tag, name)) in packwing_i18n::LANGUAGES.iter().enumerate() {
            assert_eq!(language_of_option(i + 1), Some(*tag));
            assert_eq!(options[i + 1], *name);
        }
        assert_eq!(language_of_option(options.len()), None);
    }
}
