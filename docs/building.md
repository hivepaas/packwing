# Building packwing

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

## Packaging

`scripts/package_linux.sh` packs a release build into `dist/packwing_<version>_<arch>.deb` for Debian and Ubuntu and `dist/packwing_<version>_<arch>.rpm` for Fedora and openSUSE: the program, its launcher and icon, `pkw` with its manual pages (`man pkw`, `man pkw-extract`…) and its completions for bash, zsh and fish, and context-menu-rs's native Nautilus extension, downloaded from its release and checked against the SHA-256 pinned in the script. Nautilus loads extensions only from a folder of the system's, which a package can write and a user cannot, so it is what puts packwing's items in Nautilus's own menu rather than its Scripts submenu. It makes `pkw` alone too, in `dist/pkw_<version>_linux_<arch>.tar.gz`, with its pages and completions, for a server with no desktop; the macOS and Windows scripts make its `.tar.gz` and `.zip` alike.

```sh
cargo build --release -p packwing -p packwing-cli
scripts/package_linux.sh
sudo apt install ./dist/packwing_0.1.0_amd64.deb
nautilus -q   # Nautilus loads extensions when it starts
```

`scripts/check_linux_package.sh` checks a package holds what it should, and with `--install` installs it and runs `pkw` from it, as CI does after each merge, for the `.deb`, the `.rpm` (in a Fedora container) and `pkw` alone.

On macOS, `scripts/package_macos.sh` makes `Packwing.app`, with `pkw` beside the program and context-menu-rs's native Finder extension and its helper, downloaded from its release and checked alike, in a disk image and a `.app.tar.gz` for the updater. The extension is signed with the team's Developer ID, so it is in the app only when one is given; then the app is notarized too ([docs/releasing.md](releasing.md)). Windows' installer, from `scripts/package_windows.sh`, holds the extension for Windows 11's menu. A release builds them all from a tag ([docs/releasing.md](releasing.md)).

packwing's next start moves its items for Nautilus out of the Scripts submenu, into Nautilus's own menu. The package depends on the glibc of the system it was built on, so the packages are built on the oldest Ubuntu supported, and the .rpm names the glibc it needs. Removing the package leaves each user's items for Dolphin, Nemo and Thunar, which Settings takes away before.

## The Mac App Store's build

`--features app-store` builds packwing as the Mac App Store has it: in the App Sandbox (`packaging/macos/app-store.entitlements`), updated by the store, and with no SFTP. Its right-click menu is the one the sandbox allows, macOS Services: Finder lists them under Services (or Quick Actions) for the files chosen, as `packaging/macos/app-store-services.json` names them, and the sandbox lets packwing read those files (`crates/sandbox/src/services.rs`). Settings › Right-click menu is not there. In the sandbox macOS marks everything packwing makes as quarantined, and writes that mark itself: packwing writes none there (`crates/core/src/mark.rs`). In the sandbox packwing reaches only what the user gives it: an archive opened from Finder, chosen or dropped, while it runs. A folder beside it, to extract there or to read the archive's other parts, is asked for in an open panel on that folder, and what the user chooses is kept as a security-scoped bookmark for the next runs (`crates/app/src/access.rs`, through `crates/sandbox`). Browse and Storages start from the user's home, and a folder not given yet shows **Allow access…**.

`scripts/run_sandboxed.sh [ARCHIVE…]` builds it and runs it so, opening the archives as Finder does; the UI scenarios of `ui_tests::sandbox` play the sandbox anywhere, and `ui_tests::store` checks what the build leaves out.

Pro is sold there as an in-app purchase, `com.hivepaas.packwing.pro` (`crates/store`, StoreKit 2 through a Swift package its `build.rs` builds; Settings › License shows **Buy Pro** and **Restore Purchases**). A key bought on the website still counts.

`scripts/package_macos_appstore.sh` packages it for the store: one `Packwing.app` for Apple silicon and Intel, with packwing alone (no `pkw`, no Finder extension), the store's entitlements and profile, a privacy manifest (`packaging/macos/PrivacyInfo.xcprivacy`), and the number of commits as its build number, in `dist/packwing_<version>_macos_appstore.pkg`. It stops if the program links a private framework, calls Apple's private window-server functions or names AppKit's private methods, or was built without the feature. Signing and sending it: [releasing.md](releasing.md#the-mac-app-store).
