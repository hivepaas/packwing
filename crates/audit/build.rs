//! On macOS, the few lines of C that write to the unified log (src/macos/os_log.c).

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/macos/os_log.c");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        cc::Build::new()
            .file("src/macos/os_log.c")
            .compile("packwing_os_log");
    }
}
