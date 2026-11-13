# pkw: the command line

`pkw` is packwing without the window, for a terminal, a server over SSH or a script. It is a program of its own (`make cli`, then `target/release/pkw`), with no window system in it; `make cli-slim` builds it without S3, SFTP and WebDAV, about a third of the size, for archives on this computer and the app's folder storages:

```sh
pkw list backup.7z                      # what it holds (-l: sizes and dates; --json)
pkw info backup.tar.zst.age             # its format, layers, size and encryption
pkw extract backup.7z                   # into backup/, here (-o DIR, --here, --overwrite, --no-mark)
pkw extract backup.7z docs/report.pdf   # what is picked, straight into where it goes
pkw extract --on-damaged skip old.zip   # past files whose data is damaged: ask, stop, skip, keep (--report FILE)
pkw extract --on-bad-name rename a.zip  # names this system can't use: ask, stop, skip, rename (--portable-names: every system's)
pkw extract --links-outside create backup.tar.zst  # links that lead outside the folder: ask, stop, skip, create
pkw extract --same-owner --xattrs home.tar.zst  # a backup's owners and extended attributes back (--numeric-owner: by number)
pkw test backup.7z                      # it reads whole, as it was made
pkw cat backup.zip notes.txt            # a file in it, to standard output
pkw create site.tar.zst site/           # its format by its extension, or -f; -l store…highest; no .DS_Store (--keep-mac-files)
pkw create -p --hide-names private.7z docs/      # with a password, asked; its names hidden
pkw create notes.tar.zst docs/ --encrypt-to age1…  # encrypted to a key: notes.tar.zst.age
pkw create --split 25MB mail.zip photos/          # in parts: mail.zip.001, .002…
pkw create --frames one backup.tar.zst home/      # zstd in one frame, the smallest; several open on every core
pkw create --blocks small home.tar.xz home/       # xz or 7z in blocks for every core: faster, a little larger
pkw create --owners --xattrs home.tar.zst ~/      # for a backup: owners, extended attributes and ACLs
pkw create --mtime 2026-01-01 src.tar.gz src/     # one time for every file; SOURCE_DATE_EPOCH caps them
pkw create app.zip app/ --exclude node_modules/ --exclude '*.log'   # --include too: a / at the end names folders, `src/**/gen/` any depth
pkw add archive.zip new.txt --into docs  # and delete, rename: a ZIP or 7z archive, in place; --exclude, --include too
pkw split backup.iso --size 4GB         # into parts beside it, as 7-Zip cuts one: backup.iso.001, .002…
pkw join backup.iso.001                 # back into backup.iso, the parts checked first
pkw checksum -a sha256 -a blake3 *.iso   # --save beside each, or --check SHA256SUMS
pkw audit list --since 2026-11-01        # the audit log, when it is kept (--csv; verify, status)
pkw convert photos.rar photos.tar.zst   # another format, as create makes it, an entry at a time with no folder in between (-p, -l, --split…)
pkw convert site.zip site.7z --exclude node_modules/ --exclude '*.log'   # and --include: what goes in
pkw keygen -o ~/keys                     # a key to sign with: Ed25519 and ML-DSA-65, and its .pub
pkw sign release.zip --by ~/keys/packwing-signing-key   # or an SSH Ed25519 key: release.zip.sig
pkw verify release.zip --trust their.pub # against the keys the app trusts, and these
pkw passphrase                           # random words, as the app suggests
pkw storages                             # the storages added to the app, by name
pkw list Backups:2026/                   # a folder there, or here: NAME:PATH, or a path
pkw extract Backups:2026/site.tar.zst    # read where it is, a part at a time
pkw cp site.tar.zst Backups:2026/        # copy, and mv: files and folders, into a folder
```

A password is never put on the command line, where others see it: `pkw` asks for it in a terminal, or reads it from `--password-file FILE`, `--password-stdin` or `PACKWING_PASSWORD`; an age or SSH key file comes with `--key FILE`. Its exit code says how it went: `0` done, `1` failed, `2` not understood, `3` a password or key is needed or wrong, `4` not an archive it opens, `5` done but some files' data was damaged, and skipped or kept, or some files were left out as their names could not be used. In a terminal `extract` and `convert` ask what to do with a file whose data is damaged (a check that fails, data cut short); in a script, or with `--quiet`, they stop, unless `--on-damaged` says otherwise. Names are taken as this system takes them, so that `a:b.txt` extracts on Linux; those it cannot use (that name on Windows, or `README` beside `readme` on macOS) are asked about once, all together, before anything is written, in a terminal, and skipped otherwise, unless `--on-bad-name` says to stop or rename them; `--portable-names` takes only names every system can use. `convert` takes any name an archive holds, as it writes no folder. A symbolic link that leads outside the folder (to an absolute path, as `~/snap`'s to `~/.config`, or climbing out) is asked about once, all together, before anything is written, in a terminal and in the app, and left out and said otherwise, unless `--links-outside` says to stop or to create them as they are, for an archive you trust; links are created after every file, so nothing is written through them, and `convert` keeps every link, as the new archive holds it. A name that leads outside, or a limit, always stops. `NAME:PATH` is a path on the storage the app calls `NAME` (Storages, in the app), opened with what the app keeps in the system's keychain; `pkw` reads the app's settings and never changes them. One that asks for this computer's password first is opened in the app only. On Windows and macOS what comes from an archive or a storage marked as from the Internet is marked too, as the app does; `--no-mark` asks not to, which the organization's policy may refuse. Work is shared among as many threads as the computer has, or as Settings › General says, or `--threads N` (`-T N`): xz, bzip2 and 7z are written in pieces each compressed in a thread, gzip as pigz writes it, zstd in threads too, a ZIP archive's files compressed several at once, and what is written in pieces (by packwing, `xz -T`, `pzstd`, pbzip2 or 7-Zip) is read a piece in each thread, as is any bzip2 file of more than 2 MiB and a macOS package's payload; a ZIP or RAR archive's files and a 7z archive's solid blocks are extracted several at once, a solid RAR archive's large files decoded on several cores, and a tar or cpio archive's files written by threads while it is read. What is made is the same however many threads there are. They take no more than 70% of the memory free when a job starts, leaving at least 2 GB of it, or half where less than 4 GB is free (within the limits the system puts on packwing: a control group's and `ulimit`'s on Linux, a job object's on Windows), fewer threads where it is short; a level whose one thread would take more than is free is lowered until it fits, and said, so that packwing is never stopped for want of memory. `PACKWING_MEMORY`, in bytes, stands for the memory free. `pkw help <command>` says the rest, and `pkw completions bash|zsh|fish|powershell` writes the shell's completions. The .deb and the .rpm hold them, and its manual pages, as does `pkw` alone (`pkw_<version>_<os>_<arch>`, in each release); on Windows the installer puts `pkw.exe` beside packwing and its folder in the user's PATH. With [Homebrew](https://brew.sh), on macOS and Linux, `brew install hivepaas/tap/pkw` installs `pkw` alone, its completions and its manual pages, and `brew upgrade` updates it ([hivepaas/homebrew-tap](https://github.com/hivepaas/homebrew-tap)).
