// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "CommunitasKit",
    platforms: [
        .iOS(.v17),
        .macOS(.v14)
    ],
    products: [
        .library(
            name: "CommunitasKit",
            targets: ["CommunitasKit"]),
    ],
    targets: [
        // Binary target for the pre-built XCFramework
        .binaryTarget(
            name: "communitas_bindingsFFI",
            path: "../Frameworks/CommunitasBindings.xcframework"
        ),
        // Main Swift target
        .target(
            name: "CommunitasKit",
            dependencies: ["communitas_bindingsFFI"],
            path: "Sources/CommunitasKit",
            linkerSettings: [
                .linkedFramework("SystemConfiguration"),
                .linkedFramework("Security"),
                .linkedFramework("CoreFoundation"),
                .linkedFramework("Foundation"),
                .linkedLibrary("resolv"),
            ]
        ),
        .testTarget(
            name: "CommunitasKitTests",
            dependencies: ["CommunitasKit"],
            path: "Tests/CommunitasKitTests"
        ),
    ]
)
