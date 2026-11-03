# Additional permission under GNU GPL version 3 section 7

packwing reads RAR archives through `hivepack::rar`, the `rar` module of the
hivepack crate (before hivepack 0.1.0, the crate `hivepack-rar`), a port of
RARLAB's UnRAR to Rust, which is under UnRAR's own license rather than one
compatible with the GPL. The rest of hivepack is not under UnRAR's license.
So that packwing can be distributed with it:

If you modify this Program, or any covered work, by linking or combining it with
`hivepack::rar` or `hivepack-rar` (or a modified version of that module or
library), containing parts covered by the terms of the UnRAR license, the
licensors of this Program grant you additional permission to convey the resulting
work. Corresponding Source for a non-source form of such a combination shall
include the source code for the parts of `hivepack::rar` or `hivepack-rar` used as
well as that of the covered work.

This permission covers `hivepack::rar` and `hivepack-rar` alone. You may remove it
from your own versions of packwing, as section 7 of the GPL says, in which case
they cannot include them: hivepack without its `rar` feature leaves the module out.

## What UnRAR's license asks

It applies to `hivepack::rar`, not to the rest of hivepack or of packwing, and its
full text is in that module's `LICENSE` (`src/rar/LICENSE` in the hivepack crate)
and in the third-party licenses packwing's packages hold. Its
second paragraph says:

> UnRAR source code may be used in any software to handle RAR archives without
> limitations free of charge, but cannot be used to develop RAR (WinRAR) compatible
> archiver and to re-create RAR compression algorithm, which is proprietary.
> Distribution of modified UnRAR source code in separate form or as a part of other
> software is permitted, provided that full text of this paragraph, starting from
> "UnRAR source code" words, is included in license, or in documentation if license
> is not available, and in source code comments of resulting package.

packwing reads RAR and does not write it. All copyrights to RAR and to UnRAR are
Alexander Roshal's.
