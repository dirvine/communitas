// swift-tools-version: 5.10
import PackageDescription

let package = Package(
    name: "Communitas",
    platforms: [.macOS(.v14)],
    products: [
        .executable(name: "Communitas", targets: ["Communitas"]),
        .library(name: "X0xClient", targets: ["X0xClient"]),
    ],
    dependencies: [
        .package(url: "https://github.com/sparkle-project/Sparkle", from: "2.6.0"),
    ],
    targets: [
        .executableTarget(
            name: "Communitas",
            dependencies: [
                "X0xClient",
                .product(name: "Sparkle", package: "Sparkle"),
            ]
        ),
        .target(name: "X0xClient"),
        .testTarget(name: "X0xClientTests", dependencies: ["X0xClient"]),
        // XCUITest target — UI-level golden paths driven via XCUIApplication.
        // Run with `xcodebuild -scheme Communitas -destination 'platform=macOS'
        // -only-testing:CommunitasUITests test` against an Xcode-generated
        // project. `swift test` cannot host XCUITest (no app host), but the
        // target compiles so the assertions stay in the package.
        .testTarget(name: "CommunitasUITests", dependencies: ["X0xClient"]),
    ]
)
