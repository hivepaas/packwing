//! New archive: its name and format, and under Advanced, how hard it is
//! squeezed, whether it is split into parts, and a password.

use gpui::{
    App, Context, Entity, EventEmitter, FontWeight, Render, Subscription, Window, div, prelude::*,
    px, relative, rgb,
};

use super::{DialogEvent, Job, Verb, footer, header};
use crate::colors::colors;
use crate::demo::{Selection, counted, size};
use crate::input::{Changed, TextInput};
use crate::widgets::{
    Chosen, Fill, Select, SelectLook, Size, Tone, button, check_row, icon, icon_button, legend,
    note, segment, segments, switch, tonal_tint,
};

/// A kind of archive New archive makes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Format {
    Zip,
    SevenZip,
    TarGz,
    TarXz,
    TarZst,
}

impl Format {
    pub const ALL: [Format; 5] = [
        Format::Zip,
        Format::SevenZip,
        Format::TarGz,
        Format::TarXz,
        Format::TarZst,
    ];

    /// What follows the archive's name, and what a [`Select`] of formats lists.
    pub fn ext(self) -> &'static str {
        match self {
            Format::Zip => ".zip",
            Format::SevenZip => ".7z",
            Format::TarGz => ".tar.gz",
            Format::TarXz => ".tar.xz",
            Format::TarZst => ".tar.zst",
        }
    }

    /// The format of a [`Select`]'s option.
    pub fn of_option(i: usize) -> Format {
        Format::ALL.get(i).copied().unwrap_or(Format::Zip)
    }

    /// Where its archives may not open without an archiver app, said under
    /// the name: none for ZIP, which opens on every phone and computer.
    pub fn caveat(self) -> Option<&'static str> {
        match self {
            Format::Zip => None,
            Format::SevenZip => Some(
                "Windows 11 and Linux open 7z on their own; macOS, phones and older Windows \
                 may need an archiver app. To share with anyone, pick .zip.",
            ),
            Format::TarGz | Format::TarXz => Some(
                "Linux, macOS and Windows 11 open it on their own; phones and older Windows \
                 may need an archiver app. To share with anyone, pick .zip.",
            ),
            Format::TarZst => Some(
                "Linux and Windows 11 open it on their own; macOS, phones and older Windows \
                 may need an archiver app. To share with anyone, pick .zip.",
            ),
        }
    }

    /// Whether its archives can have a password: tar has none.
    pub fn can_protect(self) -> bool {
        matches!(self, Format::Zip | Format::SevenZip)
    }
}

/// How hard the files are squeezed: higher makes a smaller archive, which
/// takes longer to make, not to open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Level {
    Lowest,
    Low,
    Medium,
    High,
    Highest,
}

pub(crate) const LEVELS: [(Level, &str); 5] = [
    (Level::Lowest, "Lowest"),
    (Level::Low, "Low"),
    (Level::Medium, "Medium"),
    (Level::High, "High"),
    (Level::Highest, "Highest"),
];

/// How the archive is split: the last takes a size typed in.
const SPLITS: [&str; 4] = [
    "Don’t split",
    "25 MB (email)",
    "4 GB (USB stick)",
    "Custom size…",
];
const CUSTOM: usize = 3;

/// The units a part's size is typed in, as the app counts sizes (`demo::size`).
const UNITS: [(&str, u64); 2] = [("MB", 1_000_000), ("GB", 1_000_000_000)];

/// The smallest part: a ZIP's parts are 64 KB or more, and parts this small
/// help nobody.
const SMALLEST_PART: u64 = 1_000_000;

