import Testing
import Foundation
@testable import FirstProofApp
import FirstProofBridge

@Suite("Sensitometric EV Zone Histogram & Sensor Clipping HUD Invariants")
struct HistogramTests {
    
    // Helper to build a 16-bit float RGBA buffer
    private func makeTestBuffer(r: Float, g: Float, b: Float, count: Int = 1000) -> Data {
        var data = Data(capacity: count * 8)
        let r16 = Float16(r).bitPattern
        let g16 = Float16(g).bitPattern
        let b16 = Float16(b).bitPattern
        let a16 = Float16(1.0).bitPattern
        
        for _ in 0..<count {
            withUnsafeBytes(of: r16) { data.append(contentsOf: $0) }
            withUnsafeBytes(of: g16) { data.append(contentsOf: $0) }
            withUnsafeBytes(of: b16) { data.append(contentsOf: $0) }
            withUnsafeBytes(of: a16) { data.append(contentsOf: $0) }
        }
        return data
    }

    @Test("Middle grey (0.18 linear scene radiance) centers predictably in Zone V (0.0 EV)")
    func testMiddleGreyZoneVCentering() {
        let buffer = makeTestBuffer(r: 0.18, g: 0.18, b: 0.18)
        let hist = computeEvHistogram(
            width: 100, height: 10, data: buffer,
            exposure: 0.0, contrast: 0.0, highlights: 0.0, shadows: 0.0, whites: 0.0, blacks: 0.0,
            cropLeft: 0.0, cropTop: 0.0, cropRight: 1.0, cropBottom: 1.0,
            lensCorrectionEnabled: false, tcaCorrectionEnabled: true,
            vignettingCorrectionEnabled: false, vignettingK1: 0.0, vignettingK2: 0.0, vignettingK3: 0.0
        )
        
        #expect(hist != nil)
        guard let hist = hist else { return }
        
        #expect(hist.numBins == 120)
        #expect(hist.evMin == -8.0)
        #expect(hist.evMax == 4.0)
        #expect(hist.shadowClipPercent == 0.0)
        #expect(hist.sensorClipPercent == 0.0)
        #expect(hist.displayRolloffPercent == 0.0)
        
        // Zone V (0.0 EV) nominal center is bin 80 (or 79 due to Float16 0.1799 representation)
        let peakInZoneV = hist.binsLuminance[79] == 1.0 || hist.binsLuminance[80] == 1.0
        #expect(peakInZoneV, "Middle grey must peak at or adjacent to Zone V (0.0 EV)")
    }

    @Test("Exposure compensation shifts sensitometric EV distribution by exact integer stops")
    func testExposureGainShiftsStops() {
        let buffer = makeTestBuffer(r: 0.18, g: 0.18, b: 0.18)
        
        // +1.0 EV exposure gain shifts peak right by exactly 10 bins (1 full stop = 10 bins)
        let plus1 = computeEvHistogram(
            width: 100, height: 10, data: buffer,
            exposure: 1.0, contrast: 0.0, highlights: 0.0, shadows: 0.0, whites: 0.0, blacks: 0.0,
            cropLeft: 0.0, cropTop: 0.0, cropRight: 1.0, cropBottom: 1.0,
            lensCorrectionEnabled: false, tcaCorrectionEnabled: true,
            vignettingCorrectionEnabled: false, vignettingK1: 0.0, vignettingK2: 0.0, vignettingK3: 0.0
        )
        #expect(plus1 != nil)
        if let plus1 = plus1 {
            let shiftedToZoneVI = plus1.binsLuminance[89] == 1.0 || plus1.binsLuminance[90] == 1.0
            #expect(shiftedToZoneVI, "+1 EV must shift peak to Zone VI (+1.0 EV, bin 89/90)")
        }
        
        // -2.0 EV exposure cut shifts peak left by exactly 20 bins (2 full stops = 20 bins)
        let minus2 = computeEvHistogram(
            width: 100, height: 10, data: buffer,
            exposure: -2.0, contrast: 0.0, highlights: 0.0, shadows: 0.0, whites: 0.0, blacks: 0.0,
            cropLeft: 0.0, cropTop: 0.0, cropRight: 1.0, cropBottom: 1.0,
            lensCorrectionEnabled: false, tcaCorrectionEnabled: true,
            vignettingCorrectionEnabled: false, vignettingK1: 0.0, vignettingK2: 0.0, vignettingK3: 0.0
        )
        #expect(minus2 != nil)
        if let minus2 = minus2 {
            let shiftedToZoneIII = minus2.binsLuminance[59] == 1.0 || minus2.binsLuminance[60] == 1.0
            #expect(shiftedToZoneIII, "-2 EV must shift peak to Zone III (-2.0 EV, bin 59/60)")
        }
    }

