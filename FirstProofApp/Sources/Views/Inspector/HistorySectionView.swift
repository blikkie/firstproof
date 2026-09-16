import SwiftUI
import FirstProofBridge

public struct HistorySectionView: View {
    @Environment(AppViewModel.self) private var viewModel
    @State private var isExpanded: Bool = true
    
    public init() {}
    
    private var activePath: String? {
        viewModel.currentlyLoadedPhotoPath ?? viewModel.selectedPhoto?.filePath
    }
    
    private var uStack: [UndoEntry] {
        viewModel.undoHistory.undoStack(for: activePath)
    }
    
    private var rStack: [UndoEntry] {
        viewModel.undoHistory.redoStack(for: activePath)
    }
    
    public var body: some View {
        DisclosureGroup(isExpanded: $isExpanded) {
            VStack(alignment: .leading, spacing: 6) {
                // Quick Undo / Redo Toolbar
                HStack(spacing: 8) {
                    Button(action: { viewModel.undo() }) {
                        Label("Undo", systemImage: "arrow.uturn.backward")
                            .font(.caption2)
                    }
                    .buttonStyle(.bordered)
                    .controlSize(.mini)
                    .disabled(!viewModel.canUndo)
                    .help(viewModel.undoActionTitle + " (⌘Z)")
                    
                    Button(action: { viewModel.redo() }) {
                        Label("Redo", systemImage: "arrow.uturn.forward")
                            .font(.caption2)
                    }
                    .buttonStyle(.bordered)
                    .controlSize(.mini)
                    .disabled(!viewModel.canRedo)
                    .help(viewModel.redoActionTitle + " (⇧⌘Z)")
                    
                    Spacer()
                    
                    if !uStack.isEmpty || !rStack.isEmpty {
                        Text("\(uStack.count) step\(uStack.count == 1 ? "" : "s")")
                            .font(.system(size: 10, design: .monospaced))
                            .foregroundColor(.secondary)
                    }
                }
                .padding(.bottom, 4)
                
                // History List
                VStack(spacing: 2) {
                    // 1. Base / Original State
                    historyRow(
                        title: "Original (As-Shot)",
                        icon: "camera.aperture",
                        isActive: uStack.isEmpty,
                        isFuture: false,
                        action: {
                            // Undo all steps for active photo
                            while viewModel.canUndo {
                                viewModel.undo()
                            }
                        }
                    )
                    
                    // 2. Applied Undo Stack (Oldest -> Newest)
                    ForEach(Array(uStack.enumerated()), id: \.element.id) { index, entry in
                        let isLatest = (index == uStack.count - 1)
                        historyRow(
                            title: entry.title,
                            icon: iconForPayload(entry.payload, title: entry.title),
                            isActive: isLatest,
                            isFuture: false,
                            action: {
                                viewModel.jumpToHistoryStep(entry)
                            }
                        )
                    }
                    
                    // 3. Undone Redo Stack (Available to Redo, shown dimmed)
                    ForEach(rStack.reversed(), id: \.id) { entry in
                        historyRow(
                            title: entry.title,
                            icon: iconForPayload(entry.payload, title: entry.title),
                            isActive: false,
                            isFuture: true,
                            action: {
                                viewModel.jumpToHistoryStep(entry)
                            }
                        )
                    }
                }
                .padding(4)
                .background(Color.secondary.opacity(0.06))
                .cornerRadius(6)
            }
            .padding(.top, 4)
        } label: {
            HStack {
                Text("HISTORY")
                    .font(.system(size: 10, weight: .bold))
                    .foregroundColor(.secondary)
                Spacer()
                if uStack.count > 0 {
                    Text("\(uStack.count)")
                        .font(.system(size: 9, weight: .semibold))
                        .padding(.horizontal, 5)
                        .padding(.vertical, 1)
                        .background(Capsule().fill(Color.accentColor.opacity(0.2)))
                        .foregroundColor(.accentColor)
                }
            }
        }
    }
    
    @ViewBuilder
    private func historyRow(
        title: String,
        icon: String,
        isActive: Bool,
        isFuture: Bool,
        action: @escaping () -> Void
    ) -> some View {
        Button(action: action) {
            HStack(spacing: 6) {
                Image(systemName: isActive ? "checkmark.circle.fill" : icon)
                    .font(.system(size: 10))
                    .foregroundColor(isActive ? .accentColor : (isFuture ? .secondary.opacity(0.4) : .secondary))
                    .frame(width: 14)
                
                Text(title)
                    .font(.system(size: 11, weight: isActive ? .semibold : .regular))
                    .foregroundColor(isActive ? .primary : (isFuture ? .secondary.opacity(0.5) : .secondary))
                    .lineLimit(1)
                
                Spacer()
            }
            .padding(.horizontal, 6)
            .padding(.vertical, 4)
            .background(
                RoundedRectangle(cornerRadius: 4)
                    .fill(isActive ? Color.accentColor.opacity(0.12) : Color.clear)
            )
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .help(isFuture ? "Redo to this state" : (isActive ? "Current state" : "Restore to this state"))
    }
    
    private func iconForPayload(_ payload: UndoActionPayload, title: String) -> String {
        switch payload {
        case .edits:
            if title.contains("Crop") || title.contains("Aspect Ratio") {
                return "crop"
            } else if title.contains("Straighten") {
                return "rotate.right"
            } else if title.contains("Rotate") || title.contains("Flip") {
                return "arrow.triangle.2.circlepath"
            } else if title.contains("Curve") {
                return "waveform.path.ecg"
            } else if title.contains("Profile") || title.contains("LUT") {
                return "camera.filters"
            } else if title.contains("Grain") {
                return "circle.dotted"
            } else if title.contains("White Balance") {
                return "eyedropper"
            }
            return "slider.horizontal.3"
        case .rating:
            return "star.fill"
        case .status:
            return "flag.fill"
        }
    }
}
