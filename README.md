# packwing

**A file archiver for macOS, Windows and Linux that opens archives safely.**

Built in Rust on [hivepack](https://github.com/hivepaas/hivepack-rs).

> **Status:** early development. There is nothing to download yet.

## 🎯 Plans

- **Formats:** ZIP (split archives too), RAR (reading), tar and `.tar.zst`.
- **Safe extraction by default:** nothing is written outside the destination folder or through links, and archive bombs stop at a size limit.

## 🐝 Made by HivePaaS

packwing is built and maintained by HivePaaS, who also make:

- [hivepaas](https://github.com/hivepaas/hivepaas): a self-hosted PaaS.
- [shellwing](https://shellwing.app): a fast, native SSH client for macOS, Windows and Linux.
- [authwing](https://github.com/hivepaas/authwing): a two-factor code app for macOS, Windows and Linux that knows the window you are in.

## 📜 License

packwing is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License, version 3 or (at your option) any later version. See [LICENSE](LICENSE). Copyright (C) 2026 HivePaaS.

The name and logo are not covered by the GPL: see [TRADEMARKS.md](TRADEMARKS.md). Contributions are welcome under the terms in [CONTRIBUTING.md](CONTRIBUTING.md).
