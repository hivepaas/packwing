#!/bin/bash
#
# Packages a release build of packwing for macOS into dist/, as shellwing's
# script does: Packwing.app, tarred for the updater, and in a disk image for
# people (docs/releasing.md).
#
#   cargo build --release -p packwing -p packwing-cli
#   scripts/package_macos.sh
#
#   dist/packwing_0.1.0_macos_aarch64.app.tar.gz
#   dist/packwing_0.1.0_macos_aarch64.dmg
#
# The app holds pkw, the command line, beside packwing, and context-menu-rs's
# native Finder extension, which puts packwing's items in Finder's own
# right-click menu (crates/app/src/right_click.rs): the extension, and its
# helper that runs packwing, are the same program, downloaded from the
# release of the context-menu crate packwing builds with and checked against
# its SHA-256. Their Info.plist and entitlements are context-menu-rs's
# (packaging/macos/ContextMenu, which a test holds to the crate's), with the
# team and the version put in here.
#
# The extension's app group starts with the team's id, so it is signed with a
# Developer ID only. Without one the app is signed ad hoc and holds no
# extension: its items are then Quick Actions, as from `cargo run`, and
# Gatekeeper asks the first time the app opens (Control-click > Open).
#
# The licenses of the Rust crates packwing is built with go in the app's
# Resources, written by cargo-about (about.toml, packaging/licenses.hbs):
#
#   cargo install cargo-about --locked --features cli
#
# Environment: BIN (default target/release/packwing), CLI (default
# target/release/pkw), ARCH (default the machine's: aarch64 or x86_64), DIST
# (default dist), EXTENSION (the Finder extension; default: downloaded and
# checked). To sign for people (docs/releasing.md):
#
#   MACOS_SIGN_IDENTITY         "Developer ID Application: Name (TEAMID)", in a
#                               keychain this user can use; none: ad hoc
#   APPLE_API_KEY_PATH, APPLE_API_KEY_ID, APPLE_API_ISSUER_ID
#                               an App Store Connect API key, to notarize

set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

VERSION="$(sed -n '/^\[workspace.package\]/,/^\[/s/^version = "\(.*\)"/\1/p' Cargo.toml)"
ARCH="${ARCH:-$(uname -m | sed 's/arm64/aarch64/')}"
BIN="${BIN:-target/release/packwing}"
CLI="${CLI:-target/release/pkw}"
DIST="${DIST:-dist}"
WORK="target/package-macos"

# context-menu-rs 0.2.1's Finder extension: one program for Apple silicon and Intel.
CONTEXT_MENU=0.2.1
EXTENSION_URL="https://github.com/hivepaas/context-menu-rs/releases/download/v$CONTEXT_MENU/context-menu-finder-macos"
EXTENSION_SHA=ca46083dcfddf02d292e95ab12fef95f9bfa8ff725c9d1e77ba4d2ed93bb2285

case "$ARCH" in
aarch64) TARGET=aarch64-apple-darwin ;;
x86_64) TARGET=x86_64-apple-darwin ;;
*) echo "no macOS package for $ARCH" >&2; exit 1 ;;
esac
[ -x "$BIN" ] || { echo "no build at $BIN: cargo build --release -p packwing" >&2; exit 1; }
[ -x "$CLI" ] || { echo "no build at $CLI: cargo build --release -p packwing-cli" >&2; exit 1; }
cargo about --version >/dev/null 2>&1 || { echo "no cargo-about: cargo install cargo-about --locked --features cli" >&2; exit 1; }
LOCKED="$(sed -n '/^name = "context-menu"$/{n;s/^version = "\(.*\)"/\1/p;}' Cargo.lock)"
[ "$LOCKED" = "$CONTEXT_MENU" ] ||
	{ echo "Cargo.lock's context-menu is ${LOCKED:-missing}, not $CONTEXT_MENU: pin its extension here" >&2; exit 1; }
