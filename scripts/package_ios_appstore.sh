#!/bin/bash
# The iOS app for the App Store: built for a device, signed with the Apple
# Distribution certificate and the iOS App Store profile, as an .ipa; checked by
# App Store Connect and sent there with UPLOAD=1, for TestFlight and review.
#
#   VERSION=0.3.0 BUILD_NUMBER=3 IOS_PROFILE=path/to.mobileprovision \
#     IOS_IDENTITY="Apple Distribution: …" [IOS_KEYCHAIN=path] \
#     [UPLOAD=1 APPLE_API_KEY_PATH=… APPLE_API_KEY_ID=… APPLE_API_ISSUER_ID=…] \
#     scripts/package_ios_appstore.sh
#
# The app is built unsigned and signed here, as scripts/package_macos_appstore.sh
# signs the Mac's: with the identity in a keychain of its own (IOS_KEYCHAIN), or
# one in the search list, and the entitlements the profile grants.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

: "${VERSION:?the version, as 0.3.0}"
: "${BUILD_NUMBER:?a build number App Store Connect has not had for this version}"
: "${IOS_PROFILE:?the iOS App Store provisioning profile}"
: "${IOS_IDENTITY:?the Apple Distribution identity}"
BUNDLE_ID=com.hivepaas.packwing

OUT="$ROOT/target/ios-app-store"
rm -rf "$OUT"
mkdir -p "$OUT"
KEYCHAIN=()
[ -z "${IOS_KEYCHAIN:-}" ] || KEYCHAIN=(--keychain "$IOS_KEYCHAIN")

## Built: the core for a device, and the app, unsigned
scripts/build_ios.sh
(cd ios && xcodegen generate --quiet)
xcodebuild archive -project ios/Packwing.xcodeproj -scheme Packwing -configuration Release \
	-destination generic/platform=iOS -archivePath "$OUT/Packwing.xcarchive" \
	-derivedDataPath "$ROOT/target/ios-release" \
	MARKETING_VERSION="$VERSION" CURRENT_PROJECT_VERSION="$BUILD_NUMBER" \
	CODE_SIGNING_ALLOWED=NO CODE_SIGNING_REQUIRED=NO CODE_SIGN_IDENTITY="" -quiet

APP="$OUT/Payload/Packwing.app"
mkdir -p "$OUT/Payload"
cp -R "$OUT/Packwing.xcarchive/Products/Applications/Packwing.app" "$APP"

## Signed, as the profile allows
cp "$IOS_PROFILE" "$APP/embedded.mobileprovision"
security cms -D -i "$IOS_PROFILE" > "$OUT/profile.plist"
/usr/libexec/PlistBuddy -x -c 'Print :Entitlements' "$OUT/profile.plist" > "$OUT/entitlements.plist"
# The profile grants the team's keychain groups by a wildcard, as Xcode never
# signs: packwing keeps its secrets in its own group, the app's identifier,
# which needs no entitlement.
/usr/libexec/PlistBuddy -c 'Delete :keychain-access-groups' "$OUT/entitlements.plist" 2>/dev/null || true
PROFILE_APP=$(/usr/libexec/PlistBuddy -c 'Print :Entitlements:application-identifier' "$OUT/profile.plist")
case "$PROFILE_APP" in
*."$BUNDLE_ID") ;;
*) echo "the profile is for $PROFILE_APP, not $BUNDLE_ID" >&2; exit 1 ;;
esac
# Anything nested is signed first: frameworks and their kind.
find "$APP" -depth \( -name '*.framework' -o -name '*.dylib' -o -name '*.appex' \) -print0 |
	while IFS= read -r -d '' nested; do
		codesign --force --sign "$IOS_IDENTITY" ${KEYCHAIN[@]+"${KEYCHAIN[@]}"} "$nested"
	done
codesign --force --sign "$IOS_IDENTITY" ${KEYCHAIN[@]+"${KEYCHAIN[@]}"} \
	--entitlements "$OUT/entitlements.plist" "$APP"
codesign --verify --strict --verbose=2 "$APP"

IPA="$OUT/Packwing-$VERSION-$BUILD_NUMBER.ipa"
(cd "$OUT" && zip -qry "$IPA" Payload)

## Sent to App Store Connect, checked by it first
if [ -n "${UPLOAD:-}" ]; then
	# altool reads the key as AuthKey_<key id>.p8 in API_PRIVATE_KEYS_DIR.
	KEYS="$(mktemp -d)"
	trap 'rm -rf "$KEYS"' EXIT
	cp "$APPLE_API_KEY_PATH" "$KEYS/AuthKey_$APPLE_API_KEY_ID.p8"
	export API_PRIVATE_KEYS_DIR="$KEYS"
	AUTH=(--apiKey "$APPLE_API_KEY_ID" --apiIssuer "$APPLE_API_ISSUER_ID")
	xcrun altool --validate-app -f "$IPA" -t ios "${AUTH[@]}"
	xcrun altool --upload-app -f "$IPA" -t ios "${AUTH[@]}"
fi

ls -l "$IPA"
