//! A damaged archive: what still reads, what is missing, and the way on.

use gpui::{
    Context, EventEmitter, FontWeight, HighlightStyle, PathPromptOptions, Render, StyledText,
    Window, div, prelude::*, px, relative, rgb,
};

use super::{DialogEvent, Job, Verb, card, footer};
use crate::colors::{Tint, colors};
use crate::demo::{Damaged, count, counted};
use crate::widgets::{Fill, Size, button, icon, legend, link, option_card, tile, titled};

pub struct DamagedDialog {
    damaged: Damaged,
    /// Repair was chosen, rather than taking the files that read.
    repair: bool,
}

impl EventEmitter<DialogEvent> for DamagedDialog {}

/// `text` with the part `strong` in `weight` and `color`.
fn with_strong(text: String, strong: &str, color: u32, weight: FontWeight) -> StyledText {
    let style = HighlightStyle {
        color: Some(rgb(color).into()),
        font_weight: Some(weight),
        ..Default::default()
    };
    let at = text.find(strong).map(|i| (i..i + strong.len(), style));
    StyledText::new(text).with_highlights(at)
}

impl DamagedDialog {
    pub fn new(damaged: Damaged) -> Self {
        DamagedDialog {
            damaged,
            repair: false,
        }
    }

    fn start(&mut self, cx: &mut Context<Self>) {
        let d = self.damaged;
        let files = if self.repair { d.files } else { d.fine };
        cx.emit(DialogEvent::Start(Job {
            verb: if self.repair {
                Verb::Repair
            } else {
                Verb::Extract
            },
            name: d.name.into(),
            to: Some(format!("{} › {}", d.location, crate::demo::stem(d.name)).into()),
            files,
            bytes: d.unpacked * files / d.files.max(1),
        }));
    }

    fn find_part(&mut self, cx: &mut Context<Self>) {
        // Nothing is done with it yet: finding the part means reading it.
        let picked = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Use this part".into()),
        });
        cx.spawn(async move |_, _| {
            let _ = picked.await;
        })
        .detach();
    }
}

impl Render for DamagedDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let d = self.damaged;
        let part = d.missing_number();
        let fine = format!("{} of {} files are fine.", count(d.fine), count(d.files));
        let summary = format!(
            "{} · {} {} in “{}” can’t be read yet.",
            d.name,
            fine,
            counted(d.files - d.fine, "file", "files"),
            d.folder
        );
        let missing = format!("Part {part} is missing.");
        let missing_text = format!(
            "{missing} We couldn’t find “{}” next to the other parts.",
            d.missing_part
        );
        let bad = Tint {
            ground: c.bad_bg,
            ink: c.bad_text,
        };
        let primary = if self.repair {
            "Repair".to_string()
        } else {
            format!("Extract {}", counted(d.fine, "file", "files"))
        };
        card()
            .gap(px(18.))
            .child(
                div()
                    .flex()
                    .items_start()
                    .gap(px(14.))
                    .child(tile(48., 16., bad).child(icon("warning", 24., bad.ink)))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .child(
                                div()
                                    .text_size(px(22.))
                                    .font_weight(FontWeight::BOLD)
                                    .child("Part of this archive is damaged"),
                            )
                            .child(
                                div()
                                    .mt(px(4.))
                                    .text_size(px(15.))
                                    .line_height(relative(1.5))
                                    .text_color(rgb(c.muted))
                                    .child(with_strong(
                                        summary,
                                        &fine,
                                        c.text,
                                        FontWeight::SEMIBOLD,
                                    )),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(14.))
                    .px(px(16.))
                    .py(px(14.))
                    .rounded(px(20.))
                    .bg(rgb(c.warn_bg))
                    .text_color(rgb(c.warn_text))
                    .child(icon("search", 22., c.warn_text))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .text_size(px(14.))
                            .line_height(relative(1.45))
                            .child(with_strong(
                                missing_text,
                                &missing,
                                c.warn_text,
                                FontWeight::BOLD,
                            )),
                    )
                    .child(
                        button(
                            "find-part",
                            format!("Find part {part}…"),
                            Fill::Raised,
                            Size::Small,
                        )
                        .text_color(rgb(c.warn_text))
                        .on_click(cx.listener(|this, _, _, cx| this.find_part(cx))),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(legend("What would you like to do?"))
                    .child(
                        option_card("take-fine", !self.repair)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.repair = false;
                                cx.notify();
                            }))
                            .child(titled(
                                "Extract the files that are fine",
                                format!("You can add the rest later, once you have part {part}"),
                                !self.repair,
                            )),
                    )
                    .when(d.recoverable, |el| {
                        el.child(
                            option_card("repair", self.repair)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.repair = true;
                                    cx.notify();
                                }))
                                .child(titled(
                                    "Try to repair",
                                    "This archive has recovery data. Repair makes a fixed copy; \
                                     the original isn’t changed.",
                                    self.repair,
                                )),
                        )
                    }),
            )
            .child(
                footer()
                    .child(link("show-damaged", "Show damaged files"))
                    .child(div().flex_1())
                    .child(
                        button("close", "Close", Fill::Text, Size::Medium)
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close))),
                    )
                    .child(
                        button("go", primary, Fill::Accent, Size::Medium)
                            .px(px(26.))
                            .on_click(cx.listener(|this, _, _, cx| this.start(cx))),
                    ),
            )
    }
}
