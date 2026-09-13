import Foundation
import AppKit
import Network

/// Lightweight Local Unix Domain Socket IPC Service for automated parity audits,
/// developer diagnostics, and AI coding agent introspection.
///
/// Stripped out of production or enabled only during development builds and explicit CLI flags.
/// In accordance with First Proof anti-slop values, this service provides inspection,
/// pixel sampling, canvas frame capture, and navigation (open detail view) WITHOUT allowing
/// automated/AI slider manipulation.
public final class DebugIPCService {
    public static let shared = DebugIPCService()
    
    private weak var viewModel: AppViewModel?
    private var serverSocket: Int32 = -1
    private var isRunning = false
    private let socketPath = "/tmp/firstproof-debug.sock"
    private let listenerQueue = DispatchQueue(label: "photo.firstproof.debug-ipc.listener", qos: .utility)
    
    private init() {}
    
    public func start(with viewModel: AppViewModel) {
        self.viewModel = viewModel
        
        #if DEBUG
        startServer()
        #else
        if ProcessInfo.processInfo.arguments.contains("--enable-debug-ipc") ||
           ProcessInfo.processInfo.environment["FIRSTPROOF_DEBUG_IPC"] != nil {
            startServer()
        }
        #endif
    }
    
    public func stop() {
        isRunning = false
        if serverSocket >= 0 {
            close(serverSocket)
            serverSocket = -1
        }
        unlink(socketPath)
    }
    
    private func startServer() {
        guard !isRunning else { return }
        isRunning = true
        
        listenerQueue.async { [weak self] in
            guard let self = self else { return }
            
            unlink(self.socketPath)
            
            let sock = socket(AF_UNIX, SOCK_STREAM, 0)
            guard sock >= 0 else {
                NSLog("[DebugIPC] Failed to create socket")
                return
            }
            self.serverSocket = sock
            
            var addr = sockaddr_un()
            addr.sun_family = sa_family_t(AF_UNIX)
            _ = withUnsafeMutablePointer(to: &addr.sun_path.0) { ptr in
                self.socketPath.withCString { cStr in
                    strncpy(ptr, cStr, 104)
                }
            }
            
            let bindResult = withUnsafePointer(to: &addr) { ptr in
                ptr.withMemoryRebound(to: sockaddr.self, capacity: 1) { sockAddrPtr in
                    bind(sock, sockAddrPtr, socklen_t(MemoryLayout<sockaddr_un>.size))
                }
            }
            
            guard bindResult == 0 else {
                NSLog("[DebugIPC] Failed to bind socket to \(self.socketPath)")
                close(sock)
                return
            }
            
            guard listen(sock, 5) == 0 else {
                NSLog("[DebugIPC] Failed to listen on socket")
                close(sock)
                return
            }
            
            chmod(self.socketPath, 0o777)
            NSLog("[DebugIPC] Listening on \(self.socketPath)")
            
            while self.isRunning {
                var clientAddr = sockaddr_un()
                var clientLen = socklen_t(MemoryLayout<sockaddr_un>.size)
                let clientSock = withUnsafeMutablePointer(to: &clientAddr) { ptr in
                    ptr.withMemoryRebound(to: sockaddr.self, capacity: 1) { sockAddrPtr in
                        accept(sock, sockAddrPtr, &clientLen)
                    }
                }
                
                guard clientSock >= 0 else {
                    if !self.isRunning { break }
                    continue
                }
                
                // Dispatch client processing to global queue so listener loop remains non-blocking
                DispatchQueue.global(qos: .userInitiated).async { [weak self] in
                    self?.handleClient(clientSock)
                }
            }
        }
    }
    
    private func handleClient(_ clientSock: Int32) {
        var buffer = [UInt8](repeating: 0, count: 65536)
        let bytesRead = read(clientSock, &buffer, buffer.count)
        guard bytesRead > 0 else {
            close(clientSock)
            return
        }
        
        let requestData = Data(buffer[0..<bytesRead])
        let response = processRequest(requestData)
        
        if let responseData = response.data(using: .utf8) {
            responseData.withUnsafeBytes { rawBuffer in
                if let baseAddress = rawBuffer.baseAddress {
                    _ = write(clientSock, baseAddress, responseData.count)
                }
            }
        }
        close(clientSock)
    }
    
    private func processRequest(_ data: Data) -> String {
        guard let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              let method = json["method"] as? String else {
            return "{\"status\":\"error\",\"message\":\"Invalid JSON or missing method\"}\n"
        }
        
        switch method {
        case "get_state":
            return getStateResponse()
            
        case "open_detail_view":
            let params = json["params"] as? [String: Any]
            let path = params?["photo_path"] as? String
            return openDetailViewResponse(photoPath: path)
            
        case "capture_canvas":
            let params = json["params"] as? [String: Any]
            let outPath = params?["output_path"] as? String ?? "/tmp/firstproof_viewport.png"
            return captureCanvasResponse(outputPath: outPath)
            
        case "sample_viewport":
            let params = json["params"] as? [String: Any]
            let uvList = params?["points"] as? [[Double]] ?? []
            return sampleViewportResponse(points: uvList)
            
        default:
            return "{\"status\":\"error\",\"message\":\"Unknown method: \(method)\"}\n"
        }
    }
    
