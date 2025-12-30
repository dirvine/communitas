// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "CommunitasApp",
    platforms: [
        .macOS(.v14),
        .iOS(.v17)
    ],
    products: [
        .library(
            name: "CommunitasAppLib",
            targets: ["CommunitasAppLib"]
        ),
        .executable(
            name: "CommunitasApp",
            targets: ["CommunitasApp"]
        )
    ],
    dependencies: [
        .package(path: "../CommunitasKit"),
        .package(url: "https://github.com/sparkle-project/Sparkle", from: "2.6.0")
    ],
    targets: [
        // Library target containing shared services and components
        .target(
            name: "CommunitasAppLib",
            dependencies: [
                .product(name: "CommunitasKit", package: "CommunitasKit"),
                .product(name: "Sparkle", package: "Sparkle")
            ],
            path: "Sources",
            exclude: ["main.swift", "Info.plist", "Generated"],
            resources: [
                .process("communitas-icon.png"),
                .process("communitas-splash.png"),
                .process("communitas-favicon.png")
            ],
            linkerSettings: [
                .linkedFramework("SystemConfiguration"),
                .linkedFramework("Security"),
                .linkedFramework("CoreFoundation"),
                .linkedFramework("Foundation"),
                .linkedLibrary("resolv"),
            ]
        ),
        // Executable target
        .executableTarget(
            name: "CommunitasApp",
            dependencies: [
                "CommunitasAppLib",
                .product(name: "CommunitasKit", package: "CommunitasKit")
            ],
            path: "Sources",
            sources: ["main.swift"],
            linkerSettings: [
                .linkedFramework("SystemConfiguration"),
                .linkedFramework("Security"),
                .linkedFramework("CoreFoundation"),
                .linkedFramework("Foundation"),
                .linkedLibrary("resolv"),
            ]
        ),
        // Test target
        .testTarget(
            name: "CommunitasAppTests",
            dependencies: [
                "CommunitasAppLib",
                .product(name: "CommunitasKit", package: "CommunitasKit")
            ],
            path: "Tests/CommunitasAppTests"
        )
    ]
)
