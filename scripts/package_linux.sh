#!/bin/bash
#
# Packages a release build of packwing for Linux into dist/, as a .deb and an
# .rpm: the program, its launcher and icon, the command line (pkw, with its
# manual pages and its completions for bash, zsh and fish), and the native
# Nautilus extension serving packwing's right-click menu. Nautilus loads
# extensions from a folder of the system's alone, which only a package can put
# one in; with it, the items show in Nautilus's own menu rather than its
# Scripts submenu (crates/app/src/right_click.rs). And pkw alone, in a tar.gz,
# for servers and scripts: no window system in it.
#
#   cargo build --release -p packwing -p packwing-cli
#   scripts/package_linux.sh
#
#   dist/packwing_0.1.0_amd64.deb
#   dist/packwing_0.1.0_x86_64.rpm
#   dist/pkw_0.1.0_linux_x86_64.tar.gz
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
# pkw writes its own pages and completions, so it is run here: a build for
# this machine's ARCH.
#
# Environment: BIN (default target/release/packwing), CLI (default
# target/release/pkw), ARCH (default the machine's: x86_64 or aarch64), DIST
# (default dist).

set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

VERSION="$(sed -n '/^\[workspace.package\]/,/^\[/s/^version = "\(.*\)"/\1/p' Cargo.toml)"
ARCH="${ARCH:-$(uname -m)}"
BIN="${BIN:-target/release/packwing}"
CLI="${CLI:-target/release/pkw}"
DIST="${DIST:-dist}"
WORK="target/package-linux"

case "$ARCH" in
x86_64) DEB_ARCH=amd64 ;;
aarch64) DEB_ARCH=arm64 ;;
*) echo "no Linux package for $ARCH" >&2; exit 1 ;;
esac
# Where Debian and Ubuntu's Nautilus loads its extensions from, and Fedora
# and openSUSE's.
EXTENSIONS="usr/lib/$ARCH-linux-gnu/nautilus/extensions-4"
RPM_EXTENSIONS="usr/lib64/nautilus/extensions-4"

# context-menu-rs 0.2.1's Nautilus extension.
CONTEXT_MENU=0.2.1
EXTENSION_URL="https://github.com/hivepaas/context-menu-rs/releases/download/v$CONTEXT_MENU/libcontext-menu-nautilus-$ARCH.so"
case "$ARCH" in
x86_64) EXTENSION_SHA=54f644412436ab390f76227272cb84c44e804ee6de037f9f6dd613cfa74241d2 ;;
aarch64) EXTENSION_SHA=1d39480d2ee155066969cd5e6e1f108cfb5bd4acf144b32a2840020669b06800 ;;
esac

LOCKED="$(sed -n '/^name = "context-menu"$/{n;s/^version = "\(.*\)"/\1/p}' Cargo.lock)"
if [ "$LOCKED" != "$CONTEXT_MENU" ]; then
	echo "packwing builds with context-menu $LOCKED, and this packs the extension of $CONTEXT_MENU:" >&2
	echo "pin the extension of the same release here" >&2
	exit 1
