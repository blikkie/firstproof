import SwiftUI
import AppKit

struct MainView: View {
    @Bindable var viewModel: AppViewModel
    
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
                Menu {
                    Button(action: {
                        viewModel.isExportSheetPresented = true
                    }) {
                        Label("Export Rendered Photo...", systemImage: "photo")
                    }
                    .keyboardShortcut("e", modifiers: .command)
                    .disabled(viewModel.selectedPhoto == nil)
                    
                    Button(action: {
                        viewModel.isExportPackageSheetPresented = true
                    }) {
                        let count = viewModel.selectedPhotoPaths.count
                        Label(count > 1 ? "Export as Portable Package (\(count))..." : "Export as Portable Package...", systemImage: "shippingbox.fill")
                    }
                    .keyboardShortcut("e", modifiers: [.command, .shift])
                    .disabled(viewModel.selectedPhoto == nil && viewModel.selectedPhotoPaths.isEmpty)
                } label: {
                    Label("Export", systemImage: "square.and.arrow.up")
                }
                .disabled(viewModel.selectedPhoto == nil && viewModel.selectedPhotoPaths.isEmpty)
                .help("Export Rendered Photo (Cmd+E) or Portable Package (Cmd+Shift+E)")
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
            WindowAccessor { window in
                window.setFrameAutosaveName("LumenBoxMainWindow")
            }
            .allowsHitTesting(false)
        )
    }
    
    private func setupKeyboardMonitor() {
        guard keyMonitor == nil else { return }
        keyMonitor = NSEvent.addLocalMonitorForEvents(matching: [.keyDown, .keyUp]) { event in
            // Skip hotkeys if user is editing in a text field, search box, or any text editor
            if let responder = NSApp.keyWindow?.firstResponder {
                if responder is NSTextInputClient || responder is NSTextView || responder is NSTextField || responder is NSSearchField {
                    return event
                }
            }
            
            // If an export sheet is presented, let Escape dismiss it and don't trigger canvas hotkeys
            if viewModel.isExportSheetPresented {
                if event.keyCode == 53 && event.type == .keyDown {
                    viewModel.isExportSheetPresented = false
                    return nil
                }
                return event
            }
            if viewModel.isExportPackageSheetPresented {
                if event.keyCode == 53 && event.type == .keyDown {
                    viewModel.isExportPackageSheetPresented = false
                    return nil
                }
                return event
            }
            
            // Allow system commands (Cmd+A, Cmd+E, Cmd+W, etc.) to pass through
            if event.modifierFlags.contains(.command) {
                return event
            }
            
            // Backslash (\): Hold for Original in Detail mode
            if event.keyCode == 42 || event.characters == "\\" {
                if viewModel.canvasMode == .detail {
                    if event.type == .keyDown {
                        if !event.isARepeat {
                            viewModel.isShowingOriginal = true
                        }
                    } else if event.type == .keyUp {
                        viewModel.isShowingOriginal = false
                    }
                    return nil
                }
            }
            
            if event.type == .keyUp {
                return event
            }
            
            switch event.keyCode {
            case 36: // Return / Enter: switch to Detail mode or exit Crop mode
                if viewModel.isCropToolActive {
                    if viewModel.undoHistory.scrubStartEdits != nil {
                        viewModel.endSliderScrub(actionName: "Crop Box")
                    }
                    viewModel.isCropToolActive = false
                    viewModel.saveCurrentEdits()
                    viewModel.notifyThumbnailChanged()
                    return nil
                } else if viewModel.canvasMode == .grid {
                    viewModel.enterDetailMode()
                    return nil
                }
            case 53: // Escape: exit Crop mode, return to Grid mode, or exit bulk selection mode
                if viewModel.isCropToolActive {
                    if viewModel.undoHistory.scrubStartEdits != nil {
                        viewModel.endSliderScrub(actionName: "Crop Box")
                    }
                    viewModel.isCropToolActive = false
                    viewModel.saveCurrentEdits()
                    viewModel.notifyThumbnailChanged()
                    return nil
                } else if viewModel.canvasMode == .detail {
                    viewModel.enterGridMode()
                    return nil
                } else if viewModel.isBulkSelectionMode {
                    viewModel.exitBulkSelectionMode()
                    return nil
                } else if !viewModel.selectedPhotoPaths.isEmpty {
                    viewModel.clearSelection()
                    return nil
                }
            case 49: // Spacebar: toggle bulk selection in Grid mode, inert in Detail mode
                if viewModel.canvasMode == .grid {
                    viewModel.handleSpacebarInGrid()
                    return nil
                } else {
                    return nil
                }
            case 123: // Left Arrow
                if viewModel.canvasMode == .detail {
                    viewModel.resetFilmstripFollow()
                    viewModel.selectPreviousPhoto()
                    return nil
                } else {
                    viewModel.moveGridSelection(deltaX: -1, deltaY: 0)
                    return nil
                }
            case 124: // Right Arrow
                if viewModel.canvasMode == .detail {
                    viewModel.resetFilmstripFollow()
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
                } else {
                    viewModel.resetFilmstripFollow()
                    return nil
                }
            case 126: // Up Arrow (Grid row up)
                if viewModel.canvasMode == .grid {
                    viewModel.moveGridSelection(deltaX: 0, deltaY: -1)
                    return nil
                } else {
                    viewModel.resetFilmstripFollow()
                    return nil
                }
            default:
                break
            }
            
            // Single-key professional muscle memory & culling (G, E, D, C, Z, 1-5, 0, P, X, U)
            if let chars = event.charactersIgnoringModifiers {
                switch chars {
                case "g", "G":
                    viewModel.enterGridMode()
                    return nil
                case "e", "E", "d", "D":
                    viewModel.enterDetailMode()
                    return nil
                case "z", "Z":
                    if viewModel.canvasMode == .detail {
                        viewModel.toggleZoom()
                        return nil
                    }
                case "c", "C":
                    if viewModel.canvasMode == .detail {
                        viewModel.toggleCropTool()
                        return nil
                    }
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
                    if viewModel.isCropToolActive {
                        viewModel.flipCropOrientation()
                        return nil
                    } else {
                        viewModel.setStatus("reject")
                        return nil
                    }
                case "u", "U":
                    viewModel.setStatus("none")
                    return nil
                case "[":
                    viewModel.rotateCounterClockwise()
                    return nil
                case "]":
                    viewModel.rotateClockwise()
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
