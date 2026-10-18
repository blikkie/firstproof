import Foundation
import SwiftUI
import AppKit
import CoreGraphics
import FirstProofBridge

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
            emulsionGrainMode: self.emulsionGrainMode,
            grainScale: self.grainScale,
            grainSize: self.grainSize,
            grainAmount: self.grainAmount,
            emulsionArchetype: self.emulsionArchetype,
            emulsionFormat: self.emulsionFormat,
            emulsionExposure: self.emulsionExposure,
            remjetBacking: self.remjetBacking,
            curves: self.curves,
            splitToningShadowHue: self.splitToningShadowHue,
            splitToningShadowSat: self.splitToningShadowSat,
            splitToningHighlightHue: self.splitToningHighlightHue,
            splitToningHighlightSat: self.splitToningHighlightSat,
            splitToningBalance: self.splitToningBalance,
            printMedia: self.printMedia,
            enlargerFilter: self.enlargerFilter,
            printPaperGrade: self.printPaperGrade,
            printBaseWarmth: self.printBaseWarmth,
            printProcessDepth: self.printProcessDepth
        )
    }
}

extension EditSettings {
    /// Gracefully migrates legacy/retired closed-domain camera profiles to First Proof's
    /// open-domain AgX Vivid filmic tone profile.
    public var migratingLegacyProfiles: EditSettings {
        var copy = self
        if copy.toneProfile == .cameraStandard ||
           copy.toneProfile == .cameraPortrait ||
           copy.toneProfile == .cameraLandscape ||
           copy.toneProfile == .cameraVivid {
            copy.toneProfile = .agxVivid
        }
        return copy
    }

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
            temperature: 5500.0,
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
            emulsionGrainMode: .off,
            grainScale: 1.0,
            grainSize: 1.0,
            grainAmount: 1.0,
            emulsionArchetype: .none,
            emulsionFormat: .format35mm,
            emulsionExposure: 0.0,
            remjetBacking: .intact,
            curves: ToneCurvesRecord.defaultIdentity,
            splitToningShadowHue: 215.0,
            splitToningShadowSat: 0.0,
            splitToningHighlightHue: 45.0,
            splitToningHighlightSat: 0.0,
            splitToningBalance: 0.0,
            printMedia: .none,
            enlargerFilter: .neutralPanchromatic,
            printPaperGrade: 0.0,
            printBaseWarmth: 0.0,
            printProcessDepth: 100.0
        )
    }
}

extension AppViewModel {
    public func applyEdits(_ edits: EditSettings) {
        let edits = edits.migratingLegacyProfiles
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
        self.emulsionGrainMode = edits.emulsionGrainMode
        self.grainScale = edits.grainScale
        self.grainSize = edits.grainSize
        self.grainAmount = edits.grainAmount
        self.emulsionArchetype = edits.emulsionArchetype
        if edits.emulsionArchetype == .cambridgeDiffusion100 && (edits.emulsionFormat == .super8 || edits.emulsionFormat == .format16mm) {
            self.emulsionFormat = .format35mm
        } else {
            self.emulsionFormat = edits.emulsionFormat
        }
        self.emulsionExposure = edits.emulsionExposure
        self.remjetBacking = edits.remjetBacking
        self.curves = edits.curves
        self.splitToningShadowHue = edits.splitToningShadowHue
        self.splitToningShadowSat = edits.splitToningShadowSat
        self.splitToningHighlightHue = edits.splitToningHighlightHue
        self.splitToningHighlightSat = edits.splitToningHighlightSat
        self.splitToningBalance = edits.splitToningBalance
        self.printMedia = edits.printMedia
        self.enlargerFilter = edits.enlargerFilter
        self.printPaperGrade = edits.printPaperGrade
        self.printBaseWarmth = edits.printBaseWarmth
        self.printProcessDepth = edits.printProcessDepth
    }
    
    public func saveCurrentEdits() {
        guard let path = currentlyLoadedPhotoPath ?? selectedPhoto?.filePath else { return }
        let editsSnapshot = currentEdits
        
        // Real-time sub-millisecond GPU thumbnail & intermediate capture for active viewport
        if let snapshot = captureThumbnailSnapshot?() {
            ThumbnailCache.shared.saveEditedThumbnail(snapshot, for: path, edits: editsSnapshot)
        }
        if let intermediate = captureIntermediateSnapshot?() {
            ThumbnailCache.shared.saveIntermediate(intermediate, for: path, edits: editsSnapshot)
        }
        
        syncXmp(for: path, rating: nil, status: nil, edits: editsSnapshot)
    }
    
    public var hasCustomEdits: Bool {
        editsHasModifications(edits: currentEdits, asShotTemp: asShotTemperature, asShotTint: asShotTint)
    }
    
