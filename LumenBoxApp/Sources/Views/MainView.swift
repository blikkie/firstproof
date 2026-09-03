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
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                Button(action: {
                    viewModel.isExportPackageSheetPresented = true
                }) {
                    Label("Package", systemImage: "shippingbox.fill")
                }
                .keyboardShortcut("e", modifiers: [.command, .shift])
                .disabled(viewModel.selectedPhoto == nil && viewModel.selectedPhotoPaths.isEmpty)
                .help("Export Selection as Portable Library (Cmd+Shift+E)")
            }
            
            ToolbarItem(placement: .primaryAction) {
                Button(action: {
                    viewModel.isExportSheetPresented = true
                }) {
                    Label("Export", systemImage: "square.and.arrow.up")
                }
                .keyboardShortcut("e", modifiers: .command)
                .disabled(viewModel.selectedPhoto == nil)
                .help("Export Rendered Photo (Cmd+E)")
            }
        }
        .sheet(isPresented: $viewModel.isExportSheetPresented) {
            ExportSheetView(viewModel: viewModel)
        }
        .sheet(isPresented: $viewModel.isExportPackageSheetPresented) {
            ExportPackageSheetView(viewModel: viewModel)
        }
        .background(
            Group {
                Button("") { viewModel.setRating(1) }.keyboardShortcut("1", modifiers: [])
                Button("") { viewModel.setRating(2) }.keyboardShortcut("2", modifiers: [])
                Button("") { viewModel.setRating(3) }.keyboardShortcut("3", modifiers: [])
                Button("") { viewModel.setRating(4) }.keyboardShortcut("4", modifiers: [])
                Button("") { viewModel.setRating(5) }.keyboardShortcut("5", modifiers: [])
                Button("") { viewModel.setRating(nil) }.keyboardShortcut("0", modifiers: [])
                Button("") { viewModel.setStatus("pick") }.keyboardShortcut("p", modifiers: [])
                Button("") { viewModel.setStatus("reject") }.keyboardShortcut("x", modifiers: [])
                Button("") { viewModel.setStatus("none") }.keyboardShortcut("u", modifiers: [])
                Button("") {
                    viewModel.canvasMode = (viewModel.canvasMode == .grid) ? .detail : .grid
                }.keyboardShortcut(.space, modifiers: [])
                Button("") {
                    viewModel.selectAllPhotos()
                }.keyboardShortcut("a", modifiers: .command)
            }
            .opacity(0)
            .allowsHitTesting(false)
        )
    }
}
