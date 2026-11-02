# packwing

**A file archiver for macOS, Windows and Linux that opens archives safely.**

Built in Rust on [hivepack](https://github.com/hivepaas/hivepack-rs).

> **Status:** early development. There is nothing to download yet. packwing opens, extracts and tests ZIP, RAR (RAR 1.4 to 7, volumes, encrypted too; it never writes RAR), 7z (encrypted too, with its password), tar and cpio archives, disc images (`.iso`: ISO 9660, and UDF as DVDs and Windows' images have it), RPM packages and macOS packages (`.pkg`, their payloads expanded as `pkgutil --expand-full` does) through hivepack, and the layers around them (gzip, zstd, xz, bzip2 and the others hivepack reads, and age, with a passphrase or a key file). An archive that needs a passphrase or a password asks for it when it turns out to, and again when the one typed is wrong; it is kept only while the archive is open. An age file encrypted to a key asks for its key file: age's own (under `age -p` too) or an unencrypted or passphrase-protected Ed25519 SSH key. It writes ZIP, 7z and `.tar.gz`, `.tar.xz` and `.tar.zst` archives (New archive, and the right-click menu's Compress items): five levels, AES-256 with a password on ZIP and 7z (7z can hide the names too), and parts as 7-Zip cuts them, which it opens again, as it does split ZIP archives; it never writes over a file that is there. Links stay links, and extracting makes those that stay inside the folder and stops at those that leave it. Add files adds to a ZIP archive of one file, in the folder shown: it writes the archive anew and puts it in place only once it reads back, and a name already there stops it, is replaced or is skipped, as chosen. What is selected in such an archive can be deleted or renamed, the same way; other formats do not offer it yet. The samples the screenshots draw are made up. Click to select entries, with Ctrl or Cmd for more and Shift for a range, and Extract offers just those, straight into the folder; a double-click opens a file, as a copy in a temporary folder, asking first (always, for a program). A file clicked shows in the preview, a picture or the start of a text, up to 32 MB. Files dropped on the window go in the archive open if it can take them; otherwise an archive opens and anything else starts a new one; New archive and Add files take drops too. Names are not detected or chosen by encoding: one in an old encoding still extracts, its file's contents right and its name perhaps not. New archive and Add files list what goes in, behind an eye button, for taking things out, and show when files are being dragged in. The search looks through all of an archive, whatever the case and accents (`bao cao` finds `Báo cáo`), and a result opens where it is. The window comes back as it was left: its size, its place where the system allows it (Wayland does not), maximized or fullscreen. Settings are kept between runs, in the system's folder for settings, with the archives Recent lists. They act: the language (English so far, and the code is ready for more: see [CONTRIBUTING.md](CONTRIBUTING.md)), the format, level and hidden names New archive starts with, Mac-only files, where Extract starts, replacing files, the size limit, and the Internet mark.

The Storages page browses folders on this computer as a file manager does: a folder's items a page at a time, with Load more, sorted by a column, searched, with a right-click menu and the keys of a file manager (Enter, Backspace, F2, Delete, Space, F5). It shows what is known about what is chosen and counts a folder when asked; renames, makes folders, moves to the system's Trash, copies and moves between folders and storages, opens archives, and shows or edits files in the text editor chosen in Settings. A copy appears under its name only when all of it is written, and a moved file's original goes once its copy has been read back and matches. Links are not followed: they stay where they are. A file that could run is never opened from here with its default app.

S3 buckets, WebDAV folders and servers over SFTP are storages too, added from Add storage with what they need, their secrets in the system's keychain. SFTP goes by `ssh_config`, the SSH agent and key files, and takes a host only if `known_hosts` already has it. They are browsed a page at a time and take everything above but what needs a file on this computer: the editor and the default app wait for a copy here. An archive on one opens where it is, read by ranges: a ZIP or 7z archive is listed, and a file taken from it, without fetching the rest, and the header says how much has come across. A tar archive or anything compressed as a whole is read from its start to reach a file, and a RAR archive is fetched first. It is only read: nothing is added or changed in it, and it extracts to this computer. On S3 and WebDAV a folder is moved a file at a time rather than renamed, and a file sent to WebDAV is held in memory, up to 1 GB. Copies and moves go through this computer, and a file to S3 goes in parts, so that a stopped one leaves nothing.

## 🎯 Plans

- **Formats:** ZIP (split archives too), RAR (reading), tar and `.tar.zst`.
- **Safe extraction by default:** nothing is written outside the destination folder or through links, and archive bombs stop at a size limit.

An organization can fix some of this for everyone, through Group Policy, a configuration profile or a file in `/etc`: keeping the Internet mark, requiring encryption, which kinds, and how long a password has to be. [docs/policies.md](docs/policies.md) is for its administrators. `packwing --policies` says what is set.

## 💳 Plans

packwing is free software, and Free does everything but keep more than one storage across a network (S3, WebDAV, SFTP); folders on this computer are not limited. Pro has no limit. Enterprise has none either, with one key an organization gives to all its computers through a policy, and an agreement behind it. A key is checked on the computer, with no account and nothing sent anywhere, and no plan changes what policies or security features do. The check asks people to pay; as the license allows, anyone can build packwing without it.

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

The welcome, or Settings › Right-click menu, puts packwing's items in the file manager's right-click menu for the user, through [context-menu-rs](https://github.com/hivepaas/context-menu-rs). Each runs the program that put it there, so a menu put in by `cargo run` runs that build; the next start of another build points it at that one. On Linux the items are in the menus of Dolphin, Nemo and Thunar; in Nautilus's own menu with the .deb's extension (below), and otherwise in its Scripts submenu, or its own menu with nautilus-python. On Windows they are under "Show more options", and on macOS in Finder's Quick Actions, until packwing's packages there hold the native extensions.

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

## 📦 Packaging

On Debian and Ubuntu, `scripts/package_linux.sh` packs a release build into `dist/packwing_<version>_<arch>.deb`: the program, its launcher and icon, and context-menu-rs's native Nautilus extension, downloaded from its release and checked against the SHA-256 pinned in the script. Nautilus loads extensions only from a folder of the system's, which a package can write and a user cannot, so it is what puts packwing's items in Nautilus's own menu rather than its Scripts submenu.

```sh
cargo build --release -p packwing
scripts/package_linux.sh
sudo apt install ./dist/packwing_0.1.0_amd64.deb
nautilus -q   # Nautilus loads extensions when it starts
```

packwing's next start moves its items for Nautilus out of the Scripts submenu, into Nautilus's own menu. The package depends on the glibc of the system it was built on, so a .deb for every supported Ubuntu is built on the oldest one. Removing the package leaves each user's items for Dolphin, Nemo and Thunar, which Settings takes away before.

## 🐝 Made by HivePaaS

packwing is built and maintained by HivePaaS, who also make:

- [hivepaas](https://github.com/hivepaas/hivepaas): a self-hosted PaaS.
- [shellwing](https://shellwing.app): a fast, native SSH client for macOS, Windows and Linux.
- [authwing](https://github.com/hivepaas/authwing): a two-factor code app for macOS, Windows and Linux that knows the window you are in.

## 📜 License

packwing is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License, version 3 or (at your option) any later version. See [LICENSE](LICENSE). Copyright (C) 2026 HivePaaS.

packwing reads RAR through `hivepack-rar`, a port of UnRAR under UnRAR's own license, which [GPL-ADDITIONAL-PERMISSION.md](GPL-ADDITIONAL-PERMISSION.md) allows it to be combined with, under section 7 of the GPL.

The name and logo are not covered by the GPL: see [TRADEMARKS.md](TRADEMARKS.md). Contributions are welcome under the terms in [CONTRIBUTING.md](CONTRIBUTING.md).

The Rust crates packwing is built with keep their licenses too: `cargo about generate -c about.toml -m crates/app/Cargo.toml packaging/licenses.hbs` lists them with their texts, and the packages hold that list (`about.toml` says which licenses may come in). Work by others keeps its license and notice: the words of a suggested passphrase are the [EFF's large word list](https://www.eff.org/dice) (CC BY 3.0 US), `crates/core/src/passphrase_words.txt`, its numbers taken out; the passwords a policy refuses are [SecLists](https://github.com/danielmiessler/SecLists)' top 10,000 (MIT, Copyright (c) 2018 Daniel Miessler), `crates/policy/src/common_passwords.txt`, lower-cased and sorted; the UI's typeface, [Nunito](https://github.com/googlefonts/nunito), is under the SIL Open Font License 1.1 (see `crates/app/assets/fonts/OFL.txt`), and the text field (`crates/app/src/input.rs`) comes from shellwing, which adapted it from GPUI's `examples/input.rs` (Apache-2.0, Zed Industries).
