// swift-tools-version: 5.10
import PackageDescription

let package = Package(
    name: "CommuniTas",
    platforms: [.macOS(.v14)],
    products: [
        .executable(name: "CommuniTas", targets: ["CommuniTas"]),
        .library(name: "X0xClient", targets: ["X0xClient"]),
    ],
    targets: [
        .executableTarget(name: "CommuniTas", dependencies: ["X0xClient"]),
        .target(name: "X0xClient"),
        .testTarget(name: "X0xClientTests", dependencies: ["X0xClient"]),
    ]
)
