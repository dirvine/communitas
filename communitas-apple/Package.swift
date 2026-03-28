// swift-tools-version: 5.10
import PackageDescription

let package = Package(
    name: "Communitas",
    platforms: [.macOS(.v14)],
    products: [
        .executable(name: "Communitas", targets: ["Communitas"]),
        .library(name: "X0xClient", targets: ["X0xClient"]),
    ],
    targets: [
        .executableTarget(name: "Communitas", dependencies: ["X0xClient"]),
        .target(name: "X0xClient"),
        .testTarget(name: "X0xClientTests", dependencies: ["X0xClient"]),
    ]
)
