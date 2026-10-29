import Foundation
import SwiftUI
import AppKit
import FirstProofBridge

extension AppViewModel {
    public func loadDefaultProofSettings() {
        if let defaultProfile = UserDefaults.standard.string(forKey: "defaultProofProfileId"), !defaultProfile.isEmpty {
            self.selectedProofProfileId = defaultProfile
        } else {
            self.selectedProofProfileId = "builtin:srgb"
        }
        
        if let intentRaw = UserDefaults.standard.string(forKey: "defaultProofIntent") {
            switch intentRaw {
            case "perceptual":
                self.proofRenderingIntent = .perceptual
            case "absoluteColorimetric":
                self.proofRenderingIntent = .absoluteColorimetric
            default:
                self.proofRenderingIntent = .relativeColorimetric
            }
        }
    }
    
    public func refreshAvailableProofProfiles() {
        let discovered = discoverColorSyncProfiles()
        self.availableIccProfiles = discovered
        
        // Ensure selected profile still exists or fallback to sRGB
        if !discovered.contains(where: { $0.id == selectedProofProfileId }) {
            self.selectedProofProfileId = "builtin:srgb"
        }
    }
    
    public var selectedProofProfile: IccProfileRecord? {
        availableIccProfiles.first(where: { $0.id == selectedProofProfileId })
    }
    
    public var selectedProofProfileTitle: String {
        guard let p = selectedProofProfile else { return "sRGB" }
        if p.id == "builtin:srgb" { return "sRGB" }
        if p.id == "builtin:display_p3" { return "Display P3" }
        return p.description
    }
    
    public func toggleSoftProof() {
        isSoftProofActive.toggle()
        if isSoftProofActive {
            if currentProofLutRecord == nil {
                refreshProofLut()
            }
        } else {
            isGamutWarningActive = false
        }
    }
    
    public func toggleGamutWarning() {
        isGamutWarningActive.toggle()
        if isGamutWarningActive && !isSoftProofActive {
            isSoftProofActive = true
            if currentProofLutRecord == nil {
                refreshProofLut()
            }
        }
    }
    
    public func selectProofProfile(id: String) {
        guard selectedProofProfileId != id || currentProofLutRecord == nil else { return }
        selectedProofProfileId = id
        UserDefaults.standard.set(id, forKey: "defaultProofProfileId")
        refreshProofLut()
    }
    
    public func setProofRenderingIntent(_ intent: ProofIntent) {
        guard proofRenderingIntent != intent || currentProofLutRecord == nil else { return }
        proofRenderingIntent = intent
        let raw: String
        switch intent {
        case .perceptual: raw = "perceptual"
        case .absoluteColorimetric: raw = "absoluteColorimetric"
        case .relativeColorimetric: raw = "relativeColorimetric"
        }
        UserDefaults.standard.set(raw, forKey: "defaultProofIntent")
        refreshProofLut()
    }
    
    public func importProofProfile(from url: URL) {
        importProfile(from: url)
    }

    public func importProfile(from url: URL) {
        do {
            let record = try importColorSyncProfile(sourcePath: url.path)
            refreshAvailableProofProfiles()
            selectProofProfile(id: record.id)
        } catch {
            print("[SoftProof] Failed to import ICC profile: \(error)")
        }
    }
    
    public func refreshProofLut() {
        proofLutTask?.cancel()
        
        let profileId = selectedProofProfileId
        let intent = proofRenderingIntent
        let sourceIsP3 = false
        let displayIsP3 = isHdrEnabled && (isScreenHdrCapable || edrHeadroom > 1.001)
        
        isProofLutGenerating = true
        proofLutTask = Task { @MainActor [weak self] in
            do {
                let lut = try await Task.detached(priority: .userInitiated) {
                    try generateSoftProof3dLut(
                        profileIdOrPath: profileId,
                        intent: intent,
                        sourceIsP3: sourceIsP3,
                        displayIsP3: displayIsP3,
                        size: 33
                    )
                }.value
                
                guard let self = self, self.selectedProofProfileId == profileId, self.proofRenderingIntent == intent else {
                    return
                }
                self.currentProofLutRecord = lut
                self.isProofLutGenerating = false
            } catch {
                print("[SoftProof] Failed to generate 3D proof LUT: \(error)")
                self?.isProofLutGenerating = false
            }
        }
    }
}