// swift-tools-version: 5.9
import Foundation
import PackageDescription

let packageDir = URL(fileURLWithPath: #filePath).deletingLastPathComponent().path

let package = Package(
    name: "FirstProofBridge",
    platforms: [
        .macOS(.v13)
    ],
    products: [
        .library(
            name: "FirstProofBridge",
            targets: ["FirstProofBridge", "FirstProofShaderTypes"]
        )
    ],
    targets: [
        .target(
            name: "FirstProofShaderTypes",
            path: "Sources/FirstProofShaderTypes",
            publicHeadersPath: "include"
        ),
        .target(
            name: "FirstProofCoreFFI",
            path: "Sources/FirstProofCoreFFI",
            publicHeadersPath: "include"
        ),
        .target(
            name: "FirstProofBridge",
            dependencies: ["FirstProofCoreFFI", "FirstProofShaderTypes"],
            path: "Sources/FirstProofBridge",
            linkerSettings: [
                .unsafeFlags([
                    "-L\(packageDir)/Sources/FirstProofCoreFFI/include",
                    "-lfirstproof_core"
                ]),
                .linkedFramework("Security"),
                .linkedFramework("CoreFoundation")
            ]
        )
    ]
)
