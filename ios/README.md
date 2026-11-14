# packwing for iOS

A SwiftUI app over the same core as the desktop: `crates/mobile` wraps
`packwing-core` (and so hivepack) for Swift with
[UniFFI](https://mozilla.github.io/uniffi-rs/). Opening, listing, searching,
reading and extracting an archive, and making one, are the desktop's own code;
the app draws them in the Soft tonal design, as iOS 26 does its bars and
sheets. The screens are on the design canvas's "F · iOS 26" page.

## Building

Needs Xcode (iOS 26 SDK or later) and XcodeGen (`brew install xcodegen`).

```sh
make ios        # the Rust library for iPhones and the Simulator, its Swift, the Xcode project
open ios/Packwing.xcodeproj
```

Then Run in Xcode, on a Simulator (Apple silicon) or an iPhone; signing uses
the team in `ios/project.yml`. `make ios-sim` builds for the Simulator from
the command line, and `make ios-test` runs the UI tests
(`PackwingUITests`: the samples in `PackwingUITests/Samples` opened,
extracted, unlocked, previewed, saved to Photos, shown as a grid of pictures, put in new archives, received as Mail gives one and its checksum compared) on the iPhone 17
Simulator; `make ios-test IOS_DEVICE="iPad Pro 13-inch (M5)"` runs them on an
iPad's. After changing `crates/mobile`, run `make ios` again
(`scripts/build_ios.sh --debug` builds its library quicker).

What is generated, and not committed: `ios/Generated/` (the XCFramework and
`packwing_mobile.swift`), `ios/Packwing.xcodeproj` and its `Info.plist`.
Edit `ios/project.yml` for the project's settings.

The app icon is `packaging/icons/packwing-ios.svg` (the desktop's, its tile
filling the square, which iOS rounds), drawn at 1024 px without transparency:

```sh
qlmanage -t -s 1024 -o /tmp packaging/icons/packwing-ios.svg
sips -s format jpeg /tmp/packwing-ios.svg.png --out /tmp/icon.jpg
sips -s format png /tmp/icon.jpg --out ios/Packwing/Assets.xcassets/AppIcon.appiconset/icon-1024.png
```

## Trying it

`-open <path>` as a launch argument (debug builds) opens that archive at once;
`-new <path>|<path>…` shows New archive, of those files and folders;
`-receive <path>` gives packwing that archive as Mail does, a copy in its
`Documents/Inbox`.
Files put in the app's `Documents` show in Files as On My iPhone › packwing:

```sh
xcrun simctl get_app_container booted com.hivepaas.packwing data   # its folder in the Simulator
xcrun simctl launch booted com.hivepaas.packwing -open "<that folder>/Documents/a.zip"
```

## What is there

| | |
| --- | --- |
| `Packwing/AppModel.swift` | Opening an archive (and asking for its password), the Recent list |
| `Packwing/HomeView.swift` | The Archives tab: open from Files, Recent; the Search tab |
| `Packwing/ArchiveView.swift` | A folder of an archive, and a search through all of it |
| `Packwing/Gallery.swift` | A folder as a grid: its pictures as themselves (read from the archive, made small as they decode), the rest by kind; the grid first where it is pictures mostly |
| `Packwing/ChecksumSheet.swift` | An archive's own checksums (SHA-256 and SHA-1 first, as the desktop's; any of five), and one pasted compared with them |
| `Packwing/PasswordSheet.swift` | An archive's password, or an age file's passphrase |
| `Packwing/ExtractSheet.swift`, `Extracting.swift` | Extract: to a new folder in Files (packwing's own, or one chosen), or the pictures and videos to Photos; a file previewed in Quick Look |
| `Packwing/RunningJob.swift`, `JobScreen.swift` | A long job: how far it is, going on once packwing is left (BGContinuedProcessingTask), how it ended |
| `Packwing/CreateSheet.swift`, `Creating.swift` | New archive: files, folders and photos chosen, ZIP or 7z, a password (AES-256, and 7z's names hidden too), made in a folder of Files and shared |
| `Packwing/SettingsView.swift` | The language, how new archives are made, the version |
| `Packwing/Theme.swift` | The desktop's colors, light and dark; Nunito; a file's tag |

## iPhone and iPad

One app for both. On an iPad the screens are the iPhone's, given room: the
grid of pictures has as many columns as fit (three at least), the sheets are
pages, and a job's screen keeps to a readable width. Files calls packwing's
folder On My iPad there, and so does packwing.

## From another app

packwing is in the Share sheet's row of apps for every archive type it
declares (`CFBundleDocumentTypes` in `project.yml`); there is no extension of
its own. Shared from Files, an archive is read where it is. Mail and Messages
give a copy instead, in an Inbox of packwing's own (`Documents/Inbox`, which
Files would show as a folder of packwing's, or `tmp/<its id>-Inbox`):
`AppModel.receive` moves it out, into `Application Support/Received`, so it is
seen without being saved where the person keeps files. Recent opens it again;
it goes when it is removed from Recent, falls off it, or has not been opened
for 30 days.
