import SwiftUI
import AppKit
import FirstProofBridge

public struct PermanentHeaderCard: View {
    public let photo: PhotoRecord
    
    public init(photo: PhotoRecord) {
        self.photo = photo
    }
    
    public var body: some View {
        let filename = URL(fileURLWithPath: photo.filePath).lastPathComponent
        let ext = URL(fileURLWithPath: photo.filePath).pathExtension.uppercased()
        
        return VStack(alignment: .leading, spacing: 8) {
            VStack(alignment: .leading, spacing: 4) {
                Text(filename)
                    .font(.headline)
                    .fontWeight(.bold)
                    .lineLimit(1)
                    .truncationMode(.middle)
                    .allowsTightening(true)
                    .help(photo.filePath)
                    .contextMenu {
                        Button("Copy Filename") {
                            NSPasteboard.general.clearContents()
                            NSPasteboard.general.setString(filename, forType: .string)
                        }
                        Button("Copy Full Path") {
                            NSPasteboard.general.clearContents()
                            NSPasteboard.general.setString(photo.filePath, forType: .string)
                        }
                        Button("Reveal in Finder") {
                            NSWorkspace.shared.selectFile(photo.filePath, inFileViewerRootedAtPath: "")
                        }
                    }
                
                HStack(spacing: 6) {
                    Text(ext)
                        .font(.system(size: 9, weight: .bold))
                        .padding(.horizontal, 5)
                        .padding(.vertical, 2)
                        .background(Color.accentColor.opacity(0.18))
                        .foregroundColor(.accentColor)
                        .cornerRadius(3)
                    
                    if let focal = formatFocalLength(photo.focalLength) {
                        Text(focal)
                            .font(.caption2)
                            .foregroundColor(.secondary)
                            .monospacedDigit()
                    }
                }
            }
            
            // Persistent Heads-Up EXIF Block
            VStack(spacing: 5) {
                // Wide Camera Badge with generous width and 2-line wraparound
                ExifBadgeView(
                    label: "CAMERA",
                    value: formatCamera(photo),
                    lineLimit: 2,
                    alignment: .leading
                )
                
                // 3-Column Exposure Triangle Badges
                HStack(spacing: 5) {
                    ExifBadgeView(label: "ISO", value: formatISO(photo.iso))
                    ExifBadgeView(label: "APERTURE", value: formatAperture(photo.aperture))
                    ExifBadgeView(label: "SHUTTER", value: formatShutterSpeed(photo.shutterSpeed))
                }
            }
        }
        .padding(10)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color.secondary.opacity(0.08))
        .cornerRadius(8)
        .overlay(
            RoundedRectangle(cornerRadius: 8)
                .stroke(Color.secondary.opacity(0.15), lineWidth: 1)
        )
    }
    
    private func formatCamera(_ photo: PhotoRecord) -> String {
        let make = photo.cameraMake?.trimmingCharacters(in: .whitespaces) ?? ""
        let model = photo.cameraModel?.trimmingCharacters(in: .whitespaces) ?? ""
        if !make.isEmpty && !model.isEmpty {
            if model.localizedCaseInsensitiveContains(make) {
                return model
            }
            return "\(make) \(model)"
        }
        if !model.isEmpty {
            return model
        }
        if !make.isEmpty {
            return make
        }
        return "—"
    }
    
    private func formatFocalLength(_ val: Float?) -> String? {
        guard let val = val, val > 0, !val.isNaN else { return nil }
        let rounded = Int(val.rounded())
        return "\(rounded)mm"
    }
    
    private func formatISO(_ iso: UInt32?) -> String {
        guard let iso = iso, iso > 0 else { return "—" }
        return "\(iso)"
    }
    
    private static let standardFStops: [Float] = [
        1.0, 1.1, 1.2, 1.4, 1.6, 1.8, 2.0, 2.2, 2.5, 2.8, 3.2, 3.5, 4.0, 4.5, 5.0,
        5.6, 6.3, 7.1, 8.0, 9.0, 10, 11, 13, 14, 16, 18, 20, 22, 25, 29, 32, 45, 64
    ]
    
    private func formatAperture(_ val: Float?) -> String {
        guard let val = val, val > 0, !val.isNaN else { return "—" }
        for fstop in Self.standardFStops {
            if abs(val - fstop) / fstop < 0.03 {
                return fstop == Float(Int(fstop)) ? "f/\(Int(fstop))" : String(format: "f/%.1f", fstop)
            }
        }
        let rounded = (val * 10).rounded() / 10
        return rounded == Float(Int(rounded)) ? "f/\(Int(rounded))" : String(format: "f/%.1f", rounded)
    }
    
    private func formatShutterSpeed(_ shutter: String?) -> String {
        guard let shutter = shutter?.trimmingCharacters(in: .whitespaces), !shutter.isEmpty else {
            return "—"
        }
        if shutter.hasSuffix("s") {
            return shutter
        }
        if shutter.contains("/") {
            return "\(shutter)s"
        }
        if let val = Double(shutter) {
            if val >= 0.4 {
                let rounded = (val * 10).rounded() / 10
                return rounded == Double(Int(rounded)) ? "\(Int(rounded))s" : String(format: "%.1fs", rounded)
            }
            let denom = Int((1.0 / val).rounded())
            return "1/\(denom)s"
        }
        return "\(shutter)s"
    }
}

