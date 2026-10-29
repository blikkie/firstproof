// SPDX-License-Identifier: MIT
// Copyright 2026 First Proof Developers

import Testing
import Foundation
import FirstProofBridge
@testable import FirstProofApp

@Suite("ColorSync Soft Proofing & Lab ICC Profile Management Invariants")
@MainActor
struct SoftProofTests {
    
    init() {
        UserDefaults.standard.removeObject(forKey: "defaultProofIntent")
        UserDefaults.standard.removeObject(forKey: "defaultProofProfileId")
    }
    
    @Test("AppViewModel initializes with soft proofing inactive and discovers built-in profiles")
    func testDefaultStateAndDiscovery() {
        UserDefaults.standard.removeObject(forKey: "defaultProofIntent")
        UserDefaults.standard.removeObject(forKey: "defaultProofProfileId")
        let vm = AppViewModel()
        
        #expect(!vm.isSoftProofActive, "Soft proofing must be off by default")
        #expect(!vm.isGamutWarningActive, "Gamut warning alarm must be off by default")
        #expect(vm.proofRenderingIntent == .relativeColorimetric, "Rendering intent must default to Relative Colorimetric")
        
        let hasSrgb = vm.availableIccProfiles.contains(where: { $0.id == "builtin:srgb" })
        let hasP3 = vm.availableIccProfiles.contains(where: { $0.id == "builtin:display_p3" })
        #expect(hasSrgb, "Built-in sRGB profile must always be discovered")
        #expect(hasP3, "Built-in Display P3 profile must always be discovered")
    }
    
    @Test("Toggling soft proofing updates active flag and cascadingly disables gamut warning")
    func testToggleSoftProof() {
        let vm = AppViewModel()
        
        #expect(!vm.isSoftProofActive)
        vm.toggleSoftProof()
        #expect(vm.isSoftProofActive, "toggleSoftProof must turn on soft proofing")
        
        vm.toggleGamutWarning()
        #expect(vm.isGamutWarningActive, "Gamut warning should be active")
        
        // Turning off soft proofing must cascadingly turn off gamut warning
        vm.toggleSoftProof()
        #expect(!vm.isSoftProofActive)
        #expect(!vm.isGamutWarningActive, "Disabling soft proofing must deactivate gamut warning")
    }
    
    @Test("Toggling gamut warning automatically turns on soft proofing if inactive")
    func testGamutWarningEnablesSoftProof() {
        let vm = AppViewModel()
        
        #expect(!vm.isSoftProofActive)
        #expect(!vm.isGamutWarningActive)
        
        vm.toggleGamutWarning()
        #expect(vm.isGamutWarningActive)
        #expect(vm.isSoftProofActive, "Activating gamut warning must auto-activate soft proofing")
        
        vm.toggleGamutWarning()
        #expect(!vm.isGamutWarningActive)
        #expect(vm.isSoftProofActive, "Deactivating gamut warning leaves soft proofing active")
    }
    
    @Test("Profile selection and rendering intent updates state predictably")
    func testProfileAndIntentSelection() {
        let vm = AppViewModel()
        
        vm.selectProofProfile(id: "builtin:display_p3")
        #expect(vm.selectedProofProfileId == "builtin:display_p3")
        #expect(vm.selectedProofProfileTitle == "Display P3")
        
        vm.setProofRenderingIntent(.perceptual)
        #expect(vm.proofRenderingIntent == .perceptual)
        
        vm.setProofRenderingIntent(.absoluteColorimetric)
        #expect(vm.proofRenderingIntent == .absoluteColorimetric)
        
        vm.selectProofProfile(id: "builtin:srgb")
        #expect(vm.selectedProofProfileId == "builtin:srgb")
        #expect(vm.selectedProofProfileTitle == "sRGB")
    }
    
    @Test("Direct LittleCMS2 3D proof LUT generation yields correct dimensions and byte size")
    func testDirectLutGeneration() throws {
        // Size 17: 17^3 = 4913 points * 16 bytes (RGBA32F) = 78,608 bytes
        let lut17 = try generateSoftProof3dLut(
            profileIdOrPath: "builtin:srgb",
            intent: .relativeColorimetric,
            sourceIsP3: true,
            displayIsP3: true,
            size: 17
        )
        
        let expected17Bytes = 17 * 17 * 17 * 16
        #expect(lut17.size == 17)
        #expect(lut17.dataRgba32f.count == expected17Bytes)
        
        // Size 33: 33^3 = 35937 points * 16 bytes = 574,992 bytes
        let lut33 = try generateSoftProof3dLut(
            profileIdOrPath: "builtin:display_p3",
            intent: .perceptual,
            sourceIsP3: true,
            displayIsP3: false,
            size: 33
        )
        
        let expected33Bytes = 33 * 33 * 33 * 16
        #expect(lut33.size == 33)
        #expect(lut33.dataRgba32f.count == expected33Bytes)
    }
}