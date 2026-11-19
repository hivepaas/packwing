//! A question a connection over SSH asks while it connects: whether to trust a
//! server packwing has not met, or the password, passphrase or code it needs.
//! It shows over any dialog, as a job's connection may ask it.

use gpui::{
    Context, Entity, EventEmitter, Focusable as _, FontWeight, KeyDownEvent, Render, Window, div,
    prelude::*, rgb,
};
use packwing_i18n::tr;
use packwing_remote::Typed;

use super::{card, footer, header};
use crate::colors::colors;
use crate::input::TextInput;
use crate::ssh::{Answer, Asked};
use crate::text_size::ui;
use crate::widgets::{
    Fill, Size, Tone, accent_tint, button, check_row, icon_button, legend, mono_font, note,
    tonal_tint,
};

/// What the user answered.
pub struct QuestionEvent(pub Answer);

pub struct SshQuestionDialog {
    asked: Asked,
    /// What is typed, for all but a host's key.
    field: Option<Entity<TextInput>>,
    /// Keep what is typed in the keychain once it works.
    keep: bool,
}

impl EventEmitter<QuestionEvent> for SshQuestionDialog {}

/// `host`, or `host:port` when it is not SSH's own port.
fn host_named(host: &str, port: u16) -> String {
    if port == 22 {
        host.to_owned()
    } else {
        format!("{host}:{port}")
    }
}

impl SshQuestionDialog {
    pub fn new(asked: Asked, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let field = match &asked {
            Asked::HostKey(_) => None,
            Asked::Password(_) => Some(cx.new(|cx| TextInput::new(tr!("Password"), cx).masked())),
            Asked::Passphrase(_) => {
                Some(cx.new(|cx| TextInput::new(tr!("Passphrase"), cx).masked()))
            }
            Asked::Code(q) => {
                let echo = q.echo;
                Some(cx.new(|cx| {
                    let input = TextInput::new(tr!("Answer"), cx);
                    if echo { input } else { input.masked() }
                }))
            }
        };
        if let Some(field) = &field {
            // Typing goes in the field at once.
            window.focus(&field.focus_handle(cx), cx);
        }
        SshQuestionDialog {
            asked,
            field,
            keep: false,
        }
    }

    /// What is asked, for tests.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn asked(&self) -> &Asked {
        &self.asked
    }

    fn typed(&self, cx: &Context<Self>) -> String {
        self.field
            .as_ref()
            .map(|f| f.read(cx).text().to_owned())
            .unwrap_or_default()
    }

    /// Says yes: trusts the server, or gives what is typed.
    fn yes(&mut self, cx: &mut Context<Self>) {
        let typed = self.typed(cx);
        let answer = match &self.asked {
            Asked::HostKey(_) => Answer::Trust(true),
            Asked::Password(_) if typed.is_empty() => return,
            Asked::Password(_) | Asked::Passphrase(_) => Answer::Typed(Some(Typed {
                secret: typed,
                keep: self.keep,
            })),
            Asked::Code(_) => Answer::Code(Some(typed)),
        };
        cx.emit(QuestionEvent(answer));
    }

    /// Says no: the server is not trusted, or nothing is given.
    fn no(&mut self, cx: &mut Context<Self>) {
        cx.emit(QuestionEvent(self.asked.no()));
    }

    /// Enter says yes, Escape no: the field leaves both to its holder.
    fn on_key(&mut self, e: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        match e.keystroke.key.as_str() {
            "enter" => self.yes(cx),
            "escape" => self.no(cx),
            _ => return,
        }
        cx.stop_propagation();
    }
}

