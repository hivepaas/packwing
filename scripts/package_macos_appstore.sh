#!/bin/bash
#
# Packages packwing for the Mac App Store (docs/building.md, "The Mac App
# Store's build"): one Packwing.app for Apple silicon and Intel, built with the
# app-store feature, in the App Sandbox, in an installer package App Store
# Connect takes; and, asked to, sends it there, for TestFlight and review.
#
#   scripts/package_macos_appstore.sh
#
#   dist/packwing_0.1.0_macos_appstore.pkg
#
# Unlike scripts/package_macos.sh's, this app holds packwing alone: no pkw (a
# command line has no place in the store's app), no Finder extension, and no
# updater, as the App Store updates it. It has the store's entitlements
# (packaging/macos/app-store.entitlements, with the app's id and team added),
# its provisioning profile, and a privacy manifest
# (packaging/macos/PrivacyInfo.xcprivacy).
#
# It builds both programs (cargo, for aarch64-apple-darwin and
# x86_64-apple-darwin) unless BIN_AARCH64 and BIN_X86_64 name them, built with
# `--features app-store`. Neither may call what Apple keeps private: the
# script looks, and stops.
#
# Environment: ARCHES (default "aarch64 x86_64"), BUILD_NUMBER (the bundle's
# CFBundleVersion, which must grow with every upload: default the number of
# commits on HEAD), DIST (default dist). To sign for the store
# (docs/releasing.md):
#
#   MAS_APP_IDENTITY        "Apple Distribution: Name (TEAMID)"
#   MAS_INSTALLER_IDENTITY  "3rd Party Mac Developer Installer: Name (TEAMID)"
#   MAS_PROFILE             the "Mac App Store" provisioning profile's file
#
# in a keychain this user can use: MAS_KEYCHAIN, or one in the search list.
# Without them the app is signed ad hoc,
# with the same entitlements, and the package is not signed: enough to check
# what is in it, not to send it. To send it, UPLOAD=1 and an App Store Connect
# API key: APPLE_API_KEY_PATH, APPLE_API_KEY_ID, APPLE_API_ISSUER_ID.

set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

VERSION="$(sed -n '/^\[workspace.package\]/,/^\[/s/^version = "\(.*\)"/\1/p' Cargo.toml)"
BUILD_NUMBER="${BUILD_NUMBER:-$(git rev-list --count HEAD)}"
ARCHES="${ARCHES:-aarch64 x86_64}"
DIST="${DIST:-dist}"
WORK="target/package-macos-appstore"
BUNDLE_ID="$(plutil -extract CFBundleIdentifier raw packaging/macos/Info.plist)"

[[ "$BUILD_NUMBER" =~ ^[0-9]+(\.[0-9]+){0,2}$ ]] ||
	{ echo "BUILD_NUMBER must be up to three numbers with dots, not $BUILD_NUMBER" >&2; exit 1; }
cargo about --version >/dev/null 2>&1 || { echo "no cargo-about: cargo install cargo-about --locked --features cli" >&2; exit 1; }

APP_IDENTITY="${MAS_APP_IDENTITY:-}"
KEYCHAIN=()
[ -z "${MAS_KEYCHAIN:-}" ] || KEYCHAIN=(--keychain "$MAS_KEYCHAIN")
INSTALLER_IDENTITY="${MAS_INSTALLER_IDENTITY:-}"
PROFILE="${MAS_PROFILE:-}"
TEAM=""
if [ -n "$APP_IDENTITY" ]; then
	[ -n "$INSTALLER_IDENTITY" ] && [ -f "$PROFILE" ] ||
		{ echo "MAS_APP_IDENTITY needs MAS_INSTALLER_IDENTITY and MAS_PROFILE too" >&2; exit 1; }
	TEAM="$(sed -n 's/.*(\([A-Z0-9]\{10\}\))$/\1/p' <<<"$APP_IDENTITY")"
	[ -n "$TEAM" ] || { echo "no team id at the end of $APP_IDENTITY" >&2; exit 1; }
