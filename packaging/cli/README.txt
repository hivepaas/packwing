pkw: packwing on the command line
=================================

pkw opens, extracts, checks and makes archives with no window, for a
terminal, a server over SSH or a script. It reads ZIP, 7z, RAR, tar and its
compressed kinds, ISO and UDF images, cpio, RPM, XAR and age, and makes ZIP,
7z and tar ones. Extraction never writes outside where it is told.

    pkw list backup.7z
    pkw extract backup.7z
    pkw create site.tar.zst site/
    pkw --help

It is the same pkw that packwing's packages and installers hold; this is it
alone, for where the app is not wanted. It does not update itself: a newer
one is at https://github.com/hivepaas/packwing/releases


Installing
----------

Linux and macOS: put pkw in a folder on your PATH, such as ~/.local/bin:

    install -m 755 pkw ~/.local/bin/pkw

Its manual pages (man/) go in a folder man reads, such as
~/.local/share/man/man1, and its completions (completions/) where your
shell looks:

    bash   completions/pkw.bash  ->  ~/.local/share/bash-completion/completions/pkw
    zsh    completions/_pkw      ->  a folder in $fpath
    fish   completions/pkw.fish  ->  ~/.config/fish/completions/pkw.fish

On macOS pkw is signed with a Developer ID and notarized.

Windows: put pkw.exe in a folder on your PATH. For PowerShell's
completions, add this to $PROFILE:

    pkw completions powershell | Out-String | Invoke-Expression


Licenses
--------

pkw is free software under the GNU General Public License, version 3 or
later (LICENSE), with the additional permission in
GPL-ADDITIONAL-PERMISSION.md. THIRD-PARTY-LICENSES holds the licenses of the
Rust crates it is built with.
