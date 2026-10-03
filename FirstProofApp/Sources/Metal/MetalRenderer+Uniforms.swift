import Foundation
import Metal
import AppKit
import FirstProofBridge
import FirstProofShaderTypes

extension MetalRenderer {
    // MARK: - Centralized Uniforms & Geometry Update
    public func updateUniforms(
        from edits: EditSettings,
        imageAspect: Float,
        isLinearRaw: Bool,
        isOriginal: Bool = false,
        isCropToolActive: Bool = false,
        lensCorrectionEnabled: Bool = true,
        lensCoefficients: LensCorrectionCoefficientsRecord? = nil,
        edrHeadroom: Float = 1.0,
        edrEnabled: Bool = true,
        isIntermediate: Bool = false
    ) {
        uniforms.isIntermediate = isIntermediate ? 1 : 0
        uniforms.exposure = edits.exposure
        uniforms.temperature = edits.temperature
        uniforms.tint = edits.tint
        uniforms.contrast = edits.contrast
        uniforms.edrHeadroom = edrHeadroom
        uniforms.edrEnabled = edrEnabled ? 1 : 0
        uniforms.highlights = edits.highlights
        uniforms.shadows = edits.shadows
        uniforms.whites = edits.whites
        uniforms.blacks = edits.blacks
        uniforms.vibrance = edits.vibrance
        uniforms.saturation = edits.saturation
        uniforms.deepBlue = edits.deepBlue
        uniforms.foliage = edits.foliage
        uniforms.straightenAngle = edits.cropAngle * (.pi / 180.0)
        uniforms.aspectRatio = imageAspect
        uniforms.lensCorrectionEnabled = lensCorrectionEnabled ? 1 : 0
        uniforms.isOriginal = isOriginal ? 1.0 : 0.0
        uniforms.isLinearRaw = isLinearRaw ? 1 : 0
        uniforms.lutEnabled = (edits.lutName != nil) ? 1 : 0
        uniforms.lutIntensity = edits.lutIntensity
        uniforms.sharpeningAmount = edits.sharpeningAmount
        uniforms.sharpeningRadius = edits.sharpeningRadius
        uniforms.sharpeningMasking = edits.sharpeningMasking
        switch edits.emulsionArchetype {
        case .none:
            uniforms.emulsionArchetype = 0
        case .rochesterPortrait400:
            uniforms.emulsionArchetype = 1
        case .ashigaraChrome50:
            uniforms.emulsionArchetype = 2
        case .rochesterK14Chrome64:
            uniforms.emulsionArchetype = 3
        case .rochesterCinema500:
            uniforms.emulsionArchetype = 4
        case .rochesterDaylight200:
            uniforms.emulsionArchetype = 5
        case .mobberleySilver400:
            uniforms.emulsionArchetype = 6
        case .cambridgeDiffusion100:
            uniforms.emulsionArchetype = 7
        case .leverkusenPortrait160:
            uniforms.emulsionArchetype = 8
        case .ashigaraPortrait400:
            uniforms.emulsionArchetype = 9
        case .liguriaCinema80:
            uniforms.emulsionArchetype = 10
        case .baodingSilver100:
            uniforms.emulsionArchetype = 11
        }

        switch edits.emulsionFormat {
        case .format35mm:
            uniforms.emulsionFormat = 0
        case .format120:
            uniforms.emulsionFormat = 1
        case .format4x5:
            uniforms.emulsionFormat = 2
        }

        uniforms.emulsionExposure = edits.emulsionExposure
        switch edits.emulsionGrainMode {
        case .off:
            uniforms.emulsionGrainMode = 0
        case .seedFromSensor:
            uniforms.emulsionGrainMode = 1
        case .synthesizePhysical:
            uniforms.emulsionGrainMode = 2
        }
        uniforms.grainScale = edits.grainScale
        uniforms.splitToningShadowSat = edits.splitToningShadowSat
        uniforms.splitToningHighlightHue = edits.splitToningHighlightHue
        uniforms.splitToningHighlightSat = edits.splitToningHighlightSat
        uniforms.splitToningBalance = edits.splitToningBalance
        uniforms.cropLeft = edits.cropLeft
        uniforms.cropTop = edits.cropTop
        uniforms.cropRight = edits.cropRight
        uniforms.cropBottom = edits.cropBottom
        uniforms.perspectiveVertical = edits.perspectiveVertical
        uniforms.perspectiveHorizontal = edits.perspectiveHorizontal
        uniforms.cropConstrain = edits.cropConstrain ? 1 : 0
        uniforms.cropModeActive = isCropToolActive ? 1 : 0
        
        switch edits.printMedia {
        case .none:
            uniforms.printMediaMode = 0
        case .chromogenicTypeC:
            uniforms.printMediaMode = 1
        case .silverDyeBleach:
            uniforms.printMediaMode = 2
        case .fineArtCottonRag:
            uniforms.printMediaMode = 3
        case .barytaSilverGelatin:
            uniforms.printMediaMode = 4
        case .warmtoneChlorobromide:
            uniforms.printMediaMode = 5
        case .platinumPalladium:
            uniforms.printMediaMode = 6
        case .cyanotype:
            uniforms.printMediaMode = 7
        case .bamakoSilverContact:
            uniforms.printMediaMode = 8
        }
        
        switch edits.enlargerFilter {
        case .neutralPanchromatic:
            uniforms.enlargerFilterMode = 0
        case .yellowK2:
            uniforms.enlargerFilterMode = 1
        case .orange16:
            uniforms.enlargerFilterMode = 2
        case .red25A:
            uniforms.enlargerFilterMode = 3
        case .greenX1:
            uniforms.enlargerFilterMode = 4
        case .orthochromatic:
            uniforms.enlargerFilterMode = 5
        }
        
        uniforms.printPaperGrade = edits.printPaperGrade
        uniforms.printBaseWarmth = edits.printBaseWarmth
        uniforms.printProcessDepth = edits.printProcessDepth
        
        switch edits.toneProfile {
        case .agxVivid:
            uniforms.toneProfileMode = 0
        case .agxFilmic:
            uniforms.toneProfileMode = 1
        case .cameraStandard:
            uniforms.toneProfileMode = 2
        case .cameraPortrait:
            uniforms.toneProfileMode = 3
        case .cameraLandscape:
            uniforms.toneProfileMode = 4
        case .cameraVivid:
            uniforms.toneProfileMode = 5
        }
        
        if let coeff = lensCoefficients, coeff.hasDistortion {
            uniforms.distortionModel = (coeff.distortionModel == "ptlens") ? 1 : 0
            uniforms.k1 = coeff.distortion.indices.contains(0) ? coeff.distortion[0] : 0.0
            uniforms.k2 = coeff.distortion.indices.contains(1) ? coeff.distortion[1] : 0.0
            uniforms.k3 = coeff.distortion.indices.contains(2) ? coeff.distortion[2] : 0.0
            uniforms.k4 = coeff.distortion.indices.contains(3) ? coeff.distortion[3] : 0.0
        } else {
            uniforms.distortionModel = 0
            uniforms.k1 = 0.0
            uniforms.k2 = 0.0
            uniforms.k3 = 0.0
            uniforms.k4 = 0.0
        }
        
        self.rotationDegrees = Int(edits.rotation)
        self.flipHorizontal = edits.flipHorizontal
        self.flipVertical = edits.flipVertical
        
        let isMasterActive = edits.curves.master.count > 2 || (edits.curves.master.count == 2 && (edits.curves.master[0].y != 0.0 || edits.curves.master[1].y != 1.0))
        let isRedActive = edits.curves.red.count > 2 || (edits.curves.red.count == 2 && (edits.curves.red[0].y != 0.0 || edits.curves.red[1].y != 1.0))
        let isGreenActive = edits.curves.green.count > 2 || (edits.curves.green.count == 2 && (edits.curves.green[0].y != 0.0 || edits.curves.green[1].y != 1.0))
        let isBlueActive = edits.curves.blue.count > 2 || (edits.curves.blue.count == 2 && (edits.curves.blue[0].y != 0.0 || edits.curves.blue[1].y != 1.0))
        let hasRgb = isRedActive || isGreenActive || isBlueActive
        let hasCurves = isMasterActive || hasRgb
        uniforms.hasMasterCurve = isMasterActive ? 1 : 0
        uniforms.hasRgbCurves = hasRgb ? 1 : 0
        uniforms.hasCurves = hasCurves ? 1 : 0
        
        if lastCurves != edits.curves {
            lastCurves = edits.curves
            updateCurves(curves: edits.curves)
        }
    }
    
    // MARK: - Hit-Testing
    public func normalizedTextureCoordinates(atNDC pt: CGPoint) -> CGPoint? {
        let x = Float(pt.x)
        let y = Float(pt.y)
        let (left, right, top, bottom) = lastQuadBounds
        guard x >= left, x <= right, y >= bottom, y <= top else { return nil }
        
        let uQuad = (x - left) / max(right - left, 1e-6)
        let vQuad = (top - y) / max(top - bottom, 1e-6)
        
        let (tl, tr, bl, br) = lastQuadUVs
        let uvTop = (1.0 - uQuad) * tl + uQuad * tr
        let uvBottom = (1.0 - uQuad) * bl + uQuad * br
        let uv = (1.0 - vQuad) * uvTop + vQuad * uvBottom
        
        return CGPoint(x: CGFloat(min(max(uv.x, 0.0), 1.0)), y: CGFloat(min(max(uv.y, 0.0), 1.0)))
    }
}
