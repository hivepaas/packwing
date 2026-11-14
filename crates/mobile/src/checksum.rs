//! An archive's own checksums, to compare with the one given where it came
//! from, and what a checksum pasted says of them: the desktop's Checksums…,
//! for both phones. A checksum says a file is the one it was made of, not who
//! made it: only a signature says that.

use std::sync::Arc;

use packwing_core::{Algorithm, algorithms_of, checksum_in, source_checksums};

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
