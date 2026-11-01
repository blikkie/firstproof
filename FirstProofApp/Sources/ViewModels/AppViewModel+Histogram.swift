import SwiftUI
import FirstProofBridge

extension AppViewModel {
    // MARK: - Sensitometric EV Zone Histogram & Telemetry Actions
    
    public func toggleHistogram() {
        isHistogramVisible.toggle()
        if isHistogramVisible && histogramRecord == nil {
            scheduleHistogramUpdate()
        }
    }
    
    public func toggleClippingOverlay() {
        isClippingOverlayActive.toggle()
    }
    
    public func scheduleHistogramUpdate() {
        guard canvasMode == .detail, isHistogramVisible else { return }
        
        histogramDebounceTask?.cancel()
        histogramDebounceTask = Task { @MainActor [weak self] in
            // Fast 28ms debounce for responsive histogram updates lockstep with slider dragging.
            // Decoupled sub-millisecond calculation executes off the main actor via zero-copy active preview buffer.
            let debounceNs: UInt64 = self?.hasActiveSliderScrub == true ? 28_000_000 : 16_000_000
            try? await Task.sleep(nanoseconds: debounceNs)
            guard !Task.isCancelled, let self = self else { return }
            
            let exp = self.exposure
            let contrast = self.contrast
            let highlights = self.highlights
            let shadows = self.shadows
            let whites = self.whites
            let blacks = self.blacks
            let cropLeft = self.cropLeft
            let cropTop = self.cropTop
            let cropRight = self.cropRight
            let cropBottom = self.cropBottom
            let lensCorrectionEnabled = self.lensCorrectionEnabled
            let tcaCorrectionEnabled = self.tcaCorrectionEnabled
            let vignettingCorrectionEnabled = self.vignettingCorrectionEnabled
            let vigK1 = self.lensCoefficients?.vignetting.indices.contains(0) == true ? self.lensCoefficients!.vignetting[0] : 0.0
            let vigK2 = self.lensCoefficients?.vignetting.indices.contains(1) == true ? self.lensCoefficients!.vignetting[1] : 0.0
            let vigK3 = self.lensCoefficients?.vignetting.indices.contains(2) == true ? self.lensCoefficients!.vignetting[2] : 0.0
            
            let rawRecord = self.binnedRawRecord ?? self.rawPreviewRecord
            
            // Decoupled asynchronous background calculation off the main thread
            let record: EvHistogramRecord? = await Task.detached(priority: .utility) {
                // Primary path: zero-copy sub-millisecond evaluation (<0.2ms) on pre-cached Rust active preview buffer
                if let active = computeEvHistogramActive(
                    exposure: exp, contrast: contrast, highlights: highlights,
                    shadows: shadows, whites: whites, blacks: blacks,
                    cropLeft: cropLeft, cropTop: cropTop,
                    cropRight: cropRight, cropBottom: cropBottom,
                    lensCorrectionEnabled: lensCorrectionEnabled,
                    tcaCorrectionEnabled: tcaCorrectionEnabled,
                    vignettingCorrectionEnabled: vignettingCorrectionEnabled,
                    vignettingK1: vigK1,
                    vignettingK2: vigK2,
                    vignettingK3: vigK3
                ) {
                    return active
                }
                
                // Fallback for non-RAW images or unprimed active buffers
                guard let raw = rawRecord, raw.width > 0, raw.height > 0 else { return nil }
                return computeEvHistogram(
                    width: raw.width, height: raw.height, data: raw.data,
                    exposure: exp, contrast: contrast, highlights: highlights,
                    shadows: shadows, whites: whites, blacks: blacks,
                    cropLeft: cropLeft, cropTop: cropTop,
                    cropRight: cropRight, cropBottom: cropBottom,
                    lensCorrectionEnabled: lensCorrectionEnabled,
                    tcaCorrectionEnabled: tcaCorrectionEnabled,
                    vignettingCorrectionEnabled: vignettingCorrectionEnabled,
                    vignettingK1: vigK1,
                    vignettingK2: vigK2,
                    vignettingK3: vigK3
                )
            }.value
            
            guard !Task.isCancelled else { return }
            self.histogramRecord = record
        }
    }
}
