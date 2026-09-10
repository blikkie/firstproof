import SwiftUI
import LumenBoxBridge

public struct MetadataWorkspaceView: View {
    @Bindable public var viewModel: AppViewModel
    public let photo: PhotoRecord
    
    public init(viewModel: AppViewModel, photo: PhotoRecord) {
        self.viewModel = viewModel
        self.photo = photo
    }
    
    public var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            RatingControlsView(viewModel: viewModel)
            FlagControlsView(viewModel: viewModel)
            Divider()
            metadataRows
        }
        .padding(.top, 4)
    }
    
    @ViewBuilder
    private var metadataRows: some View {
        VStack(alignment: .leading, spacing: 8) {
            if let title = photo.title, !title.isEmpty {
                MetadataRow(label: "Title", value: title)
            }
            if let creator = photo.creator, !creator.isEmpty {
                MetadataRow(label: "Creator", value: creator)
            }
            if let keywords = photo.keywords, !keywords.isEmpty {
                MetadataRow(label: "Keywords", value: keywords)
            }
            if let make = photo.cameraMake, let model = photo.cameraModel, !make.isEmpty || !model.isEmpty {
                MetadataRow(label: "Camera", value: "\(make) \(model)".trimmingCharacters(in: .whitespaces))
            }
            if let lens = photo.lensModel, !lens.isEmpty {
                MetadataRow(label: "Lens", value: lens)
            }
            if let iso = photo.iso {
                MetadataRow(label: "ISO", value: "\(iso)")
            }
            if let aperture = photo.aperture {
                MetadataRow(label: "Aperture", value: String(format: "f/%.1f", aperture))
            }
            if let shutter = photo.shutterSpeed, !shutter.isEmpty {
                MetadataRow(label: "Shutter", value: shutter)
            }
            if let focal = photo.focalLength {
                MetadataRow(label: "Focal Length", value: String(format: "%.0f mm", focal))
            }
            if let taken = photo.takenTime, taken > 0 {
                MetadataRow(label: "Date", value: formatDate(taken))
            }
            if let lat = photo.latitude, let lon = photo.longitude {
                MetadataRow(label: "GPS", value: String(format: "%.4f, %.4f", lat, lon))
            }
            
            let hasTakeout = viewModel.takeoutMatches.contains(where: { $0.imagePath == photo.filePath })
            HStack {
                Text("Google Takeout:")
                    .font(.caption)
                    .foregroundColor(.secondary)
                Spacer()
                if hasTakeout {
                    Label("JSON Matched", systemImage: "checkmark.circle.fill")
                        .font(.caption)
                        .foregroundColor(.green)
                } else {
                    Text("None")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }
            .padding(.top, 2)
        }
    }
    
    private func formatDate(_ timestamp: Int64) -> String {
        let date = Date(timeIntervalSince1970: TimeInterval(timestamp))
        let formatter = DateFormatter()
        formatter.dateStyle = .medium
        formatter.timeStyle = .short
        return formatter.string(from: date)
    }
}
