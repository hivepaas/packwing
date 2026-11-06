# Releasing

A release is a tag, the files CI builds for it, and an entry in the signed list
of releases the app reads ([updates.md](updates.md)). Until the list names it, no
app is told of it.

## What a release ships

| Platform | Files | Made by |
| --- | --- | --- |
| Linux x86_64, aarch64 | `packwing_<v>_<amd64\|arm64>.deb` | `scripts/package_linux.sh` |
| macOS aarch64, x86_64 | `packwing_<v>_macos_<arch>.dmg`, `.app.tar.gz` | `scripts/package_macos.sh` |
| Windows x86_64 | `packwing_<v>_windows_x86_64-setup.exe`, `.zip` | `scripts/package_windows.sh` |

and `SHA256SUMS`. The updater installs the `.app.tar.gz` and the installer, and
shows the commands that install the `.deb`; the others are for people.

- **Linux**: the `.deb` is built on Ubuntu 22.04, so it runs on glibc 2.35 and
  newer, and holds context-menu-rs's Nautilus extension.
- **macOS**: `Packwing.app`, with `pkw` beside the program and context-menu-rs's
  Finder extension and its helper, which put packwing's items in Finder's own
  right-click menu. Signed with the Developer ID and notarized when the
  repository holds one ([Signing for macOS](#signing-for-macos)). Without one it
  is signed ad hoc and holds no extension (the items are then Quick Actions),
  and Gatekeeper asks the first time it opens. The updater accepts a new `.app`
  whose signature verifies, by the team that signed the running one.
- **Windows**: an NSIS installer, for the user, into
  `%LOCALAPPDATA%\Programs\packwing` (where the updater looks for it), with
  `pkw.exe` in the user's PATH and context-menu-rs's extension for Windows 11's
  menu. It is not signed: SmartScreen asks the first time (More info › Run
  anyway), and Windows takes the extension's package only with Developer Mode on.

## Signing for macOS

As shellwing's ([its releasing.md](https://github.com/hivepaas/shellwing/blob/main/docs/releasing.md#signing-for-macos)),
with the same team, certificate and API key:

- **Apple's side**: the team's **Developer ID Application** certificate, an
  **App ID** `com.hivepaas.packwing` (the `CFBundleIdentifier` in
  `packaging/macos/Info.plist`) with a **Developer ID provisioning profile**, and
  an **App Store Connect API key** for notarizing.
- **The repository's secrets**: `MACOS_CERTIFICATE` (the `.p12`, base64),
  `MACOS_CERTIFICATE_PASSWORD`, `MACOS_PROVISIONING_PROFILE` (base64),
  `APPLE_API_KEY` (the `.p8`'s text), `APPLE_API_KEY_ID`, `APPLE_API_ISSUER_ID`.

The app claims no entitlement a profile must allow, so the profile is not put in
it yet: the Finder extension and its helper share an app group that starts with
the team's id, which needs none. Their entitlements are context-menu-rs's
(`packaging/macos/ContextMenu`).

**On a Mac**, with the certificate in the login keychain:

```sh
cargo build --release -p packwing -p packwing-cli
MACOS_SIGN_IDENTITY="Developer ID Application: Name (TEAMID)" \
APPLE_API_KEY_PATH=AuthKey_XXXX.p8 APPLE_API_KEY_ID=XXXX APPLE_API_ISSUER_ID=… \
scripts/package_macos.sh
```

## Making one

1. **The version.** `version` under `[workspace.package]` in `Cargo.toml` is
   the release's (`0.2.0`, or `0.3.0-beta1`). Write its notes in
   `docs/releases/v<version>.md`. Merge both into `main`.
2. **A rehearsal** (optional): *Actions › Release › Run workflow* on `main`
   builds and packages everything as the tag will, and keeps the files as the
   run's artifacts. A pull request that changes the packaging is rehearsed so
   too.
3. **The tag.** `git tag -a v0.2.0 -m "packwing 0.2.0" <commit> && git push origin v0.2.0`,
   on `main`'s commit with the version and the notes. The Release workflow
   checks that the tag is `Cargo.toml`'s version, builds the five platforms,
   and makes a **draft** release with the files and `SHA256SUMS`.
4. **Check the draft** ([Checking a release](#checking-a-release)), and publish it.
5. **The list.** Add the release to `release.json` with the SHA-256 of the
   `.app.tar.gz` files and the installer under `archives`, and of the `.deb`
   files under `packages` (never `archives`: [updates.md](updates.md#the-list-of-releases)),
   all from `SHA256SUMS`, and merge it. Sign what is on `main`
   ([updates.md › Signing](updates.md#signing)); commit `release.signed.json`,
   then move the `release` branch ([updates.md › Publishing a release's list](updates.md#publishing-a-releases-list)).

A release that went wrong is not deleted once the list names it: the next one
replaces it.

## Checking a release

CI builds and checks the `.deb` (installed, with `pkw` run from it), not the
packages on a real desktop. On a rehearsal's artifacts or the draft's files:

**macOS**:

- `spctl -a -t exec -vv Packwing.app` says `source=Notarized Developer ID`, the
  same with `-t open --context context:primary-signature` for the `.dmg`, and
  `xcrun stapler validate` passes on both.
- Drag the app from the disk image to Applications and open it: one question,
  *Open*. The welcome's right-click menu puts *Packwing* in Finder's own menu,
  right after Quick Actions, with no Quick Actions of packwing's left; Compress
  and Extract there run the app.
- `Packwing.app/Contents/MacOS/pkw --version` runs.

**Windows**: the installer adds a Start menu entry, an uninstall entry and
`pkw` in a new terminal; the right-click menu's items run packwing; the
uninstaller leaves no program folder.

**Linux**: the `.deb` installs; after `nautilus -q`, packwing's items are in
Nautilus's own menu.
