//! What an organization has fixed for packwing, read from the system once,
//! when it starts: the registry on Windows, a profile's forced preferences on
//! macOS, a root's file elsewhere. docs/policies.md is the administrator's side.
//!
//! A value that cannot be understood never loosens anything. What it would have
//! guarded refuses, and says why, as a typing mistake must not quietly take a
//! lock off. [`Policy::rows`] tells it all, for Settings and `packwing --policies`.
//!
//! These bind packwing, not the person: an administrator of the computer can
//! change them, and anyone can build packwing without them, as its license allows.

use std::fmt;
use std::sync::OnceLock;

use packwing_i18n::tr;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(all(unix, not(target_os = "macos")))]
mod unix;
#[cfg(windows)]
mod windows;

pub const PROPAGATE_MOTW: &str = "PropagateMotW";
pub const REQUIRE_ENCRYPTION: &str = "RequireEncryption";
pub const ALLOWED_ENCRYPTION_METHODS: &str = "AllowedEncryptionMethods";
pub const MIN_PASSWORD_LENGTH: &str = "MinPasswordLength";
pub const ORGANIZATION_NAME: &str = "ManagedByOrganizationName";
pub const ORGANIZATION_URL: &str = "ManagedByURL";

/// Every policy there is, as the systems' readers look for them.
pub const KEYS: [&str; 6] = [
    PROPAGATE_MOTW,
    REQUIRE_ENCRYPTION,
    ALLOWED_ENCRYPTION_METHODS,
    MIN_PASSWORD_LENGTH,
    ORGANIZATION_NAME,
    ORGANIZATION_URL,
];

/// The most `MinPasswordLength` may ask for.
const MOST_LENGTH: i64 = 1024;

/// A value as the system holds it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Text(String),
    List(Vec<String>),
    /// A kind no policy takes: what it was, to say so.
    Other(String),
}

/// What a system's reader found.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Found {
    /// Where it looked.
    pub origin: String,
    pub values: Vec<(String, Value)>,
    /// Why what is there could not be read at all.
    pub failure: Option<String>,
}

/// A way of encrypting a new archive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    /// WinZip's AES-256, in a ZIP archive.
    ZipAes256,
    /// 7-Zip's AES-256, in a 7z archive.
    SevenZipAes256,
    /// age, to a passphrase.
    AgePassphrase,
    /// age, to keys, post-quantum or not.
    AgeKey,
}

impl Method {
    pub const ALL: [Method; 4] = [
        Method::ZipAes256,
        Method::SevenZipAes256,
        Method::AgePassphrase,
        Method::AgeKey,
    ];

    /// Its name in a policy.
    pub fn name(self) -> &'static str {
        match self {
            Method::ZipAes256 => "zip-aes256",
            Method::SevenZipAes256 => "7z-aes256",
            Method::AgePassphrase => "age-passphrase",
            Method::AgeKey => "age-key",
        }
    }
}

/// A policy: not set, set, or set to what cannot be understood.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Setting<T> {
    Unset,
    Set(T),
    Bad(String),
}

impl<T> Setting<T> {
    fn of(value: Option<&Value>, read: impl FnOnce(&Value) -> Result<T, String>) -> Self {
        match value.map(read) {
            None => Setting::Unset,
            Some(Ok(v)) => Setting::Set(v),
            Some(Err(why)) => Setting::Bad(why),
        }
    }
}

/// Why a policy stops something. It reads as a sentence to the user.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refused {
    /// The policies, or the one that decides this, could not be understood.
    Unreadable,
    NeedsEncryption,
    MethodNotAllowed(Method),
    PasswordTooShort(u32),
}

impl fmt::Display for Refused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&match self {
            Refused::Unreadable => tr!(
                "Your organization’s policy for this cannot be read, so packwing won’t do it. \
                 Ask your IT department."
            ),
            Refused::NeedsEncryption => {
                tr!("Your organization requires new archives to be encrypted.")
            }
            Refused::MethodNotAllowed(method) => tr!(
                "Your organization does not allow this kind of encryption: {method}.",
                method = method.name()
            ),
            Refused::PasswordTooShort(least) => tr!(
                "Your organization requires a password of at least {least} characters.",
                least = least
            ),
        })
    }
}

