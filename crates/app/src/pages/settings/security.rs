//! Security: what packwing checks and keeps when it extracts. Where an
//! organization runs packwing, what it fixed is shown locked, with its name
//! in the policy file, and only the rest can be changed.

use gpui::{Context, FontWeight, Render, SharedString, Window, div, prelude::*, px, relative, rgb};

use super::{rows, section, setting};
use crate::colors::{Tint, colors};
use crate::demo::{Managed, count};
use crate::widgets::{Fill, Size, button, icon, switch, tile};

/// Monospace fonts for the policies' names, the first one installed.
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

struct Toggle {
    label: &'static str,
    detail: &'static str,
    on: bool,
}

pub struct Security {
    managed: Option<Managed>,
    /// The monospace font found, once looked for.
    mono: Option<SharedString>,
    /// The settings the user may change; with policies, only the last.
    toggles: Vec<Toggle>,
}

impl Security {
    pub fn new(managed: Option<Managed>) -> Self {
        let mut toggles = vec![
            Toggle {
                label: "Check archives before extracting",
                detail: "Blocks files that try to escape the folder, or reach out of it \
                         through links",
                on: true,
            },
            Toggle {
                label: "Keep the “downloaded from the Internet” mark",
                detail: "The system warns before running files that came from a downloaded \
                         archive",
                on: true,
            },
            Toggle {
                label: "Block programs from Internet archives",
                detail: ".exe, .msi, .js and .lnk files from downloaded archives are not \
                         extracted",
                on: false,
            },
            Toggle {
                label: "Ask before opening files without extracting",
                detail: "Opening a file inside an archive puts a copy in a temporary folder",
                on: true,
            },
        ];
        if managed.is_some() {
            // The others are the organization's to set.
            toggles.drain(..toggles.len() - 1);
            if let Some(t) = toggles.last_mut() {
                t.detail = "You can change this one";
            }
        }
        Security {
            managed,
            mono: None,
            toggles,
        }
    }

    fn banner(m: &Managed) -> impl IntoElement {
        let c = colors();
        div()
            .flex()
            .items_center()
            .gap(px(14.))
            .px(px(18.))
            .py(px(16.))
            .rounded(px(22.))
            .bg(rgb(c.warn_bg))
            .text_color(rgb(c.warn_text))
            .child(
                tile(
                    44.,
                    15.,
                    Tint {
                        ground: c.surface,
                        ink: c.warn_text,
                    },
                )
                .child(icon("building", 22., c.warn_text)),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .text_size(px(14.))
                    .line_height(relative(1.5))
                    .child(
                        div()
                            .text_size(px(15.))
                            .font_weight(FontWeight::BOLD)
                            .child("Some settings are managed by your organization"),
                    )
                    .child(format!("{} · Questions? {}", m.by, m.contact)),
            )
            .child(
                button("policies", "View policies", Fill::Raised, Size::Small)
                    .text_color(rgb(c.warn_text)),
            )
    }

    /// What a policy fixed the setting at, with a lock, and its name.
    fn locked(value: &'static str, key: &'static str, mono: SharedString) -> impl IntoElement {
        let c = colors();
        div()
            .flex()
            .flex_col()
            .items_end()
            .gap(px(3.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .px(px(12.))
                    .py(px(4.))
                    .rounded_full()
                    .bg(rgb(c.surface))
                    .text_size(px(13.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .whitespace_nowrap()
                    .child(icon("lock", 13., c.text))
                    .child(value),
            )
            .child(
                div()
                    .font_family(mono)
                    .text_size(px(11.))
                    .text_color(rgb(c.muted))
                    .child(key),
            )
    }
}

impl Render for Security {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mono = self
            .mono
            .get_or_insert_with(|| {
                let installed = window.text_system().all_font_names();
                MONO_FONTS
                    .iter()
                    .find(|f| installed.iter().any(|i| i == *f))
                    .copied()
                    .unwrap_or(crate::FONT)
                    .into()
            })
            .clone();
        let open = self.toggles.iter().enumerate().map(|(i, t)| {
            setting(
                t.label,
                t.detail,
                switch(("security", i), t.on).on_click(cx.listener(move |this, _, _, cx| {
                    this.toggles[i].on = !this.toggles[i].on;
                    cx.notify();
                })),
            )
        });
        let Some(m) = self.managed else {
            return section().child(rows(open));
        };
        let fixed = m.policies.iter().map(|p| {
            setting(
                p.label,
                p.detail,
                Self::locked(p.value, p.key, mono.clone()),
            )
        });
        let c = colors();
        section()
            .gap(px(16.))
            .child(Self::banner(&m))
            .child(rows(fixed.chain(open)))
            .child(div().flex_1())
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_x(px(20.))
                    .gap_y(px(6.))
                    .text_size(px(13.))
                    .text_color(rgb(c.muted))
                    .child(format!(
                        "Version {} · {} channel",
                        env!("CARGO_PKG_VERSION"),
                        m.channel
                    ))
                    .child(format!(
                        "Enterprise license: {} · {} seats",
                        m.organization,
                        count(m.seats)
                    ))
                    .child("Crypto module: FIPS 140-3"),
            )
    }
}
