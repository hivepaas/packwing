#!/bin/bash
#
# Runs the Mac App Store's build of packwing in the App Sandbox, as it will run
# from the store: a debug build with the app-store feature, in a bundle signed
# ad hoc with packaging/macos/app-store.entitlements. Its bundle id is
# com.hivepaas.packwing.sandbox-dev, so its container
# (~/Library/Containers/com.hivepaas.packwing.sandbox-dev) is not the store's.
#
#   scripts/run_sandboxed.sh [ARCHIVE…]
#
# The archives are opened as Finder opens them, which gives packwing those files
# and nothing beside them. A bookmark kept by one build may not open in the next:
# each ad hoc signature is another app to macOS.

set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"
[ "$(uname -s)" = Darwin ] || { echo "the App Sandbox is macOS's" >&2; exit 1; }

cargo build -p packwing --features app-store
APP="target/sandbox/Packwing.app"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS"
cp target/debug/packwing "$APP/Contents/MacOS/packwing"
sed -e "s/@VERSION@/0.0.0/g" \
	-e "s|<string>com.hivepaas.packwing</string>|<string>com.hivepaas.packwing.sandbox-dev</string>|" \
	packaging/macos/Info.plist >"$APP/Contents/Info.plist"
codesign --force --sign - --entitlements packaging/macos/app-store.entitlements "$APP"
# What Finder runs, and so offers in Open With.
/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -f "$APP"

paths=()
for p in "$@"; do
	paths+=("$(cd "$(dirname "$p")" && pwd)/$(basename "$p")")
done
echo "packwing's messages: target/sandbox/packwing.log" >&2
open -n --stdout target/sandbox/packwing.log --stderr target/sandbox/packwing.log -a "$PWD/$APP" ${paths[@]+"${paths[@]}"}
