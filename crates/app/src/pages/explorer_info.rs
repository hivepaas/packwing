//! What is known about what is chosen in an [`Explorer`], and what to do with it:
//! the panel beside it, on the pages that browse.

use gpui::{App, Context, Div, Entity, FontWeight, Window, div, prelude::*, rgb};
use packwing_core::{Item, Signed};
use packwing_i18n::{tr, tr_n};

use super::{panel, title};
use crate::colors::colors;
use crate::explorer::{Explorer, can_be_signed, is_archive, runs_when_opened};
use crate::model::{Kind, date, size, tag_of};
use crate::signatures::{self, Standing};
use crate::text_size::ui;
use crate::widgets::{Fill, Size, button, tile};

fn transfers(explorer: &Entity<Explorer>) -> Div {
    let (copy, moved) = (explorer.clone(), explorer.clone());
    div()
        .flex()
        .gap(ui(8.))
        .child(
            button("info-copy", tr!("Copy to…"), Fill::Outline, Size::Small)
                .flex_1()
                .on_click(move |_, _, cx| copy.update(cx, |e, cx| e.ask_transfer(false, cx))),
        )
        .child(
            button("info-move", tr!("Move to…"), Fill::Outline, Size::Small)
                .flex_1()
                .on_click(move |_, _, cx| moved.update(cx, |e, cx| e.ask_transfer(true, cx))),
        )
}

/// What is known about what is chosen in `explorer`, and what to do with it.
pub fn info(explorer: &Entity<Explorer>, window: &Window, cx: &App) -> Div {
    let c = colors();
    let selection = explorer.read(cx).selection();
    let body = panel()
        // A quarter of the window, as it grows; its buttons need the least.
        .w(window.viewport_size().width * 0.25)
        .min_w(ui(280.))
        .flex_shrink_0()
        .flex()
        .flex_col()
        .gap(ui(14.))
        .p(ui(18.));
    match selection.as_slice() {
        [] => body.child(
            div()
                .m_auto()
                .p(ui(16.))
                .text_center()
                .text_size(ui(13.))
                .text_color(rgb(c.muted))
                .child(tr!("Choose a file to see its information here")),
        ),
        [one] => info_of(explorer, one, body, cx),
        many => {
            let compress = explorer.clone();
            let delete = explorer.clone();
            let bytes: u64 = many.iter().map(|i| i.size).sum();
            body.child(title(
                tr_n!(many.len() as u64, "{n} selected", "{n} selected"),
                18.,
            ))
            .child(
                div()
                    .text_size(ui(13.))
                    .text_color(rgb(c.muted))
                    .child(tr!("Files in them come to {size}", size = size(bytes))),
            )
            .child(
                button("info-compress", tr!("Compress…"), Fill::Tonal, Size::Small)
                    .on_click(move |_, _, cx| compress.update(cx, |e, cx| e.compress(cx))),
            )
            .child(
                button("info-delete", tr!("Delete…"), Fill::Outline, Size::Small)
                    .on_click(move |_, _, cx| delete.update(cx, |e, cx| e.ask_delete(cx))),
            )
            .child(transfers(explorer))
        }
    }
}