/// A policy's line, for Settings and `packwing --policies`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub key: String,
    /// What it is fixed at, as the administrator wrote it.
    pub value: String,
    /// Why it cannot be understood.
    pub problem: Option<String>,
}

/// What the organization fixed: nothing, where packwing is not managed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Policy {
    origin: String,
    failure: Option<String>,
    unknown: Vec<String>,
    pub propagate_mark: Setting<bool>,
    pub require_encryption: Setting<bool>,
    pub allowed_methods: Setting<Vec<Method>>,
    pub min_password_length: Setting<u32>,
    pub organization: Setting<String>,
    pub url: Setting<String>,
}

impl Policy {
    /// What `found` says.
    pub fn of(found: Found) -> Policy {
        let value = |key: &str| {
            found
                .values
                .iter()
                .rev()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v)
        };
        Policy {
            propagate_mark: Setting::of(value(PROPAGATE_MOTW), read_bool),
            require_encryption: Setting::of(value(REQUIRE_ENCRYPTION), read_bool),
            allowed_methods: Setting::of(value(ALLOWED_ENCRYPTION_METHODS), read_methods),
            min_password_length: Setting::of(value(MIN_PASSWORD_LENGTH), read_length),
            organization: Setting::of(value(ORGANIZATION_NAME), read_text),
            url: Setting::of(value(ORGANIZATION_URL), read_url),
            unknown: found
                .values
                .iter()
                .map(|(k, _)| k)
                .filter(|k| !KEYS.contains(&k.as_str()))
                .cloned()
                .collect(),
            origin: found.origin,
            failure: found.failure,
        }
    }

    /// Where it was looked for.
    pub fn origin(&self) -> &str {
        &self.origin
    }

    /// Why what is there could not be read at all.
    pub fn failure(&self) -> Option<&str> {
        self.failure.as_deref()
    }

    /// Whether an organization has set anything, or tried to.
    pub fn is_managed(&self) -> bool {
        self.failure.is_some() || !self.rows().is_empty()
    }

    /// Whether the mark of the Internet goes on what is extracted, given what
    /// the user chose. A policy that cannot be understood keeps it.
    pub fn keeps_mark(&self, chosen: bool) -> bool {
        if self.failure.is_some() {
            return true;
        }
        match self.propagate_mark {
            Setting::Unset => chosen,
            Setting::Set(on) => on,
            Setting::Bad(_) => true,
        }
    }

    /// Whether the user's own switch for the mark has no say.
    pub fn locks_mark(&self) -> bool {
        self.failure.is_some() || self.propagate_mark != Setting::Unset
    }

    /// Whether new archives have to be encrypted, as far as can be read.
    pub fn requires_encryption(&self) -> bool {
        self.failure.is_some()
            || !matches!(
                self.require_encryption,
                Setting::Unset | Setting::Set(false)
            )
    }

    /// The least length a password has to have, as far as can be read.
    pub fn least_password_length(&self) -> Option<u32> {
        match self.min_password_length {
            Setting::Set(n) => Some(n),
            _ => None,
        }
    }

    /// Whether a new archive may be made encrypted by `method`, or by nothing.
    pub fn check_method(&self, method: Option<Method>) -> Result<(), Refused> {
        if self.failure.is_some() || matches!(self.require_encryption, Setting::Bad(_)) {
            return Err(Refused::Unreadable);
        }
        let Some(method) = method else {
            return if self.requires_encryption() {
                Err(Refused::NeedsEncryption)
            } else {
                Ok(())
            };
        };
        match &self.allowed_methods {
            Setting::Unset => Ok(()),
            Setting::Set(allowed) if allowed.contains(&method) => Ok(()),
            Setting::Set(_) => Err(Refused::MethodNotAllowed(method)),
            Setting::Bad(_) => Err(Refused::Unreadable),
        }
    }

    /// Whether `password` is long enough, counted in characters, not bytes.
    pub fn check_password(&self, password: &[u8]) -> Result<(), Refused> {
        if self.failure.is_some() {
            return Err(Refused::Unreadable);
        }
        match self.min_password_length {
            Setting::Unset => Ok(()),
            Setting::Bad(_) => Err(Refused::Unreadable),
            Setting::Set(least) => {
                let length = String::from_utf8_lossy(password).chars().count();
                if length < least as usize {
                    Err(Refused::PasswordTooShort(least))
                } else {
                    Ok(())
                }
            }
        }
    }

    /// Whether files may be added to an archive, by whether what is there is
    /// encrypted. What is taken out or renamed brings nothing new in the clear.
    pub fn check_adding(&self, encrypted: bool) -> Result<(), Refused> {
        if self.failure.is_some() || matches!(self.require_encryption, Setting::Bad(_)) {
            return Err(Refused::Unreadable);
        }
        if self.requires_encryption() && !encrypted {
            return Err(Refused::NeedsEncryption);
        }
        Ok(())
    }

    /// Each policy set, and each name that is none.
    pub fn rows(&self) -> Vec<Row> {
        let mut rows = Vec::new();
        row(&mut rows, PROPAGATE_MOTW, &self.propagate_mark, on_off);
        row(
            &mut rows,
            REQUIRE_ENCRYPTION,
            &self.require_encryption,
            on_off,
        );
        row(
            &mut rows,
            ALLOWED_ENCRYPTION_METHODS,
            &self.allowed_methods,
            |m| m.iter().map(|m| m.name()).collect::<Vec<_>>().join(", "),
        );
        row(
            &mut rows,
            MIN_PASSWORD_LENGTH,
            &self.min_password_length,
            u32::to_string,
        );
        row(
            &mut rows,
            ORGANIZATION_NAME,
            &self.organization,
            String::clone,
        );
        row(&mut rows, ORGANIZATION_URL, &self.url, String::clone);
        rows.extend(self.unknown.iter().map(|key| Row {
            key: key.clone(),
            value: String::new(),
            problem: Some(tr!("packwing has no policy of this name.")),
        }));
        rows
    }

    /// All of it, for an administrator to read.
    pub fn report(&self) -> String {
        let mut out = format!("Policies from: {}\n", self.origin);
        if let Some(why) = &self.failure {
            out += &format!("Cannot be read: {why}\n");
            out += "Until this is fixed, what policies guard refuses.\n";
        }
        let rows = self.rows();
        if rows.is_empty() && self.failure.is_none() {
            out += "None is set.\n";
        }
        for r in rows {
            match r.problem {
                Some(why) => out += &format!("{}: IGNORED OR REFUSING: {why}\n", r.key),
                None => out += &format!("{}: {}\n", r.key, r.value),
            }
        }
        out
    }
}

