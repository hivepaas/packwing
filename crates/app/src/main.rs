//! packwing, with the storages it can reach: see the library for what it does.

// A GUI program on Windows: no console window opens beside it.
#![cfg_attr(windows, windows_subsystem = "windows")]

fn main() {
    packwing_app::run(packwing_app::Extensions {
        storage_kinds: packwing_remote::kinds(),
    });
}
