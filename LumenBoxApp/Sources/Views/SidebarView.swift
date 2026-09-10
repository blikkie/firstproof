import SwiftUI
import AppKit
import LumenBoxBridge

struct SidebarView: View {
    @Bindable var viewModel: AppViewModel
    
    var body: some View {
        if viewModel.canvasMode == .detail {
            MetadataSidebarView(viewModel: viewModel)
        } else {
            LibrarySidebarView(viewModel: viewModel)
        }
    }
}
