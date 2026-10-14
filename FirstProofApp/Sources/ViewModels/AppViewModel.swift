import Foundation
import SwiftUI
import AppKit
import ImageIO
@_exported import FirstProofBridge

extension PhotoRecord {
    public var isRaw: Bool {
        let ext = URL(fileURLWithPath: filePath).pathExtension.lowercased()
        return ["cr2", "cr3", "nef", "arw", "dng", "orf", "rw2", "raf", "pef", "raw"].contains(ext)
    }
}

extension ToneCurvesRecord {
    public static var defaultIdentity: ToneCurvesRecord {
        ToneCurvesRecord(
            master: [CurvePointRecord(x: 0.0, y: 0.0), CurvePointRecord(x: 1.0, y: 1.0)],
            red: [CurvePointRecord(x: 0.0, y: 0.0), CurvePointRecord(x: 1.0, y: 1.0)],
            green: [CurvePointRecord(x: 0.0, y: 0.0), CurvePointRecord(x: 1.0, y: 1.0)],
            blue: [CurvePointRecord(x: 0.0, y: 0.0), CurvePointRecord(x: 1.0, y: 1.0)]
        )
    }
}

public enum EditSliderType {
    case exposure
    case contrast
    case highlights
    case shadows
    case whites
    case blacks
    case vibrance
    case saturation
    case deepBlue
    case foliage
    case sharpeningAmount
    case sharpeningRadius
    case sharpeningMasking
    case noiseReduction
    case temperature
    case tint
    case straighten
    case perspectiveVertical
    case perspectiveHorizontal
    case lutIntensity
    case grainScale
    case grainSize
    case grainAmount
    case splitToningShadowHue
    case splitToningShadowSat
    case splitToningHighlightHue
    case splitToningHighlightSat
    case splitToningBalance
    case printPaperGrade
    case printBaseWarmth
    case printProcessDepth
    
    public var displayName: String {
        switch self {
        case .exposure: return "Exposure"
        case .contrast: return "Contrast"
        case .highlights: return "Highlights"
        case .shadows: return "Shadows"
        case .whites: return "Whites"
        case .blacks: return "Blacks"
        case .vibrance: return "Vibrance"
        case .saturation: return "Saturation"
        case .deepBlue: return "Deep Blue"
        case .foliage: return "Foliage"
        case .sharpeningAmount: return "Sharpening"
        case .sharpeningRadius: return "Sharpening Radius"
        case .sharpeningMasking: return "Sharpening Masking"
        case .noiseReduction: return "Noise Reduction"
        case .temperature: return "Temperature"
        case .tint: return "Tint"
        case .straighten: return "Straighten"
        case .perspectiveVertical: return "Perspective Vertical"
        case .perspectiveHorizontal: return "Perspective Horizontal"
        case .lutIntensity: return "LUT Intensity"
        case .grainScale: return "Grain Scale"
        case .grainSize: return "Grain Size"
        case .grainAmount: return "Grain Amount"
        case .splitToningShadowHue: return "Shadow Hue"
        case .splitToningShadowSat: return "Shadow Saturation"
        case .splitToningHighlightHue: return "Highlight Hue"
        case .splitToningHighlightSat: return "Highlight Saturation"
        case .splitToningBalance: return "Split Toning Balance"
        case .printPaperGrade: return "Paper Grade"
        case .printBaseWarmth: return "Base Warmth"
        case .printProcessDepth: return "Process Depth"
        }
    }
    
    public var descriptorId: String? {
        switch self {
        case .exposure: return "exposure"
        case .contrast: return "contrast"
        case .highlights: return "highlights"
        case .shadows: return "shadows"
        case .whites: return "whites"
        case .blacks: return "blacks"
        case .temperature: return "temperature"
        case .tint: return "tint"
        case .vibrance: return "vibrance"
        case .saturation: return "saturation"
        case .deepBlue: return "deep_blue"
        case .foliage: return "foliage"
        case .sharpeningAmount: return "sharpen_amount"
        case .sharpeningRadius: return "sharpen_radius"
        case .sharpeningMasking: return "sharpen_masking"
        case .noiseReduction: return "noise_reduction"
        case .printPaperGrade: return "paper_grade"
        case .printProcessDepth: return "process_depth"
        default: return nil
        }
    }
    
