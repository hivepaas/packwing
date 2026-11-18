//! Disk images: the block device a container presents, the partitions the
//! table at its start names, and the file systems in them.
//!
//! Every other format packwing opens has one level — the archive, and the
//! entries in it. A disk image has two: partitions, and a file system in
//! each. **So a partition is a folder**, named by its number and what it
//! holds (`1-EFI`, `2-ext4`), and the files of its file system are under
//! it. The number is the one the systems that name partitions use, so
//! `2-ext4` is what `/dev/sda2` was.
//!
//! A partition whose file system packwing does not read comes out as one
//! file of its bytes (`3-swap.img`), so that what is on the disk can still
//! be taken off it. Nothing is invented: an empty folder would say a
//! partition holds nothing, and leaving it out would say the disk has no
//! such partition.
//!
//! **NTFS is read behind `catch_unwind`.** The crate that reads NTFS's
//! structures panics on a damaged directory index, and the fix is in no
//! release of it yet (hivepack's `docs/ntfs.md`), so hivepack leaves the
//! feature off by default and every call into it from here is caught: a
//! damaged image fails its job, as a damaged archive does, and the program
//! stays up. That is the only place in packwing that catches a panic, and
//! it goes when the crate releases the fix.

use std::fs::File;
use std::io::{self, Read};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};

use hivepack::ext::{self, Ext};
use hivepack::fat::{self, Fat};
use hivepack::image::{Disk, Part, Partition, Qcow2, Vhd, Vhdx, Vmdk, partitions};
use hivepack::ntfs::Ntfs;
use hivepack::open::{Container, Layer};

use packwing_i18n::tr;

use crate::Error;
use crate::source::Source;
use crate::spool::{Spooled, spool};

/// The file, and the path to it, that a disk image is read over: the file
/// itself where there are no layers around it, and a temporary copy where
/// there are, as a UDF volume or a SquashFS image is read.
pub(crate) struct Blocks {
    pub(crate) file: File,
    /// Where it is, where that is a file of the system: what a VMDK
    /// descriptor needs, since it names the files beside it.
    pub(crate) path: Option<PathBuf>,
    /// Kept while the disk is read: dropping it takes the copy away.
    _spooled: Option<Spooled>,
}

pub(crate) fn blocks(
    source: &Source,
    opened: &mut impl Read,
    layers: &[Layer],
) -> Result<Blocks, Error> {
    if layers.is_empty()
        && let Some(path) = source.path()
    {
        return Ok(Blocks {
            file: File::open(path)?,
            path: Some(path.to_path_buf()),
            _spooled: None,
        });
    }
    let spooled = spool(opened)?;
    let file = spooled.file.try_clone()?;
    Ok(Blocks {
        file,
        path: None,
        _spooled: Some(spooled),
    })
}

/// The disk a container presents, read from a file.
pub(crate) enum Image {
    /// A raw image: the file is the disk.
    Raw(File),
    Vhd(Vhd<File>),
    Qcow2(Qcow2<File>),
    Vhdx(Vhdx<File>),
    Vmdk(Vmdk<File>),
}

impl Image {
    /// The disk `container` holds, over `file` — or, for a VMDK descriptor,
    /// over the files `path` names beside it.
    pub(crate) fn open(
        container: Container,
        file: File,
        path: Option<&Path>,
    ) -> Result<Image, Error> {
        Ok(match container {
            Container::PartitionedDisk => Image::Raw(file),
            Container::Vhd => Image::Vhd(Vhd::open(file).map_err(said)?),
            Container::Qcow2 => Image::Qcow2(Qcow2::open(file).map_err(said)?),
            Container::Vhdx => Image::Vhdx(Vhdx::open(file).map_err(said)?),
            // A descriptor names the files beside it, which only a path can
            // reach: `Vmdk::beside` reads both kinds, and refuses a name
            // that is not a plain file name in the same folder.
            Container::Vmdk => Image::Vmdk(match path {
                Some(path) => Vmdk::beside(path).map_err(said)?,
                None => Vmdk::open(file).map_err(said)?,
            }),
            _ => return Err(Error::NotAnArchive),
        })
    }

    /// The disk itself, for the partitions and the file systems to be read
    /// over.
    pub(crate) fn disk(&self) -> &dyn Disk {
        match self {
            Image::Raw(file) => file,
            Image::Vhd(disk) => disk,
            Image::Qcow2(disk) => disk,
            Image::Vhdx(disk) => disk,
            Image::Vmdk(disk) => disk,
        }
    }
}

/// What hivepack said, as packwing carries a reader's refusal.
fn said<E: std::fmt::Display>(e: E) -> Error {
    Error::Failed(e.to_string())
}

