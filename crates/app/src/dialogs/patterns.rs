//! What goes in of what is chosen: patterns to leave out, and to take only, as
//! chips, with sets of the usual ones to add from a menu. A line says what they
//! come to while it is closed; open, it says how much they leave out.

use packwing_core::Filter;
use packwing_i18n::{tr, tr_n};

use gpui::{
    Context, Entity, EventEmitter, Focusable, FontWeight, KeyDownEvent, MouseButton, Render,
    SharedString, Subscription, Window, anchored, deferred, div, point, prelude::*, px, rgb,
};

use crate::colors::colors;
use crate::input::{Changed, TextInput};
use crate::model::{count, size};
use crate::text_size::ui;
use crate::widgets::{Fill, Size, Tone, button, icon, legend, menu_panel, note, segment, segments};

/// A set of patterns the menu adds at once.
struct Preset {
    id: &'static str,
    name: fn() -> String,
    patterns: &'static [&'static str],
}

/// What is usually left out of an archive to share.
const LEAVE_OUT: &[Preset] = &[
    Preset {
        id: "preset-vcs",
        name: || tr!("Version control"),
        patterns: &[".git/", ".svn/", ".hg/"],
    },
    Preset {
        id: "preset-builds",
        name: || tr!("Packages and builds"),
        patterns: &[
            "node_modules/",
            "target/",
            "build/",
            "dist/",
            "__pycache__/",
            ".venv/",
        ],
    },
    Preset {
        id: "preset-system",
        name: || tr!("System files"),
        patterns: &[".DS_Store", "Thumbs.db", "desktop.ini"],
    },
    Preset {
        id: "preset-temporary",
        name: || tr!("Temporary files"),
        patterns: &["*.tmp", "*.bak", "*.swp", "~$*"],
    },
    Preset {
        id: "preset-logs",
        name: || tr!("Logs"),
        patterns: &["*.log"],
    },
];

/// What is usually taken alone.
const TAKE_ONLY: &[Preset] = &[
    Preset {
        id: "preset-pictures",
        name: || tr!("Pictures"),
        patterns: &["*.jpg", "*.jpeg", "*.png", "*.gif", "*.webp", "*.heic"],
    },
    Preset {
        id: "preset-documents",
        name: || tr!("Documents"),
        patterns: &[
            "*.pdf", "*.doc", "*.docx", "*.xls", "*.xlsx", "*.ppt", "*.pptx", "*.odt", "*.txt",
            "*.md",
        ],
    },
    Preset {
        id: "preset-videos",
        name: || tr!("Videos"),
        patterns: &["*.mp4", "*.mov", "*.mkv", "*.avi"],
    },
    Preset {
        id: "preset-music",
        name: || tr!("Music"),
        patterns: &["*.mp3", "*.flac", "*.m4a", "*.wav", "*.ogg"],
    },
];

/// The patterns changed: [`Patterns::filter`] says what they take now.
pub struct PatternsChanged;

/// What goes in of what is chosen.
pub struct Patterns {
    exclude: Vec<String>,
    include: Vec<String>,
    /// The chips and the field are shown, not the line alone.
    open: bool,
    /// The tab: what is taken only, not what is left out.
    including: bool,
    field: Entity<TextInput>,
    /// Why what was typed is no pattern.
    problem: Option<String>,
    /// The menu of sets is open.
    presets: bool,
    /// What the patterns leave out, in files and bytes, once counted: none
    /// where there is nothing to count them on (converting).
    left_out: Option<(u64, u64)>,
    _follow: Subscription,
}

impl EventEmitter<PatternsChanged> for Patterns {}

