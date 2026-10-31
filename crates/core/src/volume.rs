//! A UDF volume's tree, walked: what listing it and testing it share.

use std::collections::HashSet;
use std::io::{Read, Seek};

use hivepack_udf::{Node, Stat, Udf, VType};

use crate::Error;

/// Calls `each` with every entry of the volume, parents before what they
/// hold, in the order its directories list them (hidden entries left out,
/// as hivepack-extract leaves them): its path, a component at a time, its
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