    public var descriptor: AdjustmentDescriptor? {
        guard let id = descriptorId else { return nil }
        return getAdjustmentDescriptor(id: id)
    }
    
    public var defaultRange: ClosedRange<Float> {
        if let desc = descriptor {
            return Float(desc.min)...Float(desc.max)
        }
        switch self {
        case .straighten: return -45.0...45.0
        case .perspectiveVertical, .perspectiveHorizontal: return -100.0...100.0
        case .lutIntensity: return 0.0...100.0
        case .grainScale: return 0.5...2.0
        case .grainSize: return 0.5...2.5
        case .grainAmount: return 0.0...2.0
        case .splitToningShadowHue, .splitToningHighlightHue: return 0.0...360.0
        case .splitToningShadowSat, .splitToningHighlightSat: return 0.0...100.0
        case .splitToningBalance: return -100.0...100.0
        default: return -100.0...100.0
        }
    }
    
    public var formatString: String {
        switch self {
        case .straighten: return "%.1f°"
        case .grainScale: return "%.1fx"
        case .grainSize, .grainAmount: return "%.2fx"
        case .lutIntensity: return "%.0f%%"
        case .splitToningShadowHue, .splitToningHighlightHue: return "%.0f°"
        default: return "%.0f"
        }
    }
}

extension AdjustmentDescriptor {
    public func formatValue(_ val: Double) -> String {
        switch scaleMode {
        case .mired:
            return String(format: "%.0f K", val)
        case .linear:
            let prefix = (unit == "EV" && val > 0.0) ? "+" : ""
            let unitSuffix = unit.isEmpty ? "" : (unit == "%" ? "%" : " \(unit)")
            if precision == 0 {
                return String(format: "%@%.0f%@", prefix, val, unitSuffix)
            } else {
                return String(format: "%@%.*f%@", prefix, Int(precision), val, unitSuffix)
            }
        }
    }
}

extension CropAspectRatioPreset: @retroactive CaseIterable, @retroactive Identifiable {
    public static var allCases: [CropAspectRatioPreset] {
        [.original, .square, .fourThree, .threeTwo, .sixteenNine, .freeform]
    }
    public var id: String { rawValue }
    public var rawValue: String {
        switch self {
        case .original: return "Original"
        case .square: return "1:1"
        case .fourThree: return "4:3"
        case .threeTwo: return "3:2"
        case .sixteenNine: return "16:9"
        case .freeform: return "Free"
        }
    }
}

extension WhiteBalancePreset: @retroactive CaseIterable, @retroactive Identifiable {
    public static var allCases: [WhiteBalancePreset] {
        [.asShot, .auto, .daylight, .cloudy, .shade, .tungsten, .fluorescent, .sodiumVapor, .flash, .custom]
    }
    public var id: String { displayName }
    public var displayName: String {
        switch self {
        case .asShot: return "As Shot"
        case .auto: return "Auto"
        case .daylight: return "Daylight"
        case .cloudy: return "Cloudy"
        case .shade: return "Shade"
        case .tungsten: return "Tungsten"
        case .fluorescent: return "Fluorescent"
        case .sodiumVapor: return "Sodium Vapor"
        case .flash: return "Flash"
        case .custom: return "Custom"
        }
    }
}

public enum CanvasMode: Hashable {
    case grid
    case detail
}

public enum LibraryScope: Hashable, Equatable {
    case mainLibrary
    case session(folderPath: String)
}

public enum InspectorTab: String, CaseIterable, Identifiable {
    case light = "Light"
    case color = "Color"
    case optics = "Optics"
    case detail = "Detail"
    case emulsion = "Emulsion"
    case metadata = "Info"
    
