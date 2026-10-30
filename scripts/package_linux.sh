#!/bin/bash
#
# Packages a release build of packwing for Linux into dist/, as a .deb: the
# program, its launcher and icon, and the native Nautilus extension serving
# packwing's right-click menu. Nautilus loads extensions from a folder of
# the system's alone, which only a package can put one in; with it, the
# items show in Nautilus's own menu rather than its Scripts submenu
# (crates/app/src/right_click.rs).
#
#   cargo build --release -p packwing
#   scripts/package_linux.sh
#
#   dist/packwing_0.1.0_amd64.deb
#
# The extension is context-menu-rs's, downloaded from the release of the
# context-menu crate packwing builds with (it reads the menu's definition
# that crate writes) and checked against its SHA-256 before it is packed.
#
# The licenses of the Rust crates the program is built with go in its doc
# folder, written by cargo-about (about.toml, packaging/licenses.hbs):
#
#   cargo install cargo-about --locked --features cli
#
# Environment: BIN (default target/release/packwing), ARCH (default the
# machine's: x86_64 or aarch64), DIST (default dist).

set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

VERSION="$(sed -n '/^\[workspace.package\]/,/^\[/s/^version = "\(.*\)"/\1/p' Cargo.toml)"
ARCH="${ARCH:-$(uname -m)}"
BIN="${BIN:-target/release/packwing}"
DIST="${DIST:-dist}"
WORK="target/package-linux"

case "$ARCH" in
x86_64) DEB_ARCH=amd64 ;;
aarch64) DEB_ARCH=arm64 ;;
*) echo "no Linux package for $ARCH" >&2; exit 1 ;;
esac
# Where Debian and Ubuntu's Nautilus loads its extensions from.
EXTENSIONS="usr/lib/$ARCH-linux-gnu/nautilus/extensions-4"

# context-menu-rs 0.2.0's Nautilus extension.
CONTEXT_MENU=0.2.0
EXTENSION_URL="https://github.com/hivepaas/context-menu-rs/releases/download/v$CONTEXT_MENU/libcontext-menu-nautilus-$ARCH.so"
case "$ARCH" in
x86_64) EXTENSION_SHA=a85c9d885d0faaa65948a71bd2ca762a64fb0c993647b69f9b1faa3983017d1e ;;
aarch64) EXTENSION_SHA=1d5cb8a183afa8d4eb61bab54a6a460b8d1ff364a88a16b098948dfe23d4ce60 ;;
esac

LOCKED="$(sed -n '/^name = "context-menu"$/{n;s/^version = "\(.*\)"/\1/p}' Cargo.lock)"
if [ "$LOCKED" != "$CONTEXT_MENU" ]; then
	echo "packwing builds with context-menu $LOCKED, and this packs the extension of $CONTEXT_MENU:" >&2
	echo "pin the extension of the same release here" >&2
	exit 1
fi
[ -x "$BIN" ] || { echo "no build at $BIN: cargo build --release -p packwing" >&2; exit 1; }
cargo about --version >/dev/null 2>&1 || { echo "no cargo-about: cargo install cargo-about --locked --features cli" >&2; exit 1; }
rm -rf "$WORK"
mkdir -p "$WORK" "$DIST"

# A download, kept only when its SHA-256 is the one expected.
fetch() {
	local url="$1" sha="$2" out="$3"
	curl -fsSL --retry 3 -o "$out" "$url"
	echo "$sha  $out" | sha256sum -c --quiet - || { echo "$url: not the file expected" >&2; exit 1; }
}

DEB="$WORK/deb"
install -Dm755 "$BIN" "$DEB/usr/bin/packwing"
strip "$DEB/usr/bin/packwing"
install -Dm644 packaging/linux/packwing.desktop "$DEB/usr/share/applications/packwing.desktop"
install -Dm644 packaging/icons/packwing.svg "$DEB/usr/share/icons/hicolor/scalable/apps/packwing.svg"
install -Dm644 packaging/linux/copyright "$DEB/usr/share/doc/packwing/copyright"
# Those of the crates built for this machine; one under a license
# about.toml does not accept stops here.
cargo about generate --locked --fail -c about.toml -m crates/app/Cargo.toml \
	--target "$ARCH-unknown-linux-gnu" -o "$WORK/THIRD-PARTY-LICENSES" packaging/licenses.hbs
install -Dm644 "$WORK/THIRD-PARTY-LICENSES" "$DEB/usr/share/doc/packwing/THIRD-PARTY-LICENSES"
# Named after packwing's menu, which is how the extension knows it.
fetch "$EXTENSION_URL" "$EXTENSION_SHA" "$WORK/extension.so"
install -Dm644 "$WORK/extension.so" "$DEB/$EXTENSIONS/libcontext-menu-packwing.so"

# The libraries the program links, as the system names their packages. The
# extension links none of its own: Nautilus's are there when it is loaded.
mkdir -p "$WORK/shlibs/debian"
printf 'Source: packwing\n\nPackage: packwing\nArchitecture: any\n' >"$WORK/shlibs/debian/control"
DEPENDS="$(cd "$WORK/shlibs" && dpkg-shlibdeps -O "$ROOT/$DEB/usr/bin/packwing" 2>/dev/null | sed -n 's/^shlibs:Depends=//p')"
mkdir -p "$DEB/DEBIAN"
cat >"$DEB/DEBIAN/control" <<EOF
Package: packwing
Version: $VERSION
Architecture: $DEB_ARCH
Maintainer: HivePaaS <support@hivepaas.com>
Installed-Size: $(du -sk --exclude=DEBIAN "$DEB" | cut -f1)
Depends: $DEPENDS
Recommends: libwayland-client0, libxcb1, libx11-xcb1, libvulkan1, mesa-vulkan-drivers
Enhances: nautilus
Section: utils
Priority: optional
Homepage: https://github.com/hivepaas/packwing
Description: File archiver that opens archives safely
 Opens ZIP, RAR, 7z and tar archives without writing outside the folder
 chosen, and makes smaller files to share. Its items in the right-click
 menu of Nautilus, Dolphin, Nemo and Thunar extract and compress from any
 folder.
EOF
chmod -R go-w "$DEB"
dpkg-deb --root-owner-group --build "$DEB" "$DIST/packwing_${VERSION}_${DEB_ARCH}.deb"

ls -l "$DIST"
