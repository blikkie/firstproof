import SwiftUI
import AppKit

struct MainView: View {
    @State private var viewModel = AppViewModel()
    
    @State private var keyMonitor: Any? = nil
    
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
            
            // MARK: - Bottom Filmstrip Carousel Pane (Only in Detail View)
            if viewModel.canvasMode == .detail {
                FilmstripView(viewModel: viewModel)
                    .transition(.move(edge: .bottom).combined(with: .opacity))
            }
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
        .onAppear {
            setupKeyboardMonitor()
        }
        .onDisappear {
            if let monitor = keyMonitor {
                NSEvent.removeMonitor(monitor)
                keyMonitor = nil
            }
        }
        .background(
            Group {
                Button("") { viewModel.selectAllPhotos() }.keyboardShortcut("a", modifiers: .command)
                WindowAccessor { window in
                    window.setFrameAutosaveName("LumenBoxMainWindow")
                }
            }
            .opacity(0)
            .allowsHitTesting(false)
        )
    }
    
    private func setupKeyboardMonitor() {
        guard keyMonitor == nil else { return }
        keyMonitor = NSEvent.addLocalMonitorForEvents(matching: .keyDown) { event in
            // Skip hotkeys if user is editing in a text field or text area
            if let responder = NSApp.keyWindow?.firstResponder,
               responder is NSTextView || responder is NSTextField {
                return event
            }
            
            // Allow system commands (Cmd+A, Cmd+E, Cmd+W, etc.) to pass through
            if event.modifierFlags.contains(.command) {
                return event
            }
            
            switch event.keyCode {
            case 36: // Return / Enter: switch to Detail mode
                if viewModel.canvasMode == .grid {
                    viewModel.enterDetailMode()
                    return nil
                }
            case 53: // Escape: return to Grid mode or clear selection
                if viewModel.canvasMode == .detail {
                    viewModel.enterGridMode()
                    return nil
                } else if !viewModel.selectedPhotoPaths.isEmpty {
                    viewModel.clearSelection()
                    return nil
                }
            case 49: // Spacebar: toggle selection in Grid mode
                if viewModel.canvasMode == .grid {
                    viewModel.toggleSpaceSelection()
                    return nil
                }
            case 123: // Left Arrow
                if viewModel.canvasMode == .detail {
                    viewModel.selectPreviousPhoto()
                    return nil
                } else {
                    viewModel.moveGridSelection(deltaX: -1, deltaY: 0)
                    return nil
                }
            case 124: // Right Arrow
                if viewModel.canvasMode == .detail {
                    viewModel.selectNextPhoto()
                    return nil
                } else {
                    viewModel.moveGridSelection(deltaX: 1, deltaY: 0)
                    return nil
                }
            case 125: // Down Arrow (Grid row down)
                if viewModel.canvasMode == .grid {
                    viewModel.moveGridSelection(deltaX: 0, deltaY: 1)
                    return nil
                }
            case 126: // Up Arrow (Grid row up)
                if viewModel.canvasMode == .grid {
                    viewModel.moveGridSelection(deltaX: 0, deltaY: -1)
                    return nil
                }
            default:
                break
            }
            
            // Single-key culling hotkeys (1-5, 0, P, X, U)
            if let chars = event.charactersIgnoringModifiers {
                switch chars {
                case "1":
                    viewModel.setRating(1)
                    return nil
                case "2":
                    viewModel.setRating(2)
                    return nil
                case "3":
                    viewModel.setRating(3)
                    return nil
                case "4":
                    viewModel.setRating(4)
                    return nil
                case "5":
                    viewModel.setRating(5)
                    return nil
                case "0":
                    viewModel.setRating(nil)
                    return nil
                case "p", "P":
                    viewModel.setStatus("pick")
                    return nil
                case "x", "X":
                    viewModel.setStatus("reject")
                    return nil
                case "u", "U":
                    viewModel.setStatus("none")
                    return nil
                default:
                    break
                }
            }
            
            return event
        }
    }
}

struct WindowAccessor: NSViewRepresentable {
    let callback: (NSWindow) -> Void
    
    func makeNSView(context: Context) -> NSView {
        let view = NSView()
        DispatchQueue.main.async {
            if let window = view.window {
                callback(window)
            }
        }
        return view
    }
    
    func updateNSView(_ nsView: NSView, context: Context) {}
}
