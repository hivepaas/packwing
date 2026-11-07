//! Files signed with a key: the signature goes beside the file, as `name.sig`, in
//! the format of `ssh-keygen -Y sign`. A signature proves something only by a key
//! the user trusts: one by any other key says nothing, whatever it matches.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use hivepack::age::RandomSource;
use hivepack::sign::{Error as SignError, FILE_NAMESPACE, PrivateKey, Signature, sign, verify};
use packwing_i18n::tr;

pub use hivepack::sign::PublicKey;

use crate::job::Control;
use crate::source::Source;
use crate::storage::{Item, LocalFolder, Storage};
use crate::transfer::Watched;
use crate::{Error, Progress};

/// The most a signature file is read of: far more than a signature is, as a file
/// of that name may be anything.
const MOST_SIGNATURE: u64 = 64 << 10;

/// What a new key's file is called, in the folder chosen for it.
const NEW_KEY_NAME: &str = "packwing-signing-key";

/// The name of the signature beside the file called `name`.
pub fn signature_name(name: &str) -> String {
    format!("{name}.sig")
}

/// Whether `name` is a signature's, going by its name: one is not signed again.
pub fn is_signature_name(name: &str) -> bool {
    name.to_ascii_lowercase().ends_with(".sig")
}

/// Whether `name` is one part of a file cut in several, going by its name: a
/// signature covers one file, and would seem to cover them all.
pub fn is_part_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    let Some((base, extension)) = lower.rsplit_once('.') else {
        return false;
    };
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    let numbered = |letter: char| {
        extension.len() == 3 && extension.starts_with(letter) && digits(&extension[1..])
    };
    let part_of_rar = extension == "rar"
        && base
            .rsplit_once(".part")
            .is_some_and(|(_, number)| digits(number));
    (extension.len() == 3 && digits(extension)) || numbered('r') || numbered('z') || part_of_rar
}

/// Whether the file at `path` on this computer has a signature beside it.
pub fn has_signature(path: &Path) -> bool {
    path.file_name().is_some_and(|name| {
        path.with_file_name(signature_name(&name.to_string_lossy()))
            .is_file()
    })
}

/// A private key, opened to sign with. Cleared when the last of it goes.
#[derive(Clone)]
pub struct SigningKey(Arc<PrivateKey>);

impl SigningKey {
    pub fn public(&self) -> &PublicKey {
        self.0.public()
    }
}

impl PartialEq for SigningKey {
    fn eq(&self, other: &SigningKey) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl std::fmt::Debug for SigningKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SigningKey({})", self.public().fingerprint())
    }
}

/// Why a private key was not opened.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyRefused {
    /// It is encrypted, and no passphrase was given.
    NeedsPassphrase,
    WrongPassphrase,
    /// It is no key that can sign, and why.
    Not(String),
}

/// The private key in the text of `file`: an SSH Ed25519 key's, or a hybrid key's.
pub fn read_signing_key(file: &[u8], passphrase: Option<&[u8]>) -> Result<SigningKey, KeyRefused> {
    match PrivateKey::parse(file, passphrase) {
        Ok(key) => Ok(SigningKey(Arc::new(key))),
        Err(SignError::PassphraseNeeded) => Err(KeyRefused::NeedsPassphrase),
        Err(SignError::WrongPassphrase) => Err(KeyRefused::WrongPassphrase),
        Err(SignError::PrivateKey(hivepack::age::SshKeyError::IncorrectPassword)) => {
            Err(KeyRefused::WrongPassphrase)
        }
        Err(e) => Err(KeyRefused::Not(e.to_string())),
    }
}

/// The public key of a line of a `.pub` file, the first of `text` that is one.
pub fn read_public_key(text: &str) -> Result<PublicKey, String> {
    let line = text
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with('#'))
        .ok_or_else(|| tr!("There is no public key here."))?;
    PublicKey::parse(line.as_bytes()).map_err(|e| e.to_string())
}

fn random() -> RandomSource {
    Arc::new(|buffer: &mut [u8]| getrandom::fill(buffer).map_err(io::Error::other))
}