elif [ -n "${UPLOAD:-}" ]; then
	echo "UPLOAD needs MAS_APP_IDENTITY, MAS_INSTALLER_IDENTITY and MAS_PROFILE" >&2
	exit 1
fi
rm -rf "$WORK"
mkdir -p "$WORK" "$DIST"

## The programs, one for each architecture, then one for both
TARGETS=()
PROGRAMS=()
for arch in $ARCHES; do
	target="$arch-apple-darwin"
	TARGETS+=("$target")
	case "$arch" in
	aarch64) bin="${BIN_AARCH64:-}" ;;
	x86_64) bin="${BIN_X86_64:-}" ;;
	*) echo "no Mac App Store build for $arch" >&2; exit 1 ;;
	esac
	if [ -z "$bin" ]; then
		cargo build --release --locked -p packwing --features app-store --target "$target"
		bin="target/$target/release/packwing"
	fi
	[ -x "$bin" ] || { echo "no build at $bin" >&2; exit 1; }
	PROGRAMS+=("$bin")
done
lipo -create -output "$WORK/packwing" "${PROGRAMS[@]}"
strip -x "$WORK/packwing"
lipo -info "$WORK/packwing"

## What the store would refuse: Apple's private frameworks and functions
## (the store's review looks for them too), and a build that is not the
## store's: it would look for updates and offer what the sandbox forbids.
if otool -L "$WORK/packwing" | grep -q '/PrivateFrameworks/'; then
	otool -L "$WORK/packwing" | grep '/PrivateFrameworks/' >&2
	echo "packwing links a private framework" >&2
	exit 1
fi
# CoreGraphics' and SkyLight's window server calls, and CoreFoundation's
# underscored ones, are Apple's own.
PRIVATE="$(nm -u -arch all "$WORK/packwing" | grep -E '^_(CGS|SLS|_CFX|CGSPrivate)' | sort -u || true)"
if [ -n "$PRIVATE" ]; then
	echo "$PRIVATE" >&2
	echo "packwing calls functions Apple keeps private" >&2
	exit 1
fi
# AppKit's private methods and classes, named in the program for the
# Objective-C runtime: the selectors with an underscore and a word after it,
# and the classes AppKit does not publish.
PRIVATE="$(strings -a "$WORK/packwing" | grep -oE '^_[a-z][A-Za-z]{7,}:?$|NSWindowRestorationOptions' | sort -u || true)"
if [ -n "$PRIVATE" ]; then
	echo "$PRIVATE" >&2
	echo "packwing names AppKit's private methods or classes" >&2
	exit 1
fi
# Only the app-store feature links StoreKit (crates/store).
otool -L "$WORK/packwing" | grep -q '/StoreKit.framework/' ||
	{ echo "packwing was not built with --features app-store" >&2; exit 1; }

## The icon, from the PNGs
ICONSET="$WORK/packwing.iconset"
mkdir -p "$ICONSET"
for size in 16 32 128 256 512; do
	cp "packaging/icons/png/packwing-$size.png" "$ICONSET/icon_${size}x${size}.png"
	cp "packaging/icons/png/packwing-$((size * 2)).png" "$ICONSET/icon_${size}x${size}@2x.png"
done
iconutil -c icns -o "$WORK/packwing.icns" "$ICONSET"

## The third-party licenses, of the crates built for these Macs
ABOUT_TARGETS=()
for target in "${TARGETS[@]}"; do
	ABOUT_TARGETS+=(--target "$target")
done
cargo about generate --locked --fail -c about.toml -m Cargo.toml --workspace \
	"${ABOUT_TARGETS[@]}" -o "$WORK/THIRD-PARTY-LICENSES.txt" packaging/licenses.hbs

