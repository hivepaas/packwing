//! Windows: the registry's Policies keys, where Group Policy and Intune write and
//! only an administrator can. A value for the computer wins over one for the user.

use winreg::RegKey;
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, RegType};

use crate::{Found, Value};

const KEY: &str = r"SOFTWARE\Policies\HivePaaS\Packwing";

pub fn read() -> Found {
    let mut found = Found {
        origin: format!(r"HKLM\{KEY}, HKCU\{KEY}"),
        ..Found::default()
    };
    // Later ones win, so the computer's come last.
    for root in [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE] {
        match RegKey::predef(root).open_subkey(KEY) {
            Ok(key) => {
                for value in key.enum_values() {
                    match value {
                        Ok((name, value)) => found
                            .values
                            .push((name, value_of(&value.vtype, &value.bytes))),
                        Err(e) => found.failure = Some(e.to_string()),
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => found.failure = Some(e.to_string()),
        }
    }
    found
}

fn value_of(kind: &RegType, bytes: &[u8]) -> Value {
    match kind {
        RegType::REG_DWORD if bytes.len() == 4 => Value::Int(i64::from(u32::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
        ]))),
        RegType::REG_SZ | RegType::REG_EXPAND_SZ => Value::Text(texts(bytes).join("")),
        RegType::REG_MULTI_SZ => Value::List(texts(bytes)),
        other => Value::Other(format!("{other:?}")),
    }
}

/// The texts in a value, each ended by a zero.
fn texts(bytes: &[u8]) -> Vec<String> {
    let units: Vec<u16> = bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|b| u16::from_le_bytes(*b))
        .collect();
    units
        .split(|u| *u == 0)
        .filter(|text| !text.is_empty())
        .map(String::from_utf16_lossy)
        .collect()
}
