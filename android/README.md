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

Android hands out `content://` URIs; the core works on paths. Each way in and
out is bridged in `Documents.kt`:

- **Reading usually costs a copy.** The descriptor the system opens for an
  archive is a path of its own, `/proc/self/fd/N`, and the core needs a path it
  can open again for every file it reads. Opening that path is a fresh open of
  the file underneath, checked against the app's own rights rather than the
  grant that handed over the descriptor — so for a file from the document
  picker or MediaStore, which the app has no rights of its own to, it is
  refused. The app tries the descriptor, checks whether it opens again, and
  copies the archive into its cache when it does not, which is the usual case.
  A core that could read an archive from a file descriptor would remove that
  copy.
- **Writing costs a second pass.** A document tree cannot be written to by
  path, so the core extracts into the app's own folder and what it wrote is
  then copied into the folder the user pointed at, or to the gallery through
  MediaStore. Every extracted byte is therefore written twice. A core that
  could extract into a file descriptor (or a `Sink`, which `hivepack` already
  has) would remove that second pass; until then, large archives pay for it.
- **No storage permission is asked for.** The document picker and MediaStore
  are the only ways in and out, which is also why the folder a user picks is
  kept with `takePersistableUriPermission` instead of being found again by
  path.
- **A long job runs in a foreground service** (`ExtractService`), as Android
  asks of work the user started and is waiting for. The notification shows how
  far it got and stops it.

## What is not here yet

- Picking only some files to extract. The core takes a list of paths
  (`Archive.extract`'s `picked`), and the screen has no way to choose them yet.
- Pausing a job. The core can stop one, not hold it; the progress screen
  offers Cancel alone. The design's Pause button has nothing behind it.
- The comic reader, storages (S3, WebDAV, SFTP), signatures, checksums,
  editing an archive, `age` files and the audit log: all of these are the
  desktop's for now, as on iOS.
- The system's own wallpaper colours. The app keeps packwing's palette on
  every device; the design names this as an option, and it is not wired up.