fn on_off(on: &bool) -> String {
    if *on { "true" } else { "false" }.to_owned()
}

fn row<T>(rows: &mut Vec<Row>, key: &str, setting: &Setting<T>, shown: impl Fn(&T) -> String) {
    let (value, problem) = match setting {
        Setting::Unset => return,
        Setting::Set(v) => (shown(v), None),
        Setting::Bad(why) => (String::new(), Some(why.clone())),
    };
    rows.push(Row {
        key: key.to_owned(),
        value,
        problem,
    });
}

/// Yes or no; the registry has only numbers for it.
fn read_bool(value: &Value) -> Result<bool, String> {
    match value {
        Value::Bool(on) => Ok(*on),
        Value::Int(0) => Ok(false),
        Value::Int(1) => Ok(true),
        other => Err(tr!(
            "Expected true or false, not {other}.",
            other = said(other)
        )),
    }
}

fn read_length(value: &Value) -> Result<u32, String> {
    match value {
        Value::Int(n) if (1..=MOST_LENGTH).contains(n) => Ok(*n as u32),
        other => Err(tr!(
            "Expected a number from 1 to {most}, not {other}.",
            most = MOST_LENGTH,
            other = said(other)
        )),
    }
}

fn read_text(value: &Value) -> Result<String, String> {
    match value {
        Value::Text(text) if !text.trim().is_empty() => Ok(text.trim().to_owned()),
        other => Err(tr!("Expected some text, not {other}.", other = said(other))),
    }
}

/// An address for the user to open, so only the Web's.
fn read_url(value: &Value) -> Result<String, String> {
    let text = read_text(value)?;
    if text.starts_with("https://") || text.starts_with("http://") {
        Ok(text)
    } else {
        Err(tr!(
            "Expected an address starting with https://, not {other}.",
            other = text
        ))
    }
}

