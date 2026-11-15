//! An archive changed: files added to it, deleted from it, renamed in it, as
//! the desktop changes one. Only a ZIP or 7z archive of one file, under no
//! layers, can be ([`Archive::can_change`]).
//!
//! Each change writes the archive again beside it and puts it in its place,
//! so a change that fails or is stopped leaves it as it was. A phone does not
//! always let the app write beside the file it opened (iOS gives an app the
//! file chosen, not its folder): the app then changes a copy of it, at the
//! path it gives, and writes that over the file. So every change takes the
//! path of the file to change, the archive's own or its copy's.

use std::path::PathBuf;
use std::sync::Arc;

use packwing_core::{AddOptions, can_add};

use crate::{Archive, Job, MobileError};

/// What becomes of a file added whose name the folder holds already.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, uniffi::Enum)]
pub enum OnExisting {
    /// Nothing is changed, and it says which.
    #[default]
    Stop,
    Replace,
    /// The one in the archive stays.
    Skip,
}

impl From<OnExisting> for packwing_core::OnExisting {
    fn from(on: OnExisting) -> packwing_core::OnExisting {
        match on {
            OnExisting::Stop => packwing_core::OnExisting::Stop,
            OnExisting::Replace => packwing_core::OnExisting::Replace,
            OnExisting::Skip => packwing_core::OnExisting::Skip,
        }
    }
}

#[uniffi::export]
impl Archive {
    /// Whether it can be changed: a ZIP or 7z archive, of one file, under no
    /// layers. Another kind would be written whole again, or has no place to
    /// add to.
    /// One on a storage, or one Android gave, is not changed here.
    pub fn can_change(&self) -> bool {
        matches!(self.source, packwing_core::Source::File(_)) && can_add(&self.listing)
    }

    /// It read again, with the secrets it was opened with: what it holds, once
    /// it is changed.
    pub fn reopen(&self, job: Arc<Job>) -> Result<Arc<Archive>, MobileError> {
        Archive::of(self.source.clone(), self.secrets.clone(), job)
    }

    /// Deletes `picked` (paths in it), each with what is under it, from the
    /// archive at `file`. All of it cannot go: an archive of nothing is not
    /// one.
    pub fn delete(
        &self,
        file: String,
        picked: Vec<Vec<String>>,
        job: Arc<Job>,
    ) -> Result<(), MobileError> {
        Ok(packwing_core::delete(
            &PathBuf::from(file),
            &picked,
            &self.secrets.keys(),
            &job.control(),
        )?)
    }

    /// Renames what is at `from` to `to`, in the same folder of the archive at
    /// `file`. A name there already stops it.
    pub fn rename(
        &self,
        file: String,
        from: Vec<String>,
        to: String,
        job: Arc<Job>,
    ) -> Result<(), MobileError> {
        Ok(packwing_core::rename(
            &PathBuf::from(file),
            &from,
            &to,
            &self.secrets.keys(),
            &job.control(),
        )?)
    }

    /// Adds `paths` (files and folders, each with what is under it) to the
    /// archive at `file`, in its folder `folder`. In an archive whose files
    /// are encrypted, they are encrypted too, with its password.
    pub fn add(
        &self,
        file: String,
        paths: Vec<String>,
        folder: Vec<String>,
        on_existing: OnExisting,
        job: Arc<Job>,
    ) -> Result<(), MobileError> {
        let paths: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
        let how = AddOptions {
            on_existing: on_existing.into(),
            ..AddOptions::default()
        };
        Ok(packwing_core::add_with(
            &paths,
            &PathBuf::from(file),
            &folder,
            &how,
            &self.secrets.keys(),
            &job.control(),
        )?)
    }
}
