// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "LumenBoxBridge",
    platforms: [
        .macOS(.v13)
    ],
    products: [
        .library(
            name: "LumenBoxBridge",
            targets: ["LumenBoxBridge"]
        )
    ],
    targets: [
        .target(
            name: "LumenBoxCoreFFI",
            path: "Sources/LumenBoxCoreFFI",
            publicHeadersPath: "include",
            linkerSettings: [
                .unsafeFlags(["Sources/LumenBoxCoreFFI/include/liblumenbox_core.a"]),
                .linkedFramework("Security"),
                .linkedFramework("CoreFoundation")
            ]
        ),
        .target(
            name: "LumenBoxBridge",
            dependencies: ["LumenBoxCoreFFI"],
            path: "Sources/LumenBoxBridge"
        )
    ]
)
