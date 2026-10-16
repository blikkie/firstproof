import SwiftUI
import AppKit

final class AppDelegate: NSObject, NSApplicationDelegate {
    func applicationDidFinishLaunching(_ notification: Notification) {
        NSApplication.shared.setActivationPolicy(.regular)
        NSApplication.shared.activate(ignoringOtherApps: true)
        
        // Dynamically apply custom First Proof icon to macOS Dock
        let iconCandidates = [
            Bundle.main.url(forResource: "AppIcon", withExtension: "icns"),
            ProcessInfo.processInfo.environment["FIRSTPROOF_RESOURCES_DIR"].map { URL(fileURLWithPath: $0).appendingPathComponent("AppIcon.icns") }
        ]
        for url in iconCandidates.compactMap({ $0 }) {
            if let image = NSImage(contentsOf: url) {
                NSApplication.shared.applicationIconImage = image
                break
            }
        }
    }
    
    // Automatically un-hide / recreate main window when user clicks Dock icon
    func applicationShouldHandleReopen(_ sender: NSApplication, hasVisibleWindows flag: Bool) -> Bool {
        if !flag {
            for window in sender.windows where window.canBecomeKey {
                window.makeKeyAndOrderFront(nil)
                return true
            }
        }
        return true
    }
    
    // Automatically respawn main window when user Cmd+Tabs to First Proof
    func applicationDidBecomeActive(_ notification: Notification) {
        let hasVisibleKeyWindow = NSApplication.shared.windows.contains(where: { $0.isVisible && $0.canBecomeKey })
        if !hasVisibleKeyWindow {
            for window in NSApplication.shared.windows where window.canBecomeKey {
                window.makeKeyAndOrderFront(nil)
                break
            }
        }
    }
}

