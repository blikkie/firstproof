import SwiftUI
import AppKit

@MainActor
public struct SettingsView: View {
    @Environment(AppViewModel.self) private var viewModel
    
    @AppStorage("autoDetectMemoryCards") private var autoDetectMemoryCards: Bool = true
    @AppStorage("defaultDeduplicateRawOverJpeg") private var defaultDeduplicateRawOverJpeg: Bool = true
    @AppStorage("defaultSkipDuplicates") private var defaultSkipDuplicates: Bool = true
    @AppStorage("defaultConsolidateSidecars") private var defaultConsolidateSidecars: Bool = true
    @AppStorage("defaultAutoTca") private var defaultAutoTca: Bool = true
    @AppStorage("defaultAutoDistortion") private var defaultAutoDistortion: Bool = false
    @AppStorage("customMainLibraryPath") private var customMainLibraryPath: String = ""
    
    @State private var pendingNewLibraryPath: String? = nil
    @State private var showingChangeConfirmation: Bool = false
    @State private var showingResetConfirmation: Bool = false
    
    public init() {}
    
    public var body: some View {
        VStack(alignment: .leading, spacing: 18) {
            // MARK: - Library Location
            VStack(alignment: .leading, spacing: 8) {
                Text("Library Location")
                    .font(.headline)
                
                HStack {
                    Text(currentLibraryDisplay)
                        .font(.system(.body, design: .monospaced))
                        .lineLimit(1)
                        .truncationMode(.middle)
                        .padding(6)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .background(Color(NSColor.controlBackgroundColor))
                        .cornerRadius(4)
                    
                    Button("Choose...") {
                        chooseCustomLibraryFolder()
                    }
                    
                    if !customMainLibraryPath.isEmpty {
                        Button("Reset") {
                            showingResetConfirmation = true
                        }
                        .help("Reset to ~/Pictures/First Proof Library")
                    }
                }
                
                Text("New imports will be saved here. Photos in previous locations remain intact on disk and can be opened anytime via File > Open Session.")
                    .font(.caption)
                    .foregroundColor(.secondary)
                    .lineLimit(nil)
                    .fixedSize(horizontal: false, vertical: true)
            }
            
            Divider()
            
            // MARK: - Importing Options
            VStack(alignment: .leading, spacing: 10) {
                Text("Importing")
                    .font(.headline)
                
                Toggle("Prompt to import when a camera card is connected", isOn: $autoDetectMemoryCards)
                Toggle("Prefer RAW over JPEG when both exist for a photo", isOn: $defaultDeduplicateRawOverJpeg)
                Toggle("Skip duplicate photos already present in library", isOn: $defaultSkipDuplicates)
                Toggle("Consolidate third-party sidecars into First Proof XMP", isOn: $defaultConsolidateSidecars)
            }
            
            Divider()
            
            // MARK: - Optical Corrections
            VStack(alignment: .leading, spacing: 10) {
                Text("Optical Corrections")
                    .font(.headline)
                
                Toggle("Auto-correct chromatic aberration when lens profile is available", isOn: $defaultAutoTca)
                Toggle("Auto-correct geometric distortion when lens profile is available", isOn: $defaultAutoDistortion)
                
                Text("Default states applied to newly opened photos without existing edits. Individual photos can toggle corrections at any time in the Optics panel.")
                    .font(.caption)
                    .foregroundColor(.secondary)
                    .lineLimit(nil)
                    .fixedSize(horizontal: false, vertical: true)
            }
        }
        .padding(20)
        .frame(width: 500)
        .fixedSize(horizontal: true, vertical: true)
        .alert("Change Library Location?", isPresented: $showingChangeConfirmation) {
            Button("Change Location") {
                if let path = pendingNewLibraryPath {
                    applyNewLibraryPath(path)
                }
            }
            Button("Cancel", role: .cancel) {
                pendingNewLibraryPath = nil
            }
        } message: {
            if let path = pendingNewLibraryPath {
                Text("New imports will be saved to \"\(URL(fileURLWithPath: path).lastPathComponent)\". Photos in your current library remain in their existing folder and can still be opened via File > Open Session.")
            }
        }
        .alert("Reset Library Location?", isPresented: $showingResetConfirmation) {
            Button("Reset Location") {
                resetLibraryPath()
            }
            Button("Cancel", role: .cancel) {}
        } message: {
            Text("Reset library path to ~/Pictures/First Proof Library? Photos in your current folder remain intact on disk.")
        }
        .background(
            Button("") {
                closeSettingsWindow()
            }
            .keyboardShortcut(.cancelAction)
            .opacity(0)
            .frame(width: 0, height: 0)
        )
    }
    
    private func closeSettingsWindow() {
        if let keyWindow = NSApp.keyWindow, keyWindow.attachedSheet == nil {
            keyWindow.close()
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
            guard url.path != currentLibraryDisplay else { return }
            pendingNewLibraryPath = url.path
            showingChangeConfirmation = true
        }
    }
    
    private func applyNewLibraryPath(_ path: String) {
        customMainLibraryPath = path
        if case .mainLibrary = viewModel.currentScope {
            viewModel.openDirectory(at: path, isMain: true)
        }
        pendingNewLibraryPath = nil
    }
    
    private func resetLibraryPath() {
        customMainLibraryPath = ""
        if case .mainLibrary = viewModel.currentScope {
            viewModel.openDirectory(at: viewModel.mainLibraryPath, isMain: true)
        }
    }
}
