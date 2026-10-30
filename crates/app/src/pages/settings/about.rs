//! About: what packwing is, its version, and its license.

use gpui::{Context, FontWeight, Render, Window, div, prelude::*, px, relative, rgb};

use super::{group, rows, section, setting};
use crate::colors::colors;
use crate::widgets::{Fill, Size, accent_tint, button, logo, tile};

const SOURCE: &str = "https://github.com/hivepaas/packwing";
const LICENSE: &str = "https://github.com/hivepaas/packwing/blob/main/LICENSE";

pub struct About;

impl Render for About {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let link = |id: &'static str, name: &'static str, url: &'static str| {
            button(id, name, Fill::Outline, Size::Small).on_click(move |_, _, cx| cx.open_url(url))
        };
        section()
            .child(
                group()
                    .flex_row()
                    .items_center()
                    .gap(px(18.))
                    .py(px(20.))
                    .child(tile(72., 24., accent_tint()).child(logo(36., accent_tint())))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(2.))
                            .child(
                                div()
                                    .text_size(px(28.))
                                    .font_weight(FontWeight::BOLD)
                                    .line_height(relative(1.2))
                                    .child("packwing"),
                            )
                            .child(
                                div()
                                    .text_size(px(14.))
                                    .text_color(rgb(c.muted))
                                    .child(format!("Version {}", env!("CARGO_PKG_VERSION"))),
                            )
                            .child(
                                div()
                                    .pt(px(6.))
                                    .text_size(px(15.))
                                    .child("A file archiver that opens archives safely."),
                            ),
                    ),
            )
            .child(rows([
                setting(
                    "Free software",
                    "Under the GNU General Public License, version 3 or later",
                    link("license", "View license", LICENSE),
                ),
                setting(
                    "Made by HivePaaS",
                    "On hivepack, our library for reading and writing archives",
                    link("source", "Source code", SOURCE),
                ),
            ]))
    }
}