/// A list, or the names in one text with commas between, as not every tool
/// that writes policies can write a list.
fn read_methods(value: &Value) -> Result<Vec<Method>, String> {
    let names: Vec<&str> = match value {
        Value::List(names) => names.iter().map(String::as_str).collect(),
        Value::Text(text) => text.split(',').collect(),
        other => return Err(tr!("Expected a list, not {other}.", other = said(other))),
    };
    let mut methods = Vec::new();
    for name in names.iter().map(|n| n.trim()).filter(|n| !n.is_empty()) {
        let Some(method) = Method::ALL.into_iter().find(|m| m.name() == name) else {
            return Err(tr!("No kind of encryption is called {name}.", name = name));
        };
        methods.push(method);
    }
    if methods.is_empty() {
        return Err(tr!("The list names no kind of encryption."));
    }
    Ok(methods)
}

fn said(value: &Value) -> String {
    match value {
        Value::Bool(on) => on.to_string(),
        Value::Int(n) => n.to_string(),
        Value::Text(text) => format!("“{text}”"),
        Value::List(names) => names.join(", "),
        Value::Other(what) => what.clone(),
    }
}

/// What this system says.
fn read() -> Found {
    #[cfg(target_os = "macos")]
    return macos::read();
    #[cfg(windows)]
    return windows::read();
    #[cfg(all(unix, not(target_os = "macos")))]
    return unix::read();
}

static CURRENT: OnceLock<Policy> = OnceLock::new();

/// What is fixed on this computer, read the first time it is asked for. A change
/// takes effect when packwing next starts.
pub fn current() -> &'static Policy {
    CURRENT.get_or_init(|| Policy::of(read()))
}

