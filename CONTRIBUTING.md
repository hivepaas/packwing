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