fn info_of(entity: &Entity<Explorer>, item: &Item, body: Div, cx: &App) -> Div {
    let c = colors();
    let explorer = entity.read(cx);
    let kind = Kind::of(&item.name, item.dir);
    let row = |key: String, value: String| {
        div()
            .flex()
            .gap(ui(12.))
            .text_size(ui(13.))
            .child(
                div()
                    .w(ui(84.))
                    .flex_shrink_0()
                    .text_color(rgb(c.muted))
                    .child(key),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .font_weight(FontWeight::MEDIUM)
                    .child(value),
            )
    };
    let what = match () {
        _ if item.link && item.dir => tr!("Link to a folder"),
        _ if item.link => tr!("Link"),
        _ if item.dir => tr!("Folder"),
        _ => tr!("{tag} file", tag = tag_of(&item.name, kind)),
    };
    let counted = item.dir.then(|| explorer.count_of(&item.name)).flatten();
    let count_rows = counted.as_ref().map(|shown| {
        let n = shown.counted;
        let holds = tr!(
            "{files}, {folders}",
            files = tr_n!(n.files, "{n} file", "{n} files"),
            folders = tr_n!(n.folders, "{n} folder", "{n} folders")
        );
        let bytes = if shown.done {
            size(n.bytes)
        } else {
            tr!("{size} so far", size = size(n.bytes))
        };
        (bytes, holds, shown.unread)
    });
    let mut rows = vec![row(tr!("Kind"), what)];
    if !item.dir {
        rows.push(row(tr!("Size"), size(item.size)));
    }
    if let Some((bytes, holds, _)) = &count_rows {
        rows.push(row(tr!("Size"), bytes.clone()));
        rows.push(row(tr!("Holds"), holds.clone()));
    }
    if let Some(when) = item.modified {
        rows.push(row(tr!("Modified"), date(when)));
    }
    if let Some(mode) = item.mode {
        rows.push(row(tr!("Permissions"), permissions(mode)));
    }
    rows.push(row(tr!("Path"), explorer.shown_path_of(&item.name)));
    let signature = explorer.signature_of(&item.name);
    let looking = matches!(signature, Some(None));
    let not_checked = matches!(signature, Some(Some(Signed::NotChecked { .. })));
    let signature_said = match signature {
        Some(Some(signed)) => signatures::said(signed, cx),
        _ => None,
    };
    if looking {
        rows.push(row(tr!("Signature"), signatures::looking_words()));
    }
    let signature_row = signature_said.map(|(words, standing)| {
        let ink = match standing {
            Standing::Good | Standing::Plain => c.text,
            Standing::Warn | Standing::Bad => standing.colors().1,
        };
        row(tr!("Signature"), words).text_color(rgb(ink))
    });
    rows.extend(signature_row);
    let unread = count_rows.as_ref().map_or(0, |(_, _, unread)| *unread);
    let handle = entity.clone();
    let on = |f: fn(&mut Explorer, &Item, &mut Context<Explorer>)| {
        let (handle, item) = (handle.clone(), item.clone());
        move |_: &gpui::ClickEvent, _: &mut Window, cx: &mut gpui::App| {
            handle.update(cx, |e, cx| f(e, &item, cx));
        }
    };
    let open_label = if item.dir || is_archive(item) {
        tr!("Open")
    } else {
        tr!("View")
    };
    let local = explorer.is_local();
    let mut actions = div()
        .flex()
        .flex_col()
        .gap(ui(8.))
        .when(local || !is_archive(item), |d| {
            d.child(
                button("info-open", open_label, Fill::Accent, Size::Small)
                    .on_click(on(|e, i, cx| e.open(i, cx))),
            )
        });
    if is_archive(item) && local {
        actions = actions.child(
            button("info-extract", tr!("Extract…"), Fill::Tonal, Size::Small)
                .on_click(on(|e, i, cx| e.extract(i, cx))),
        );
    }
    if !item.dir && !is_archive(item) && local {
        actions = actions.child(
            button(
                "info-edit",
                Explorer::edit_label(cx),
                Fill::Tonal,
                Size::Small,
            )
            .on_click(on(|e, i, cx| e.edit(i, cx))),
        );
        if !runs_when_opened(item) {
            actions = actions.child(
                button(
                    "info-app",
                    tr!("Open with default app"),
                    Fill::Outline,
                    Size::Small,
                )
                .on_click(on(|e, i, cx| e.open_with_system(i, cx))),
            );
        }
    }
    if item.dir && counted.is_none() {
        let (handle, name) = (entity.clone(), item.name.clone());
        actions = actions.child(
            button(
                "info-count",
                tr!("Count size and files"),
                Fill::Outline,
                Size::Small,
            )
            .on_click(move |_, _, cx| {
                handle.update(cx, |e, cx| e.count_folder(&name, cx));
            }),
        );
    }
    body.child(
        div()
            .flex()
            .items_center()
            .gap(ui(12.))
            .child(
                tile(48., 16., kind.tint(&c))
                    .text_size(ui(12.))
                    .font_weight(FontWeight::BOLD)
                    .child(tag_of(&item.name, kind)),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .text_size(ui(16.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(item.name.clone()),
            ),
    )
    .child(div().flex().flex_col().gap(ui(7.)).children(rows))
    .when(unread > 0, |d| {
        d.child(
            div()
                .text_size(ui(13.))
                .text_color(rgb(c.muted))
                .child(tr_n!(
                    unread,
                    "{n} folder in it could not be read.",
                    "{n} folders in it could not be read."
                )),
        )
    })
    .child(
        actions
            .when(not_checked, |d| {
                let handle = entity.clone();
                d.child(
                    button(
                        "info-check-signature",
                        tr!("Check the signature"),
                        Fill::Tonal,
                        Size::Small,
                    )
                    .on_click(move |_, _, cx| handle.update(cx, |e, cx| e.check_signature(cx))),
                )
            })
            .child(transfers(entity))
            // A row of its own: three would not fit the panel at its narrowest.
            .when(can_be_signed(item), |d| {
                let handle = entity.clone();
                d.child(
                    button("info-sign", tr!("Sign…"), Fill::Outline, Size::Small)
                        .on_click(move |_, _, cx| handle.update(cx, |e, cx| e.ask_sign(cx))),
                )
            }),
    )
}

/// Unix permissions as `ls` writes them.
fn permissions(mode: u32) -> String {
    (0..9)
        .map(|bit| {
            if mode & (0o400 >> bit) == 0 {
                '-'
            } else {
                ['r', 'w', 'x'][bit % 3]
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permissions_read_as_ls_writes_them() {
        assert_eq!(permissions(0o644), "rw-r--r--");
        assert_eq!(permissions(0o755), "rwxr-xr-x");
        assert_eq!(permissions(0o000), "---------");
    }
}
