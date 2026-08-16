import SwiftUI
import AppKit

@main
struct LumenBoxApp: App {
    var body: some Scene {
        WindowGroup {
            MainView()
                .navigationTitle("LumenBox - RAW Photo Manager")
        }
        .windowStyle(.titleBar)
        .windowToolbarStyle(.unified)
    }
}
