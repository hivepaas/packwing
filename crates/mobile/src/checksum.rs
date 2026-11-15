//! An archive's own checksums, to compare with the one given where it came
//! from, and what a checksum pasted says of them; and the checksums of the
//! files in it, all or those picked, in one read of it, listed as `sha256sum`
//! writes them: the desktop's Checksums…, for both phones. A checksum says a file is the one it was made of, not who
//! made it: only a signature says that.

use std::sync::Arc;

use packwing_core::{
    Algorithm, EntrySums, Listed, Verdict, algorithms_of, checksum_in, checksum_list_file_name,
    checksum_list_in, compare_list, entry_checksums, source_checksums,
};

use crate::{Archive, Job, MobileError};

/// A way to make a checksum.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, uniffi::Enum)]
pub enum ChecksumKind {
    Sha256,
    Sha512,
    Sha1,
    Md5,
    Blake3,
}

impl From<ChecksumKind> for Algorithm {
    fn from(kind: ChecksumKind) -> Algorithm {
        match kind {
            ChecksumKind::Sha256 => Algorithm::Sha256,
            ChecksumKind::Sha512 => Algorithm::Sha512,
            ChecksumKind::Sha1 => Algorithm::Sha1,
            ChecksumKind::Md5 => Algorithm::Md5,
            ChecksumKind::Blake3 => Algorithm::Blake3,
        }
    }
}

impl From<Algorithm> for ChecksumKind {
    fn from(algorithm: Algorithm) -> ChecksumKind {
        match algorithm {
            Algorithm::Sha256 => ChecksumKind::Sha256,
            Algorithm::Sha512 => ChecksumKind::Sha512,
            Algorithm::Sha1 => ChecksumKind::Sha1,
            Algorithm::Md5 => ChecksumKind::Md5,
            Algorithm::Blake3 => ChecksumKind::Blake3,
        }
    }
}

/// A way to make a checksum, as the app shows it.
#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct ChecksumKindRow {
    pub kind: ChecksumKind,
    /// What it is called, as anyone writes it ("SHA-256"): not to translate.
    pub name: String,
    /// What the app keeps it as in its settings ("sha256").
    pub key: String,
    /// Two files can be made to share it: it still tells a damaged file, not
    /// one changed on purpose.
    pub weak: bool,
}

/// Every way, in the order they are shown.
#[uniffi::export]
pub fn checksum_kinds() -> Vec<ChecksumKindRow> {
    Algorithm::ALL
        .into_iter()
        .map(|a| ChecksumKindRow {
            kind: a.into(),
            name: a.name().to_owned(),
            key: a.key().to_owned(),
            weak: a.is_weak(),
        })
        .collect()
}

/// A checksum made.
#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct Checksum {
    pub kind: ChecksumKind,
    /// In lowercase hex.
    pub sum: String,
}

/// What a checksum pasted says of the file.
#[derive(Clone, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum Compared {
    /// Nothing in it is a checksum.
    NotOne,
    /// No way packwing has makes one `digits` long.
    Unknown {
        digits: u32,
    },
    Matches {
        kind: ChecksumKind,
    },
    /// It may be one of those being made: it is compared once they are.
    Waiting,
    /// It is none of `made`, and may be one of `not_made`, not made yet.
    Differs {
        made: Vec<ChecksumKind>,
        not_made: Vec<ChecksumKind>,
    },
}

/// What `pasted`, the checksum given with the file called `name`, says of
/// it, beside those `made` of it and those being made (`making`). It is found
/// in what was pasted however it was written: alone, in capitals or groups,
/// as `sha256sum` and BSD's write it, or as the line for `name` of a
/// `SHA256SUMS`. Nothing for nothing pasted.
#[uniffi::export]
pub fn compare_checksum(
    pasted: String,
    name: String,
    made: Vec<Checksum>,
    making: Vec<ChecksumKind>,
) -> Option<Compared> {
    if pasted.trim().is_empty() {
        return None;
    }
    let Some(given) = checksum_in(&pasted, &name) else {
        return Some(Compared::NotOne);
    };
    let may_be: Vec<ChecksumKind> = algorithms_of(given.len())
        .into_iter()
        .map(ChecksumKind::from)
        .collect();
    if may_be.is_empty() {
        return Some(Compared::Unknown {
            digits: u32::try_from(given.len()).unwrap_or(u32::MAX),
        });
    }
    if let Some(found) = made
        .iter()
        .find(|m| may_be.contains(&m.kind) && m.sum == given)
    {
        return Some(Compared::Matches { kind: found.kind });
    }
    if may_be.iter().any(|k| making.contains(k)) {
        return Some(Compared::Waiting);
    }
    let (made, not_made) = may_be
        .into_iter()
        .partition(|k| made.iter().any(|m| m.kind == *k));
    Some(Compared::Differs { made, not_made })
}

