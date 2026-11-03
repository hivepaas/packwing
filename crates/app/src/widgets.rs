//! The pieces the screens are built from, drawn as the Soft tonal design draws
//! them: pill buttons, chips, switches, radio and check marks, option cards,
//! notes, a segmented choice, and a select with its menu.

use gpui::{
    AnyView, App, AppContext as _, BoxShadow, Context, Div, ElementId, EventEmitter, FontWeight,
    IntoElement, MouseButton, Render, SharedString, Stateful, Svg, Window, anchored, deferred, div,
    point, prelude::*, px, relative, rgb, rgba, svg,
};

use crate::colors::{Tint, colors};
use crate::text_size::ui;

/// Monospace fonts, for what is read character by character, the first one
/// installed.
const MONO_FONTS: [&str; 8] = [
    "SF Mono",
    "Menlo",
    "Cascadia Mono",
    "Consolas",
    "JetBrains Mono",
    "DejaVu Sans Mono",
    "Liberation Mono",
    "Ubuntu Mono",
];

/// The first of [`MONO_FONTS`] installed, or packwing's own font.
pub fn mono_font(window: &Window) -> SharedString {
    let installed = window.text_system().all_font_names();
    MONO_FONTS
        .iter()
        .find(|f| installed.iter().any(|i| i == *f))
        .copied()
        .unwrap_or(crate::FONT)
        .into()
}

/// An icon from `assets/icons`, by its name, `size` px square.
pub fn icon(name: &str, size: f32, color: u32) -> Svg {
    svg()
        .path(SharedString::from(format!("icons/{name}.svg")))
        .size(ui(size))
        .flex_shrink_0()
        .text_color(rgb(color))
}

/// A rounded square holding an icon or a few letters.
pub fn tile(size: f32, radius: f32, tint: Tint) -> Div {
    div()
        .size(ui(size))
        .flex_shrink_0()
        .rounded(ui(radius))
        .bg(rgb(tint.ground))
        .text_color(rgb(tint.ink))
        .flex()
        .items_center()
        .justify_center()
}

/// The accent's tint, for the logo's tile.
pub fn accent_tint() -> Tint {
    let c = colors();
    Tint {
        ground: c.accent,
        ink: c.on_accent,
    }
}

/// packwing's logo on a tile of `tint`: the zipper in the tile's ink, and
/// its pull, a feather, in the design's orange. GPUI draws an icon in one
/// color, so the two are icons of their own, one over the other.
pub fn logo(size: f32, tint: Tint) -> Div {
    div()
        .relative()
        .size(ui(size))
        .flex_shrink_0()
        .child(icon("logo", size, tint.ink).absolute().top_0().left_0())
        .child(
            icon("logo-feather", size, colors().feather(tint))
                .absolute()
                .top_0()
                .left_0(),
        )
}

/// The tint of the accent's soft fill, for a dialog's icon.
pub fn tonal_tint() -> Tint {
    let c = colors();
    Tint {
        ground: c.tonal,
        ink: c.on_tonal,
    }
}

/// How a button is filled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fill {
    /// The accent: the one thing to do.
    Accent,
    /// The accent's soft fill.
    Tonal,
    /// The card's own white, on the soft fill.
    Raised,
    /// White with a line around it.
    Outline,
    /// None: the accent's text alone.
    Text,
}

/// How big a button is: its height and its text. Every other control, a field, a
/// select, the segments, an icon button, is as tall as a Medium one beside it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Size {
    /// 48 px, at the top of a page, as its search is.
    Large,
    /// 40 px: in a dialog, its footer's and its rows', and in Settings.
    Medium,
}

/// A pill button saying `label`.
pub fn button(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    fill: Fill,
    size: Size,
) -> Stateful<Div> {
    let c = colors();
    let id = id.into();
    let selector = id.to_string();
    let (h, text, pad) = match size {
        Size::Large => (48., 16., 22.),
        Size::Medium => (40., 15., 18.),
    };
    let (ground, ink, hover) = match fill {
        Fill::Accent => (Some(c.accent), c.on_accent, c.accent_hover),
        Fill::Tonal => (Some(c.tonal), c.on_tonal, c.tonal_hover),
        Fill::Raised => (Some(c.surface), c.on_tonal, c.hover),
        Fill::Outline => (Some(c.surface), c.text, c.hover),
        Fill::Text => (None, c.accent, c.hover),
    };
    div()
        .debug_selector(|| selector)
        .id(id)
        .flex()
        .flex_shrink_0()
        .items_center()
        .justify_center()
        .gap(ui(8.))
        .h(ui(h))
        .px(ui(pad))
        .rounded_full()
        .when_some(ground, |d, g| d.bg(rgb(g)))
        .when(fill == Fill::Outline, |d| {
            d.border_1().border_color(rgb(c.outline))
        })
        .text_color(rgb(ink))
        .text_size(ui(text))
        .font_weight(FontWeight::SEMIBOLD)
        .whitespace_nowrap()
        .cursor_pointer()
        .hover(move |s| s.bg(rgb(hover)))
        .child(label.into())
}

