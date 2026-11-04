//! Signing files and checking their signatures, as the app does: a key of
//! Ed25519 and ML-DSA-65 together, or an SSH Ed25519 key; the signature beside
//! the file, as `NAME.sig`; and the keys trusted, the app's (Settings ›
//! Security) and those given.

use std::path::{Path, PathBuf};

use packwing_core::{
    KeyRefused, LocalFolder, PublicKey, Signed, SigningKey, check_signature, make_signing_key,
    read_public_key, read_signing_key, sign_file, signature_name,
};
use serde_json::{Value, json};

use crate::commands::ask_new_secret;
use crate::{Context, Exit};

/// The storage a local file is in, and its path there.
fn local(file: &Path) -> (LocalFolder, Vec<String>) {
    let folder = file
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let name = file
        .file_name()
        .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
    (LocalFolder::new(folder), vec![name])
}

pub fn keygen_cmd(folder: &Path, cx: &mut Context) -> Result<(), Exit> {
    let passphrase = match cx.secrets.password() {
        Some(p) => p,
        None if cx.secrets.can_ask() => ask_new_secret("Passphrase for the new key")?,
        None => {
            return Err(Exit::Usage(
                "the key needs a passphrase: from a terminal, --password-file, --password-stdin or PACKWING_PASSWORD".into(),
            ));
        }
    };
    let (key, file) = make_signing_key(folder, &passphrase)?;
    let mut public = file.clone().into_os_string();
    public.push(".pub");
    let public = PathBuf::from(public);
    let fingerprint = key.public().fingerprint();
    if cx.json {
        println!(
            "{:#}",
            json!({
                "key": file.display().to_string(),
                "public": public.display().to_string(),
                "fingerprint": fingerprint,
            })
        );
    } else if !cx.quiet {
        println!("{}", file.display());
        println!("{}", public.display());
        println!("{fingerprint}");
    }
    Ok(())
}

/// The key in `file`, its passphrase given or asked.
fn signing_key(file: &Path, cx: &Context) -> Result<SigningKey, Exit> {
    let text = std::fs::read(file)
        .map_err(|e| Exit::Failed(format!("cannot read {}: {e}", file.display())))?;
    let mut passphrase = cx.secrets.key_passphrase();
    let mut tries = 0;
    loop {
        match read_signing_key(&text, passphrase.as_deref()) {
            Ok(key) => return Ok(key),
            Err(KeyRefused::Not(why)) => return Err(Exit::Failed(why)),
            Err(refused) => {
                let said = match refused {
                    KeyRefused::NeedsPassphrase => "the key needs its passphrase",
                    _ => "the key's passphrase is wrong",
                };
                if !cx.secrets.can_ask() || tries == 3 {
                    return Err(Exit::Locked(said.into()));
                }
                if tries > 0 {
                    eprintln!("pkw: {said}");
                }
                tries += 1;
                let typed =
                    rpassword::prompt_password(format!("Passphrase of {}: ", file.display()))
                        .map_err(|e| Exit::Failed(format!("cannot ask: {e}")))?;
                passphrase = Some(typed.into_bytes());
            }
        }
    }
}

pub fn sign_cmd(files: &[PathBuf], by: &Path, cx: &mut Context) -> Result<(), Exit> {
    let key = signing_key(by, cx)?;
    let mut made = Vec::new();
    for file in files {
        let (storage, path) = local(file);
        let (control, bar) =
            crate::progress::control(&format!("Signing {}", path.join("/")), cx.progress);
        sign_file(&storage, &path, &key, &control)?;
        drop(bar);
        let name = signature_name(&path.join("/"));
        made.push(file.with_file_name(name));
    }
    if cx.json {
        let rows: Vec<Value> = files
            .iter()
            .zip(&made)
            .map(|(f, s)| json!({ "file": f.display().to_string(), "signature": s.display().to_string() }))
            .collect();
        println!("{:#}", json!(rows));
    } else if !cx.quiet {
        for signature in &made {
            println!("{}", signature.display());
        }
    }
    Ok(())
}

/// The app's settings file, where it keeps the keys it trusts, as its
/// `prefs.rs` finds it.
fn app_settings() -> Option<PathBuf> {
    let env = |name: &str| {
        std::env::var_os(name)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    };
    let folder = if cfg!(windows) {
        env("APPDATA")?
    } else if cfg!(target_os = "macos") {
        env("HOME")?.join("Library/Application Support")
    } else {
        env("XDG_CONFIG_HOME").or_else(|| Some(env("HOME")?.join(".config")))?
    };
    Some(folder.join("packwing").join("settings"))
}

