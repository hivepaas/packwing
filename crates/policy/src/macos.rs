//! macOS: what a configuration profile forces on packwing's preferences. A value
//! the user wrote with `defaults` is not forced, and so is not a policy.
#![allow(unsafe_code)]

use core_foundation::array::CFArray;
use core_foundation::base::{Boolean, CFType, CFTypeRef, TCFType};
use core_foundation::boolean::CFBoolean;
use core_foundation::number::CFNumber;
use core_foundation::string::{CFString, CFStringRef};

use crate::{Found, KEYS, Value};

const DOMAIN: &str = "com.hivepaas.packwing";

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFPreferencesAppValueIsForced(key: CFStringRef, app: CFStringRef) -> Boolean;
    fn CFPreferencesCopyAppValue(key: CFStringRef, app: CFStringRef) -> CFTypeRef;
}

pub fn read() -> Found {
    let app = CFString::new(DOMAIN);
    let mut found = Found {
        origin: format!("the forced preferences of {DOMAIN}"),
        ..Found::default()
    };
    for key in KEYS {
        if let Some(value) = forced(&CFString::new(key), &app) {
            found.values.push((key.to_owned(), value));
        }
    }
    found
}

fn forced(key: &CFString, app: &CFString) -> Option<Value> {
    let (key, app) = (key.as_concrete_TypeRef(), app.as_concrete_TypeRef());
    // SAFETY: both are live CFStrings, for the call only, and neither call keeps them.
    if unsafe { CFPreferencesAppValueIsForced(key, app) } == 0 {
        return None;
    }
    // SAFETY: as above. What it gives is ours to release, or null.
    let value = unsafe { CFPreferencesCopyAppValue(key, app) };
    if value.is_null() {
        return None;
    }
    // SAFETY: it is a live CF object, and "Copy" in the name makes it ours.
    let value = unsafe { CFType::wrap_under_create_rule(value) };
    Some(value_of(&value))
}

fn value_of(value: &CFType) -> Value {
    if let Some(on) = value.downcast::<CFBoolean>() {
        return Value::Bool(on.into());
    }
    if let Some(n) = value.downcast::<CFNumber>() {
        return n
            .to_i64()
            .map_or(Value::Other("a number".into()), Value::Int);
    }
    if let Some(text) = value.downcast::<CFString>() {
        return Value::Text(text.to_string());
    }
    if let Some(items) = value.downcast::<CFArray>() {
        let names: Option<Vec<String>> = items
            .get_all_values()
            .into_iter()
            .map(|item| {
                // SAFETY: an item of a live array, which this retains for its own use.
                let item = unsafe { CFType::wrap_under_get_rule(item as CFTypeRef) };
                item.downcast::<CFString>().map(|t| t.to_string())
            })
            .collect();
        return names.map_or(Value::Other("a list of other things".into()), Value::List);
    }
    Value::Other("a kind no policy takes".into())
}
