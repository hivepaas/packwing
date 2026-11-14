# Releasing

A release is a tag, the files CI builds for it, and an entry in the signed list
of releases the app reads ([updates.md](updates.md)). Until the list names it, no
app is told of it.

## What a release ships

| Platform | Files | Made by |
| --- | --- | --- |
| Linux x86_64, aarch64 | `packwing_<v>_<amd64\|arm64>.deb`, `packwing_<v>_<x86_64\|aarch64>.rpm`, `pkw_<v>_linux_<arch>.tar.gz` | `scripts/package_linux.sh` |
| macOS aarch64, x86_64 | `packwing_<v>_macos_<arch>.dmg`, `.app.tar.gz`, `pkw_<v>_macos_<arch>.tar.gz` | `scripts/package_macos.sh` |
| Windows x86_64 | `packwing_<v>_windows_x86_64-setup.exe`, `.zip`, `pkw_<v>_windows_x86_64.zip` | `scripts/package_windows.sh` |

and `SHA256SUMS`. The updater installs the `.app.tar.gz` and the installer, and
shows the commands that install the `.deb` or the `.rpm`; the others are for
people.

- **Linux**: the packages are built on Ubuntu 22.04, so they run on glibc 2.35
  and newer, and hold context-menu-rs's Nautilus extension: the `.deb` for
  Debian and Ubuntu, the `.rpm` for Fedora and openSUSE (in `/usr/lib64`, where
  their Nautilus looks). The `.rpm` is made with `rpmbuild` on Ubuntu, so it
  names the glibc it needs, and an older system refuses it at install.
- **pkw alone** (`pkw_<v>_<os>_<arch>`): the command line with no app, for
  servers and scripts, with its manual pages, completions, licenses and a
  README (`packaging/cli/README.txt`). On Linux it links only the C library; on
  macOS it is the app's `pkw`, signed with the Developer ID and notarized on its
  own. It does not update itself.
- **macOS**: `Packwing.app`, with `pkw` beside the program and context-menu-rs's
  Finder extension and its helper, which put packwing's items in Finder's own
  right-click menu. Signed with the Developer ID and notarized when the
  repository holds one ([Signing for macOS](#signing-for-macos)). Without one it
  is signed ad hoc and holds no extension (the items are then Quick Actions),
  and Gatekeeper asks the first time it opens. The updater accepts a new `.app`
  whose signature verifies, by the team that signed the running one.
- **Windows**: an NSIS installer, which asks whom for: everyone on the computer,
  by default, into `C:\Program Files\packwing` with an administrator's
  permission (Windows asks), or the user alone into
  `%LOCALAPPDATA%\Programs\packwing` with none; both are where the updater looks
  for it. `/ALLUSERS` or `/CURRENTUSER` says it without the page, as with `/S`
  for a quiet install; where packwing is installed already, it goes there again,
  as it was. `pkw.exe` goes in the PATH (the computer's, or the user's), and
  context-menu-rs's extension for Windows 11's menu is registered for the user
  who installs. It is not signed: SmartScreen asks the first time (More info ›
  Run anyway), and Windows takes the extension's package only with Developer
  Mode on.

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

## The Mac App Store

The store's build ([building.md](building.md#the-mac-app-stores-build)) is
packaged by the Release workflow's `macos-app-store` job, beside the others but
not in the GitHub release. A tag sends it to App Store Connect (app **Packwing**,
`6834744685`), where it shows in TestFlight once Apple has processed it; a run by
hand sends it with **app_store_upload**. Each upload needs a build number the
store has not seen: the count of commits, so a commit is sent once.

- **Apple's side**: the team's **Apple Distribution** and **Mac Installer
  Distribution** certificates, and a **Mac App Store** provisioning profile for
  `com.hivepaas.packwing` ("packwing Mac App Store"). The API key above uploads.
- **The repository's secrets**: `MAS_APP_CERTIFICATE` and
  `MAS_INSTALLER_CERTIFICATE` (the two `.p12`, base64), `MAS_CERTIFICATE_PASSWORD`
  (one password for both), `MAS_PROVISIONING_PROFILE` (base64), and the
  `APPLE_API_KEY*` above. Without them the job still builds and checks the
  package, signed ad hoc and not sendable.

**On a Mac**, with both certificates in a keychain:

```sh
MAS_APP_IDENTITY="Apple Distribution: Name (TEAMID)" \
MAS_INSTALLER_IDENTITY="3rd Party Mac Developer Installer: Name (TEAMID)" \
MAS_PROFILE=packwing_app_store.provisionprofile [MAS_KEYCHAIN=…] \
scripts/package_macos_appstore.sh
```

and `UPLOAD=1` with the `APPLE_API_KEY_*` variables to send it. A version is
then released from App Store Connect: its page, screenshots and review, with
the in-app purchase sent for review with the first version.

A tag sends the package, so on a tag the job fails while the repository does
not have the store's secrets. The GitHub release does not wait for it: its
draft is made from the other jobs' files all the same.

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
5. **The list.** Add the release to `release.json`, its `releaseDate` the day
   it is published, with the SHA-256 of the
   `.app.tar.gz` files and the installer under `archives`, and of the `.deb`
   and `.rpm` files under `packages` (never `archives`: [updates.md](updates.md#the-list-of-releases)),
   all from `SHA256SUMS`, and merge it. Sign what is on `main`
   ([updates.md › Signing](updates.md#signing)); commit `release.signed.json`,
   then move the `release` branch ([updates.md › Publishing a release's list](updates.md#publishing-a-releases-list)).
6. **Homebrew.** Within three hours of the `release` branch moving,
   [hivepaas/homebrew-tap](https://github.com/hivepaas/homebrew-tap)'s Bump
   opens a pull request for `pkw` and the `packwing` cask at the new stable
   release (*Actions › Bump › Run workflow* there, sooner). Merge it once its
   Tests pass.

A release that went wrong is not deleted once the list names it: the next one
replaces it. Until then, a tag can be made again: delete it
(`git push origin :refs/tags/v0.2.0`), with no draft left of it, and tag the
commit that should be released.

**Who does what.** Claude Code makes a release when asked, from the version's
pull request to the Homebrew bump: it tags, checks the draft, and writes the
list's pull request. It stops to ask before publishing the draft, which is
public from then on. The list is signed by a maintainer on the signing
machine, where the offline keys are ([updates.md › Signing](updates.md#signing));
Claude Code then commits `release.signed.json` and moves the `release` branch
if asked to. The tags' ruleset asks for a signature, which a maintainer's
account bypasses: the push says so.

**Lines and Pro.** A version's line is its major and minor numbers (0.2), and is
as old as its first release in `release.json`: a Pro license bought outright
keeps every line released before its updates end, with all of its fixes. So a
fix goes out as a patch of each line still in use (0.2.3, beside 0.3.1), and a
new feature as a new line. The build reads the list for its line's day
(`crates/license/build.rs`); a line's first release, not listed yet when it is
built, takes the day it is built.

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

**Linux**: the `.deb` installs on Ubuntu and the `.rpm` on Fedora (CI installs
each, the `.rpm` in a Fedora container, and runs `pkw` from it); after
`nautilus -q`, packwing's items are in Nautilus's own menu.

**pkw alone**: unpacked, `pkw --version` runs; on macOS, downloaded with a
browser, it runs from Terminal with no question (`spctl -a -t install -vv pkw`
says `Notarized Developer ID`).
