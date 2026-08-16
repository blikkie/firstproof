import SwiftUI
import AppKit

struct FilmstripView: View {
    @Bindable var viewModel: AppViewModel
    
    var body: some View {
        VStack(spacing: 0) {
            Divider()
            
            HStack(spacing: 12) {
                // MARK: Keybinding Quick Rating Toolbar
                HStack(spacing: 8) {
                    Text("Rating:")
                        .font(.caption2)
                        .foregroundColor(.secondary)
                    
                    ForEach(1...5, id: \.self) { star in
                        Button("\(star)") {
                            viewModel.setRating(Int32(star))
                        }
                        .buttonStyle(.bordered)
                        .controlSize(.small)
                    }
                    
                    Divider().frame(height: 16)
                    
                    Button("Pick (P)") {
                        viewModel.setStatus("pick")
                    }
                    .buttonStyle(.bordered)
                    .tint(.green)
                    .controlSize(.small)
                    
                    Button("Reject (X)") {
                        viewModel.setStatus("reject")
                    }
                    .buttonStyle(.bordered)
                    .tint(.red)
                    .controlSize(.small)
                }
                .padding(.leading, 12)
                
                Spacer()
                
                // MARK: Filmstrip Thumbnail Carousel
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 8) {
                        ForEach(viewModel.filteredPhotos, id: \.filePath) { photo in
                            FilmstripThumbnailCell(
                                photo: photo,
                                isSelected: viewModel.selectedPhoto?.filePath == photo.filePath
                            ) {
                                viewModel.selectPhoto(photo)
                            }
                        }
                    }
                    .padding(.horizontal, 12)
                    .padding(.vertical, 8)
                }
            }
            .frame(height: 72)
            .background(Color(NSColor.windowBackgroundColor))
        }
    }
}

struct FilmstripThumbnailCell: View {
    let photo: PhotoRecord
    let isSelected: Bool
    let action: () -> Void
    
    var body: some View {
        ZStack(alignment: .bottomTrailing) {
            RoundedRectangle(cornerRadius: 4)
                .fill(Color.secondary.opacity(0.2))
                .frame(width: 56, height: 56)
                .overlay(
                    Image(systemName: "photo")
                        .font(.caption)
                        .foregroundColor(.secondary)
                )
            
            if let rating = photo.rating, rating > 0 {
                Text("\(rating)★")
                    .font(.system(size: 9, weight: .bold))
                    .foregroundColor(.yellow)
                    .padding(2)
                    .background(Color.black.opacity(0.7))
                    .cornerRadius(2)
                    .padding(2)
            }
        }
        .overlay(
            RoundedRectangle(cornerRadius: 4)
                .stroke(isSelected ? Color.accentColor : Color.clear, lineWidth: 2)
        )
        .onTapGesture(perform: action)
    }
}