/// A file system in a partition, opened over it.
pub(crate) enum Holds<'d> {
    Fat(Box<Fat<Part<&'d dyn Disk>>>, fat::Kind),
    Ext(Box<Ext<Part<&'d dyn Disk>>>, ext::Kind),
    /// Every call into this one is caught: see the module's own note.
    Ntfs(Box<Ntfs<Part<&'d dyn Disk>>>),
    /// None that packwing reads, so the partition's bytes are the file.
    Bytes,
}

impl Holds<'_> {
    /// What it is called in a partition's name: the file system, in the
    /// word a system calls it by.
    fn word(&self) -> Option<&'static str> {
        Some(match self {
            Holds::Fat(_, fat::Kind::Fat12) => "fat12",
            Holds::Fat(_, fat::Kind::Fat16) => "fat16",
            Holds::Fat(_, fat::Kind::Fat32) => "fat32",
            Holds::Ext(_, ext::Kind::Ext2) => "ext2",
            Holds::Ext(_, ext::Kind::Ext3) => "ext3",
            Holds::Ext(_, ext::Kind::Ext4) => "ext4",
            Holds::Ntfs(_) => "ntfs",
            Holds::Bytes => return None,
        })
    }
}

/// What is in `partition` of `disk`, as far as packwing reads it: each
/// reader is asked in turn, and each turns away what is not its own by the
/// structures at the partition's start.
///
/// A reader refusing is not an error here. A partition holds what it holds,
/// and a disk is not made of archives: one packwing cannot read is a file
/// of its bytes, not a failure of the image.
pub(crate) fn holds<'d>(disk: &'d dyn Disk, partition: &Partition) -> Holds<'d> {
    holds_at(disk, partition.at, partition.size)
}

/// What is in the `size` bytes at `at` of `disk`, as [`holds`] asks it of a
/// partition: the whole disk, where no table names any partition.
pub(crate) fn holds_at(disk: &dyn Disk, at: u64, size: u64) -> Holds<'_> {
    let part = || Part::new(disk, at, size);
    if let Ok(fs) = Fat::open(part()) {
        let kind = fs.kind();
        return Holds::Fat(Box::new(fs), kind);
    }
    if let Ok(fs) = Ext::open(part()) {
        let kind = fs.kind();
        return Holds::Ext(Box::new(fs), kind);
    }
    // Opening reads the boot sector and nothing else, and hivepack checks
    // the whole of it before the library sees the disk, so this cannot
    // reach the panic — but it is caught all the same, because what cannot
    // panic today is a thing to be measured, not assumed.
    if let Ok(Ok(fs)) = catch_unwind(AssertUnwindSafe(|| Ntfs::open(part()))) {
        return Holds::Ntfs(Box::new(fs));
    }
    Holds::Bytes
}

/// What a partition is called in a listing: its number, and what it is for
/// or what it holds — `1-EFI`, `2-ext4`, `3-swap`, `4-fat32`.
///
/// **The table's word wins where it says what the partition is for**, as an
/// EFI system partition holds FAT and is not called `fat32` by anyone, and
/// swap holds no file system at all. Otherwise it is the file system, and
/// where packwing reads none, what the table says the partition holds.
pub(crate) fn name_of(partition: &Partition, holds: &Holds<'_>) -> String {
    let number = partition.number;
    match (purpose(partition), holds.word(), kind_word(partition)) {
        (Some(word), _, _) => format!("{number}-{word}"),
        (None, Some(word), _) => format!("{number}-{word}"),
        (None, None, Some(word)) => format!("{number}-{word}"),
        (None, None, None) => format!("partition{number}"),
    }
}

/// A GUID as it is written, from the bytes as they are on the disk: the
/// first three fields are little-endian there, which is why these are not
/// compared as bytes.
fn guid_text(guid: &[u8; 16]) -> String {
    let field = |at: usize, len: usize, flip: bool| {
        let bytes: Vec<u8> = if flip {
            guid[at..at + len].iter().rev().copied().collect()
        } else {
            guid[at..at + len].to_vec()
        };
        bytes.iter().map(|b| format!("{b:02X}")).collect::<String>()
    };
    format!(
        "{}-{}-{}-{}-{}",
        field(0, 4, true),
        field(4, 2, true),
        field(6, 2, true),
        field(8, 2, false),
        field(10, 6, false),
    )
}