    // MARK: - Undo/Redo & Coalesced Scrubbing
    public var hasActiveSliderScrub: Bool {
        undoHistory.scrubStartEdits != nil
    }
    
    public func beginSliderScrub() {
        guard let path = currentlyLoadedPhotoPath ?? selectedPhoto?.filePath else { return }
        undoHistory.beginScrub(photoPath: path, currentEdits: currentEdits)
    }
    
    public func cancelSliderScrub() {
        guard let path = currentlyLoadedPhotoPath ?? selectedPhoto?.filePath,
              let startEdits = undoHistory.scrubStartEdits else {
            undoHistory.cancelScrub()
            return
        }
        undoHistory.cancelScrub()
        applyEdits(startEdits)
        if let intermediate = captureIntermediateSnapshot?() {
            ThumbnailCache.shared.saveIntermediate(intermediate, for: path, edits: currentEdits)
        }
    }
    
    public func endSliderScrub(actionName: String) {
        guard let path = currentlyLoadedPhotoPath ?? selectedPhoto?.filePath else { return }
        undoHistory.endScrub(photoPath: path, currentEdits: currentEdits, title: actionName)
        saveCurrentEdits()
    }
    
    public func handleSliderEditing(_ isEditing: Bool, slider: EditSliderType) {
        if isEditing {
            beginSliderScrub()
        } else {
            let title: String
            switch slider {
            case .exposure:
                title = "Exposure (\(String(format: "%.2f EV", exposure)))"
            case .contrast:
                title = "Contrast (\(contrast >= 0 ? "+" : "")\(Int(contrast)))"
            case .highlights:
                title = "Highlights (\(highlights >= 0 ? "+" : "")\(Int(highlights)))"
            case .shadows:
                title = "Shadows (\(shadows >= 0 ? "+" : "")\(Int(shadows)))"
            case .whites:
                title = "Whites (\(whites >= 0 ? "+" : "")\(Int(whites)))"
            case .blacks:
                title = "Blacks (\(blacks >= 0 ? "+" : "")\(Int(blacks)))"
            case .vibrance:
                title = "Vibrance (\(vibrance >= 0 ? "+" : "")\(Int(vibrance)))"
            case .saturation:
                title = "Saturation (\(saturation >= 0 ? "+" : "")\(Int(saturation)))"
            case .deepBlue:
                title = "Deep Blue (\(deepBlue >= 0 ? "+" : "")\(Int(deepBlue)))"
            case .foliage:
                title = "Foliage (\(foliage >= 0 ? "+" : "")\(Int(foliage)))"
            case .temperature:
                title = "Temperature (\(Int(temperature)) K)"
            case .tint:
                title = "Tint (\(tint >= 0 ? "+" : "")\(Int(tint)))"
            case .straighten:
                title = "Straighten (\(String(format: "%.1f°", straightenAngle)))"
            case .perspectiveVertical:
                title = "Perspective V (\(Int(perspectiveVertical)))"
            case .perspectiveHorizontal:
                title = "Perspective H (\(Int(perspectiveHorizontal)))"
            case .sharpeningAmount:
                title = "Sharpening (\(Int(sharpeningAmount)))"
            case .sharpeningRadius:
                title = "Sharpening Radius (\(String(format: "%.1f px", sharpeningRadius)))"
            case .sharpeningMasking:
                title = "Sharpening Masking (\(Int(sharpeningMasking)))"
            case .noiseReduction:
                title = "Noise Reduction (\(Int(noiseReduction)))"
            case .grainScale:
                title = "Grain Scale (\(String(format: "%.1fx", grainScale)))"
            case .grainSize:
                title = "Grain Size (\(String(format: "%.2fx", grainSize)))"
            case .grainAmount:
                title = "Grain Amount (\(String(format: "%.2fx", grainAmount)))"
            case .lutIntensity:
                title = "LUT Intensity (\(Int(lutIntensity * 100))%)"
            case .splitToningShadowHue:
                title = "Shadow Hue (\(Int(splitToningShadowHue))°)"
            case .splitToningShadowSat:
                title = "Shadow Saturation (\(Int(splitToningShadowSat)))"
            case .splitToningHighlightHue:
                title = "Highlight Hue (\(Int(splitToningHighlightHue))°)"
            case .splitToningHighlightSat:
                title = "Highlight Saturation (\(Int(splitToningHighlightSat)))"
            case .splitToningBalance:
                title = "Split Balance (\(splitToningBalance >= 0 ? "+" : "")\(Int(splitToningBalance)))"
            case .printPaperGrade:
                title = "Paper Grade (\(String(format: "%+.1f", printPaperGrade)))"
            case .printBaseWarmth:
                title = "Base Warmth (\(printBaseWarmth >= 0 ? "+" : "")\(Int(printBaseWarmth)))"
            case .printProcessDepth:
                title = "Process Depth (\(Int(printProcessDepth))%)"
            }
            endSliderScrub(actionName: title)
        }
    }
    
