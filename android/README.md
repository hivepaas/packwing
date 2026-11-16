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
make android-test                       # the UI tests, on the device or emulator that is running
```

`make android-test` runs the UI tests (`app/src/androidTest`): the app driven
as a person would, over the archives the iOS app's tests open
(`ios/PackwingUITests/Samples`, one set for both phones). An archive is
opened, looked through, searched and previewed; a locked one asks for its
password again when it is wrong; one of pictures is a grid, and goes to the
gallery. A device with the screen on is needed: an emulator started with
`-no-window` is one, and `adb shell svc power stayon true` keeps it awake.

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
extract it to a folder you point at or to the gallery, and make a new archive.
The work is all `packwing-core`'s, as on the desktop.

**New archive** takes files, a folder or photos, and makes an archive in the
desktop's formats: ZIP, 7z, the tar ones (`tar.zst`, `tar.gz`, `tar.xz`,
`tar.bz2`, `tar.lz4`, `tar`), and, for one file by itself, `gz`, `zst`, `xz`,
`bz2` or `lz4`. A password is ZIP's and 7z's own AES-256 (7z's names hidden
too), or age around all of it: to a passphrase, or to public keys (age's, or
SSH ones; typed, or a file of them). It is cut in parts of 25 MB, 4 GB or a
size typed, and made in a folder of the phone's or of a storage. The format
and how hard it compresses are kept for the next one (`CreateScreen.kt`).

A long press picks a file or folder, and a tap then picks more: what is
picked is extracted alone. Its ⋮ menu, and the archive's, read **Checksums**,
as the desktop's Checksums… does: of what is picked, of all of its files (in
one read of it, listed as `sha256sum` writes them, to copy or save in a
folder), or of the archive's own file; and checksums given with the files,
pasted, are compared with them, a list or one alone (`ChecksumsScreen.kt`).

The archive's ⋮ also checks its **Signature** (`name.sig`, as `ssh-keygen -Y
sign` makes one) by the keys trusted in Settings › Keys you trust: one beside
it where packwing can look (a file it reaches by its path), or one chosen with
the document picker, which is what a file Android handed over alone needs. A
signature by a key not trusted says whose it is, and offers to trust it
(`SignatureSheet.kt`).

A ZIP or 7z archive packwing may write over is **changed** as the desktop
changes one: files, a folder or photos added (the + in its bar), files
renamed and deleted (a file's ⋮, or those picked). Android gives no path to
change it at, so a copy of it is changed in the app's own folder and written
over it, and it is read again: a change that fails or is stopped leaves the
archive as it was. One another app handed over to be looked at is read only,
and offers none of this (`AppModel.change`).

**Storages** across the network (S3, WebDAV, SFTP) are added on the Archives
tab, tried before they are kept, and browsed; an archive on one is opened
where it is, the core reading only what it needs of it, and any other file
previewed (`StorageScreen.kt`). A storage's secrets are not kept with its
settings: they go to Android's keychain, `Vault.kt`, each encrypted by a key of
the Android Keystore that never leaves it, and left out of backups, which the
key could never come with. The keyring crate has no Android store it can set
for itself (the one there is needs JNI, and the app reaches the core through
JNA), so the app's vault is keyring-core's default store
(`crates/mobile/src/vault.rs`).

SFTP asks as it connects, over whatever shows (`SshAsker.kt`): whether to
trust a server packwing has not connected to before, with its fingerprint
whole; a password or a key's passphrase, kept in the vault if the user says
so; a code. The servers trusted are in packwing's own known_hosts, as a phone
has no `~/.ssh`. A storage signs in with a key packwing keeps, or any of them;
Settings › SSH keys keeps them (pasted or from their file, opened with their
passphrase), copies their public halves and removes them (`SshScreens.kt`).

A storage is **written to** from the phone, as the desktop writes to one:
folders made; files and folders renamed, deleted (for good: a storage across
the network has no Trash), copied and moved to a folder of any storage added,
and downloaded to a folder of the phone; files, a folder or photos sent up;
new archives made there (its ⋮ › New archive). What comes from the phone is
copied into the app's own folder first, and what goes to it through there,
as a document tree has no path the core can write to (`AppModel.onStorage`).

The storage tests run against the servers of `make webdav-start` and
`make sftp-start`, which the emulator reaches at `10.0.2.2`, and are skipped
without them.

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

- Pausing a job. The core can stop one, not hold it; the progress screen
  offers Cancel alone. The design's Pause button has nothing behind it.
- Signing, the desktop's other settings (Settings › New archives and
  Extracting), and Pro: the iOS app has them (`ios/README.md`), and they come
  here one by one.
- The comic reader and the audit log, which are the desktop's alone.
- The system's own wallpaper colours. The app keeps packwing's palette on
  every device; the design names this as an option, and it is not wired up.