#[uniffi::export]
impl Archive {
    /// The `kinds` of checksums of the file it was opened from, all of it read
    /// once for them; `job` is told the bytes read.
    pub fn checksums(
        &self,
        kinds: Vec<ChecksumKind>,
        job: Arc<Job>,
    ) -> Result<Vec<Checksum>, MobileError> {
        let algorithms: Vec<Algorithm> = kinds.iter().map(|k| Algorithm::from(*k)).collect();
        let sums = source_checksums(&self.source, &algorithms, &job.control())?;
        Ok(sums
            .into_iter()
            .map(|(a, sum)| Checksum {
                kind: a.into(),
                sum,
            })
            .collect())
    }
}

/// A file of an archive, and its checksums.
#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct EntryChecksum {
    /// Its path in the archive.
    pub path: Vec<String>,
    /// What a list of checksums names it: its path from the folder it is
    /// extracted into (of a file picked, its name alone).
    pub name: String,
    pub size: u64,
    /// In the order they were asked for.
    pub sums: Vec<Checksum>,
}

/// What reading an archive's files for their checksums made.
#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct EntryChecksums {
    /// The files, by name.
    pub files: Vec<EntryChecksum>,
    /// The symbolic links met, which have none: a checking tool reads what
    /// they lead to.
    pub links: u64,
    /// The paths of the files whose data is damaged, which have none.
    pub damaged: Vec<String>,
}

fn entry_sums(f: &EntryChecksum) -> EntrySums {
    EntrySums {
        path: f.path.clone(),
        name: f.name.clone(),
        size: f.size,
        sums: f
            .sums
            .iter()
            .map(|c| (Algorithm::from(c.kind), c.sum.clone()))
            .collect(),
    }
}

/// The list of the `kind` checksums of `files`, as `sha256sum` writes one and
/// `sha256sum -c` checks it, where they are extracted.
#[uniffi::export]
pub fn checksum_list(files: Vec<EntryChecksum>, kind: ChecksumKind) -> String {
    let files: Vec<EntrySums> = files.iter().map(entry_sums).collect();
    packwing_core::checksum_list(&files, kind.into())
}

/// The name of the file beside the archive `name` that keeps that list:
/// `name.contents.sha256`.
#[uniffi::export]
pub fn checksum_list_name(name: String, kind: ChecksumKind) -> String {
    checksum_list_file_name(&name, kind.into())
}

#[uniffi::export]
impl Archive {
    /// The `kinds` of checksums of its files: all of them, or those `picked`
    /// (paths in it, each with what is under it), the archive read once for
    /// them and nothing written. A damaged file is left out, and said. `job`
    /// is told how far it is.
    pub fn entry_checksums(
        &self,
        picked: Vec<Vec<String>>,
        kinds: Vec<ChecksumKind>,
        job: Arc<Job>,
    ) -> Result<EntryChecksums, MobileError> {
        let algorithms: Vec<Algorithm> = kinds.iter().map(|k| Algorithm::from(*k)).collect();
        let made = entry_checksums(
            self.source.clone(),
            &picked,
            &self.secrets.keys(),
            &algorithms,
            &job.control(),
        )?;
        Ok(EntryChecksums {
            files: made
                .files
                .into_iter()
                .map(|f| EntryChecksum {
                    path: f.path,
                    name: f.name,
                    size: f.size,
                    sums: f
                        .sums
                        .into_iter()
                        .map(|(a, sum)| Checksum {
                            kind: a.into(),
                            sum,
                        })
                        .collect(),
                })
                .collect(),
            links: made.links,
            damaged: made.damaged.into_iter().map(|d| d.path).collect(),
        })
    }
}