fi
[ -x "$BIN" ] || { echo "no build at $BIN: cargo build --release -p packwing" >&2; exit 1; }
[ -x "$CLI" ] || { echo "no build at $CLI: cargo build --release -p packwing-cli" >&2; exit 1; }
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
install -Dm755 "$CLI" "$DEB/usr/bin/pkw"
strip "$DEB/usr/bin/pkw"
# pkw's pages, compressed as Debian keeps them, and its completions where
# each shell looks for a package's.
"$CLI" man "$WORK/man"
for page in "$WORK"/man/*.1; do
	gzip -9n <"$page" >"$page.gz"
	install -Dm644 "$page.gz" "$DEB/usr/share/man/man1/$(basename "$page").gz"
done
"$CLI" completions bash >"$WORK/pkw.bash"
"$CLI" completions zsh >"$WORK/_pkw"
"$CLI" completions fish >"$WORK/pkw.fish"
install -Dm644 "$WORK/pkw.bash" "$DEB/usr/share/bash-completion/completions/pkw"
install -Dm644 "$WORK/_pkw" "$DEB/usr/share/zsh/vendor-completions/_pkw"
install -Dm644 "$WORK/pkw.fish" "$DEB/usr/share/fish/vendor_completions.d/pkw.fish"
install -Dm644 packaging/linux/packwing.desktop "$DEB/usr/share/applications/packwing.desktop"
install -Dm644 packaging/icons/packwing.svg "$DEB/usr/share/icons/hicolor/scalable/apps/packwing.svg"
install -Dm644 packaging/linux/copyright "$DEB/usr/share/doc/packwing/copyright"
# Those of the crates built for this machine, into the program or pkw; one
# under a license about.toml does not accept stops here.
cargo about generate --locked --fail -c about.toml -m Cargo.toml --workspace \
	--target "$ARCH-unknown-linux-gnu" -o "$WORK/THIRD-PARTY-LICENSES" packaging/licenses.hbs
install -Dm644 "$WORK/THIRD-PARTY-LICENSES" "$DEB/usr/share/doc/packwing/THIRD-PARTY-LICENSES"
# Named after packwing's menu, which is how the extension knows it.
fetch "$EXTENSION_URL" "$EXTENSION_SHA" "$WORK/extension.so"
install -Dm644 "$WORK/extension.so" "$DEB/$EXTENSIONS/libcontext-menu-packwing.so"

# The libraries the programs link, as the system names their packages. The
# extension links none of its own: Nautilus's are there when it is loaded.
mkdir -p "$WORK/shlibs/debian"
printf 'Source: packwing\n\nPackage: packwing\nArchitecture: any\n' >"$WORK/shlibs/debian/control"
DEPENDS="$(cd "$WORK/shlibs" && dpkg-shlibdeps -O "$ROOT/$DEB/usr/bin/packwing" "$ROOT/$DEB/usr/bin/pkw" 2>/dev/null | sed -n 's/^shlibs:Depends=//p')"
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
Description: File archiver that puts every core to work and opens archives safely
 Opens ZIP, RAR, 7z and tar archives without writing outside the folder
 chosen, and makes smaller files to share. Its items in the right-click
 menu of Nautilus, Dolphin, Nemo and Thunar extract and compress from any
 folder. pkw does the same on the command line, with no window, over SSH
 and in scripts.
EOF
chmod -R go-w "$DEB"
dpkg-deb --root-owner-group --build "$DEB" "$DIST/packwing_${VERSION}_${DEB_ARCH}.deb"

## The .rpm, for Fedora and openSUSE: the .deb's files where those systems
## keep them. rpmbuild works out what the programs need from them, the glibc
## version included, so a system too old for them refuses the package at
## install rather than packwing failing when it starts. The extension is
## left out of that: what it uses is Nautilus's, there when Nautilus loads it.
RPM="$WORK/rpm"
RPMROOT="$ROOT/$WORK/rpm-root"
install -Dm755 "$DEB/usr/bin/packwing" "$RPMROOT/usr/bin/packwing"
install -Dm755 "$DEB/usr/bin/pkw" "$RPMROOT/usr/bin/pkw"
for page in "$WORK"/man/*.1.gz; do
	install -Dm644 "$page" "$RPMROOT/usr/share/man/man1/$(basename "$page")"
done
install -Dm644 "$WORK/pkw.bash" "$RPMROOT/usr/share/bash-completion/completions/pkw"
install -Dm644 "$WORK/_pkw" "$RPMROOT/usr/share/zsh/site-functions/_pkw"
install -Dm644 "$WORK/pkw.fish" "$RPMROOT/usr/share/fish/vendor_completions.d/pkw.fish"
install -Dm644 packaging/linux/packwing.desktop "$RPMROOT/usr/share/applications/packwing.desktop"
install -Dm644 packaging/icons/packwing.svg "$RPMROOT/usr/share/icons/hicolor/scalable/apps/packwing.svg"
install -Dm644 LICENSE "$RPMROOT/usr/share/licenses/packwing/LICENSE"
install -Dm644 GPL-ADDITIONAL-PERMISSION.md "$RPMROOT/usr/share/licenses/packwing/GPL-ADDITIONAL-PERMISSION.md"
install -Dm644 "$WORK/THIRD-PARTY-LICENSES" "$RPMROOT/usr/share/licenses/packwing/THIRD-PARTY-LICENSES"
install -Dm644 "$WORK/extension.so" "$RPMROOT/$RPM_EXTENSIONS/libcontext-menu-packwing.so"
mkdir -p "$RPM"
cat >"$RPM/packwing.spec" <<SPEC
%global debug_package %{nil}
%global _build_id_links none
%global __brp_strip %{nil}
%global __requires_exclude_from ^/usr/lib64/nautilus/.*\$
%global __provides_exclude_from ^/usr/lib64/nautilus/.*\$
Name: packwing
# An RPM version has no hyphen; ~ sorts a beta before its release.
Version: ${VERSION/-/\~}
Release: 1
Summary: File archiver that puts every core to work and opens archives safely
License: GPL-3.0-or-later
URL: https://github.com/hivepaas/packwing
Recommends: libwayland-client, libxcb, libX11-xcb, vulkan-loader, mesa-vulkan-drivers
Enhances: nautilus

%description
Opens ZIP, RAR, 7z and tar archives without writing outside the folder
chosen, and makes smaller files to share. Its items in the right-click
menu of Nautilus, Dolphin, Nemo and Thunar extract and compress from any
folder. pkw does the same on the command line, with no window, over SSH
and in scripts.

%install
cp -a "$RPMROOT/." "%{buildroot}/"

%files
/usr/bin/packwing
/usr/bin/pkw
/usr/share/man/man1/pkw*.1*
/usr/share/bash-completion/completions/pkw
/usr/share/zsh/site-functions/_pkw
/usr/share/fish/vendor_completions.d/pkw.fish
/usr/share/applications/packwing.desktop
/usr/share/icons/hicolor/scalable/apps/packwing.svg
/$RPM_EXTENSIONS/libcontext-menu-packwing.so
%license /usr/share/licenses/packwing/LICENSE
%license /usr/share/licenses/packwing/GPL-ADDITIONAL-PERMISSION.md
%license /usr/share/licenses/packwing/THIRD-PARTY-LICENSES
SPEC
rpmbuild -bb --quiet --target "$ARCH" --define "_topdir $ROOT/$RPM" "$RPM/packwing.spec"
cp "$RPM/RPMS/$ARCH/"*.rpm "$DIST/packwing_${VERSION}_${ARCH}.rpm"

## pkw alone: the program, its pages and completions, and the licenses of
## the crates it is built with, in a folder of its own (packaging/cli).
CLI_NAME="pkw_${VERSION}_linux_${ARCH}"
CLI_DIR="$WORK/cli/$CLI_NAME"
install -Dm755 "$DEB/usr/bin/pkw" "$CLI_DIR/pkw"
mkdir -p "$CLI_DIR/man" "$CLI_DIR/completions"
cp "$WORK"/man/*.1 "$CLI_DIR/man/"
cp "$WORK/pkw.bash" "$WORK/_pkw" "$WORK/pkw.fish" "$CLI_DIR/completions/"
cp packaging/cli/README.txt "$CLI_DIR/README.txt"
cp LICENSE "$CLI_DIR/LICENSE"
cp GPL-ADDITIONAL-PERMISSION.md "$CLI_DIR/GPL-ADDITIONAL-PERMISSION.md"
cargo about generate --locked --fail -c about.toml -m crates/cli/Cargo.toml \
	--target "$ARCH-unknown-linux-gnu" -o "$CLI_DIR/THIRD-PARTY-LICENSES" packaging/licenses.hbs
tar -C "$WORK/cli" --owner=0 --group=0 --numeric-owner -czf "$DIST/$CLI_NAME.tar.gz" "$CLI_NAME"

ls -l "$DIST"
