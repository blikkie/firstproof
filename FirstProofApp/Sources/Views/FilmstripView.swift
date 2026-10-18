import SwiftUI
import AppKit
import FirstProofBridge

struct FilmstripView: View {
    @Environment(AppViewModel.self) private var viewModel
    
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
                    .background(FilmstripScrollListener {
                        viewModel.isFilmstripFollowingSelection = false
                    })
                }
                .onAppear {
                    viewModel.isFilmstripFollowingSelection = true
                    DispatchQueue.main.asyncAfter(deadline: .now() + 0.05) {
                        scrollToSelected(proxy: proxy, animated: false)
                    }
                }
                .onChange(of: viewModel.selectedPhoto?.filePath) { _, newPath in
                    if viewModel.isFilmstripFollowingSelection, newPath != nil {
                        scrollToSelected(proxy: proxy, animated: true)
                    }
                }
                .onChange(of: viewModel.filmstripScrollTrigger) { _, _ in
                    scrollToSelected(proxy: proxy, animated: true)
                }
                .onChange(of: viewModel.filteredPhotos.count) { _, count in
                    if count > 0 && viewModel.isFilmstripFollowingSelection {
                        DispatchQueue.main.asyncAfter(deadline: .now() + 0.05) {
                            scrollToSelected(proxy: proxy, animated: false)
                        }
                    }
                }
            }
            .frame(height: 72)
            .background(Color(NSColor.windowBackgroundColor))
        }
    }
    
    private func scrollToSelected(proxy: ScrollViewProxy, animated: Bool) {
        guard let target = viewModel.selectedPhoto?.filePath else { return }
        if animated {
            withAnimation(.easeInOut(duration: 0.15)) {
                proxy.scrollTo(target, anchor: .center)
            }
        } else {
            proxy.scrollTo(target, anchor: .center)
        }
    }
    
    private func handleCellClick(for photo: PhotoRecord) {
        viewModel.resetFilmstripFollow()
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

// MARK: - AppKit Live Scroll Listener
struct FilmstripScrollListener: NSViewRepresentable {
    var onUserScroll: () -> Void
    
    func makeNSView(context: Context) -> NSView {
        let view = ScrollListenerView()
        view.onUserScroll = onUserScroll
        return view
    }
    
    func updateNSView(_ nsView: NSView, context: Context) {
        if let v = nsView as? ScrollListenerView {
            v.onUserScroll = onUserScroll
        }
    }
    
    class ScrollListenerView: NSView {
        var onUserScroll: (() -> Void)?
        private var observer: Any?
        
        override func viewDidMoveToWindow() {
            super.viewDidMoveToWindow()
            if let obs = observer {
                NotificationCenter.default.removeObserver(obs)
                observer = nil
            }
            if let scrollView = enclosingScrollView {
                observer = NotificationCenter.default.addObserver(
                    forName: NSScrollView.didLiveScrollNotification,
                    object: scrollView,
                    queue: .main
                ) { [weak self] _ in
                    self?.onUserScroll?()
                }
            }
        }
        
        deinit {
            if let obs = observer {
                NotificationCenter.default.removeObserver(obs)
            }
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
        lhs.photo.status.caseInsensitiveCompare(rhs.photo.status) == .orderedSame &&
        lhs.isSelected == rhs.isSelected
    }
    
    var body: some View {
        ZStack(alignment: .bottomTrailing) {
            AsyncThumbnailView(filePath: photo.filePath)
                .frame(width: 56, height: 56)
                .clipShape(RoundedRectangle(cornerRadius: 4))
            
            let status = photo.status.lowercased()
            if status == "pick" {
                VStack {
                    HStack {
                        Image(systemName: "flag.fill")
                            .font(.system(size: 8, weight: .bold))
                            .foregroundColor(.green)
                            .padding(2)
                            .background(Color.black.opacity(0.7))
                            .cornerRadius(2)
                        Spacer()
                    }
                    Spacer()
                }
            } else if status == "reject" {
                VStack {
                    HStack {
                        Image(systemName: "xmark.circle.fill")
                            .font(.system(size: 8, weight: .bold))
                            .foregroundColor(.red)
                            .padding(2)
                            .background(Color.black.opacity(0.7))
                            .cornerRadius(2)
                        Spacer()
                    }
                    Spacer()
                }
            }
            
            if let rating = photo.rating, rating > 0 {
                Text("\(rating)★")
                    .font(.system(size: 8, weight: .bold))
                    .foregroundColor(.yellow)
                    .padding(2)
                    .background(Color.black.opacity(0.7))
                    .cornerRadius(2)
            } else {
                Text("–★")
                    .font(.system(size: 8, weight: .bold))
                    .foregroundColor(.clear)
                    .padding(2)
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