## Packwing.app
APP="$WORK/Packwing.app"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$WORK/packwing" "$APP/Contents/MacOS/packwing"
cp "$WORK/packwing.icns" "$APP/Contents/Resources/packwing.icns"
cp packaging/macos/PrivacyInfo.xcprivacy "$APP/Contents/Resources/PrivacyInfo.xcprivacy"
cp LICENSE "$APP/Contents/Resources/LICENSE.txt"
cp GPL-ADDITIONAL-PERMISSION.md "$APP/Contents/Resources/GPL-ADDITIONAL-PERMISSION.txt"
cp "$WORK/THIRD-PARTY-LICENSES.txt" "$APP/Contents/Resources/THIRD-PARTY-LICENSES.txt"
sed "s/@VERSION@/$VERSION/g" packaging/macos/Info.plist >"$APP/Contents/Info.plist"
# The store tells uploads of one version apart by their build number.
plutil -replace CFBundleVersion -string "$BUILD_NUMBER" "$APP/Contents/Info.plist"
# Export compliance, answered here rather than for each upload: packwing
# implements standard algorithms (AES for archives, TLS through rustls,
# Ed25519 and ML-DSA for signing), and the store does not offer it in France,
# so App Store Connect asks for no documents. Offered in France, it would need
# France's declaration, and this to say YES with its compliance code.
plutil -replace ITSAppUsesNonExemptEncryption -bool NO "$APP/Contents/Info.plist"
plutil -lint "$APP/Contents/Info.plist" packaging/macos/PrivacyInfo.xcprivacy

## The entitlements: the sandbox's, and, for the store, the app's id and team,
## as its profile has them.
ENTITLEMENTS="$WORK/packwing.entitlements"
cp packaging/macos/app-store.entitlements "$ENTITLEMENTS"
if [ -n "$TEAM" ]; then
	cp "$PROFILE" "$APP/Contents/embedded.provisionprofile"
	security cms -D -i "$PROFILE" >"$WORK/profile.plist"
	PROFILE_APP="$(plutil -extract Entitlements.com\\.apple\\.application-identifier raw "$WORK/profile.plist")"
	[ "$PROFILE_APP" = "$TEAM.$BUNDLE_ID" ] ||
		{ echo "MAS_PROFILE is for $PROFILE_APP, not $TEAM.$BUNDLE_ID" >&2; exit 1; }
	plutil -insert com\\.apple\\.application-identifier -string "$TEAM.$BUNDLE_ID" "$ENTITLEMENTS"
	plutil -insert com\\.apple\\.developer\\.team-identifier -string "$TEAM" "$ENTITLEMENTS"
fi
plutil -lint "$ENTITLEMENTS"

## Signed, then checked: sandboxed, as the store requires
if [ -n "$APP_IDENTITY" ]; then
	codesign --force --sign "$APP_IDENTITY" ${KEYCHAIN[@]+"${KEYCHAIN[@]}"} --options runtime --entitlements "$ENTITLEMENTS" "$APP"
else
	echo "warning: no MAS_APP_IDENTITY: signed ad hoc, and the package unsigned; the store takes neither" >&2
	codesign --force --sign - --options runtime --timestamp=none --entitlements "$ENTITLEMENTS" "$APP"
fi
codesign --verify --deep --strict "$APP"
codesign -d --entitlements - --xml "$APP" 2>/dev/null >"$WORK/signed.entitlements"
[ "$(plutil -extract com\\.apple\\.security\\.app-sandbox raw "$WORK/signed.entitlements")" = true ] ||
	{ echo "Packwing.app is not sandboxed" >&2; exit 1; }

## The installer package, which installs Packwing.app in /Applications
PKG="$DIST/packwing_${VERSION}_macos_appstore.pkg"
if [ -n "$INSTALLER_IDENTITY" ]; then
	productbuild --component "$APP" /Applications --sign "$INSTALLER_IDENTITY" ${KEYCHAIN[@]+"${KEYCHAIN[@]}"} "$PKG"
	pkgutil --check-signature "$PKG"
else
	productbuild --component "$APP" /Applications "$PKG"
fi

## Sent to App Store Connect, checked by it first
if [ -n "${UPLOAD:-}" ]; then
	AUTH=(--api-key "$APPLE_API_KEY_ID" --api-issuer "$APPLE_API_ISSUER_ID" --p8-file-path "$APPLE_API_KEY_PATH")
	xcrun altool --validate-app "$PKG" --type macos "${AUTH[@]}"
	xcrun altool --upload-package "$PKG" "${AUTH[@]}"
fi

ls -l "$PKG"