    public func registerDiscreteEdit(title: String, before: EditSettings) {
        guard let path = currentlyLoadedPhotoPath ?? selectedPhoto?.filePath else { return }
        let after = currentEdits
        if before != after {
            undoHistory.registerEdit(title: title, photoPath: path, before: before, after: after)
        }
        saveCurrentEdits()
    }
    
    public func undo() {
        let activePath = currentlyLoadedPhotoPath ?? selectedPhoto?.filePath
        guard let entry = undoHistory.popUndo(isDetailMode: canvasMode == .detail, activePhotoPath: activePath) else { return }
        executeUndoPayload(entry.payload, isUndo: true, primaryPath: entry.primaryPhotoPath)
    }
    
    public func redo() {
        let activePath = currentlyLoadedPhotoPath ?? selectedPhoto?.filePath
        guard let entry = undoHistory.popRedo(isDetailMode: canvasMode == .detail, activePhotoPath: activePath) else { return }
        executeUndoPayload(entry.payload, isUndo: false, primaryPath: entry.primaryPhotoPath)
    }
    
    public func jumpToHistoryStep(_ targetEntry: UndoEntry) {
        let isDetail = (canvasMode == .detail)
        let activePath = currentlyLoadedPhotoPath ?? selectedPhoto?.filePath
        let uStack = undoHistory.undoStack(for: isDetail ? activePath : nil)
        let rStack = undoHistory.redoStack(for: isDetail ? activePath : nil)
        
        if let targetIdx = uStack.firstIndex(where: { $0.id == targetEntry.id }) {
            let stepsToUndo = uStack.count - 1 - targetIdx
            for _ in 0..<stepsToUndo {
                undo()
            }
        } else if let targetIdx = rStack.firstIndex(where: { $0.id == targetEntry.id }) {
            let stepsToRedo = rStack.count - targetIdx
            for _ in 0..<stepsToRedo {
                redo()
            }
        }
    }
    
    private func executeUndoPayload(_ payload: UndoActionPayload, isUndo: Bool, primaryPath: String) {
        if canvasMode == .grid {
            if let currentPath = currentlyLoadedPhotoPath ?? selectedPhoto?.filePath, currentPath != primaryPath {
                if let matching = photos.first(where: { $0.filePath == primaryPath }) {
                    selectPhoto(matching)
                }
            }
        }
        
        switch payload {
        case .edits(let before, let after):
            let targetEdits = isUndo ? before : after
            applyEdits(targetEdits)
            saveCurrentEdits()
            
        case .rating(let targets):
            guard let db = db else { return }
            for target in targets {
                let targetRating = isUndo ? target.oldRating : target.newRating
                _ = db.updateRatingStatusBatch(filePaths: [target.path], rating: targetRating, updateRating: true, status: nil)
                if let idx = photos.firstIndex(where: { $0.filePath == target.path }) {
                    photos[idx].rating = targetRating
                }
                if selectedPhoto?.filePath == target.path {
                    selectedPhoto?.rating = targetRating
                }
                syncXmp(for: target.path, rating: targetRating, status: nil, edits: nil)
            }
            
        case .status(let targets):
            guard let db = db else { return }
            for target in targets {
                let targetStatus = isUndo ? target.oldStatus : target.newStatus
                _ = db.updateRatingStatusBatch(filePaths: [target.path], rating: nil, updateRating: false, status: targetStatus)
                if let idx = photos.firstIndex(where: { $0.filePath == target.path }) {
                    photos[idx].status = targetStatus
                }
                if selectedPhoto?.filePath == target.path {
                    selectedPhoto?.status = targetStatus
                }
                syncXmp(for: target.path, rating: nil, status: targetStatus, edits: nil)
            }
        }
    }
    
    // MARK: - Crop & Composition Controls
    
    /// Transient snapshot capturing pre-crop darkroom state so users can cancel via Escape.
    public struct CropSessionSnapshot {
        public let edits: EditSettings
        public let preset: CropAspectRatioPreset
        public let photoPath: String?
        public let undoStack: [UndoEntry]
        public let redoStack: [UndoEntry]
        