    public var id: String { rawValue }
    
    public var iconName: String {
        switch self {
        case .light: return "sun.max.fill"
        case .color: return "paintpalette.fill"
        case .optics: return "crop.rotate"
        case .detail: return "slider.horizontal.2.square"
        case .emulsion: return "film"
        case .metadata: return "info.circle.fill"
        }
    }
}

@MainActor
@Observable
public final class AppViewModel {
    // MARK: - Navigation & Directory State
    public var currentScope: LibraryScope = .mainLibrary
    public var canvasMode: CanvasMode = .grid
    public var currentFolderPath: String? = nil
    public var openedFolders: [String] = []
    public var isExternalVolume: Bool = false
    public var isPortableCatalog: Bool = false
    public var isScanning: Bool = false
    public var scanningStatusMessage: String? = nil
    
    // Main Library Path (~/Pictures/First Proof Library)
    public var mainLibraryPath: String {
        if let custom = UserDefaults.standard.string(forKey: "customMainLibraryPath"), !custom.isEmpty {
            return custom
        }
        let pictures = FileManager.default.urls(for: .picturesDirectory, in: .userDomainMask).first ?? URL(fileURLWithPath: NSHomeDirectory()).appendingPathComponent("Pictures")
        let firstProofLib = pictures.appendingPathComponent("First Proof Library")
        try? FileManager.default.createDirectory(at: firstProofLib, withIntermediateDirectories: true)
        return firstProofLib.path
    }
    
    // MARK: - Photos & Selection
    public var photos: [PhotoRecord] = []
    public var selectedPhoto: PhotoRecord? = nil
    public var selectedPhotoPaths: Set<String> = []
    public var isBulkSelectionMode: Bool = false
    public var previewImage: NSImage? = nil
    public var rawPreviewRecord: RawPreviewRecord? = nil
    public var isIntermediatePreviewActive: Bool = false
    public var currentGainMapRecord: GainMapRecord? = nil
    public var isDemosaicing: Bool = false
    
    /// Canonical uncropped sensor/image dimensions in pixels/points.
    public var currentImageDimensions: (width: CGFloat, height: CGFloat) {
        if let raw = rawPreviewRecord, raw.width > 0, raw.height > 0 {
            return (CGFloat(raw.width), CGFloat(raw.height))
        }
        if let img = previewImage, img.size.width > 0, img.size.height > 0 {
            return (img.size.width, img.size.height)
        }
        return (4000.0, 3000.0)
    }
    
    @ObservationIgnored var activeDecodeToken: UUID = UUID()
    @ObservationIgnored var activeDecodeTask: Task<Void, Never>? = nil
    @ObservationIgnored var activeCatalogTask: Task<Void, Never>? = nil
    @ObservationIgnored public var captureThumbnailSnapshot: (() -> NSImage?)? = nil
    @ObservationIgnored public var captureIntermediateSnapshot: (() -> NSImage?)? = nil
    public var isExportSheetPresented: Bool = false
    public var isExportPackageSheetPresented: Bool = false
    
    // MARK: - Ingestion & Import State (Story 2.3)
    public var isImportSheetPresented: Bool = false
    public var importSourcePath: String? = nil
    public var importSelectedPaths: [String] = []
    public var isCameraCardBannerPresented: Bool = false
    public var detectedCameraCard: CameraCardInfo? = nil
    
    public var isFilmstripFollowingSelection: Bool = true
    public var filmstripScrollTrigger: UUID = UUID()
    public var gridColumnCount: Int = 4
    
    // MARK: - Navigation & Inspector State (macOS 14 Modernization)
    public var isInspectorPresented: Bool = true
    public var columnVisibility: NavigationSplitViewVisibility = .all
    
    public func toggleInspector() {
        isInspectorPresented.toggle()
    }
    
    var currentlyLoadedPhotoPath: String? = nil
    @ObservationIgnored public var lastNavDirection: NavigationDirection = .neutral
    
