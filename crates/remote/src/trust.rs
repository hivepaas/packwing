//! Trusting a server over SSH, the same in every build: on a desktop, in the Mac
//! App Store's sandbox, and on a phone, where `~/.ssh` cannot be reached.
//!
//! - A host's key is checked against the user's known_hosts (`~/.ssh/known_hosts`,
//!   or the `UserKnownHostsFile` ssh_config names) when that knows the host, as ssh
//!   does. Otherwise it is checked against packwing's own, in the folder
//!   [`keep_in`] names, and a host found in neither is asked about
//!   ([`Questions::host_key`]). What the user trusts is written to the known_hosts
//!   the host was checked against. A key that changed is refused, with no way past it here
//!   ([`HostKeyChanged`]).
//! - Passwords, passphrases and one-time codes are asked for as they are needed.
//!   A password or passphrase the user asks to keep is kept in the keychain once
//!   it works.
//!
//! The app answers the questions, through [`ask_with`], from a worker thread: a
//! storage's calls block while it does.

use std::collections::HashSet;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use packwing_i18n::tr;
use shellwing_ssh::keys::ssh_key::PublicKey;
use shellwing_ssh::{Prompt, Prompter, Resolved, known_hosts};

use crate::dal::runtime;

/// A host whose key is in no known_hosts: trust it?
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostKeyQuestion {
    pub host: String,
    pub port: u16,
    /// `ED25519`, `ECDSA-SHA2-NISTP256`, `RSA`…
    pub algorithm: String,
    /// `SHA256:…`, as `ssh-keygen -l` writes it.
    pub fingerprint: String,
}

/// The passphrase of a key file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PassphraseQuestion {
    /// The key file's name.
    pub key: String,
    /// The one typed before did not open it.
    pub retry: bool,
}

/// The password of `user@host`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PasswordQuestion {
    pub target: String,
    /// The server refused the one before.
    pub retry: bool,
}

/// What a server asks for besides a password, such as a one-time code.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeQuestion {
    pub name: String,
    pub instructions: String,
    pub text: String,
    /// Whether what is typed may be shown.
    pub echo: bool,
}

/// A secret typed, and whether to keep it in the keychain once it works.
#[derive(Clone, PartialEq, Eq)]
pub struct Typed {
    pub secret: String,
    pub keep: bool,
}

impl fmt::Debug for Typed {
    /// Never prints the secret itself.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Typed {{ keep: {} }}", self.keep)
    }
}

/// Where the questions a connection has go: the app asks the user. Each method
/// blocks the worker thread it is called on until the user answers; `false` or
/// `None` is the user saying no, or closing the question.
pub trait Questions: Send + Sync {
    fn host_key(&self, question: HostKeyQuestion) -> bool;
    fn passphrase(&self, question: PassphraseQuestion) -> Option<Typed>;
    fn password(&self, question: PasswordQuestion) -> Option<Typed>;
    fn code(&self, question: CodeQuestion) -> Option<String>;
}

static QUESTIONS: Mutex<Option<Arc<dyn Questions>>> = Mutex::new(None);
static FOLDER: Mutex<Option<PathBuf>> = Mutex::new(None);

/// Where the questions of every storage opened go from now on; `None` for nowhere,
/// so that only what is known and kept is used.
pub fn ask_with(questions: Option<Arc<dyn Questions>>) {
    *QUESTIONS.lock().unwrap_or_else(PoisonError::into_inner) = questions;
}

pub(crate) fn questions() -> Option<Arc<dyn Questions>> {
    QUESTIONS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()
}

/// Keeps packwing's own known_hosts, and the list of the keys it keeps, in
/// `folder`, packwing's settings folder. Before, the user's known_hosts is the
/// only one, and no key can be kept.
pub fn keep_in(folder: PathBuf) {
    *FOLDER.lock().unwrap_or_else(PoisonError::into_inner) = Some(folder);
}

/// packwing's folder for SSH, if [`keep_in`] named one.
pub(crate) fn folder() -> Option<PathBuf> {
    FOLDER
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .as_ref()
        .map(|f| f.join("ssh"))
}

/// packwing's own known_hosts, if [`keep_in`] named a folder.
pub fn known_hosts_file() -> Option<PathBuf> {
    folder().map(|f| f.join("known_hosts"))
}