        public init(
            edits: EditSettings,
            preset: CropAspectRatioPreset,
            photoPath: String?,
            undoStack: [UndoEntry],
            redoStack: [UndoEntry]
        ) {
            self.edits = edits
            self.preset = preset
            self.photoPath = photoPath
            self.undoStack = undoStack
            self.redoStack = redoStack
        }
    }
    public func enterCropMode() {
        guard !isCropToolActive else { return }
        let currentPhotoPath = selectedPhoto?.filePath
        let currentUndoStack = undoHistory.undoStack(for: currentPhotoPath)
        let currentRedoStack = undoHistory.redoStack(for: currentPhotoPath)
        
        cropSessionSnapshot = CropSessionSnapshot(
            edits: currentEdits,
            preset: cropAspectRatioPreset,
            photoPath: currentPhotoPath,
            undoStack: currentUndoStack,
            redoStack: currentRedoStack
        )
        isCropToolActive = true
    }
    
    public func commitCrop() {
        guard isCropToolActive else { return }
        if undoHistory.scrubStartEdits != nil {
            endSliderScrub(actionName: "Crop Box")
        }
        isCropToolActive = false
        cropSessionSnapshot = nil
        saveCurrentEdits()
    }
    
    public func cancelCrop() {
        guard isCropToolActive else { return }
        
        // Discard any active gesture scrub without recording undo
        undoHistory.scrubStartEdits = nil
        undoHistory.scrubPhotoPath = nil
        
        if let snapshot = cropSessionSnapshot {
            applyEdits(snapshot.edits)
            cropAspectRatioPreset = snapshot.preset
            if let path = snapshot.photoPath {
                undoHistory.restorePhotoHistory(
                    for: path,
                    undoStack: snapshot.undoStack,
                    redoStack: snapshot.redoStack
                )
            }
            cropSessionSnapshot = nil
        }
        
        isCropToolActive = false
        saveCurrentEdits()
    }
    
    public func toggleCropTool() {
        if isCropToolActive {
            commitCrop()
        } else {
            enterCropMode()
        }
    }
    
    public func setCropAspectRatioPreset(_ preset: CropAspectRatioPreset) {
        let before = currentEdits
        self.cropAspectRatioPreset = preset
        guard preset != .freeform else { return }
        
        let isOrthogonal = (rotationDegrees == 90 || rotationDegrees == 270)
        let dims = currentImageDimensions
        let origW: CGFloat = isOrthogonal ? dims.height : dims.width
        let origH: CGFloat = isOrthogonal ? dims.width : dims.height
        let originalAspect: CGFloat = origW / max(origH, 1.0)
        
        let currentW: CGFloat = CGFloat(cropRight - cropLeft)
        let currentH: CGFloat = CGFloat(cropBottom - cropTop)
        let center = CGPoint(x: CGFloat(cropLeft + cropRight) / 2.0, y: CGFloat(cropTop + cropBottom) / 2.0)
        
        let currentPhotoAspect: CGFloat = (currentW / max(currentH, 0.001)) * originalAspect
        let isPortrait = (currentW >= 0.99 && currentH >= 0.99) ? (origW < origH) : (currentPhotoAspect < 1.0)
        
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
        
        let rCurr: CGFloat = currentPhotoAspect / originalAspect
        let maxWCurr: CGFloat = (rCurr >= 1.0) ? 1.0 : rCurr
        let maxHCurr: CGFloat = (rCurr >= 1.0) ? (1.0 / max(rCurr, 0.001)) : 1.0
        let rawScale: CGFloat = min(currentW / max(maxWCurr, 0.001), currentH / max(maxHCurr, 0.001), 1.0)
        let scale: CGFloat = (rawScale > 0.995) ? 1.0 : rawScale
        
        let rTarget: CGFloat = targetAspect / originalAspect
        let maxWTarget: CGFloat = (rTarget >= 1.0) ? 1.0 : rTarget
        let maxHTarget: CGFloat = (rTarget >= 1.0) ? (1.0 / max(rTarget, 0.001)) : 1.0
        
        let newW: CGFloat = min(scale * maxWTarget, 1.0)
        let newH: CGFloat = min(scale * maxHTarget, 1.0)
        
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
        registerDiscreteEdit(title: "Aspect Ratio (\(preset.rawValue))", before: before)
        notifyThumbnailChanged()
    }
    