    // MARK: - Editable Metadata Properties
    public var metadataTitle: String = ""
    public var metadataDescription: String = ""
    public var metadataCreator: String = ""
    public var metadataCopyright: String = ""
    public var metadataKeywords: String = ""
    @ObservationIgnored private var metadataSaveTask: Task<Void, Never>? = nil
    @ObservationIgnored private var pendingMetadataSaveBlock: (() -> Void)? = nil
    
    // MARK: - Batch Export Engine State
    public var isBatchExporting: Bool = false
    public var batchExportTotalCount: Int = 0
    public var batchExportCompletedCount: Int = 0
    public var batchExportStatusText: String = ""
    public var batchExportProgress: Double = 0.0
    public var lastBatchExportResults: [BatchExportResultRecord] = []
    @ObservationIgnored nonisolated(unsafe) public var isBatchExportCancelled: Bool = false
    
    public var selectedPhotosList: [PhotoRecord] {
        if !selectedPhotoPaths.isEmpty {
            let set = selectedPhotoPaths
            return photos.filter { set.contains($0.filePath) }
        } else if let current = selectedPhoto {
            return [current]
        }
        return []
    }
    
    // MARK: - EDR & HDR Display State (Story 18.1)
    public var isHdrEnabled: Bool = false
    public var edrHeadroom: Float = 1.0
    public var isScreenHdrCapable: Bool = false
    @ObservationIgnored private var screenChangeObserver: NSObjectProtocol?
    @ObservationIgnored private var windowChangeObserver: NSObjectProtocol?
    @ObservationIgnored private var appTerminationObserver: NSObjectProtocol?
    
    public func cancelBatchExport() {
        self.isBatchExportCancelled = true
        self.isBatchExporting = false
        self.batchExportStatusText = "Export cancelled"
    }
    
    public func resetFilmstripFollow() {
        self.isFilmstripFollowingSelection = true
        self.filmstripScrollTrigger = UUID()
    }
    
    public var selectedPhotoIsRaw: Bool {
        guard let path = selectedPhoto?.filePath else { return false }
        let ext = URL(fileURLWithPath: path).pathExtension.lowercased()
        return ["cr2", "cr3", "nef", "arw", "dng", "orf", "rw2", "raf", "pef", "raw"].contains(ext)
    }
    
    // MARK: - Filters
    public var ratingFilter: Int? = nil // nil = All, 1..5
    public var statusFilter: String? = nil // nil = All, "pick", "reject"
    
    // MARK: - Edit Sliders (P0 + Lens Corrections)
    public var exposure: Float = 0.0      // -3.0 .. +3.0 EV
    public var contrast: Float = 0.0      // -100 .. +100
    public var highlights: Float = 0.0    // -100 .. +100
    public var shadows: Float = 0.0       // -100 .. +100
    public var whites: Float = 0.0        // -100 .. +100
    public var blacks: Float = 0.0        // -100 .. +100
    public var vibrance: Float = 0.0      // -100 .. +100
    public var saturation: Float = 0.0    // -100 .. +100
    public var deepBlue: Float = 0.0      // -100 .. +100
    public var foliage: Float = 0.0       // -100 .. +100
    public var sharpeningAmount: Float = 0.0     // 0 .. 100
    public var sharpeningRadius: Float = 1.0     // 0.5 .. 3.0 px
    public var sharpeningMasking: Float = 0.0    // 0 .. 100
    public var noiseReduction: Float = 0.0       // 0 .. 100

    // MARK: - Upstream Recording Emulsions & Photochemical Grain
    public var emulsionArchetype: EmulsionArchetype = .none
    public var emulsionFormat: EmulsionFormat = .format35mm
    public var emulsionExposure: Float = 0.0          // -2.0 .. +2.0 EV
    public var remjetBacking: RemjetBackingMode = .intact
    public var defaultEmulsionExposure: Float { 0.0 }
    public var emulsionGrainMode: EmulsionGrainMode = .off
    public var grainScale: Float = 1.0                // 0.5 .. 2.0
    public var defaultGrainScale: Float { 1.0 }
    public var grainSize: Float = 1.0                 // 0.2 .. 3.0
    public var defaultGrainSize: Float { 1.0 }
    public var grainAmount: Float = 1.0               // 0.0 .. 2.0
    public var defaultGrainAmount: Float { 1.0 }
    