@main
struct FirstProofApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) var appDelegate
    @State private var viewModel = AppViewModel()
    
    var body: some Scene {
        WindowGroup {
            MainView()
                .environment(viewModel)
                .tint(Color(white: 0.85))
                .accentColor(Color(white: 0.85))
                .navigationTitle("First Proof - Editorial RAW Developer")
                .onAppear {
                    DebugIPCService.shared.start(with: viewModel)
                }
        }
        .windowStyle(.titleBar)
        .windowToolbarStyle(.unified)
        .defaultSize(width: 1280, height: 800)
        .commands {
            SidebarCommands()
            InspectorCommands()
            CommandGroup(replacing: .newItem) {
                Button("Open Folder...") {
                    viewModel.promptOpenFolder()
                }
                .keyboardShortcut("o", modifiers: [.command])
                
                Button("Import into Main Library...") {
                    viewModel.promptImport()
                }
                .keyboardShortcut("i", modifiers: [.command, .shift])
                
                Divider()
                
                Button("Export Photo...") {
                    viewModel.isExportSheetPresented = true
                }
                .keyboardShortcut("e", modifiers: [.command])
                .disabled(viewModel.selectedPhoto == nil)
                
                Button("Export as Portable Package...") {
                    viewModel.isExportPackageSheetPresented = true
                }
                .keyboardShortcut("e", modifiers: [.command, .shift])
                .disabled(viewModel.selectedPhoto == nil && viewModel.selectedPhotoPaths.isEmpty)
            }
            
            CommandGroup(replacing: .undoRedo) {
                Button(viewModel.undoActionTitle) {
                    viewModel.undo()
                }
                .keyboardShortcut("z", modifiers: [.command])
                .disabled(!viewModel.canUndo)
                
                Button(viewModel.redoActionTitle) {
                    viewModel.redo()
                }
                .keyboardShortcut("z", modifiers: [.command, .shift])
                .disabled(!viewModel.canRedo)
            }
            
            CommandGroup(after: .undoRedo) {
                Divider()
                Button("Select All") {
                    viewModel.selectAllPhotos()
                }
                .keyboardShortcut("a", modifiers: [.command])
                
                Button("Deselect All") {
                    viewModel.clearSelection()
                }
                .keyboardShortcut("d", modifiers: [.command])
                .disabled(viewModel.selectedPhoto == nil && viewModel.selectedPhotoPaths.isEmpty)
            }
            
            CommandMenu("Photo") {
                Button("Auto Adjust Tone") {
                    viewModel.applyAutoAdjustments()
                }
                .keyboardShortcut("u", modifiers: [.command])
                .disabled(viewModel.selectedPhoto == nil || viewModel.canvasMode != .detail)
                
                Button("Reset All Edits") {
                    viewModel.resetAllSliders()
                }
                .keyboardShortcut("r", modifiers: [.command, .shift])
                .disabled(viewModel.selectedPhoto == nil || !viewModel.hasCustomEdits)
                
                Divider()
                
                Button("Rotate Clockwise") {
                    viewModel.rotateClockwise()
                }
                .keyboardShortcut("]", modifiers: [.command])
                .disabled(viewModel.selectedPhoto == nil)
                
                Button("Rotate Counter-Clockwise") {
                    viewModel.rotateCounterClockwise()
                }
                .keyboardShortcut("[", modifiers: [.command])
                .disabled(viewModel.selectedPhoto == nil)
                
                Button("Flip Horizontal") {
                    viewModel.toggleFlipHorizontal()
                }
                .disabled(viewModel.selectedPhoto == nil)
                
                Button("Flip Vertical") {
                    viewModel.toggleFlipVertical()
                }
                .disabled(viewModel.selectedPhoto == nil)
                
                Divider()
                
                Menu("Rating") {
                    Button("5 Stars")   { viewModel.setRating(5) }.keyboardShortcut("5", modifiers: [])
                    Button("4 Stars")   { viewModel.setRating(4) }.keyboardShortcut("4", modifiers: [])
                    Button("3 Stars")   { viewModel.setRating(3) }.keyboardShortcut("3", modifiers: [])
                    Button("2 Stars")   { viewModel.setRating(2) }.keyboardShortcut("2", modifiers: [])
                    Button("1 Star")    { viewModel.setRating(1) }.keyboardShortcut("1", modifiers: [])
                    Divider()
                    Button("No Rating") { viewModel.setRating(nil) }.keyboardShortcut("0", modifiers: [])
                }
                .disabled(viewModel.selectedPhoto == nil && viewModel.selectedPhotoPaths.isEmpty)
                
                Menu("Flag") {
                    Button("Pick")       { viewModel.setStatus("pick") }.keyboardShortcut("p", modifiers: [])
                    Button("Reject")     { viewModel.setStatus("reject") }.keyboardShortcut("x", modifiers: [])
                    Button("Clear Flag") { viewModel.setStatus("none") }.keyboardShortcut("u", modifiers: [])
                }
                .disabled(viewModel.selectedPhoto == nil && viewModel.selectedPhotoPaths.isEmpty)
                
                Divider()
                
                Button(viewModel.isShowingOriginal ? "Show Edited" : "Show Original") {
                    viewModel.isShowingOriginal.toggle()
                }
                .keyboardShortcut("\\", modifiers: [])
                .disabled(viewModel.selectedPhoto == nil || viewModel.canvasMode != .detail)
            }
            
            CommandMenu("View") {
                Button("Show Grid View") {
                    viewModel.enterGridMode()
                }
                .keyboardShortcut("g", modifiers: [])
                .disabled(viewModel.canvasMode == .grid)
                
                Button("Show Detail View") {
                    viewModel.enterDetailMode()
                }
                .keyboardShortcut("e", modifiers: [])
                .disabled(viewModel.canvasMode == .detail && viewModel.selectedPhoto == nil)
                
                Button("Crop & Straighten Tool") {
                    viewModel.toggleCropTool()
                }
                .keyboardShortcut("c", modifiers: [])
                .disabled(viewModel.selectedPhoto == nil || viewModel.canvasMode != .detail)
                
                Divider()
                
                Button("Zoom to Fit") {
                    viewModel.zoomToFit()
                }
                .keyboardShortcut("0", modifiers: [.command])
                .disabled(viewModel.canvasMode != .detail)
                
                Button("Zoom 100% (1:1 Loupe)") {
                    viewModel.toggleZoom()
                }
                .keyboardShortcut("z", modifiers: [])
                .disabled(viewModel.canvasMode != .detail)
                
                Button("Zoom In") {
                    viewModel.zoomIn()
                }
                .keyboardShortcut("=", modifiers: [.command])
                .disabled(viewModel.canvasMode != .detail)
                
                Button("Zoom Out") {
                    viewModel.zoomOut()
                }
                .keyboardShortcut("-", modifiers: [.command])
                .disabled(viewModel.canvasMode != .detail)
                
                Divider()
                
                Button(viewModel.isInspectorPresented ? "Hide Inspector" : "Show Inspector") {
                    viewModel.toggleInspector()
                }
                .keyboardShortcut("i", modifiers: [.command, .option])
                
                Menu("Inspector Workspace") {
                    Button("Develop") {
                        viewModel.selectedInspectorTab = .develop
                    }
                    .keyboardShortcut("1", modifiers: [.command, .option])
                    
                    Button("Optics & Geometry") {
                        viewModel.selectedInspectorTab = .optics
                    }
                    .keyboardShortcut("2", modifiers: [.command, .option])
                    
                    Button("Emulsion & Darkroom Print") {
                        viewModel.selectedInspectorTab = .emulsion
                    }
                    .keyboardShortcut("3", modifiers: [.command, .option])
                    
                    Button("Detail & Sharpening") {
                        viewModel.selectedInspectorTab = .detail
                    }
                    .keyboardShortcut("4", modifiers: [.command, .option])
                }
            }
        }
        
        Settings {
            SettingsView()
                .environment(viewModel)
        }
    }
}


