# Test archives

Disc images and 7z archives of the files the tests' `sample()` holds, and
a macOS package and a XAR archive from hivepack's own test data, as
packwing-core's tests read them.

- `sample.iso.zst`: ISO 9660, with Rock Ridge and Joliet, under zstd.
- `sample-udf.iso.zst`: UDF beside ISO 9660, as DVDs and Windows' images
  are, which hivepack::open finds as a UDF volume, under zstd.
- `card.img.zst`: a GPT disk of 10 MiB under zstd, 1.5 KB of it since the
  rest is zeros - three partitions: an EFI system partition holding FAT
  (`qemu-img`'s vvfat wrote it from a folder), a Linux one holding ext2
  (`mke2fs -d`), and a swap partition, which holds no file system at all. It
  is what `tests/images.rs` reads, so that the disk-image tests need no tool
  and run everywhere. How it was made is at the end of this file.
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

- `tiny.rpm`: libarchive 3.8.10's `cpio_svr4_gzip_rpm.rpm` test package
  (BSD-2-Clause, Copyright (c) 2003-2011 Tim Kientzle and others): its payload,
  `etc/file1` to `etc/file3`, a newc cpio archive under gzip.

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

- `sample-xz.deb`, `sample-zstd.deb`, `sample-gzip.deb`: Debian packages of
  the files below, made by dpkg-deb 1.23 with their `data.tar` under xz,
  Zstandard and gzip (commands below).
- `sample.sqfs`: a SquashFS image of the same files, as a snap is (xz, blocks
  of 4 KiB), made by mksquashfs 4.7.5.
- `links.sqfs`: a SquashFS image (zstd) of `hello.txt` and its second name
  `hello-again.txt`, links to it and to `/etc/passwd`, `sparse.bin` (8 KiB of
  zeros, two holes, then `end`), and `bin/run`, setuid.
- `squashfs/*.sqfs`: hivepack-rs's `testdata/squashfs`, one image for each
  compressor (gzip, LZMA, LZO, xz, LZ4, zstd, and nothing compressed), made by
  squashfs-tools 4.6.1's mksquashfs from a small tree with a second name of a
  file, a link, a block device and a FIFO (MIT, Copyright (c) 2026
  HivePaaS).

cpio archives, RPM packages, AppImages and encrypted ZIP archives are made
by the tests themselves: an AppImage of type 2 is an ELF program marked `AI`
2 at its 8th byte, then `sample.sqfs`; one of type 1 is `sample.iso` with an
ELF program marked `AI` 1 in its first 32 KiB.

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
# The snap and the packages.
mksquashfs src sample.sqfs -noappend -quiet -no-progress -comp xz -b 4K \
  -all-root -no-xattrs -mkfs-time 1788264000 -processors 1
mkdir -p pkg/DEBIAN && cp -a src/. pkg/
printf 'Package: sample\nVersion: 1.0\nArchitecture: all\nMaintainer: packwing tests <tests@example.invalid>\nDescription: packwing test package\n' \
  > pkg/DEBIAN/control
find pkg -exec touch -h -d '2026-09-01 12:00:00 UTC' {} +
for z in xz zstd gzip; do
  SOURCE_DATE_EPOCH=1788264000 dpkg-deb --root-owner-group -Z$z --build pkg sample-$z.deb
done
# Links, a hole and a setuid program.
mkdir -p links/bin
printf 'Hello from a snap.\n' > links/hello.txt
ln links/hello.txt links/hello-again.txt
printf '#!/bin/sh\necho run\n' > links/bin/run && chmod 4755 links/bin/run
ln -s hello.txt links/link-inside
ln -s /etc/passwd links/link-outside
truncate -s 8192 links/sparse.bin && printf 'end\n' >> links/sparse.bin
find links -exec touch -h -d '2026-09-01 12:00:00 UTC' {} +
mksquashfs links links.sqfs -noappend -quiet -no-progress -comp zstd -b 4K \
  -all-root -no-xattrs -mkfs-time 1788264000 -processors 1
```

A disk image of three partitions, for `tests/images.rs`. The partitions go
where the table says they are, which is not where their sizes alone would put
them: `sfdisk` aligns the first to a mebibyte.

```sh
mkdir -p tree/docs root/etc
printf 'hello from the boot partition\n' > tree/README.TXT
printf 'a note\n' > tree/docs/note.txt
printf 'root file system\n' > root/etc/hostname
printf 'deeper\n' > root/etc/deeper.conf
qemu-img convert -f vvfat -O raw "fat:floppy:$PWD/tree" p1.img   # 2,949,120 bytes
mke2fs -q -F -t ext2 -b 1024 -d root p2.img 3072
truncate -s 10M card.img
printf 'label: gpt\nsize=5760, type=uefi, name="EFI"\nsize=6144, type=linux, name="root"\nsize=2048, type=swap, name="swap"\n' \
  | sfdisk card.img
python3 -c '
import json, subprocess
table = json.loads(subprocess.run(["sfdisk", "-J", "card.img"],
                                  capture_output=True, text=True).stdout)["partitiontable"]
card = bytearray(open("card.img", "rb").read())
for part, name in zip(table["partitions"], ("p1.img", "p2.img", None)):
    if name is None:
        continue
    data = open(name, "rb").read()
    at = part["start"] * table["sectorsize"]
    card[at:at + len(data)] = data
open("card.img", "wb").write(bytes(card))
'
zstd -19 card.img -o card.img.zst
```
