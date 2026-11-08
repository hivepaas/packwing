//! A file's checksums, read through once for all of those asked for, kept beside it as
//! `sha256sum` and the others write them, and a checksum someone gave found in what
//! they pasted. A checksum says a file is the one it was made of, not who made it:
//! only a signature says that.

use crate::audit;
use packwing_i18n::tr;
use serde_json::json;
use std::io::Read;

use md5::Md5;
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha512};

use crate::job::Control;
use crate::storage::{Item, Storage};
use crate::{Error, Progress};

/// How much of the file is read at a time.
const CHUNK: usize = 1 << 20;

/// A way to make a checksum.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Algorithm {
    Sha256,
    Sha512,
    Sha1,
    Md5,
    Blake3,
}

impl Algorithm {
    /// Every one, in the order they are shown.
    pub const ALL: [Algorithm; 5] = [
        Algorithm::Sha256,
        Algorithm::Sha512,
        Algorithm::Sha1,
        Algorithm::Md5,
        Algorithm::Blake3,
    ];

    /// What it is called, as anyone writes it: a name, not words to translate.
    pub fn name(self) -> &'static str {
        match self {
            Algorithm::Sha256 => "SHA-256",
            Algorithm::Sha512 => "SHA-512",
            Algorithm::Sha1 => "SHA-1",
            Algorithm::Md5 => "MD5",
            Algorithm::Blake3 => "BLAKE3",
        }
    }

    /// What it is kept as in the settings, and the extension `sha256sum` and the
    /// others give a file of them.
    pub fn key(self) -> &'static str {
        match self {
            Algorithm::Sha256 => "sha256",
            Algorithm::Sha512 => "sha512",
            Algorithm::Sha1 => "sha1",
            Algorithm::Md5 => "md5",
            Algorithm::Blake3 => "b3",
        }
    }

    pub fn keyed(key: &str) -> Option<Algorithm> {
        Algorithm::ALL.into_iter().find(|a| a.key() == key)
    }

    /// How many hex digits its checksum has.
    pub fn digits(self) -> usize {
        match self {
            Algorithm::Md5 => 32,
            Algorithm::Sha1 => 40,
            Algorithm::Sha256 | Algorithm::Blake3 => 64,
            Algorithm::Sha512 => 128,
        }
    }

    /// Two files can be made to share its checksum: one still tells a damaged
    /// download, not one changed on purpose.
    pub fn is_weak(self) -> bool {
        matches!(self, Algorithm::Sha1 | Algorithm::Md5)
    }
}

/// The algorithms whose checksums are `digits` long.
pub fn algorithms_of(digits: usize) -> Vec<Algorithm> {
    Algorithm::ALL
        .into_iter()
        .filter(|a| a.digits() == digits)
        .collect()
}

enum Hasher {
    Sha256(Sha256),
    Sha512(Sha512),
    Sha1(Sha1),
    Md5(Md5),
    Blake3(Box<blake3::Hasher>),
}

impl Hasher {
    fn new(algorithm: Algorithm) -> Hasher {
        match algorithm {
            Algorithm::Sha256 => Hasher::Sha256(Sha256::new()),
            Algorithm::Sha512 => Hasher::Sha512(Sha512::new()),
            Algorithm::Sha1 => Hasher::Sha1(Sha1::new()),
            Algorithm::Md5 => Hasher::Md5(Md5::new()),
            Algorithm::Blake3 => Hasher::Blake3(Box::new(blake3::Hasher::new())),
        }
    }

    fn update(&mut self, bytes: &[u8]) {
        match self {
            Hasher::Sha256(h) => h.update(bytes),
            Hasher::Sha512(h) => h.update(bytes),
            Hasher::Sha1(h) => h.update(bytes),
            Hasher::Md5(h) => h.update(bytes),
            Hasher::Blake3(h) => {
                h.update(bytes);
            }
        }
    }

