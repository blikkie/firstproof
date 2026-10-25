// SPDX-License-Identifier: MIT
// Copyright 2026 First Proof Developers

import Foundation
import FirstProofBridge
@testable import FirstProofApp

extension PhotoRecord {
    public static func mock(
        id: Int64 = 1,
        path: String,
        rating: Int32? = nil,
        status: String = "unset"
    ) -> PhotoRecord {
        let name = URL(fileURLWithPath: path).lastPathComponent
        return PhotoRecord(
            id: id,
            filePath: path,
            fileName: name,
            fileSize: 1024 * 1024,
            modifiedTime: 1700000000,
            cameraMake: "Canon",
            cameraModel: "Canon EOS 60D",
            lensModel: "EF 50mm f/1.8",
            iso: 100,
            aperture: 1.8,
            shutterSpeed: "1/250",
            focalLength: 50.0,
            takenTime: 1700000000,
            latitude: nil,
            longitude: nil,
            title: nil,
            description: nil,
            creator: nil,
            copyright: nil,
            keywords: nil,
            rating: rating,
            status: status,
            hasXmp: false,
            hasTakeout: false,
            createdAt: 1700000000,
            updatedAt: 1700000000
        )
    }
}