/// What a list of checksums says of a file of an archive.
#[derive(Clone, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum FileVerdict {
    Matches {
        kind: ChecksumKind,
    },
    /// The one listed is none of `made`; it may be one of `not_made`, not made yet.
    Differs {
        made: Vec<ChecksumKind>,
        not_made: Vec<ChecksumKind>,
    },
    /// The one listed is no checksum packwing makes: it has this many digits.
    Unknown {
        digits: u32,
    },
    /// The list does not name it.
    NotListed,
}

impl From<Verdict> for FileVerdict {
    fn from(v: Verdict) -> FileVerdict {
        let kinds = |a: Vec<Algorithm>| a.into_iter().map(ChecksumKind::from).collect();
        match v {
            Verdict::Matches(a) => FileVerdict::Matches { kind: a.into() },
            Verdict::Differs { made, not_made } => FileVerdict::Differs {
                made: kinds(made),
                not_made: kinds(not_made),
            },
            Verdict::Unknown(digits) => FileVerdict::Unknown {
                digits: u32::try_from(digits).unwrap_or(u32::MAX),
            },
            Verdict::NotListed => FileVerdict::NotListed,
        }
    }
}

/// A file of those made, found by a checksum alone.
#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct FoundBySum {
    /// Where it is in the files made.
    pub index: u32,
    pub kind: ChecksumKind,
}

/// What checksums pasted say of an archive's files.
#[derive(Clone, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum PastedCompared {
    /// Nothing in it is a checksum.
    NotOne,
    /// A list: what it says of each file, in their order, and the names it has
    /// that none of them is.
    Listed {
        verdicts: Vec<FileVerdict>,
        not_found: Vec<String>,
    },
    /// One checksum alone, naming no file, `digits` long: the files it is of,
    /// and the ways it may be made that were not.
    One {
        digits: u32,
        found: Vec<FoundBySum>,
        not_made: Vec<ChecksumKind>,
    },
}

/// What `pasted` (a list as `sha256sum` and the others write one, or one
/// checksum alone) says of `files`, whose checksums of `made` were made: as
/// the desktop's Compare… of an archive's files says it. Nothing for nothing
/// pasted.
#[uniffi::export]
pub fn compare_pasted(
    pasted: String,
    files: Vec<EntryChecksum>,
    made: Vec<ChecksumKind>,
) -> Option<PastedCompared> {
    if pasted.trim().is_empty() {
        return None;
    }
    let sums: Vec<EntrySums> = files.iter().map(entry_sums).collect();
    let mut listed = checksum_list_in(&pasted);
    // A checksum alone, of the one file there is.
    if listed.is_empty()
        && let [one] = sums.as_slice()
        && let Some(sum) = checksum_in(&pasted, &one.name)
    {
        listed.push(Listed {
            sum,
            name: one.name.clone(),
        });
    }
    if !listed.is_empty() {
        let compared = compare_list(&sums, &listed);
        return Some(PastedCompared::Listed {
            verdicts: compared
                .verdicts
                .into_iter()
                .map(FileVerdict::from)
                .collect(),
            not_found: compared.not_found,
        });
    }
    let Some(sum) = checksum_in(&pasted, "") else {
        return Some(PastedCompared::NotOne);
    };
    // One checksum alone, of whichever file has it.
    let may_be = algorithms_of(sum.len());
    let found = sums
        .iter()
        .enumerate()
        .filter_map(|(i, f)| {
            may_be
                .iter()
                .find(|a| f.sum(**a) == Some(sum.as_str()))
                .map(|a| FoundBySum {
                    index: u32::try_from(i).unwrap_or(u32::MAX),
                    kind: (*a).into(),
                })
        })
        .collect();
    let made: Vec<Algorithm> = made.into_iter().map(Algorithm::from).collect();
    Some(PastedCompared::One {
        digits: u32::try_from(sum.len()).unwrap_or(u32::MAX),
        found,
        not_made: may_be
            .into_iter()
            .filter(|a| !made.contains(a))
            .map(ChecksumKind::from)
            .collect(),
    })
}
