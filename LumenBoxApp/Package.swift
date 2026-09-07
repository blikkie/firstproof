// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "LumenBoxApp",
    platforms: [
        .macOS(.v14)
    ],
    products: [
        .executable(name: "LumenBoxApp", targets: ["LumenBoxApp"])
    ],
    dependencies: [
        .package(path: "../LumenBoxBridge")
    ],
    targets: [
        .executableTarget(
            name: "LumenBoxApp",
            dependencies: [
                .product(name: "LumenBoxBridge", package: "LumenBoxBridge")
            ],
            path: "Sources",
            exclude: [
                "Shaders"
            ]
        )
    ]
)