public struct ExifBadgeView: View {
    public let label: String
    public let value: String
    public var lineLimit: Int?
    public var alignment: HorizontalAlignment
    
    public init(
        label: String,
        value: String,
        lineLimit: Int? = 1,
        alignment: HorizontalAlignment = .center
    ) {
        self.label = label
        self.value = value
        self.lineLimit = lineLimit
        self.alignment = alignment
    }
    
    public var body: some View {
        VStack(alignment: alignment, spacing: 2) {
            Text(label)
                .font(.system(size: 8, weight: .bold))
                .foregroundColor(.secondary.opacity(0.8))
            Text(value)
                .font(.system(size: 10, weight: .semibold))
                .monospacedDigit()
                .foregroundColor(.primary)
                .lineLimit(lineLimit)
                .multilineTextAlignment(alignment == .leading ? .leading : .center)
                .fixedSize(horizontal: false, vertical: true)
        }
        .frame(maxWidth: .infinity, alignment: alignment == .leading ? .leading : .center)
        .padding(.vertical, 4)
        .padding(.horizontal, alignment == .leading ? 8 : 4)
        .background(Color(NSColor.controlBackgroundColor))
        .cornerRadius(4)
        .overlay(
            RoundedRectangle(cornerRadius: 4)
                .stroke(Color.secondary.opacity(0.18), lineWidth: 0.5)
        )
    }
}

public struct RatingControlsView: View {
    @Environment(AppViewModel.self) private var viewModel
    
    public init() {}
    
    public var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("Star Rating:")
                .font(.caption)
                .foregroundColor(.secondary)
            
            HStack(spacing: 6) {
                ForEach(1...5, id: \.self) { star in
                    let isSelected: Bool = {
                        if let r = viewModel.selectedPhoto?.rating {
                            return r >= Int32(star)
                        }
                        return false
                    }()
                    Button(action: {
                        viewModel.setRating(Int32(star))
                    }) {
                        Image(systemName: isSelected ? "star.fill" : "star")
                            .font(.system(size: 13, weight: isSelected ? .bold : .regular))
                            .foregroundColor(isSelected ? .yellow : .secondary.opacity(0.6))
                            .padding(4)
                            .background(isSelected ? Color.yellow.opacity(0.15) : Color.clear)
                            .clipShape(RoundedRectangle(cornerRadius: 4))
                    }
                    .buttonStyle(.plain)
                    .help("Rate \(star) Star\(star > 1 ? "s" : "") (Key \(star))")
                }
                
                Button(action: {
                    viewModel.setRating(nil)
                }) {
                    Image(systemName: "xmark")
                        .font(.system(size: 11, weight: .medium))
                        .foregroundColor(.secondary)
                        .padding(4)
                }
                .buttonStyle(.plain)
                .help("Clear Rating (Key 0)")
            }
        }
    }
}

