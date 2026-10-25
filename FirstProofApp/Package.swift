// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "FirstProofApp",
    platforms: [
        .macOS(.v14)
    ],
    products: [
        .executable(name: "FirstProofApp", targets: ["FirstProofApp"])
    ],
    dependencies: [
        .package(path: "../FirstProofBridge")
    ],
    targets: [
        .executableTarget(
            name: "FirstProofApp",
            dependencies: [
                .product(name: "FirstProofBridge", package: "FirstProofBridge")
            ],
            path: "Sources",
            exclude: [
                "Shaders"
            ]
        ),
        .testTarget(
            name: "FirstProofAppTests",
            dependencies: [
                "FirstProofApp",
                .product(name: "FirstProofBridge", package: "FirstProofBridge")
            ],
            path: "Tests"
        )
    ]
)