/// Makes a hybrid key of Ed25519 and ML-DSA-65 in `folder`, under the first name
/// free, its private file encrypted with `passphrase` and its `.pub` beside it.
/// Nothing that is there is written over. It gives the key and its file.
pub fn make_signing_key(folder: &Path, passphrase: &[u8]) -> Result<(SigningKey, PathBuf), Error> {
    let bad = |e: String| Error::Failed(tr!("Could not make the key: {e}", e = e));
    let random = random();
    let key = PrivateKey::generate_hybrid(&random).map_err(|e| bad(e.to_string()))?;
    let mut file = key
        .to_file(Some(passphrase), &random)
        .map_err(|e| bad(e.to_string()))?;
    let public = format!("{}\n", key.public());
    let (mut written, path) = free_key_file(folder).map_err(|e| bad(e.to_string()))?;
    let wrote = written
        .write_all(&file)
        .and_then(|()| written.sync_all())
        .and_then(|()| fs::write(public_file(&path), &public));
    file.fill(0);
    if let Err(e) = wrote {
        let _ = fs::remove_file(&path);
        return Err(bad(e.to_string()));
    }
    Ok((SigningKey(Arc::new(key)), path))
}

/// A new private file in `folder`, under the first name with no `.pub` either.
fn free_key_file(folder: &Path) -> io::Result<(fs::File, PathBuf)> {
    let mut n = 1u32;
    loop {
        let path = match n {
            1 => folder.join(NEW_KEY_NAME),
            n => folder.join(format!("{NEW_KEY_NAME}-{n}")),
        };
        n += 1;
        if public_file(&path).exists() {
            continue;
        }
        match create_private(&path) {
            Ok(file) => return Ok((file, path)),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e),
        }
    }
}

fn public_file(private: &Path) -> PathBuf {
    let mut name = private.as_os_str().to_owned();
    name.push(".pub");
    PathBuf::from(name)
}

/// A new file only its owner can read.
fn create_private(path: &Path) -> io::Result<fs::File> {
    let mut options = fs::File::options();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path)
}

fn signature_path(path: &[String]) -> Option<Vec<String>> {
    let (name, folder) = path.split_last()?;
    let mut at = folder.to_vec();
    at.push(signature_name(name));
    Some(at)
}

/// Reads the file at `path` of `storage` through and writes its signature beside
/// it, in place of one that is there.
pub fn sign_file(
    storage: &dyn Storage,
    path: &[String],
    key: &SigningKey,
    control: &Control,
) -> Result<(), Error> {
    let failed = |e: String| Error::Failed(e);
    let (Some((name, folder)), Some(to)) = (path.split_last(), signature_path(path)) else {
        return Err(failed(tr!("There is no file to sign.")));
    };
    let item = storage
        .stat(folder, name)
        .map_err(|e| failed(e.to_string()))?
        .filter(|item| !item.dir)
        .ok_or_else(|| failed(tr!("{name} is not there any more.", name = name)))?;
    let mut progress = Progress::default();
    progress.total_entries = 1;
    progress.total_bytes = item.size;
    let reader = Watched {
        inner: storage.open(path).map_err(|e| failed(e.to_string()))?,
        crc: 0,
        control,
        progress: &mut progress,
    };
    let signature = match sign(&key.0, FILE_NAMESPACE, reader) {
        Ok(signature) => signature,
        Err(_) if control.check().is_err() => return Err(Error::Cancelled),
        Err(e) => return Err(failed(e.to_string())),
    };
    let armored = signature.to_armored();
    let like = Item {
        name: signature_name(name),
        dir: false,
        link: false,
        hidden: false,
        size: armored.len() as u64,
        modified: None,
        mode: Some(0o644),
    };
    storage
        .put(&to, &mut armored.as_bytes(), &like, true)
        .map_err(|e| failed(e.to_string()))?;
    Ok(())
}

/// What is known of a file's signature.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Signed {
    /// There is no signature beside it.
    No,
    /// The file beside it is no signature packwing can read, and why.
    Unreadable(String),
    /// By a key the user does not trust: the file was not read, as it would prove
    /// nothing either way.
    Stranger { fingerprint: String },
    /// By a key the user trusts, but the file has not been read yet.
    NotChecked { fingerprint: String },
    /// The file is what the key the user trusts signed.
    Good { fingerprint: String },
    /// The signature names a key the user trusts, and the file is not what it signed.
    Mismatch { fingerprint: String },
    /// The file could not be read, and why.
    Failed(String),
}