/// A round button with an icon alone, such as a dialog's close button.
pub fn icon_button(id: impl Into<ElementId>, name: &str, size: f32) -> Stateful<Div> {
    let c = colors();
    let id = id.into();
    let selector = id.to_string();
    div()
        .debug_selector(|| selector)
        .id(id)
        .size(ui(size))
        .flex_shrink_0()
        .rounded_full()
        .bg(rgb(c.well))
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .hover(|s| s.bg(rgb(c.well_hover)))
        .child(icon(name, 20., c.muted))
}

/// A small pill of text, such as an archive's size in its header.
pub fn chip(text: impl Into<SharedString>, ground: u32, ink: u32) -> Div {
    div()
        .flex()
        .flex_shrink_0()
        .items_center()
        .gap(ui(6.))
        .px(ui(12.))
        .py(ui(6.))
        .rounded_full()
        .bg(rgb(ground))
        .text_color(rgb(ink))
        .text_size(ui(13.))
        .font_weight(FontWeight::SEMIBOLD)
        .whitespace_nowrap()
        .child(text.into())
}

/// An on/off switch.
pub fn switch(id: impl Into<ElementId>, on: bool) -> Stateful<Div> {
    let c = colors();
    let id = id.into();
    let selector = id.to_string();
    div()
        .debug_selector(|| selector)
        .id(id)
        .flex_shrink_0()
        .w(ui(52.))
        .h(ui(32.))
        .p(ui(4.))
        .rounded_full()
        .flex()
        .when(on, |d| d.justify_end())
        .bg(rgb(if on { c.accent } else { c.outline }))
        .cursor_pointer()
        .child(
            div()
                .size(ui(24.))
                .rounded_full()
                .bg(rgb(if on { c.knob_on } else { c.knob_off })),
        )
}

/// A radio button's mark.
pub fn radio(on: bool) -> Div {
    let c = colors();
    div()
        .size(ui(20.))
        .flex_shrink_0()
        .rounded_full()
        .border_2()
        .border_color(rgb(if on { c.accent } else { c.muted }))
        .flex()
        .items_center()
        .justify_center()
        .when(on, |d| {
            d.child(div().size(ui(10.)).rounded_full().bg(rgb(c.accent)))
        })
}

/// A check box's mark.
pub fn checkbox(on: bool) -> Div {
    let c = colors();
    div()
        .size(ui(18.))
        .flex_shrink_0()
        .rounded(ui(5.))
        .flex()
        .items_center()
        .justify_center()
        .when(on, |d| {
            d.bg(rgb(c.accent)).child(icon("check", 14., c.on_accent))
        })
        .when(!on, |d| d.border_2().border_color(rgb(c.muted)))
}

/// A check box with its label, the whole of it clickable.
pub fn check_row(
    id: impl Into<ElementId>,
    on: bool,
    label: impl Into<SharedString>,
) -> Stateful<Div> {
    let id = id.into();
    let selector = id.to_string();
    div()
        .debug_selector(|| selector)
        .id(id)
        .flex()
        .items_center()
        .gap(ui(10.))
        .text_size(ui(14.))
        .cursor_pointer()
        .child(checkbox(on))
        .child(label.into())
}

/// A choice drawn as a card with a radio mark: the soft fill when chosen.
/// The caller adds what it says.
pub fn option_card(id: impl Into<ElementId>, on: bool) -> Stateful<Div> {
    let c = colors();
    let id = id.into();
    let selector = id.to_string();
    div()
        .debug_selector(|| selector)
        .id(id)
        .flex()
        .min_w_0()
        .items_center()
        .gap(ui(14.))
        .px(ui(16.))
        .py(ui(14.))
        .rounded(ui(20.))
        .bg(rgb(if on { c.tonal } else { c.well }))
        .text_color(rgb(if on { c.on_tonal } else { c.text }))
        .cursor_pointer()
        .when(!on, |d| d.hover(|s| s.bg(rgb(c.well_hover))))
        .child(radio(on))
}

