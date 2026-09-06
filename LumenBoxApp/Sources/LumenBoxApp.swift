import SwiftUI
import AppKit

final class AppDelegate: NSObject, NSApplicationDelegate {
    func applicationDidFinishLaunching(_ notification: Notification) {
        NSApplication.shared.setActivationPolicy(.regular)
        NSApplication.shared.activate(ignoringOtherApps: true)
        
        // Dynamically apply custom LumenBox lightbox icon to macOS Dock
        let iconCandidates = [
            Bundle.main.url(forResource: "AppIcon", withExtension: "icns"),
            URL(fileURLWithPath: "/Users/blikkie/src/LumenBox/LumenBoxApp/Resources/AppIcon.icns")
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
    
    // Automatically respawn main window when user Cmd+Tabs to LumenBox
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
struct LumenBoxApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) var appDelegate
    @State private var viewModel = AppViewModel()
    
    var body: some Scene {
        WindowGroup {
            MainView(viewModel: viewModel)
                .navigationTitle("LumenBox - RAW Photo Manager")
        }
        .windowStyle(.titleBar)
        .windowToolbarStyle(.unified)
        .defaultSize(width: 1280, height: 800)
        .commands {
            SidebarCommands()
            CommandGroup(replacing: .newItem) {}
            
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
                
                Divider()
                
                Menu("Rating") {
                    Button("5 Stars") { viewModel.setRating(5) }.keyboardShortcut("5", modifiers: [.command])
                    Button("4 Stars") { viewModel.setRating(4) }.keyboardShortcut("4", modifiers: [.command])
                    Button("3 Stars") { viewModel.setRating(3) }.keyboardShortcut("3", modifiers: [.command])
                    Button("2 Stars") { viewModel.setRating(2) }.keyboardShortcut("2", modifiers: [.command])
                    Button("1 Star")  { viewModel.setRating(1) }.keyboardShortcut("1", modifiers: [.command])
                    Divider()
                    Button("No Rating") { viewModel.setRating(nil) }.keyboardShortcut("0", modifiers: [.command])
                }
                
                Menu("Flag") {
                    Button("Pick")   { viewModel.setStatus("pick") }.keyboardShortcut("p", modifiers: [.command])
                    Button("Reject") { viewModel.setStatus("reject") }.keyboardShortcut("x", modifiers: [.command])
                    Button("Clear Flag") { viewModel.setStatus("none") }.keyboardShortcut("u", modifiers: [.command, .shift])
                }
                
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
            
            CommandMenu("View") {
                Button(viewModel.canvasMode == .detail ? "Show Grid View" : "Show Detail View") {
                    if viewModel.canvasMode == .detail {
                        viewModel.enterGridMode()
                    } else {
                        viewModel.enterDetailMode()
                    }
                }
                
                Divider()
                
                Menu("Inspector Workspace") {
                    Button("Light & Tone") {
                        viewModel.selectedInspectorTab = .light
                    }
                    .keyboardShortcut("1", modifiers: [.command, .option])
                    
                    Button("Color & Look") {
                        viewModel.selectedInspectorTab = .color
                    }
                    .keyboardShortcut("2", modifiers: [.command, .option])
                    
                    Button("Optics & Geometry") {
                        viewModel.selectedInspectorTab = .optics
                    }
                    .keyboardShortcut("3", modifiers: [.command, .option])
                    
                    Button("Detail & Sharpening") {
                        viewModel.selectedInspectorTab = .detail
                    }
                    .keyboardShortcut("4", modifiers: [.command, .option])
                    
                    Button("EXIF Metadata") {
                        viewModel.selectedInspectorTab = .metadata
                    }
                    .keyboardShortcut("5", modifiers: [.command, .option])
                }
            }
        }
    }
}


