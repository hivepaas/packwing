#!/bin/bash
#
# Packages a release build of packwing for Windows into dist/, as shellwing's
# script does: the installer, which the updater runs quietly over the installed
# app, and a zip of the program to run from a folder. Run from Git Bash or
# MSYS2, with NSIS 3 installed (or makensis on PATH; on Linux the nsis
# package's works too).
#
#   cargo build --release -p packwing -p packwing-cli
#   scripts/package_windows.sh
#
#   dist/packwing_0.1.0_windows_x86_64-setup.exe
#   dist/packwing_0.1.0_windows_x86_64.zip
#   dist/pkw_0.1.0_windows_x86_64.zip
#
# The installer holds context-menu-rs's native extension for Windows 11's own
# right-click menu, and the package that declares it (packaging/windows): see
# installer.nsi. Neither is signed (no Authenticode certificate yet):
# SmartScreen asks the first time the installer runs (More info > Run anyway),
# and Windows takes the package only where Developer Mode is on.
#
# Both hold pkw.exe, the command line, beside packwing.exe; the installer puts
# its folder in the user's PATH. pkw.exe also comes alone, in a zip with the
# licenses of the crates it is built with (packaging/cli).
#
# Environment: EXE (default target/release/packwing.exe), CLI (default
# target/release/pkw.exe), ARCH (default
# x86_64), DIST (default dist), MAKENSIS (default: on PATH, else where NSIS
# installs itself), EXTENSION (the native extension; default: downloaded from
# context-menu-rs's release, its checksum checked), LICENSES and CLI_LICENSES
# (the third-party licenses of the app and pkw, and of pkw alone; default: made
# by cargo-about).

set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

VERSION="$(sed -n '/^\[workspace.package\]/,/^\[/s/^version = "\(.*\)"/\1/p' Cargo.toml)"
ARCH="${ARCH:-x86_64}"
EXE="$(realpath "${EXE:-target/release/packwing.exe}")"
CLI="$(realpath "${CLI:-target/release/pkw.exe}")"
DIST="${DIST:-dist}"
WORK="target/package-windows"

# context-menu-rs's native extension for Windows, of the release whose crate
# packwing builds with (Cargo.lock): the menu's definition is theirs alike.
CONTEXT_MENU_VERSION="0.2.1"
case "$ARCH" in
	x86_64) EXTENSION_SHA="557b1126fec515de47870b3aaf946624ecfbf6d4ecddf732bb73c9c78f1f7bf9" ;;
	*) echo "no native extension pinned for $ARCH" >&2; exit 1 ;;
esac
EXTENSION_URL="https://github.com/hivepaas/context-menu-rs/releases/download/v$CONTEXT_MENU_VERSION/context-menu-windows-$ARCH.dll"

if [ -z "${MAKENSIS:-}" ]; then
	MAKENSIS="$(command -v makensis || true)"
	for dir in "/c/Program Files (x86)/NSIS" "/c/Program Files/NSIS"; do
		[ -n "$MAKENSIS" ] || { [ -x "$dir/makensis.exe" ] && MAKENSIS="$dir/makensis.exe"; } || true
	done
fi
[ -n "$MAKENSIS" ] || { echo "no makensis: install NSIS 3 (choco install nsis)" >&2; exit 1; }
[ -f "$EXE" ] || { echo "no build at $EXE: cargo build --release -p packwing" >&2; exit 1; }
[ -f "$CLI" ] || { echo "no build at $CLI: cargo build --release -p packwing-cli" >&2; exit 1; }
grep -q "^name = \"context-menu\"\$" Cargo.lock &&
	grep -A1 "^name = \"context-menu\"\$" Cargo.lock | grep -q "^version = \"$CONTEXT_MENU_VERSION\"\$" ||
	{ echo "Cargo.lock's context-menu is not $CONTEXT_MENU_VERSION: pin its extension here" >&2; exit 1; }
rm -rf "$WORK"
mkdir -p "$WORK" "$DIST"
DIST="$(realpath "$DIST")"

# A path as the tool takes it: Windows' own under MSYS, as is elsewhere.
native() { if command -v cygpath >/dev/null; then cygpath -w "$1"; else echo "$1"; fi; }

