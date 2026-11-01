//! packwing as it is published: see the library for what it does.

// A GUI program on Windows: no console window opens beside it.
#![cfg_attr(windows, windows_subsystem = "windows")]

fn main() {
    packwing_app::run(packwing_app::Extensions::default());
}
