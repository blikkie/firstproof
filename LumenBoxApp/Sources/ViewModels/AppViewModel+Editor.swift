import Foundation
import SwiftUI
import AppKit
import CoreGraphics
import LumenBoxBridge

extension AppViewModel {
    // MARK: - Current Edits Snapshot
    public var currentEdits: EditSettings {
        EditSettings(
            exposure: self.exposure,
            contrast: self.contrast,
            highlights: self.highlights,
            shadows: self.shadows,
            whites: self.whites,
            blacks: self.blacks,
            vibrance: self.vibrance,
            saturation: self.saturation,
            deepBlue: self.deepBlue,
            foliage: self.foliage,
            temperature: self.temperature,
            tint: self.tint,
            cropLeft: self.cropLeft,
            cropTop: self.cropTop,
            cropRight: self.cropRight,
            cropBottom: self.cropBottom,
            cropAngle: self.straightenAngle,
            perspectiveVertical: self.perspectiveVertical,
            perspectiveHorizontal: self.perspectiveHorizontal,
            cropConstrain: self.cropConstrain,
            rotation: Int32(self.rotationDegrees),
            flipHorizontal: self.flipHorizontal,
            flipVertical: self.flipVertical,
            toneProfile: self.toneProfile,
            lutName: self.selectedLutName,
            lutIntensity: self.lutIntensity,
            sharpeningAmount: self.sharpeningAmount,
            sharpeningRadius: self.sharpeningRadius,
            sharpeningMasking: self.sharpeningMasking,
            noiseReduction: self.noiseReduction,
            grainAmount: self.grainAmount,
            grainSize: self.grainSize,
            grainRoughness: self.grainRoughness,
            grainChroma: self.grainChroma,
            curves: self.curves
        )
    }
}

extension EditSettings {
    public static var defaultIdentity: EditSettings {
        EditSettings(
            exposure: 0.0,
            contrast: 0.0,
            highlights: 0.0,
            shadows: 0.0,
            whites: 0.0,
            blacks: 0.0,
            vibrance: 0.0,
            saturation: 0.0,
            deepBlue: 0.0,
            foliage: 0.0,
            temperature: 5000.0,
            tint: 0.0,
            cropLeft: 0.0,
            cropTop: 0.0,
            cropRight: 1.0,
            cropBottom: 1.0,
            cropAngle: 0.0,
            perspectiveVertical: 0.0,
            perspectiveHorizontal: 0.0,
            cropConstrain: true,
            rotation: 0,
            flipHorizontal: false,
            flipVertical: false,
            toneProfile: .agxVivid,
            lutName: nil,
            lutIntensity: 1.0,
            sharpeningAmount: 0.0,
            sharpeningRadius: 1.0,
            sharpeningMasking: 0.0,
            noiseReduction: 0.0,
            grainAmount: 0.0,
            grainSize: 1.0,
            grainRoughness: 50.0,
            grainChroma: 0.0,
            curves: ToneCurvesRecord.defaultIdentity
        )
    }
}

extension AppViewModel {
    public func applyEdits(_ edits: EditSettings) {
        self.exposure = edits.exposure
        self.contrast = edits.contrast
        self.highlights = edits.highlights
        self.shadows = edits.shadows
        self.whites = edits.whites
        self.blacks = edits.blacks
        self.vibrance = edits.vibrance
        self.saturation = edits.saturation
        self.deepBlue = edits.deepBlue
        self.foliage = edits.foliage
        self.temperature = edits.temperature
        self.tint = edits.tint
        self.cropLeft = edits.cropLeft
        self.cropTop = edits.cropTop
        self.cropRight = edits.cropRight
        self.cropBottom = edits.cropBottom
        self.straightenAngle = edits.cropAngle
        self.perspectiveVertical = edits.perspectiveVertical
        self.perspectiveHorizontal = edits.perspectiveHorizontal
        self.cropConstrain = edits.cropConstrain
        self.rotationDegrees = Int(edits.rotation)
        self.flipHorizontal = edits.flipHorizontal
        self.flipVertical = edits.flipVertical
        self.toneProfile = edits.toneProfile
        self.selectedLutName = edits.lutName
        self.lutIntensity = edits.lutIntensity
        self.sharpeningAmount = edits.sharpeningAmount
        self.sharpeningRadius = edits.sharpeningRadius
        self.sharpeningMasking = edits.sharpeningMasking
        self.noiseReduction = edits.noiseReduction
        self.grainAmount = edits.grainAmount
        self.grainSize = edits.grainSize
        self.grainRoughness = edits.grainRoughness
        self.grainChroma = edits.grainChroma
        self.curves = edits.curves
        
        // Update or clear selectedGrainPresetId matching the loaded photo's grain settings
        if edits.grainAmount <= 0.0 {
            self.selectedGrainPresetId = nil
        } else if let matchingPreset = FilmGrainPreset.builtInPresets.first(where: {
            abs($0.amount - edits.grainAmount) < 0.1 &&
            abs($0.size - edits.grainSize) < 0.05 &&
            abs($0.roughness - edits.grainRoughness) < 0.1 &&
            abs($0.chroma - edits.grainChroma) < 0.1
        }) {
            self.selectedGrainPresetId = matchingPreset.id
        } else {
            self.selectedGrainPresetId = nil
        }
    }
    