impl Patterns {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let field = cx.new(|cx| TextInput::new(tr!("node_modules/, *.log, src/**/gen/"), cx));
        // What was wrong goes as it is typed again.
        let follow = cx.subscribe(&field, |this: &mut Self, _, _: &Changed, cx| {
            if this.problem.take().is_some() {
                cx.notify();
            }
        });
        Patterns {
            exclude: Vec::new(),
            include: Vec::new(),
            open: false,
            including: false,
            field,
            problem: None,
            presets: false,
            left_out: None,
            _follow: follow,
        }
    }

    /// What the patterns take.
    pub fn filter(&self) -> Filter {
        // Each was checked as it went in.
        Filter::new(&self.include, &self.exclude).unwrap_or_default()
    }

    /// Says that the patterns leave out `files` of `bytes`.
    pub fn set_left_out(&mut self, left_out: Option<(u64, u64)>, cx: &mut Context<Self>) {
        self.left_out = left_out;
        cx.notify();
    }

    /// Has these patterns, open on the tab of what is taken only if
    /// `including`, as a user would leave it; nothing is counted again.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn set(
        &mut self,
        include: &[&str],
        exclude: &[&str],
        including: bool,
        cx: &mut Context<Self>,
    ) {
        self.include = include.iter().map(|p| (*p).to_owned()).collect();
        self.exclude = exclude.iter().map(|p| (*p).to_owned()).collect();
        self.open = true;
        self.including = including;
        cx.notify();
    }

    /// Opens the menu of the usual patterns.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn open_presets(&mut self, cx: &mut Context<Self>) {
        self.presets = true;
        cx.notify();
    }

    /// The patterns: to take only, and to leave out.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn patterns(&self) -> (Vec<String>, Vec<String>) {
        (self.include.clone(), self.exclude.clone())
    }

    fn list(&mut self) -> &mut Vec<String> {
        if self.including {
            &mut self.include
        } else {
            &mut self.exclude
        }
    }

    /// Puts in what is typed, a pattern a line, and empties the field; or says
    /// why it is no pattern.
    fn add_typed(&mut self, cx: &mut Context<Self>) {
        let typed = self.field.read(cx).text().to_owned();
        let lines: Vec<&str> = typed
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect();
        if let Some(e) = lines.iter().find_map(|l| Filter::check(l).err()) {
            self.problem = Some(e.to_string());
            return cx.notify();
        }
        if lines.is_empty() {
            return;
        }
        let added = self.add(&lines);
        self.field.update(cx, |f, cx| f.set_text("", 0..0, cx));
        if added {
            cx.emit(PatternsChanged);
        }
        cx.notify();
    }

    /// Puts in those of `patterns` not there yet: whether any was.
    fn add(&mut self, patterns: &[&str]) -> bool {
        let list = self.list();
        let before = list.len();
        for p in patterns {
            if !list.iter().any(|q| q == p) {
                list.push((*p).to_owned());
            }
        }
        list.len() > before
    }

    fn remove(&mut self, including: bool, at: usize, cx: &mut Context<Self>) {
        let list = if including {
            &mut self.include
        } else {
            &mut self.exclude
        };
        if at < list.len() {
            list.remove(at);
            cx.emit(PatternsChanged);
            cx.notify();
        }
    }

    fn add_preset(&mut self, preset: &Preset, cx: &mut Context<Self>) {
        self.presets = false;
        if self.add(preset.patterns) {
            cx.emit(PatternsChanged);
        }
        cx.notify();
    }

    fn on_key(&mut self, e: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        match e.keystroke.key.as_str() {
            "enter" => self.add_typed(cx),
            // The last chip goes, as in a field of tags, when there is nothing
            // to take back in the field.
            "backspace" if self.field.read(cx).text().is_empty() => {
                let including = self.including;
                let last = self.list().len().checked_sub(1);
                match last {
                    Some(at) => self.remove(including, at, cx),
                    None => return,
                }
            }
            _ => return,
        }
        cx.stop_propagation();
    }

    /// The line beside the heading: what the patterns come to.
    fn said(&self) -> String {
        let mut said = Vec::new();
        if !self.include.is_empty() {
            said.push(tr!("Only {list}", list = self.include.join(", ")));
        }
        if !self.exclude.is_empty() {
            said.push(tr!("Not {list}", list = self.exclude.join(", ")));
        }
        if said.is_empty() {
            return tr!("Everything chosen");
        }
        if let Some((files, _)) = self.left_out.filter(|(files, _)| *files > 0) {
            said.push(tr_n!(
                files,
                "{n} file left out",
                "{n} files left out",
                n = count(files)
            ));
        }
        said.join(" · ")
    }

    fn chip(
        &self,
        including: bool,
        at: usize,
        text: &str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let c = colors();
        let shown = text.to_owned();
        div()
            .debug_selector(move || format!("pattern: {shown}"))
            .flex()
            .flex_shrink_0()
            .items_center()
            .gap(ui(4.))
            .h(ui(30.))
            .pl(ui(12.))
            .pr(ui(4.))
            .rounded_full()
            .bg(rgb(if including { c.tonal } else { c.surface }))
            .border_1()
            .border_color(rgb(if including { c.tonal } else { c.outline }))
            .text_color(rgb(if including { c.on_tonal } else { c.text }))
            .text_size(ui(13.))
            .font_weight(FontWeight::MEDIUM)
            .whitespace_nowrap()
            .child(SharedString::from(text.to_owned()))
            .child(
                div()
                    .id(("remove-pattern", at))
                    .size(ui(22.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_full()
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(c.hover)))
                    .on_click(cx.listener(move |this, _, _, cx| this.remove(including, at, cx)))
                    .child(icon("x", 12., c.muted)),
            )
    }

    fn presets_menu(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let sets = if self.including { TAKE_ONLY } else { LEAVE_OUT };
        let items = sets.iter().enumerate().map(|(i, preset)| {
            let id = preset.id;
            div()
                .debug_selector(move || id.to_owned())
                .id(id)
                .px(ui(12.))
                .py(ui(8.))
                .flex()
                .flex_col()
                .gap(ui(2.))
                .rounded(ui(12.))
                .cursor_pointer()
                .hover(|s| s.bg(rgb(c.hover)))
                .child(div().whitespace_nowrap().child((preset.name)()))
                .child(
                    div()
                        .text_size(ui(12.))
                        .text_color(rgb(c.muted))
                        .child(preset.patterns.join("  ")),
                )
                .on_click(cx.listener(move |this, _, _, cx| {
                    let sets = if this.including { TAKE_ONLY } else { LEAVE_OUT };
                    if let Some(preset) = sets.get(i) {
                        this.add_preset(preset, cx);
                    }
                }))
        });
        let size = window.viewport_size();
        let catcher = div()
            .id("presets-catcher")
            .w(size.width)
            .h(size.height)
            .occlude()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.presets = false;
                    cx.notify();
                }),
            );
        div()
            .child(
                deferred(anchored().position(point(px(0.), px(0.))).child(catcher))
                    .with_priority(2),
            )
            .child(
                deferred(
                    anchored()
                        .offset(point(px(0.), px(4.)))
                        .snap_to_window_with_margin(px(8.))
                        .child(menu_panel("presets").min_w(ui(260.)).children(items)),
                )
                .with_priority(3),
            )
    }

    fn editor(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let including = self.including;
        let shown = if including {
            &self.include
        } else {
            &self.exclude
        };
        let chips: Vec<_> = shown
            .iter()
            .enumerate()
            .map(|(at, p)| self.chip(including, at, p, cx).into_any_element())
            .collect();
        let tab = |id: &'static str, label: String, on: bool, n: usize| {
            let label = if n == 0 {
                label
            } else {
                format!("{label} ({n})")
            };
            segment(id, label, on)
        };
        let left = match self.left_out {
            Some((0, _)) if !(self.include.is_empty() && self.exclude.is_empty()) => {
                Some(tr!("They leave nothing out."))
            }
            Some((files, bytes)) if files > 0 => Some(tr_n!(
                files,
                "They leave out {n} file, {size}.",
                "They leave out {n} files, {size}.",
                n = count(files),
                size = size(bytes)
            )),
            _ => None,
        };
        div()
            .flex()
            .flex_col()
            .gap(ui(8.))
            .child(segments([
                tab(
                    "leave-out",
                    tr!("Leave out"),
                    !including,
                    self.exclude.len(),
                )
                .on_click(cx.listener(|this, _, _, cx| {
                    this.including = false;
                    this.presets = false;
                    cx.notify();
                })),
                tab("take-only", tr!("Take only"), including, self.include.len()).on_click(
                    cx.listener(|this, _, _, cx| {
                        this.including = true;
                        this.presets = false;
                        cx.notify();
                    }),
                ),
            ]))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(ui(6.))
                    .p(ui(6.))
                    .rounded(ui(16.))
                    .bg(rgb(c.well))
                    .children(chips)
                    .child(
                        div()
                            .debug_selector(|| "pattern-field".to_owned())
                            .flex_1()
                            .min_w(ui(180.))
                            .on_key_down(cx.listener(Self::on_key))
                            .child(self.field.clone()),
                    ),
            )
            .children(self.problem.clone().map(|p| note(Tone::Bad).child(p)))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(ui(10.))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(
                                button("presets", tr!("Add the usual"), Fill::Outline, Size::Medium)
                                    .child(icon("chevron-down", 14., c.text))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.presets = !this.presets;
                                        cx.notify();
                                    })),
                            )
                            .children(self.presets.then(|| self.presets_menu(window, cx))),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .text_size(ui(13.))
                            .text_color(rgb(c.muted))
                            .child(if including {
                                tr!("Only what these name goes in, and the folders it is in.")
                            } else {
                                tr!("A name is anywhere; a / at its end names a folder; ** is any folders.")
                            }),
                    ),
            )
            .children(left.map(|l| {
                div()
                    .debug_selector(|| "left-out".to_owned())
                    .text_size(ui(13.))
                    .text_color(rgb(c.muted))
                    .child(l)
            }))
    }
}