    public func flipCropOrientation() {
        let before = currentEdits
        let currentW: CGFloat = CGFloat(cropRight - cropLeft)
        let currentH: CGFloat = CGFloat(cropBottom - cropTop)
        let center = CGPoint(x: CGFloat(cropLeft + cropRight) / 2.0, y: CGFloat(cropTop + cropBottom) / 2.0)
        
        let isOrthogonal = (rotationDegrees == 90 || rotationDegrees == 270)
        let dims = currentImageDimensions
        let origW: CGFloat = isOrthogonal ? dims.height : dims.width
        let origH: CGFloat = isOrthogonal ? dims.width : dims.height
        let originalAspect: CGFloat = origW / max(origH, 1.0)
        
        // 1. Current physical aspect ratio and normalized ratio R_curr
        let currentPhotoAspect: CGFloat = (currentW / max(currentH, 0.001)) * originalAspect
        let rCurr: CGFloat = currentPhotoAspect / originalAspect // == currentW / currentH
        let maxWCurr: CGFloat = (rCurr >= 1.0) ? 1.0 : rCurr
        let maxHCurr: CGFloat = (rCurr >= 1.0) ? (1.0 / max(rCurr, 0.001)) : 1.0
        let rawScale: CGFloat = min(currentW / max(maxWCurr, 0.001), currentH / max(maxHCurr, 0.001), 1.0)
        let scale: CGFloat = (rawScale > 0.995) ? 1.0 : rawScale
        
        // 2. Inverted target physical aspect ratio and normalized ratio R_target
        let targetPhotoAspect: CGFloat = 1.0 / max(currentPhotoAspect, 0.001)
        let rTarget: CGFloat = targetPhotoAspect / originalAspect
        let maxWTarget: CGFloat = (rTarget >= 1.0) ? 1.0 : rTarget
        let maxHTarget: CGFloat = (rTarget >= 1.0) ? (1.0 / max(rTarget, 0.001)) : 1.0
        
        let newW: CGFloat = min(scale * maxWTarget, 1.0)
        let newH: CGFloat = min(scale * maxHTarget, 1.0)
        
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
        registerDiscreteEdit(title: "Flip Aspect Ratio", before: before)
        notifyThumbnailChanged()
    }
    
    public func resetCrop() {
        let before = currentEdits
        cropLeft = 0.0
        cropTop = 0.0
        cropRight = 1.0
        cropBottom = 1.0
        cropAspectRatioPreset = .original
        registerDiscreteEdit(title: "Reset Crop", before: before)
        notifyThumbnailChanged()
    }
    
    public func resetGeometry() {
        let before = currentEdits
        cropLeft = 0.0
        cropTop = 0.0
        cropRight = 1.0
        cropBottom = 1.0
        straightenAngle = 0.0
        perspectiveVertical = 0.0
        perspectiveHorizontal = 0.0
        cropConstrain = true
        cropAspectRatioPreset = .original
        registerDiscreteEdit(title: "Reset Geometry", before: before)
        notifyThumbnailChanged()
    }
    
    public func rotateClockwise() {
        let before = currentEdits
        rotationDegrees = (rotationDegrees + 90) % 360
        registerDiscreteEdit(title: "Rotate 90° Clockwise", before: before)
        notifyThumbnailChanged()
    }
    
    public func rotateCounterClockwise() {
        let before = currentEdits
        rotationDegrees = (rotationDegrees + 270) % 360
        registerDiscreteEdit(title: "Rotate 90° Counter-Clockwise", before: before)
        notifyThumbnailChanged()
    }
    
    public func toggleFlipHorizontal() {
        let before = currentEdits
        flipHorizontal.toggle()
        registerDiscreteEdit(title: "Flip Horizontal", before: before)
        notifyThumbnailChanged()
    }
    
    public func toggleFlipVertical() {
        let before = currentEdits
        flipVertical.toggle()
        registerDiscreteEdit(title: "Flip Vertical", before: before)
        notifyThumbnailChanged()
    }
    
    public func notifyThumbnailChanged() {
        saveCurrentEdits()
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
        let before = currentEdits
        
        // 1. Fast-path: Calculate from already decoded half-float preview buffer
        if let rawRecord = rawPreviewRecord {
            if let adj = computeAutoAdjustmentsFromBuffer(width: rawRecord.width, height: rawRecord.height, data: rawRecord.data) {
                withAnimation(.easeInOut(duration: 0.15)) {
                    self.exposure = adj.exposure
                    self.contrast = adj.contrast
                }
                registerDiscreteEdit(title: "Auto Tone", before: before)
                return
            }
        }
        
        // 2. Background path: Decode from file
        let path = photo.filePath
        Task { [weak self] in
            let adj = await Task.detached(priority: .userInitiated) {
                computeAutoAdjustments(path: path)
            }.value
            guard let self = self, self.selectedPhoto?.filePath == path, let adj = adj else { return }
            withAnimation(.easeInOut(duration: 0.15)) {
                self.exposure = adj.exposure
                self.contrast = adj.contrast
            }
            self.registerDiscreteEdit(title: "Auto Tone", before: before)
        }
    }
    
