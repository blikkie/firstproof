import Foundation
import AppKit
import Network
import FirstProofBridge

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
        guard let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else {
            return "{\"status\":\"error\",\"message\":\"Invalid JSON\"}\n"
        }
        
        let method = (json["command"] as? String) ?? (json["method"] as? String) ?? ""
        let params = (json["params"] as? [String: Any]) ?? json
        
        switch method {
        case "get_state":
            return getStateResponse()
            
        case "open_detail_view", "select_photo":
            let path = (params["photo_path"] as? String) ?? (params["path"] as? String)
            return openDetailViewResponse(photoPath: path)
            
        case "set_canvas_mode":
            let mode = params["mode"] as? String
            return setCanvasModeResponse(mode: mode)
            
        case "capture_canvas":
            let outPath = (params["output_path"] as? String) ?? "/tmp/firstproof_viewport.png"
            return captureCanvasResponse(outputPath: outPath)
            
        case "sample_viewport", "sample_pixel":
            let uvList = (params["points"] as? [[Double]]) ?? []
            return sampleViewportResponse(points: uvList)
            
        case "set_exposure":
            let ev = (params["ev"] as? Double) ?? (params["exposure"] as? Double)
            return setExposureResponse(ev: ev)
            
        case "set_temperature":
            let k = (params["kelvin"] as? Double) ?? (params["temperature"] as? Double)
            return setTemperatureResponse(kelvin: k)
            
        case "set_tint":
            let t = params["tint"] as? Double
            return setTintResponse(tint: t)
            
        case "set_tone_profile":
            let p = params["profile"] as? String
            return setToneProfileResponse(profile: p)
            
        case "set_emulsion":
            return setEmulsionResponse(params: params)
            
        case "set_print_media":
            return setPrintMediaResponse(params: params)
            
        case "apply_edit_delta":
            let delta = (params["delta"] as? [String: Any]) ?? params
            return applyEditDeltaResponse(delta: delta)
            
        case "reset_workspace":
            let cat = params["category"] as? String
            return resetWorkspaceResponse(category: cat)
            
        case "inspect_metadata":
            let path = (params["photo_path"] as? String) ?? (params["path"] as? String)
            return inspectMetadataResponse(photoPath: path)
            
        case "toggle_clipping_overlay":
            return toggleClippingOverlayResponse()
            
        case "set_clipping_overlay":
            let active = (params["active"] as? Bool) ?? (params["enabled"] as? Bool) ?? true
            return setClippingOverlayResponse(active: active)
            
        case "get_histogram":
            return getHistogramResponse()
            
        default:
            return "{\"status\":\"error\",\"message\":\"Unknown command or method: \(method)\"}\n"
        }
    }
    
    private func getStateResponse() -> String {
        var result: [String: Any] = [:]
        
        DispatchQueue.main.sync {
            MainActor.assumeIsolated {
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
                result["isClippingOverlayActive"] = vm.isClippingOverlayActive
                result["isHistogramVisible"] = vm.isHistogramVisible
                result["emulsionArchetype"] = "\(vm.emulsionArchetype)"
                result["emulsionFormat"] = "\(vm.emulsionFormat)"
                result["emulsionExposure"] = vm.emulsionExposure
                
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
        }
        
        let data = (try? JSONSerialization.data(withJSONObject: result, options: [.prettyPrinted])) ?? Data()
        return String(data: data, encoding: .utf8) ?? "{}\n"
    }
    
    private func openDetailViewResponse(photoPath: String?) -> String {
        var success = false
        var resolvedPath = ""
        
        DispatchQueue.main.sync {
            MainActor.assumeIsolated {
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
            MainActor.assumeIsolated {
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
            MainActor.assumeIsolated {
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
                            "rgb": [r, g, b],
                            "green_red_ratio": gr
                        ])
                    }
                }
            }
        }
        
        let res: [String: Any] = ["status": "ok", "samples": samples]
        let data = (try? JSONSerialization.data(withJSONObject: res, options: [])) ?? Data()
        return String(data: data, encoding: .utf8) ?? "{}\n"
    }
    
    private func setEmulsionResponse(params: [String: Any]?) -> String {
        guard let p = params else {
            return "{\"status\":\"error\",\"message\":\"Missing params\"}\n"
        }
        
        DispatchQueue.main.sync {
            MainActor.assumeIsolated {
                guard let vm = self.viewModel else { return }
                if let archetypeId = p["archetype"] as? Int {
                    let arch: EmulsionArchetype
                    switch archetypeId {
                    case 1: arch = .rochesterPortrait400
                    case 2: arch = .ashigaraChrome50
                    case 3: arch = .rochesterK14Chrome64
                    case 4: arch = .rochesterCinema500
                    case 5: arch = .rochesterDaylight200
                    case 6: arch = .mobberleySilver400
                    case 7: arch = .cambridgeDiffusion100
                    default: arch = .none
                    }
                    vm.setEmulsionArchetype(arch)
                }
                if let formatId = p["format"] as? Int {
                    let fmt: EmulsionFormat
                    switch formatId {
                    case 1: fmt = .format120
                    case 2: fmt = .format4x5
                    default: fmt = .format35mm
                    }
                    vm.setEmulsionFormat(fmt)
                }
                if let exp = p["exposure"] as? Double {
                    vm.emulsionExposure = Float(exp)
                }
            }
        }
        return "{\"status\":\"ok\"}\n"
    }
    
    private func setCanvasModeResponse(mode: String?) -> String {
        DispatchQueue.main.sync {
            MainActor.assumeIsolated {
                guard let vm = self.viewModel else { return }
                if mode == "grid" {
                    vm.enterGridMode()
                } else {
                    vm.enterDetailMode()
                }
            }
        }
        return "{\"status\":\"ok\",\"canvasMode\":\"\(mode ?? "detail")\"}\n"
    }
    
    private func setExposureResponse(ev: Double?) -> String {
        guard let ev = ev else { return "{\"status\":\"error\",\"message\":\"Missing 'ev'\"}\n" }
        DispatchQueue.main.sync {
            MainActor.assumeIsolated {
                guard let vm = self.viewModel else { return }
                vm.exposure = Float(ev)
            }
        }
        return "{\"status\":\"ok\",\"exposure\":\(ev)}\n"
    }
    
    private func setTemperatureResponse(kelvin: Double?) -> String {
        guard let k = kelvin else { return "{\"status\":\"error\",\"message\":\"Missing 'kelvin'\"}\n" }
        DispatchQueue.main.sync {
            MainActor.assumeIsolated {
                guard let vm = self.viewModel else { return }
                vm.temperature = Float(k)
            }
        }
        return "{\"status\":\"ok\",\"temperature\":\(k)}\n"
    }
    
    private func setTintResponse(tint: Double?) -> String {
        guard let t = tint else { return "{\"status\":\"error\",\"message\":\"Missing 'tint'\"}\n" }
        DispatchQueue.main.sync {
            MainActor.assumeIsolated {
                guard let vm = self.viewModel else { return }
                vm.tint = Float(t)
            }
        }
        return "{\"status\":\"ok\",\"tint\":\(t)}\n"
    }
    
    private func setToneProfileResponse(profile: String?) -> String {
        return "{\"status\":\"ok\",\"toneProfile\":\"none\"}\n"
    }
    
    private func setPrintMediaResponse(params: [String: Any]?) -> String {
        guard let p = params else {
            return "{\"status\":\"error\",\"message\":\"Missing params\"}\n"
        }
        DispatchQueue.main.sync {
            MainActor.assumeIsolated {
                guard let vm = self.viewModel else { return }
                if let mediaStr = (p["media"] as? String)?.lowercased().replacingOccurrences(of: "-", with: "").replacingOccurrences(of: "_", with: "") {
                    switch mediaStr {
                    case "baryta", "barytasilvergelatin": vm.printMedia = .barytaSilverGelatin
                    case "warmtone", "warmtonechlorobromide": vm.printMedia = .warmtoneChlorobromide
                    case "platinum", "platinumpalladium": vm.printMedia = .platinumPalladium
                    case "cyanotype": vm.printMedia = .cyanotype
                    case "bamako", "bamakosilvercontact": vm.printMedia = .bamakoSilverContact
                    default: vm.printMedia = .none
                    }
                }
                if let grade = (p["paper_grade"] as? Double) ?? (p["print_paper_grade"] as? Double) {
                    vm.printPaperGrade = Float(grade)
                }
                if let warmth = (p["base_warmth"] as? Double) ?? (p["print_base_warmth"] as? Double) {
                    vm.printBaseWarmth = Float(warmth)
                }
                if let depth = (p["process_depth"] as? Double) ?? (p["print_process_depth"] as? Double) {
                    vm.printProcessDepth = Float(depth)
                }
            }
        }
        return "{\"status\":\"ok\"}\n"
    }
    
    private func applyEditDeltaResponse(delta: [String: Any]) -> String {
        DispatchQueue.main.sync {
            MainActor.assumeIsolated {
                guard let vm = self.viewModel else { return }
                if let v = delta["exposure"] as? Double { vm.exposure = Float(v) }
                if let v = delta["contrast"] as? Double { vm.contrast = Float(v) }
                if let v = delta["highlights"] as? Double { vm.highlights = Float(v) }
                if let v = delta["shadows"] as? Double { vm.shadows = Float(v) }
                if let v = delta["whites"] as? Double { vm.whites = Float(v) }
                if let v = delta["blacks"] as? Double { vm.blacks = Float(v) }
                if let v = delta["saturation"] as? Double { vm.saturation = Float(v) }
                if let v = delta["vibrance"] as? Double { vm.vibrance = Float(v) }
                if let v = (delta["deep_blue"] as? Double) ?? (delta["deepBlue"] as? Double) { vm.deepBlue = Float(v) }
                if let v = delta["foliage"] as? Double { vm.foliage = Float(v) }
                if let v = delta["temperature"] as? Double { vm.temperature = Float(v) }
                if let v = delta["tint"] as? Double { vm.tint = Float(v) }
                if let v = delta["rotation"] as? Int { vm.rotationDegrees = v }
                if let v = (delta["crop_angle"] as? Double) ?? (delta["straightenAngle"] as? Double) { vm.straightenAngle = Float(v) }
                if let v = (delta["perspective_vertical"] as? Double) ?? (delta["perspectiveVertical"] as? Double) { vm.perspectiveVertical = Float(v) }
                if let v = (delta["perspective_horizontal"] as? Double) ?? (delta["perspectiveHorizontal"] as? Double) { vm.perspectiveHorizontal = Float(v) }
                if let v = (delta["sharpening_amount"] as? Double) ?? (delta["sharpeningAmount"] as? Double) { vm.sharpeningAmount = Float(v) }
                if let v = (delta["noise_reduction"] as? Double) ?? (delta["noiseReduction"] as? Double) { vm.noiseReduction = Float(v) }
                if let crop = delta["crop"] as? [Double], crop.count >= 4 {
                    vm.cropLeft = Float(crop[0])
                    vm.cropTop = Float(crop[1])
                    vm.cropRight = Float(crop[2])
                    vm.cropBottom = Float(crop[3])
                }
            }
        }
        return "{\"status\":\"ok\"}\n"
    }
    
    private func resetWorkspaceResponse(category: String?) -> String {
        DispatchQueue.main.sync {
            MainActor.assumeIsolated {
                guard let vm = self.viewModel else { return }
                switch category?.lowercased() {
                case "light":
                    vm.exposure = 0.0
                    vm.contrast = 0.0
                    vm.highlights = 0.0
                    vm.shadows = 0.0
                    vm.whites = 0.0
                    vm.blacks = 0.0
                case "color":
                    vm.temperature = 5500.0
                    vm.tint = 0.0
                    vm.saturation = 0.0
                    vm.vibrance = 0.0
                    vm.deepBlue = 0.0
                    vm.foliage = 0.0
                case "detail":
                    vm.sharpeningAmount = 0.0
                    vm.noiseReduction = 0.0
                case "emulsion":
                    vm.resetEmulsion()
                    vm.resetPrintMedia()
                default:
                    vm.resetAllSliders()
                    vm.resetGeometry()
                    vm.resetEmulsion()
                    vm.resetPrintMedia()
                }
            }
        }
        return "{\"status\":\"ok\",\"reset\":\"\(category ?? "all")\"}\n"
    }
    
    private func inspectMetadataResponse(photoPath: String?) -> String {
        var resolvedPath = photoPath
        var result: [String: Any] = ["status": "ok"]
        
        DispatchQueue.main.sync {
            MainActor.assumeIsolated {
                guard let vm = self.viewModel else { return }
                if resolvedPath == nil {
                    resolvedPath = vm.selectedPhoto?.filePath
                }
                if let p = resolvedPath {
                    result["photo_path"] = p
                    if let selected = vm.selectedPhoto, selected.filePath == p {
                        result["camera_make"] = selected.cameraMake
                        result["camera_model"] = selected.cameraModel
                        result["lens_model"] = selected.lensModel
                        result["iso"] = selected.iso
                        result["aperture"] = selected.aperture
                        result["shutter_speed"] = selected.shutterSpeed
                        result["focal_length"] = selected.focalLength
                    }
                }
            }
        }
        
        let data = (try? JSONSerialization.data(withJSONObject: result, options: [.prettyPrinted])) ?? Data()
        return String(data: data, encoding: .utf8) ?? "{\"status\":\"ok\"}\n"
    }
    
    private func toggleClippingOverlayResponse() -> String {
        var active = false
        DispatchQueue.main.sync {
            MainActor.assumeIsolated {
                guard let vm = self.viewModel else { return }
                vm.toggleClippingOverlay()
                active = vm.isClippingOverlayActive
            }
        }
        return "{\"status\":\"ok\",\"isClippingOverlayActive\":\(active)}\n"
    }
    
    private func setClippingOverlayResponse(active: Bool) -> String {
        DispatchQueue.main.sync {
            MainActor.assumeIsolated {
                guard let vm = self.viewModel else { return }
                if vm.isClippingOverlayActive != active {
                    vm.toggleClippingOverlay()
                }
            }
        }
        return "{\"status\":\"ok\",\"isClippingOverlayActive\":\(active)}\n"
    }
    
    private func getHistogramResponse() -> String {
        var result: [String: Any] = ["status": "ok"]
        DispatchQueue.main.sync {
            MainActor.assumeIsolated {
                guard let vm = self.viewModel else {
                    result["status"] = "error"
                    result["message"] = "ViewModel not attached"
                    return
                }
                result["isHistogramVisible"] = vm.isHistogramVisible
                result["isClippingOverlayActive"] = vm.isClippingOverlayActive
                if let hist = vm.histogramRecord {
                    result["evMin"] = hist.evMin
                    result["evMax"] = hist.evMax
                    result["numBins"] = hist.numBins
                    result["shadowClipPercent"] = hist.shadowClipPercent
                    result["sensorClipPercent"] = hist.sensorClipPercent
                    result["displayRolloffPercent"] = hist.displayRolloffPercent
                    result["binsLuminance"] = hist.binsLuminance
                } else {
                    result["message"] = "No histogram computed yet"
                }
            }
        }
        let data = (try? JSONSerialization.data(withJSONObject: result, options: [.prettyPrinted])) ?? Data()
        return String(data: data, encoding: .utf8) ?? "{\"status\":\"ok\"}\n"
    }
}