/// The signature beside the file at `path`, or what is known without one.
fn read_signature(storage: &dyn Storage, path: &[String]) -> Result<Signature, Signed> {
    let at = signature_path(path).ok_or(Signed::No)?;
    let (name, folder) = at.split_last().ok_or(Signed::No)?;
    match storage.stat(folder, name) {
        Ok(Some(item)) if !item.dir => {}
        Ok(_) => return Err(Signed::No),
        Err(e) => return Err(Signed::Failed(e.to_string())),
    }
    let bytes = storage
        .read(&at, MOST_SIGNATURE)
        .map_err(|e| Signed::Failed(e.to_string()))?;
    Signature::parse(&bytes).map_err(|e| Signed::Unreadable(e.to_string()))
}

/// A file's signature, read and compared with the keys trusted, without reading
/// the file.
pub fn look_at_signature(storage: &dyn Storage, path: &[String], trusted: &[PublicKey]) -> Signed {
    match read_signature(storage, path) {
        Err(known) => known,
        Ok(signature) => {
            let fingerprint = signature.public_key().fingerprint();
            if trusted.contains(signature.public_key()) {
                Signed::NotChecked { fingerprint }
            } else {
                Signed::Stranger { fingerprint }
            }
        }
    }
}

/// [`look_at_signature`], and a signature by a key trusted checked against all of
/// the file.
pub fn check_signature(
    storage: &dyn Storage,
    path: &[String],
    trusted: &[PublicKey],
    control: &Control,
) -> Result<Signed, Error> {
    let signature = match read_signature(storage, path) {
        Ok(signature) => signature,
        Err(known) => return Ok(known),
    };
    let fingerprint = signature.public_key().fingerprint();
    let Some(key) = trusted.iter().find(|k| *k == signature.public_key()) else {
        return Ok(Signed::Stranger { fingerprint });
    };
    let inner = match storage.open(path) {
        Ok(inner) => inner,
        Err(e) => return Ok(Signed::Failed(e.to_string())),
    };
    let mut progress = Progress::default();
    let reader = Watched {
        inner,
        crc: 0,
        control,
        progress: &mut progress,
    };
    Ok(match verify(&signature, key, FILE_NAMESPACE, reader) {
        Ok(()) => Signed::Good { fingerprint },
        Err(_) if control.check().is_err() => return Err(Error::Cancelled),
        Err(SignError::BadSignature) => Signed::Mismatch { fingerprint },
        Err(e @ SignError::Io(_)) => Signed::Failed(e.to_string()),
        Err(e) => Signed::Unreadable(e.to_string()),
    })
}

