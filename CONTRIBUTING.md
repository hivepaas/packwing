# Contributing to packwing

Thank you for helping. Issues and pull requests are welcome; for anything large,
open an issue first so we can agree on the shape before you write it.

## Before a pull request

- `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`
  and `cargo test --workspace` pass.
- Changes people see come with a line in the README when they change how
  something is used.

## The Rust version

`rust-toolchain.toml` pins the Rust every build uses, and `rust-version` in
`Cargo.toml` follows it, so a new Rust never brings new clippy warnings on its
own. Rust ships every six weeks; take every second release (about every twelve
weeks), or one sooner when a fix or a feature in it is needed, once it is two
weeks old (its point release, if any, is out by then). Raise both in a pull
request of its own, with what the new clippy finds fixed.

## UI scenarios

`crates/app/src/ui_tests` drives the real window as a user does, with clicks and typing, on real files, with no display: `make test-ui`, or `make test-ui FILTER=create::`. Each scenario runs in a process of its own with a scratch home, so none touches your settings or keychain. A click finds its element by the `.debug_selector(..)` the widgets give their ids; a `Select` made `.marked("format")` is opened as `select: format`, and `ui.choose("format", ".7z")` picks an option by its name. On CI they run on Linux for every pull request, with the format, clippy and every test (`.github/workflows/ci.yml`), and on Linux, macOS and Windows daily and by hand (Actions > UI tests); a pull request that changes them runs those too. To see one fail in detail: `cargo test -p packwing --features ui-tests --exact ui_tests::<path>::scenario --ignored --nocapture`, with `PACKWING_UI_TEST_DIR` set to a folder.

## Words on the screen

Say them through `tr!("Open")`, `tr!("Could not read {path}", path = p)` or `tr_n!(n, "{n} file", "{n} files")`, so that they can be translated, and run `scripts/i18n.py update` to bring `crates/i18n/locales/en.ftl` up to date; `scripts/i18n.py check` says if it is out of step. packwing speaks 15 languages. New words stay English in each until they are translated: `scripts/i18n.py todo <tag>` lists what is left, and `scripts/i18n.py apply <tag> <file>` takes the translations. A language takes a line in each of `LANGUAGES` and `CATALOGS` in `crates/i18n/src/lib.rs`, and its `locales/<tag>.ftl`. To see what one does to the layout: `SHOTS_LANGUAGE=<tag> cargo test -p packwing --no-default-features --features wayland,shots shots::shots`, which draws to `target/shots/<tag>`. Text sizes are written for body text at 15 px, in rems (`text_size::ui`), and the window's rem follows the system's UI font, or Settings' Text size: `SHOTS_TEXT_SIZE=12` draws at Windows's, to `target/shots/12px`.

## Licensing your contribution

packwing is licensed under GPL-3.0-or-later and maintained by HivePaaS, who
also ships builds where the GPL cannot be used as is (for example on app
stores). So that this stays possible, by submitting a contribution you agree
that:

1. You wrote it, or otherwise have the right to submit it under these terms
   (the [Developer Certificate of Origin 1.1](https://developercertificate.org/)).
2. It is licensed to everyone under GPL-3.0-or-later, like the rest of the code.
3. You also grant HivePaaS a perpetual, worldwide, non-exclusive, royalty-free,
   irrevocable license to use, modify and distribute it under other terms as
   well, including proprietary ones, and a license to any of your patents that
   it necessarily practises.

You keep the copyright in your work. Sign off your commits (`git commit -s`) to
confirm the above.
