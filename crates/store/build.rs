//! Builds swift/ (StoreKit 2 as C functions) into a static library for the Mac being
//! built for, and links it with the Swift runtime: the system's, in /usr/lib/swift,
//! and the toolchain's back-deployment libraries; `cfg(storekit)` says it is there.
//! Nothing is built without the `storekit` feature, or for any other system: there
//! src/lib.rs has no App Store.

use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=swift/Package.swift");
    println!("cargo:rerun-if-changed=swift/Sources");
    println!("cargo::rustc-check-cfg=cfg(storekit)");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("macos")
        || std::env::var_os("CARGO_FEATURE_STOREKIT").is_none()
    {
        return;
    }
    println!("cargo::rustc-cfg=storekit");
    let arch = match std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
        Ok("aarch64") => "arm64",
        Ok("x86_64") => "x86_64",
        other => panic!("no Swift for the architecture {other:?}"),
    };
    // packwing's macOS release builds for the oldest macOS it takes (Info.plist's
    // LSMinimumSystemVersion), as CI sets it.
    println!("cargo:rerun-if-env-changed=MACOSX_DEPLOYMENT_TARGET");
    let macos = std::env::var("MACOSX_DEPLOYMENT_TARGET").unwrap_or_else(|_| "13.0".to_owned());
    let triple = format!("{arch}-apple-macosx{macos}");
    let package =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("cargo sets it")).join("swift");
    let scratch = PathBuf::from(std::env::var("OUT_DIR").expect("cargo sets it")).join("swift");
    let swift_build = |more: &[&str]| {
        let mut command = Command::new("swift");
        command
            .args(["build", "-c", "release", "--triple", &triple])
            .arg("--package-path")
            .arg(&package)
            .arg("--scratch-path")
            .arg(&scratch)
            .args(more)
            // A build script's environment is cargo's, for Rust: not Swift's.
            .env_remove("SDKROOT");
        command
    };
    let built = swift_build(&[])
        .status()
        .expect("swift, from Xcode or its command line tools");
    assert!(built.success(), "swift build failed for {triple}");
    let bin = swift_build(&["--show-bin-path"])
        .output()
        .expect("swift build --show-bin-path");
    let bin = String::from_utf8(bin.stdout).expect("a path");
    println!("cargo:rustc-link-search=native={}", bin.trim());
    println!("cargo:rustc-link-lib=static=PackwingStore");

    // Where the libraries the Swift code names (by autolink) are: the SDK's first, whose
    // runtime is the system's (/usr/lib/swift, there since macOS 12 with concurrency), so
    // the program needs no rpath; then the toolchain's, for its static compatibility
    // libraries, whose dylibs would be found by @rpath.
    let sdk = Command::new("xcrun")
        .args(["--sdk", "macosx", "--show-sdk-path"])
        .output()
        .expect("xcrun");
    let sdk = String::from_utf8(sdk.stdout).expect("a path");
    println!(
        "cargo:rustc-link-search=native={}/usr/lib/swift",
        sdk.trim()
    );
    let info = Command::new("swift")
        .args(["-target", &triple, "-print-target-info"])
        .output()
        .expect("swift -print-target-info");
    let info: serde_json::Value = serde_json::from_slice(&info.stdout).expect("its JSON");
    for path in info["paths"]["runtimeLibraryPaths"]
        .as_array()
        .expect("runtimeLibraryPaths")
    {
        if let Some(path) = path.as_str() {
            println!("cargo:rustc-link-search=native={path}");
        }
    }
    println!("cargo:rustc-link-lib=framework=StoreKit");
    println!("cargo:rustc-link-lib=framework=Foundation");
}
