import SwiftUI
import AppKit

@MainActor
struct MainView: View {
    @Environment(AppViewModel.self) private var viewModel
    
    @State private var keyMonitor: Any? = nil
    
    var body: some View {
        @Bindable var viewModel = viewModel
        NavigationSplitView(columnVisibility: $viewModel.columnVisibility) {
            SidebarView()
                .navigationSplitViewColumnWidth(min: 200, ideal: 220, max: 280)
        } detail: {
            VStack(spacing: 0) {
                // Camera Card Detection Banner
                if viewModel.isCameraCardBannerPresented, let card = viewModel.detectedCameraCard {
                    HStack(spacing: 12) {
                        Image(systemName: "externaldrive.fill.badge.plus")
                            .foregroundColor(.accentColor)
                            .font(.title3)
                        
                        VStack(alignment: .leading, spacing: 2) {
                            Text("Camera Card Detected: \(card.name)")
                                .font(.subheadline)
                                .fontWeight(.semibold)
                            Text("Import original negatives into your Primary Library with sidecar consolidation and duplicate suppression.")
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }
                        
                        Spacer()
                        
                        Button("Import Photos...") {
                            viewModel.isCameraCardBannerPresented = false
                            viewModel.promptImport(from: card.path)
                        }
                        .buttonStyle(.borderedProminent)
                        .controlSize(.small)
                        
                        Button("Open as Session") {
                            viewModel.isCameraCardBannerPresented = false
                            viewModel.addFolder(at: card.path)
                        }
                        .buttonStyle(.bordered)
                        .controlSize(.small)
                        
                        Button(action: {
                            viewModel.isCameraCardBannerPresented = false
                        }) {
                            Image(systemName: "xmark")
                                .foregroundColor(.secondary)
                                .font(.caption)
                        }
                        .buttonStyle(.plain)
                        .padding(.leading, 4)
                    }
                    .padding(.horizontal, 16)
                    .padding(.vertical, 8)
                    .background(Color.accentColor.opacity(0.12))
                    .overlay(Divider(), alignment: .bottom)
                    .transition(.move(edge: .top).combined(with: .opacity))
                }
                
                CanvasView()
                    .frame(minWidth: 400, idealWidth: 600, maxWidth: .infinity)
                
                // MARK: - Bottom Filmstrip Carousel Pane (Only in Detail View)
                if viewModel.canvasMode == .detail {
                    FilmstripView()
                        .transition(.move(edge: .bottom).combined(with: .opacity))
                }
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .inspector(isPresented: $viewModel.isInspectorPresented) {
                InspectorView()
                    .inspectorColumnWidth(min: 240, ideal: 270, max: 320)
            }
        }
        .frame(minWidth: 900, minHeight: 600)
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                Button(action: {
                    viewModel.promptImport()
                }) {
                    Label("Import", systemImage: "square.and.arrow.down")
                }
                .help("Import photos into Main Library (Cmd+Shift+I)")
            }
            
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
            
            ToolbarItem(placement: .primaryAction) {
                Button(action: {
                    viewModel.toggleInspector()
                }) {
                    Label("Toggle Inspector", systemImage: "sidebar.trailing")
                }
                .help("Toggle Inspector (Cmd+Option+I)")
            }
        }
        .sheet(isPresented: $viewModel.isImportSheetPresented) {
            ImportSheetView()
        }
        .sheet(isPresented: $viewModel.isExportSheetPresented) {
            ExportSheetView()
        }
        .sheet(isPresented: $viewModel.isExportPackageSheetPresented) {
            ExportPackageSheetView()
        }
        .sheet(isPresented: $viewModel.isCullDuplicatesSheetPresented) {
            CullDuplicatesSheetView()
        }
        .sheet(isPresented: $viewModel.isDngConversionSheetPresented) {
            DngConversionSheetView()
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
                window.setFrameAutosaveName("FirstProofMainWindow")
            }
            .allowsHitTesting(false)
        )
    }
    
    private func setupKeyboardMonitor() {
        guard keyMonitor == nil else { return }
        keyMonitor = NSEvent.addLocalMonitorForEvents(matching: [.keyDown, .keyUp]) { event in
            // Skip hotkeys if user is editing in a text field, search box, or any text editor
            if KeyboardShielding.shouldShieldKeyboardEvent(responder: NSApp.keyWindow?.firstResponder) {
                return event
            }
            
            // If the Settings window is active, Escape dismisses it and canvas hotkeys are shielded
            if let keyWindow = NSApp.keyWindow, isSettingsWindow(keyWindow) {
                if event.keyCode == 53 && event.type == .keyDown {
                    if keyWindow.attachedSheet == nil {
                        keyWindow.close()
                        return nil
                    }
                }
                return event
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
            if viewModel.isCullDuplicatesSheetPresented {
                if event.keyCode == 53 && event.type == .keyDown {
                    viewModel.isCullDuplicatesSheetPresented = false
                    return nil
                }
                return event
            }
            if viewModel.isImportSheetPresented {
                if event.keyCode == 53 && event.type == .keyDown {
                    viewModel.isImportSheetPresented = false
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
            case 36: // Return / Enter: switch to Detail mode or commit Crop mode
                if viewModel.isCropToolActive {
                    viewModel.commitCrop()
                    return nil
                } else if viewModel.canvasMode == .grid {
                    viewModel.enterDetailMode()
                    return nil
                }
            case 53: // Escape: rollback slider scrub, cancel Crop mode, return to Grid mode, or exit bulk selection mode
                if viewModel.hasActiveSliderScrub {
                    viewModel.cancelSliderScrub()
                    return nil
                } else if viewModel.isCropToolActive {
                    viewModel.cancelCrop()
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
                case "h", "H":
                    if viewModel.canvasMode == .detail {
                        viewModel.toggleHistogram()
                        return nil
                    }
                case "j", "J":
                    if viewModel.canvasMode == .detail {
                        viewModel.toggleClippingOverlay()
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
    
    private func isSettingsWindow(_ window: NSWindow) -> Bool {
        let title = window.title.lowercased()
        let identifier = window.identifier?.rawValue.lowercased() ?? ""
        return title.contains("settings") || title.contains("preferences") || identifier.contains("settings")
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
