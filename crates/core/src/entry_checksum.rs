//! The checksums of the files in an archive, all of them or those picked, made in one
//! read of it as it is extracted, with nothing written; a list of them as `sha256sum`
//! writes one, to check what is extracted with; and a list someone gave compared with
//! them.

use std::collections::{HashMap, HashSet};
use std::io;
use std::sync::Arc;

use serde_json::json;

use crate::audit;
use crate::checksum::{Algorithm, Hasher, Listed, algorithms_of, checksum_line, put_beside};
use crate::destination::{Destination, extract_to};
use crate::job::{Control, Damage, ExtractOptions, damages};
use crate::source::Source;
use crate::storage::Storage;
use crate::{Error, Keys, OnDamaged};

/// A file of an archive, and its checksums.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntrySums {
    /// Its path in the archive.
    pub path: Vec<String>,
    /// What a list of checksums names it: its path, `/` apart, from the folder it is
    /// extracted into. Of all of the archive, its path in it; of what was picked, as
    /// [`extract_selected`](crate::extract_selected) puts it, `docs/a/b.txt` picked
    /// is `b.txt`, and `docs/a` picked makes it `a/b.txt`.
    pub name: String,
    /// Its bytes.
    pub size: u64,
    /// In the order they were asked for.
    pub sums: Vec<(Algorithm, String)>,
}

impl EntrySums {
    /// Its `algorithm` checksum, if it was made.
    pub fn sum(&self, algorithm: Algorithm) -> Option<&str> {
        self.sums
            .iter()
            .find(|(a, _)| *a == algorithm)
            .map(|(_, s)| s.as_str())
    }
}

/// What [`entry_checksums`] made.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EntryChecksums {
    /// The files, by name.
    pub files: Vec<EntrySums>,
    /// The symbolic links met, which have none: a link holds no data, and a
    /// checking tool reads what it leads to.
    pub links: u64,
    /// The files whose data is damaged, which have none either: the others are
    /// made all the same.
    pub damaged: Vec<Damage>,
}

/// The checksums of the files of the archive of `source`, each read once for all
/// of `algorithms`, the archive read once for all of them: every file of it if
/// `picked` is empty, or else those picked, files or folders given by their paths
/// in it, each with what is under it. Nothing is written. A file whose data is
/// damaged is left out and said ([`EntryChecksums::damaged`]), whatever
/// [`Control::on_damaged`] says; what else goes wrong stops it.
pub fn entry_checksums(
    source: impl Into<Source>,
    picked: &[Vec<String>],
    keys: &Keys,
    algorithms: &[Algorithm],
    control: &Control,
) -> Result<EntryChecksums, Error> {
    let source = source.into();
    let mut fields = audit::fields([
        ("source", audit::source(&source)),
        (
            "algorithms",
            json!(algorithms.iter().map(|a| a.key()).collect::<Vec<_>>()),
        ),
    ]);
    if !picked.is_empty() {
        fields.insert("picked".into(), json!(picked.len()));
        fields.extend(audit::fields([(
            "picked_names",
            audit::list(picked.iter().map(|p| p.join("/"))),
        )]));
    }
    let job = audit::begin("checksum_entries", fields)?;
    let result = made(source, picked, keys, algorithms, control);
    // Counted, not named, but where records name what packwing came on; never
    // the checksums of what is in a file the user did not choose by name.
    audit::end(job, &result, |made| {
        let mut fields = audit::fields([
            ("files", json!(made.files.len())),
            ("links", json!(made.links)),
            ("damaged", json!(made.damaged.len())),
            (
                "damaged_names",
                audit::names(made.damaged.iter().map(|d| d.path.clone())),
            ),
        ]);
        if !made.damaged.is_empty() {
            fields.insert("result".into(), json!("partial"));
        }
        fields
    });
    result
}

fn made(
    source: Source,
    picked: &[Vec<String>],
    keys: &Keys,
    algorithms: &[Algorithm],
    control: &Control,
) -> Result<EntryChecksums, Error> {
    let control = Control {
        on_damaged: Some(Arc::new(|_: &Damage| OnDamaged::Skip)),
        ..control.clone()
    };
    // Nothing is written, so no limit of what may be is wanted: the ratio of
    // what is read to what it was compressed to still stops a bomb.
    let how = ExtractOptions {
        limit: None,
        keep_mark: false,
        ..ExtractOptions::default()
    };
    let mut hashing = Hashing {
        algorithms,
        picked,
        open: None,
        files: Vec::new(),
        at: HashMap::new(),
        links: 0,
    };
    let summary = extract_to(source, &mut hashing, picked, keys, &how, &control)?;
    let mut files = hashing.files;
    files.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(EntryChecksums {
        files,
        links: hashing.links,
        damaged: damages(&summary),
    })
}

