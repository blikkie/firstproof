import SwiftUI
import AppKit

public struct SettingsView: View {
    var viewModel: AppViewModel
    
    @AppStorage("autoDetectMemoryCards") private var autoDetectMemoryCards: Bool = true
    @AppStorage("defaultImportMoveFiles") private var defaultImportMoveFiles: Bool = false
    @AppStorage("defaultDeduplicateRawOverJpeg") private var defaultDeduplicateRawOverJpeg: Bool = true
    @AppStorage("defaultSkipDuplicates") private var defaultSkipDuplicates: Bool = true
    @AppStorage("defaultConsolidateSidecars") private var defaultConsolidateSidecars: Bool = true
    @AppStorage("customMainLibraryPath") private var customMainLibraryPath: String = ""
    
    @State private var selectedTab: SettingsTab = .general
    
    public init(viewModel: AppViewModel) {
        self.viewModel = viewModel
    }
    
    public enum SettingsTab: String, CaseIterable, Identifiable {
        case general = "General"
        case ingestion = "Import & Cards"
        case darkroom = "Darkroom & Display"
        
        public var id: String { rawValue }
        
        public var systemImage: String {
            switch self {
            case .general: return "gearshape.fill"
            case .ingestion: return "square.and.arrow.down.fill"
            case .darkroom: return "slider.horizontal.3"
            }
        }
    }
    
    public var body: some View {
        TabView(selection: $selectedTab) {
            generalTab
                .tabItem {
                    Label("General", systemImage: "gearshape.fill")
                }
                .tag(SettingsTab.general)
            
            ingestionTab
                .tabItem {
                    Label("Import & Cards", systemImage: "square.and.arrow.down.fill")
                }
                .tag(SettingsTab.ingestion)
            
            darkroomTab
                .tabItem {
                    Label("Darkroom", systemImage: "slider.horizontal.3")
                }
                .tag(SettingsTab.darkroom)
        }
        .frame(width: 540, height: 380)
        .padding(20)
    }
    
    // MARK: - General Tab
    private var generalTab: some View {
        Form {
            Section(header: Text("Primary Library Location").font(.headline)) {
                VStack(alignment: .leading, spacing: 8) {
                    Text("First Proof stores its master negative index and non-destructive catalog database in this primary library.")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    
                    HStack {
                        Text(currentLibraryDisplay)
                            .font(.system(.body, design: .monospaced))
                            .lineLimit(1)
                            .truncationMode(.middle)
                            .padding(6)
                            .background(Color(NSColor.controlBackgroundColor))
                            .cornerRadius(4)
                        
                        Button("Choose...") {
                            chooseCustomLibraryFolder()
                        }
                        
                        if !customMainLibraryPath.isEmpty {
                            Button("Reset") {
                                customMainLibraryPath = ""
                            }
                            .help("Reset to ~/Pictures/First Proof Library")
                        }
                    }
                }
                .padding(.vertical, 4)
            }
            
            Divider()
            
            Section(header: Text("Philosophy & Color Science").font(.headline)) {
                VStack(alignment: .leading, spacing: 6) {
                    Text("\"Pure capture. Honest color. Zero digital plastic.\"")
                        .font(.caption)
                        .italic()
                        .foregroundColor(.secondary)
                    Text("First Proof processes camera Bayer sensor pixels directly in Linear Rec.2020 wide-gamut color space with analog chemical film roll-off via AgX.")
                        .font(.caption2)
                        .foregroundColor(.secondary)
                }
                .padding(.vertical, 4)
            }
        }
    }
    
    // MARK: - Ingestion & Memory Cards Tab
    private var ingestionTab: some View {
        Form {
            Section(header: Text("Memory Card Detection").font(.headline)) {
                Toggle("Prompt to import when a camera card is connected", isOn: $autoDetectMemoryCards)
                Text("Detects removable media and volumes containing a DCIM folder (Canon, Sony, Nikon, Leica, Fuji).")
                    .font(.caption2)
                    .foregroundColor(.secondary)
            }
            
            Divider()
            
            Section(header: Text("Deduplication & Precedence").font(.headline)) {
                Toggle("Prefer RAW over JPEG when both exist for the same photo", isOn: $defaultDeduplicateRawOverJpeg)
                Toggle("Skip duplicate photos already present in destination library", isOn: $defaultSkipDuplicates)
                Toggle("Consolidate third-party sidecars into unified First Proof XMP", isOn: $defaultConsolidateSidecars)
                    .help("Merges Adobe CRS, Darktable, and Google Takeout metadata into a single clean standard XMP sidecar.")
            }
            
            Divider()
            
            Section(header: Text("Safety & Negatives").font(.headline)) {
                HStack(spacing: 8) {
                    Image(systemName: "checkmark.shield.fill")
                        .foregroundColor(.green)
                    Text("Safe Ingest: Portable sessions and camera cards are copied by default, preserving source folders intact as your field backup. Moving negatives requires explicit user opt-in and bit-level cryptographic checksum verification.")
                        .font(.caption2)
                        .foregroundColor(.secondary)
                }
            }
        }
    }
    
    // MARK: - Darkroom & Display Tab
    private var darkroomTab: some View {
        Form {
            Section(header: Text("Display Calibration").font(.headline)) {
                VStack(alignment: .leading, spacing: 8) {
                    Text("Viewport Dynamic Range Default")
                        .font(.subheadline)
                    Text("Print & Web First: The viewport canvas always initializes to standard dynamic range (SDR) so tonal adjustments remain calibrated for universal viewing and physical darkroom prints. EDR highlights can be inspected on demand.")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }
        }
    }
    
    private var currentLibraryDisplay: String {
        if !customMainLibraryPath.isEmpty {
            return customMainLibraryPath
        }
        return viewModel.mainLibraryPath
    }
    
    private func chooseCustomLibraryFolder() {
        let panel = NSOpenPanel()
        panel.canChooseFiles = false
        panel.canChooseDirectories = true
        panel.allowsMultipleSelection = false
        panel.prompt = "Select Library Folder"
        if panel.runModal() == .OK, let url = panel.url {
            customMainLibraryPath = url.path
            viewModel.openDirectory(at: url.path, isMain: true)
        }
    }
}
