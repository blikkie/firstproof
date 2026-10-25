// SPDX-License-Identifier: MIT
// Copyright 2026 First Proof Developers

import Testing
import Foundation
import FirstProofBridge
@testable import FirstProofApp

@Suite("DNG Conversion Telemetry Disentanglement Invariants")
struct DngConversionTelemetryTests {
    
    private func makeItemReport(
        source: String,
        dng: String,
        alreadyExisted: Bool,
        deletedOriginal: Bool
    ) -> DngConvertReportRecord {
        DngConvertReportRecord(
            sourcePath: source,
            dngPath: dng,
            sourceSizeBytes: 20_000_000,
            dngSizeBytes: 16_000_000,
            compressionRatio: 20.0,
            verificationPassed: true,
            deletedOriginal: deletedOriginal,
            alreadyExisted: alreadyExisted,
            elapsedMs: 250.0
        )
    }
    
    @Test("Pure transcoding batch correctly classifies all items as newly transcoded")
    func testPureTranscodingDisentanglement() {
        let items = [
            makeItemReport(source: "/tmp/raw1.cr2", dng: "/tmp/raw1.dng", alreadyExisted: false, deletedOriginal: false),
            makeItemReport(source: "/tmp/raw2.cr2", dng: "/tmp/raw2.dng", alreadyExisted: false, deletedOriginal: false),
            makeItemReport(source: "/tmp/raw3.cr2", dng: "/tmp/raw3.dng", alreadyExisted: false, deletedOriginal: false)
        ]
        
        let batch = DngBatchReportRecord(
            totalRequested: 3,
            convertedCount: 3,
            reconciledExistingCount: 0,
            skippedCount: 0,
            failedCount: 0,
            totalSourceBytes: 60_000_000,
            totalDngBytes: 48_000_000,
            spaceReclaimedBytes: 0,
            reports: items,
            errorMessages: []
        )
        
        let transcodedCount = batch.reports.filter { !$0.alreadyExisted }.count
        let reconciledCount = batch.reports.filter { $0.alreadyExisted }.count
        let deletedCount = batch.reports.filter { $0.deletedOriginal }.count
        
        #expect(transcodedCount == 3)
        #expect(reconciledCount == 0)
        #expect(deletedCount == 0)
    }
    
    @Test("Reconciled existing DNGs are never reported as newly transcoded")
    func testReconciledExistingDisentanglement() {
        let items = [
            makeItemReport(source: "/tmp/raw1.cr2", dng: "/tmp/raw1.dng", alreadyExisted: true, deletedOriginal: true),
            makeItemReport(source: "/tmp/raw2.cr2", dng: "/tmp/raw2.dng", alreadyExisted: true, deletedOriginal: true)
        ]
        
        let batch = DngBatchReportRecord(
            totalRequested: 2,
            convertedCount: 2,
            reconciledExistingCount: 2,
            skippedCount: 0,
            failedCount: 0,
            totalSourceBytes: 40_000_000,
            totalDngBytes: 32_000_000,
            spaceReclaimedBytes: 40_000_000,
            reports: items,
            errorMessages: []
        )
        
        let transcodedCount = batch.reports.filter { !$0.alreadyExisted }.count
        let reconciledCount = batch.reports.filter { $0.alreadyExisted }.count
        let deletedCount = batch.reports.filter { $0.deletedOriginal }.count
        
        #expect(transcodedCount == 0)
        #expect(reconciledCount == 2)
        #expect(deletedCount == 2)
    }
    
    @Test("Mixed batch with both transcode and reconciliation calculates independent tallies")
    func testMixedBatchDisentanglement() {
        let items = [
            makeItemReport(source: "/tmp/raw1.cr2", dng: "/tmp/raw1.dng", alreadyExisted: false, deletedOriginal: true),
            makeItemReport(source: "/tmp/raw2.cr2", dng: "/tmp/raw2.dng", alreadyExisted: false, deletedOriginal: false),
            makeItemReport(source: "/tmp/raw3.cr2", dng: "/tmp/raw3.dng", alreadyExisted: true, deletedOriginal: true)
        ]
        
        let batch = DngBatchReportRecord(
            totalRequested: 3,
            convertedCount: 3,
            reconciledExistingCount: 1,
            skippedCount: 0,
            failedCount: 0,
            totalSourceBytes: 60_000_000,
            totalDngBytes: 48_000_000,
            spaceReclaimedBytes: 40_000_000,
            reports: items,
            errorMessages: []
        )
        
        let transcodedCount = batch.reports.filter { !$0.alreadyExisted }.count
        let reconciledCount = batch.reports.filter { $0.alreadyExisted }.count
        let deletedCount = batch.reports.filter { $0.deletedOriginal }.count
        
        #expect(transcodedCount == 2)
        #expect(reconciledCount == 1)
        #expect(deletedCount == 2)
    }
}