/// The keys the app trusts, by the names it gives them, in its settings'
/// `trusted_key=NAME<TAB>LINE` lines.
fn app_trusted(settings: &str) -> Vec<(String, PublicKey)> {
    settings
        .lines()
        .filter_map(|l| l.strip_prefix("trusted_key="))
        .filter_map(|v| v.split_once('\t'))
        .filter_map(|(name, line)| {
            let name = name
                .replace("%0A", "\n")
                .replace("%09", "\t")
                .replace("%25", "%");
            Some((name, read_public_key(line).ok()?))
        })
        .collect()
}

/// What a signature is found to be, as a word, and whether it proves the file.
fn verdict(signed: &Signed) -> (&'static str, bool) {
    match signed {
        Signed::Good { .. } => ("good", true),
        Signed::Mismatch { .. } => ("changed since it was signed", false),
        Signed::Stranger { .. } => ("signed by a key not trusted", false),
        Signed::NotChecked { .. } => ("not checked", false),
        Signed::No => ("no signature", false),
        Signed::Unreadable(_) => ("unreadable signature", false),
        Signed::Failed(_) => ("could not be read", false),
    }
}

pub fn verify_cmd(
    files: &[PathBuf],
    trust: &[PathBuf],
    only: bool,
    cx: &mut Context,
) -> Result<(), Exit> {
    let mut trusted: Vec<(String, PublicKey)> = Vec::new();
    if !only && let Some(text) = app_settings().and_then(|f| std::fs::read_to_string(f).ok()) {
        trusted.extend(app_trusted(&text));
    }
    for file in trust {
        let text = std::fs::read_to_string(file)
            .map_err(|e| Exit::Failed(format!("cannot read {}: {e}", file.display())))?;
        let key = read_public_key(&text)
            .map_err(|why| Exit::Failed(format!("{}: {why}", file.display())))?;
        let name = file
            .file_name()
            .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
        trusted.push((name, key));
    }
    let keys: Vec<PublicKey> = trusted.iter().map(|(_, k)| k.clone()).collect();
    let name_of = |fingerprint: &str| {
        trusted
            .iter()
            .find(|(_, k)| k.fingerprint() == fingerprint)
            .map(|(n, _)| n.clone())
    };
    let mut rows = Vec::new();
    for file in files {
        let (storage, path) = local(file);
        let (control, bar) =
            crate::progress::control(&format!("Checking {}", path.join("/")), cx.progress);
        let signed = check_signature(&storage, &path, &keys, &control)
            .unwrap_or_else(|e| Signed::Failed(e.to_string()));
        drop(bar);
        let fingerprint = match &signed {
            Signed::Good { fingerprint }
            | Signed::Mismatch { fingerprint }
            | Signed::Stranger { fingerprint }
            | Signed::NotChecked { fingerprint } => Some(fingerprint.clone()),
            _ => None,
        };
        let why = match &signed {
            Signed::Unreadable(why) | Signed::Failed(why) => Some(why.clone()),
            _ => None,
        };
        rows.push((file.display().to_string(), signed, fingerprint, why));
    }
    let bad = rows.iter().filter(|(_, s, _, _)| !verdict(s).1).count();
    if cx.json {
        let out: Vec<Value> = rows
            .iter()
            .map(|(file, signed, fingerprint, why)| {
                json!({
                    "file": file,
                    "good": verdict(signed).1,
                    "result": verdict(signed).0,
                    "fingerprint": fingerprint,
                    "by": fingerprint.as_deref().and_then(name_of),
                    "why": why,
                })
            })
            .collect();
        println!("{:#}", json!(out));
    } else {
        for (file, signed, fingerprint, why) in &rows {
            let (said, good) = verdict(signed);
            if cx.quiet && good {
                continue;
            }
            let by = match fingerprint {
                Some(f) => match name_of(f) {
                    Some(name) => format!(", key {name} ({f})"),
                    None => format!(", key {f}"),
                },
                None => String::new(),
            };
            let why = why.as_deref().map(|w| format!(": {w}")).unwrap_or_default();
            println!("{file}: {said}{by}{why}");
        }
    }
    if bad > 0 {
        Err(Exit::Failed(match (bad, rows.len()) {
            (1, 1) => "it is not signed by a key trusted".into(),
            (bad, all) => format!("{bad} of {all} are not signed by a key trusted"),
        }))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_app_s_trusted_keys_are_read_from_its_settings() {
        let dir = std::env::temp_dir().join(format!("pkw-trusted-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a folder");
        let pass = "khoá".as_bytes();
        let (key, _) = make_signing_key(&dir, pass).expect("a key");
        let line = key.public().to_string();
        let settings = format!(
            "language=vi\ntrusted_key=Máy%09ở nhà\t{line}\ntrusted_key=broken\tnot a key\n"
        );
        let found = app_trusted(&settings);
        assert_eq!(found.len(), 1, "the broken one is left out");
        assert_eq!(found[0].0, "Máy\tở nhà");
        assert_eq!(found[0].1.fingerprint(), key.public().fingerprint());
        let _ = std::fs::remove_dir_all(dir);
    }
}
