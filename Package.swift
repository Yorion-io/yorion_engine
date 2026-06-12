// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "YorionEngine",
    platforms: [
        .macOS(.v12),
        .iOS(.v15)
    ],
    products: [
        .library(
            name: "YorionEngine",
            targets: ["YorionEngine"]
        ),
    ],
    targets: [
        // Binary target containing the Rust library
        .binaryTarget(
            name: "YorionEngineFFI",
            path: "dist/YorionEngine.xcframework"
        ),
        // Swift wrapper target
        .target(
            name: "YorionEngine",
            dependencies: ["YorionEngineFFI"],
            path: "dist/swift",
            sources: ["yorion_engine.swift"]
        ),
    ]
)