/// What the table says the partition is **for**, where that is more than
/// the file system in it: an EFI system partition, swap, and the places a
/// system keeps for itself. None where the file system is the answer.
fn purpose(partition: &Partition) -> Option<&'static str> {
    use hivepack::image::Kind;
    match partition.kind {
        Kind::Type(0xEF) => Some("EFI"),
        Kind::Type(0x82) => Some("swap"),
        Kind::Type(0x05 | 0x0F) => Some("extended"),
        Kind::Type(_) => None,
        Kind::Guid(guid) => match guid_text(&guid).as_str() {
            "C12A7328-F81F-11D2-BA4B-00A0C93EC93B" => Some("EFI"),
            "0657FD6D-A4AB-43C4-84E5-0933C84B4F4F" => Some("swap"),
            "21686148-6449-6E6F-744E-656564454649" => Some("bios-boot"),
            "E3C9E316-0B5C-4DB8-817D-F92DF00215AE" => Some("reserved"),
            "DE94BBA4-06D1-4D40-A16A-BFD50179D6AC" => Some("recovery"),
            "A19D880F-05FC-4D3B-A006-743F0F84911E" => Some("raid"),
            "E6D6D379-F507-44C2-A23C-238F2A3DF928" => Some("lvm"),
            "CA7D7CCB-63ED-4C53-861C-1742536059CC" => Some("luks"),
            _ => None,
        },
    }
}

/// What the table says the partition holds, for one whose file system
/// packwing does not read: a word, where the type says one.
fn kind_word(partition: &Partition) -> Option<&'static str> {
    use hivepack::image::Kind;
    match partition.kind {
        Kind::Type(0x01 | 0x04 | 0x06 | 0x0E) => Some("fat"),
        Kind::Type(0x0B | 0x0C) => Some("fat32"),
        Kind::Type(0x07) => Some("ntfs"),
        Kind::Type(0x83) => Some("linux"),
        Kind::Type(0xEE) => Some("gpt"),
        Kind::Type(_) => None,
        Kind::Guid(guid) => match guid_text(&guid).as_str() {
            "0FC63DAF-8483-4772-8E79-3D69D8477DE4" => Some("linux"),
            "EBD0A0A2-B9E5-4433-87C0-68B6B72699C7" => Some("windows"),
            "933AC7E1-2EB4-4F13-B844-0E14E2AEF915" => Some("home"),
            "48465300-0000-11AA-AA11-00306543ECAC" => Some("hfs"),
            "7C3457EF-0000-11AA-AA11-00306543ECAC" => Some("apfs"),
            _ => None,
        },
    }
}

/// The partitions of `disk`, or none where no table names any: a disk whose
/// whole length is one file system, as WSL2's `ext4.vhdx` is.
pub(crate) fn table_of(disk: &dyn Disk) -> Result<Vec<Partition>, Error> {
    match partitions(&disk).map_err(said)? {
        Some(found) => Ok(found.partitions),
        None => Ok(Vec::new()),
    }
}

/// A `Disk`'s bytes as a reader, from its start: what writes a partition
/// packwing reads no file system in.
pub(crate) struct Reading<D> {
    disk: D,
    at: u64,
}

impl<D: Disk> Reading<D> {
    pub(crate) fn new(disk: D) -> Reading<D> {
        Reading { disk, at: 0 }
    }
}

impl<D: Disk> Read for Reading<D> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let read = self.disk.read_at(self.at, buf)?;
        self.at += read as u64;
        Ok(read)
    }
}

/// Reads an NTFS file system behind `catch_unwind`: the panic the released
/// crate answers a damaged directory index with becomes an error, as a
/// damaged archive's is.
pub(crate) fn caught<T>(what: impl FnOnce() -> Result<T, Error>) -> Result<T, Error> {
    match catch_unwind(AssertUnwindSafe(what)) {
        Ok(done) => done,
        Err(_) => Err(Error::Failed(tr!(
            "This NTFS file system is damaged in a way the library that reads it cannot report."
        ))),
    }
}

/// What [`each_file`] gives a file to: the path to it (with `/` between the
/// names), how long it is, and a reader over its data.
pub(crate) type EachFile<'a> = dyn FnMut(&str, u64, &mut dyn Read) -> Result<(), Error> + 'a;

