# packwing

**A file archiver for macOS, Windows and Linux that opens archives safely.**

Built in Rust on [hivepack](https://github.com/hivepaas/hivepack-rs).

> **Status:** early development. There is nothing to download yet. packwing opens, extracts and tests ZIP, 7z (encrypted too, with its password), tar and cpio archives, disc images (`.iso`: ISO 9660, and UDF as DVDs and Windows' images have it), RPM packages and macOS packages (`.pkg`, their payloads expanded as `pkgutil --expand-full` does) through hivepack, and the layers around them (gzip, zstd, xz, bzip2 and the others hivepack reads, and age, with a passphrase or a key file). An archive that needs a passphrase or a password asks for it when it turns out to, and again when the one typed is wrong; it is kept only while the archive is open. An age file encrypted to a key asks for its key file: age's own (under `age -p` too) or an unencrypted or passphrase-protected Ed25519 SSH key. It writes ZIP, 7z and `.tar.gz`, `.tar.xz` and `.tar.zst` archives (New archive, and the right-click menu's Compress items): five levels, AES-256 with a password on ZIP and 7z (7z can hide the names too), and parts as 7-Zip cuts them, which it opens again, as it does split ZIP archives; it never writes over a file that is there. Links stay links, and extracting makes those that stay inside the folder and stops at those that leave it. Add files adds to a ZIP archive of one file, in the folder shown: it writes the archive anew and puts it in place only once it reads back, and a name already there stops it with nothing changed; other formats do not offer it yet. Repair still shows made-up progress, as do Recent and the samples the screenshots draw.

## 🎯 Plans

- **Formats:** ZIP (split archives too), RAR (reading), tar and `.tar.zst`.
- **Safe extraction by default:** nothing is written outside the destination folder or through links, and archive bombs stop at a size limit.

## 🛠️ Building

packwing is a [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui) app, built with the Rust version in `rust-toolchain.toml` (rustup installs it by itself):

```sh
cargo run -p packwing   # `-- --first-run`: the welcome; `-- --managed`: as an organization runs it
cargo test -p packwing
```

On Linux without the X11 development libraries (`libxkbcommon-x11-dev` and the like), build for Wayland alone: `cargo run -p packwing --no-default-features --features wayland`.

The welcome, or Settings › Right-click menu, puts packwing's items in the file manager's right-click menu for the user, through [context-menu-rs](https://github.com/hivepaas/context-menu-rs). Each runs the program that put it there, so a menu put in by `cargo run` runs that build; the next start of another build points it at that one. On Linux the items are in the menus of Dolphin, Nemo and Thunar; in Nautilus's own menu with the .deb's extension (below), and otherwise in its Scripts submenu, or its own menu with nautilus-python. On Windows they are under "Show more options", and on macOS in Finder's Quick Actions, until packwing's packages there hold the native extensions.

To see every screen, light and dark, without opening the app (Linux, with a Vulkan driver such as Mesa's lavapipe):

```sh
cargo test -p packwing --features shots shots   # writes target/shots/*.png
```

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

The name and logo are not covered by the GPL: see [TRADEMARKS.md](TRADEMARKS.md). Contributions are welcome under the terms in [CONTRIBUTING.md](CONTRIBUTING.md).

The Rust crates packwing is built with keep their licenses too: `cargo about generate -c about.toml -m crates/app/Cargo.toml packaging/licenses.hbs` lists them with their texts, and the packages hold that list (`about.toml` says which licenses may come in). Work by others keeps its license and notice: the UI's typeface, [Nunito](https://github.com/googlefonts/nunito), is under the SIL Open Font License 1.1 (see `crates/app/assets/fonts/OFL.txt`), and the text field (`crates/app/src/input.rs`) comes from shellwing, which adapted it from GPUI's `examples/input.rs` (Apache-2.0, Zed Industries).
