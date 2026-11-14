//! The app's words in the user's language, from the desktop's catalogs
//! (`crates/i18n`): the Swift side says its English through [`tr`] and
//! [`tr_n`], as the desktop's `tr!` and `tr_n!` do, so both apps share one
//! translation of each message.

use std::collections::HashMap;

use packwing_i18n::Arg;

/// A language the app speaks, named in itself.
#[derive(Clone, Debug, PartialEq, uniffi::Record)]
pub struct LanguageRow {
    pub tag: String,
    pub name: String,
}

fn args_of(args: &HashMap<String, String>) -> Vec<(&str, Arg)> {
    args.iter()
        .map(|(k, v)| (k.as_str(), packwing_i18n::arg(v)))
        .collect()
}

/// `english` in the language in use, its `{name}`s filled from `args`.
#[uniffi::export]
pub fn tr(english: String, args: HashMap<String, String>) -> String {
    packwing_i18n::tr(&english, &args_of(&args))
}

/// A message that says a count `n` (as `{n}`): English's two forms.
#[uniffi::export]
pub fn tr_n(n: i64, one: String, other: String, args: HashMap<String, String>) -> String {
    packwing_i18n::tr_n(n, &one, &other, &args_of(&args))
}

/// The languages there are.
#[uniffi::export]
pub fn languages() -> Vec<LanguageRow> {
    packwing_i18n::LANGUAGES
        .iter()
        .map(|(tag, name)| LanguageRow {
            tag: tag.to_string(),
            name: name.to_string(),
        })
        .collect()
}

/// The language in use, by tag.
#[uniffi::export]
pub fn language() -> String {
    packwing_i18n::language().to_string()
}

/// Speaks `choice` (a tag), or the system's language for none.
#[uniffi::export]
pub fn set_language(choice: Option<String>) {
    packwing_i18n::set_language(choice.as_deref());
}