/// Every file of a file system, in the order its directories list them:
/// `each` is given the path to it (with `/` between the names), how long it
/// is, and a reader over its data, which it may read or leave.
///
/// A file whose data a reader will not give is passed over rather than
/// failing the whole image: NTFS compresses files and encrypts them, and
/// neither is read (hivepack's `docs/ntfs.md`). Which ones those were is in
/// the listing, where each shows the size it says it has.
pub(crate) fn each_file(holds: &Holds<'_>, each: &mut EachFile<'_>) -> Result<(), Error> {
    match holds {
        Holds::Fat(fs, _) => {
            let mut walk = fs.walk().map_err(said)?;
            while let Some((path, entry)) = walk.next(fs).map_err(said)? {
                if entry.is_dir() {
                    continue;
                }
                let mut file = fs.file(&entry).map_err(said)?;
                each(&path, entry.size, &mut file)?;
            }
        }
        Holds::Ext(fs, _) => {
            let mut walk = fs.walk().map_err(said)?;
            while let Some(step) = walk.next(fs).map_err(said)? {
                if !step.node.is_file() {
                    continue;
                }
                let mut file = fs.file(&step.node).map_err(said)?;
                each(
                    &String::from_utf8_lossy(&step.path),
                    step.node.size,
                    &mut file,
                )?;
            }
        }
        Holds::Ntfs(fs) => caught(|| {
            let mut walk = fs.walk().map_err(said)?;
            while let Some((path, entry)) = walk.next(fs).map_err(said)? {
                // Compressed, encrypted and reparse points are not read:
                // the entry says so without going to the disk.
                if !entry.is_file() || entry.readable().is_err() {
                    continue;
                }
                let mut file = fs.file(&entry).map_err(said)?;
                each(&path, entry.size, &mut file)?;
            }
            Ok(())
        })?,
        Holds::Bytes => {}
    }
    Ok(())
}

/// A partition's bytes, as a reader over the disk: what a partition
/// packwing reads no file system in is written from.
pub(crate) fn bytes_of(disk: &dyn Disk, at: u64, size: u64) -> Reading<Part<&dyn Disk>> {
    Reading::new(Part::new(disk, at, size))
}

#[cfg(test)]
mod tests {
    use super::*;
    use hivepack::image::Kind;

    fn partition(number: u32, kind: Kind) -> Partition {
        Partition {
            number,
            at: 1 << 20,
            size: 1 << 20,
            kind,
            name: None,
            id: None,
            bootable: false,
        }
    }

    /// A GUID is written from the bytes on the disk, where its first three
    /// fields are little-endian: the EFI system partition's type, which
    /// every table on every disk with one carries.
    #[test]
    fn a_guid_is_written_as_the_tools_write_it() {
        let efi = [
            0x28, 0x73, 0x2A, 0xC1, 0x1F, 0xF8, 0xD2, 0x11, 0xBA, 0x4B, 0x00, 0xA0, 0xC9, 0x3E,
            0xC9, 0x3B,
        ];
        assert_eq!(guid_text(&efi), "C12A7328-F81F-11D2-BA4B-00A0C93EC93B");
    }

    /// What a partition is called: the table's word where it says what the
    /// partition is **for**, the file system where it does not, and the
    /// number alone where nothing says anything.
    #[test]
    fn a_partition_is_named_by_its_number_and_what_it_holds() {
        let efi = Kind::Guid([
            0x28, 0x73, 0x2A, 0xC1, 0x1F, 0xF8, 0xD2, 0x11, 0xBA, 0x4B, 0x00, 0xA0, 0xC9, 0x3E,
            0xC9, 0x3B,
        ]);
        let linux = Kind::Guid([
            0xAF, 0x3D, 0xC6, 0x0F, 0x83, 0x84, 0x72, 0x47, 0x8E, 0x79, 0x3D, 0x69, 0xD8, 0x47,
            0x7D, 0xE4,
        ]);
        // An EFI system partition holds FAT, and nobody calls it `fat32`.
        assert_eq!(
            name_of(&partition(1, efi), &Holds::Bytes),
            "1-EFI".to_owned()
        );
        // A Linux partition is named by the file system in it, and by what
        // the table says where packwing reads none.
        assert_eq!(
            name_of(&partition(2, linux), &Holds::Bytes),
            "2-linux".to_owned()
        );
        // A type no table word covers, and no file system read: the number.
        assert_eq!(
            name_of(&partition(5, Kind::Type(0x42)), &Holds::Bytes),
            "partition5".to_owned()
        );
        assert_eq!(
            name_of(&partition(3, Kind::Type(0x82)), &Holds::Bytes),
            "3-swap".to_owned()
        );
    }

    /// The guard around the library that reads NTFS: a panic inside it is
    /// an error, and the program goes on. This is what makes a damaged
    /// image a job that failed rather than one that took packwing with it.
    #[test]
    fn a_panic_in_the_ntfs_library_becomes_an_error() {
        let quiet = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let panicked: Result<(), Error> =
            caught(|| panic!("as ntfs 0.4.0 does on a damaged index"));
        std::panic::set_hook(quiet);
        assert!(matches!(panicked, Err(Error::Failed(_))), "{panicked:?}");
        // And what does not panic comes back as it is.
        assert!(caught(|| Ok::<u32, Error>(7)).is_ok_and(|n| n == 7));
    }
}
