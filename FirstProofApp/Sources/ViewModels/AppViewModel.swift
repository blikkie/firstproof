import Foundation
import SwiftUI
import AppKit
import ImageIO
import FirstProofBridge

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
    case grainAmount
    case grainSize
    case grainRoughness
    case grainChroma
    
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
        case .grainAmount: return "Film Grain"
        case .grainSize: return "Grain Size"
        case .grainRoughness: return "Grain Roughness"
        case .grainChroma: return "Grain Color"
        }
    }
}

public enum CropAspectRatioPreset: String, CaseIterable, Identifiable {
    case original = "Original"
    case square = "1:1"
    case fourThree = "4:3"
    case threeTwo = "3:2"
    case sixteenNine = "16:9"
    case freeform = "Free"
    
    public var id: String { rawValue }
}

public enum WhiteBalancePreset: String, CaseIterable, Identifiable {
    case asShot = "As Shot"
    case auto = "Auto"
    case daylight = "Daylight"
    case cloudy = "Cloudy"
    case shade = "Shade"
    case tungsten = "Tungsten"
    case fluorescent = "Fluorescent"
    case flash = "Flash"
    case custom = "Custom"
    
    public var id: String { rawValue }
    public var displayName: String { rawValue }
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
    case metadata = "Info"
    
    public var id: String { rawValue }
    