/// The user's known_hosts for `target`: its `UserKnownHostsFile`, else
/// `~/.ssh/known_hosts`. In a sandbox, or on a phone, it is not there.
fn users_known_hosts(target: &Resolved) -> Option<PathBuf> {
    target
        .known_hosts
        .clone()
        .or_else(|| std::env::home_dir().map(|h| h.join(".ssh").join("known_hosts")))
}

/// The keys `file` has for `host` on `port`, not its authorities or revoked ones.
fn keys_in(file: &Path, host: &str, port: u16) -> Vec<PublicKey> {
    known_hosts::entries(file)
        .unwrap_or_default()
        .into_iter()
        .filter(|e| e.marker.is_none() && e.is_for(host, port))
        .filter_map(|e| PublicKey::from_openssh(&e.key.public).ok())
        .collect()
}

/// Whether `file` says anything of `host` on `port`.
fn knows(file: &Path, host: &str, port: u16) -> bool {
    known_hosts::entries(file)
        .unwrap_or_default()
        .iter()
        .any(|e| e.is_for(host, port))
}

/// How an algorithm is named in a question about a host key.
fn algorithm_of(key: &PublicKey) -> String {
    key.algorithm()
        .as_str()
        .trim_start_matches("ssh-")
        .to_ascii_uppercase()
}

/// `host`, or `[host]:port`, as a host key's question names it.
fn host_and_port(label: &str) -> (String, u16) {
    if let Some(rest) = label.strip_prefix('[')
        && let Some((host, port)) = rest.split_once("]:")
        && let Ok(port) = port.parse()
    {
        return (host.to_owned(), port);
    }
    (label.to_owned(), 22)
}

/// A host whose key is not the one known: refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostKeyChanged {
    /// `host`, or `[host]:port`.
    pub host: String,
    /// The key the server showed.
    pub fingerprint: String,
    /// The keys known for it.
    pub known: Vec<String>,
    /// The known_hosts they are in.
    pub file: PathBuf,
}

impl fmt::Display for HostKeyChanged {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&tr!(
            "The key of {host} is not the one it had. Someone may be posing as it, or the server was set up again. Its key now is {fingerprint}; the one known is in {file}.",
            host = self.host,
            fingerprint = self.fingerprint,
            file = self.file.display().to_string(),
        ))
    }
}

impl std::error::Error for HostKeyChanged {}

/// Where a connection's questions go, and what came of them.
pub(crate) struct Asking {
    questions: Option<Arc<dyn Questions>>,
    /// The user's known_hosts, for jump hosts it knows while the target is
    /// checked against packwing's own.
    users: Option<PathBuf>,
    /// Key files whose passphrase, and whether the password, is to be kept.
    keep_passphrases: Mutex<HashSet<String>>,
    keep_password: Mutex<bool>,
    changed: Mutex<Option<HostKeyChanged>>,
}