    public var splitToningShadowHue: Float = 215.0      // 0 .. 360
    public var splitToningShadowSat: Float = 0.0        // 0 .. 100
    public var splitToningHighlightHue: Float = 45.0    // 0 .. 360
    public var splitToningHighlightSat: Float = 0.0     // 0 .. 100
    public var splitToningBalance: Float = 0.0          // -100 .. +100
    public var selectedSplitToningPresetId: String? = nil
    
    // MARK: - Curated Fine-Art Print Media Archetypes & Optical Filtration
    public var printMedia: PrintMediaMode = .none
    public var enlargerFilter: EnlargerFilterMode = .neutralPanchromatic
    public var printPaperGrade: Float = 0.0     // -2.0 .. +2.0
    public var printBaseWarmth: Float = 0.0     // -100 .. +100
    public var printProcessDepth: Float = 100.0 // 0 .. 100

    public var defaultSharpeningAmount: Float { 0.0 }
    public var defaultSharpeningRadius: Float { 1.0 }
    public var defaultSharpeningMasking: Float { 0.0 }
    public var defaultNoiseReduction: Float { 0.0 }

    public var isEyedropperActive: Bool = false
    public var temperature: Float = 5500.0 // 2000K .. 20000K
    public var tint: Float = 0.0          // -200 .. +200
    
    public var activeWhiteBalancePreset: WhiteBalancePreset {
        detectWhiteBalancePreset(
            temperature: Double(temperature),
            tint: Double(tint),
            asShotTemp: Double(asShotTemperature),
            asShotTint: Double(asShotTint)
        )
    }
    public var straightenAngle: Float = 0.0 // -45 .. +45 deg
    public var cropLeft: Float = 0.0
    public var cropTop: Float = 0.0
    public var cropRight: Float = 1.0
    public var cropBottom: Float = 1.0
    public var perspectiveVertical: Float = 0.0 // -100 .. +100
    public var perspectiveHorizontal: Float = 0.0 // -100 .. +100
    public var cropConstrain: Bool = true
    public var isCropToolActive: Bool = false
    public var cropAspectRatioPreset: CropAspectRatioPreset = .original
    @ObservationIgnored public var cropSessionSnapshot: CropSessionSnapshot? = nil
    public var lensCorrectionEnabled: Bool = true
    public var toneProfile: ColorProfileMode = .agxVivid
    public var selectedLutName: String? = nil
    public var lutIntensity: Float = 1.0
    public var availableLutTitles: [String] = []
    public var customLuts: [CubeLutRecord] = []
    public var curves: ToneCurvesRecord = .defaultIdentity
    
    // MARK: - Workspace Navigation & Tabs
    public var selectedInspectorTab: InspectorTab = .light
    public var rotationDegrees: Int = 0 // 0, 90, 180, 270
    public var flipHorizontal: Bool = false
    public var flipVertical: Bool = false
    public var defaultExifRotation: Int = 0
    public var defaultExifFlipH: Bool = false
    public var defaultExifFlipV: Bool = false
    public var isShowingOriginal: Bool = false
    public var isShowingCameraJpeg: Bool = false
    
    // MARK: - Undo/Redo & Session History
    public var undoHistory = UndoHistoryManager()
    
