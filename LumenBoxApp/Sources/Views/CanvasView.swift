import SwiftUI
import AppKit
import LumenBoxBridge

enum CanvasMode {
    case grid
    case detail
}

struct CanvasView: View {
    @Bindable var viewModel: AppViewModel
    @State private var mode: CanvasMode = .grid
    @State private var isShowingOriginal: Bool = false
    
    var body: some View {
        VStack(spacing: 0) {
            // MARK: - Toolbar Header
            HStack {
                Picker("View Mode", selection: $mode) {
                    Label("Grid", systemImage: "square.grid.3x3.fill").tag(CanvasMode.grid)
                    Label("Detail", systemImage: "photo.fill").tag(CanvasMode.detail)
                }
                .pickerStyle(.segmented)
                .frame(width: 160)
                
                Spacer()
                
                if mode == .detail && viewModel.selectedPhoto != nil {
                    Button(action: {}) {
                        Label("Hold for Original (\\)", systemImage: "eye.fill")
                    }
                    .buttonStyle(.bordered)
                    .simultaneousGesture(
                        DragGesture(minimumDistance: 0)
                            .onChanged { _ in isShowingOriginal = true }
                            .onEnded { _ in isShowingOriginal = false }
                    )
                }
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 8)
            .background(Color(NSColor.windowBackgroundColor))
            
            Divider()
            
            // MARK: - Main Canvas Content
            if viewModel.photos.isEmpty {
                VStack(spacing: 12) {
                    Image(systemName: "photo.on.rectangle.angled")
                        .font(.system(size: 48))
                        .foregroundColor(.secondary)
                    Text("No Photos Selected")
                        .font(.title2)
                        .fontWeight(.semibold)
                    Text("Open a directory from the left sidebar to start indexing RAW photos.")
                        .font(.subheadline)
                        .foregroundColor(.secondary)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
                .background(Color(NSColor.underPageBackgroundColor))
            } else if mode == .grid {
                // MARK: Grid View
                ScrollView {
                    LazyVGrid(columns: [GridItem(.adaptive(minimum: 160, maximum: 220), spacing: 12)], spacing: 12) {
                        ForEach(viewModel.filteredPhotos, id: \.filePath) { photo in
                            GridThumbnailCell(photo: photo, isSelected: viewModel.selectedPhoto?.filePath == photo.filePath) {
                                viewModel.selectPhoto(photo)
                            }
                        }
                    }
                    .padding(16)
                }
                .background(Color(NSColor.underPageBackgroundColor))
            } else {
                // MARK: Detail View
                ZStack {
                    Color(NSColor.underPageBackgroundColor)
                        .edgesIgnoringSafeArea(.all)
                    
                    if let image = viewModel.previewImage {
                        Image(nsImage: image)
                            .resizable()
                            .aspectRatio(contentMode: .fit)
                            .rotationEffect(.degrees(Double(viewModel.straightenAngle)))
                            .padding(24)
                            .shadow(color: .black.opacity(0.3), radius: 12, x: 0, y: 6)
                            .overlay(
                                isShowingOriginal ?
                                VStack {
                                    Text("ORIGINAL RAW (AS SHOT)")
                                        .font(.caption)
                                        .fontWeight(.bold)
                                        .padding(.horizontal, 12)
                                        .padding(.vertical, 6)
                                        .background(Color.black.opacity(0.8))
                                        .foregroundColor(.white)
                                        .cornerRadius(4)
                                    Spacer()
                                }
                                .padding(.top, 32) : nil
                            )
                    } else {
                        ProgressView("Loading RAW Preview...")
                    }
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            }
        }
    }
}

struct GridThumbnailCell: View {
    let photo: PhotoRecord
    let isSelected: Bool
    let action: () -> Void
    
    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            ZStack(alignment: .bottomTrailing) {
                RoundedRectangle(cornerRadius: 6)
                    .fill(Color.secondary.opacity(0.2))
                    .frame(height: 120)
                    .overlay {
                        Image(systemName: "photo")
                            .foregroundColor(.secondary)
                    }
                
                if photo.status == "pick" {
                    Image(systemName: "flag.fill")
                        .foregroundColor(.green)
                        .padding(6)
                } else if photo.status == "reject" {
                    Image(systemName: "xmark.circle.fill")
                        .foregroundColor(.red)
                        .padding(6)
                }
            }
            
            Text(URL(fileURLWithPath: photo.filePath).lastPathComponent)
                .font(.caption)
                .lineLimit(1)
            
            if let rating = photo.rating, rating > 0 {
                HStack(spacing: 1) {
                    ForEach(1...Int(rating), id: \.self) { _ in
                        Image(systemName: "star.fill")
                            .font(.system(size: 8))
                            .foregroundColor(.yellow)
                    }
                }
            }
        }
        .padding(8)
        .background(isSelected ? Color.accentColor.opacity(0.25) : Color(NSColor.controlBackgroundColor))
        .cornerRadius(8)
        .overlay(
            RoundedRectangle(cornerRadius: 8)
                .stroke(isSelected ? Color.accentColor : Color.clear, lineWidth: 2)
        )
        .onTapGesture(perform: action)
    }
}
