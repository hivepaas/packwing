//! The pieces the screens are built from, drawn as the Soft tonal design draws
//! them: pill buttons, chips, switches, radio and check marks, option cards,
//! notes, a segmented choice, and a select with its menu.

use gpui::{
    BoxShadow, Context, Div, ElementId, EventEmitter, FontWeight, IntoElement, MouseButton, Render,
    SharedString, Stateful, Svg, Window, anchored, deferred, div, point, prelude::*, px, relative,
    rgb, rgba, svg,
};

use crate::colors::{Tint, colors};

/// An icon from `assets/icons`, by its name, `size` px square.
pub fn icon(name: &str, size: f32, color: u32) -> Svg {
    svg()
        .path(SharedString::from(format!("icons/{name}.svg")))
        .size(px(size))
        .flex_shrink_0()
        .text_color(rgb(color))
}

/// A rounded square holding an icon or a few letters.
pub fn tile(size: f32, radius: f32, tint: Tint) -> Div {
    div()
        .size(px(size))
        .flex_shrink_0()
        .rounded(px(radius))
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
        .size(px(size))
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

/// How big a button is: its height and its text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Size {
    /// 52 px, on a page.
    Large,
    /// 48 px, in a dialog's footer.
    Medium,
    /// 40 px, beside a note or a row.
    Small,
}

/// A pill button saying `label`.
pub fn button(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    fill: Fill,
    size: Size,
) -> Stateful<Div> {
    let c = colors();
    let (h, text, pad) = match size {
        Size::Large => (52., 16., 22.),
        Size::Medium => (48., 15., 22.),
        Size::Small => (40., 14., 16.),
    };
    let (ground, ink, hover) = match fill {
        Fill::Accent => (Some(c.accent), c.on_accent, c.accent_hover),
        Fill::Tonal => (Some(c.tonal), c.on_tonal, c.tonal_hover),
        Fill::Raised => (Some(c.surface), c.on_tonal, c.hover),
        Fill::Outline => (Some(c.surface), c.text, c.hover),
        Fill::Text => (None, c.accent, c.hover),
    };
    div()
        .id(id)
        .flex()
        .flex_shrink_0()
        .items_center()
        .justify_center()
        .gap(px(8.))
        .h(px(h))
        .px(px(pad))
        .rounded_full()
        .when_some(ground, |d, g| d.bg(rgb(g)))
        .when(fill == Fill::Outline, |d| {
            d.border_1().border_color(rgb(c.outline))
        })
        .text_color(rgb(ink))
        .text_size(px(text))
        .font_weight(FontWeight::SEMIBOLD)
        .whitespace_nowrap()
        .cursor_pointer()
        .hover(move |s| s.bg(rgb(hover)))
        .child(label.into())
}