    public var canUndo: Bool {
        let activePath = currentlyLoadedPhotoPath ?? selectedPhoto?.filePath
        return undoHistory.canUndo(isDetailMode: canvasMode == .detail, activePhotoPath: activePath)
    }
    public var canRedo: Bool {
        let activePath = currentlyLoadedPhotoPath ?? selectedPhoto?.filePath
        return undoHistory.canRedo(isDetailMode: canvasMode == .detail, activePhotoPath: activePath)
    }
    public var undoActionTitle: String {
        let activePath = currentlyLoadedPhotoPath ?? selectedPhoto?.filePath
        return undoHistory.undoTitle(isDetailMode: canvasMode == .detail, activePhotoPath: activePath)
    }
    public var redoActionTitle: String {
        let activePath = currentlyLoadedPhotoPath ?? selectedPhoto?.filePath
        return undoHistory.redoTitle(isDetailMode: canvasMode == .detail, activePhotoPath: activePath)
    }
    
    // MARK: - Viewport Zoom & Pan
    public var zoomScale: Float = 1.0
    public var panOffset: CGPoint = .zero
    public var zoomScale100: Float = 1.0
    
    public var isZoomed: Bool {
        zoomScale > 1.05
    }
    
    public var zoomPercentage: Int {
        if zoomScale <= 1.02 {
            let fitRatio = (1.0 / max(1.0, zoomScale100)) * 100.0
            return max(1, Int(round(fitRatio)))
        } else {
            let actualRatio = (zoomScale / max(1.0, zoomScale100)) * 100.0
            return max(1, Int(round(actualRatio)))
        }
    }
    
    public var zoomReadoutText: String {
        if zoomScale <= 1.02 {
            return "Fit (\(zoomPercentage)%)"
        } else {
            return "\(zoomPercentage)%"
        }
    }
    
    // MARK: - Takeout & Metadata
    public var takeoutMatches: [TakeoutMatchRecord] = []
    public var lensCoefficients: LensCorrectionCoefficientsRecord? = nil
    
    // MARK: - As-Shot Baselines
    public var asShotTemperature: Float = 5500.0
    public var asShotTint: Float = 0.0
    
    // MARK: - Rust Core Objects
    var db: FirstProofDatabase? = nil
    var lensfun: FirstProofLensfun? = nil
    
    public init() {
        self.lensfun = FirstProofLensfun()
        if let bundlePath = Bundle.main.path(forResource: "lensfun", ofType: "json") {
            _ = self.lensfun?.loadJsonFile(path: bundlePath)
        }
        self.availableLutTitles = getBuiltinCubeLutTitles()
        
        // Restore multi-project opened folder list (preserves disconnected external drive sessions)
        self.openedFolders = UserDefaults.standard.stringArray(forKey: "openedFolders") ?? []
        
        let lastScope = UserDefaults.standard.string(forKey: "lastSelectedScope") ?? UserDefaults.standard.string(forKey: "lastOpenedDirectory")
        if lastScope == "main_library" {
            selectScope(.mainLibrary)
        } else if let target = lastScope, FileManager.default.fileExists(atPath: target) {
            if !self.openedFolders.contains(target) {
                self.openedFolders.append(target)
            }
            selectScope(.session(folderPath: target))
        } else if let firstAvailable = self.openedFolders.first(where: { FileManager.default.fileExists(atPath: $0) }) {
            selectScope(.session(folderPath: firstAvailable))
        } else {
            selectScope(.mainLibrary)
        }
        
        setupScreenObservers()
        setupMediaDetection()
        setupTerminationObserver()
    }
    
    private func setupTerminationObserver() {
        appTerminationObserver = NotificationCenter.default.addObserver(
            forName: NSApplication.willTerminateNotification,
            object: nil,
            queue: .main
        ) { [weak self] _ in
            MainActor.assumeIsolated {
                self?.saveCurrentEdits()
                self?.checkpointDatabase()
            }
        }
    }
    
