import SwiftUI
import AppKit

@main
struct LumenBoxApp: App {
    init() {
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
    
    var body: some Scene {
        WindowGroup {
            MainView()
                .navigationTitle("LumenBox - RAW Photo Manager")
        }
        .windowStyle(.titleBar)
        .windowToolbarStyle(.unified)
        .defaultSize(width: 1280, height: 800)
        .commands {
            SidebarCommands()
            CommandGroup(replacing: .newItem) {}
            CommandMenu("Photo") {
                Button("Export Photo...") {
                    NotificationCenter.default.post(name: NSNotification.Name("ExportCurrentPhoto"), object: nil)
                }
                .keyboardShortcut("e", modifiers: [.command])
            }
        }
    }
}

