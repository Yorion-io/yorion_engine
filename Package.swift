// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "BsCalendarCore",
    platforms: [
        .macOS(.v12),
        .iOS(.v15)
    ],
    products: [
        .library(
            name: "BsCalendarCore",
            targets: ["BsCalendarCore"]
        ),
    ],
    targets: [
        // Binary target containing the Rust library
        .binaryTarget(
            name: "BsCalendarCoreFFI",
            path: "dist/BsCalendarCore.xcframework"
        ),
        // Swift wrapper target
        .target(
            name: "BsCalendarCore",
            dependencies: ["BsCalendarCoreFFI"],
            path: "dist/swift",
            sources: ["bs_calendar_core.swift"]
        ),
    ]
)
