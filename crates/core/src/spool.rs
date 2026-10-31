//! An archive under layers that is read by random access, taken out to a
//! temporary file: a ZIP archive, read from its end, or a UDF volume, read
//! where its descriptors point, which a stream cannot give.

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

/// A file in the system's temporary folder, readable by its owner only,
/// removed when dropped.
pub(crate) struct Spooled {
    path: PathBuf,
    pub(crate) file: File,
}

impl Spooled {
    /// The file, from its start: for a reader that is opened more than once.
    pub(crate) fn rewound(&self) -> io::Result<&File> {
        let mut file = &self.file;
        file.seek(SeekFrom::Start(0))?;
        Ok(file)
    }
}

impl Drop for Spooled {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// What `from` reads, in a new temporary file.
pub(crate) fn spool(from: &mut impl Read) -> io::Result<Spooled> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let mut options = OpenOptions::new();
    options.read(true).write(true).create_new(true);
    // What was encrypted is not left for others to read.
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let path = std::env::temp_dir().join(format!(
        "packwing-{}-{}.spool",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let file = options.open(&path)?;
    let mut spooled = Spooled { path, file };
    io::copy(from, &mut spooled.file)?;
    spooled.file.seek(SeekFrom::Start(0))?;
    Ok(spooled)
}