    // MARK: - Auto White Balance & Presets
    public func applyAutoWhiteBalance() {
        guard let photo = selectedPhoto else { return }
        let before = currentEdits
        
        // 1. Fast-path: Calculate from already decoded half-float preview buffer
        if let rawRecord = rawPreviewRecord {
            if let wb = computeAutoWhiteBalanceFromBuffer(width: rawRecord.width, height: rawRecord.height, data: rawRecord.data) {
                withAnimation(.easeInOut(duration: 0.15)) {
                    self.temperature = wb.temperature
                    self.tint = wb.tint
                }
                registerDiscreteEdit(title: "Auto White Balance", before: before)
                return
            }
        }
        
        // 2. Background path: Decode from file
        let path = photo.filePath
        Task { [weak self] in
            let wb = await Task.detached(priority: .userInitiated) {
                computeAutoWhiteBalance(path: path)
            }.value
            guard let self = self, self.selectedPhoto?.filePath == path, let wb = wb else { return }
            withAnimation(.easeInOut(duration: 0.15)) {
                self.temperature = wb.temperature
                self.tint = wb.tint
            }
            self.registerDiscreteEdit(title: "Auto White Balance", before: before)
        }
    }
    
    public func applyWhiteBalancePreset(_ preset: WhiteBalancePreset) {
        let before = currentEdits
        if preset == .asShot {
            withAnimation(.easeInOut(duration: 0.15)) {
                self.temperature = self.asShotTemperature
                self.tint = self.asShotTint
            }
            registerDiscreteEdit(title: "White Balance: As Shot", before: before)
        } else if preset == .auto {
            applyAutoWhiteBalance()
        } else if let values = getWhiteBalancePresetValues(preset: preset) {
            withAnimation(.easeInOut(duration: 0.15)) {
                self.temperature = Float(values.temperature)
                self.tint = Float(values.tint)
            }
            registerDiscreteEdit(title: "White Balance: \(preset.displayName)", before: before)
        }
    }
    
