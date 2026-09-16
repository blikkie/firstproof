import SwiftUI
import AppKit
import FirstProofBridge

struct SidebarView: View {
    @Environment(AppViewModel.self) private var viewModel
    
    var body: some View {
        if viewModel.canvasMode == .detail {
            MetadataSidebarView()
        } else {
            LibrarySidebarView()
        }
    }
}
