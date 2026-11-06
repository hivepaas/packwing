#!/bin/bash
#
# Checks what scripts/package_linux.sh made holds what it is to hold. CI runs
# it on each file it makes; it runs here too, on Linux.
#
#   scripts/check_linux_package.sh dist/packwing_0.1.0_amd64.deb
#   scripts/check_linux_package.sh dist/packwing_0.1.0_x86_64.rpm
#   scripts/check_linux_package.sh dist/pkw_0.1.0_linux_x86_64.tar.gz
#
# A package: the program and pkw, pkw's pages and completions, the launcher
# and icon, the Nautilus extension, the licenses, and the libraries it
# depends on. pkw alone: the program, linking no window system, its pages,
# completions and licenses; it is run from where it is unpacked.
#
# With --install, a package is installed as a user does, then pkw is run from
# it: its version, an archive made, tested and listed, and its bash
# completions loaded. The .deb on this system, with sudo; the .rpm in a
# Fedora container, with Docker.

set -euo pipefail

INSTALL=false
ROUND_TRIP=false
case "${1:-}" in
--install) INSTALL=true; shift ;;
# What the Fedora container runs, with the package installed: pkw's round trip.
--round-trip) ROUND_TRIP=true; shift ;;
esac
FILE="${1:?the .deb, .rpm or pkw .tar.gz to check}"
$ROUND_TRIP || [ -f "$FILE" ] || { echo "no file at $FILE" >&2; exit 1; }

FAILED=0
fail() { echo "  missing: $1" >&2; FAILED=1; }
wrong() { echo "  $1" >&2; FAILED=1; }
FILES=""
has() { grep -qx "$1" <<<"$FILES" || fail "$1"; }
hasnt() { if grep -qx "$1" <<<"$FILES"; then wrong "not to be there: $1"; fi; }
done_checking() {
	[ "$FAILED" = 0 ] || { echo "$FILE is not as it should be" >&2; exit 1; }
	echo "  holds what it should"
}

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

# The crates pkw alone is built with are in these licenses.
licenses_name_pkws_crates() {
	local crate
	for crate in clap rpassword hivepack; do
		grep -q "\b$crate\b" "$1" || wrong "no license for $crate in $(basename "$1")"
	done
}

# pkw at $1 opens what it makes: an archive made, tested, listed and read.
pkw_round_trip() {
	local pkw="$1" dir
	dir="$(mktemp -d "$WORK/run.XXXX")"
	(
		cd "$dir"
		mkdir -p in/docs
		echo "xin chào" >in/docs/notes.txt
		"$pkw" create -q made.tar.zst in
		"$pkw" test made.tar.zst
		"$pkw" list made.tar.zst | grep -qx 'in/docs/notes.txt' || { echo "  pkw list does not show what it made" >&2; exit 1; }
		[ "$("$pkw" cat made.tar.zst in/docs/notes.txt)" = "xin chào" ] || { echo "  pkw cat is not what went in" >&2; exit 1; }
	)
}

# Pages: one for each command, none for clap's own `help` nor the hidden `man`.
PAGES="pkw pkw-extract pkw-storages"
NO_PAGES="pkw-help pkw-man"

check_deb() {
	local listing program
	listing="$(dpkg-deb -c "$FILE")"
	FILES="$(awk '{print $6}' <<<"$listing" | sed 's|^\./|/|')"
	for file in \
		/usr/bin/packwing \
		/usr/bin/pkw \
		/usr/share/applications/packwing.desktop \
		/usr/share/icons/hicolor/scalable/apps/packwing.svg \
		/usr/share/doc/packwing/copyright \
		/usr/share/doc/packwing/THIRD-PARTY-LICENSES \
		/usr/share/bash-completion/completions/pkw \
		/usr/share/zsh/vendor-completions/_pkw \
		/usr/share/fish/vendor_completions.d/pkw.fish; do
		has "$file"
	done
	for page in $PAGES; do has "/usr/share/man/man1/$page.1.gz"; done
	for page in $NO_PAGES; do hasnt "/usr/share/man/man1/$page.1.gz"; done
	grep -q '/nautilus/extensions-4/libcontext-menu-packwing.so$' <<<"$FILES" || fail "the Nautilus extension"
	# The programs are executable, and depend on the system's libraries by
	# their packages' names.
	for program in usr/bin/packwing usr/bin/pkw; do
		grep -q "^-rwxr-xr-x .* \./$program\$" <<<"$listing" || wrong "not executable: /$program"
	done
	local depends
	depends="$(dpkg-deb -f "$FILE" Depends)"
	grep -q 'libc6' <<<"$depends" || wrong "Depends has no libc6: $depends"
	dpkg-deb --fsys-tarfile "$FILE" | tar -xf - -C "$WORK" ./usr/share/doc/packwing/THIRD-PARTY-LICENSES
	licenses_name_pkws_crates "$WORK/usr/share/doc/packwing/THIRD-PARTY-LICENSES"
	done_checking

	$INSTALL || return 0
	sudo DEBIAN_FRONTEND=noninteractive apt-get install -y -qq --no-install-recommends "$(realpath "$FILE")" >/dev/null
	echo "  installed"
	pkw --version
	pkw_round_trip pkw
	bash -c 'source /usr/share/bash-completion/completions/pkw && complete -p pkw' >/dev/null ||
		{ echo "  bash does not load pkw's completions" >&2; exit 1; }
	echo "  pkw runs from the package"
}

