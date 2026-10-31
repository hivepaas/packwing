//! The passphrase or password an encrypted archive needs: asked for when
//! it turns out to need one, and again, with why, when the one typed is
//! wrong.

use gpui::{
    Context, Entity, EventEmitter, Focusable as _, KeyDownEvent, Render, SharedString, Window, div,
    prelude::*, px, rgb,
};
use packwing_core::{Asked, Error, Keys};

use super::{card, footer, header};
use crate::colors::colors;
use crate::input::TextInput;
use crate::widgets::{Fill, Size, Tone, button, icon, icon_button, legend, note, tonal_tint};

/// What the user chose.
pub enum PasswordEvent {
    /// Open with what was typed.
    Typed(Vec<u8>),
    Cancelled,
}

pub struct PasswordDialog {
    asked: Asked,
    /// The archive, as its name says it.
    name: SharedString,
    /// Why it is asked again: the one typed before was wrong.
    again: Option<SharedString>,
    field: Entity<TextInput>,
    /// The text is shown as typed.
    shown: bool,
}

impl EventEmitter<PasswordEvent> for PasswordDialog {}

impl PasswordDialog {
    pub fn new(
        asked: Asked,
        name: impl Into<SharedString>,
        again: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let field = cx.new(|cx| TextInput::new(placeholder(asked), cx).masked());
        // Typing goes in the field at once.
        window.focus(&field.focus_handle(cx), cx);
        PasswordDialog {
            asked,
            name: name.into(),
            again: again.map(Into::into),
            field,
            shown: false,
        }
    }

    /// Opens with what is typed: nothing typed, nothing to open with.
    fn open(&mut self, cx: &mut Context<Self>) {
        let typed = self.field.read(cx).text().as_bytes().to_vec();
        if typed.is_empty() {
            return;
        }
        cx.emit(PasswordEvent::Typed(typed));
    }

    /// Enter opens, Escape gives up: the field leaves both to its holder.
    fn on_key(&mut self, e: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        match e.keystroke.key.as_str() {
            "enter" => self.open(cx),
            "escape" => cx.emit(PasswordEvent::Cancelled),
            _ => return,
        }
        cx.stop_propagation();
    }
}

/// What a person can type to get past `e`, met with `keys`, and whether
/// they typed it before (it was wrong); `None` for what typing does not
/// open: a key packwing does not have, damage, anything else.
pub fn asked_for(e: &Error, keys: &Keys) -> Option<(Asked, bool)> {
    match e {
        Error::Locked { needs } => {
            let asked = Keys::asks_for(needs)?;
            Some((asked, false))
        }
        // Only a password typed can be the wrong one: a ZIP or 7z archive's
        // damaged data with none says nothing of passwords.
        Error::WrongPassword if keys.password.is_some() => Some((Asked::Password, true)),
        Error::WrongPassphrase => Some((Asked::Passphrase, true)),
        _ => None,
    }
}

/// What the field says before anything is typed.
fn placeholder(asked: Asked) -> &'static str {
    match asked {
        Asked::Passphrase => "The passphrase",
        Asked::Password => "The archive’s password",
    }
}

/// The dialog's title, the field's label, and what it says under it.
fn words(asked: Asked) -> (&'static str, &'static str, &'static str) {
    match asked {
        Asked::Passphrase => (
            "Enter the passphrase",
            "Passphrase",
            "This file is encrypted with age. packwing keeps the passphrase only while the \
             archive is open, and never writes it down.",
        ),
        Asked::Password => (
            "Enter the password",
            "Password",
            "This archive is encrypted. packwing keeps its password only while the archive is \
             open, and never writes it down.",
        ),
    }
}

impl Render for PasswordDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let (title, label, why) = words(self.asked);
        let shown = self.shown;
        let close = icon_button("close", "x", 44.)
            .on_click(cx.listener(|_, _, _, cx| cx.emit(PasswordEvent::Cancelled)));
        let eye = div()
            .id("show-password")
            .size(px(52.))
            .flex_shrink_0()
            .flex()
            .items_center()
            .justify_center()
            .rounded_full()
            .cursor_pointer()
            .hover(|s| s.bg(rgb(c.well_hover)))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.shown = !shown;
                this.field.update(cx, |f, cx| f.set_masked(shown, cx));
                cx.notify();
            }))
            .child(icon(if shown { "eye-off" } else { "eye" }, 20., c.muted));
        let field = div()
            .flex()
            .items_center()
            .h(px(52.))
            .pl(px(18.))
            .rounded(px(16.))
            .bg(rgb(c.field))
            .text_size(px(16.))
            .child(self.field.clone())
            .child(eye);
        card()
            .id("password-dialog")
            .w(px(520.))
            .on_key_down(cx.listener(Self::on_key))
            .child(header(
                "lock",
                tonal_tint(),
                title,
                self.name.clone(),
                Some(close),
            ))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .child(legend(label))
                    .child(field),
            )
            .children(
                self.again
                    .clone()
                    .map(|again| note(Tone::Warn).child(again)),
            )
            .child(div().text_size(px(14.)).text_color(rgb(c.muted)).child(why))
            .child(
                footer()
                    .child(div().flex_1())
                    .child(
                        button("cancel", "Cancel", Fill::Text, Size::Medium)
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(PasswordEvent::Cancelled))),
                    )
                    .child(
                        button("open", "Open", Fill::Accent, Size::Medium)
                            .px(px(28.))
                            .on_click(cx.listener(|this, _, _, cx| this.open(cx))),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn locked(needs: &[&str]) -> Error {
        Error::Locked {
            needs: needs.iter().map(|n| n.to_string()).collect(),
        }
    }

    #[test]
    fn what_is_asked_and_when_again() {
        let none = Keys::default();
        let typed = Keys {
            password: Some(b"wrong".to_vec()),
            ..Keys::default()
        };
        // First asked: what the archive needs.
        assert_eq!(
            asked_for(&locked(&["scrypt"]), &none),
            Some((Asked::Passphrase, false))
        );
        assert_eq!(
            asked_for(&locked(&["password"]), &none),
            Some((Asked::Password, false))
        );
        // Asked again, the one typed being wrong.
        assert_eq!(
            asked_for(&Error::WrongPassphrase, &none),
            Some((Asked::Passphrase, true))
        );
        assert_eq!(
            asked_for(&Error::WrongPassword, &typed),
            Some((Asked::Password, true))
        );
        // Not asked: data that fails with no password typed, a key no one
        // can type, and what is not about keys at all.
        assert_eq!(asked_for(&Error::WrongPassword, &none), None);
        assert_eq!(asked_for(&locked(&["X25519"]), &none), None);
        assert_eq!(asked_for(&Error::NotAnArchive, &none), None);
        assert_eq!(asked_for(&Error::Failed("damaged".into()), &typed), None);
    }

    #[test]
    fn each_secret_has_its_words() {
        assert_eq!(words(Asked::Passphrase).1, "Passphrase");
        assert_eq!(words(Asked::Password).1, "Password");
        assert_ne!(placeholder(Asked::Passphrase), placeholder(Asked::Password));
    }
}
