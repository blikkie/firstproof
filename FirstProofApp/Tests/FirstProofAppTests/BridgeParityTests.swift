// SPDX-License-Identifier: MIT
// Copyright 2026 First Proof Developers

import Testing
import Foundation
import FirstProofBridge
@testable import FirstProofApp

@Suite("Swift Bridge & Domain Model Parity Invariants")
struct BridgeParityTests {
    
    @Test("EditSettings.defaultIdentity matches sensitometric neutral baseline")
    func testDefaultEditSettingsBaseline() {
        let edits = EditSettings.defaultIdentity
        
        #expect(edits.exposure == 0.0)
        #expect(edits.contrast == 0.0)
        #expect(edits.highlights == 0.0)
        #expect(edits.shadows == 0.0)
        #expect(edits.whites == 0.0)
        #expect(edits.blacks == 0.0)
        #expect(edits.temperature == 5500.0) // Daylight D55 neutral
        #expect(edits.tint == 0.0)
        
        // Geometry defaults
        #expect(edits.cropLeft == 0.0)
        #expect(edits.cropTop == 0.0)
        #expect(edits.cropRight == 1.0)
        #expect(edits.cropBottom == 1.0)
        #expect(edits.cropAngle == 0.0)
        #expect(edits.rotation == 0)
        #expect(!edits.flipHorizontal)
        #expect(!edits.flipVertical)
        
        // Optics defaults
        #expect(!edits.lensCorrectionEnabled)
        #expect(edits.tcaCorrectionEnabled)
        #expect(!edits.vignettingCorrectionEnabled)
    }
    
    @Test("PhotoRecord distinguishes RAW vs raster and proprietary vs open DNG")
    func testPhotoRecordFormatClassification() {
        let cr2 = PhotoRecord.mock(path: "/photos/IMG_0001.CR2")
        let nef = PhotoRecord.mock(path: "/photos/DSC_0001.NEF")
        let arw = PhotoRecord.mock(path: "/photos/SONY_001.ARW")
        let dng = PhotoRecord.mock(path: "/photos/IMG_0001.dng")
        let jpg = PhotoRecord.mock(path: "/photos/IMG_0001.JPG")
        let png = PhotoRecord.mock(path: "/photos/preview.png")
        
        // isRaw checks
        #expect(cr2.isRaw)
        #expect(nef.isRaw)
        #expect(arw.isRaw)
        #expect(dng.isRaw)
        #expect(!jpg.isRaw)
        #expect(!png.isRaw)
        
        // isProprietaryRaw checks: DNG is open-standard archival container
        #expect(cr2.isProprietaryRaw)
        #expect(nef.isProprietaryRaw)
        #expect(arw.isProprietaryRaw)
        #expect(!dng.isProprietaryRaw)
        #expect(!jpg.isProprietaryRaw)
        #expect(!png.isProprietaryRaw)
    }
    
    @Test("ToneCurvesRecord.defaultIdentity strictly enforces boundary anchoring at (0,0) and (1,1)")
    func testToneCurvesBoundaryAnchoring() {
        let curves = ToneCurvesRecord.defaultIdentity
        
        for (channelName, points) in [
            ("Master", curves.master),
            ("Red", curves.red),
            ("Green", curves.green),
            ("Blue", curves.blue)
        ] {
            #expect(points.count >= 2, "Channel \(channelName) must have at least 2 control points")
            #expect(points.first?.x == 0.0, "Channel \(channelName) must start at x=0.0")
            #expect(points.first?.y == 0.0, "Channel \(channelName) must start at y=0.0")
            #expect(points.last?.x == 1.0, "Channel \(channelName) must end at x=1.0")
            #expect(points.last?.y == 1.0, "Channel \(channelName) must end at y=1.0")
        }
    }
}
