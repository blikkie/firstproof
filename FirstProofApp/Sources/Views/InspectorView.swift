import SwiftUI
import AppKit
import FirstProofBridge

@MainActor
struct InspectorView: View {
    @Environment(AppViewModel.self) private var viewModel
    
    private var isBulkSelection: Bool {
        viewModel.selectedPhotoPaths.count > 1
    }
    
    private var selectedCount: Int {
        viewModel.selectedPhotoPaths.count > 0 ? viewModel.selectedPhotoPaths.count : (viewModel.selectedPhoto != nil ? 1 : 0)
    }
    
    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 14) {
                if isBulkSelection {
                    // MARK: - Multi-Selection Bulk Culling Panel
                    BulkCullingPanel()
                } else if let photo = viewModel.selectedPhoto {
                    // MARK: - Single Photo (Detail View & Grid Single-Selection)
                    singlePhotoPanel(for: photo)
                } else {
                    // No photo selected
                    emptySelectionPanel
                }
                
                Spacer()
            }
            .padding(14)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Color(NSColor.controlBackgroundColor))
    }
    
    // MARK: - Empty State
    private var emptySelectionPanel: some View {
        VStack(spacing: 12) {
            Image(systemName: "photo.on.rectangle.angled")
                .font(.system(size: 36))
                .foregroundColor(.secondary.opacity(0.6))
                .padding(.top, 40)
            Text("No Photo Selected")
                .font(.subheadline)
                .foregroundColor(.secondary)
        }
        .frame(maxWidth: .infinity)
    }
    
    // MARK: - Single Photo Panel (Context-Aware: Metadata in Grid, Workspace Tabs in Detail)
    @ViewBuilder
    private func singlePhotoPanel(for photo: PhotoRecord) -> some View {
        @Bindable var viewModel = viewModel
        VStack(alignment: .leading, spacing: 12) {
            // 1. Permanent Filename & Header Card (Always Visible)
            PermanentHeaderCard(photo: photo)
            
            if viewModel.canvasMode == .detail {
                // Sensitometric EV Zone Histogram & Sensor Clipping HUD (Story 3.25)
                HistogramView()
                
                // In Detail Mode: Full Development Workspace Tabs & Reset All Button
                HStack(spacing: 8) {
                    Picker("Workspace Tab", selection: $viewModel.selectedInspectorTab) {
                        ForEach(InspectorTab.allCases) { tab in
                            Image(systemName: tab.iconName)
                                .help(tab.rawValue)
                                .tag(tab)
                        }
                    }
                    .pickerStyle(.segmented)
                    
                    Button(action: {
                        viewModel.resetAllSliders()
                    }) {
                        Image(systemName: "arrow.counterclockwise")
                            .font(.system(size: 11, weight: .semibold))
                    }
                    .buttonStyle(.bordered)
                    .controlSize(.small)
                    .disabled(!viewModel.hasCustomEdits)
                    .help("Reset All Edits to As-Shot (Cmd+Shift+R)")
                }
                
                Divider()
                
                // Active Workspace Tab View
                switch viewModel.selectedInspectorTab {
                case .develop:
                    DevelopWorkspaceView()
                case .optics:
                    OpticsGeometryWorkspaceView()
                case .emulsion:
                    EmulsionWorkspaceView()
                case .detail:
                    DetailWorkspaceView()
                }
                
                Divider()
                
                HistorySectionView()
            } else {
                // In Grid Mode: Metadata editing only (Rating, Flags, EXIF, Takeout)
                MetadataWorkspaceView(photo: photo)
                
                Divider()
                
                Button(action: {
                    viewModel.enterDetailMode()
                }) {
                    Label("Develop in Detail View (Return)", systemImage: "slider.horizontal.3")
                        .font(.caption)
                        .frame(maxWidth: .infinity)
                }
                .buttonStyle(.bordered)
                .controlSize(.regular)
                .padding(.top, 2)
            }
        }
    }
}
