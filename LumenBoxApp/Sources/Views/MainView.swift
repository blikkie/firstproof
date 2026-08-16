import SwiftUI
import AppKit

struct MainView: View {
    @State private var viewModel = AppViewModel()
    
    var body: some View {
        VStack(spacing: 0) {
            // MARK: - Upper 3-Pane Split (Sidebar, Canvas, Inspector)
            HSplitView {
                SidebarView(viewModel: viewModel)
                
                CanvasView(viewModel: viewModel)
                    .frame(minWidth: 400, idealWidth: 600, maxWidth: .infinity)
                
                InspectorView(viewModel: viewModel)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            
            // MARK: - Bottom Filmstrip Carousel Pane
            FilmstripView(viewModel: viewModel)
        }
        .frame(minWidth: 900, minHeight: 600)
    }
}