/// What [`entry_checksums`] extracts into: each file's data through the hashers.
struct Hashing<'a> {
    algorithms: &'a [Algorithm],
    picked: &'a [Vec<String>],
    /// The file being read: its path, its bytes so far, and its hashers.
    open: Option<(Vec<String>, u64, Vec<Hasher>)>,
    files: Vec<EntrySums>,
    /// Where each file is in `files`, by its path, for a hard link to it.
    at: HashMap<Vec<String>, usize>,
    links: u64,
}

impl Hashing<'_> {
    fn push(&mut self, path: Vec<String>, size: u64, sums: Vec<(Algorithm, String)>) {
        let name = name_of(&path, self.picked);
        self.at.insert(path.clone(), self.files.len());
        self.files.push(EntrySums {
            path,
            name,
            size,
            sums,
        });
    }
}

impl Destination for Hashing<'_> {
    fn directory(&mut self, _path: &[String]) -> io::Result<()> {
        Ok(())
    }

    fn start_file(&mut self, path: &[String], _size: Option<u64>) -> io::Result<()> {
        let hashers = self.algorithms.iter().map(|a| Hasher::new(*a)).collect();
        self.open = Some((path.to_vec(), 0, hashers));
        Ok(())
    }

    fn write(&mut self, data: &[u8]) -> io::Result<()> {
        if let Some((_, size, hashers)) = &mut self.open {
            *size += data.len() as u64;
            for h in hashers {
                h.update(data);
            }
        }
        Ok(())
    }

    fn finish_file(&mut self) -> io::Result<()> {
        if let Some((path, size, hashers)) = self.open.take() {
            let sums = self
                .algorithms
                .iter()
                .copied()
                .zip(hashers.into_iter().map(Hasher::hex))
                .collect();
            self.push(path, size, sums);
        }
        Ok(())
    }

    fn discard_file(&mut self) {
        self.open = None;
    }

    fn symlink(&mut self, _path: &[String], _target: &str) -> io::Result<()> {
        self.links += 1;
        Ok(())
    }

    /// The same data as the file it is another name for, extracted as a file.
    fn hard_link(&mut self, path: &[String], target: &[String]) -> io::Result<()> {
        if let Some(&i) = self.at.get(target) {
            let (size, sums) = (self.files[i].size, self.files[i].sums.clone());
            self.push(path.to_vec(), size, sums);
        }
        Ok(())
    }
}

/// What a list names the file at `path`, of those `picked`: see [`EntrySums::name`].
fn name_of(path: &[String], picked: &[Vec<String>]) -> String {
    picked
        .iter()
        .find(|p| !p.is_empty() && path.starts_with(p))
        .map_or_else(|| path.join("/"), |p| path[p.len() - 1..].join("/"))
}

/// The list of the `algorithm` checksums of `files`, as `sha256sum` writes one and
/// `sha256sum -c` checks it, in the folder they are extracted into. A file of which
/// that one was not made is not in it.
pub fn checksum_list(files: &[EntrySums], algorithm: Algorithm) -> String {
    files
        .iter()
        .filter_map(|f| f.sum(algorithm).map(|sum| checksum_line(sum, &f.name)))
        .collect()
}

/// The name of the file beside the archive called `name` that keeps the list of the
/// `algorithm` checksums of what is in it: `name.contents.sha256`. Its own are kept
/// in `name.sha256` ([`checksum_file_name`](crate::checksum_file_name)).
pub fn checksum_list_file_name(name: &str, algorithm: Algorithm) -> String {
    format!("{name}.contents.{}", algorithm.key())
}

/// Keeps the list of the `algorithm` checksums of `files` beside the archive at
/// `path` of `storage`, in place of one that is there. Says what that file is
/// called.
pub fn save_checksum_list(
    storage: &dyn Storage,
    path: &[String],
    algorithm: Algorithm,
    files: &[EntrySums],
) -> Result<String, Error> {
    let name = path.last().map(String::as_str).unwrap_or_default();
    let kept = checksum_list_file_name(name, algorithm);
    put_beside(storage, path, &kept, &checksum_list(files, algorithm))?;
    Ok(kept)
}

