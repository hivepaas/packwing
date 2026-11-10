# packwing

**A file archiver for macOS, Windows and Linux that puts every core of your computer to work, and opens archives safely.**

Built in Rust on [hivepack](https://github.com/hivepaas/hivepack-rs).

## 🌟 Key Features

* **Every core at work:** ZIP, 7z, `.tar.gz`, `.tar.xz`, `.tar.zst` and `.tar.bz2` are compressed on all of the computer's cores at once (gzip as pigz writes it, and a ZIP archive's deflate the same way, so that one large file takes every core too: 3 GB in 3.2 s, where it took 65 s), and what is written in pieces (by packwing, `xz -T`, `pzstd`, pbzip2 or 7-Zip) is read on all of them too, as is any bzip2 file of more than 2 MiB and a macOS package's payload; a ZIP or RAR archive's files and a 7z archive's solid blocks are extracted several at once, a solid RAR archive's large files decoded on several cores, and a tar or cpio archive's files written by threads while it is read. A `.tar.zst` of 256 MiB or more goes in frames of 32 MiB, which every core opens at once, with zstd's seek table at the end; New archive and `pkw --frames` choose one frame, the smallest, or several. A `.tar.xz` and a 7z archive go in their format's own blocks, the smallest, of up to 192 MiB (xz) or 256 MiB (7z), which are fewer than the cores where there is less than 6 to 8 GB to compress at the highest level; New archive and `pkw --blocks small` cut smaller ones, enough for 32 cores: up to twice as fast, and a few percent larger. The archive is the same, byte for byte, however many cores made it. The threads take no more than 70% of the memory free, and leave at least 2 GB of it to everything else (or half, where little is free), so a big job never runs the computer out of memory, and Settings › General, or `pkw --threads N`, sets how many.
* **Safe by default:** nothing is written outside the destination folder or through links, archive bombs stop at a size limit, and an existing file is never written over unless you say so. A file opens as a copy in a temporary folder, after asking, and a program always asks.
* **Opens what you come across:** ZIP (split archives too), RAR 1.4 to 7 (volumes too), 7z, tar, cpio, disc images (ISO 9660 and UDF), RPM packages and macOS packages, inside any of the layers around them: gzip, zstd, xz, bzip2 and more.
* **Writes archives:** ZIP, 7z, `.tar.gz`, `.tar.xz` and `.tar.zst`, with five levels, AES-256 on ZIP and 7z (7z can hide the names too), and parts as 7-Zip cuts them. A tar archive holds a file with several names (hard links) once, and keeps for a backup, when asked, each file's owner and group and its extended attributes (Linux's ACLs among them), as GNU tar's `--xattrs` does; they are left out otherwise, so that an archive to share does not tell the names of its users. Extracting gives them back when asked (Settings › Extracting, or `pkw extract --same-owner`, `--numeric-owner`, `--xattrs`), on Linux and macOS: only an administrator gives files to others.
* **age built in:** opens and writes `.age` files with a passphrase, age keys, Ed25519 SSH keys and post-quantum keys.
* **Edits ZIP and 7z archives:** add, delete and rename inside an archive, which goes in place only once the new one reads back, and only starts where there is room beside it for the new one. A 7z archive's solid blocks that lose nothing are copied as they are, encrypted or not; one that loses a file is compressed again as it was (its method, filter and dictionary), and what is added as the archive's blocks agree it was, solid or not, with its own password.
* **Look before you extract:** a search through all of an archive, whatever the case and accents (`bao cao` finds `Báo cáo`), a preview of pictures and text, and Extract for just what is selected. The archive's password is asked once: it is kept in memory, never written, until ten minutes after it was last used.
* **Storages:** browse folders, S3 buckets, WebDAV folders and SFTP servers as a file manager does, and copy and move between them, with secrets in the system's keychain.
* **Archives in place:** a ZIP or 7z archive on a storage is listed, and a file taken from it, without fetching the rest.
* **Signed files:** sign with an SSH Ed25519 key, or a new key of Ed25519 and ML-DSA-65 together; an SSH key's signature can be checked with `ssh-keygen -Y verify`, without packwing.
* **Checksums:** a file's SHA-256 and SHA-1, or SHA-512, MD5 and BLAKE3 as Settings › Security chooses, read in one pass, saved beside it as `sha256sum -c` checks them, and one pasted from a download page (or a `SHA256SUMS` list) compared with them.
* **In the background:** extracting, compressing, testing, copying and signing go on behind when asked, while the window does other things; two at a time, the others waiting their turn; a Jobs page says how far each is and how it ended, and waits for a password one needs; the system tells of one that ends while you are elsewhere, and closing the window asks first.
* **In your file manager:** Compress and Extract in the right-click menu of Windows Explorer, macOS Finder and Linux file managers, and drag and drop in the window.
* **Updates you can trust:** a daily look for a newer version, in a list believed only when its signatures verify, with a badge on the title bar; nothing is downloaded or installed unless asked, and the look can be turned off ([docs/updates.md](docs/updates.md)).
* **Ready for organizations:** Group Policy, a configuration profile or a file in `/etc` can keep the Internet mark, require encryption, set which kinds and how long a password has to be, and keep an audit log ([docs/policies.md](docs/policies.md)).
* **Audit log:** a record on the computer of what packwing does, which archives were extracted, made and copied, by whom and when, one JSON line each, chained so that a record changed or taken out shows. Kept when your organization's policy says, whatever the license, or when you turn it on with a Pro or Enterprise license; never a password or what is in a file ([docs/audit.md](docs/audit.md)).
* **On the command line too:** `pkw` lists, extracts, tests, makes, changes and converts archives, signs files and checks signatures, makes and checks checksums, and copies and moves files between this computer and the storages added to the app, reading archives where they are, with no window, as safely as the app and under the same policies, with JSON for scripts and exit codes that say what went wrong (Command line, below).
* **Made for every day:** an interface in 15 languages ([CONTRIBUTING.md](CONTRIBUTING.md)), light and dark, and a window that comes back as it was left.

