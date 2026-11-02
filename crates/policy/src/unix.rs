//! Linux and the BSDs: a file only root can change.

use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use packwing_i18n::tr;
use serde_json::Value as Json;

use crate::{Found, Value};

const FILE: &str = "/etc/packwing/policy.json";

/// More than any list of policies comes to.
const MOST_BYTES: u64 = 1 << 20;

pub fn read() -> Found {
    read_file(Path::new(FILE), 0)
}

/// The file at `path`, trusted only if `owner` alone can change it. Nothing is
/// found if there is none.
fn read_file(path: &Path, owner: u32) -> Found {
    let found = |failure, values| Found {
        origin: path.display().to_string(),
        values,
        failure,
    };
    let meta = match fs::metadata(path) {
        Ok(meta) => meta,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return found(None, vec![]),
        Err(e) => return found(Some(e.to_string()), vec![]),
    };
    let held = path.parent().map_or(Ok(()), |folder| safe(folder, owner));
    if let Err(why) = safe(path, owner).and(held) {
        return found(Some(why), vec![]);
    }
    if meta.len() > MOST_BYTES {
        return found(
            Some(tr!("The file is too big to be a list of policies.")),
            vec![],
        );
    }
    let text = match fs::read(path) {
        Ok(text) => text,
        Err(e) => return found(Some(e.to_string()), vec![]),
    };
    match serde_json::from_slice::<Json>(&text) {
        Ok(Json::Object(map)) => found(
            None,
            map.into_iter().map(|(k, v)| (k, value_of(v))).collect(),
        ),
        Ok(_) => found(Some(tr!("The file has to hold one JSON object.")), vec![]),
        Err(e) => found(Some(e.to_string()), vec![]),
    }
}

/// Whether only `owner` can change `path`.
fn safe(path: &Path, owner: u32) -> Result<(), String> {
    let meta = fs::metadata(path).map_err(|e| e.to_string())?;
    if meta.uid() != owner {
        return Err(tr!("{path} has to belong to root.", path = path.display()));
    }
    if meta.mode() & 0o022 != 0 {
        return Err(tr!(
            "{path} can be changed by others than root.",
            path = path.display()
        ));
    }
    Ok(())
}

fn value_of(json: Json) -> Value {
    match json {
        Json::Bool(on) => Value::Bool(on),
        Json::Number(n) => n.as_i64().map_or(Value::Other(n.to_string()), Value::Int),
        Json::String(text) => Value::Text(text),
        Json::Array(items) => {
            let shown = items
                .iter()
                .map(Json::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            let names: Option<Vec<String>> = items
                .into_iter()
                .map(|item| match item {
                    Json::String(name) => Some(name),
                    _ => None,
                })
                .collect();
            names.map_or(Value::Other(shown), Value::List)
        }
        other => Value::Other(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;

    use super::*;

    /// A folder of its own for the file, as this user would own it.
    fn folder(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("packwing-policy-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir(&dir).expect("a temporary folder");
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).expect("its mode");
        dir
    }

    fn written(dir: &Path, text: &str) -> (PathBuf, u32) {
        let file = dir.join("policy.json");
        fs::write(&file, text).expect("the file");
        fs::set_permissions(&file, fs::Permissions::from_mode(0o644)).expect("its mode");
        let owner = fs::metadata(&file).expect("the file").uid();
        (file, owner)
    }

    #[test]
    fn no_file_is_no_policy() {
        let dir = folder("none");
        let found = read_file(&dir.join("policy.json"), 0);
        assert_eq!(found.failure, None);
        assert!(found.values.is_empty());
    }

    #[test]
    fn each_kind_of_value_is_read() {
        let dir = folder("kinds");
        let (file, owner) = written(
            &dir,
            r#"{"A": true, "B": 15, "C": "text", "D": ["x", "y"], "E": 1.5, "F": [1], "G": {}}"#,
        );
        let found = read_file(&file, owner);
        assert_eq!(found.failure, None);
        let of = |key: &str| {
            found
                .values
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.clone())
                .expect("the key")
        };
        assert_eq!(of("A"), Value::Bool(true));
        assert_eq!(of("B"), Value::Int(15));
        assert_eq!(of("C"), Value::Text("text".into()));
        assert_eq!(of("D"), Value::List(vec!["x".into(), "y".into()]));
        assert!(matches!(of("E"), Value::Other(_)));
        assert!(matches!(of("F"), Value::Other(_)));
        assert!(matches!(of("G"), Value::Other(_)));
    }

    #[test]
    fn a_broken_file_is_a_failure_not_nothing() {
        let dir = folder("broken");
        for text in ["{ not json", "[1, 2]"] {
            let (file, owner) = written(&dir, text);
            assert!(read_file(&file, owner).failure.is_some());
        }
    }

    #[test]
    fn a_file_of_someone_else_is_not_trusted() {
        let dir = folder("owner");
        let (file, owner) = written(&dir, r#"{"RequireEncryption": false}"#);
        let found = read_file(&file, owner + 1);
        assert!(found.failure.is_some());
        assert!(found.values.is_empty());
    }

    #[test]
    fn a_file_or_folder_others_can_change_is_not_trusted() {
        let dir = folder("writable");
        let (file, owner) = written(&dir, r#"{"RequireEncryption": false}"#);
        fs::set_permissions(&file, fs::Permissions::from_mode(0o664)).expect("its mode");
        assert!(read_file(&file, owner).failure.is_some());
        fs::set_permissions(&file, fs::Permissions::from_mode(0o644)).expect("its mode");
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o777)).expect("its mode");
        assert!(read_file(&file, owner).failure.is_some());
    }

    #[test]
    fn a_file_too_big_is_refused() {
        let dir = folder("big");
        let (file, owner) = written(&dir, &" ".repeat(MOST_BYTES as usize + 1));
        assert!(read_file(&file, owner).failure.is_some());
    }
}
