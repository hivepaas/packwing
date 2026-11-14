// swift-tools-version:5.9
// StoreKit 2 for packwing's Mac App Store build: a static library of C functions
// (Sources/PackwingStore/Store.swift) that crates/store's build.rs builds and links.

import PackageDescription

let package = Package(
    name: "PackwingStore",
    platforms: [.macOS(.v13)],
    products: [
        .library(name: "PackwingStore", type: .static, targets: ["PackwingStore"]),
    ],
    targets: [
        .target(name: "PackwingStore"),
    ]
)