    private func getStateResponse() -> String {
        var result: [String: Any] = [:]
        
        DispatchQueue.main.sync {
            guard let vm = self.viewModel else {
                result["status"] = "error"
                result["message"] = "ViewModel not attached"
                return
            }
            
            result["status"] = "ok"
            result["selectedPhoto"] = vm.selectedPhoto?.filePath
            result["isRaw"] = vm.selectedPhotoIsRaw
            result["canvasMode"] = "\(vm.canvasMode)"
            result["lensCorrectionEnabled"] = vm.lensCorrectionEnabled
            
            let e = vm.currentEdits
            result["edits"] = [
                "temperature": e.temperature,
                "tint": e.tint,
                "exposure": e.exposure,
                "contrast": e.contrast,
                "highlights": e.highlights,
                "shadows": e.shadows,
                "whites": e.whites,
                "blacks": e.blacks,
                "saturation": e.saturation,
                "vibrance": e.vibrance,
                "rotation": e.rotation,
                "crop": [e.cropLeft, e.cropTop, e.cropRight, e.cropBottom]
            ]
            
            if let lens = vm.lensCoefficients {
                result["lensCoefficients"] = [
                    "model": lens.distortionModel,
                    "distortion": lens.distortion,
                    "hasDistortion": lens.hasDistortion
                ]
            }
        }
        
        let data = (try? JSONSerialization.data(withJSONObject: result, options: [.prettyPrinted])) ?? Data()
        return String(data: data, encoding: .utf8) ?? "{}\n"
    }
    
    private func openDetailViewResponse(photoPath: String?) -> String {
        var success = false
        var resolvedPath = ""
        
        DispatchQueue.main.sync {
            guard let vm = self.viewModel else { return }
            
            if let path = photoPath {
                if let target = vm.photos.first(where: { $0.filePath == path }) ??
                                 vm.filteredPhotos.first(where: { $0.filePath == path }) {
                    vm.selectPhoto(target)
                    vm.enterDetailMode()
                    resolvedPath = target.filePath
                    success = true
                }
            } else {
                vm.enterDetailMode()
                resolvedPath = vm.selectedPhoto?.filePath ?? ""
                success = true
            }
        }
        
        if success {
            return "{\"status\":\"ok\",\"selectedPhoto\":\"\(resolvedPath)\",\"canvasMode\":\"detail\"}\n"
        } else {
            return "{\"status\":\"error\",\"message\":\"Could not locate photo at specified path\"}\n"
        }
    }
    
    private func captureCanvasResponse(outputPath: String) -> String {
        var success = false
        var imageSize: CGSize = .zero
        
        DispatchQueue.main.sync {
            guard let vm = self.viewModel else { return }
            
            let snapshot = vm.captureIntermediateSnapshot?() ?? vm.captureThumbnailSnapshot?()
            if let img = snapshot {
                imageSize = img.size
                if let tiffData = img.tiffRepresentation,
                   let bitmap = NSBitmapImageRep(data: tiffData),
                   let pngData = bitmap.representation(using: .png, properties: [:]) {
                    let url = URL(fileURLWithPath: outputPath)
                    try? pngData.write(to: url)
                    success = true
                }
            }
        }
        
        if success {
            return "{\"status\":\"ok\",\"output_path\":\"\(outputPath)\",\"width\":\(Int(imageSize.width)),\"height\":\(Int(imageSize.height))}\n"
        } else {
            return "{\"status\":\"error\",\"message\":\"Failed to capture viewport snapshot\"}\n"
        }
    }
    
    private func sampleViewportResponse(points: [[Double]]) -> String {
        var samples: [[String: Any]] = []
        
        DispatchQueue.main.sync {
            guard let vm = self.viewModel else { return }
            guard let img = vm.captureIntermediateSnapshot?() ?? vm.captureThumbnailSnapshot?() else { return }
            guard let tiffData = img.tiffRepresentation,
                  let bitmap = NSBitmapImageRep(data: tiffData) else { return }
            
            let w = bitmap.pixelsWide
            let h = bitmap.pixelsHigh
            
            for pt in points {
                guard pt.count >= 2 else { continue }
                let u = pt[0]
                let v = pt[1]
                let px = min(max(Int(Double(w) * u), 0), w - 1)
                let py = min(max(Int(Double(h) * v), 0), h - 1)
                
                if let color = bitmap.colorAt(x: px, y: py) {
                    let r = Int((color.redComponent * 255.0).rounded())
                    let g = Int((color.greenComponent * 255.0).rounded())
                    let b = Int((color.blueComponent * 255.0).rounded())
                    let gr = (r > 0) ? Double(g) / Double(r) : 0.0
                    
                    samples.append([
                        "uv": [u, v],
                        "pixel": [px, py],
                        "rgb": [r, g, b],
                        "gr_ratio": gr
                    ])
                }
            }
        }
        
        let res: [String: Any] = ["status": "ok", "samples": samples]
        let data = (try? JSONSerialization.data(withJSONObject: res, options: [])) ?? Data()
        return String(data: data, encoding: .utf8) ?? "{}\n"
    }
}