impl Source {
    /// The storage it is on and its path there; a file here, by the folder it is in.
    pub fn place(&self) -> Option<(Arc<dyn Storage>, Vec<String>)> {
        match self {
            Source::File(path) => {
                let name = path.file_name()?.to_string_lossy().into_owned();
                let folder = path.parent()?;
                Some((Arc::new(LocalFolder::new(folder)), vec![name]))
            }
            Source::Stored(stored) => Some(stored.place()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Scratch {
            let dir =
                std::env::temp_dir().join(format!("packwing-sign-{}-{name}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).expect("mkdir");
            fs::write(dir.join("a.zip"), vec![7u8; 300_000]).expect("write");
            Scratch(dir)
        }

        fn storage(&self) -> LocalFolder {
            LocalFolder::new(&self.0)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn none() -> Control {
        Control {
            progress: None,
            cancel: None,
            on_damaged: None,
            on_unnamable: None,
            on_links_outside: None,
            on_gone: None,
        }
    }

    fn file() -> Vec<String> {
        vec!["a.zip".to_owned()]
    }

    fn new_key(s: &Scratch) -> SigningKey {
        make_signing_key(&s.0, b"a passphrase").expect("a key").0
    }

    #[test]
    fn what_is_signed_checks_and_a_change_does_not() {
        let s = Scratch::new("round");
        let key = new_key(&s);
        let trusted = [key.public().clone()];
        sign_file(&s.storage(), &file(), &key, &none()).expect("sign");
        assert!(has_signature(&s.0.join("a.zip")));
        assert!(matches!(
            look_at_signature(&s.storage(), &file(), &trusted),
            Signed::NotChecked { .. }
        ));
        let fingerprint = key.public().fingerprint();
        assert_eq!(
            check_signature(&s.storage(), &file(), &trusted, &none()).expect("check"),
            Signed::Good {
                fingerprint: fingerprint.clone()
            }
        );
        fs::write(s.0.join("a.zip"), vec![8u8; 300_000]).expect("write");
        assert_eq!(
            check_signature(&s.storage(), &file(), &trusted, &none()).expect("check"),
            Signed::Mismatch { fingerprint }
        );
    }

    #[test]
    fn a_key_not_trusted_proves_nothing_whatever_it_matches() {
        let s = Scratch::new("stranger");
        let key = new_key(&s);
        sign_file(&s.storage(), &file(), &key, &none()).expect("sign");
        let fingerprint = key.public().fingerprint();
        let other = new_key(&s);
        for trusted in [vec![], vec![other.public().clone()]] {
            assert_eq!(
                check_signature(&s.storage(), &file(), &trusted, &none()).expect("check"),
                Signed::Stranger {
                    fingerprint: fingerprint.clone()
                }
            );
        }
    }

    #[test]
    fn no_signature_and_one_that_is_not_are_told_apart() {
        let s = Scratch::new("none");
        assert_eq!(look_at_signature(&s.storage(), &file(), &[]), Signed::No);
        fs::write(s.0.join("a.zip.sig"), "not one").expect("write");
        assert!(matches!(
            look_at_signature(&s.storage(), &file(), &[]),
            Signed::Unreadable(_)
        ));
        fs::remove_file(s.0.join("a.zip.sig")).expect("remove");
        fs::create_dir(s.0.join("a.zip.sig")).expect("mkdir");
        assert_eq!(look_at_signature(&s.storage(), &file(), &[]), Signed::No);
    }

    #[test]
    fn signing_again_replaces_the_signature() {
        let s = Scratch::new("again");
        let (one, two) = (new_key(&s), new_key(&s));
        sign_file(&s.storage(), &file(), &one, &none()).expect("sign");
        sign_file(&s.storage(), &file(), &two, &none()).expect("sign");
        let trusted = [two.public().clone()];
        assert!(matches!(
            check_signature(&s.storage(), &file(), &trusted, &none()).expect("check"),
            Signed::Good { .. }
        ));
    }

    #[test]
    fn cancelled_it_writes_no_signature() {
        let s = Scratch::new("cancel");
        let key = new_key(&s);
        let cancel = crate::Cancel::new();
        cancel.cancel();
        let control = Control {
            progress: None,
            cancel: Some(cancel),
            on_damaged: None,
            on_unnamable: None,
            on_links_outside: None,
            on_gone: None,
        };
        assert!(matches!(
            sign_file(&s.storage(), &file(), &key, &control),
            Err(Error::Cancelled)
        ));
        assert!(!s.0.join("a.zip.sig").exists());
    }

    #[test]
    fn a_new_key_takes_a_free_name_opens_with_its_passphrase_alone() {
        let s = Scratch::new("new");
        fs::write(s.0.join(NEW_KEY_NAME), "taken").expect("write");
        let (key, path) = make_signing_key(&s.0, b"a passphrase").expect("a key");
        assert_eq!(path, s.0.join(format!("{NEW_KEY_NAME}-2")));
        assert_eq!(fs::read(s.0.join(NEW_KEY_NAME)).expect("read"), b"taken");
        let bytes = fs::read(&path).expect("read");
        assert_eq!(
            read_signing_key(&bytes, None).expect_err("locked"),
            KeyRefused::NeedsPassphrase
        );
        assert_eq!(
            read_signing_key(&bytes, Some(b"wrong")).expect_err("locked"),
            KeyRefused::WrongPassphrase
        );
        let opened = read_signing_key(&bytes, Some(b"a passphrase")).expect("opens");
        assert_eq!(opened.public(), key.public());
        let public = fs::read_to_string(public_file(&path)).expect("read");
        assert_eq!(&read_public_key(&public).expect("reads"), key.public());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&path).expect("meta").permissions().mode();
            assert_eq!(mode & 0o777, 0o600);
        }
    }

    #[test]
    fn parts_and_signatures_are_told_by_their_names() {
        for part in [
            "a.7z.001",
            "a.zip.002",
            "a.part1.rar",
            "a.part02.RAR",
            "a.r00",
            "a.z01",
        ] {
            assert!(is_part_name(part), "{part}");
        }
        for whole in [
            "a.zip",
            "a.rar",
            "a.7z",
            "a.2025",
            "report.r1",
            "a.partial.rar",
            "a",
        ] {
            assert!(!is_part_name(whole), "{whole}");
        }
        assert!(is_signature_name("a.zip.SIG"));
        assert!(!is_signature_name("a.zip"));
    }

    #[test]
    fn an_ssh_key_with_a_passphrase_asks_for_it() {
        let s = Scratch::new("ssh-passphrase");
        let key_file = s.0.join("id");
        let made = Command::new("ssh-keygen")
            .args(["-q", "-t", "ed25519", "-N", "right one", "-f"])
            .arg(&key_file)
            .status();
        if !made.is_ok_and(|status| status.success()) {
            eprintln!("no ssh-keygen: skipped");
            return;
        }
        let bytes = fs::read(&key_file).expect("read");
        assert_eq!(
            read_signing_key(&bytes, None).expect_err("locked"),
            KeyRefused::NeedsPassphrase
        );
        assert_eq!(
            read_signing_key(&bytes, Some(b"wrong")).expect_err("locked"),
            KeyRefused::WrongPassphrase
        );
        read_signing_key(&bytes, Some(b"right one")).expect("opens");
        assert!(matches!(
            read_signing_key(b"not a key", None),
            Err(KeyRefused::Not(_))
        ));
    }

    /// `ssh-keygen`, if this system has it: what the two make of each other's.
    #[test]
    fn ssh_keygen_and_packwing_agree() {
        let s = Scratch::new("oracle");
        let key_file = s.0.join("id");
        let made = Command::new("ssh-keygen")
            .args(["-q", "-t", "ed25519", "-N", "", "-C", "me", "-f"])
            .arg(&key_file)
            .status();
        if !made.is_ok_and(|status| status.success()) {
            eprintln!("no ssh-keygen: skipped");
            return;
        }
        let key = read_signing_key(&fs::read(&key_file).expect("read"), None).expect("opens");
        let public = fs::read_to_string(s.0.join("id.pub")).expect("read");
        assert_eq!(&read_public_key(&public).expect("reads"), key.public());
        sign_file(&s.storage(), &file(), &key, &none()).expect("sign");
        let line = format!("me {public}");
        fs::write(s.0.join("signers"), line).expect("write");
        let run = |sig: &str| {
            Command::new("ssh-keygen")
                .args(["-Y", "verify", "-n", "file", "-I", "me", "-f"])
                .arg(s.0.join("signers"))
                .arg("-s")
                .arg(s.0.join(sig))
                .stdin(fs::File::open(s.0.join("a.zip")).expect("open"))
                .output()
                .expect("runs")
                .status
                .success()
        };
        assert!(run("a.zip.sig"));
        let ours = fs::read(s.0.join("a.zip.sig")).expect("read");
        fs::remove_file(s.0.join("a.zip.sig")).expect("remove");
        let signed = Command::new("ssh-keygen")
            .args(["-q", "-Y", "sign", "-n", "file", "-f"])
            .arg(&key_file)
            .arg(s.0.join("a.zip"))
            .status()
            .expect("runs");
        assert!(signed.success());
        assert_eq!(fs::read(s.0.join("a.zip.sig")).expect("read"), ours);
        let trusted = [key.public().clone()];
        assert!(matches!(
            check_signature(&s.storage(), &file(), &trusted, &none()).expect("check"),
            Signed::Good { .. }
        ));
    }
}
