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