/// A card's title and the line under it.
pub fn titled(title: impl Into<SharedString>, detail: impl Into<SharedString>, on: bool) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(ui(2.))
        .min_w_0()
        .child(
            div()
                .text_size(ui(15.))
                .font_weight(if on {
                    FontWeight::BOLD
                } else {
                    FontWeight::SEMIBOLD
                })
                .child(title.into()),
        )
        .child(
            div()
                .text_size(ui(13.))
                .truncate()
                .when(!on, |d| d.text_color(rgb(colors().muted)))
                .child(detail.into()),
        )
}

/// One of a few choices side by side, the chosen one raised.
pub fn segment(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    on: bool,
) -> Stateful<Div> {
    let c = colors();
    let id = id.into();
    let selector = id.to_string();
    div()
        .debug_selector(|| selector)
        .id(id)
        .flex_1()
        .min_w_0()
        .h(ui(32.))
        .px(ui(12.))
        .flex()
        .items_center()
        .justify_center()
        .rounded_full()
        .text_size(ui(14.))
        .font_weight(if on {
            FontWeight::BOLD
        } else {
            FontWeight::MEDIUM
        })
        .cursor_pointer()
        .when(on, |d| d.bg(rgb(c.surface)))
        .when(!on, |d| d.hover(|s| s.bg(rgb(c.well_hover))))
        .child(div().min_w_0().truncate().child(label.into()))
}

/// The track a row of [`segment`]s sits in.
pub fn segments(items: impl IntoIterator<Item = Stateful<Div>>) -> Div {
    div()
        .flex()
        .gap(ui(4.))
        .p(ui(4.))
        .rounded_full()
        .bg(rgb(colors().field))
        .children(items)
}

/// How a note is meant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    /// Worth knowing before going on.
    Warn,
    /// It went well.
    Good,
    /// It is wrong.
    Bad,
}

/// A short note in a tinted box.
pub fn note(tone: Tone) -> Div {
    let c = colors();
    let (ground, ink) = match tone {
        Tone::Warn => (c.warn_bg, c.warn_text),
        Tone::Good => (c.tonal, c.on_tonal),
        Tone::Bad => (c.bad_bg, c.bad_text),
    };
    div()
        .px(ui(12.))
        .py(ui(10.))
        .rounded(ui(14.))
        .bg(rgb(ground))
        .text_color(rgb(ink))
        .text_size(ui(13.))
        .line_height(relative(1.45))
}

/// A heading over a group of choices.
pub fn legend(text: impl Into<SharedString>) -> Div {
    div()
        .text_size(ui(15.))
        .font_weight(FontWeight::SEMIBOLD)
        .child(text.into())
}

/// The shadow under a dialog, a menu or a card that floats.
pub fn float_shadow(y: f32, blur: f32) -> Vec<BoxShadow> {
    vec![BoxShadow {
        color: rgba(colors().shadow).into(),
        offset: point(px(0.), px(y)),
        blur_radius: px(blur),
        spread_radius: px(0.),
        inset: false,
    }]
}

/// How a [`Select`] is drawn closed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectLook {
    /// A small pill with a line around it, beside other content.
    Pill,
    /// A white field in a well, as tall as a row of text fields.
    Field,
}

/// A choice of a few options: closed, the one chosen and a chevron; open, a
/// menu of them all under it.
pub struct Select {
    options: Vec<SharedString>,
    chosen: usize,
    open: bool,
    look: SelectLook,
    /// What a test clicks to open it: `select: {mark}`.
    mark: Option<&'static str>,
}

/// Another option of a [`Select`] was chosen: [`Select::chosen`] says which.
pub struct Chosen;

impl EventEmitter<Chosen> for Select {}

impl Select {
    pub fn new(options: &[impl AsRef<str>], chosen: usize, look: SelectLook) -> Self {
        Select {
            options: options
                .iter()
                .map(|o| SharedString::from(o.as_ref().to_owned()))
                .collect(),
            chosen: chosen.min(options.len().saturating_sub(1)),
            open: false,
            look,
            mark: None,
        }
    }

    /// Is clicked by tests as `select: {mark}`, and its options as `option: {name}`.
    pub fn marked(mut self, mark: &'static str) -> Self {
        self.mark = Some(mark);
        self
    }

    /// Has `options` from now on, and chooses the one at `chosen`.
    pub fn set_options(
        &mut self,
        options: &[impl AsRef<str>],
        chosen: usize,
        cx: &mut Context<Self>,
    ) {
        self.options = options
            .iter()
            .map(|o| SharedString::from(o.as_ref().to_owned()))
            .collect();
        self.chosen = chosen.min(options.len().saturating_sub(1));
        cx.notify();
    }

    /// The index of the option chosen.
    pub fn chosen(&self) -> usize {
        self.chosen
    }