rm -rf "$WORK"
mkdir -p "$WORK" "$DIST"

IDENTITY="${MACOS_SIGN_IDENTITY:-}"
TEAM=""
if [ -n "$IDENTITY" ]; then
	TEAM="$(sed -n 's/.*(\([A-Z0-9]\{10\}\))$/\1/p' <<<"$IDENTITY")"
	[ -n "$TEAM" ] || { echo "no team id at the end of $IDENTITY" >&2; exit 1; }
fi

## The icon, from the PNGs
ICONSET="$WORK/packwing.iconset"
mkdir -p "$ICONSET"
for size in 16 32 128 256 512; do
	cp "packaging/icons/png/packwing-$size.png" "$ICONSET/icon_${size}x${size}.png"
	cp "packaging/icons/png/packwing-$((size * 2)).png" "$ICONSET/icon_${size}x${size}@2x.png"
done
iconutil -c icns -o "$WORK/packwing.icns" "$ICONSET"

## The third-party licenses, of the crates built for this Mac
cargo about generate --locked --fail -c about.toml -m Cargo.toml --workspace \
	--target "$TARGET" -o "$WORK/THIRD-PARTY-LICENSES.txt" packaging/licenses.hbs

## Packwing.app
APP="$WORK/Packwing.app"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$BIN" "$APP/Contents/MacOS/packwing"
cp "$CLI" "$APP/Contents/MacOS/pkw"
strip -x "$APP/Contents/MacOS/packwing" "$APP/Contents/MacOS/pkw"
cp "$WORK/packwing.icns" "$APP/Contents/Resources/packwing.icns"
cp LICENSE "$APP/Contents/Resources/LICENSE.txt"
cp GPL-ADDITIONAL-PERMISSION.md "$APP/Contents/Resources/GPL-ADDITIONAL-PERMISSION.txt"
cp "$WORK/THIRD-PARTY-LICENSES.txt" "$APP/Contents/Resources/THIRD-PARTY-LICENSES.txt"
sed "s/@VERSION@/$VERSION/g" packaging/macos/Info.plist >"$APP/Contents/Info.plist"
plutil -lint "$APP/Contents/Info.plist"

## The Finder extension and its helper, with a Developer ID only
EXTENSION_DIR="$APP/Contents/PlugIns/ContextMenu.appex"
HELPER_DIR="$APP/Contents/Helpers/ContextMenuHelper.app"
fill() { sed -e "s/@TEAM@/$TEAM/g" -e "s/@VERSION@/$VERSION/g" "packaging/macos/ContextMenu/$1"; }
if [ -n "$IDENTITY" ]; then
	if [ -z "${EXTENSION:-}" ]; then
		EXTENSION="$WORK/context-menu-finder"
		curl -fsSL --retry 5 --retry-all-errors --retry-delay 10 -o "$EXTENSION" "$EXTENSION_URL"
		echo "$EXTENSION_SHA  $EXTENSION" | shasum -a 256 -c --quiet
	fi
	for bundle in "$EXTENSION_DIR" "$HELPER_DIR"; do
		mkdir -p "$bundle/Contents/MacOS"
		cp "$EXTENSION" "$bundle/Contents/MacOS/context-menu-finder"
		chmod 755 "$bundle/Contents/MacOS/context-menu-finder"
	done
	fill extension.plist >"$EXTENSION_DIR/Contents/Info.plist"
	fill helper.plist >"$HELPER_DIR/Contents/Info.plist"
	fill extension.entitlements >"$WORK/extension.entitlements"
	fill helper.entitlements >"$WORK/helper.entitlements"
	plutil -lint "$EXTENSION_DIR/Contents/Info.plist" "$HELPER_DIR/Contents/Info.plist" \
		"$WORK/extension.entitlements" "$WORK/helper.entitlements"
else
	echo "warning: no MACOS_SIGN_IDENTITY: signed ad hoc, with no Finder extension (Quick Actions instead)" >&2
