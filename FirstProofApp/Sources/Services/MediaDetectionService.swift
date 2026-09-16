import Foundation
import AppKit

public struct CameraCardInfo: Equatable, Sendable {
    public let name: String
    public let url: URL
    public let path: String
    public let isCameraCard: Bool
    public let hasDcim: Bool
    
    public init(name: String, url: URL, isCameraCard: Bool, hasDcim: Bool) {
        self.name = name
        self.url = url
        self.path = url.path
        self.isCameraCard = isCameraCard
        self.hasDcim = hasDcim
    }
}

public final class MediaDetectionService: @unchecked Sendable {
    public static let shared = MediaDetectionService()
    
    public var onCameraCardDetected: ((CameraCardInfo) -> Void)?
    public var onVolumeUnmounted: ((String) -> Void)?
    
    private var isStarted = false
    
    private init() {}
    
    public func start() {
        guard !isStarted else { return }
        isStarted = true
        
        let center = NSWorkspace.shared.notificationCenter
        center.addObserver(
            self,
            selector: #selector(handleVolumeDidMount(_:)),
            name: NSWorkspace.didMountNotification,
            object: nil
        )
        center.addObserver(
            self,
            selector: #selector(handleVolumeDidUnmount(_:)),
            name: NSWorkspace.didUnmountNotification,
            object: nil
        )
        
        // Initial scan of currently mounted volumes for camera cards
        scanMountedVolumes()
    }
    
    public func scanMountedVolumes() {
        let keys: [URLResourceKey] = [.volumeIsRemovableKey, .volumeIsEjectableKey, .volumeIsInternalKey]
        let volumes = FileManager.default.mountedVolumeURLs(includingResourceValuesForKeys: keys, options: [.skipHiddenVolumes]) ?? []
        
        for url in volumes {
            inspectVolume(url: url)
        }
    }
    
    @objc private func handleVolumeDidMount(_ notification: Notification) {
        guard let url = notification.userInfo?[NSWorkspace.volumeURLUserInfoKey] as? URL else { return }
        inspectVolume(url: url)
    }
    
    @objc private func handleVolumeDidUnmount(_ notification: Notification) {
        guard let url = notification.userInfo?[NSWorkspace.volumeURLUserInfoKey] as? URL else { return }
        DispatchQueue.main.async { [weak self] in
            self?.onVolumeUnmounted?(url.path)
        }
    }
    
    private func inspectVolume(url: URL) {
        let path = url.path
        // Skip root Mac volume
        if path == "/" || path == "/Volumes/Macintosh HD" { return }
        
        // Inspect volume properties
        var isRemovable = false
        var isEjectable = false
        if let values = try? url.resourceValues(forKeys: [.volumeIsRemovableKey, .volumeIsEjectableKey, .volumeIsInternalKey]) {
            isRemovable = values.volumeIsRemovable ?? false
            isEjectable = values.volumeIsEjectable ?? false
            if values.volumeIsInternal == true && !isRemovable && !isEjectable {
                return
            }
        }
        
        // Detect camera card signature: DCIM directory
        let dcimURL = url.appendingPathComponent("DCIM")
        var isDir: ObjCBool = false
        let hasDcim = FileManager.default.fileExists(atPath: dcimURL.path, isDirectory: &isDir) && isDir.boolValue
        
        let isCameraCard = hasDcim || isRemovable || isEjectable
        if isCameraCard {
            let cardInfo = CameraCardInfo(
                name: url.lastPathComponent,
                url: url,
                isCameraCard: isCameraCard,
                hasDcim: hasDcim
            )
            DispatchQueue.main.async { [weak self] in
                self?.onCameraCardDetected?(cardInfo)
            }
        }
    }
}