## 🛠️ Building

packwing is a [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui) app, built with the Rust version in `rust-toolchain.toml` (rustup installs it by itself):

```sh
make release   # the window
make debug     # the same, built for debugging
make test
make           # lists the rest
```

On Debian and Ubuntu, building needs `libpam0g-dev` too, to check the login password for a storage that asks for it.

On Linux, `make` builds for Wayland alone, which needs no X11 development libraries (`libxkbcommon-x11-dev` and the like). For X11 too: `make release FEATURES=wayland,x11`. `cargo run -p packwing` builds for both; add `-- --first-run` for the welcome.

The welcome, or Settings › Right-click menu, puts packwing's items in the file manager's right-click menu for the user, through [context-menu-rs](https://github.com/hivepaas/context-menu-rs). Each runs the program that put it there, so a menu put in by `cargo run` runs that build; the next start of another build points it at that one. On Linux the items are in the menus of Dolphin, Nemo and Thunar; in Nautilus's own menu with the .deb's extension (below), and otherwise in its Scripts submenu, or its own menu with nautilus-python. On Windows they are under "Show more options" for a build run by hand, and in Windows 11's own menu with the installer's extension, where Windows takes its package (with Developer Mode on, until the package is signed); on macOS in Finder's own menu with Packwing.app's extension, and in its Quick Actions for a build run by hand.

To see every screen, light and dark, without opening the app (Linux, with a Vulkan driver such as Mesa's lavapipe):

```sh
cargo test -p packwing --features shots shots   # writes target/shots/*.png
```

To try storages with no account anywhere, Docker runs throwaway servers, reached from this computer only. Stopping one throws away what it holds.

```sh
make s3-start       # endpoint http://127.0.0.1:18333, bucket test, access key packwing, secret key packwing
make webdav-start   # address http://127.0.0.1:18335, user packwing, password packwing
make sftp-start     # host tester@127.0.0.1:18322, password packwing
S3_TEST_ENDPOINT=http://127.0.0.1:18333 S3_TEST_ACCESS_KEY=packwing S3_TEST_SECRET_KEY=packwing \
  WEBDAV_TEST_ENDPOINT=http://127.0.0.1:18335 WEBDAV_TEST_USER=packwing WEBDAV_TEST_PASSWORD=packwing \
  SFTP_TEST_HOST=tester@127.0.0.1:18322 SFTP_TEST_KEY=$PWD/target/test-sftp/key \
  SFTP_TEST_KNOWN_HOSTS=$PWD/target/test-sftp/known_hosts make test
make s3-stop
make webdav-stop
make sftp-stop
```

packwing refuses a host not in `~/.ssh/known_hosts`, so `make sftp-start` adds the server's line there for its port, and `make sftp-stop` takes it out.

## ⌨️ Command line

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
pkw add archive.zip new.txt --into docs  # and delete, rename: a ZIP or 7z archive, in place
pkw split backup.iso --size 4GB         # into parts beside it, as 7-Zip cuts one: backup.iso.001, .002…
pkw join backup.iso.001                 # back into backup.iso, the parts checked first
pkw checksum -a sha256 -a blake3 *.iso   # --save beside each, or --check SHA256SUMS
pkw audit list --since 2026-11-01        # the audit log, when it is kept (--csv; verify, status)
pkw convert photos.rar photos.tar.zst   # another format, as create makes it, an entry at a time with no folder in between (-p, -l, --split…)
pkw convert site.zip site.7z --exclude '**/node_modules' --exclude '*.log'   # and --include: what goes in
pkw keygen -o ~/keys                     # a key to sign with: Ed25519 and ML-DSA-65, and its .pub
pkw sign release.zip --by ~/keys/packwing-signing-key   # or an SSH Ed25519 key: release.zip.sig
pkw verify release.zip --trust their.pub # against the keys the app trusts, and these
pkw passphrase                           # random words, as the app suggests
pkw storages                             # the storages added to the app, by name
pkw list Backups:2026/                   # a folder there, or here: NAME:PATH, or a path
pkw extract Backups:2026/site.tar.zst    # read where it is, a part at a time
pkw cp site.tar.zst Backups:2026/        # copy, and mv: files and folders, into a folder
```

A password is never put on the command line, where others see it: `pkw` asks for it in a terminal, or reads it from `--password-file FILE`, `--password-stdin` or `PACKWING_PASSWORD`; an age or SSH key file comes with `--key FILE`. Its exit code says how it went: `0` done, `1` failed, `2` not understood, `3` a password or key is needed or wrong, `4` not an archive it opens, `5` done but some files' data was damaged, and skipped or kept, or some files were left out as their names could not be used. In a terminal `extract` and `convert` ask what to do with a file whose data is damaged (a check that fails, data cut short); in a script, or with `--quiet`, they stop, unless `--on-damaged` says otherwise. Names are taken as this system takes them, so that `a:b.txt` extracts on Linux; those it cannot use (that name on Windows, or `README` beside `readme` on macOS) are asked about once, all together, before anything is written, in a terminal, and skipped otherwise, unless `--on-bad-name` says to stop or rename them; `--portable-names` takes only names every system can use. `convert` takes any name an archive holds, as it writes no folder. A symbolic link that leads outside the folder (to an absolute path, as `~/snap`'s to `~/.config`, or climbing out) is asked about once, all together, before anything is written, in a terminal and in the app, and left out and said otherwise, unless `--links-outside` says to stop or to create them as they are, for an archive you trust; links are created after every file, so nothing is written through them, and `convert` keeps every link, as the new archive holds it. A name that leads outside, or a limit, always stops. `NAME:PATH` is a path on the storage the app calls `NAME` (Storages, in the app), opened with what the app keeps in the system's keychain; `pkw` reads the app's settings and never changes them. One that asks for this computer's password first is opened in the app only. On Windows and macOS what comes from an archive or a storage marked as from the Internet is marked too, as the app does; `--no-mark` asks not to, which the organization's policy may refuse. Work is shared among as many threads as the computer has, or as Settings › General says, or `--threads N` (`-T N`): xz, bzip2 and 7z are written in pieces each compressed in a thread, gzip as pigz writes it, zstd in threads too, a ZIP archive's files compressed several at once, and what is written in pieces (by packwing, `xz -T`, `pzstd`, pbzip2 or 7-Zip) is read a piece in each thread, as is any bzip2 file of more than 2 MiB and a macOS package's payload; a ZIP or RAR archive's files and a 7z archive's solid blocks are extracted several at once, a solid RAR archive's large files decoded on several cores, and a tar or cpio archive's files written by threads while it is read. What is made is the same however many threads there are. They take no more than 70% of the memory free when a job starts, leaving at least 2 GB of it, or half where less than 4 GB is free (within the limits the system puts on packwing: a control group's and `ulimit`'s on Linux, a job object's on Windows), fewer threads where it is short; a level whose one thread would take more than is free is lowered until it fits, and said, so that packwing is never stopped for want of memory. `PACKWING_MEMORY`, in bytes, stands for the memory free. `pkw help <command>` says the rest, and `pkw completions bash|zsh|fish|powershell` writes the shell's completions. The .deb and the .rpm hold them, and its manual pages, as does `pkw` alone (`pkw_<version>_<os>_<arch>`, in each release); on Windows the installer puts `pkw.exe` beside packwing and its folder in the user's PATH.

## 📦 Packaging

`scripts/package_linux.sh` packs a release build into `dist/packwing_<version>_<arch>.deb` for Debian and Ubuntu and `dist/packwing_<version>_<arch>.rpm` for Fedora and openSUSE: the program, its launcher and icon, `pkw` with its manual pages (`man pkw`, `man pkw-extract`…) and its completions for bash, zsh and fish, and context-menu-rs's native Nautilus extension, downloaded from its release and checked against the SHA-256 pinned in the script. Nautilus loads extensions only from a folder of the system's, which a package can write and a user cannot, so it is what puts packwing's items in Nautilus's own menu rather than its Scripts submenu. It makes `pkw` alone too, in `dist/pkw_<version>_linux_<arch>.tar.gz`, with its pages and completions, for a server with no desktop; the macOS and Windows scripts make its `.tar.gz` and `.zip` alike.

```sh
cargo build --release -p packwing -p packwing-cli
scripts/package_linux.sh
sudo apt install ./dist/packwing_0.1.0_amd64.deb
nautilus -q   # Nautilus loads extensions when it starts
```

`scripts/check_linux_package.sh` checks a package holds what it should, and with `--install` installs it and runs `pkw` from it, as CI does after each merge, for the `.deb`, the `.rpm` (in a Fedora container) and `pkw` alone.

On macOS, `scripts/package_macos.sh` makes `Packwing.app`, with `pkw` beside the program and context-menu-rs's native Finder extension and its helper, downloaded from its release and checked alike, in a disk image and a `.app.tar.gz` for the updater. The extension is signed with the team's Developer ID, so it is in the app only when one is given; then the app is notarized too ([docs/releasing.md](docs/releasing.md)). Windows' installer, from `scripts/package_windows.sh`, holds the extension for Windows 11's menu. A release builds them all from a tag ([docs/releasing.md](docs/releasing.md)).

packwing's next start moves its items for Nautilus out of the Scripts submenu, into Nautilus's own menu. The package depends on the glibc of the system it was built on, so the packages are built on the oldest Ubuntu supported, and the .rpm names the glibc it needs. Removing the package leaves each user's items for Dolphin, Nemo and Thunar, which Settings takes away before.

## 🐝 Made by HivePaaS

packwing is built and maintained by HivePaaS, who also make:

- [hivepaas](https://github.com/hivepaas/hivepaas): a self-hosted PaaS.
- [hivepack](https://github.com/hivepaas/hivepack-rs): read and write archives and compressed files in Rust, the library under packwing.
- [context-menu-rs](https://github.com/hivepaas/context-menu-rs): put your app in the right-click menu of Windows Explorer, macOS Finder and Linux file managers, from Rust.
- [shellwing](https://shellwing.app): a fast, native SSH client for macOS, Windows and Linux.
- [authwing](https://github.com/hivepaas/authwing): a two-factor code app for macOS, Windows and Linux that knows the window you are in.

⭐ If packwing, or any of these, is useful to you, please give it a star on GitHub: it helps others find it, and helps us keep making them.

## 📜 License

packwing is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License, version 3 or (at your option) any later version. See [LICENSE](LICENSE). Copyright (C) 2026 HivePaaS.

packwing reads RAR through `hivepack::rar`, hivepack's port of UnRAR under UnRAR's own license, which [GPL-ADDITIONAL-PERMISSION.md](GPL-ADDITIONAL-PERMISSION.md) allows it to be combined with, under section 7 of the GPL.

The name and logo are not covered by the GPL: see [TRADEMARKS.md](TRADEMARKS.md). Contributions are welcome under the terms in [CONTRIBUTING.md](CONTRIBUTING.md).

The Rust crates packwing is built with keep their licenses too: `cargo about generate -c about.toml -m Cargo.toml --workspace packaging/licenses.hbs` lists them, the program's and pkw's, with their texts, and the packages hold that list (`about.toml` says which licenses may come in). Work by others keeps its license and notice: the words of a suggested passphrase are the [EFF's large word list](https://www.eff.org/dice) (CC BY 3.0 US), `crates/core/src/passphrase_words.txt`, its numbers taken out; the passwords a policy refuses are [SecLists](https://github.com/danielmiessler/SecLists)' top 10,000 (MIT, Copyright (c) 2018 Daniel Miessler), `crates/policy/src/common_passwords.txt`, lower-cased and sorted; the UI's typeface, [Nunito](https://github.com/googlefonts/nunito), is under the SIL Open Font License 1.1 (see `crates/app/assets/fonts/OFL.txt`), and the text field (`crates/app/src/input.rs`) comes from shellwing, which adapted it from GPUI's `examples/input.rs` (Apache-2.0, Zed Industries).
