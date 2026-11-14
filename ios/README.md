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
the command line. After changing `crates/mobile`, run `make ios` again
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

`-open <path>` as a launch argument (debug builds) opens that archive at once.
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
| `Packwing/PasswordSheet.swift` | An archive's password, or an age file's passphrase |
| `Packwing/SettingsView.swift` | The language, the version |
| `Packwing/Theme.swift` | The desktop's colors, light and dark; Nunito; a file's tag |

Not yet: previewing a file, extracting (to Files or Photos), new archives,
going on in the background (BGContinuedProcessingTask), and the Share sheet's
own extension.