    // MARK: - Ingestion & Camera Card Detection (Story 2.3)
    private func setupMediaDetection() {
        let detector = MediaDetectionService.shared
        detector.onCameraCardDetected = { [weak self] card in
            guard let self = self else { return }
            let autoPrompt = UserDefaults.standard.object(forKey: "autoDetectMemoryCards") as? Bool ?? true
            if autoPrompt {
                self.detectedCameraCard = card
                self.isCameraCardBannerPresented = true
            }
        }
        detector.onVolumeUnmounted = { [weak self] path in
            guard let self = self else { return }
            if self.detectedCameraCard?.path == path {
                self.isCameraCardBannerPresented = false
                self.detectedCameraCard = nil
            }
        }
        detector.start()
    }
    
    public func promptImport(from sourcePath: String? = nil, selectedPaths: [String] = []) {
        self.importSourcePath = sourcePath ?? currentFolderPath
        self.importSelectedPaths = selectedPaths
        self.isImportSheetPresented = true
    }
    
    // MARK: - Safe Database Checkpoint & Close
    public func checkpointDatabase() {
        db?.close()
    }
    
    deinit {
        if let observer = screenChangeObserver {
            NotificationCenter.default.removeObserver(observer)
        }
        if let observer = windowChangeObserver {
            NotificationCenter.default.removeObserver(observer)
        }
        if let observer = appTerminationObserver {
            NotificationCenter.default.removeObserver(observer)
        }
    }
    
    // MARK: - Display & EDR Headroom Detection
    public func updateScreenCapabilities(for window: NSWindow? = nil) {
        let screen = window?.screen ?? NSScreen.main ?? NSScreen.screens.first
        guard let screen = screen else {
            if self.isScreenHdrCapable != false || self.edrHeadroom != 1.0 {
                self.isScreenHdrCapable = false
                self.edrHeadroom = 1.0
            }
            return
        }
        
        let potentialHeadroom = screen.maximumPotentialExtendedDynamicRangeColorComponentValue
        let referenceHeadroom = screen.maximumReferenceExtendedDynamicRangeColorComponentValue
        let currentHeadroom = screen.maximumExtendedDynamicRangeColorComponentValue
        
        // True HDR displays on macOS:
        // 1. Apple Reference displays (Liquid Retina XDR on MacBook Pro 14"/16", Pro Display XDR): reference > 1.0.
        // 2. External HDR monitors with HDR active in macOS Display Settings: potential > 2.05 (typically 3.0 to 16.0).
        // Standard SDR displays (like MacBook Air or SDR monitors) report potential <= 2.0 even when dimmed.
        let isTrueHdr = referenceHeadroom > 1.0 || potentialHeadroom > 2.05
        let headroom: Float
        if isTrueHdr && currentHeadroom > 1.05 {
            headroom = Float(currentHeadroom)
        } else {
            headroom = 1.0
        }
        
        if self.isScreenHdrCapable != isTrueHdr || abs(self.edrHeadroom - headroom) > 0.01 {
            self.isScreenHdrCapable = isTrueHdr
            self.edrHeadroom = headroom
            if !isTrueHdr {
                self.isHdrEnabled = false
            }
        }
    }
    
    private func setupScreenObservers() {
        updateScreenCapabilities()
        
        screenChangeObserver = NotificationCenter.default.addObserver(
            forName: NSApplication.didChangeScreenParametersNotification,
            object: nil,
            queue: .main
        ) { [weak self] _ in
            MainActor.assumeIsolated {
                self?.updateScreenCapabilities()
            }
        }
        
        windowChangeObserver = NotificationCenter.default.addObserver(
            forName: NSWindow.didChangeScreenNotification,
            object: nil,
            queue: .main
        ) { [weak self] note in
            let win = note.object as? NSWindow
            MainActor.assumeIsolated {
                self?.updateScreenCapabilities(for: win)
            }
        }
    }
    
    // MARK: - Metadata Persistence & Sync
    public func loadMetadataForSelectedPhoto() {
        guard let photo = selectedPhoto else {
            metadataTitle = ""
            metadataDescription = ""
            metadataCreator = ""
            metadataCopyright = ""
            metadataKeywords = ""
            return
        }
        metadataTitle = photo.title ?? ""
        metadataDescription = photo.description ?? ""
        metadataCreator = photo.creator ?? ""
        metadataCopyright = photo.copyright ?? ""
        metadataKeywords = photo.keywords ?? ""
    }

