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
            // Debounce by 24ms for responsive updates during slider scrubbing
            try? await Task.sleep(nanoseconds: 24_000_000)
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
            let rawRecord = self.rawPreviewRecord
            let path = self.selectedPhoto?.filePath
            
            // Decoupled asynchronous background calculation
            let record: EvHistogramRecord? = await Task.detached(priority: .utility) {
                if let raw = rawRecord, raw.width > 0, raw.height > 0 {
                    return computeEvHistogram(
                        width: raw.width, height: raw.height, data: raw.data,
                        exposure: exp, contrast: contrast, highlights: highlights,
                        shadows: shadows, whites: whites, blacks: blacks,
                        cropLeft: cropLeft, cropTop: cropTop,
                        cropRight: cropRight, cropBottom: cropBottom,
                        lensCorrectionEnabled: lensCorrectionEnabled,
                        tcaCorrectionEnabled: tcaCorrectionEnabled
                    )
                } else if let filePath = path {
                    return computeEvHistogramFromPath(
                        path: filePath, exposure: exp, contrast: contrast,
                        highlights: highlights, shadows: shadows, whites: whites, blacks: blacks,
                        cropLeft: cropLeft, cropTop: cropTop,
                        cropRight: cropRight, cropBottom: cropBottom,
                        lensCorrectionEnabled: lensCorrectionEnabled,
                        tcaCorrectionEnabled: tcaCorrectionEnabled
                    )
                }
                return nil
            }.value
            
            guard !Task.isCancelled else { return }
            self.histogramRecord = record
        }
    }
}
