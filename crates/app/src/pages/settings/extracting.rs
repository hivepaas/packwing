//! Extracting: where the files go, what happens to a file already there, and
//! how big an archive may unpack.

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, rgb};
use packwing_i18n::tr;

use super::{group, heading, rows, section, setting};
use crate::colors::colors;
use crate::prefs::{ExtractTo, LIMITS, change, prefs};
use crate::text_size::ui;
use crate::widgets::{Chosen, Select, SelectLook, option_card, titled};

/// What hivepack can do with a file that is there, the first what it does
/// unless told.
fn existing() -> [String; 2] {
    [tr!("Stop and say so"), tr!("Replace it")]
}

pub struct Extracting {
    extract_to: ExtractTo,
    existing: Entity<Select>,
    limit: Entity<Select>,
    _chosen: Vec<Subscription>,
}

impl Extracting {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let kept = prefs(cx);
        let existing = cx.new(|_| {
            Select::new(&existing(), usize::from(kept.overwrite), SelectLook::Pill)
                .marked("existing")
        });
        let limit = cx.new(|_| {
            Select::new(
                &LIMITS.map(|(name, limit)| match limit {
                    Some(_) => name.to_owned(),
                    None => tr!("No limit"),
                }),
                kept.limit,
                SelectLook::Pill,
            )
        });
        let chosen = vec![
            cx.subscribe(&existing, |_, select, _: &Chosen, cx| {
                let overwrite = select.read(cx).chosen() == 1;
                change(cx, |p| p.overwrite = overwrite);
            }),
            cx.subscribe(&limit, |_, select, _: &Chosen, cx| {
                let limit = select.read(cx).chosen();
                change(cx, |p| p.limit = limit);
            }),
        ];
        Extracting {
            extract_to: kept.extract_to,
            existing,
            limit,
            _chosen: chosen,
        }
    }
}

impl Render for Extracting {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let place = |id: &'static str, to: ExtractTo, title: String, detail: String| {
            let on = self.extract_to == to;
            option_card(id, on)
                .px(ui(14.))
                .py(ui(12.))
                .rounded(ui(18.))
                .when(!on, |d| d.bg(rgb(c.surface)))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.extract_to = to;
                    change(cx, |p| p.extract_to = to);
                    cx.notify();
                }))
                .child(titled(title, detail, on))
        };
        section()
            .child(
                group()
                    .child(heading(
                        tr!("Where the files go"),
                        tr!("Where Extract starts: you can pick another there."),
                    ))
                    .child(
                        div()
                            .grid()
                            .grid_cols(3)
                            .gap(ui(8.))
                            .child(place(
                                "smart",
                                ExtractTo::Smart,
                                tr!("Smart"),
                                tr!("One item right here, more in a new folder"),
                            ))
                            .child(place(
                                "into-new-folder",
                                ExtractTo::NewFolder,
                                tr!("A new folder next to it"),
                                tr!("Named after the archive"),
                            ))
                            .child(place(
                                "beside",
                                ExtractTo::Here,
                                tr!("Next to the archive"),
                                tr!("Straight into its folder"),
                            )),
                    ),
            )
            .child(rows([
                setting(
                    tr!("When a file is already there"),
                    tr!("A file of the same name in the folder the files go to"),
                    self.existing.clone(),
                ),
                setting(
                    tr!("Largest an archive may unpack to"),
                    tr!("Stops archive bombs: small files that unpack to fill the disk"),
                    self.limit.clone(),
                ),
            ]))
    }
}