    @Test("Tone recovery sliders (whites, blacks, highlights, shadows, contrast) alter histogram distribution")
    func testToneRecoveryAltersHistogram() {
        let buffer = makeTestBuffer(r: 0.5, g: 0.5, b: 0.5)
        
        let baseline = computeEvHistogram(
            width: 100, height: 10, data: buffer,
            exposure: 0.0, contrast: 0.0, highlights: 0.0, shadows: 0.0, whites: 0.0, blacks: 0.0,
            cropLeft: 0.0, cropTop: 0.0, cropRight: 1.0, cropBottom: 1.0,
            lensCorrectionEnabled: false, tcaCorrectionEnabled: true,
            vignettingCorrectionEnabled: false, vignettingK1: 0.0, vignettingK2: 0.0, vignettingK3: 0.0
        )
        #expect(baseline != nil)
        
        // Boosting whites (+100) must increase luminance / shift distribution upwards
        let boostedWhites = computeEvHistogram(
            width: 100, height: 10, data: buffer,
            exposure: 0.0, contrast: 0.0, highlights: 0.0, shadows: 0.0, whites: 100.0, blacks: 0.0,
            cropLeft: 0.0, cropTop: 0.0, cropRight: 1.0, cropBottom: 1.0,
            lensCorrectionEnabled: false, tcaCorrectionEnabled: true,
            vignettingCorrectionEnabled: false, vignettingK1: 0.0, vignettingK2: 0.0, vignettingK3: 0.0
        )
        #expect(boostedWhites != nil)
        if let baseline = baseline, let boostedWhites = boostedWhites {
            #expect(boostedWhites.binsLuminance != baseline.binsLuminance)
        }
        
        // Pulling blacks (-100) on shadow buffer shifts distribution downwards
        let shadowBuffer = makeTestBuffer(r: 0.04, g: 0.04, b: 0.04)
        let baselineShadow = computeEvHistogram(
            width: 100, height: 10, data: shadowBuffer,
            exposure: 0.0, contrast: 0.0, highlights: 0.0, shadows: 0.0, whites: 0.0, blacks: 0.0,
            cropLeft: 0.0, cropTop: 0.0, cropRight: 1.0, cropBottom: 1.0,
            lensCorrectionEnabled: false, tcaCorrectionEnabled: true,
            vignettingCorrectionEnabled: false, vignettingK1: 0.0, vignettingK2: 0.0, vignettingK3: 0.0
        )
        let crushedBlacks = computeEvHistogram(
            width: 100, height: 10, data: shadowBuffer,
            exposure: 0.0, contrast: 0.0, highlights: 0.0, shadows: 0.0, whites: 0.0, blacks: -100.0,
            cropLeft: 0.0, cropTop: 0.0, cropRight: 1.0, cropBottom: 1.0,
            lensCorrectionEnabled: false, tcaCorrectionEnabled: true,
            vignettingCorrectionEnabled: false, vignettingK1: 0.0, vignettingK2: 0.0, vignettingK3: 0.0
        )
        if let baselineShadow = baselineShadow, let crushedBlacks = crushedBlacks {
            #expect(crushedBlacks.binsLuminance != baselineShadow.binsLuminance)
        }
    }

