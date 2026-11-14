# packwing for Android

The Android app: Jetpack Compose over `packwing-core`, reached through the same
UniFFI crate the iOS app uses (`crates/mobile`). One module, `app/`.

Material 3, with the Soft tonal palette the desktop draws with
(`crates/app/src/colors.rs`) and Nunito, its typeface, read straight from
`crates/app/assets/fonts`. The screens are drawn from the Android page of the
design canvas, as the iOS app is drawn from its own: a navigation bar docked at
the foot, an extended floating button, bottom sheets, and the system's own
back gesture.

## Building

Needs the Android SDK, the NDK, a JDK 21, [cargo-ndk] and the two Rust targets:

```
rustup target add aarch64-linux-android x86_64-linux-android
cargo install cargo-ndk
export JAVA_HOME=/path/to/jdk-21        # unless java is already 21
export ANDROID_HOME=$HOME/Android/Sdk
make android                            # the library, its Kotlin, and a debug APK
make android-install                    # onto the device or emulator that is running
```

`make android` runs [`scripts/build_android.sh`](../scripts/build_android.sh),
which builds `packwing-mobile` as a shared library for `arm64-v8a` and
`x86_64` and writes the Kotlin that calls it, read from the library's own
symbols so the two always match. Neither
`app/src/main/jniLibs` nor `app/build/generated/uniffi` is committed.

[cargo-ndk]: https://github.com/bbqsrc/cargo-ndk

## What it does

Open an archive from the Files app, from `Share › packwing` in any app, or by
tapping one in a chat or an email. Look inside, folder by folder, search it
(`bao cao` finds `Báo cáo`), preview its pictures and texts without extracting,
extract it to a folder you point at or to the gallery, and make a ZIP or a 7z
with AES-256. The work is all `packwing-core`'s, as on the desktop.

## How Android's storage shapes it

Android hands out `content://` URIs, and nothing in its storage has a path
the app can use. The core does not need one: it reads where it is told to and
writes what it reads to whatever takes it (`Documents.kt`, `Trees.kt`).

- **Reading costs no copy.** packwing-core already reads an archive it cannot
  reach by path by asking for the ranges it wants, as it does for one kept on
  a storage; `crates/mobile` gives Android the same way in (`Document`, which
  only that system's build has), and `pread` answers from the descriptor the
  system opened. ZIP and 7z archives, which is what a phone opens and makes,
  are listed, previewed and extracted straight from the document. Two things
  still cost a copy: a RAR archive, because the library that reads one takes a
  filename, and a provider whose descriptor cannot be read at an offset, which
  a stream's cannot.

  What does not work is the obvious thing: `/proc/self/fd/N` is a path, but
  opening it is a fresh open of the file underneath, checked against the app's
  own rights rather than against the grant that handed over the descriptor, so
  for a file from the document picker or MediaStore it is refused.
- **Writing costs no second pass either.** A document tree cannot be written
  to by path, so the core does not write to one: `packwing_core::Destination`
  takes each entry as it is read, and `crates/mobile`'s `Tree` passes it to
  the app, which makes it in the folder the user pointed at or puts it in the
  gallery through MediaStore (`Trees.kt`). Nothing is staged and nothing is
  copied. A job that fails or is stopped takes back what it made, as one into
  a folder does.
- **A file keeps the name it came with.** A document provider adds an
  extension of its own when the name it is given disagrees with the kind, so
  what is extracted is made as the kind that provider's own table gives for
  the name's extension, and as nothing it knows when there is none.
- **Replacing is the app's own check.** What hivepack checks of a folder —
  nothing written outside it, nor over what is there — has no meaning for a
  destination that takes entries, so Settings' "Replace a file already there"
  is honoured in `DocumentTree` itself. It was not honoured at all while the
  app copied what it extracted across.
- **No storage permission is asked for.** The document picker and MediaStore
  are the only ways in and out, which is also why the folder a user picks is
  kept with `takePersistableUriPermission` instead of being found again by
  path.
- **A long job runs in a foreground service** (`ExtractService`), as Android
  asks of work the user started and is waiting for. The notification shows how
  far it got and stops it.

## What is not here yet

- Picking only some files to extract. The core takes a list of paths
  (`Archive.extractInto`'s `picked`), and the screen has no way to choose
  them yet.
- Pictures in the grid. An archive that is mostly pictures opens on a grid,
  but each is a tile saying its kind, not a thumbnail of it.
- Pausing a job. The core can stop one, not hold it; the progress screen
  offers Cancel alone. The design's Pause button has nothing behind it.
- The comic reader, storages (S3, WebDAV, SFTP), signatures, checksums,
  editing an archive, `age` files and the audit log: all of these are the
  desktop's for now, as on iOS.
- The system's own wallpaper colours. The app keeps packwing's palette on
  every device; the design names this as an option, and it is not wired up.