impl Render for Patterns {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let open = self.open;
        div()
            .flex()
            .flex_col()
            .gap(ui(10.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(ui(8.))
                    .child(
                        div().flex_1().min_w_0().child(legend(tr!("Filter"))).child(
                            div()
                                .debug_selector(|| "patterns-said".to_owned())
                                .truncate()
                                .text_size(ui(13.))
                                .text_color(rgb(c.muted))
                                .child(self.said()),
                        ),
                    )
                    .child(
                        button(
                            "edit-patterns",
                            if open { tr!("Done") } else { tr!("Edit…") },
                            Fill::Outline,
                            Size::Medium,
                        )
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.open = !this.open;
                            this.presets = false;
                            if this.open {
                                let focus = this.field.focus_handle(cx);
                                window.focus(&focus, cx);
                            } else if !this.field.read(cx).text().trim().is_empty() {
                                // What is typed and not put in yet goes in on Done,
                                // or it stays open to say why it cannot.
                                this.add_typed(cx);
                                this.open = this.problem.is_some();
                            }
                            cx.notify();
                        })),
                    ),
            )
            .when(open, |d| d.child(self.editor(window, cx)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_preset_is_a_pattern() {
        for preset in LEAVE_OUT.iter().chain(TAKE_ONLY) {
            for p in preset.patterns {
                assert!(Filter::check(p).is_ok(), "{p}");
            }
        }
    }
}