/// What a list of checksums says of a file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// Its checksum is the one listed.
    Matches(Algorithm),
    /// The one listed is none of `made`, made of it: if `not_made` is empty, it
    /// is not the file it was made of, or it changed. It may be one of
    /// `not_made`, which were not made of it.
    Differs {
        made: Vec<Algorithm>,
        not_made: Vec<Algorithm>,
    },
    /// The one listed is no checksum packwing makes: it has this many digits.
    Unknown(usize),
    /// The list does not name it.
    NotListed,
}

/// What a list of checksums says of the files it was compared with.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ListCompared {
    /// Of each file, in their order.
    pub verdicts: Vec<Verdict>,
    /// The names the list has that none of the files is, in its order.
    pub not_found: Vec<String>,
}

impl ListCompared {
    /// How many files the list says are as it says.
    pub fn matching(&self) -> usize {
        self.verdicts
            .iter()
            .filter(|v| matches!(v, Verdict::Matches(_)))
            .count()
    }

    /// Whether every file is in the list and as it says, and the list names nothing
    /// else.
    pub fn all_match(&self) -> bool {
        self.not_found.is_empty() && self.matching() == self.verdicts.len()
    }
}

/// `listed` compared with `files`. A line is a file's where it names it as a list
/// made of it would, or by its path in the archive; or else where one name ends the
/// other at a `/` (`./a/b.txt`, `pkg/a/b.txt` for `a/b.txt`), if it is the only such
/// line. Backslashes are taken for `/`.
pub fn compare_list(files: &[EntrySums], listed: &[Listed]) -> ListCompared {
    let normal = |name: &str| -> String {
        let name = name.replace('\\', "/");
        let mut name = name.as_str();
        while let Some(rest) = name.strip_prefix("./") {
            name = rest;
        }
        name.trim_start_matches('/').to_owned()
    };
    let names: Vec<String> = listed.iter().map(|l| normal(&l.name)).collect();
    let mut first: HashMap<&str, usize> = HashMap::new();
    for (i, n) in names.iter().enumerate() {
        first.entry(n.as_str()).or_insert(i);
    }
    let mut used: HashSet<&str> = HashSet::new();
    let verdicts = files
        .iter()
        .map(|f| {
            let path = f.path.join("/");
            let line = first
                .get(f.name.as_str())
                .or_else(|| first.get(path.as_str()))
                .copied()
                .or_else(|| {
                    let ends = |long: &str, short: &str| {
                        long.len() > short.len()
                            && long.ends_with(short)
                            && long.as_bytes()[long.len() - short.len() - 1] == b'/'
                    };
                    let mut found = names
                        .iter()
                        .enumerate()
                        .filter(|(_, n)| ends(n, &f.name) || ends(&f.name, n))
                        .map(|(i, _)| i);
                    match (found.next(), found.next()) {
                        (Some(i), None) => Some(i),
                        _ => None,
                    }
                });
            let Some(i) = line else {
                return Verdict::NotListed;
            };
            used.insert(names[i].as_str());
            verdict(f, &listed[i].sum)
        })
        .collect();
    let not_found = listed
        .iter()
        .zip(&names)
        .filter(|(_, n)| !used.contains(n.as_str()))
        .map(|(l, _)| l.name.clone())
        .collect();
    ListCompared {
        verdicts,
        not_found,
    }
}