    fn hex(self) -> String {
        match self {
            Hasher::Sha256(h) => hex(&h.finalize()),
            Hasher::Sha512(h) => hex(&h.finalize()),
            Hasher::Sha1(h) => hex(&h.finalize()),
            Hasher::Md5(h) => hex(&h.finalize()),
            Hasher::Blake3(h) => hex(h.finalize().as_bytes()),
        }
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// The checksums of what `reader` holds, in the order of `algorithms`, in lowercase
/// hex.
pub fn checksums_of(
    mut reader: impl Read,
    algorithms: &[Algorithm],
    control: &Control,
) -> Result<Vec<(Algorithm, String)>, Error> {
    let mut hashers: Vec<_> = algorithms.iter().map(|a| Hasher::new(*a)).collect();
    let mut buf = vec![0; CHUNK];
    let mut progress = Progress::default();
    loop {
        control.check()?;
        let n = match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(Error::Failed(e.to_string())),
        };
        for h in &mut hashers {
            h.update(&buf[..n]);
        }
        progress.bytes += n as u64;
        control.report(&progress);
    }
    Ok(algorithms
        .iter()
        .copied()
        .zip(hashers.into_iter().map(Hasher::hex))
        .collect())
}

/// The checksums of the file at `path` of `storage`, all of it read once.
pub fn checksums(
    storage: &dyn Storage,
    path: &[String],
    algorithms: &[Algorithm],
    control: &Control,
) -> Result<Vec<(Algorithm, String)>, Error> {
    let job = audit::begin(
        "checksum",
        audit::fields([("file", json!(audit::place(storage, path)))]),
    );
    let result = storage
        .open(path)
        .map_err(|e| Error::Failed(e.to_string()))
        .and_then(|reader| checksums_of(reader, algorithms, control));
    audit::end(job, &result, |sums| {
        let sums: serde_json::Map<String, serde_json::Value> = sums
            .iter()
            .map(|(a, sum)| (a.name().to_lowercase(), json!(sum)))
            .collect();
        audit::fields([("sums", json!(sums))])
    });
    result
}

/// The name of the file beside the one called `name` that keeps its `algorithm`
/// checksum, as `sha256sum` and the others are used to: `name.sha256`.
pub fn checksum_file_name(name: &str, algorithm: Algorithm) -> String {
    format!("{name}.{}", algorithm.key())
}

/// A line of a file of checksums as `sha256sum` writes it, and `sha256sum -c` checks
/// it: the checksum, two spaces and the name. A name with a backslash or a line's end
/// in it has them escaped, and the line starts with a backslash to say so.
pub fn checksum_line(sum: &str, name: &str) -> String {
    if name.contains(['\\', '\n', '\r']) {
        let escaped = name
            .replace('\\', "\\\\")
            .replace('\n', "\\n")
            .replace('\r', "\\r");
        format!("\\{sum}  {escaped}\n")
    } else {
        format!("{sum}  {name}\n")
    }
}

/// Keeps `sum`, the `algorithm` checksum of the file at `path`, beside it in a file of
/// its own, in place of one that is there. Says what that file is called.
pub fn save_checksum(
    storage: &dyn Storage,
    path: &[String],
    algorithm: Algorithm,
    sum: &str,
) -> Result<String, Error> {
    let Some((name, folder)) = path.split_last() else {
        return Err(Error::Failed(tr!("There is no file to save it beside.")));
    };
    let kept = checksum_file_name(name, algorithm);
    let line = checksum_line(sum, name);
    let like = Item {
        name: kept.clone(),
        dir: false,
        link: false,
        hidden: false,
        size: line.len() as u64,
        modified: None,
        mode: Some(0o644),
    };
    let mut to = folder.to_vec();
    to.push(kept.clone());
    storage
        .put(&to, &mut line.as_bytes(), &like, true)
        .map_err(|e| Error::Failed(e.to_string()))?;
    Ok(kept)
}

/// The checksum in what was pasted, in lowercase hex, whatever its length: a
/// checksum alone, written in capitals, in groups or with colons, as `sha256sum`
/// writes it (`<hex>  name`), as BSD's and `sha256sum --tag` do
/// (`SHA256 (name) = <hex>`), or `sha256:<hex>`. Of several lines, as a
/// `SHA256SUMS` holds, the one for the file `name`, or else the first.
pub fn checksum_in(pasted: &str, name: &str) -> Option<String> {
    let lines: Vec<&str> = pasted
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    let line = lines.iter().find(|l| names(l, name)).or(lines.first())?;
    let is_hex = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_hexdigit());
    let joined: String = line
        .chars()
        .filter(|c| !c.is_whitespace() && *c != ':')
        .collect();
    let after = line
        .rsplit_once([':', '='])
        .map_or("", |(_, rest)| rest.trim());
    let first = line.split_whitespace().next().unwrap_or_default();
    let found: Vec<&str> = [joined.as_str(), after, first]
        .into_iter()
        .filter(|s| is_hex(s))
        .collect();
    found
        .iter()
        .find(|s| !algorithms_of(s.len()).is_empty())
        .or(found.first())
        .map(|s| s.to_ascii_lowercase())
}