impl Asking {
    fn answer(&self, prompt: Prompt) -> Option<String> {
        let questions = self.questions.as_deref();
        match prompt {
            // Every answer given is yes or no: there is no asking again.
            Prompt::HostKey { again: true, .. } => Some("no".to_owned()),
            Prompt::HostKey {
                host,
                algorithm,
                fingerprint,
                ..
            } => {
                let (name, port) = host_and_port(&host);
                if let Some(users) = &self.users {
                    let known = keys_in(users, &name, port);
                    let of = |k: &PublicKey| {
                        k.fingerprint(shellwing_ssh::keys::ssh_key::HashAlg::Sha256)
                            .to_string()
                    };
                    if known.iter().any(|k| of(k) == fingerprint) {
                        return Some("yes".to_owned());
                    }
                    if known.iter().any(|k| algorithm_of(k) == algorithm) {
                        *self.changed.lock().unwrap_or_else(PoisonError::into_inner) =
                            Some(HostKeyChanged {
                                host,
                                fingerprint,
                                known: known.iter().map(of).collect(),
                                file: users.clone(),
                            });
                        return Some("no".to_owned());
                    }
                }
                let trusted = questions.is_some_and(|q| {
                    q.host_key(HostKeyQuestion {
                        host: name,
                        port,
                        algorithm,
                        fingerprint,
                    })
                });
                Some(if trusted { "yes" } else { "no" }.to_owned())
            }
            Prompt::Password { target, retry } => {
                let typed = questions?.password(PasswordQuestion { target, retry })?;
                *self
                    .keep_password
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner) = typed.keep;
                Some(typed.secret)
            }
            Prompt::Passphrase { key, retry } => {
                let name = Path::new(&key)
                    .file_name()
                    .map_or_else(|| key.clone(), |n| n.to_string_lossy().into_owned());
                let typed = questions?.passphrase(PassphraseQuestion { key: name, retry })?;
                let mut keep = self
                    .keep_passphrases
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner);
                if typed.keep {
                    keep.insert(key);
                } else {
                    keep.remove(&key);
                }
                Some(typed.secret)
            }
            Prompt::Question {
                name,
                instructions,
                text,
                echo,
            } => questions?.code(CodeQuestion {
                name,
                instructions,
                text,
                echo,
            }),
        }
    }

    /// Whether the password typed is to be kept, now that it worked.
    pub(crate) fn keeps_password(&self) -> bool {
        *self
            .keep_password
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    /// Whether the passphrase typed for the key file at `path` is to be kept.
    pub(crate) fn keeps_passphrase(&self, path: &str) -> bool {
        self.keep_passphrases
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .contains(path)
    }

    /// What became of a connection that failed: the questions' answers say
    /// more than the SSH error does.
    pub(crate) fn failed(&self, e: shellwing_ssh::Error) -> std::io::Error {
        use shellwing_ssh::Error as E;
        if let Some(changed) = self
            .changed
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take()
        {
            return std::io::Error::other(changed);
        }
        let e = match e {
            E::Jump { source, .. } => *source,
            e => e,
        };
        match e {
            E::HostKeyChanged(c) => std::io::Error::other(HostKeyChanged {
                host: c.label,
                fingerprint: c.fingerprint,
                known: c.old,
                file: c.file,
            }),
            E::HostKeyRejected { host } => std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                if self.questions.is_some() {
                    tr!("{host} was not trusted.", host = host)
                } else {
                    tr!(
                        "{host} is not known yet, and there is no one to ask whether to trust it.",
                        host = host
                    )
                },
            ),
            E::Cancelled if self.questions.is_none() => std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                E::AuthFailed.to_string(),
            ),
            E::Cancelled => std::io::Error::new(std::io::ErrorKind::Interrupted, tr!("Cancelled.")),
            e => std::io::Error::other(e.to_string()),
        }
    }
}

/// How `target` is checked: the known_hosts to check it against (`None`: the
/// user's, as ssh does), and where its questions go. `only` is the one
/// known_hosts there is, in place of the user's and packwing's (tests).
pub(crate) fn asking(
    target: &Resolved,
    only: Option<PathBuf>,
    questions: Option<Arc<dyn Questions>>,
) -> (Option<PathBuf>, Prompter, Arc<Asking>) {
    let (known_hosts, users) = match only {
        Some(only) => (Some(only), None),
        None => {
            let users = users_known_hosts(target).filter(|f| f.exists());
            let known = users
                .as_ref()
                .is_some_and(|u| knows(u, &target.target.host, target.target.port));
            (if known { None } else { known_hosts_file() }, users)
        }
    };
    let asking = Arc::new(Asking {
        questions,
        users,
        keep_passphrases: Mutex::new(HashSet::new()),
        keep_password: Mutex::new(false),
        changed: Mutex::new(None),
    });
    let (prompter, mut asked) = Prompter::new();
    let answering = asking.clone();
    runtime().spawn(async move {
        while let Some(request) = asked.recv().await {
            let answering = answering.clone();
            let prompt = request.prompt;
            let answer = tokio::task::spawn_blocking(move || answering.answer(prompt))
                .await
                .ok()
                .flatten();
            let _ = request.reply.send(answer);
        }
    });
    (known_hosts, prompter, asking)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_host_key_question_names_the_port() {
        assert_eq!(host_and_port("example.com"), ("example.com".into(), 22));
        assert_eq!(
            host_and_port("[127.0.0.1]:2222"),
            ("127.0.0.1".into(), 2222)
        );
    }
}
