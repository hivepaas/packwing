//! General: the language, light or dark and how each looks, the text's size,
//! how many cores to use, and the text editor.

use gpui::{
    Context, Div, ElementId, Entity, EventEmitter, FontWeight, Render, Stateful, Subscription,
    Window, div, prelude::*, rgb,
};
use packwing_i18n::tr;

use super::{explain, group, heading, label, section};
use crate::colors::{self, DarkStyle, LightStyle, Palette, colors};
use crate::input::{Changed, TextInput};
use crate::text_size::ui;
use crate::widgets::{Chosen, Select, SelectLook, segment, segments};

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

/// The thread counts offered on a computer of `cores`: all of them (0), then
/// 1 and each power of 2 below it, and `kept` too if it is none of those.
fn thread_choices(cores: u32, kept: u32) -> Vec<u32> {
    let mut choices: Vec<u32> = std::iter::once(0)
        .chain(std::iter::successors(Some(1u32), |n| n.checked_mul(2)).take_while(|n| *n < cores))
        .collect();
    if !choices.contains(&kept) {
        choices.push(kept);
        choices.sort_unstable();
    }
    choices
}

pub struct General {
    language: Entity<Select>,
    _language_chosen: Subscription,
    /// Body text's size: the system's, then [`text_size::CHOICES`].
    text_size: Entity<Select>,
    _text_size_chosen: Subscription,
    /// How many threads may be taken, as [`thread_choices`] offers them.
    threads: Entity<Select>,
    _threads_chosen: Subscription,
    /// The editors found on this system: (name, command).
    editors: Vec<(String, String)>,
    editor: Entity<Select>,
    /// The command typed for Custom.
    editor_command: Entity<TextInput>,
    _editor_chosen: [Subscription; 2],
    appearance: Appearance,
}

impl EventEmitter<AppearanceChosen> for General {}

