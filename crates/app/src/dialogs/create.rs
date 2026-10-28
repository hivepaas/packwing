//! New archive: its name, what it is for (which picks the format), how hard
//! it is squeezed and split, and a password.

use gpui::{
    Context, Entity, EventEmitter, FontWeight, Render, Subscription, Window, div, prelude::*, px,
    relative, rgb,
};

use super::{DialogEvent, Job, Verb, footer, header};
use crate::colors::colors;
use crate::demo::{Selection, counted, size};
use crate::input::{Changed, TextInput};
use crate::widgets::{
    Fill, Select, SelectLook, Size, Tone, button, check_row, icon, icon_button, legend, note,
    segment, segments, switch, tonal_tint,
};

/// What the archive is for: each picks a format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Purpose {
    Share,
    Small,
    Backup,
}

pub(crate) struct PurposeInfo {
    pub purpose: Purpose,
    pub label: &'static str,
    pub hint: &'static str,
    pub format: &'static str,
    pub ext: &'static str,
}

pub(crate) const PURPOSES: [PurposeInfo; 3] = [
    PurposeInfo {
        purpose: Purpose::Share,
        label: "Share anywhere",
        hint: "Opens on every phone and computer",
        format: "ZIP",
        ext: ".zip",
    },
    PurposeInfo {
        purpose: Purpose::Small,
        label: "Smallest",
        hint: "Best for big folders and sensitive files",
        format: "7z",
        ext: ".7z",
    },
    PurposeInfo {
        purpose: Purpose::Backup,
        label: "Fast backup",
        hint: "Quickest to make, for your own copies",
        format: "tar.zst",
        ext: ".tar.zst",
    },
];

/// How hard the files are squeezed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Level {
    Faster,
    Balanced,
    Smaller,
}

pub(crate) const LEVELS: [(Level, &str); 3] = [
    (Level::Faster, "Faster"),
    (Level::Balanced, "Balanced"),
    (Level::Smaller, "Smaller"),
];

const SPLITS: [&str; 3] = ["Don’t split", "25 MB (email)", "4 GB (USB stick)"];

/// What the archive will take, as a share of the files: made up until the
/// sizes come from compressing a sample of them.
fn estimate(purpose: Purpose, level: Level) -> f64 {
    match (purpose, level) {
        (Purpose::Share, Level::Faster) => 0.61,
        (Purpose::Share, Level::Balanced) => 0.54,
        (Purpose::Share, Level::Smaller) => 0.53,
        (Purpose::Small, Level::Faster) => 0.46,
        (Purpose::Small, Level::Balanced) => 0.38,
        (Purpose::Small, Level::Smaller) => 0.34,
        (Purpose::Backup, Level::Faster) => 0.50,
        (Purpose::Backup, Level::Balanced) => 0.42,
        (Purpose::Backup, Level::Smaller) => 0.37,
    }
}

/// How hard a password is to guess, roughly: its length and the kinds of
/// characters in it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Strength {
    Weak,
    Fair,
    Strong,
}

fn strength(password: &str) -> Strength {
    let n = password.chars().count();
    let kinds = [
        password.chars().any(|c| c.is_lowercase()),
        password.chars().any(|c| c.is_uppercase()),
        password.chars().any(|c| c.is_numeric()),
        password.chars().any(|c| !c.is_alphanumeric()),
    ]
    .iter()
    .filter(|k| **k)
    .count();
    if n >= 16 || (n >= 12 && kinds >= 3) {
        Strength::Strong
    } else if n >= 8 {
        Strength::Fair
    } else {
        Strength::Weak
    }
}

pub struct CreateDialog {
    selection: Selection,
    name: Entity<TextInput>,
    purpose: Purpose,
    level: Level,
    advanced: bool,
    protect: bool,
    password: Entity<TextInput>,
    hide_names: bool,
    split: Entity<Select>,
    _typing: Subscription,
}

impl EventEmitter<DialogEvent> for CreateDialog {}

impl CreateDialog {
    pub fn new(selection: Selection, cx: &mut Context<Self>) -> Self {
        let name = cx.new(|cx| {
            let mut field = TextInput::new("Name", cx);
            let len = selection.name.len();
            field.set_text(selection.name, len..len, cx);
            field
        });
        let password = cx.new(|cx| TextInput::new("Password", cx).masked());
        // The strength under it follows what is typed.
        let typing = cx.subscribe(&password, |_, _, _: &Changed, cx| cx.notify());
        CreateDialog {
            selection,
            name,
            purpose: Purpose::Share,
            level: Level::Balanced,
            advanced: false,
            protect: true,
            password,
            hide_names: true,
            split: cx.new(|_| Select::new(&SPLITS, 0, SelectLook::Pill)),
            _typing: typing,
        }
    }