fi

## Signed inside out: what a bundle holds before the bundle
if [ -z "$IDENTITY" ]; then
	codesign --force --sign - --timestamp=none "$APP/Contents/MacOS/pkw"
	codesign --force --sign - --timestamp=none "$APP"
else
	# The hardened runtime and a secure timestamp, as notarizing requires.
	SIGN=(codesign --force --sign "$IDENTITY" --options runtime --timestamp)
	"${SIGN[@]}" --entitlements "$WORK/helper.entitlements" "$HELPER_DIR"
	"${SIGN[@]}" --entitlements "$WORK/extension.entitlements" "$EXTENSION_DIR"
	"${SIGN[@]}" "$APP/Contents/MacOS/pkw"
	"${SIGN[@]}" "$APP"
fi
codesign --verify --deep --strict "$APP"

# Apple checks what it is given and keeps a ticket for it, stapled here so
# Gatekeeper lets it open offline too.
notarize() {
	local file="$1" result status id
	result="$(xcrun notarytool submit "$file" --wait --output-format json \
		--key "$APPLE_API_KEY_PATH" --key-id "$APPLE_API_KEY_ID" --issuer "$APPLE_API_ISSUER_ID")"
	status="$(plutil -extract status raw - <<<"$result" 2>/dev/null || true)"
	if [ "$status" != "Accepted" ]; then
		echo "notarizing $file: ${status:-no answer}" >&2
		id="$(plutil -extract id raw - <<<"$result" 2>/dev/null || true)"
		[ -z "$id" ] || xcrun notarytool log "$id" \
			--key "$APPLE_API_KEY_PATH" --key-id "$APPLE_API_KEY_ID" --issuer "$APPLE_API_ISSUER_ID" >&2
		exit 1
	fi
}
# Stapling fetches the ticket from Apple's CloudKit, and a dropped connection
# there is no reason to lose a build that Apple has already accepted.
staple() {
	local try
	for try in 1 2 3 4 5; do
		if xcrun stapler staple "$1"; then
			return 0
		fi
		echo "stapling $1 failed (try $try of 5)" >&2
		[ "$try" -lt 5 ] && sleep 30
	done
	return 1
}
NOTARIZE=""
if [ -n "$IDENTITY" ] && [ -n "${APPLE_API_KEY_PATH:-}" ]; then
	NOTARIZE=1
elif [ -n "$IDENTITY" ]; then
	echo "warning: not notarized (no APPLE_API_KEY_PATH): Gatekeeper refuses it downloaded" >&2
fi

## Notarized, the app first: the disk image and the updater's tarball carry
## it with its ticket.
if [ -n "$NOTARIZE" ]; then
	ditto -c -k --keepParent "$APP" "$WORK/Packwing.zip"
	notarize "$WORK/Packwing.zip"
	staple "$APP"
	spctl --assess --type execute --verbose=2 "$APP"
fi

## For people: a disk image with the app and a link to /Applications
STAGE="$WORK/dmg"
mkdir -p "$STAGE"
cp -R "$APP" "$STAGE/"
ln -s /Applications "$STAGE/Applications"
DMG="$DIST/packwing_${VERSION}_macos_${ARCH}.dmg"
hdiutil create -volname "Packwing $VERSION" -srcfolder "$STAGE" -fs HFS+ -format UDZO -ov "$DMG"

## The disk image signed, and notarized too.
if [ -n "$IDENTITY" ]; then
	codesign --force --sign "$IDENTITY" --timestamp "$DMG"
fi
if [ -n "$NOTARIZE" ]; then
	notarize "$DMG"
	staple "$DMG"
fi

## For the updater: the .app, tarred
tar -C "$WORK" -czf "$DIST/packwing_${VERSION}_macos_${ARCH}.app.tar.gz" Packwing.app

ls -l "$DIST"
