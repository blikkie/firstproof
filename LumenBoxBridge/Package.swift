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
            targets: ["LumenBoxBridge", "LumenBoxShaderTypes"]
        )
    ],
    targets: [
        .target(
            name: "LumenBoxShaderTypes",
            path: "Sources/LumenBoxShaderTypes",
            publicHeadersPath: "include"
        ),
        .target(
            name: "LumenBoxCoreFFI",
            path: "Sources/LumenBoxCoreFFI",
            publicHeadersPath: "include"
        ),
        .target(
            name: "LumenBoxBridge",
            dependencies: ["LumenBoxCoreFFI", "LumenBoxShaderTypes"],
            path: "Sources/LumenBoxBridge",
            linkerSettings: [
                .unsafeFlags([
                    "-L\(packageDir)/Sources/LumenBoxCoreFFI/include",
                    "-llumenbox_core"
                ]),
                .linkedFramework("Security"),
                .linkedFramework("CoreFoundation")
            ]
        )
    ]
)
