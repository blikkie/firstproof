import SwiftUI
import AppKit
import LumenBoxBridge

struct FilmstripView: View {
    @Bindable var viewModel: AppViewModel
    
    var body: some View {
        VStack(spacing: 0) {
            Divider()
            
            // MARK: Filmstrip Thumbnail Carousel
            ScrollViewReader { proxy in
                ScrollView(.horizontal, showsIndicators: false) {
                    LazyHStack(spacing: 8) {
                        ForEach(viewModel.filteredPhotos, id: \.filePath) { photo in
                            let isSelected = viewModel.selectedPhotoPaths.contains(photo.filePath) || viewModel.selectedPhoto?.filePath == photo.filePath
                            FilmstripThumbnailCell(
                                photo: photo,
                                isSelected: isSelected
                            ) {
                                handleCellClick(for: photo)
                            }
                            .id(photo.filePath)
                        }
                    }
                    .padding(.horizontal, 12)
                    .padding(.vertical, 8)
                }
                .onChange(of: viewModel.selectedPhoto?.filePath) { _, newPath in
                    if let target = newPath {
                        proxy.scrollTo(target, anchor: .center)
                    }
                }
            }
            .frame(height: 72)
            .background(Color(NSColor.windowBackgroundColor))
        }
    }
    
    private func handleCellClick(for photo: PhotoRecord) {
        let flags = NSEvent.modifierFlags
        if flags.contains(.command) {
            viewModel.togglePhotoSelection(photo)
        } else if flags.contains(.shift) {
            viewModel.selectPhotoRange(to: photo)
        } else {
            viewModel.selectPhoto(photo)
        }
    }
}

struct FilmstripThumbnailCell: View, Equatable {
    let photo: PhotoRecord
    let isSelected: Bool
    let action: () -> Void
    
    static func == (lhs: FilmstripThumbnailCell, rhs: FilmstripThumbnailCell) -> Bool {
        lhs.photo.filePath == rhs.photo.filePath &&
        lhs.photo.rating == rhs.photo.rating &&
        lhs.photo.status == rhs.photo.status &&
        lhs.isSelected == rhs.isSelected
    }
    
    var body: some View {
        ZStack(alignment: .bottomTrailing) {
            AsyncThumbnailView(filePath: photo.filePath)
                .frame(width: 56, height: 56)
                .clipShape(RoundedRectangle(cornerRadius: 4))
            
            if let rating = photo.rating, rating > 0 {
                Text("\(rating)★")
                    .font(.system(size: 8, weight: .bold))
                    .foregroundColor(.yellow)
                    .padding(2)
                    .background(Color.black.opacity(0.7))
                    .cornerRadius(2)
            }
        }
        .overlay(
            RoundedRectangle(cornerRadius: 4)
                .stroke(isSelected ? Color.accentColor : Color.clear, lineWidth: 2)
        )
        .contentShape(Rectangle())
        .onTapGesture(perform: action)
    }
}
