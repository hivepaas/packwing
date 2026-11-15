//! The questions a connection over SSH asks (trust this server? its password?),
//! carried from the storage's worker thread to the window and back: the worker
//! waits while the window asks the user.

use std::sync::mpsc;

use futures_channel::mpsc::UnboundedSender;
use packwing_remote::{
    CodeQuestion, HostKeyQuestion, PassphraseQuestion, PasswordQuestion, Questions, Typed,
};

/// What is asked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Asked {
    HostKey(HostKeyQuestion),
    Password(PasswordQuestion),
    Passphrase(PassphraseQuestion),
    Code(CodeQuestion),
}

/// What the user answered: trusted or not, or what they typed; `None` for
/// cancelled.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answer {
    Trust(bool),
    Typed(Option<Typed>),
    Code(Option<String>),
}

impl Asked {
    /// The answer that says no: not trusted, or nothing given.
    pub fn no(&self) -> Answer {
        match self {
            Asked::HostKey(_) => Answer::Trust(false),
            Asked::Password(_) | Asked::Passphrase(_) => Answer::Typed(None),
            Asked::Code(_) => Answer::Code(None),
        }
    }
}

/// A question waiting for the user. Dropped unanswered, it is cancelled.
pub struct Question {
    pub asked: Asked,
    reply: mpsc::Sender<Answer>,
}

impl Question {
    pub fn answer(self, answer: Answer) {
        let _ = self.reply.send(answer);
    }
}

/// Where a connection's questions go: the window's queue of them.
pub struct Asker(pub UnboundedSender<Question>);

impl Asker {
    /// Waits for the answer; `None` if the window is gone.
    fn ask(&self, asked: Asked) -> Option<Answer> {
        let (reply, answer) = mpsc::channel();
        self.0.unbounded_send(Question { asked, reply }).ok()?;
        answer.recv().ok()
    }
}

impl Questions for Asker {
    fn host_key(&self, question: HostKeyQuestion) -> bool {
        matches!(
            self.ask(Asked::HostKey(question)),
            Some(Answer::Trust(true))
        )
    }

    fn passphrase(&self, question: PassphraseQuestion) -> Option<Typed> {
        match self.ask(Asked::Passphrase(question))? {
            Answer::Typed(typed) => typed,
            _ => None,
        }
    }

    fn password(&self, question: PasswordQuestion) -> Option<Typed> {
        match self.ask(Asked::Password(question))? {
            Answer::Typed(typed) => typed,
            _ => None,
        }
    }

    fn code(&self, question: CodeQuestion) -> Option<String> {
        match self.ask(Asked::Code(question))? {
            Answer::Code(code) => code,
            _ => None,
        }
    }
}
