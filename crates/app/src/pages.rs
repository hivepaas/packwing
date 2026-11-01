//! The main window's pages, each a view: the open archive, this computer's
//! folders and the archives opened before, Settings, and the first run's welcome.

pub mod archive;
pub mod browse;
mod explorer_info;
pub mod onboarding;
pub mod recent;
pub mod settings;
pub mod storages;

use gpui::{Div, FontWeight, SharedString, div, prelude::*, px, rgb};

use crate::colors::colors;

/// A page's or a card's big title.
pub fn title(text: impl Into<SharedString>, size: f32) -> Div {
    div()
        .text_size(px(size))
        .font_weight(FontWeight::BOLD)
        .child(text.into())
}

/// A white card on the page.
pub fn panel() -> Div {
    div().rounded(px(28.)).bg(rgb(colors().surface))
}
