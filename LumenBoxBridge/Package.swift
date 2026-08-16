// swift-tools-version: 5.9
import Foundation
import PackageDescription

let packageDir = URL(fileURLWithPath: #filePath).deletingLastPathComponent().path

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
                .unsafeFlags([
                    "-L\(packageDir)/Sources/LumenBoxCoreFFI/include",
                    "-llumenbox_core"
                ]),
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
