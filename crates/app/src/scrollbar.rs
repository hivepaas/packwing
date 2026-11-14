//! The bar beside a list that scrolls: where it is in the list, how much of
//! it is in sight, and a thumb to drag. GPUI scrolls with the wheel but draws
//! no bar of its own.
//!
//! It is placed by its caller, absolutely, over the edge it goes along:
//! `scrollbar(id, handle).absolute().top_0().bottom_0().right(…)`. It reads
//! the list's [`ScrollHandle`] once the list is laid out, so it comes after
//! the list among its parent's children. It draws nothing while all of the
//! list is in sight.

use std::cell::Cell;
use std::rc::Rc;

use gpui::{
    App, Bounds, CursorStyle, Element, ElementId, GlobalElementId, Hitbox, HitboxBehavior,
    InspectorElementId, IntoElement, LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, Pixels, Refineable as _, ScrollHandle, Style, StyleRefinement, Styled,
    UniformListScrollHandle, Window, point, px, quad, rgb, size, transparent_black,
};

use crate::colors::colors;
use crate::text_size::ui;

/// How wide the bar is, as a pointer sees it.
pub const WIDTH: f32 = 12.;
/// The thumb, at rest and under the pointer or held.
const THIN: f32 = 6.;
const WIDE: f32 = 8.;
/// Between the thumb and the edge it goes along.
const INSET: f32 = 2.;
/// The thumb is never shorter, however long the list.
const SHORTEST: f32 = 28.;

/// The handle of what a bar moves: a `div`'s, or a `uniform_list`'s.
pub trait Scrolls {
    fn base(&self) -> ScrollHandle;
}

impl Scrolls for ScrollHandle {
    fn base(&self) -> ScrollHandle {
        self.clone()
    }
}

impl Scrolls for UniformListScrollHandle {
    fn base(&self) -> ScrollHandle {
        self.0.borrow().base_handle.clone()
    }
}

/// A bar for what `handle` scrolls.
pub fn scrollbar(id: impl Into<ElementId>, handle: &impl Scrolls) -> Scrollbar {
    Scrollbar {
        id: id.into(),
        handle: handle.base(),
        style: StyleRefinement::default(),
    }
}

/// How far in from a card's edge its lists start: the padding of the
/// pages' panels, and of the dialogs' wells.
const PAD: f32 = 10.;
/// The roundness of a panel's corners, the largest the bar goes beside.
const CORNER: f32 = 28.;

/// A bar for a list held [`PAD`] in from a rounded card's edge: along the
/// card's own edge, out of the list's way, and short of the rounded corner
/// below. Its parent is the list's, made `relative()`.
pub fn edge_bar(id: impl Into<ElementId>, handle: &impl Scrolls) -> Scrollbar {
    scrollbar(id, handle)
        .absolute()
        .top_0()
        .bottom(ui(CORNER - PAD))
        .right(ui(-PAD))
}

/// A bar for a list that fills a well `pad` in from its edges, its corners
/// `corner` round: along the well's edge, short of its corners above and
/// below. Its parent is the list's, made `relative()`.
pub fn well_bar(
    id: impl Into<ElementId>,
    handle: &impl Scrolls,
    pad: f32,
    corner: f32,
) -> Scrollbar {
    let short = ui((corner - pad).max(0.));
    scrollbar(id, handle)
        .absolute()
        .top(short)
        .bottom(short)
        .right(ui(-pad))
}

/// The thumb, drawn `top` down and `height` long, along a bar whose right
/// edge is `right`; wide, under the pointer or held. For what scrolls
/// itself and draws its own bar, as a text field does.
pub fn paint_thumb(window: &mut Window, right: Pixels, top: Pixels, height: Pixels, wide: bool) {
    let c = colors();
    let w = px(if wide { WIDE } else { THIN });
    window.paint_quad(quad(
        Bounds::new(point(right - px(INSET) - w, top), size(w, height)),
        w / 2.,
        rgb(if wide { c.faint } else { c.outline }),
        px(0.),
        transparent_black(),
        Default::default(),
    ));
}

/// A bar for a rounded card that scrolls itself: along its edge, short of
/// its rounded corners above and below. Its parent holds the card, made
/// `relative()`, and is as large as it.
pub fn card_bar(id: impl Into<ElementId>, handle: &impl Scrolls) -> Scrollbar {
    scrollbar(id, handle)
        .absolute()
        .top(ui(CORNER))
        .bottom(ui(CORNER))
        .right_0()
}

pub struct Scrollbar {
    id: ElementId,
    handle: ScrollHandle,
    style: StyleRefinement,
}

impl Styled for Scrollbar {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl IntoElement for Scrollbar {
    type Element = Self;

    fn into_element(self) -> Self {
        self
    }
}

/// What a bar keeps from one frame to the next: where on the thumb it was
/// taken hold of, while it is held.
#[derive(Clone, Default)]
struct Held(Rc<Cell<Option<Pixels>>>);

/// Where the thumb is drawn, in the bar's bounds.
#[derive(Clone, Copy)]
struct Thumb {
    top: Pixels,
    height: Pixels,
    /// How far the list scrolls, all the way down.
    most: Pixels,
    /// How far the thumb travels, all the way down.
    travel: Pixels,
}

impl Thumb {
    fn of(handle: &ScrollHandle, bounds: Bounds<Pixels>) -> Option<Thumb> {
        Thumb::new(
            handle.bounds().size.height,
            handle.max_offset().y,
            -handle.offset().y,
            bounds.size.height,
        )
    }

