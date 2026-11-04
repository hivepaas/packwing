//! Signing files and checking their signatures, as the app does: a key of
//! Ed25519 and ML-DSA-65 together, or an SSH Ed25519 key; the signature beside
//! the file, as `NAME.sig`; and the keys trusted, the app's (Settings ›
//! Security) and those given.

use std::path::{Path, PathBuf};

use packwing_core::{
    KeyRefused, PublicKey, Signed, SigningKey, check_signature, make_signing_key, read_public_key,
    read_signing_key, sign_file, signature_name,
};
use serde_json::{Value, json};

use crate::commands::ask_new_secret;
use crate::places::Place;
use crate::{Context, Exit};

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
    std::fs::create_dir_all(folder)
        .map_err(|e| Exit::Failed(format!("cannot make {}: {e}", folder.display())))?;
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

pub fn sign_cmd(files: &[String], by: &Path, cx: &mut Context) -> Result<(), Exit> {
    let key = signing_key(by, cx)?;
    let mut made = Vec::new();
    for file in files {
        let place = Place::of(file)?;
        let (control, bar) =
            crate::progress::control(&format!("Signing {}", place.name()), cx.progress);
        sign_file(&*place.storage, &place.path, &key, &control)?;
        drop(bar);
        // Beside it, said as it was typed.
        let name = place.name();
        let signature = signature_name(&name);
        let shown = match file.strip_suffix(name.as_str()) {
            Some(before) => format!("{before}{signature}"),
            None => signature,
        };
        made.push((file.clone(), shown));
    }
    if cx.json {
        let rows: Vec<Value> = made
            .iter()
            .map(|(f, s)| json!({ "file": f, "signature": s }))
            .collect();
        println!("{:#}", json!(rows));
    } else if !cx.quiet {
        for (_, signature) in &made {
            println!("{signature}");
        }
    }
    Ok(())
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
    files: &[String],
    trust: &[PathBuf],
    only: bool,
    cx: &mut Context,
) -> Result<(), Exit> {
    let mut trusted: Vec<(String, PublicKey)> = Vec::new();
    if !only {
        trusted.extend(crate::settings::trusted(&crate::settings::text()));
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
        let place = Place::of(file)?;
        let (control, bar) =
            crate::progress::control(&format!("Checking {}", place.name()), cx.progress);
        let signed = check_signature(&*place.storage, &place.path, &keys, &control)
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
        rows.push((file.clone(), signed, fingerprint, why));
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