    @Test("Cropping limits histogram evaluation strictly to within active crop window")
    func testCropSamplingIsolation() {
        // Half black (left half, x < 50), half blown white (right half, x >= 50)
        let width = 100
        let height = 20
        var data = Data(capacity: width * height * 8)
        let black16 = Float16(0.0).bitPattern
        let white16 = Float16(1.5).bitPattern
        let one16 = Float16(1.0).bitPattern

        for _ in 0..<height {
            for x in 0..<width {
                let val = x < 50 ? black16 : white16
                withUnsafeBytes(of: val) { data.append(contentsOf: $0) }
                withUnsafeBytes(of: val) { data.append(contentsOf: $0) }
                withUnsafeBytes(of: val) { data.append(contentsOf: $0) }
                withUnsafeBytes(of: one16) { data.append(contentsOf: $0) }
            }
        }

        // 1. Full uncropped: ~50% shadow clip, ~50% sensor clip
        let full = computeEvHistogram(
            width: UInt32(width), height: UInt32(height), data: data,
            exposure: 0.0, contrast: 0.0, highlights: 0.0, shadows: 0.0, whites: 0.0, blacks: 0.0,
            cropLeft: 0.0, cropTop: 0.0, cropRight: 1.0, cropBottom: 1.0,
            lensCorrectionEnabled: false, tcaCorrectionEnabled: true,
            vignettingCorrectionEnabled: false, vignettingK1: 0.0, vignettingK2: 0.0, vignettingK3: 0.0
        )
        #expect(full != nil)
        if let full = full {
            #expect(abs(full.shadowClipPercent - 50.0) < 2.0)
            #expect(abs(full.sensorClipPercent - 50.0) < 2.0)
        }

        // 2. Cropped to left half: 100% shadow clip, 0% sensor clip
        let leftCrop = computeEvHistogram(
            width: UInt32(width), height: UInt32(height), data: data,
            exposure: 0.0, contrast: 0.0, highlights: 0.0, shadows: 0.0, whites: 0.0, blacks: 0.0,
            cropLeft: 0.0, cropTop: 0.0, cropRight: 0.5, cropBottom: 1.0,
            lensCorrectionEnabled: false, tcaCorrectionEnabled: true,
            vignettingCorrectionEnabled: false, vignettingK1: 0.0, vignettingK2: 0.0, vignettingK3: 0.0
        )
        #expect(leftCrop != nil)
        if let leftCrop = leftCrop {
            #expect(leftCrop.shadowClipPercent == 100.0)
            #expect(leftCrop.sensorClipPercent == 0.0)
        }

        // 3. Cropped to right half: 0% shadow clip, 100% sensor clip
        let rightCrop = computeEvHistogram(
            width: UInt32(width), height: UInt32(height), data: data,
            exposure: 0.0, contrast: 0.0, highlights: 0.0, shadows: 0.0, whites: 0.0, blacks: 0.0,
            cropLeft: 0.5, cropTop: 0.0, cropRight: 1.0, cropBottom: 1.0,
            lensCorrectionEnabled: false, tcaCorrectionEnabled: true,
            vignettingCorrectionEnabled: false, vignettingK1: 0.0, vignettingK2: 0.0, vignettingK3: 0.0
        )
        #expect(rightCrop != nil)
        if let rightCrop = rightCrop {
            #expect(rightCrop.shadowClipPercent == 0.0)
            #expect(rightCrop.sensorClipPercent == 100.0)
        }
    }