    public func saveCurrentEdits() {
        guard let path = currentlyLoadedPhotoPath ?? selectedPhoto?.filePath else { return }
        let editsSnapshot = currentEdits
        
        // Real-time sub-millisecond GPU thumbnail capture for active viewport
        if let snapshot = captureThumbnailSnapshot?() {
            ThumbnailCache.shared.saveEditedThumbnail(snapshot, for: path, edits: editsSnapshot)
        } else {
            // Fallback for edits outside active Metal canvas
            ThumbnailCache.shared.removeImage(for: path)
            NotificationCenter.default.post(name: .thumbnailDidChange, object: path)
        }
        
        DispatchQueue.global(qos: .utility).async { [weak self] in
            self?.syncXmp(for: path, rating: nil, status: nil, edits: editsSnapshot)
        }
    }
    
    public var hasCustomEdits: Bool {
        editsHasModifications(edits: currentEdits, asShotTemp: asShotTemperature, asShotTint: asShotTint)
    }
    
    // MARK: - Crop & Composition Controls
    public func toggleCropTool() {
        isCropToolActive.toggle()
    }
    
    public func setCropAspectRatioPreset(_ preset: CropAspectRatioPreset) {
        self.cropAspectRatioPreset = preset
        guard preset != .freeform else { return }
        
        let isOrthogonal = (rotationDegrees == 90 || rotationDegrees == 270)
        let baseW = previewImage?.size.width ?? 4000
        let baseH = previewImage?.size.height ?? 3000
        let origW: CGFloat = CGFloat(isOrthogonal ? baseH : baseW)
        let origH: CGFloat = CGFloat(isOrthogonal ? baseW : baseH)
        let originalAspect: CGFloat = origW / max(origH, 1.0)
        
        let isPortrait = (origW < origH)
        let targetAspect: CGFloat
        switch preset {
        case .original:
            targetAspect = originalAspect
        case .square:
            targetAspect = 1.0
        case .fourThree:
            targetAspect = isPortrait ? (3.0 / 4.0) : (4.0 / 3.0)
        case .threeTwo:
            targetAspect = isPortrait ? (2.0 / 3.0) : (3.0 / 2.0)
        case .sixteenNine:
            targetAspect = isPortrait ? (9.0 / 16.0) : (16.0 / 9.0)
        case .freeform:
            return
        }
        
        let normTargetAspect: CGFloat = targetAspect / originalAspect
        let currentW: CGFloat = CGFloat(cropRight - cropLeft)
        let currentH: CGFloat = CGFloat(cropBottom - cropTop)
        let center = CGPoint(x: CGFloat(cropLeft + cropRight) / 2.0, y: CGFloat(cropTop + cropBottom) / 2.0)
        
        var newW: CGFloat = currentW
        var newH: CGFloat = currentW / max(normTargetAspect, 0.001)
        if newH > 1.0 || newH > currentH {
            newH = min(currentH, 1.0)
            newW = newH * normTargetAspect
        }
        if newW > 1.0 {
            newW = 1.0
            newH = newW / max(normTargetAspect, 0.001)
        }
        
        let newLeft = max(center.x - newW / 2.0, 0.0)
        let newRight = min(newLeft + newW, 1.0)
        let finalLeft = max(newRight - newW, 0.0)
        
        let newTop = max(center.y - newH / 2.0, 0.0)
        let newBottom = min(newTop + newH, 1.0)
        let finalTop = max(newBottom - newH, 0.0)
        
        self.cropLeft = Float(finalLeft)
        self.cropTop = Float(finalTop)
        self.cropRight = Float(newRight)
        self.cropBottom = Float(newBottom)
        saveCurrentEdits()
        notifyThumbnailChanged()
    }
    
