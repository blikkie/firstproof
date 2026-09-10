import SwiftUI
import AppKit
import LumenBoxBridge

public struct MetadataSidebarView: View {
    @Bindable public var viewModel: AppViewModel
    @State private var isContentExpanded: Bool = true
    @State private var isExposureExpanded: Bool = true
    @State private var isLocationExpanded: Bool = true
    @State private var isFileSpecsExpanded: Bool = false
    
    public init(viewModel: AppViewModel) {
        self.viewModel = viewModel
    }
    
    public var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 12) {
                // MARK: - 1. Top Navigation & Active Session Header
                HStack(spacing: 8) {
                    Button(action: {
                        viewModel.enterGridMode()
                    }) {
                        HStack(spacing: 4) {
                            Image(systemName: "chevron.left")
                                .font(.system(size: 10, weight: .bold))
                            Text("Grid (Esc)")
                                .font(.caption)
                                .fontWeight(.medium)
                        }
                        .foregroundColor(.accentColor)
                        .padding(.horizontal, 8)
                        .padding(.vertical, 4)
                        .background(Color.accentColor.opacity(0.12))
                        .cornerRadius(6)
                    }
                    .buttonStyle(.plain)
                    .help("Return to Grid View (Esc)")
                    
                    Spacer()
                    
                    // Active scope label
                    HStack(spacing: 4) {
                        Image(systemName: "folder")
                            .font(.caption2)
                            .foregroundColor(.secondary)
                        Text(scopeTitle)
                            .font(.caption2)
                            .fontWeight(.medium)
                            .foregroundColor(.secondary)
                            .lineLimit(1)
                    }
                }
                .padding(.horizontal, 12)
                .padding(.top, 10)
                
                Divider()
                    .padding(.horizontal, 8)
                
                // MARK: - 2. Active Photo Summary Card
                if let photo = viewModel.selectedPhoto {
                    VStack(alignment: .leading, spacing: 6) {
                        HStack(alignment: .center, spacing: 8) {
                            Text(photo.fileName)
                                .font(.headline)
                                .fontWeight(.semibold)
                                .lineLimit(1)
                                .truncationMode(.middle)
                            
                            Spacer()
                            
                            if photo.isRaw {
                                Text("RAW")
                                    .font(.system(size: 9, weight: .bold))
                                    .padding(.horizontal, 5)
                                    .padding(.vertical, 2)
                                    .background(Color.orange.opacity(0.2))
                                    .foregroundColor(.orange)
                                    .cornerRadius(4)
                            } else {
                                let ext = URL(fileURLWithPath: photo.filePath).pathExtension.uppercased()
                                Text(ext)
                                    .font(.system(size: 9, weight: .bold))
                                    .padding(.horizontal, 5)
                                    .padding(.vertical, 2)
                                    .background(Color.secondary.opacity(0.15))
                                    .foregroundColor(.secondary)
                                    .cornerRadius(4)
                            }
                        }
                        
                        let mb = Double(photo.fileSize) / (1024.0 * 1024.0)
                        Text(String(format: "%.1f MB", mb))
                            .font(.caption2)
                            .foregroundColor(.secondary)
                    }
                    .padding(.horizontal, 12)
                    
                    // MARK: - 3. Tier 1: Triage & Culling Panel (Always Visible)
                    VStack(alignment: .leading, spacing: 8) {
                        HStack {
                            Text("TRIAGE & RATING")
                                .font(.caption2)
                                .fontWeight(.bold)
                                .foregroundColor(.secondary)
                            Spacer()
                            if let rating = photo.rating, rating > 0 {
                                Text("\(rating) ★")
                                    .font(.caption2)
                                    .fontWeight(.bold)
                                    .foregroundColor(.yellow)
                            }
                        }
                        
                        // Star Rating Row
                        HStack(spacing: 4) {
                            ForEach(1...5, id: \.self) { star in
                                let isSelected = (photo.rating ?? 0) >= Int32(star)
                                Button(action: {
                                    viewModel.setRating(Int32(star))
                                }) {
                                    Image(systemName: isSelected ? "star.fill" : "star")
                                        .font(.system(size: 15, weight: isSelected ? .bold : .regular))
                                        .foregroundColor(isSelected ? .yellow : .secondary.opacity(0.5))
                                        .frame(maxWidth: .infinity)
                                        .padding(.vertical, 4)
                                        .background(isSelected ? Color.yellow.opacity(0.15) : Color.secondary.opacity(0.06))
                                        .cornerRadius(4)
                                }
                                .buttonStyle(.plain)
                                .help("Rate \(star) Star (Key \(star))")
                            }
                            
                            Button(action: {
                                viewModel.setRating(nil)
                            }) {
                                Image(systemName: "xmark")
                                    .font(.system(size: 11, weight: .medium))
                                    .foregroundColor(.secondary)
                                    .padding(5)
                                    .background(Color.secondary.opacity(0.06))
                                    .cornerRadius(4)
                            }
                            .buttonStyle(.plain)
                            .help("Clear Star Rating (Key 0)")
                        }
                        
                        // Pick / Reject Solid Badges
                        let currentStatus = photo.status.lowercased()
                        let isPick = currentStatus == "pick"
                        let isReject = currentStatus == "reject"
                        
                        HStack(spacing: 8) {
                            // Pick Button
                            Button(action: {
                                viewModel.setStatus("pick")
                            }) {
                                HStack(spacing: 5) {
                                    Image(systemName: isPick ? "flag.fill" : "flag")
                                        .font(.system(size: 12, weight: .bold))
                                    Text("PICK (P)")
                                        .font(.caption)
                                        .fontWeight(.bold)
                                }
                                .frame(maxWidth: .infinity)
                                .padding(.vertical, 7)
                                .foregroundColor(isPick ? .white : .green)
                                .background(isPick ? Color.green : Color.green.opacity(0.12))
                                .cornerRadius(6)
                                .overlay(
                                    RoundedRectangle(cornerRadius: 6)
                                        .stroke(isPick ? Color.green : Color.green.opacity(0.35), lineWidth: 1)
                                )
                            }
                            .buttonStyle(.plain)
                            .help("Flag as Pick (P)")
                            
                            // Reject Button
                            Button(action: {
                                viewModel.setStatus("reject")
                            }) {
                                HStack(spacing: 5) {
                                    Image(systemName: isReject ? "xmark.circle.fill" : "xmark.circle")
                                        .font(.system(size: 12, weight: .bold))
                                    Text("REJECT (X)")
                                        .font(.caption)
                                        .fontWeight(.bold)
                                }
                                .frame(maxWidth: .infinity)
                                .padding(.vertical, 7)
                                .foregroundColor(isReject ? .white : .red)
                                .background(isReject ? Color.red : Color.red.opacity(0.12))
                                .cornerRadius(6)
                                .overlay(
                                    RoundedRectangle(cornerRadius: 6)
                                        .stroke(isReject ? Color.red : Color.red.opacity(0.35), lineWidth: 1)
                                )
                            }
                            .buttonStyle(.plain)
                            .help("Flag as Reject (X)")
                            
                            // Clear Unflag Button
                            Button(action: {
                                viewModel.setStatus("none")
                            }) {
                                Text("CLEAR (U)")
                                    .font(.system(size: 10, weight: .semibold))
                                    .foregroundColor(currentStatus == "none" ? .secondary : .primary)
                                    .padding(.horizontal, 6)
                                    .padding(.vertical, 7)
                                    .background(Color.secondary.opacity(0.08))
                                    .cornerRadius(6)
                            }
                            .buttonStyle(.plain)
                            .help("Clear Flag (U)")
                        }
                    }
                    .padding(10)
                    .background(Color.secondary.opacity(0.06))
                    .cornerRadius(8)
                    .padding(.horizontal, 10)
                    
                    Divider()
                        .padding(.horizontal, 8)
                    
                    // MARK: - 4. Tier 2: Content & IPTC (Editable)
                    DisclosureGroup(isExpanded: $isContentExpanded) {
                        VStack(alignment: .leading, spacing: 10) {
                            VStack(alignment: .leading, spacing: 3) {
                                Text("Title")
                                    .font(.caption2)
                                    .foregroundColor(.secondary)
                                TextField("Title / Headline", text: $viewModel.metadataTitle)
                                    .textFieldStyle(.roundedBorder)
                                    .font(.caption)
                                    .onChange(of: viewModel.metadataTitle) { _, _ in
                                        viewModel.scheduleMetadataSave()
                                    }
                            }
                            
                            VStack(alignment: .leading, spacing: 3) {
                                Text("Caption / Description")
                                    .font(.caption2)
                                    .foregroundColor(.secondary)
                                TextField("Description / Notes", text: $viewModel.metadataDescription, axis: .vertical)
                                    .textFieldStyle(.roundedBorder)
                                    .lineLimit(2...4)
                                    .font(.caption)
                                    .onChange(of: viewModel.metadataDescription) { _, _ in
                                        viewModel.scheduleMetadataSave()
                                    }
                            }
                            
                            VStack(alignment: .leading, spacing: 3) {
                                Text("Keywords / Tags")
                                    .font(.caption2)
                                    .foregroundColor(.secondary)
                                TextField("Nature, Landscape, 2026...", text: $viewModel.metadataKeywords)
                                    .textFieldStyle(.roundedBorder)
                                    .font(.caption)
                                    .onChange(of: viewModel.metadataKeywords) { _, _ in
                                        viewModel.scheduleMetadataSave()
                                    }
                            }
                            
                            VStack(alignment: .leading, spacing: 3) {
                                Text("Creator / Photographer")
                                    .font(.caption2)
                                    .foregroundColor(.secondary)
                                TextField("Photographer name", text: $viewModel.metadataCreator)
                                    .textFieldStyle(.roundedBorder)
                                    .font(.caption)
                                    .onChange(of: viewModel.metadataCreator) { _, _ in
                                        viewModel.scheduleMetadataSave()
                                    }
                            }
                            
                            VStack(alignment: .leading, spacing: 3) {
                                Text("Copyright")
                                    .font(.caption2)
                                    .foregroundColor(.secondary)
                                TextField("© 2026 All Rights Reserved", text: $viewModel.metadataCopyright)
                                    .textFieldStyle(.roundedBorder)
                                    .font(.caption)
                                    .onChange(of: viewModel.metadataCopyright) { _, _ in
                                        viewModel.scheduleMetadataSave()
                                    }
                            }
                        }
                        .padding(.top, 6)
                    } label: {
                        Label("Content & IPTC", systemImage: "doc.text.fill")
                            .font(.caption)
                            .fontWeight(.bold)
                            .foregroundColor(.primary)
                    }
                    .padding(.horizontal, 12)
                    
                    Divider()
                        .padding(.horizontal, 8)
                    
                    // MARK: - 5. Tier 3: Camera & EXIF Exposure Triangle
                    DisclosureGroup(isExpanded: $isExposureExpanded) {
                        VStack(alignment: .leading, spacing: 8) {
                            // 4-Tile Exposure Card
                            HStack(spacing: 4) {
                                exposureTile(title: "SHUTTER", value: photo.shutterSpeed ?? "--")
                                exposureTile(title: "APERTURE", value: photo.aperture != nil ? String(format: "f/%.1f", photo.aperture!) : "--")
                                exposureTile(title: "ISO", value: photo.iso != nil ? "\(photo.iso!)" : "--")
                                exposureTile(title: "FOCAL", value: photo.focalLength != nil ? "\(Int(photo.focalLength!))mm" : "--")
                            }
                            
                            if let make = photo.cameraMake, let model = photo.cameraModel {
                                metadataInfoRow(label: "Camera", value: "\(make) \(model)")
                            } else if let model = photo.cameraModel {
                                metadataInfoRow(label: "Camera", value: model)
                            }
                            
                            if let lens = photo.lensModel {
                                metadataInfoRow(label: "Lens", value: lens)
                            }
                            
                            if let taken = photo.takenTime {
                                let date = Date(timeIntervalSince1970: TimeInterval(taken))
                                metadataInfoRow(label: "Date Taken", value: formattedDate(date))
                            }
                        }
                        .padding(.top, 6)
                    } label: {
                        Label("Exposure & Camera", systemImage: "camera.fill")
                            .font(.caption)
                            .fontWeight(.bold)
                            .foregroundColor(.primary)
                    }
                    .padding(.horizontal, 12)
                    
                    Divider()
                        .padding(.horizontal, 8)
                    
                    // MARK: - 6. Tier 4: Location & GPS
                    DisclosureGroup(isExpanded: $isLocationExpanded) {
                        VStack(alignment: .leading, spacing: 8) {
                            if let lat = photo.latitude, let lon = photo.longitude {
                                HStack {
                                    VStack(alignment: .leading, spacing: 2) {
                                        Text(String(format: "%.4f° N, %.4f° W", lat, abs(lon)))
                                            .font(.caption)
                                            .fontWeight(.semibold)
                                        Text("GPS Coordinates")
                                            .font(.caption2)
                                            .foregroundColor(.secondary)
                                    }
                                    Spacer()
                                    Button(action: {
                                        openAppleMaps(lat: lat, lon: lon)
                                    }) {
                                        HStack(spacing: 4) {
                                            Image(systemName: "map.fill")
                                                .font(.caption2)
                                            Text("Apple Maps")
                                                .font(.caption2)
                                                .fontWeight(.medium)
                                        }
                                        .foregroundColor(.accentColor)
                                        .padding(.horizontal, 8)
                                        .padding(.vertical, 4)
                                        .background(Color.accentColor.opacity(0.12))
                                        .cornerRadius(4)
                                    }
                                    .buttonStyle(.plain)
                                }
                            } else {
                                Text("No GPS coordinates in EXIF")
                                    .font(.caption2)
                                    .foregroundColor(.secondary)
                            }
                        }
                        .padding(.top, 6)
                    } label: {
                        Label("Location & GPS", systemImage: "location.fill")
                            .font(.caption)
                            .fontWeight(.bold)
                            .foregroundColor(.primary)
                    }
                    .padding(.horizontal, 12)
                    
                    Divider()
                        .padding(.horizontal, 8)
                    
                    // MARK: - 7. Tier 5: File & Sidecars
                    DisclosureGroup(isExpanded: $isFileSpecsExpanded) {
                        VStack(alignment: .leading, spacing: 6) {
                            metadataInfoRow(label: "XMP Sidecar", value: photo.hasXmp ? "Synchronized (.xmp)" : "Embedded / Pending")
                            if photo.hasTakeout {
                                metadataInfoRow(label: "Google Takeout", value: "Matched (.json)")
                            }
                            metadataInfoRow(label: "File Path", value: photo.filePath)
                        }
                        .padding(.top, 6)
                    } label: {
                        Label("File & Sidecars", systemImage: "internaldrive.fill")
                            .font(.caption)
                            .fontWeight(.bold)
                            .foregroundColor(.primary)
                    }
                    .padding(.horizontal, 12)
                } else {
                    VStack(spacing: 8) {
                        Image(systemName: "photo.badge.exclamationmark")
                            .font(.largeTitle)
                            .foregroundColor(.secondary)
                        Text("No photo selected")
                            .font(.subheadline)
                            .foregroundColor(.secondary)
                    }
                    .frame(maxWidth: .infinity, minHeight: 200)
                    .padding()
                }
                
                Spacer(minLength: 20)
            }
        }
        .frame(minWidth: 230, idealWidth: 260, maxWidth: 300)
        .background(Color(NSColor.controlBackgroundColor))
    }
    
    // MARK: - Subviews & Helpers
    private var scopeTitle: String {
        switch viewModel.currentScope {
        case .mainLibrary:
            return "Main Library"
        case .session(let folderPath):
            return URL(fileURLWithPath: folderPath).lastPathComponent
        }
    }
    
    private func exposureTile(title: String, value: String) -> some View {
        VStack(spacing: 2) {
            Text(title)
                .font(.system(size: 8, weight: .bold))
                .foregroundColor(.secondary)
            Text(value)
                .font(.system(size: 11, weight: .bold))
                .lineLimit(1)
        }
        .frame(maxWidth: .infinity)
        .padding(.vertical, 6)
        .background(Color.secondary.opacity(0.08))
        .cornerRadius(6)
    }
    
    private func metadataInfoRow(label: String, value: String) -> some View {
        VStack(alignment: .leading, spacing: 1) {
            Text(label)
                .font(.caption2)
                .foregroundColor(.secondary)
            Text(value)
                .font(.caption)
                .foregroundColor(.primary)
                .lineLimit(2)
        }
    }
    
    private func formattedDate(_ date: Date) -> String {
        let formatter = DateFormatter()
        formatter.dateStyle = .medium
        formatter.timeStyle = .short
        return formatter.string(from: date)
    }
    
    private func openAppleMaps(lat: Double, lon: Double) {
        if let url = URL(string: "https://maps.apple.com/?q=\(lat),\(lon)") {
            NSWorkspace.shared.open(url)
        }
    }
}
