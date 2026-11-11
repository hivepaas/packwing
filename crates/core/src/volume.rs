//! A UDF volume's tree and a SquashFS image's, walked: what listing them and
//! testing them share.

use std::collections::HashSet;
use std::io::{Read, Seek};

use hivepack::open::Container;
use hivepack::squashfs::{Inode, Squashfs};
use hivepack::udf::{Node, Stat, Udf, VType};

use crate::Error;

/// Calls `each` with every entry of the volume, parents before what they
/// hold, in the order its directories list them (hidden entries left out,
/// as hivepack::extract leaves them): its path, a component at a time, its
/// node and its attributes. A directory met again, which UDF does not
/// allow, is not walked twice, so that a volume made to loop ends.
pub(crate) fn walk<R: Read + Seek>(
    udf: &mut Udf<R>,
    mut each: impl FnMut(&mut Udf<R>, &[String], &Node, &Stat) -> Result<(), Error>,
) -> Result<(), Error> {
    let root = udf.root().clone();
    let at = root.location();
    let mut seen = HashSet::from([(at.part_num, at.lb_num)]);
    // Directories still to list, the next last.
    let mut dirs: Vec<(Node, Vec<String>)> = vec![(root, Vec::new())];
    while let Some((dir, path)) = dirs.pop() {
        let listed = udf.read_dir(&dir)?.collect::<Result<Vec<_>, _>>()?;
        let mut children = Vec::new();
        for entry in listed {
            if entry.is_parent() {
                continue;
            }
            let name = String::from_utf8_lossy(&entry.name).into_owned();
            if name.is_empty() {
                continue;
            }
            let node = udf.node(entry.icb)?;
            let stat = udf.stat(&node);
            let dir = stat.kind == VType::VDIR;
            if dir && !seen.insert((entry.icb.part_num, entry.icb.lb_num)) {
                continue;
            }
            let mut entry_path = path.clone();
            entry_path.push(name);
            each(udf, &entry_path, &node, &stat)?;
            if dir {
                children.push((node, entry_path));
            }
        }
        // Depth first, in the order listed.
        dirs.extend(children.into_iter().rev());
    }
    Ok(())
}

/// The SquashFS image of `container` in `reader`: a snap's or a `.sqfs`
/// file's from its start, a type 2 AppImage's where its program ends.
pub(crate) fn squashfs<R: Read + Seek>(
    container: Container,
    reader: R,
) -> Result<Squashfs<R>, Error> {
    Ok(match container {
        Container::AppImageType2 => hivepack::appimage::open_squashfs(reader)?,
        _ => Squashfs::open(reader)?,
    })
}

/// Calls `each` with every entry of the SquashFS image, parents before what
/// they hold, each directory in its order (by name), as
/// hivepack::extract walks it: its path, a component at a time, its inode,
/// and whether the inode is met for the first time. A file met again is
/// another name of the same one, which extracts as a hard link to the
/// first; a directory met again, which a damaged image can name, is not
/// walked twice, so that an image made to loop ends.
pub(crate) fn walk_squashfs<R: Read + Seek>(
    fs: &mut Squashfs<R>,
    mut each: impl FnMut(&mut Squashfs<R>, &[String], &Inode, bool) -> Result<(), Error>,
) -> Result<(), Error> {
    let root = fs.root()?;
    let mut met = HashSet::from([fs.root_id()]);
    // Directories still to list, the next last.
    let mut dirs: Vec<(Inode, Vec<String>)> = vec![(root, Vec::new())];
    while let Some((dir, path)) = dirs.pop() {
        let listed = fs.read_dir(&dir)?.collect::<Result<Vec<_>, _>>()?;
        let mut children = Vec::new();
        for entry in listed {
            let name = String::from_utf8_lossy(&entry.name).into_owned();
            if name.is_empty() {
                continue;
            }
            let inode = fs.inode(entry.inode)?;
            let first = met.insert(entry.inode);
            if inode.is_dir() && !first {
                continue;
            }
            let mut entry_path = path.clone();
            entry_path.push(name);
            each(fs, &entry_path, &inode, first)?;
            if inode.is_dir() {
                children.push((inode, entry_path));
            }
        }
        // Depth first, in the order listed.
        dirs.extend(children.into_iter().rev());
    }
    Ok(())
}