    public func flipCropOrientation() {
        let currentW: CGFloat = CGFloat(cropRight - cropLeft)
        let currentH: CGFloat = CGFloat(cropBottom - cropTop)
        let center = CGPoint(x: CGFloat(cropLeft + cropRight) / 2.0, y: CGFloat(cropTop + cropBottom) / 2.0)
        
        let isOrthogonal = (rotationDegrees == 90 || rotationDegrees == 270)
        let baseW = previewImage?.size.width ?? 4000
        let baseH = previewImage?.size.height ?? 3000
        let origW: CGFloat = CGFloat(isOrthogonal ? baseH : baseW)
        let origH: CGFloat = CGFloat(isOrthogonal ? baseW : baseH)
        let originalAspect: CGFloat = origW / max(origH, 1.0)
        
        let currentPhotoAspect: CGFloat = (currentW / max(currentH, 0.001)) * originalAspect
        let newPhotoAspect: CGFloat = 1.0 / max(currentPhotoAspect, 0.01)
        let newNormAspect: CGFloat = newPhotoAspect / originalAspect
        
        var newW: CGFloat = currentH * newNormAspect
        var newH: CGFloat = currentH
        if newW > 1.0 {
            newW = 1.0
            newH = newW / max(newNormAspect, 0.001)
        }
        if newH > 1.0 {
            newH = 1.0
            newW = newH * newNormAspect
        }
        
        let newLeft = max(center.x - newW / 2.0, 0.0)
        let newRight = min(newLeft + newW, 1.0)
        let finalLeft = max(newRight - newW, 0.0)
        
        let newTop = max(center.y - newH / 2.0, 0.0)
        let newBottom = min(newTop + newH, 1.0)
        let finalTop = max(newBottom - newH, 0.0)
        
        self.cropLeft = Float(finalLeft)
        self.cropTop = Float(finalTop)
        self.cropRight = Float(newRight)
        self.cropBottom = Float(newBottom)
        saveCurrentEdits()
        notifyThumbnailChanged()
    }
    
    public func resetCrop() {
        cropLeft = 0.0
        cropTop = 0.0
        cropRight = 1.0
        cropBottom = 1.0
        cropAspectRatioPreset = .original
        saveCurrentEdits()
        notifyThumbnailChanged()
    }
    
    public func resetGeometry() {
        cropLeft = 0.0
        cropTop = 0.0
        cropRight = 1.0
        cropBottom = 1.0
        straightenAngle = 0.0
        perspectiveVertical = 0.0
        perspectiveHorizontal = 0.0
        cropConstrain = true
        cropAspectRatioPreset = .original
        saveCurrentEdits()
        notifyThumbnailChanged()
    }
    
    public func rotateClockwise() {
        rotationDegrees = (rotationDegrees + 90) % 360
        saveCurrentEdits()
        notifyThumbnailChanged()
    }
    
    public func rotateCounterClockwise() {
        rotationDegrees = (rotationDegrees + 270) % 360
        saveCurrentEdits()
        notifyThumbnailChanged()
    }
    
    public func toggleFlipHorizontal() {
        flipHorizontal.toggle()
        saveCurrentEdits()
        notifyThumbnailChanged()
    }
    
    public func toggleFlipVertical() {
        flipVertical.toggle()
        saveCurrentEdits()
        notifyThumbnailChanged()
    }
    
    public func notifyThumbnailChanged() {
        guard let path = currentlyLoadedPhotoPath ?? selectedPhoto?.filePath else { return }
        NotificationCenter.default.post(name: .thumbnailDidChange, object: path)
    }
    
    public var allLutTitles: [String] {
        availableLutTitles + customLuts.map(\.title)
    }
    
    public func resolveLutRecord(for title: String?) -> CubeLutRecord? {
        guard let title = title, !title.isEmpty else { return nil }
        if let custom = customLuts.first(where: { $0.title == title }) {
            return custom
        }
        return getBuiltinCubeLut(title: title, size: 33)
    }
    