public struct FlagControlsView: View {
    @Environment(AppViewModel.self) private var viewModel
    
    public init() {}
    
    public var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("Flag:")
                .font(.caption)
                .foregroundColor(.secondary)
            
            HStack(spacing: 8) {
                let currentStatus = viewModel.selectedPhoto?.status.lowercased() ?? "none"
                let isPick = currentStatus == "pick"
                let isReject = currentStatus == "reject"
                
                Button(action: { viewModel.setStatus("pick") }) {
                    HStack(spacing: 5) {
                        Image(systemName: "flag.fill")
                            .font(.system(size: 11, weight: .semibold))
                        Text("Pick (P)")
                            .font(.caption)
                            .fontWeight(isPick ? .bold : .medium)
                    }
                    .foregroundColor(isPick ? .white : .green)
                    .padding(.horizontal, 10)
                    .padding(.vertical, 5)
                    .background(isPick ? Color.green : Color.green.opacity(0.12))
                    .cornerRadius(6)
                    .overlay(
                        RoundedRectangle(cornerRadius: 6)
                            .stroke(isPick ? Color.green : Color.green.opacity(0.3), lineWidth: 1)
                    )
                }
                .buttonStyle(.plain)
                .help("Mark as Pick (P)")
                
                Button(action: { viewModel.setStatus("reject") }) {
                    HStack(spacing: 5) {
                        Image(systemName: "xmark.circle.fill")
                            .font(.system(size: 11, weight: .semibold))
                        Text("Reject (X)")
                            .font(.caption)
                            .fontWeight(isReject ? .bold : .medium)
                    }
                    .foregroundColor(isReject ? .white : .red)
                    .padding(.horizontal, 10)
                    .padding(.vertical, 5)
                    .background(isReject ? Color.red : Color.red.opacity(0.12))
                    .cornerRadius(6)
                    .overlay(
                        RoundedRectangle(cornerRadius: 6)
                            .stroke(isReject ? Color.red : Color.red.opacity(0.3), lineWidth: 1)
                    )
                }
                .buttonStyle(.plain)
                .help("Mark as Reject (X)")
                
                Button(action: { viewModel.setStatus("none") }) {
                    Text("Clear (U)")
                        .font(.caption)
                        .foregroundColor(currentStatus == "none" ? .secondary : .primary)
                        .padding(.horizontal, 8)
                        .padding(.vertical, 5)
                        .background(Color.secondary.opacity(0.08))
                        .cornerRadius(6)
                }
                .buttonStyle(.plain)
                .help("Unflag (U)")
            }
        }
    }
}

public struct BulkCullingPanel: View {
    @Environment(AppViewModel.self) private var viewModel
    
    public init() {}
    
    private var selectedCount: Int {
        viewModel.selectedPhotoPaths.count > 0 ? viewModel.selectedPhotoPaths.count : (viewModel.selectedPhoto != nil ? 1 : 0)
    }
    
    public var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            Text("BULK SELECTION")
                .font(.caption)
                .fontWeight(.bold)
                .foregroundColor(.secondary)
                .padding(.top, 4)
            
            HStack(spacing: 8) {
                Image(systemName: "square.grid.3x3.fill")
                    .foregroundColor(.accentColor)
                Text("\(selectedCount) Photos Selected")
                    .font(.subheadline)
                    .fontWeight(.semibold)
            }
            .padding(8)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(Color.secondary.opacity(0.1))
            .cornerRadius(6)
            
            RatingControlsView()
            FlagControlsView()
            
            Divider()
            
            VStack(alignment: .leading, spacing: 8) {
                Label("Bulk Actions", systemImage: "info.circle.fill")
                    .font(.subheadline)
                    .foregroundColor(.accentColor)
                
                Text("Ratings and flags apply to all \(selectedCount) selected photos across SQLite and XMP sidecars. Use the Export button in the toolbar to package.")
                    .font(.caption2)
                    .foregroundColor(.secondary)
            }
            .padding(10)
            .background(Color.accentColor.opacity(0.08))
            .cornerRadius(8)
        }
    }
}
