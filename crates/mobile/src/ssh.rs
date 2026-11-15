//! SFTP's questions and keys, for a phone: what packwing-remote asks as it
//! connects (a server not known yet, a password, a key's passphrase, a code),
//! answered by the app, and the private keys it keeps in the keychain.
//!
//! The questions come on the worker thread a storage's call runs on, which
//! waits for the answer: the app shows each, and answers from there. A phone
//! has no `~/.ssh` of the user's, so the servers trusted go in packwing's own
//! known_hosts, in the folder `start` was given.

use std::io;
use std::sync::Arc;

use packwing_remote::{
    CodeQuestion, HostKeyQuestion, KeptKey, KeyRefused, PassphraseQuestion, PasswordQuestion,
    Questions, Typed,
};

use crate::MobileError;

/// A secret typed, and whether to keep it in the keychain once it works.
#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct TypedSecret {
    pub secret: String,
    pub keep: bool,
}

/// What the app answers SFTP's questions with. Each is called on a worker
/// thread, never the main one, and waits for the user: `false` or nothing is
/// the user saying no, or closing the question.
#[uniffi::export(with_foreign)]
pub trait SshQuestions: Send + Sync {
    /// A server whose key packwing does not know yet: trust it?
    fn host_key(&self, host: String, port: u16, algorithm: String, fingerprint: String) -> bool;
    /// The password of `target` (`user@host`); `retry` when the last was refused.
    fn password(&self, target: String, retry: bool) -> Option<TypedSecret>;
    /// The passphrase of the key `key`; `retry` when the last did not open it.
    fn passphrase(&self, key: String, retry: bool) -> Option<TypedSecret>;
    /// What the server asks besides, such as a one-time code.
    fn code(&self, name: String, instructions: String, text: String, echo: bool) -> Option<String>;
}

struct Asking(Arc<dyn SshQuestions>);

fn typed(answer: Option<TypedSecret>) -> Option<Typed> {
    answer.map(|t| Typed {
        secret: t.secret,
        keep: t.keep,
    })
}

impl Questions for Asking {
    fn host_key(&self, q: HostKeyQuestion) -> bool {
        self.0.host_key(q.host, q.port, q.algorithm, q.fingerprint)
    }

    fn passphrase(&self, q: PassphraseQuestion) -> Option<Typed> {
        typed(self.0.passphrase(q.key, q.retry))
    }

    fn password(&self, q: PasswordQuestion) -> Option<Typed> {
        typed(self.0.password(q.target, q.retry))
    }

    fn code(&self, q: CodeQuestion) -> Option<String> {
        self.0.code(q.name, q.instructions, q.text, q.echo)
    }
}

/// Where SFTP's questions go from now on; nothing for nowhere, so that only
/// what is known and kept is used.
#[uniffi::export]
pub fn ask_ssh_with(questions: Option<Arc<dyn SshQuestions>>) {
    packwing_remote::ask_with(questions.map(|q| Arc::new(Asking(q)) as Arc<dyn Questions>));
}

/// A private key packwing keeps, to sign in with.
#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct SshKey {
    /// What a storage names it by: its fingerprint.
    pub id: String,
    pub name: String,
    /// `ED25519`, `RSA 3072`…
    pub kind: String,
    pub fingerprint: String,
    /// Its public half, as a server's `authorized_keys` takes it.
    pub public: String,
}

impl From<KeptKey> for SshKey {
    fn from(k: KeptKey) -> SshKey {
        SshKey {
            id: k.id,
            name: k.name,
            kind: k.kind,
            fingerprint: k.fingerprint,
            public: k.public,
        }
    }
}

fn failed(e: io::Error) -> MobileError {
    MobileError::Failed {
        reason: e.to_string(),
    }
}

/// The keys kept.
#[uniffi::export]
pub fn ssh_keys() -> Result<Vec<SshKey>, MobileError> {
    Ok(packwing_remote::kept_keys()
        .map_err(failed)?
        .into_iter()
        .map(SshKey::from)
        .collect())
}

/// Keeps the private key `text` (as its file holds it) as `name`, opened with
/// `passphrase`. One with a passphrase, and none or not its own given, fails
/// with [`MobileError::Locked`], to ask for it.
#[uniffi::export]
pub fn import_ssh_key(
    name: String,
    text: String,
    passphrase: Option<String>,
) -> Result<SshKey, MobileError> {
    packwing_remote::import_key(&name, &text, passphrase.as_deref())
        .map(SshKey::from)
        .map_err(|e| {
            let reason = e.to_string();
            match e.get_ref().and_then(|e| e.downcast_ref::<KeyRefused>()) {
                Some(KeyRefused::Passphrase) => MobileError::Locked {
                    reason,
                    passphrase: true,
                },
                _ => MobileError::Failed { reason },
            }
        })
}

/// Forgets the key `id`.
#[uniffi::export]
pub fn remove_ssh_key(id: String) -> Result<(), MobileError> {
    packwing_remote::remove_key(&id).map_err(failed)
}