    public func scheduleMetadataSave() {
        guard let path = selectedPhoto?.filePath else { return }
        metadataSaveTask?.cancel()

        let title = metadataTitle.trimmingCharacters(in: .whitespacesAndNewlines)
        let description = metadataDescription.trimmingCharacters(in: .whitespacesAndNewlines)
        let creator = metadataCreator.trimmingCharacters(in: .whitespacesAndNewlines)
        let copyright = metadataCopyright.trimmingCharacters(in: .whitespacesAndNewlines)
        let keywords = metadataKeywords.trimmingCharacters(in: .whitespacesAndNewlines)

        let saveAction: () -> Void = { [weak self] in
            self?.saveMetadata(
                for: path,
                title: title.isEmpty ? nil : title,
                description: description.isEmpty ? nil : description,
                creator: creator.isEmpty ? nil : creator,
                copyright: copyright.isEmpty ? nil : copyright,
                keywords: keywords.isEmpty ? nil : keywords
            )
        }
        self.pendingMetadataSaveBlock = saveAction

        metadataSaveTask = Task { [weak self] in
            try? await Task.sleep(nanoseconds: 400_000_000)
            guard !Task.isCancelled else { return }
            self?.pendingMetadataSaveBlock = nil
            saveAction()
        }
    }

    public func flushPendingMetadataSave() {
        metadataSaveTask?.cancel()
        metadataSaveTask = nil
        if let action = pendingMetadataSaveBlock {
            pendingMetadataSaveBlock = nil
            action()
        }
    }

    public func saveMetadata(
        for path: String,
        title: String?,
        description: String?,
        creator: String?,
        copyright: String?,
        keywords: String?
    ) {
        guard let db = db else { return }
        _ = db.updatePhotoMetadata(
            filePath: path,
            title: title,
            description: description,
            creator: creator,
            copyright: copyright,
            keywords: keywords
        )

        // Update in-memory records
        if let idx = photos.firstIndex(where: { $0.filePath == path }) {
            photos[idx].title = title
            photos[idx].description = description
            photos[idx].creator = creator
            photos[idx].copyright = copyright
            photos[idx].keywords = keywords
        }
        if selectedPhoto?.filePath == path {
            selectedPhoto?.title = title
            selectedPhoto?.description = description
            selectedPhoto?.creator = creator
            selectedPhoto?.copyright = copyright
            selectedPhoto?.keywords = keywords
        }

        // Sync to XMP sidecar off main thread
        let activeSnapshot = currentEdits
        let currentRating = photos.first(where: { $0.filePath == path })?.rating
        let currentStatus = photos.first(where: { $0.filePath == path })?.status ?? "None"
        let currentLat = photos.first(where: { $0.filePath == path })?.latitude
        let currentLon = photos.first(where: { $0.filePath == path })?.longitude
        let isCurrent = (path == currentlyLoadedPhotoPath)

        Task.detached(priority: .utility) {
            let existing = readXmp(path: path)
            let splitKeywords: [String] = keywords?
                .split(separator: ",")
                .map { $0.trimmingCharacters(in: .whitespacesAndNewlines) }
                .filter { !$0.isEmpty } ?? existing?.keywords ?? []

            let record = XmpDataRecord(
                rating: currentRating ?? existing?.rating,
                status: currentStatus,
                title: title ?? existing?.title,
                description: description ?? existing?.description,
                creator: creator ?? existing?.creator,
                copyright: copyright ?? existing?.copyright,
                keywords: splitKeywords,
                latitude: currentLat ?? existing?.latitude,
                longitude: currentLon ?? existing?.longitude,
                edits: isCurrent ? activeSnapshot : (existing?.edits ?? EditSettings.defaultIdentity)
            )
            _ = writeXmp(path: path, record: record)
        }
    }
}
