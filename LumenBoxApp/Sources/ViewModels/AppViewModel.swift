import Foundation
import SwiftUI
import AppKit
import ImageIO
import LumenBoxBridge

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
    
    // Main Library Path (~/Pictures/LumenBox Library)
    public var mainLibraryPath: String {
        let pictures = FileManager.default.urls(for: .picturesDirectory, in: .userDomainMask).first ?? URL(fileURLWithPath: NSHomeDirectory()).appendingPathComponent("Pictures")
        let lumenBoxLib = pictures.appendingPathComponent("LumenBox Library")
        try? FileManager.default.createDirectory(at: lumenBoxLib, withIntermediateDirectories: true)
        return lumenBoxLib.path
    }
    
    // MARK: - Photos & Selection
    public var photos: [PhotoRecord] = []
    public var selectedPhoto: PhotoRecord? = nil
    public var selectedPhotoPaths: Set<String> = []
    public var isBulkSelectionMode: Bool = false
    public var previewImage: NSImage? = nil
    public var rawPreviewRecord: RawPreviewRecord? = nil
    public var currentGainMapRecord: GainMapRecord? = nil
    public var isDemosaicing: Bool = false
    @ObservationIgnored var activeDecodeToken: UUID = UUID()
    @ObservationIgnored public var captureThumbnailSnapshot: (() -> NSImage?)? = nil
    public var isExportSheetPresented: Bool = false
    public var isExportPackageSheetPresented: Bool = false
    public var isFilmstripFollowingSelection: Bool = true
    public var filmstripScrollTrigger: UUID = UUID()
    public var gridColumnCount: Int = 4
    var currentlyLoadedPhotoPath: String? = nil
    
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
    public var exposure: Float = 0.0      // -5.0 .. +5.0 EV
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
    public var temperature: Float = 5500.0 // 2000K .. 50000K
    public var tint: Float = 0.0          // -150 .. +150
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
    var db: LumenBoxDatabase? = nil
    var lensfun: LumenBoxLensfun? = nil
    
    public init() {
        self.lensfun = LumenBoxLensfun()
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
}