    #[cfg(all(test, feature = "test-hooks"))]
    pub fn open_menu(&mut self, cx: &mut Context<Self>) {
        self.open = true;
        cx.notify();
    }

    #[cfg(all(test, feature = "test-hooks"))]
    pub fn choose(&mut self, i: usize, cx: &mut Context<Self>) {
        self.chosen = i.min(self.options.len().saturating_sub(1));
        cx.emit(Chosen);
        cx.notify();
    }

    fn close(&mut self, cx: &mut Context<Self>) {
        self.open = false;
        cx.notify();
    }

    fn menu(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let items = self.options.iter().enumerate().map(|(i, o)| {
            let on = i == self.chosen;
            let name = o.clone();
            div()
                .debug_selector(move || format!("option: {name}"))
                .id(("option", i))
                .flex_shrink_0()
                .h(ui(40.))
                .px(ui(12.))
                .flex()
                .items_center()
                .gap(ui(10.))
                .rounded(ui(12.))
                .cursor_pointer()
                .hover(|s| s.bg(rgb(c.hover)))
                .when(on, |d| d.font_weight(FontWeight::SEMIBOLD))
                .on_click(cx.listener(move |this, _, _, cx| {
                    if this.chosen != i {
                        this.chosen = i;
                        cx.emit(Chosen);
                    }
                    this.close(cx);
                }))
                .child(div().flex_1().whitespace_nowrap().child(o.clone()))
                .when(on, |d| d.child(icon("check", 16., c.accent)))
        });
        let menu = div()
            .debug_selector(|| "menu".to_owned())
            .id("menu")
            // A press on it is not one elsewhere: the catcher below would close
            // the menu before the press's release chooses the option.
            .occlude()
            .min_w(ui(220.))
            // More than this scroll.
            .max_h(ui(372.))
            .overflow_y_scroll()
            .p(ui(6.))
            .flex()
            .flex_col()
            .rounded(ui(16.))
            .bg(rgb(c.surface))
            .border_1()
            .border_color(rgb(c.outline))
            .shadow(float_shadow(8., 24.))
            .text_size(ui(14.))
            .text_color(rgb(c.text))
            .children(items);
        // A press anywhere else closes the menu, and goes no further.
        let size = window.viewport_size();
        let catcher = div()
            .id("menu-catcher")
            .w(size.width)
            .h(size.height)
            .occlude()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| this.close(cx)),
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
                        .child(menu),
                )
                .with_priority(3),
            )
    }
}

impl Render for Select {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let label = self.options.get(self.chosen).cloned().unwrap_or_default();
        let mark = self.mark;
        let trigger = div()
            .when_some(mark, |d, mark| {
                d.debug_selector(move || format!("select: {mark}"))
            })
            .id("select")
            .flex()
            .items_center()
            .gap(ui(6.))
            .rounded_full()
            .bg(rgb(c.surface))
            .text_color(rgb(c.text))
            .whitespace_nowrap()
            .cursor_pointer()
            .hover(|s| s.bg(rgb(c.hover)))
            .map(|d| match self.look {
                SelectLook::Pill => d
                    .h(ui(40.))
                    .px(ui(14.))
                    .border_1()
                    .border_color(rgb(c.outline))
                    .text_size(ui(14.)),
                SelectLook::Field => d
                    .w_full()
                    .h(ui(40.))
                    .pl(ui(16.))
                    .pr(ui(14.))
                    .text_size(ui(15.)),
            })
            .on_click(cx.listener(|this, _, _, cx| {
                this.open = !this.open;
                cx.notify();
            }))
            .child(div().flex_1().min_w_0().truncate().child(label))
            .child(icon("chevron-down", 14., c.text));
        let menu = self.open.then(|| self.menu(window, cx));
        div()
            .flex()
            .flex_col()
            .flex_shrink_0()
            .when(self.look == SelectLook::Field, |d| d.w_full())
            .child(trigger)
            .children(menu)
    }
}

/// A tooltip: a short line on a small card by the pointer.
pub struct Tip(SharedString);

impl Render for Tip {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        div()
            .px(ui(10.))
            .py(ui(6.))
            .rounded(ui(10.))
            .bg(rgb(c.surface))
            .border_1()
            .border_color(rgb(c.outline))
            .shadow(float_shadow(4., 12.))
            .font_family(crate::FONT)
            .text_size(ui(13.))
            .text_color(rgb(c.text))
            .child(self.0.clone())
    }
}

/// A tooltip saying `text`, for an element's `tooltip`.
pub fn tip(text: impl Into<SharedString>) -> impl Fn(&mut Window, &mut App) -> AnyView + 'static {
    let text = text.into();
    move |_, cx| cx.new(|_| Tip(text.clone())).into()
}