/// Puts `policy` in place of what the system says, for a test of what obeys it:
/// a program's first ask decides, so a test runs in a process of its own.
pub fn install(policy: Policy) -> bool {
    CURRENT.set(policy).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy(values: &[(&str, Value)]) -> Policy {
        Policy::of(Found {
            origin: "test".into(),
            values: values
                .iter()
                .map(|(k, v)| ((*k).to_owned(), v.clone()))
                .collect(),
            failure: None,
        })
    }

    #[test]
    fn nothing_set_changes_nothing() {
        let p = policy(&[]);
        assert!(!p.is_managed());
        assert!(p.keeps_mark(true));
        assert!(!p.keeps_mark(false));
        assert!(!p.locks_mark());
        assert_eq!(p.check_method(None), Ok(()));
        assert_eq!(p.check_method(Some(Method::AgeKey)), Ok(()));
        assert_eq!(p.check_password(b"a"), Ok(()));
        assert_eq!(p.check_adding(false), Ok(()));
    }

    #[test]
    fn the_mark_follows_the_policy_not_the_user() {
        let on = policy(&[(PROPAGATE_MOTW, Value::Bool(true))]);
        assert!(on.keeps_mark(false));
        assert!(on.locks_mark());
        let off = policy(&[(PROPAGATE_MOTW, Value::Int(0))]);
        assert!(!off.keeps_mark(true));
    }

    #[test]
    fn a_mistake_keeps_the_mark_on() {
        let p = policy(&[(PROPAGATE_MOTW, Value::Text("yes".into()))]);
        assert!(p.keeps_mark(false));
        assert!(p.locks_mark());
        assert!(p.rows()[0].problem.is_some());
    }

    #[test]
    fn encryption_can_be_required() {
        let p = policy(&[(REQUIRE_ENCRYPTION, Value::Int(1))]);
        assert_eq!(p.check_method(None), Err(Refused::NeedsEncryption));
        assert_eq!(p.check_method(Some(Method::ZipAes256)), Ok(()));
        assert_eq!(p.check_adding(false), Err(Refused::NeedsEncryption));
        assert_eq!(p.check_adding(true), Ok(()));
        let off = policy(&[(REQUIRE_ENCRYPTION, Value::Bool(false))]);
        assert_eq!(off.check_method(None), Ok(()));
    }

    #[test]
    fn a_mistake_in_requiring_encryption_refuses_everything() {
        let p = policy(&[(REQUIRE_ENCRYPTION, Value::Text("on".into()))]);
        assert_eq!(p.check_method(None), Err(Refused::Unreadable));
        assert_eq!(
            p.check_method(Some(Method::ZipAes256)),
            Err(Refused::Unreadable)
        );
        assert_eq!(p.check_adding(true), Err(Refused::Unreadable));
    }

    #[test]
    fn methods_can_be_limited_by_a_list_or_commas() {
        for value in [
            Value::List(vec!["age-key".into(), "7z-aes256".into()]),
            Value::Text("age-key, 7z-aes256".into()),
        ] {
            let p = policy(&[(ALLOWED_ENCRYPTION_METHODS, value)]);
            assert_eq!(p.check_method(Some(Method::AgeKey)), Ok(()));
            assert_eq!(p.check_method(Some(Method::SevenZipAes256)), Ok(()));
            assert_eq!(
                p.check_method(Some(Method::ZipAes256)),
                Err(Refused::MethodNotAllowed(Method::ZipAes256))
            );
            // It limits how, not whether.
            assert_eq!(p.check_method(None), Ok(()));
        }
    }

    #[test]
    fn a_mistyped_or_empty_list_of_methods_refuses_encrypting() {
        for value in [
            Value::Text("aes".into()),
            Value::List(vec![]),
            Value::Int(1),
        ] {
            let p = policy(&[(ALLOWED_ENCRYPTION_METHODS, value)]);
            assert_eq!(
                p.check_method(Some(Method::AgeKey)),
                Err(Refused::Unreadable)
            );
        }
    }

    #[test]
    fn a_password_is_counted_in_characters() {
        let p = policy(&[(MIN_PASSWORD_LENGTH, Value::Int(4))]);
        assert_eq!(p.check_password("abcd".as_bytes()), Ok(()));
        assert_eq!(
            p.check_password("áàạ".as_bytes()),
            Err(Refused::PasswordTooShort(4))
        );
        assert_eq!(p.check_password("áàạã".as_bytes()), Ok(()));
    }

    #[test]
    fn a_length_out_of_range_refuses_passwords() {
        for value in [
            Value::Int(0),
            Value::Int(-3),
            Value::Int(1025),
            Value::Text("12".into()),
        ] {
            let p = policy(&[(MIN_PASSWORD_LENGTH, value)]);
            assert_eq!(
                p.check_password(b"long enough by far"),
                Err(Refused::Unreadable)
            );
        }
    }

    #[test]
    fn what_cannot_be_read_at_all_refuses() {
        let p = Policy::of(Found {
            origin: "test".into(),
            values: vec![],
            failure: Some("broken".into()),
        });
        assert!(p.is_managed());
        assert!(p.keeps_mark(false));
        assert!(p.requires_encryption());
        assert_eq!(p.check_method(None), Err(Refused::Unreadable));
        assert_eq!(
            p.check_method(Some(Method::AgeKey)),
            Err(Refused::Unreadable)
        );
        assert_eq!(p.check_password(b"anything"), Err(Refused::Unreadable));
        assert_eq!(p.check_adding(true), Err(Refused::Unreadable));
        assert!(p.report().contains("broken"));
    }

    #[test]
    fn a_name_that_is_no_policy_is_told() {
        let p = policy(&[("RequireEncrytion", Value::Bool(true))]);
        assert!(p.is_managed());
        let rows = p.rows();
        assert_eq!(rows[0].key, "RequireEncrytion");
        assert!(rows[0].problem.is_some());
    }

    #[test]
    fn only_web_addresses_are_kept() {
        let bad = policy(&[(ORGANIZATION_URL, Value::Text("file:///etc/passwd".into()))]);
        assert!(matches!(bad.url, Setting::Bad(_)));
        let good = policy(&[(
            ORGANIZATION_URL,
            Value::Text("https://it.example.com".into()),
        )]);
        assert_eq!(good.url, Setting::Set("https://it.example.com".into()));
    }

    #[test]
    fn the_report_says_each_policy() {
        let p = policy(&[
            (REQUIRE_ENCRYPTION, Value::Bool(true)),
            (MIN_PASSWORD_LENGTH, Value::Int(0)),
        ]);
        let report = p.report();
        assert!(report.contains("RequireEncryption: true"));
        assert!(report.contains("MinPasswordLength: IGNORED OR REFUSING"));
    }
}