/// A link-like button: the accent's text, no fill.
pub fn link(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Stateful<Div> {
    div()
        .id(id)
        .flex_shrink_0()
        .text_size(px(14.))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(rgb(colors().accent))
        .cursor_pointer()
        .hover(|s| s.underline())
        .child(label.into())
}

/// A round button with an icon alone, such as a dialog's close button.
pub fn icon_button(id: impl Into<ElementId>, name: &str, size: f32) -> Stateful<Div> {
    let c = colors();
    div()
        .id(id)
        .size(px(size))
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
        .gap(px(6.))
        .px(px(12.))
        .py(px(6.))
        .rounded_full()
        .bg(rgb(ground))
        .text_color(rgb(ink))
        .text_size(px(13.))
        .font_weight(FontWeight::SEMIBOLD)
        .whitespace_nowrap()
        .child(text.into())
}

/// An on/off switch.
pub fn switch(id: impl Into<ElementId>, on: bool) -> Stateful<Div> {
    let c = colors();
    div()
        .id(id)
        .flex_shrink_0()
        .w(px(52.))
        .h(px(32.))
        .p(px(4.))
        .rounded_full()
        .flex()
        .when(on, |d| d.justify_end())
        .bg(rgb(if on { c.accent } else { c.outline }))
        .cursor_pointer()
        .child(
            div()
                .size(px(24.))
                .rounded_full()
                .bg(rgb(if on { c.knob_on } else { c.knob_off })),
        )
}

/// A radio button's mark.
pub fn radio(on: bool) -> Div {
    let c = colors();
    div()
        .size(px(20.))
        .flex_shrink_0()
        .rounded_full()
        .border_2()
        .border_color(rgb(if on { c.accent } else { c.muted }))
        .flex()
        .items_center()
        .justify_center()
        .when(on, |d| {
            d.child(div().size(px(10.)).rounded_full().bg(rgb(c.accent)))
        })
}

/// A check box's mark.
pub fn checkbox(on: bool) -> Div {
    let c = colors();
    div()
        .size(px(18.))
        .flex_shrink_0()
        .rounded(px(5.))
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
    div()
        .id(id)
        .flex()
        .items_center()
        .gap(px(10.))
        .text_size(px(14.))
        .cursor_pointer()
        .child(checkbox(on))
        .child(label.into())
}

/// A choice drawn as a card with a radio mark: the soft fill when chosen.
/// The caller adds what it says.
pub fn option_card(id: impl Into<ElementId>, on: bool) -> Stateful<Div> {
    let c = colors();
    div()
        .id(id)
        .flex()
        .items_center()
        .gap(px(14.))
        .px(px(16.))
        .py(px(14.))
        .rounded(px(20.))
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
        .gap(px(2.))
        .min_w_0()
        .child(
            div()
                .text_size(px(15.))
                .font_weight(if on {
                    FontWeight::BOLD
                } else {
                    FontWeight::SEMIBOLD
                })
                .child(title.into()),
        )
        .child(
            div()
                .text_size(px(13.))
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
    div()
        .id(id)
        .flex_1()
        .h(px(36.))
        .flex()
        .items_center()
        .justify_center()
        .rounded_full()
        .text_size(px(14.))
        .font_weight(if on {
            FontWeight::BOLD
        } else {
            FontWeight::MEDIUM
        })
        .cursor_pointer()
        .when(on, |d| d.bg(rgb(c.surface)))
        .when(!on, |d| d.hover(|s| s.bg(rgb(c.well_hover))))
        .child(label.into())
}

/// The track a row of [`segment`]s sits in.
pub fn segments(items: impl IntoIterator<Item = Stateful<Div>>) -> Div {
    div()
        .flex()
        .gap(px(4.))
        .p(px(4.))
        .rounded_full()
        .bg(rgb(colors().field))
        .children(items)
}

/// How a note is meant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    /// Worth knowing before going on.
    Warn,
}

/// A short note in a tinted box.
pub fn note(tone: Tone) -> Div {
    let c = colors();
    let (ground, ink) = match tone {
        Tone::Warn => (c.warn_bg, c.warn_text),
    };
    div()
        .px(px(12.))
        .py(px(10.))
        .rounded(px(14.))
        .bg(rgb(ground))
        .text_color(rgb(ink))
        .text_size(px(13.))
        .line_height(relative(1.45))
}

/// A heading over a group of choices.
pub fn legend(text: impl Into<SharedString>) -> Div {
    div()
        .text_size(px(15.))
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
}

/// Another option of a [`Select`] was chosen: [`Select::chosen`] says which.
pub struct Chosen;

impl EventEmitter<Chosen> for Select {}

impl Select {
    pub fn new(options: &[&str], chosen: usize, look: SelectLook) -> Self {
        Select {
            options: options
                .iter()
                .map(|o| SharedString::from(o.to_string()))
                .collect(),
            chosen: chosen.min(options.len().saturating_sub(1)),
            open: false,
            look,
        }
    }

    /// The index of the option chosen.
    pub fn chosen(&self) -> usize {
        self.chosen
    }

    #[cfg(all(test, feature = "shots"))]
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
            div()
                .id(("option", i))
                .h(px(40.))
                .px(px(12.))
                .flex()
                .items_center()
                .gap(px(10.))
                .rounded(px(12.))
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
            .id("menu")
            .min_w(px(220.))
            .p(px(6.))
            .flex()
            .flex_col()
            .rounded(px(16.))
            .bg(rgb(c.surface))
            .border_1()
            .border_color(rgb(c.outline))
            .shadow(float_shadow(8., 24.))
            .text_size(px(14.))
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
        let trigger = div()
            .id("select")
            .flex()
            .items_center()
            .gap(px(6.))
            .rounded_full()
            .bg(rgb(c.surface))
            .text_color(rgb(c.text))
            .whitespace_nowrap()
            .cursor_pointer()
            .hover(|s| s.bg(rgb(c.hover)))
            .map(|d| match self.look {
                SelectLook::Pill => d
                    .h(px(40.))
                    .px(px(14.))
                    .border_1()
                    .border_color(rgb(c.outline))
                    .text_size(px(14.)),
                SelectLook::Field => d
                    .w_full()
                    .h(px(44.))
                    .pl(px(16.))
                    .pr(px(14.))
                    .text_size(px(15.)),
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