check_rpm() {
	FILES="$(rpm -qlp "$FILE" 2>/dev/null)"
	for file in \
		/usr/bin/packwing \
		/usr/bin/pkw \
		/usr/share/applications/packwing.desktop \
		/usr/share/icons/hicolor/scalable/apps/packwing.svg \
		/usr/share/licenses/packwing/LICENSE \
		/usr/share/licenses/packwing/THIRD-PARTY-LICENSES \
		/usr/share/bash-completion/completions/pkw \
		/usr/share/zsh/site-functions/_pkw \
		/usr/share/fish/vendor_completions.d/pkw.fish \
		/usr/lib64/nautilus/extensions-4/libcontext-menu-packwing.so; do
		has "$file"
	done
	for page in $PAGES; do grep -q "^/usr/share/man/man1/$page\.1" <<<"$FILES" || fail "the page $page.1"; done
	for page in $NO_PAGES; do
		if grep -q "^/usr/share/man/man1/$page\.1" <<<"$FILES"; then wrong "not to be there: the page $page.1"; fi
	done
	# The glibc the programs were linked with, and nothing for the extension:
	# Nautilus's libraries are there when it is loaded.
	local requires modes
	requires="$(rpm -qp --requires "$FILE" 2>/dev/null)"
	grep -q '^libc\.so\.6(GLIBC_' <<<"$requires" || wrong "Requires names no glibc: $requires"
	if grep -qi 'nautilus' <<<"$requires"; then wrong "Requires names Nautilus's libraries"; fi
	modes="$(rpm -qp --queryformat '[%{FILEMODES:perms} %{FILENAMES}\n]' "$FILE" 2>/dev/null)"
	for program in /usr/bin/packwing /usr/bin/pkw; do
		grep -q "^-rwxr-xr-x $program\$" <<<"$modes" || wrong "not executable: $program"
	done
	local rpm
	rpm="$(realpath "$FILE")"
	(cd "$WORK" && rpm2cpio "$rpm" | cpio -idm --quiet ./usr/share/licenses/packwing/THIRD-PARTY-LICENSES)
	licenses_name_pkws_crates "$WORK/usr/share/licenses/packwing/THIRD-PARTY-LICENSES"
	done_checking

	$INSTALL || return 0
	# Installed as on Fedora, with the dependencies it names; then pkw run.
	local dir name
	dir="$(cd "$(dirname "$FILE")" && pwd)"
	name="$(basename "$FILE")"
	docker run --rm -v "$dir:/dist:ro" -v "$(realpath "$0"):/check.sh:ro" fedora:latest bash -c "
		set -e
		dnf install -y -q /dist/$name grep >/dev/null
		echo '  installed in Fedora'
		pkw --version
		/check.sh --round-trip pkw
		bash -c 'source /usr/share/bash-completion/completions/pkw && complete -p pkw' >/dev/null
		rpm -V packwing
	"
	echo "  pkw runs from the package"
}

check_cli() {
	local name dir pkw
	name="$(basename "$FILE" .tar.gz)"
	FILES="$(tar -tzf "$FILE" | sed 's|/$||')"
	for file in pkw README.txt LICENSE GPL-ADDITIONAL-PERMISSION.md THIRD-PARTY-LICENSES \
		completions/pkw.bash completions/_pkw completions/pkw.fish; do
		has "$name/$file"
	done
	for page in $PAGES; do has "$name/man/$page.1"; done
	for page in $NO_PAGES; do hasnt "$name/man/$page.1"; done
	tar -xzf "$FILE" -C "$WORK"
	dir="$WORK/$name"
	pkw="$dir/pkw"
	[ -x "$pkw" ] || wrong "not executable: pkw"
	licenses_name_pkws_crates "$dir/THIRD-PARTY-LICENSES"
	# For servers: no window system, nor anything but the C library's own.
	local needed
	needed="$(readelf -d "$pkw" | sed -n 's/.*(NEEDED).*\[\(.*\)\]/\1/p')"
	while read -r lib; do
		case "$lib" in
		libc.so.* | libm.so.* | libgcc_s.so.* | libdl.so.* | libpthread.so.* | librt.so.* | ld-linux*) ;;
		*) wrong "pkw links $lib" ;;
		esac
	done <<<"$needed"
	done_checking

	"$pkw" --version
	pkw_round_trip "$pkw"
	bash -c "source '$dir/completions/pkw.bash' && complete -p pkw" >/dev/null ||
		{ echo "  bash does not load pkw's completions" >&2; exit 1; }
	echo "  pkw runs from where it is unpacked"
}

if $ROUND_TRIP; then
	pkw_round_trip "$FILE"
	exit 0
fi
case "$FILE" in
*.deb) echo "$FILE"; check_deb ;;
*.rpm) echo "$FILE"; check_rpm ;;
*.tar.gz) echo "$FILE"; check_cli ;;
*) echo "not a .deb, .rpm or .tar.gz: $FILE" >&2; exit 1 ;;
esac