    #[cfg(all(test, feature = "shots"))]
    pub fn show_advanced(&mut self, cx: &mut Context<Self>) {
        self.advanced = true;
        cx.notify();
    }

    fn info(&self) -> &'static PurposeInfo {
        &PURPOSES[PURPOSES
            .iter()
            .position(|p| p.purpose == self.purpose)
            .unwrap_or(0)]
    }

    /// A password can be set: tar.zst has no encryption.
    fn can_protect(&self) -> bool {
        self.purpose != Purpose::Backup
    }

    fn start(&mut self, cx: &mut Context<Self>) {
        let name = format!("{}{}", self.name.read(cx).text().trim(), self.info().ext);
        cx.emit(DialogEvent::Start(Job {
            verb: Verb::Create,
            name: name.into(),
            to: Some("Documents".into()),
            files: self.selection.items,
            bytes: self.selection.size,
        }));
    }

    fn name_field(&self) -> impl IntoElement {
        let c = colors();
        div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .child(legend("Name"))
            .child(
                div()
                    .flex()
                    .items_center()
                    .h(px(52.))
                    .pl(px(18.))
                    .pr(px(8.))
                    .rounded(px(16.))
                    .bg(rgb(c.field))
                    .text_size(px(16.))
                    .child(self.name.clone())
                    .child(
                        div()
                            .flex_shrink_0()
                            .px(px(12.))
                            .py(px(6.))
                            .rounded(px(12.))
                            .bg(rgb(c.surface))
                            .text_size(px(14.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(self.info().ext),
                    ),
            )
    }

    fn purposes(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let cards = PURPOSES.iter().map(|p| {
            let on = p.purpose == self.purpose;
            let purpose = p.purpose;
            div()
                .id(p.label)
                .flex()
                .flex_col()
                .gap(px(6.))
                .min_h(px(104.))
                .p(px(14.))
                .rounded(px(20.))
                .border_2()
                .border_color(rgb(if on { c.accent } else { c.well }))
                .bg(rgb(if on { c.tonal } else { c.well }))
                .text_color(rgb(if on { c.on_tonal } else { c.text }))
                .cursor_pointer()
                .when(!on, |d| d.hover(|s| s.bg(rgb(c.well_hover))))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.purpose = purpose;
                    cx.notify();
                }))
                .child(
                    div()
                        .text_size(px(15.))
                        .font_weight(FontWeight::BOLD)
                        .child(p.label),
                )
                .child(
                    div()
                        .text_size(px(13.))
                        .line_height(relative(1.4))
                        .child(p.hint),
                )
                .child(div().flex_1())
                .child(
                    div()
                        .text_size(px(12.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .opacity(0.85)
                        .child(p.format),
                )
        });
        let est = (self.selection.size as f64 * estimate(self.purpose, self.level)) as u64;
        div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(legend("What’s it for?"))
            .child(div().grid().grid_cols(3).gap(px(8.)).children(cards))
            .child(
                div()
                    .mt(px(2.))
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .child(
                        div()
                            .px(px(12.))
                            .py(px(4.))
                            .rounded_full()
                            .bg(rgb(c.well))
                            .text_size(px(14.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(format!("≈ {}", size(est))),
                    )
                    .child(
                        div()
                            .id("advanced")
                            .px(px(4.))
                            .py(px(8.))
                            .text_size(px(14.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(c.accent))
                            .cursor_pointer()
                            .hover(|s| s.underline())
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.advanced = !this.advanced;
                                cx.notify();
                            }))
                            .child(if self.advanced {
                                "Hide advanced"
                            } else {
                                "Advanced options"
                            }),
                    ),
            )
    }

    fn advanced(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let levels = LEVELS.iter().map(|&(level, label)| {
            segment(label, label, level == self.level).on_click(cx.listener(
                move |this, _, _, cx| {
                    this.level = level;
                    cx.notify();
                },
            ))
        });
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .px(px(16.))
            .py(px(14.))
            .rounded(px(20.))
            .bg(rgb(c.well))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .child(
                        div()
                            .text_size(px(13.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Compression"),
                    )
                    .child(segments(levels)),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .child(
                        div()
                            .flex_1()
                            .text_size(px(14.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Split into parts"),
                    )
                    .child(self.split.clone()),
            )
    }

    fn protection(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let on = self.protect && self.can_protect();
        let typed = self.password.read(cx).text().to_string();
        let strength = (!typed.is_empty()).then(|| strength(&typed));
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .p(px(16.))
            .rounded(px(20.))
            .bg(rgb(c.well))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .child(icon("lock", 22., c.accent))
                    .child(
                        div()
                            .flex_1()
                            .text_size(px(15.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Protect with a password"),
                    )
                    .child(switch("protect", on).on_click(cx.listener(|this, _, _, cx| {
                        this.protect = !this.protect;
                        cx.notify();
                    }))),
            )
            .when(on, |d| {
                d.child(
                    div()
                        .flex()
                        .items_center()
                        .h(px(48.))
                        .px(px(16.))
                        .rounded(px(14.))
                        .bg(rgb(c.surface))
                        .text_size(px(16.))
                        .child(self.password.clone()),
                )
                .when_some(strength, |d, s| d.child(strength_bar(s)))
                .when(self.purpose == Purpose::Small, |d| {
                    d.child(
                        check_row("hide-names", self.hide_names, "Hide file names too").on_click(
                            cx.listener(|this, _, _, cx| {
                                this.hide_names = !this.hide_names;
                                cx.notify();
                            }),
                        ),
                    )
                })
                .when(self.purpose == Purpose::Share, |d| {
                    d.child(note(Tone::Warn).child(
                        "With a password, the built-in Windows and macOS tools can’t open this ZIP. \
                         The recipient needs an archiver app. File names also stay visible; \
                         choose “Smallest” to hide them.",
                    ))
                })
            })
            .when(!self.can_protect(), |d| {
                d.child(note(Tone::Warn).child(
                    "Fast backups can’t have a password. Pick another option to protect it.",
                ))
            })
    }
}

/// How strong the password is: a bar and a word.
fn strength_bar(s: Strength) -> impl IntoElement {
    let c = colors();
    let (part, color, word) = match s {
        Strength::Weak => (0.33, c.bad_text, "Weak"),
        Strength::Fair => (0.66, c.warn_text, "Fair"),
        Strength::Strong => (1., c.accent, "Strong"),
    };
    div()
        .flex()
        .items_center()
        .gap(px(10.))
        .text_size(px(13.))
        .child(
            div()
                .flex_1()
                .h(px(6.))
                .rounded_full()
                .bg(rgb(c.field))
                .child(
                    div()
                        .h_full()
                        .w(relative(part))
                        .rounded_full()
                        .bg(rgb(color)),
                ),
        )
        .child(word)
}

impl Render for CreateDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let s = self.selection;
        let detail = format!("{} · {}", counted(s.items, "item", "items"), size(s.size));
        let close = icon_button("close", "x", 44.)
            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close)));
        super::card()
            .p(px(0.))
            .gap(px(0.))
            .overflow_hidden()
            .child(div().px(px(28.)).pt(px(24.)).pb(px(8.)).child(header(
                "logo",
                tonal_tint(),
                "New archive",
                detail,
                Some(close),
            )))
            .child(
                div()
                    .id("create-body")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .px(px(28.))
                    .pt(px(12.))
                    .pb(px(8.))
                    .flex()
                    .flex_col()
                    .gap(px(18.))
                    .child(self.name_field())
                    .child(self.purposes(cx))
                    .when(self.advanced, |d| d.child(self.advanced(cx)))
                    .child(self.protection(cx)),
            )
            .child(
                footer()
                    .justify_end()
                    .px(px(28.))
                    .pt(px(16.))
                    .pb(px(24.))
                    .child(
                        button("cancel", "Cancel", Fill::Text, Size::Medium)
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close))),
                    )
                    .child(
                        button("create", "Create", Fill::Accent, Size::Medium)
                            .px(px(28.))
                            .on_click(cx.listener(|this, _, _, cx| this.start(cx))),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn longer_and_mixed_passwords_are_stronger() {
        assert_eq!(strength("abc"), Strength::Weak);
        assert_eq!(strength("abcdefgh"), Strength::Fair);
        assert_eq!(strength("Abcdefgh12!x"), Strength::Strong);
        assert_eq!(strength("correct horse battery"), Strength::Strong);
    }

    #[test]
    fn smaller_levels_estimate_smaller_archives() {
        for p in [Purpose::Share, Purpose::Small, Purpose::Backup] {
            assert!(estimate(p, Level::Faster) > estimate(p, Level::Balanced));
            assert!(estimate(p, Level::Balanced) > estimate(p, Level::Smaller));
        }
    }
}
