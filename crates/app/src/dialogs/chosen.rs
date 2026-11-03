//! What goes in an archive, as New archive and Add files ask for it: a line that
//! says it, buttons for more, and a list to look through and take things out of.
//! Files can also be dropped on either dialog.

use std::path::Path;

use gpui::{Context, FontWeight, UniformListScrollHandle, div, prelude::*, px, rgb, uniform_list};
use packwing_i18n::tr;

use crate::colors::colors;
use crate::model::{Kind, Selection, tag_of};
use crate::text_size::ui;
use crate::widgets::{Fill, Size, Tone, button, icon, icon_button, legend, note, tile, tip};

/// How tall a row of the list is, and how many show before it scrolls.
const ROW: f32 = 44.;
const MOST_ROWS: usize = 7;

/// A dialog that asks what goes in an archive.
pub trait Choosing: Sized + 'static {
    fn selection(&self) -> &Selection;
    fn list_shown(&self) -> bool;
    /// Where the list has scrolled to.
    fn list_scroll(&self) -> &UniformListScrollHandle;
    fn show_list(&mut self, shown: bool);
    /// Asks the system for files, or for folders.
    fn pick(&mut self, folders: bool, cx: &mut Context<Self>);
    /// Takes `path` out of what goes in.
    fn remove(&mut self, path: &Path, cx: &mut Context<Self>);
}

/// The first few names, and how many more there are.
fn summary(selection: &Selection) -> String {
    let names: Vec<String> = selection
        .paths
        .iter()
        .take(3)
        .filter_map(|p| p.file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .collect();
    let more = selection.paths.len().saturating_sub(names.len());
    match (names.is_empty(), more) {
        (true, _) => tr!("Nothing chosen yet"),
        (false, 0) => names.join(", "),
        (false, more) => tr!(
            "{join} and {more} more",
            join = names.join(", "),
            more = more
        ),
    }
}

/// The toggle for the list: an eye, and how many there are to see.
fn eye<V: Choosing>(this: &V, cx: &mut Context<V>) -> impl IntoElement {
    let c = colors();
    let shown = this.list_shown();
    div()
        .id("show-chosen")
        .flex()
        .flex_shrink_0()
        .items_center()
        .gap(ui(6.))
        .h(ui(40.))
        .px(ui(12.))
        .rounded_full()
        .border_1()
        .border_color(rgb(c.outline))
        .bg(rgb(c.surface))
        .text_size(ui(14.))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(c.hover)))
        .tooltip(tip(if shown {
            tr!("Hide what goes in")
        } else {
            tr!("Show what goes in")
        }))
        .on_click(cx.listener(move |this, _, _, cx| {
            this.show_list(!shown);
            cx.notify();
        }))
        .child(icon(if shown { "eye-off" } else { "eye" }, 18., c.text))
        .child(this.selection().paths.len().to_string())
}

/// One thing chosen: its kind, its name, where it is, and a way to take it out.
fn row<V: Choosing>(this: &V, at: usize, cx: &mut Context<V>) -> Option<impl IntoElement + use<V>> {
    let c = colors();
    let path = this.selection().paths.get(at)?.clone();
    let name = path.file_name().map_or_else(
        || path.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    );
    let kind = Kind::of(&name, this.selection().is_folder(&path));
    let folder = path
        .parent()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    let remove = path.clone();
    Some(
        div()
            .id(("chosen", at))
            .w_full()
            .h(px(ROW))
            .flex()
            .items_center()
            .gap(ui(10.))
            .px(ui(10.))
            .rounded(ui(12.))
            .hover(|s| s.bg(rgb(c.well_hover)))
            .child(
                tile(30., 10., kind.tint(&c))
                    .text_size(ui(9.))
                    .font_weight(FontWeight::BOLD)
                    .child(tag_of(&name, kind)),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_size(ui(14.))
                    .font_weight(FontWeight::MEDIUM)
                    .child(name),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_size(ui(13.))
                    .text_color(rgb(c.muted))
                    .child(folder),
            )
            .child(
                icon_button(("remove-chosen", at), "x", 28.)
                    .tooltip(tip(tr!("Take it out")))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.remove(&remove, cx);
                    })),
            ),
    )
}

/// The line, its buttons, and the list if it is shown. `checked`: the dialog was
/// sent with nothing chosen, which it says.
pub fn section<V: Choosing>(this: &V, checked: bool, cx: &mut Context<V>) -> impl IntoElement {
    let c = colors();
    let selection = this.selection();
    let count = selection.paths.len();
    let dragging = cx.has_active_drag();
    let line = if dragging {
        tr!("Drop them to add them")
    } else {
        summary(selection)
    };
    let rows = count.min(MOST_ROWS) as f32;
    let list = (this.list_shown() && count > 0).then(|| {
        div()
            .id("chosen-frame")
            .p(ui(4.))
            .rounded(ui(16.))
            .bg(rgb(c.well))
            // Every scrolling area under the pointer takes the wheel. One that
            // has more to show keeps it, so the dialog holds still under it; a
            // short one has no use for it, and the dialog gets it.
            .when(count > MOST_ROWS, |d| {
                d.on_scroll_wheel(|_, _, cx| cx.stop_propagation())
            })
            .child(
                uniform_list(
                    "chosen-list",
                    count,
                    cx.processor(|this: &mut V, range: std::ops::Range<usize>, _, cx| {
                        range.filter_map(|at| row(this, at, cx)).collect::<Vec<_>>()
                    }),
                )
                .track_scroll(this.list_scroll())
                .w_full()
                .h(px(rows * ROW)),
            )
    });
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
                    div()
                        .flex_1()
                        .min_w_0()
                        .child(legend(tr!("What goes in")))
                        .child(
                            div()
                                .truncate()
                                .text_size(ui(13.))
                                .text_color(rgb(if dragging { c.accent } else { c.muted }))
                                .child(line),
                        ),
                )
                .when(count > 0, |d| d.child(eye(this, cx)))
                .child(
                    button("add-files", tr!("+ Files"), Fill::Outline, Size::Small)
                        .on_click(cx.listener(|this, _, _, cx| this.pick(false, cx))),
                )
                .child(
                    button("add-folder", tr!("+ Folder"), Fill::Outline, Size::Small)
                        .on_click(cx.listener(|this, _, _, cx| this.pick(true, cx))),
                ),
        )
        .children(list)
        .when(count == 0 && checked, |d| {
            d.child(note(Tone::Warn).child(tr!("Add the files or a folder to put in the archive.")))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_line_says_the_first_few_and_how_many_more() {
        let mut s = Selection::default();
        assert_eq!(summary(&s), "Nothing chosen yet");
        s.add(vec!["/a/one".into(), "/a/two".into()]);
        assert_eq!(summary(&s), "one, two");
        s.add(vec!["/a/three".into(), "/a/four".into(), "/a/five".into()]);
        assert_eq!(summary(&s), "one, two, three and 2 more");
    }
}