/// Whether the line `line` of a list of checksums is the one for the file `name`.
fn names(line: &str, name: &str) -> bool {
    if line.contains(&format!("({name})")) {
        return true;
    }
    line.split_once(char::is_whitespace)
        .is_some_and(|(_, rest)| {
            let rest = rest.trim_start();
            rest.strip_prefix('*').unwrap_or(rest) == name
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn of(bytes: &[u8], a: Algorithm) -> String {
        checksums_of(bytes, &[a], &Control::default())
            .expect("read")
            .remove(0)
            .1
    }

    #[test]
    fn each_says_what_its_tools_say() {
        assert_eq!(
            of(b"abc", Algorithm::Sha256),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            of(b"abc", Algorithm::Sha512),
            "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a\
             2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f"
        );
        assert_eq!(
            of(b"abc", Algorithm::Sha1),
            "a9993e364706816aba3e25717850c26c9cd0d89d"
        );
        assert_eq!(
            of(b"abc", Algorithm::Md5),
            "900150983cd24fb0d6963f7d28e17f72"
        );
        assert_eq!(
            of(b"", Algorithm::Blake3),
            "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"
        );
        for a in Algorithm::ALL {
            assert_eq!(of(b"abc", a).len(), a.digits(), "{}", a.name());
            assert_eq!(Algorithm::keyed(a.key()), Some(a));
        }
    }

    #[test]
    fn all_are_made_in_one_read_across_chunks() {
        let big = vec![7u8; CHUNK * 2 + 5];
        let all = checksums_of(big.as_slice(), &Algorithm::ALL, &Control::default()).expect("read");
        assert_eq!(all.len(), Algorithm::ALL.len());
        for (a, sum) in all {
            assert_eq!(sum, of(&big, a));
        }
    }

    #[test]
    fn a_stopped_read_says_so() {
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
            checksums_of(&b"abc"[..], &[Algorithm::Sha256], &control),
            Err(Error::Cancelled)
        ));
    }

    #[test]
    fn a_checksum_is_found_however_it_was_written() {
        let sum = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
        let found = |text: &str| checksum_in(text, "a.iso");
        assert_eq!(found(sum).as_deref(), Some(sum));
        assert_eq!(
            found(&format!("  {}\n", sum.to_uppercase())).as_deref(),
            Some(sum)
        );
        assert_eq!(found(&format!("{sum}  a.iso")).as_deref(), Some(sum));
        assert_eq!(found(&format!("{sum} *a.iso")).as_deref(), Some(sum));
        assert_eq!(
            found(&format!("SHA256 (a.iso) = {sum}")).as_deref(),
            Some(sum)
        );
        assert_eq!(found(&format!("sha256:{sum}")).as_deref(), Some(sum));
        let grouped = sum
            .as_bytes()
            .chunks(8)
            .map(|c| std::str::from_utf8(c).expect("ascii"))
            .collect::<Vec<_>>()
            .join(" ");
        assert_eq!(found(&grouped).as_deref(), Some(sum));
        let colons = sum
            .as_bytes()
            .chunks(2)
            .map(|c| std::str::from_utf8(c).expect("ascii"))
            .collect::<Vec<_>>()
            .join(":");
        assert_eq!(found(&colons).as_deref(), Some(sum));
        // A file named in hex is not taken for more of the checksum.
        assert_eq!(found(&format!("{sum}  cafe")).as_deref(), Some(sum));
        assert_eq!(found("abc").as_deref(), Some("abc"));
        assert_eq!(found("not one"), None);
        assert_eq!(found("  \n "), None);
    }

    #[test]
    fn of_a_list_the_line_for_the_file_is_taken() {
        let (a, b) = ("1".repeat(64), "2".repeat(64));
        let list = format!("{a}  other.iso\n{b}  a.iso\n");
        assert_eq!(checksum_in(&list, "a.iso"), Some(b.clone()));
        assert_eq!(checksum_in(&list, "missing.iso"), Some(a.clone()));
        let tagged = format!("SHA256 (other.iso) = {a}\nSHA256 (a.iso) = {b}");
        assert_eq!(checksum_in(&tagged, "a.iso"), Some(b));
    }

    #[test]
    fn a_line_is_as_sha256sum_writes_it() {
        assert_eq!(checksum_line("ab", "a.iso"), "ab  a.iso\n");
        assert_eq!(checksum_line("ab", "a\\b\nc"), "\\ab  a\\\\b\\nc\n");
        assert_eq!(
            checksum_file_name("a.iso", Algorithm::Sha256),
            "a.iso.sha256"
        );
        assert_eq!(checksum_file_name("a.iso", Algorithm::Blake3), "a.iso.b3");
    }

    /// Each kept beside the file, and checked by its tool, if this system has it.
    #[test]
    fn what_is_kept_is_what_the_tools_check() {
        let dir = std::env::temp_dir().join(format!("packwing-checksum-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        std::fs::write(dir.join("a b.iso"), vec![7u8; 300_000]).expect("write");
        let folder = crate::LocalFolder::new(&dir);
        let path = ["a b.iso".to_owned()];
        let all = checksums(&folder, &path, &Algorithm::ALL, &Control::default()).expect("read");
        for (a, sum) in all {
            let kept = save_checksum(&folder, &path, a, &sum).expect("kept");
            assert_eq!(kept, checksum_file_name("a b.iso", a));
            let text = std::fs::read_to_string(dir.join(&kept)).expect("read");
            assert_eq!(checksum_in(&text, "a b.iso"), Some(sum));
            let tool = format!("{}sum", a.key());
            match std::process::Command::new(&tool)
                .arg("-c")
                .arg(&kept)
                .current_dir(&dir)
                .output()
            {
                Ok(out) => assert!(out.status.success(), "{tool}: {out:?}"),
                Err(_) => eprintln!("no {tool}: skipped"),
            }
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_length_says_which_it_may_be() {
        assert_eq!(algorithms_of(40), [Algorithm::Sha1]);
        assert_eq!(algorithms_of(64), [Algorithm::Sha256, Algorithm::Blake3]);
        assert!(algorithms_of(10).is_empty());
    }
}