## The native extension
if [ -z "${EXTENSION:-}" ]; then
	EXTENSION="$WORK/context-menu-windows-$ARCH.dll"
	curl -fsSL --retry 5 --retry-all-errors --retry-delay 10 -o "$EXTENSION" "$EXTENSION_URL"
	echo "$EXTENSION_SHA  $EXTENSION" | sha256sum -c --quiet
fi
EXTENSION="$(realpath "$EXTENSION")"

## The third-party licenses, of the crates built for Windows, into the
## program or pkw
if [ -z "${LICENSES:-}" ]; then
	cargo about --version >/dev/null 2>&1 || { echo "no cargo-about: cargo install cargo-about --locked --features cli" >&2; exit 1; }
	LICENSES="$WORK/THIRD-PARTY-LICENSES.txt"
	cargo about generate --locked --fail -c about.toml -m Cargo.toml --workspace \
		--target "$ARCH-pc-windows-msvc" -o "$LICENSES" packaging/licenses.hbs
fi
LICENSES="$(realpath "$LICENSES")"

## The package's manifest, with the version as Windows writes it
FILEVERSION="${VERSION%%-*}.0"
MANIFEST="$ROOT/$WORK/AppxManifest.xml"
sed "s/@VERSION@/$FILEVERSION/" packaging/windows/AppxManifest.xml >"$MANIFEST"

## The installer
SETUP="$DIST/packwing_${VERSION}_windows_${ARCH}-setup.exe"
# The paths are Windows' already: MSYS is not to rewrite them.
MSYS2_ARG_CONV_EXCL='*' "$MAKENSIS" -V3 \
	"-DVERSION=$VERSION" "-DFILEVERSION=$FILEVERSION" \
	"-DEXE=$(native "$EXE")" "-DCLI=$(native "$CLI")" "-DEXTENSION=$(native "$EXTENSION")" \
	"-DMANIFEST=$(native "$MANIFEST")" "-DLICENSES=$(native "$LICENSES")" \
	"-DOUT=$(native "$SETUP")" \
	"$(native "$ROOT/packaging/windows/installer.nsi")"

## The zip: the program, pkw and their licenses, in a folder of their own.
## Run from there, packwing's menu is the classic one, and pkw is where it is.
NAME="packwing-${VERSION}-windows-${ARCH}"
mkdir -p "$WORK/$NAME"
cp "$EXE" "$WORK/$NAME/packwing.exe"
cp "$CLI" "$WORK/$NAME/pkw.exe"
cp LICENSE "$WORK/$NAME/LICENSE.txt"
cp GPL-ADDITIONAL-PERMISSION.md "$WORK/$NAME/GPL-ADDITIONAL-PERMISSION.txt"
cp "$LICENSES" "$WORK/$NAME/THIRD-PARTY-LICENSES.txt"
# A folder in $WORK, zipped into $DIST.
zip_dir() {
	local zip="$DIST/$1.zip"
	rm -f "$zip"
	if command -v 7z >/dev/null; then
		(cd "$WORK" && 7z a -tzip -bso0 "$(native "$zip")" "$2")
	else
		(cd "$WORK" && zip -qr "$zip" "$2")
	fi
}
zip_dir "packwing_${VERSION}_windows_${ARCH}" "$NAME"

## pkw alone, with its licenses
CLI_NAME="pkw_${VERSION}_windows_${ARCH}"
mkdir -p "$WORK/$CLI_NAME"
cp "$CLI" "$WORK/$CLI_NAME/pkw.exe"
cp packaging/cli/README.txt "$WORK/$CLI_NAME/README.txt"
cp LICENSE "$WORK/$CLI_NAME/LICENSE.txt"
cp GPL-ADDITIONAL-PERMISSION.md "$WORK/$CLI_NAME/GPL-ADDITIONAL-PERMISSION.txt"
if [ -z "${CLI_LICENSES:-}" ]; then
	CLI_LICENSES="$WORK/$CLI_NAME/THIRD-PARTY-LICENSES.txt"
	cargo about generate --locked --fail -c about.toml -m crates/cli/Cargo.toml \
		--target "$ARCH-pc-windows-msvc" -o "$CLI_LICENSES" packaging/licenses.hbs
else
	cp "$CLI_LICENSES" "$WORK/$CLI_NAME/THIRD-PARTY-LICENSES.txt"
fi
zip_dir "$CLI_NAME" "$CLI_NAME"

ls -l "$DIST"
