#!/bin/bash
#
# Checks a .deb that scripts/package_linux.sh made holds what it is to hold:
# the program and pkw, pkw's pages and completions, the launcher and icon,
# the Nautilus extension, the licenses, and the libraries it depends on. CI
# runs it on each package it makes; it runs here too, on Linux.
#
#   scripts/check_linux_package.sh dist/packwing_0.1.0_amd64.deb
#
# With --install (and sudo), it installs the package as a user does, then
# runs pkw from it: its version, an archive made, tested and listed, and its
# bash completions loaded.

set -euo pipefail

INSTALL=false
if [ "${1:-}" = "--install" ]; then
	INSTALL=true
	shift
fi
DEB="${1:?the .deb to check}"
[ -f "$DEB" ] || { echo "no package at $DEB" >&2; exit 1; }

FAILED=0
fail() { echo "  missing: $1" >&2; FAILED=1; }

LISTING="$(dpkg-deb -c "$DEB")"
FILES="$(awk '{print $6}' <<<"$LISTING" | sed 's|^\./|/|')"
has() { grep -qx "$1" <<<"$FILES" || fail "$1"; }

echo "$DEB"
for file in \
	/usr/bin/packwing \
	/usr/bin/pkw \
	/usr/share/applications/packwing.desktop \
	/usr/share/icons/hicolor/scalable/apps/packwing.svg \
	/usr/share/doc/packwing/copyright \
	/usr/share/doc/packwing/THIRD-PARTY-LICENSES \
	/usr/share/man/man1/pkw.1.gz \
	/usr/share/man/man1/pkw-extract.1.gz \
	/usr/share/man/man1/pkw-storages.1.gz \
	/usr/share/bash-completion/completions/pkw \
	/usr/share/zsh/vendor-completions/_pkw \
	/usr/share/fish/vendor_completions.d/pkw.fish; do
	has "$file"
done
grep -q '/nautilus/extensions-4/libcontext-menu-packwing.so$' <<<"$FILES" || fail "the Nautilus extension"
# No page for clap's own `help`, nor for the hidden `man`.
for none in /usr/share/man/man1/pkw-help.1.gz /usr/share/man/man1/pkw-man.1.gz; do
	if grep -qx "$none" <<<"$FILES"; then echo "  not to be there: $none" >&2; FAILED=1; fi
done

# The programs are executable, and depend on the system's libraries by their
# packages' names.
for program in usr/bin/packwing usr/bin/pkw; do
	grep -q "^-rwxr-xr-x .* \./$program\$" <<<"$LISTING" || { echo "  not executable: /$program" >&2; FAILED=1; }
done
DEPENDS="$(dpkg-deb -f "$DEB" Depends)"
grep -q 'libc6' <<<"$DEPENDS" || { echo "  Depends has no libc6: $DEPENDS" >&2; FAILED=1; }
# The crates pkw alone is built with are in the licenses.
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
dpkg-deb --fsys-tarfile "$DEB" | tar -xf - -C "$WORK" ./usr/share/doc/packwing/THIRD-PARTY-LICENSES
for crate in clap rpassword hivepack; do
	grep -q "\b$crate\b" "$WORK/usr/share/doc/packwing/THIRD-PARTY-LICENSES" || { echo "  no license for $crate" >&2; FAILED=1; }
done
[ "$FAILED" = 0 ] || { echo "the package is not as it should be" >&2; exit 1; }
echo "  holds what it should"

$INSTALL || exit 0

sudo DEBIAN_FRONTEND=noninteractive apt-get install -y -qq --no-install-recommends "$(realpath "$DEB")" >/dev/null
echo "  installed"
pkw --version
cd "$WORK"
mkdir -p in/docs
echo "xin chào" >in/docs/notes.txt
pkw create -q made.tar.zst in
pkw test made.tar.zst
pkw list made.tar.zst | grep -qx 'in/docs/notes.txt' || { echo "  pkw list does not show what it made" >&2; exit 1; }
[ "$(pkw cat made.tar.zst in/docs/notes.txt)" = "xin chào" ] || { echo "  pkw cat is not what went in" >&2; exit 1; }
bash -c 'source /usr/share/bash-completion/completions/pkw && complete -p pkw' >/dev/null ||
	{ echo "  bash does not load pkw's completions" >&2; exit 1; }
echo "  pkw runs from the package"