    public var iconName: String {
        switch self {
        case .light: return "sun.max.fill"
        case .color: return "paintpalette.fill"
        case .optics: return "crop.rotate"
        case .detail: return "slider.horizontal.2.square"
        case .metadata: return "info.circle.fill"
        }
    }
}

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
    @ObservationIgnored var activeDecodeToken: UUID = UUID()
    @ObservationIgnored public var captureThumbnailSnapshot: (() -> NSImage?)? = nil
    @ObservationIgnored public var captureIntermediateSnapshot: (() -> NSImage?)? = nil
    public var isExportSheetPresented: Bool = false
    public var isExportPackageSheetPresented: Bool = false
    public var isFilmstripFollowingSelection: Bool = true
    public var filmstripScrollTrigger: UUID = UUID()
    public var gridColumnCount: Int = 4
    var currentlyLoadedPhotoPath: String? = nil
    @ObservationIgnored public var lastNavDirection: NavigationDirection = .neutral
    
    // MARK: - Editable Metadata Properties
    public var metadataTitle: String = ""
    public var metadataDescription: String = ""
    public var metadataCreator: String = ""
    public var metadataCopyright: String = ""
    public var metadataKeywords: String = ""
    @ObservationIgnored var metadataSaveWorkItem: DispatchWorkItem? = nil
    
    // MARK: - Batch Export Engine State
    public var isBatchExporting: Bool = false
    public var batchExportTotalCount: Int = 0
    public var batchExportCompletedCount: Int = 0
    public var batchExportStatusText: String = ""
    public var batchExportProgress: Double = 0.0
    public var lastBatchExportResults: [BatchExportResultRecord] = []
    @ObservationIgnored public var isBatchExportCancelled: Bool = false
    
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
    public var grainAmount: Float = 0.0          // 0 .. 100
    public var grainSize: Float = 1.0            // 0.5 .. 3.0
    public var grainRoughness: Float = 50.0      // 0 .. 100
    public var grainChroma: Float = 0.0          // 0 .. 100
    public var selectedGrainPresetId: String? = nil

    public var defaultSharpeningAmount: Float {
        (selectedPhoto?.isRaw ?? false) ? 20.0 : 0.0
    }
    public var defaultSharpeningRadius: Float { 1.0 }
    public var defaultSharpeningMasking: Float {
        (selectedPhoto?.isRaw ?? false) ? 15.0 : 0.0
    }
    public var defaultNoiseReduction: Float { 0.0 }
    public var defaultGrainAmount: Float { 0.0 }
    public var defaultGrainSize: Float { 1.0 }
    public var defaultGrainRoughness: Float { 50.0 }
    public var defaultGrainChroma: Float { 0.0 }

    public var isEyedropperActive: Bool = false
    public var temperature: Float = 5500.0 // 2000K .. 20000K
    public var tint: Float = 0.0          // -200 .. +200
    
    public var activeWhiteBalancePreset: WhiteBalancePreset {
        // Compare against As Shot
        if abs(temperature - asShotTemperature) < 25.0 && abs(tint - asShotTint) < 1.0 {
            return .asShot
        }
        // Daylight: 5500K, +10
        if abs(temperature - 5500.0) < 25.0 && abs(tint - 10.0) < 1.0 {
            return .daylight
        }
        // Cloudy: 6500K, +10
        if abs(temperature - 6500.0) < 25.0 && abs(tint - 10.0) < 1.0 {
            return .cloudy
        }
        // Shade: 7500K, +10
        if abs(temperature - 7500.0) < 25.0 && abs(tint - 10.0) < 1.0 {
            return .shade
        }
        // Tungsten: 2850K, 0
        if abs(temperature - 2850.0) < 25.0 && abs(tint - 0.0) < 1.0 {
            return .tungsten
        }
        // Fluorescent: 3800K, +45
        if abs(temperature - 3800.0) < 25.0 && abs(tint - 45.0) < 1.0 {
            return .fluorescent
        }
        // Flash: 5500K, 0
        if abs(temperature - 5500.0) < 25.0 && abs(tint - 0.0) < 1.0 {
            return .flash
        }
        return .custom
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
        self.availableLutTitles = getBuiltinCubeLutTitles()
        
        // Restore multi-project opened folder list (preserves disconnected external drive sessions)
        self.openedFolders = UserDefaults.standard.stringArray(forKey: "openedFolders") ?? []
        
        let lastPath = UserDefaults.standard.string(forKey: "lastOpenedDirectory")
        if let target = lastPath, FileManager.default.fileExists(atPath: target) {
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
    }
    
    deinit {
        if let observer = screenChangeObserver {
            NotificationCenter.default.removeObserver(observer)
        }
        if let observer = windowChangeObserver {
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
            self?.updateScreenCapabilities()
        }
        
        windowChangeObserver = NotificationCenter.default.addObserver(
            forName: NSWindow.didChangeScreenNotification,
            object: nil,
            queue: .main
        ) { [weak self] note in
            let win = note.object as? NSWindow
            self?.updateScreenCapabilities(for: win)
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
        metadataSaveWorkItem?.cancel()

        let title = metadataTitle.trimmingCharacters(in: .whitespacesAndNewlines)
        let description = metadataDescription.trimmingCharacters(in: .whitespacesAndNewlines)
        let creator = metadataCreator.trimmingCharacters(in: .whitespacesAndNewlines)
        let copyright = metadataCopyright.trimmingCharacters(in: .whitespacesAndNewlines)
        let keywords = metadataKeywords.trimmingCharacters(in: .whitespacesAndNewlines)

        let item = DispatchWorkItem { [weak self] in
            guard let self = self else { return }
            self.saveMetadata(
                for: path,
                title: title.isEmpty ? nil : title,
                description: description.isEmpty ? nil : description,
                creator: creator.isEmpty ? nil : creator,
                copyright: copyright.isEmpty ? nil : copyright,
                keywords: keywords.isEmpty ? nil : keywords
            )
        }
        metadataSaveWorkItem = item
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.4, execute: item)
    }

    public func flushPendingMetadataSave() {
        if let workItem = metadataSaveWorkItem {
            workItem.perform()
            metadataSaveWorkItem = nil
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

        // Sync to XMP sidecar
        let activeSnapshot = currentEdits
        DispatchQueue.global(qos: .utility).async { [weak self] in
            guard let self = self else { return }
            let existing = readXmp(path: path)
            let splitKeywords: [String] = keywords?
                .split(separator: ",")
                .map { $0.trimmingCharacters(in: .whitespacesAndNewlines) }
                .filter { !$0.isEmpty } ?? existing?.keywords ?? []

            let currentRating = self.photos.first(where: { $0.filePath == path })?.rating ?? existing?.rating
            let currentStatus = self.photos.first(where: { $0.filePath == path })?.status ?? existing?.status ?? "None"
            let currentLat = self.photos.first(where: { $0.filePath == path })?.latitude ?? existing?.latitude
            let currentLon = self.photos.first(where: { $0.filePath == path })?.longitude ?? existing?.longitude

            let record = XmpDataRecord(
                rating: currentRating,
                status: currentStatus,
                title: title ?? existing?.title,
                description: description ?? existing?.description,
                creator: creator ?? existing?.creator,
                copyright: copyright ?? existing?.copyright,
                keywords: splitKeywords,
                latitude: currentLat,
                longitude: currentLon,
                edits: (path == self.currentlyLoadedPhotoPath) ? activeSnapshot : (existing?.edits ?? EditSettings.defaultIdentity)
            )
            _ = writeXmp(path: path, record: record)
        }
    }
}