    // MARK: - Auto Development Adjustments
    public func applyAutoAdjustments() {
        guard let photo = selectedPhoto else { return }
        
        // 1. Fast-path: Calculate from already decoded half-float preview buffer
        if let rawRecord = rawPreviewRecord {
            if let adj = computeAutoAdjustmentsFromBuffer(width: rawRecord.width, height: rawRecord.height, data: rawRecord.data) {
                withAnimation(.easeInOut(duration: 0.15)) {
                    self.exposure = adj.exposure
                    self.contrast = adj.contrast
                }
                saveCurrentEdits()
                return
            }
        }
        
        // 2. Background path: Decode from file
        let path = photo.filePath
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            guard let adj = computeAutoAdjustments(path: path) else { return }
            DispatchQueue.main.async {
                guard let self = self, self.selectedPhoto?.filePath == path else { return }
                withAnimation(.easeInOut(duration: 0.15)) {
                    self.exposure = adj.exposure
                    self.contrast = adj.contrast
                }
                self.saveCurrentEdits()
            }
        }
    }
    
    // MARK: - Slider Reset Helpers
    public func resetSlider(_ type: EditSliderType) {
        switch type {
        case .exposure:
            exposure = 0.0
        case .contrast:
            contrast = 0.0
        case .highlights:
            highlights = 0.0
        case .shadows:
            shadows = 0.0
        case .whites:
            whites = 0.0
        case .blacks:
            blacks = 0.0
        case .vibrance:
            vibrance = 0.0
        case .saturation:
            saturation = 0.0
        case .deepBlue:
            deepBlue = 0.0
        case .foliage:
            foliage = 0.0
        case .sharpeningAmount:
            sharpeningAmount = defaultSharpeningAmount
        case .sharpeningRadius:
            sharpeningRadius = defaultSharpeningRadius
        case .sharpeningMasking:
            sharpeningMasking = defaultSharpeningMasking
        case .noiseReduction:
            noiseReduction = defaultNoiseReduction
        case .grainAmount:
            grainAmount = defaultGrainAmount
            selectedGrainPresetId = nil
        case .grainSize:
            grainSize = defaultGrainSize
            selectedGrainPresetId = nil
        case .grainRoughness:
            grainRoughness = defaultGrainRoughness
            selectedGrainPresetId = nil
        case .grainChroma:
            grainChroma = defaultGrainChroma
            selectedGrainPresetId = nil
        case .temperature:
            temperature = asShotTemperature
        case .tint:
            tint = asShotTint
        case .straighten:
            straightenAngle = 0.0
        case .perspectiveVertical:
            perspectiveVertical = 0.0
        case .perspectiveHorizontal:
            perspectiveHorizontal = 0.0
        case .lutIntensity:
            lutIntensity = 1.0
        }
        saveCurrentEdits()
    }
    
    public func applyGrainPreset(_ preset: FilmGrainPreset?) {
        guard let p = preset else {
            selectedGrainPresetId = nil
            grainAmount = defaultGrainAmount
            grainSize = defaultGrainSize
            grainRoughness = defaultGrainRoughness
            grainChroma = defaultGrainChroma
            saveCurrentEdits()
            return
        }
        selectedGrainPresetId = p.id
        grainAmount = p.amount
        grainSize = p.size
        grainRoughness = p.roughness
        grainChroma = p.chroma
        saveCurrentEdits()
    }
    
    public func resetAllSliders() {
        exposure = 0.0
        contrast = 0.0
        highlights = 0.0
        shadows = 0.0
        whites = 0.0
        blacks = 0.0
        vibrance = 0.0
        saturation = 0.0
        deepBlue = 0.0
        foliage = 0.0
        sharpeningAmount = defaultSharpeningAmount
        sharpeningRadius = defaultSharpeningRadius
        sharpeningMasking = defaultSharpeningMasking
        noiseReduction = defaultNoiseReduction
        grainAmount = defaultGrainAmount
        grainSize = defaultGrainSize
        grainRoughness = defaultGrainRoughness
        grainChroma = defaultGrainChroma
        selectedGrainPresetId = nil
        temperature = asShotTemperature
        tint = asShotTint
        straightenAngle = 0.0
        cropLeft = 0.0
        cropTop = 0.0
        cropRight = 1.0
        cropBottom = 1.0
        perspectiveVertical = 0.0
        perspectiveHorizontal = 0.0
        cropConstrain = true
        isCropToolActive = false
        cropAspectRatioPreset = .original
        rotationDegrees = defaultExifRotation
        flipHorizontal = defaultExifFlipH
        flipVertical = defaultExifFlipV
        toneProfile = .agxVivid
        selectedLutName = nil
        lutIntensity = 1.0
        curves = .defaultIdentity
        saveCurrentEdits()
        notifyThumbnailChanged()
    }
    
    // MARK: - White Balance Eyedropper Sampling
    public func applyEyedropperSample(atNormalizedUV uv: CGPoint) {
        if let rawRecord = rawPreviewRecord, rawRecord.width > 0, rawRecord.height > 0 {
            let width = Int(rawRecord.width)
            let height = Int(rawRecord.height)
            let cx = min(max(Int(uv.x * CGFloat(width)), 0), width - 1)
            let cy = min(max(Int(uv.y * CGFloat(height)), 0), height - 1)
            
            var sumR: Float = 0.0
            var sumG: Float = 0.0
            var sumB: Float = 0.0
            var count: Float = 0.0
            
            rawRecord.data.withUnsafeBytes { rawBuffer in
                guard let basePtr = rawBuffer.baseAddress else { return }
                for dy in -1...1 {
                    let y = min(max(cy + dy, 0), height - 1)
                    for dx in -1...1 {
                        let x = min(max(cx + dx, 0), width - 1)
                        let offset = (y * width + x) * 8
                        let r16 = basePtr.load(fromByteOffset: offset, as: UInt16.self)
                        let g16 = basePtr.load(fromByteOffset: offset + 2, as: UInt16.self)
                        let b16 = basePtr.load(fromByteOffset: offset + 4, as: UInt16.self)
                        sumR += Float(Float16(bitPattern: r16))
                        sumG += Float(Float16(bitPattern: g16))
                        sumB += Float(Float16(bitPattern: b16))
                        count += 1.0
                    }
                }
            }
            
            if count > 0 {
                let wb = calculateWhiteBalanceFromSample(r: sumR / count, g: sumG / count, b: sumB / count)
                withAnimation(.easeInOut(duration: 0.15)) {
                    self.temperature = wb.temperature
                    self.tint = wb.tint
                }
                saveCurrentEdits()
            }
        } else if let preview = previewImage, let cgImage = preview.cgImage(forProposedRect: nil, context: nil, hints: nil) {
            let width = cgImage.width
            let height = cgImage.height
            guard width > 0, height > 0 else { return }
            let cx = min(max(Int(uv.x * CGFloat(width)), 0), width - 1)
            let cy = min(max(Int(uv.y * CGFloat(height)), 0), height - 1)
            
            guard let colorSpace = CGColorSpace(name: CGColorSpace.sRGB) else { return }
            var pixel = [UInt8](repeating: 0, count: 4)
            guard let context = CGContext(
                data: &pixel,
                width: 1,
                height: 1,
                bitsPerComponent: 8,
                bytesPerRow: 4,
                space: colorSpace,
                bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue
            ) else { return }
            
            context.draw(cgImage, in: CGRect(x: -cx, y: -(height - 1 - cy), width: width, height: height))
            
            let srgbToLinear: (Float) -> Float = { c in
                let cf = min(max(c, 0.0), 1.0)
                return cf <= 0.04045 ? (cf / 12.92) : pow((cf + 0.055) / 1.055, 2.4)
            }
            
            let linR = srgbToLinear(Float(pixel[0]) / 255.0)
            let linG = srgbToLinear(Float(pixel[1]) / 255.0)
            let linB = srgbToLinear(Float(pixel[2]) / 255.0)
            
            let wb = calculateWhiteBalanceFromSample(r: linR, g: linG, b: linB)
            withAnimation(.easeInOut(duration: 0.15)) {
                self.temperature = wb.temperature
                self.tint = wb.tint
            }
            saveCurrentEdits()
        }
    }
    
    // MARK: - Custom 3D LUT Import
    public func importCustomLut() {
        let panel = NSOpenPanel()
        panel.allowedContentTypes = [.init(filenameExtension: "cube")].compactMap { $0 }
        panel.allowsMultipleSelection = false
        panel.canChooseDirectories = false
        panel.canChooseFiles = true
        panel.title = "Import 3D LUT (.cube)"
        
        if panel.runModal() == .OK, let url = panel.url {
            if let customLut = loadCubeLut(path: url.path) {
                if !self.customLuts.contains(where: { $0.title == customLut.title }) {
                    self.customLuts.append(customLut)
                }
                self.selectedLutName = customLut.title
                self.lutIntensity = 1.0
                saveCurrentEdits()
            }
        }
    }
}
