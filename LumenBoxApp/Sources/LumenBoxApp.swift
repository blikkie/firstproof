import SwiftUI
import AppKit

@main
struct LumenBoxApp: App {
    init() {
        NSApplication.shared.setActivationPolicy(.regular)
        NSApplication.shared.activate(ignoringOtherApps: true)
    }
    
    var body: some Scene {
        WindowGroup {
            MainView()
                .navigationTitle("LumenBox - RAW Photo Manager")
        }
        .windowStyle(.titleBar)
        .windowToolbarStyle(.unified)
        .defaultSize(width: 1280, height: 800)
    }
}