    @Test("Lens distortion correction auto-crops peripheral warped border from histogram evaluation")
    func testLensDistortionAutoCrop() {
        let width = 100
        let height = 100
        var data = Data(capacity: width * height * 8)
        let black16 = Float16(0.0).bitPattern
        let white16 = Float16(1.5).bitPattern
        let one16 = Float16(1.0).bitPattern

        for y in 0..<height {
            for x in 0..<width {
                let isOuterBorder = x == 0 || x == width - 1 || y == 0 || y == height - 1
                let val = isOuterBorder ? white16 : black16
                withUnsafeBytes(of: val) { data.append(contentsOf: $0) }
                withUnsafeBytes(of: val) { data.append(contentsOf: $0) }
                withUnsafeBytes(of: val) { data.append(contentsOf: $0) }
                withUnsafeBytes(of: one16) { data.append(contentsOf: $0) }
            }
        }

        let uncorrected = computeEvHistogram(
            width: UInt32(width), height: UInt32(height), data: data,
            exposure: 0.0, contrast: 0.0, highlights: 0.0, shadows: 0.0, whites: 0.0, blacks: 0.0,
            cropLeft: 0.0, cropTop: 0.0, cropRight: 1.0, cropBottom: 1.0,
            lensCorrectionEnabled: false, tcaCorrectionEnabled: true,
            vignettingCorrectionEnabled: false, vignettingK1: 0.0, vignettingK2: 0.0, vignettingK3: 0.0
        )
        #expect(uncorrected != nil)
        if let uncorrected = uncorrected {
            #expect(uncorrected.sensorClipPercent > 0.0)
        }

        let corrected = computeEvHistogram(
            width: UInt32(width), height: UInt32(height), data: data,
            exposure: 0.0, contrast: 0.0, highlights: 0.0, shadows: 0.0, whites: 0.0, blacks: 0.0,
            cropLeft: 0.0, cropTop: 0.0, cropRight: 1.0, cropBottom: 1.0,
            lensCorrectionEnabled: true, tcaCorrectionEnabled: true,
            vignettingCorrectionEnabled: false, vignettingK1: 0.0, vignettingK2: 0.0, vignettingK3: 0.0
        )
        #expect(corrected != nil)
        if let corrected = corrected {
            #expect(corrected.sensorClipPercent == 0.0)
        }
    }

    @Test("Telemetry disentangles shadow floor clipping from true hardware photosite saturation")
    func testClippingTelemetryDisentanglement() {
        // 1. Shadow Floor (Pure black)
        let blackBuffer = makeTestBuffer(r: 0.0, g: 0.0, b: 0.0, count: 500)
        let histBlack = computeEvHistogram(
            width: 50, height: 10, data: blackBuffer,
            exposure: 0.0, contrast: 0.0, highlights: 0.0, shadows: 0.0, whites: 0.0, blacks: 0.0,
            cropLeft: 0.0, cropTop: 0.0, cropRight: 1.0, cropBottom: 1.0,
            lensCorrectionEnabled: false, tcaCorrectionEnabled: true,
            vignettingCorrectionEnabled: false, vignettingK1: 0.0, vignettingK2: 0.0, vignettingK3: 0.0
        )
        #expect(histBlack != nil)
        if let histBlack = histBlack {
            #expect(histBlack.shadowClipPercent == 100.0)
            #expect(histBlack.sensorClipPercent == 0.0)
            #expect(histBlack.binsLuminance[0] == 1.0)
        }
        
        // 2. Hardware Photosite Saturation (raw value >= 1.0)
        let blownBuffer = makeTestBuffer(r: 1.3, g: 0.5, b: 0.5, count: 500)
        let histBlown = computeEvHistogram(
            width: 50, height: 10, data: blownBuffer,
            exposure: 0.0, contrast: 0.0, highlights: 0.0, shadows: 0.0, whites: 0.0, blacks: 0.0,
            cropLeft: 0.0, cropTop: 0.0, cropRight: 1.0, cropBottom: 1.0,
            lensCorrectionEnabled: false, tcaCorrectionEnabled: true,
            vignettingCorrectionEnabled: false, vignettingK1: 0.0, vignettingK2: 0.0, vignettingK3: 0.0
        )
        #expect(histBlown != nil)
        if let histBlown = histBlown {
            #expect(histBlown.sensorClipPercent == 100.0)
            #expect(histBlown.shadowClipPercent == 0.0)
        }
        
        // 3. AgX Filmic Shoulder Compression (Zone VII+, >= 0.72 radiance without sensor saturation)
        let shoulderBuffer = makeTestBuffer(r: 0.85, g: 0.85, b: 0.85, count: 500)
        let histShoulder = computeEvHistogram(
            width: 50, height: 10, data: shoulderBuffer,
            exposure: 0.0, contrast: 0.0, highlights: 0.0, shadows: 0.0, whites: 0.0, blacks: 0.0,
            cropLeft: 0.0, cropTop: 0.0, cropRight: 1.0, cropBottom: 1.0,
            lensCorrectionEnabled: false, tcaCorrectionEnabled: true,
            vignettingCorrectionEnabled: false, vignettingK1: 0.0, vignettingK2: 0.0, vignettingK3: 0.0
        )
        #expect(histShoulder != nil)
        if let histShoulder = histShoulder {
            #expect(histShoulder.sensorClipPercent == 0.0)
            #expect(histShoulder.displayRolloffPercent == 100.0)
        }
    }