fn verdict(file: &EntrySums, given: &str) -> Verdict {
    let may_be = algorithms_of(given.len());
    if may_be.is_empty() {
        return Verdict::Unknown(given.len());
    }
    if let Some(a) = may_be.iter().find(|a| file.sum(**a) == Some(given)) {
        return Verdict::Matches(*a);
    }
    let (made, not_made) = may_be.into_iter().partition(|a| file.sum(*a).is_some());
    Verdict::Differs { made, not_made }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn picked(paths: &[&str]) -> Vec<Vec<String>> {
        paths
            .iter()
            .map(|p| p.split('/').map(str::to_owned).collect())
            .collect()
    }

    fn path(p: &str) -> Vec<String> {
        p.split('/').map(str::to_owned).collect()
    }

    #[test]
    fn a_file_is_named_as_its_extraction_puts_it() {
        assert_eq!(name_of(&path("docs/a/b.txt"), &[]), "docs/a/b.txt");
        assert_eq!(
            name_of(&path("docs/a/b.txt"), &picked(&["docs/a/b.txt"])),
            "b.txt"
        );
        assert_eq!(
            name_of(&path("docs/a/b.txt"), &picked(&["docs/a"])),
            "a/b.txt"
        );
        assert_eq!(
            name_of(&path("docs/a/b.txt"), &picked(&["other", "docs"])),
            "docs/a/b.txt"
        );
    }

    fn file(name: &str, sums: &[(Algorithm, &str)]) -> EntrySums {
        EntrySums {
            path: path(&format!("top/{name}")),
            name: name.to_owned(),
            size: 1,
            sums: sums.iter().map(|(a, s)| (*a, (*s).to_owned())).collect(),
        }
    }

    fn line(sum: &str, name: &str) -> Listed {
        Listed {
            sum: sum.to_owned(),
            name: name.to_owned(),
        }
    }

    #[test]
    fn a_list_says_of_each_file_what_it_finds() {
        let (a, b, c) = ("a".repeat(64), "b".repeat(64), "c".repeat(40));
        let files = [
            file("same.txt", &[(Algorithm::Sha256, &a)]),
            file("changed.txt", &[(Algorithm::Sha256, &a)]),
            file("dir/deep.txt", &[(Algorithm::Sha256, &b)]),
            file("sha1.txt", &[(Algorithm::Sha256, &a)]),
            file("odd.txt", &[(Algorithm::Sha256, &a)]),
            file("unlisted.txt", &[(Algorithm::Sha256, &a)]),
        ];
        let listed = [
            line(&a, "same.txt"),
            line(&b, "changed.txt"),
            // Made of the folder it was in, from another place.
            line(&b, "./pkg\\dir/deep.txt"),
            line(&c, "sha1.txt"),
            line("abc", "odd.txt"),
            line(&a, "gone.txt"),
        ];
        let compared = compare_list(&files, &listed);
        assert_eq!(
            compared.verdicts,
            [
                Verdict::Matches(Algorithm::Sha256),
                // It may be BLAKE3's, not made yet.
                Verdict::Differs {
                    made: vec![Algorithm::Sha256],
                    not_made: vec![Algorithm::Blake3]
                },
                Verdict::Matches(Algorithm::Sha256),
                Verdict::Differs {
                    made: vec![],
                    not_made: vec![Algorithm::Sha1]
                },
                Verdict::Unknown(3),
                Verdict::NotListed,
            ]
        );
        assert_eq!(compared.not_found, ["gone.txt"]);
        assert_eq!(compared.matching(), 2);
        assert!(!compared.all_match());
        // Both of its length made, it differs.
        let both = [file(
            "changed.txt",
            &[(Algorithm::Sha256, &a), (Algorithm::Blake3, &a)],
        )];
        assert_eq!(
            compare_list(&both, &[line(&b, "changed.txt")]).verdicts,
            [Verdict::Differs {
                made: vec![Algorithm::Sha256, Algorithm::Blake3],
                not_made: vec![]
            }]
        );
    }

    #[test]
    fn a_name_ending_two_lines_is_neither_of_them() {
        let a = "a".repeat(64);
        let files = [file("b.txt", &[(Algorithm::Sha256, &a)])];
        let listed = [line(&a, "x/b.txt"), line(&a, "y/b.txt")];
        let compared = compare_list(&files, &listed);
        assert_eq!(compared.verdicts, [Verdict::NotListed]);
        assert_eq!(compared.not_found, ["x/b.txt", "y/b.txt"]);
        // Nor does a name end another but at a `/`.
        let compared = compare_list(&files, &[line(&a, "ab.txt")]);
        assert_eq!(compared.verdicts, [Verdict::NotListed]);
    }

    #[test]
    fn a_file_is_found_by_its_path_in_the_archive_too() {
        let a = "a".repeat(64);
        let files = [file("b.txt", &[(Algorithm::Sha256, &a)])];
        let compared = compare_list(&files, &[line(&a, "top/b.txt")]);
        assert!(compared.all_match());
    }

    #[test]
    fn a_list_holds_what_was_made_of_each() {
        let a = "a".repeat(64);
        let files = [
            file("a b.txt", &[(Algorithm::Sha256, &a)]),
            file("none.txt", &[(Algorithm::Blake3, &a)]),
        ];
        assert_eq!(
            checksum_list(&files, Algorithm::Sha256),
            format!("{a}  a b.txt\n")
        );
        assert_eq!(
            checksum_list_file_name("a.zip", Algorithm::Blake3),
            "a.zip.contents.b3"
        );
    }
}