impl General {
    pub fn new(appearance: Appearance, cx: &mut Context<Self>) -> Self {
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
        let system = crate::text_size::system(cx);
        let sizes: Vec<String> = std::iter::once(tr!(
            "The system’s ({size} px)",
            size = format!("{}", (system * 10.).round() / 10.)
        ))
        .chain(crate::text_size::CHOICES.iter().map(|s| format!("{s} px")))
        .collect();
        let chosen = crate::prefs::prefs(cx)
            .text_size
            .and_then(|s| crate::text_size::CHOICES.iter().position(|c| *c == s))
            .map_or(0, |at| at + 1);
        let text_size =
            cx.new(|_| Select::new(&sizes, chosen, SelectLook::Field).marked("text-size"));
        let text_size_chosen = cx.subscribe(&text_size, |_, select, _: &Chosen, cx| {
            let at = select.read(cx).chosen();
            let size = at
                .checked_sub(1)
                .and_then(|i| crate::text_size::CHOICES.get(i).copied());
            crate::prefs::change(cx, |p| p.text_size = size);
            // Every window draws at the new size now.
            cx.refresh_windows();
        });
        let cores = packwing_core::Threads::Auto.count();
        let kept_threads = crate::prefs::prefs(cx).threads;
        let choices = thread_choices(cores, kept_threads);
        let names: Vec<String> = choices
            .iter()
            .map(|n| match n {
                0 => tr!("All ({count})", count = cores),
                n => n.to_string(),
            })
            .collect();
        let chosen = choices.iter().position(|n| *n == kept_threads).unwrap_or(0);
        let threads = cx.new(|_| Select::new(&names, chosen, SelectLook::Field).marked("threads"));
        let threads_chosen = cx.subscribe(&threads, move |_, select, _: &Chosen, cx| {
            let count = choices.get(select.read(cx).chosen()).copied().unwrap_or(0);
            crate::prefs::change(cx, |p| p.threads = count);
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
            text_size,
            _text_size_chosen: text_size_chosen,
            threads,
            _threads_chosen: threads_chosen,
            editors,
            editor,
            editor_command,
            _editor_chosen: editor_chosen,
            appearance,
        }
    }

    pub fn show_appearance(&mut self, a: Appearance, cx: &mut Context<Self>) {
        self.appearance = a;
        cx.notify();
    }

    /// Kept, and every window drawn in it at once.
    fn choose_styles(light: LightStyle, dark: DarkStyle, cx: &mut Context<Self>) {
        crate::prefs::change(cx, |p| {
            p.light_style = light;
            p.dark_style = dark;
        });
        colors::set_styles(light, dark);
        cx.refresh_windows();
    }

    /// Light, dark or the system's, and under it a row of styles for each
    /// mode that can be on screen: light's when light is chosen, dark's when
    /// dark is, both when the system's is.
    fn appearance(&self, cx: &mut Context<Self>) -> Div {
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
        let kept = crate::prefs::prefs(cx);
        let (now_light, now_dark) = (kept.light_style, kept.dark_style);
        let light = LightStyle::ALL.into_iter().map(|style| {
            style_chip(
                ("light-style", style as usize),
                chip_name(style.name(), style == LightStyle::default()),
                style.palette(),
                style == now_light,
            )
            .on_click(cx.listener(move |_, _, _, cx| Self::choose_styles(style, now_dark, cx)))
        });
        let dark = DarkStyle::ALL.into_iter().map(|style| {
            style_chip(
                ("dark-style", style as usize),
                chip_name(style.name(), style == DarkStyle::default()),
                style.palette(),
                style == now_dark,
            )
            .on_click(cx.listener(move |_, _, _, cx| Self::choose_styles(now_light, style, cx)))
        });
        group()
            .child(heading(
                tr!("Appearance"),
                tr!("Light and dark each come in styles: the grounds change, the text and the accent stay."),
            ))
            .child(div().max_w(ui(480.)).child(segments(looks)))
            .when(self.appearance != Appearance::Dark, |d| {
                d.child(style_row(tr!("Light"))).child(chips().children(light))
            })
            .when(self.appearance != Appearance::Light, |d| {
                d.child(style_row(tr!("Dark"))).child(chips().children(dark))
            })
    }
}

fn chip_name(name: String, default: bool) -> String {
    if default {
        tr!("{name} (default)", name = name)
    } else {
        name
    }
}

/// The name over a row of styles.
fn style_row(name: String) -> Div {
    div()
        .pt(ui(2.))
        .text_size(ui(13.))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(rgb(colors().muted))
        .child(name)
}

fn chips() -> Div {
    div().flex().flex_row().flex_wrap().gap(ui(6.))
}

/// A style's chip: its name after a dot of its card on its page, so that it
/// shows how it looks before it is chosen.
fn style_chip(
    id: impl Into<ElementId>,
    name: String,
    palette: &Palette,
    on: bool,
) -> Stateful<Div> {
    let c = colors();
    let id = id.into();
    let selector = id.to_string();
    div()
        .debug_selector(|| selector)
        .id(id)
        .h(ui(32.))
        .pl(ui(7.))
        .pr(ui(13.))
        .rounded_full()
        .flex()
        .items_center()
        .gap(ui(8.))
        .cursor_pointer()
        .text_size(ui(13.))
        .font_weight(FontWeight::BOLD)
        .when(on, |d| d.bg(rgb(c.accent)).text_color(rgb(c.on_accent)))
        .when(!on, |d| {
            d.bg(rgb(c.field))
                .text_color(rgb(c.text))
                .hover(|s| s.bg(rgb(c.well_hover)))
        })
        .child(
            div()
                .size(ui(20.))
                .flex_shrink_0()
                .p(ui(1.))
                .rounded_full()
                .bg(rgb(palette.outline))
                .child(
                    div()
                        .size_full()
                        .rounded_full()
                        .border(ui(4.))
                        .border_color(rgb(palette.page))
                        .bg(rgb(palette.surface)),
                ),
        )
        .child(name)
}

impl Render for General {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let custom = self.editor.read(cx).chosen() == self.editors.len() + 1;
        // Two to a row, but the appearance, whose styles take a row of their own.
        let two = || div().grid().grid_cols(2).gap(ui(12.));
        section()
            .child(
                two()
                    .child(
                        group()
                            .child(label(tr!("Language")))
                            .child(self.language.clone()),
                    )
                    .child(
                        group()
                            .child(label(tr!("Text size")))
                            .child(self.text_size.clone()),
                    ),
            )
            .child(self.appearance(cx))
            .child(
                two()
                    .child(
                        group()
                            .child(heading(
                                tr!("Processor cores to use"),
                                tr!("More is faster; fewer leaves room for other work."),
                            ))
                            .child(self.threads.clone()),
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
                                        .h(ui(40.))
                                        .flex()
                                        .items_center()
                                        .px(ui(16.))
                                        .rounded_full()
                                        .bg(rgb(colors().field))
                                        .child(self.editor_command.clone()),
                                )
                                .child(explain(tr!("The file’s path is added at the end.")))
                            }),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_cores_come_first_then_powers_of_two_below_them() {
        assert_eq!(thread_choices(1, 0), [0]);
        assert_eq!(thread_choices(8, 0), [0, 1, 2, 4]);
        assert_eq!(thread_choices(12, 0), [0, 1, 2, 4, 8]);
        // What was kept is offered, though it is none of them.
        assert_eq!(thread_choices(8, 6), [0, 1, 2, 4, 6]);
        assert_eq!(thread_choices(4, 16), [0, 1, 2, 16]);
    }

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