    // MARK: - Slider Reset Helpers
    public func resetSlider(_ type: EditSliderType) {
        let before = currentEdits
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
        case .grainScale:
            grainScale = defaultGrainScale
        case .grainSize:
            grainSize = defaultGrainSize
        case .grainAmount:
            grainAmount = defaultGrainAmount
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
        case .splitToningShadowHue:
            splitToningShadowHue = 215.0
            selectedSplitToningPresetId = nil
        case .splitToningShadowSat:
            splitToningShadowSat = 0.0
            selectedSplitToningPresetId = nil
        case .splitToningHighlightHue:
            splitToningHighlightHue = 45.0
            selectedSplitToningPresetId = nil
        case .splitToningHighlightSat:
            splitToningHighlightSat = 0.0
            selectedSplitToningPresetId = nil
        case .splitToningBalance:
            splitToningBalance = 0.0
            selectedSplitToningPresetId = nil
        case .printPaperGrade:
            printPaperGrade = 0.0
        case .printBaseWarmth:
            printBaseWarmth = 0.0
        case .printProcessDepth:
            printProcessDepth = 100.0
        }
        registerDiscreteEdit(title: "Reset \(type.displayName)", before: before)
    }
    
    public func applySplitToningPreset(_ preset: SplitToningPreset?) {
        let before = currentEdits
        guard let p = preset else {
            selectedSplitToningPresetId = nil
            splitToningShadowSat = 0.0
            splitToningHighlightSat = 0.0
            splitToningBalance = 0.0
            registerDiscreteEdit(title: "Split Toning (Off)", before: before)
            return
        }
        selectedSplitToningPresetId = p.id
        splitToningShadowHue = p.shadowHue
        splitToningShadowSat = p.shadowSat
        splitToningHighlightHue = p.highlightHue
        splitToningHighlightSat = p.highlightSat
        splitToningBalance = p.balance
        registerDiscreteEdit(title: "Split Toning: \(p.name)", before: before)
    }
    
    public func resetAllSliders() {
        let before = currentEdits
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
        emulsionArchetype = .none
        emulsionFormat = .format35mm
        emulsionExposure = defaultEmulsionExposure
        emulsionGrainMode = .off
        grainScale = defaultGrainScale
        grainSize = defaultGrainSize
        grainAmount = defaultGrainAmount
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
        cropSessionSnapshot = nil
        cropAspectRatioPreset = .original
        rotationDegrees = defaultExifRotation
        remjetBacking = .intact
        flipHorizontal = defaultExifFlipH
        flipVertical = defaultExifFlipV
        toneProfile = .agxVivid
        selectedLutName = nil
        lutIntensity = 1.0
        curves = .defaultIdentity
        splitToningShadowHue = 215.0
        splitToningShadowSat = 0.0
        splitToningHighlightHue = 45.0
        splitToningHighlightSat = 0.0
        splitToningBalance = 0.0
        printMedia = .none
        enlargerFilter = .neutralPanchromatic
        printPaperGrade = 0.0
        printBaseWarmth = 0.0
        printProcessDepth = 100.0
        registerDiscreteEdit(title: "Reset All Edits", before: before)
        notifyThumbnailChanged()
    }
    
    // MARK: - White Balance Eyedropper Sampling
    public func applyEyedropperSample(atNormalizedUV uv: CGPoint) {
        let before = currentEdits
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
                registerDiscreteEdit(title: "White Balance Pick", before: before)
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
            registerDiscreteEdit(title: "White Balance Pick", before: before)
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
            let before = currentEdits
            if let customLut = loadCubeLut(path: url.path) {
                if !self.customLuts.contains(where: { $0.title == customLut.title }) {
                    self.customLuts.append(customLut)
                }
                self.selectedLutName = customLut.title
                self.lutIntensity = 1.0
                registerDiscreteEdit(title: "3D LUT: \(customLut.title)", before: before)
            }
        }
    }
    
    public func setToneProfile(_ profile: ColorProfileMode) {
        guard self.toneProfile != profile else { return }
        let before = currentEdits
        self.toneProfile = profile
        let title: String
        switch profile {
        case .agxVivid: title = "Tone Profile: AgX Vivid"
        case .agxFilmic: title = "Tone Profile: AgX Neutral"
        case .cameraStandard: title = "Tone Profile: Camera Standard"
        case .cameraPortrait: title = "Tone Profile: Camera Portrait"
        case .cameraLandscape: title = "Tone Profile: Camera Landscape"
        case .cameraVivid: title = "Tone Profile: Camera Vivid"
        case .agxPhotographic: title = "Tone Profile: AgX Photographic"
        }
        registerDiscreteEdit(title: title, before: before)
        notifyThumbnailChanged()
    }
    
    public func setSelectedLutName(_ name: String?) {
        guard self.selectedLutName != name else { return }
        let before = currentEdits
        self.selectedLutName = name
        let title = name != nil ? "LUT: \(name!)" : "LUT: None"
        registerDiscreteEdit(title: title, before: before)
        notifyThumbnailChanged()
    }
    
    public func toggleCropConstrain() {
        let before = currentEdits
        cropConstrain.toggle()
        registerDiscreteEdit(title: cropConstrain ? "Enable Constrain Crop" : "Disable Constrain Crop", before: before)
    }
    
    // MARK: - Curated Fine-Art Print Media
    public func setPrintMedia(_ media: PrintMediaMode) {
        guard self.printMedia != media else { return }
        let before = currentEdits
        self.printMedia = media
        let title: String
        switch media {
        case .none: title = "Print Media: None (Display)"
        case .chromogenicTypeC: title = "Print Media: Chromogenic Type-C"
        case .silverDyeBleach: title = "Print Media: Silver Dye-Bleach"
        case .barytaSilverGelatin: title = "Print Media: Baryta Silver Gelatin"
        case .warmtoneChlorobromide: title = "Print Media: Warmtone Chlorobromide"
        case .platinumPalladium: title = "Print Media: Platinum/Palladium"
        case .cyanotype: title = "Print Media: Historic Cyanotype"
        case .bamakoSilverContact: title = "Print Media: Bamako Silver Contact"
        }
        registerDiscreteEdit(title: title, before: before)
        notifyThumbnailChanged()
    }
    
    public func setEnlargerFilter(_ filter: EnlargerFilterMode) {
        guard self.enlargerFilter != filter else { return }
        let before = currentEdits
        self.enlargerFilter = filter
        let title: String
        switch filter {
        case .neutralPanchromatic: title = "Enlarger Filter: Neutral"
        case .yellowK2: title = "Enlarger Filter: Yellow (K2)"
        case .orange16: title = "Enlarger Filter: Orange (16)"
        case .red25A: title = "Enlarger Filter: Red (25A)"
        case .greenX1: title = "Enlarger Filter: Green (X1)"
        case .orthochromatic: title = "Enlarger Filter: Orthochromatic"
        }
        registerDiscreteEdit(title: title, before: before)
        notifyThumbnailChanged()
    }
    
    public func resetPrintMedia() {
        let before = currentEdits
        self.printMedia = .none
        self.enlargerFilter = .neutralPanchromatic
        self.printPaperGrade = 0.0
        self.printBaseWarmth = 0.0
        self.printProcessDepth = 100.0
        registerDiscreteEdit(title: "Reset Print Media", before: before)
        notifyThumbnailChanged()
    }
    
    // MARK: - Upstream Recording Emulsions & Photochemical Grain
    public func setEmulsionArchetype(_ archetype: EmulsionArchetype) {
        guard self.emulsionArchetype != archetype else { return }
        let before = currentEdits
        self.emulsionArchetype = archetype
        if archetype != .none {
            if self.emulsionGrainMode == .off {
                self.emulsionGrainMode = .seedFromSensor
            }
        } else {
            self.emulsionGrainMode = .off
        }
        if archetype == .cambridgeDiffusion100 && (self.emulsionFormat == .super8 || self.emulsionFormat == .format16mm) {
            self.emulsionFormat = .format35mm
        }
        let title: String
        switch archetype {
        case .none: title = "Emulsion: Clean Sensor"
        case .rochesterPortrait400: title = "Emulsion: Rochester Portrait 400"
        case .ashigaraChrome50: title = "Emulsion: Ashigara Chrome 50"
        case .rochesterK14Chrome64: title = "Emulsion: Rochester K-14 Chrome 64"
        case .rochesterCinema500: title = "Emulsion: Rochester Cinema 500"
        case .rochesterDaylight200: title = "Emulsion: Rochester Daylight 200"
        case .mobberleySilver400: title = "Emulsion: Mobberley Silver 400"
        case .cambridgeDiffusion100: title = "Emulsion: Cambridge Diffusion 100"
        case .leverkusenPortrait160: title = "Emulsion: Leverkusen Portrait 160"
        case .ashigaraPortrait400: title = "Emulsion: Ashigara Portrait 400"
        case .liguriaCinema80: title = "Emulsion: Liguria Cinema 80"
        case .baodingSilver100: title = "Emulsion: Baoding Silver 100"
        }
        registerDiscreteEdit(title: title, before: before)
        notifyThumbnailChanged()
    }
    
    public func setEmulsionFormat(_ format: EmulsionFormat) {
        guard self.emulsionFormat != format else { return }
        let before = currentEdits
        self.emulsionFormat = format
        let title: String
        switch format {
        case .super8:
            title = "Film Format: Super 8 (7.3x Magnification)"
        case .format16mm:
            title = "Film Format: 16mm Cinema (3.4x Magnification)"
        case .format35mm:
            title = self.emulsionArchetype == .cambridgeDiffusion100
                ? "Print Format: Standard Frame (SX-70 / 600 / Packfilm)"
                : "Film Format: 35mm (1.0x Baseline)"
        case .format120:
            title = self.emulsionArchetype == .cambridgeDiffusion100
                ? "Print Format: 4x5 Large Format Sheet (Type 59)"
                : "Film Format: 120 (0.48x)"
        case .format4x5:
            title = self.emulsionArchetype == .cambridgeDiffusion100
                ? "Print Format: 8x10 Studio Sheet (Type 809)"
                : "Film Format: 4x5 (0.28x)"
        }
        registerDiscreteEdit(title: title, before: before)
        notifyThumbnailChanged()
    }
    
    public func setEmulsionGrainMode(_ mode: EmulsionGrainMode) {
        guard self.emulsionGrainMode != mode else { return }
        let before = currentEdits
        self.emulsionGrainMode = mode
        let title: String
        switch mode {
        case .off: title = "Grain: Off (Sensor Passthrough)"
        case .seedFromSensor: title = "Grain: Optical Transmutation"
        case .synthesizePhysical: title = "Grain: Synthetic Granularity"
        }
        registerDiscreteEdit(title: title, before: before)
        notifyThumbnailChanged()
    }
    
    public func setGrainScale(_ scale: Float) {
        self.grainScale = scale
        saveCurrentEdits()
    }
    
    public func setRemjetBacking(_ mode: RemjetBackingMode) {
        guard self.remjetBacking != mode else { return }
        let before = currentEdits
        self.remjetBacking = mode
        let title: String
        switch mode {
        case .intact: title = "Rem-Jet: Intact (Studio ECN-2)"
        case .stripped: title = "Rem-Jet: Stripped (Halation)"
        }
        registerDiscreteEdit(title: title, before: before)
        notifyThumbnailChanged()
    }
    
    public func resetEmulsion() {
        let before = currentEdits
        self.emulsionArchetype = .none
        self.emulsionFormat = .format35mm
        self.emulsionExposure = 0.0
        self.remjetBacking = .intact
        self.emulsionGrainMode = .off
        self.grainScale = 1.0
        self.grainSize = 1.0
        self.grainAmount = 1.0
        registerDiscreteEdit(title: "Reset Film Emulsion", before: before)
        notifyThumbnailChanged()
    }
}
