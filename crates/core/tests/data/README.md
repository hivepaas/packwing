# Test archives

Disc images and 7z archives of the files the tests' `sample()` holds, and
a macOS package and a XAR archive from hivepack's own test data, as
packwing-core's tests read them.

- `sample.iso.zst`: ISO 9660, with Rock Ridge and Joliet, under zstd.
- `sample-udf.iso.zst`: UDF beside ISO 9660, as DVDs and Windows' images
  are, which hivepack::open finds as a UDF volume, under zstd.
- `sample.7z`: 7-Zip's defaults, LZMA2 in one solid block.
- `secret.7z`: the same, its files encrypted with AES; password `secret`.
- `hidden.7z`: the same, its names encrypted too (`-mhe=on`).
- `notes.tlz`: a tar archive (`./docs/notes.txt`, `./numbers.txt`,
  `./table.csv`) under `lzma -9` (xz 5.8), with the short name of `.tar.lzma`,
  which has no magic: told by its name.
- `tiny.pkg`, `default.xar`: hivepack-rs's `testdata/xar` (made on macOS
  27.0.1 with `pkgbuild` and `xar`; MIT, Copyright (c) 2026 HivePaaS). The
  package's one file is `Payload/usr/local/share/example/readme.txt`, in a
  cpio archive under gzip.

- `notes.tar.zst.age`: a tar archive (`docs/notes.txt`, `docs/data.bin`)
  under zstd, encrypted by age v1.3.2 to the passphrase `password` at
  work factor 10: hivepack-rs's `testdata/oracle/open.txt`, its first file
  (MIT, Copyright (c) 2026 HivePaaS).

- `x25519.tar.zst.age`, `pq.tar.zst.age`, `ssh.tar.zst.age`,
  `ssh-locked.tar.zst.age`: a tar archive (`docs/notes.txt`) under zstd,
  encrypted by age to `test-age.key`, `test-age-pq.key` (`age-keygen -pq`, age
  v1.3.2), `test-ssh.key` and `test-ssh-locked.key` (passphrase
  `khoá ssh`). `test-age-locked.key` is `test-age.key` under `age -p`
  (passphrase `khoá age`). These keys were made for these tests and open
  nothing else.

- `rar5_multiple_files.rar`, `rar4_solid_encrypted.rar`,
  `rar5_solid_encrypted.rar`, `rar4_encrypted_filenames.rar`,
  `rar5_encrypted_filenames.rar` (password `password`), `test_rar_multivolume_multiple_files.part*.rar`:
  libarchive 3.8.10's test archives (`libarchive/test/*rar*.uu`), decoded
  (BSD-2-Clause). packwing never writes RAR, so its tests read what others made.

cpio archives, RPM packages and encrypted ZIP archives are made by the
tests themselves.

```sh
mkdir -p src/docs/plans
printf 'Notes from inside the archive.\n' > src/docs/notes.txt
printf 'aaaa' > src/docs/plans/a.txt
printf '# Read me\n' > src/readme.md
find src -exec touch -h -d '2026-09-01 12:00:00 UTC' {} +
genisoimage -quiet -R -J -V PACKWING -o sample.iso src
genisoimage -quiet -udf -R -J -V PACKWING -o sample-udf.iso src
zstd -q -19 sample.iso sample-udf.iso
# 7-Zip 26.05's 7zz, as hivepack-rs's scripts/testdata.py builds it.
(cd src && 7zz a -mtc=off -mta=off ../sample.7z -- *)
(cd src && 7zz a -mtc=off -mta=off -psecret ../secret.7z -- *)
(cd src && 7zz a -mtc=off -mta=off -psecret -mhe=on ../hidden.7z -- *)
```