/// The size of a part typed in, in bytes: a number of `unit`s, a comma
/// working as the decimal point, of [`SMALLEST_PART`] or more.
fn part_size(typed: &str, unit: u64) -> Option<u64> {
    let n: f64 = typed.trim().replace(',', ".").parse().ok()?;
    let bytes = n * unit as f64;
    (bytes.is_finite() && bytes >= SMALLEST_PART as f64).then(|| bytes.round() as u64)
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
    format: Entity<Select>,
    level: Level,
    advanced: bool,
    split: Entity<Select>,
    part: Entity<TextInput>,
    unit: Entity<Select>,
    protect: bool,
    password: Entity<TextInput>,
    hide_names: bool,
    /// Create was pressed with something missing: the notes say what.
    checked: bool,
    _follow: Vec<Subscription>,
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
        let format = cx.new(|_| Select::new(&Format::ALL.map(Format::ext), 0, SelectLook::Pill));
        let split = cx.new(|_| Select::new(&SPLITS, 0, SelectLook::Pill));
        let part = cx.new(|cx| TextInput::new("Size", cx));
        let unit = cx.new(|_| Select::new(&UNITS.map(|(u, _)| u), 0, SelectLook::Pill));
        let password = cx.new(|cx| TextInput::new("Password", cx).masked());
        // What is drawn follows what is typed and chosen: the extension, the
        // notes, the password's strength.
        let follow = vec![
            cx.subscribe(&format, |_, _, _: &Chosen, cx| cx.notify()),
            cx.subscribe(&split, |_, _, _: &Chosen, cx| cx.notify()),
            cx.subscribe(&part, |_, _, _: &Changed, cx| cx.notify()),
            cx.subscribe(&unit, |_, _, _: &Chosen, cx| cx.notify()),
            cx.subscribe(&password, |_, _, _: &Changed, cx| cx.notify()),
        ];
        CreateDialog {
            selection,
            name,
            format,
            level: Level::Medium,
            advanced: false,
            split,
            part,
            unit,
            protect: false,
            password,
            hide_names: true,
            checked: false,
            _follow: follow,
        }
    }

    #[cfg(all(test, feature = "shots"))]
    pub fn show_advanced(&mut self, cx: &mut Context<Self>) {
        self.advanced = true;
        cx.notify();
    }

    /// Fills the dialog as a user might: `format`, split into parts of a size
    /// typed in (`part`, maybe nothing yet), with a password asked for.
    #[cfg(all(test, feature = "shots"))]
    pub fn fill(&mut self, format: Format, part: &str, cx: &mut Context<Self>) {
        let i = Format::ALL.iter().position(|f| *f == format).unwrap_or(0);
        self.format.update(cx, |s, cx| s.choose(i, cx));
        self.split.update(cx, |s, cx| s.choose(CUSTOM, cx));
        self.part
            .update(cx, |t, cx| t.set_text(part, part.len()..part.len(), cx));
        self.protect = true;
        cx.notify();
    }

    #[cfg(all(test, feature = "shots"))]
    pub fn press_create(&mut self, cx: &mut Context<Self>) {
        self.start(cx);
    }

    fn format(&self, cx: &App) -> Format {
        Format::of_option(self.format.read(cx).chosen())
    }

    fn custom_split(&self, cx: &App) -> bool {
        self.split.read(cx).chosen() == CUSTOM
    }

    /// The size typed for a part, if it is one.
    fn part_size(&self, cx: &App) -> Option<u64> {
        let (_, unit) = UNITS[self.unit.read(cx).chosen().min(UNITS.len() - 1)];
        part_size(self.part.read(cx).text(), unit)
    }

    /// Whether a password is set: asked for, and the format takes one.
    fn protected(&self, cx: &App) -> bool {
        self.protect && self.format(cx).can_protect()
    }

    /// Whatever Advanced asks for is there: a part's size when one is to be
    /// typed, a password when one is asked for.
    fn ready(&self, cx: &App) -> bool {
        let part = !self.custom_split(cx) || self.part_size(cx).is_some();
        let password = !self.protected(cx) || !self.password.read(cx).text().is_empty();
        part && password
    }

    fn start(&mut self, cx: &mut Context<Self>) {
        if !self.ready(cx) {
            // What is missing is under Advanced, which may be hidden.
            self.checked = true;
            self.advanced = true;
            cx.notify();
            return;
        }
        let name = format!(
            "{}{}",
            self.name.read(cx).text().trim(),
            self.format(cx).ext()
        );
        cx.emit(DialogEvent::Start(Job {
            verb: Verb::Create,
            name: name.into(),
            to: Some("Documents".into()),
            files: self.selection.items,
            bytes: self.selection.size,
        }));
    }

    /// What Advanced has changed, said beside it while it is hidden: the
    /// level, the parts, a password.
    fn changed(&self, cx: &App) -> Vec<String> {
        let mut said = Vec::new();
        if self.level != Level::Medium
            && let Some((_, name)) = LEVELS.iter().find(|(l, _)| *l == self.level)
        {
            said.push(format!("{name} compression"));
        }
        match self.split.read(cx).chosen() {
            0 => {}
            1 => said.push("Parts of 25 MB".into()),
            2 => said.push("Parts of 4 GB".into()),
            _ => said.push(match self.part_size(cx) {
                Some(bytes) => format!("Parts of {}", size(bytes)),
                None => "Parts of a size to type".into(),
            }),
        }
        if self.protected(cx) {
            said.push("Password".into());
        }
        said
    }

    fn name_field(&self, cx: &App) -> impl IntoElement {
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
                    .gap(px(8.))
                    .h(px(52.))
                    .pl(px(18.))
                    .pr(px(6.))
                    .rounded(px(16.))
                    .bg(rgb(c.field))
                    .text_size(px(16.))
                    .child(self.name.clone())
                    .child(self.format.clone()),
            )
            .children(
                self.format(cx)
                    .caveat()
                    .map(|caveat| note(Tone::Warn).mt(px(4.)).child(caveat)),
            )
    }

    fn advanced_toggle(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let changed = if self.advanced {
            Vec::new()
        } else {
            self.changed(cx)
        };
        div()
            .flex()
            .items_center()
            .gap(px(10.))
            .child(
                div()
                    .id("advanced")
                    .flex_shrink_0()
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
            )
            .when(!changed.is_empty(), |d| {
                d.child(
                    div()
                        .min_w_0()
                        .truncate()
                        .text_size(px(13.))
                        .text_color(rgb(c.muted))
                        .child(changed.join(" · ")),
                )
            })
    }

    fn compression(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let levels = LEVELS.iter().map(|&(level, label)| {
            segment(label, label, level == self.level).on_click(cx.listener(
                move |this, _, _, cx| {
                    this.level = level;
                    cx.notify();
                },
            ))
        });
        let custom = self.custom_split(cx);
        let typed = !self.part.read(cx).text().trim().is_empty();
        let bad_part = custom && self.part_size(cx).is_none() && (typed || self.checked);
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
                            .child("Compression level"),
                    )
                    .child(segments(levels))
                    .child(div().text_size(px(13.)).text_color(rgb(c.muted)).child(
                        "Higher makes a smaller archive, which takes longer to make, not to open.",
                    )),
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
            .when(custom, |d| {
                d.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(div().flex_1().text_size(px(14.)).child("Each part"))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .w(px(120.))
                                .h(px(40.))
                                .px(px(14.))
                                .rounded_full()
                                .border_1()
                                .border_color(rgb(if bad_part { c.warn_text } else { c.outline }))
                                .bg(rgb(c.surface))
                                .text_size(px(14.))
                                .child(self.part.clone()),
                        )
                        .child(self.unit.clone()),
                )
            })
            .when(bad_part, |d| {
                d.child(
                    note(Tone::Warn).child("A part takes a size of 1 MB or more, such as 100 MB."),
                )
            })
    }

    fn protection(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let format = self.format(cx);
        let on = self.protected(cx);
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
                .when(typed.is_empty() && self.checked, |d| {
                    d.child(note(Tone::Warn).child("Type a password, or turn this off."))
                })
                .when(format == Format::SevenZip, |d| {
                    d.child(
                        check_row("hide-names", self.hide_names, "Hide file names too").on_click(
                            cx.listener(|this, _, _, cx| {
                                this.hide_names = !this.hide_names;
                                cx.notify();
                            }),
                        ),
                    )
                })
                .when(format == Format::Zip, |d| {
                    d.child(note(Tone::Warn).child(
                        "With a password, the built-in Windows and macOS tools can’t open this ZIP. \
                         The recipient needs an archiver app. File names also stay visible; \
                         pick .7z to hide them.",
                    ))
                })
            })
            .when(!format.can_protect(), |d| {
                d.child(note(Tone::Warn).child(
                    "tar archives can’t have a password. Pick .zip or .7z to protect it.",
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
                    .gap(px(14.))
                    .child(self.name_field(cx))
                    .child(self.advanced_toggle(cx))
                    .when(self.advanced, |d| {
                        d.child(self.compression(cx)).child(self.protection(cx))
                    }),
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
    fn every_format_but_zip_says_where_it_may_not_open() {
        for f in Format::ALL {
            assert_eq!(f.caveat().is_none(), f == Format::Zip, "{f:?}");
        }
    }

    #[test]
    fn tar_archives_take_no_password() {
        for f in Format::ALL {
            assert_eq!(f.can_protect(), !f.ext().starts_with(".tar"), "{f:?}");
        }
    }

    #[test]
    fn each_option_is_its_format() {
        for (i, f) in Format::ALL.into_iter().enumerate() {
            assert_eq!(Format::of_option(i), f);
        }
    }

    #[test]
    fn a_part_takes_a_size_of_a_megabyte_or_more() {
        let (mb, gb) = (UNITS[0].1, UNITS[1].1);
        assert_eq!(part_size("100", mb), Some(100_000_000));
        assert_eq!(part_size(" 1.5 ", gb), Some(1_500_000_000));
        assert_eq!(part_size("1,5", gb), Some(1_500_000_000));
        assert_eq!(part_size("1", mb), Some(1_000_000));
        for bad in ["", "0.5", "0", "-3", "abc", "NaN", "inf"] {
            assert_eq!(part_size(bad, mb), None, "{bad:?}");
        }
    }
}
