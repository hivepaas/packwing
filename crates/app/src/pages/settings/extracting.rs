//! Extracting: where the files go, what happens to a file already there, how
//! big an archive may unpack, whose names it takes, and what a backup gives
//! back.

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, rgb};
use packwing_i18n::tr;

use super::{group, heading, rows, section, setting};
use crate::colors::colors;
use crate::prefs::{ExtractTo, LIMITS, change, prefs};
use crate::text_size::ui;
use crate::widgets::{Chosen, Select, SelectLook, option_card, switch, titled};
use packwing_core::Owners;

/// What hivepack can do with a file that is there, the first what it does
/// unless told.
fn existing() -> [String; 2] {
    [tr!("Stop and say so"), tr!("Replace it")]
}

/// Whose the files extracted are, as [`OWNERS`] holds them in order.
fn owners() -> [String; 3] {
    [tr!("Yours"), tr!("By name"), tr!("By number")]
}

/// What [`owners`] offers, in its order.
const OWNERS: [Owners; 3] = [Owners::Ignore, Owners::ByName, Owners::ByNumber];

/// Whether this system gives files back their owners and extended attributes.
const GIVES_BACK: bool = cfg!(any(target_os = "linux", target_os = "macos"));

pub struct Extracting {
    extract_to: ExtractTo,
    existing: Entity<Select>,
    limit: Entity<Select>,
    portable_names: bool,
    restore_owners: Entity<Select>,
    restore_xattrs: bool,
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
        let restore_owners = cx.new(|_| {
            Select::new(
                &owners(),
                OWNERS
                    .iter()
                    .position(|o| *o == kept.restore_owners)
                    .unwrap_or(0),
                SelectLook::Pill,
            )
            .marked("restore-owners")
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
            cx.subscribe(&restore_owners, |_, select, _: &Chosen, cx| {
                let owners = OWNERS[select.read(cx).chosen().min(OWNERS.len() - 1)];
                change(cx, |p| p.restore_owners = owners);
            }),
        ];
        Extracting {
            extract_to: kept.extract_to,
            existing,
            limit,
            portable_names: kept.portable_names,
            restore_owners,
            restore_xattrs: kept.restore_xattrs,
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
            .child(rows(
                [
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
                    setting(
                        tr!("Only names every system can use"),
                        tr!(
                            "Names Windows can’t use, such as a:b.txt, and names differing only \
                         in case are asked about on this computer too"
                        ),
                        switch("portable-names", self.portable_names).on_click(cx.listener(
                            |this, _, _, cx| {
                                this.portable_names = !this.portable_names;
                                let on = this.portable_names;
                                change(cx, |p| p.portable_names = on);
                                cx.notify();
                            },
                        )),
                    ),
                ]
                .into_iter()
                .chain(GIVES_BACK.then(|| {
                    setting(
                        tr!("Owners from a backup"),
                        tr!(
                            "Where a tar archive kept them: yours, or the archive’s by name \
                             or by number. Only an administrator gives files to others"
                        ),
                        self.restore_owners.clone(),
                    )
                }))
                .chain(GIVES_BACK.then(|| {
                    setting(
                        tr!("Extended attributes from a backup"),
                        tr!("Where a tar archive kept them, ACLs among them"),
                        switch("restore-xattrs", self.restore_xattrs).on_click(cx.listener(
                            |this, _, _, cx| {
                                this.restore_xattrs = !this.restore_xattrs;
                                let on = this.restore_xattrs;
                                change(cx, |p| p.restore_xattrs = on);
                                cx.notify();
                            },
                        )),
                    )
                })),
            ))
    }
}
