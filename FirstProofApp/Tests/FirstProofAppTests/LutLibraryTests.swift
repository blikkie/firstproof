// SPDX-License-Identifier: MIT
// Copyright 2026 First Proof Developers

import Testing
import Foundation
import FirstProofBridge
@testable import FirstProofApp

@Suite("Global 3D LUT Library Management & Default Look Preset Invariants")
@MainActor
struct LutLibraryTests {
    
    init() {
        UserDefaults.standard.removeObject(forKey: "defaultLookLutName")
        UserDefaults.standard.removeObject(forKey: "defaultLookLutIntensity")
    }
    
    @Test("Default look setting persists to UserDefaults and reads back cleanly")
    func testDefaultLookUserDefaultsPersistence() {
        UserDefaults.standard.removeObject(forKey: "defaultLookLutName")
        UserDefaults.standard.removeObject(forKey: "defaultLookLutIntensity")
        let vm = AppViewModel()
        
        #expect(vm.defaultLookLutName == nil)
        #expect(vm.defaultLookLutIntensity == 1.0)
        
        vm.defaultLookLutName = "Signature Astia"
        vm.defaultLookLutIntensity = 0.45
        
        #expect(UserDefaults.standard.string(forKey: "defaultLookLutName") == "Signature Astia")
        #expect(abs((UserDefaults.standard.object(forKey: "defaultLookLutIntensity") as? Float ?? 0.0) - 0.45) < 0.001)
        
        #expect(vm.defaultLookLutName == "Signature Astia")
        #expect(abs(vm.defaultLookLutIntensity - 0.45) < 0.001)
        
        // Clearing default look
        vm.defaultLookLutName = nil
        #expect(UserDefaults.standard.string(forKey: "defaultLookLutName") == nil)
        #expect(vm.defaultLookLutName == nil)
    }
    
    @Test("Reset all sliders applies default signature look when configured")
    func testResetAllSlidersWithDefaultLook() {
        UserDefaults.standard.removeObject(forKey: "defaultLookLutName")
        UserDefaults.standard.removeObject(forKey: "defaultLookLutIntensity")
        let vm = AppViewModel()
        
        vm.defaultLookLutName = "Classic Chrome 100"
        vm.defaultLookLutIntensity = 0.70
        
        // Set some edits
        vm.selectedLutName = "Other Look"
        vm.lutIntensity = 0.20
        vm.exposure = 1.5
        
        vm.resetAllSliders()
        
        #expect(vm.selectedLutName == "Classic Chrome 100", "Reset must apply the configured default look LUT")
        #expect(abs(vm.lutIntensity - 0.70) < 0.001, "Reset must apply the configured default look intensity")
        #expect(vm.exposure == 0.0)
    }
    
    @Test("Reset all sliders falls back to None (Bypass) when no default look is set")
    func testResetAllSlidersWithoutDefaultLook() {
        UserDefaults.standard.removeObject(forKey: "defaultLookLutName")
        UserDefaults.standard.removeObject(forKey: "defaultLookLutIntensity")
        let vm = AppViewModel()
        
        vm.selectedLutName = "Custom Look"
        vm.lutIntensity = 0.50
        
        vm.resetAllSliders()
        
        #expect(vm.selectedLutName == nil, "Reset must bypass LUT when no default is set")
        #expect(vm.lutIntensity == 1.0)
    }
    
    @Test("allLutTitles aggregates both built-in and globally managed custom LUTs")
    func testAllLutTitlesAggregation() {
        let vm = AppViewModel()
        vm.availableLutTitles = ["Builtin Look 1", "Builtin Look 2"]
        vm.availableGlobalLuts = [
            LutRecord(id: "global:portra.cube", title: "Kodak Portra 400", filePath: "/path/portra.cube", size: 33, isBuiltin: false),
            LutRecord(id: "global:tri_x.cube", title: "Kodak Tri-X 400", filePath: "/path/tri_x.cube", size: 33, isBuiltin: false)
        ]
        
        let all = vm.allLutTitles
        #expect(all.contains("Builtin Look 1"))
        #expect(all.contains("Builtin Look 2"))
        #expect(all.contains("Kodak Portra 400"))
        #expect(all.contains("Kodak Tri-X 400"))
    }
    
    @Test("Deleting active or default LUT cascades to clear selection and default settings")
    func testDeletingDefaultLutClearsSettings() {
        let vm = AppViewModel()
        
        let tmpDir = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        try? FileManager.default.createDirectory(at: tmpDir, withIntermediateDirectories: true)
        let sampleCube = tmpDir.appendingPathComponent("Test_Cascade.cube")
        let content = "TITLE \"Test Cascade\"\nLUT_3D_SIZE 2\nDOMAIN_MIN 0.0 0.0 0.0\nDOMAIN_MAX 1.0 1.0 1.0\n0.0 0.0 0.0\n0.5 0.5 0.5\n0.5 0.5 0.5\n1.0 1.0 1.0\n0.5 0.5 0.5\n1.0 1.0 1.0\n1.0 1.0 1.0\n1.0 1.0 1.0\n"
        try? content.write(to: sampleCube, atomically: true, encoding: .utf8)
        
        guard let imported = try? vm.importLutToLibrary(from: sampleCube) else {
            Issue.record("Failed to import test cube")
            return
        }
        
        vm.defaultLookLutName = imported.title
        vm.selectedLutName = imported.title
        
        #expect(vm.defaultLookLutName == "Test Cascade")
        #expect(vm.selectedLutName == "Test Cascade")
        
        vm.deleteLutFromLibrary(imported)
        
        #expect(vm.selectedLutName == nil, "Deleting active LUT must clear selectedLutName")
        #expect(vm.defaultLookLutName == nil, "Deleting default LUT must clear defaultLookLutName")
        
        try? FileManager.default.removeItem(at: tmpDir)
    }
    
    @Test("Selecting a LUT from bypass initializes strength to defaultLookLutIntensity")
    func testSelectingLutFromBypassAppliesDefaultIntensity() {
        let vm = AppViewModel()
        vm.defaultLookLutIntensity = 0.35
        vm.selectedLutName = nil
        vm.lutIntensity = 1.0
        
        vm.setSelectedLutName("Fujifilm 3513DI Print Stock")
        #expect(abs(vm.lutIntensity - 0.35) < 0.001, "Selecting LUT from bypass must adopt default strength")
        
        // Changing to a custom strength then switching LUTs keeps custom strength
        vm.lutIntensity = 0.60
        vm.setSelectedLutName("Kodak 2383")
        #expect(abs(vm.lutIntensity - 0.60) < 0.001, "Switching between active LUTs preserves custom dialed strength")
        
        // Reset slider resets to default intensity
        vm.resetSlider(.lutIntensity)
        #expect(abs(vm.lutIntensity - 0.35) < 0.001, "Reset slider must restore defaultLookLutIntensity")
    }
}