impl Render for SshQuestionDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let close = icon_button("close", "x", 40., tr!("Close"))
            .on_click(cx.listener(|this, _, _, cx| this.no(cx)));
        let field = self.field.clone().map(|input| {
            div()
                .flex()
                .items_center()
                .h(ui(40.))
                .px(ui(18.))
                .rounded(ui(16.))
                .bg(rgb(c.field))
                .text_size(ui(16.))
                .child(input)
        });
        let keep = check_row(
            "ssh-keep",
            self.keep,
            tr!("Keep it in the system’s keychain"),
        )
        .on_click(cx.listener(|this, _, _, cx| {
            this.keep = !this.keep;
            cx.notify();
        }));
        let (icon_name, tint, title, detail, body, yes, no) = match &self.asked {
            Asked::HostKey(q) => (
                "shield",
                accent_tint(),
                tr!("Trust this server?"),
                host_named(&q.host, q.port),
                div()
                    .flex()
                    .flex_col()
                    .gap(ui(12.))
                    .child(
                        div()
                            .text_size(ui(14.))
                            .text_color(rgb(c.muted))
                            .child(tr!(
                                "packwing has not connected to this server before. Trust it only if this is its key: whoever runs the server can tell you its fingerprint."
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(ui(6.))
                            .child(legend(tr!(
                                "{algorithm} key fingerprint",
                                algorithm = q.algorithm
                            )))
                            .child(
                                div()
                                    .debug_selector(|| "ssh-fingerprint".to_owned())
                                    .p(ui(14.))
                                    .rounded(ui(16.))
                                    .bg(rgb(c.field))
                                    .font_family(mono_font(window))
                                    .text_size(ui(14.))
                                    .font_weight(FontWeight::MEDIUM)
                                    .child(q.fingerprint.clone()),
                            ),
                    ),
                tr!("Trust and connect"),
                tr!("Don’t connect"),
            ),
            Asked::Password(q) => (
                "lock",
                tonal_tint(),
                tr!("Sign in to the server"),
                q.target.clone(),
                div()
                    .flex()
                    .flex_col()
                    .gap(ui(12.))
                    .when(q.retry, |d| {
                        d.child(note(Tone::Warn).child(tr!("The server did not take that password.")))
                    })
                    .child(legend(tr!("Password")))
                    .children(field)
                    .child(keep),
                tr!("Sign in"),
                tr!("Cancel"),
            ),
            Asked::Passphrase(q) => (
                "lock",
                tonal_tint(),
                tr!("Enter the key’s passphrase"),
                q.key.clone(),
                div()
                    .flex()
                    .flex_col()
                    .gap(ui(12.))
                    .when(q.retry, |d| {
                        d.child(note(Tone::Warn).child(tr!("That passphrase does not open the key.")))
                    })
                    .child(legend(tr!("Passphrase")))
                    .children(field)
                    .child(keep),
                tr!("Open"),
                tr!("Cancel"),
            ),
            Asked::Code(q) => (
                "lock",
                tonal_tint(),
                if q.name.trim().is_empty() {
                    tr!("The server asks")
                } else {
                    q.name.clone()
                },
                q.instructions.clone(),
                div()
                    .flex()
                    .flex_col()
                    .gap(ui(12.))
                    .child(legend(q.text.trim().to_owned()))
                    .children(field),
                tr!("Send"),
                tr!("Cancel"),
            ),
        };
        card()
            .id("ssh-question")
            .debug_selector(|| "ssh-question".to_owned())
            .w(ui(540.))
            .on_key_down(cx.listener(Self::on_key))
            .child(header(icon_name, tint, title, detail, Some(close)))
            .child(body)
            .child(
                footer()
                    .child(div().flex_1())
                    .child(
                        button("ssh-no", no, Fill::Text, Size::Medium)
                            .on_click(cx.listener(|this, _, _, cx| this.no(cx))),
                    )
                    .child(
                        button("ssh-yes", yes, Fill::Accent, Size::Medium)
                            .px(ui(28.))
                            .on_click(cx.listener(|this, _, _, cx| this.yes(cx))),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_host_is_named_with_its_port_when_it_is_not_22() {
        assert_eq!(host_named("example.com", 22), "example.com");
        assert_eq!(host_named("127.0.0.1", 2222), "127.0.0.1:2222");
    }
}