    @Test("Vignetting correction lifts peripheral shadow floor and updates histogram reactively")
    func testVignettingCorrectionLiftsShadowFloor() {
        let width = 100
        let height = 100
        var data = Data(capacity: width * height * 8)
        let centerVal = Float16(0.18).bitPattern
        let cornerVal = Float16(0.04).bitPattern
        let alpha = Float16(1.0).bitPattern
        
        for y in 0..<height {
            for x in 0..<width {
                let dx = Double(x) / Double(width) - 0.5
                let dy = Double(y) / Double(height) - 0.5
                let dist = sqrt(dx * dx + dy * dy)
                let val = dist > 0.4 ? cornerVal : centerVal
                withUnsafeBytes(of: val) { data.append(contentsOf: $0) }
                withUnsafeBytes(of: val) { data.append(contentsOf: $0) }
                withUnsafeBytes(of: val) { data.append(contentsOf: $0) }
                withUnsafeBytes(of: alpha) { data.append(contentsOf: $0) }
            }
        }
        
        let uncorrected = computeEvHistogram(
            width: UInt32(width), height: UInt32(height), data: data,
            exposure: 0.0, contrast: 0.0, highlights: 0.0, shadows: 0.0, whites: 0.0, blacks: 0.0,
            cropLeft: 0.0, cropTop: 0.0, cropRight: 1.0, cropBottom: 1.0,
            lensCorrectionEnabled: false, tcaCorrectionEnabled: true,
            vignettingCorrectionEnabled: false, vignettingK1: -0.8, vignettingK2: 0.0, vignettingK3: 0.0
        )
        
        let corrected = computeEvHistogram(
            width: UInt32(width), height: UInt32(height), data: data,
            exposure: 0.0, contrast: 0.0, highlights: 0.0, shadows: 0.0, whites: 0.0, blacks: 0.0,
            cropLeft: 0.0, cropTop: 0.0, cropRight: 1.0, cropBottom: 1.0,
            lensCorrectionEnabled: false, tcaCorrectionEnabled: true,
            vignettingCorrectionEnabled: true, vignettingK1: -0.8, vignettingK2: 0.0, vignettingK3: 0.0
        )
        
        #expect(uncorrected != nil)
        #expect(corrected != nil)
        if let uncorrected = uncorrected, let corrected = corrected {
            #expect(corrected.binsLuminance != uncorrected.binsLuminance, "Vignetting correction must modify EV histogram distribution")
        }
    }

    @Test("AppViewModel toggles histogram visibility and clipping overlay cleanly")
    @MainActor
    func testViewModelHistogramState() {
        let vm = AppViewModel()
        #expect(vm.isHistogramVisible == true)
        #expect(vm.isClippingOverlayActive == false)
        
        vm.toggleHistogram()
        #expect(vm.isHistogramVisible == false)
        
        vm.toggleHistogram()
        #expect(vm.isHistogramVisible == true)
        
        vm.toggleClippingOverlay()
        #expect(vm.isClippingOverlayActive == true)
        
        vm.toggleClippingOverlay()
        #expect(vm.isClippingOverlayActive == false)
    }
}