    /// With `seen` of the list in sight, `most` more below it all the way
    /// up, scrolled `down`, along a bar `track` long.
    fn new(seen: Pixels, most: Pixels, down: Pixels, track: Pixels) -> Option<Thumb> {
        if most <= px(0.5) || seen <= px(0.) {
            return None;
        }
        let height = (track * (seen / (seen + most)))
            .max(px(SHORTEST))
            .min(track);
        let travel = track - height;
        let down = down.clamp(px(0.), most);
        Some(Thumb {
            top: travel * (down / most),
            height,
            most,
            travel,
        })
    }

    /// How far down the list is with the thumb's top at `top`.
    fn offset_for(&self, top: Pixels) -> Pixels {
        if self.travel <= px(0.) {
            return px(0.);
        }
        self.most * (top.clamp(px(0.), self.travel) / self.travel)
    }
}

pub struct Prepainted {
    hitbox: Hitbox,
    thumb: Option<Thumb>,
    held: Held,
}

impl Element for Scrollbar {
    type RequestLayoutState = ();
    type PrepaintState = Prepainted;

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        let mut style = Style::default();
        style.refine(&self.style);
        style.size.width = px(WIDTH).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        id: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        _: &mut App,
    ) -> Prepainted {
        let held = window.with_element_state(id.expect("a bar has an id"), |held, _| {
            let held: Held = held.unwrap_or_default();
            (held.clone(), held)
        });
        let thumb = Thumb::of(&self.handle, bounds);
        let hitbox = window.insert_hitbox(
            if thumb.is_some() {
                bounds
            } else {
                Bounds::default()
            },
            HitboxBehavior::BlockMouse,
        );
        Prepainted {
            hitbox,
            thumb,
            held,
        }
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        prepainted: &mut Prepainted,
        window: &mut Window,
        _: &mut App,
    ) {
        let Some(thumb) = prepainted.thumb else {
            return;
        };
        let hitbox = prepainted.hitbox.clone();
        let held = prepainted.held.0.clone();
        let wide = held.get().is_some() || hitbox.is_hovered(window);
        paint_thumb(
            window,
            bounds.right(),
            bounds.top() + thumb.top,
            thumb.height,
            wide,
        );
        window.set_cursor_style(CursorStyle::Arrow, &hitbox);

        let handle = self.handle.clone();
        let top = bounds.top();
        let scroll_to = move |thumb_top: Pixels, window: &mut Window| {
            let x = handle.offset().x;
            handle.set_offset(point(x, -thumb.offset_for(thumb_top)));
            window.refresh();
        };

        // Taken hold of: on the thumb, where it was pressed; beside it, the
        // thumb comes to the pointer, its middle under it, and is held there.
        window.on_mouse_event({
            let (hitbox, held, scroll_to) = (hitbox.clone(), held.clone(), scroll_to.clone());
            move |e: &MouseDownEvent, phase, window, cx| {
                if !phase.bubble() || e.button != MouseButton::Left || !hitbox.is_hovered(window) {
                    return;
                }
                let y = e.position.y - top;
                let grab = if y >= thumb.top && y <= thumb.top + thumb.height {
                    y - thumb.top
                } else {
                    let grab = thumb.height / 2.;
                    scroll_to(y - grab, window);
                    grab
                };
                held.set(Some(grab));
                window.refresh();
                cx.stop_propagation();
            }
        });
        // Held, it follows the pointer, wherever the pointer goes.
        window.on_mouse_event({
            let held = held.clone();
            move |e: &MouseMoveEvent, phase, window, cx| {
                let Some(grab) = held.get() else {
                    return;
                };
                if !phase.bubble() {
                    return;
                }
                if e.pressed_button != Some(MouseButton::Left) {
                    held.set(None);
                    window.refresh();
                    return;
                }
                scroll_to(e.position.y - top - grab, window);
                cx.stop_propagation();
            }
        });
        window.on_mouse_event(move |e: &MouseUpEvent, phase, window, cx| {
            if phase.bubble() && e.button == MouseButton::Left && held.take().is_some() {
                window.refresh();
                cx.stop_propagation();
            }
        });
        // Under the pointer, it is drawn wider; on its way in and out, again.
        window.on_mouse_event({
            let was = hitbox.is_hovered(window);
            move |_: &MouseMoveEvent, phase, window, _| {
                if phase.bubble() && hitbox.is_hovered(window) != was {
                    window.refresh();
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn thumb(seen: f32, most: f32, down: f32, track: f32) -> Thumb {
        Thumb::new(px(seen), px(most), px(down), px(track)).expect("a thumb")
    }

    #[test]
    fn all_of_it_in_sight_has_no_thumb() {
        assert!(Thumb::new(px(400.), px(0.), px(0.), px(400.)).is_none());
    }

    #[test]
    fn the_thumb_is_as_long_as_what_is_in_sight() {
        let t = thumb(400., 400., 0., 400.);
        assert_eq!(t.height, px(200.));
        assert_eq!(t.top, px(0.));
        let t = thumb(400., 400., 400., 400.);
        assert_eq!(t.top, px(200.));
    }

    #[test]
    fn a_long_list_still_has_a_thumb_to_take_hold_of() {
        let t = thumb(400., 400_000., 0., 400.);
        assert_eq!(t.height, px(SHORTEST));
    }

    #[test]
    fn dragging_the_thumb_moves_the_list_as_far_and_no_further() {
        let t = thumb(400., 400., 0., 400.);
        assert_eq!(t.offset_for(px(100.)), px(200.));
        assert_eq!(t.offset_for(px(-50.)), px(0.));
        assert_eq!(t.offset_for(px(900.)), px(400.));
    }
}
