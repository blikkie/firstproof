import SwiftUI
import FirstProofBridge

public struct EmulsionWorkspaceView: View {
    @Environment(AppViewModel.self) private var viewModel
    
    public init() {}
    
    private var isMonochromeActive: Bool {
        switch viewModel.printMedia {
        case .barytaSilverGelatin, .warmtoneChlorobromide, .platinumPalladium, .cyanotype:
            return true
        default:
            return false
        }
    }
    
    private var hasActiveEdits: Bool {
        viewModel.printMedia != .none ||
        viewModel.enlargerFilter != .neutralPanchromatic ||
        viewModel.printPaperGrade != 0.0 ||
        viewModel.printBaseWarmth != 0.0 ||
        viewModel.printProcessDepth != 100.0
    }
    
    public var body: some View {
        @Bindable var viewModel = viewModel
        VStack(alignment: .leading, spacing: 14) {
            // Header
            HStack {
                Text("DARKROOM PRINT MEDIA")
                    .font(.caption)
                    .fontWeight(.bold)
                    .foregroundColor(.secondary)
                Spacer()
                Button(action: {
                    viewModel.resetPrintMedia()
                }) {
                    Image(systemName: "arrow.counterclockwise")
                        .font(.system(size: 11, weight: .semibold))
                }
                .buttonStyle(.bordered)
                .controlSize(.small)
                .disabled(!hasActiveEdits)
                .help("Reset Darkroom Print Media to Digital Passthrough")
            }
            
            // Print Media Archetype Selector
            VStack(alignment: .leading, spacing: 6) {
                Text("Print Medium Archetype")
                    .font(.caption)
                    .foregroundColor(.secondary)
                
                Picker("", selection: Binding<PrintMediaMode>(
                    get: { viewModel.printMedia },
                    set: { viewModel.setPrintMedia($0) }
                )) {
                    Text("None (Clean Digital Display)").tag(PrintMediaMode.none)
                    
                    Divider()
                    
                    Section("Color Print Media") {
                        Text("Gallery Chromogenic Type-C (RA-4)").tag(PrintMediaMode.chromogenicTypeC)
                        Text("Silver Dye-Bleach Classic (Azo Reversal)").tag(PrintMediaMode.silverDyeBleach)
                        Text("Archival Fine-Art Cotton Rag (Mineral)").tag(PrintMediaMode.fineArtCottonRag)
                    }
                    
                    Section("Monochrome Media & Archival Chemistry") {
                        Text("Baryta Silver Gelatin FB").tag(PrintMediaMode.barytaSilverGelatin)
                        Text("Warmtone Chlorobromide (Selenium Toned)").tag(PrintMediaMode.warmtoneChlorobromide)
                        Text("Platinum / Palladium (Hand-Coated Rag)").tag(PrintMediaMode.platinumPalladium)
                        Text("Historic Cyanotype (Prussian Blue)").tag(PrintMediaMode.cyanotype)
                    }
                }
                .labelsHidden()
                .pickerStyle(.menu)
            }
            
            // Process Summary Description
            if viewModel.printMedia != .none {
                HStack(spacing: 8) {
                    Image(systemName: isMonochromeActive ? "moon.stars.fill" : "paintpalette.fill")
                        .font(.system(size: 11))
                        .foregroundColor(.accentColor)
                    Text(archetypeDescription(for: viewModel.printMedia))
                        .font(.caption2)
                        .foregroundColor(.secondary)
                        .fixedSize(horizontal: false, vertical: true)
                }
                .padding(8)
                .background(Color(NSColor.controlBackgroundColor).opacity(0.6))
                .cornerRadius(6)
            }
            
            // Optical Enlarger Spectral Filtration (B&W Conversion)
            if isMonochromeActive {
                Divider()
                    .padding(.vertical, 2)
                
                VStack(alignment: .leading, spacing: 6) {
                    HStack {
                        Text("ENLARGER SPECTRAL FILTRATION")
                            .font(.system(size: 10, weight: .bold))
                            .foregroundColor(.secondary)
                        Spacer()
                    }
                    
                    Picker("", selection: Binding<EnlargerFilterMode>(
                        get: { viewModel.enlargerFilter },
                        set: { viewModel.setEnlargerFilter($0) }
                    )) {
                        Text("Neutral Panchromatic (Standard)").tag(EnlargerFilterMode.neutralPanchromatic)
                        Text("Yellow Filter (K2) — Subtle Contrast & Sky").tag(EnlargerFilterMode.yellowK2)
                        Text("Orange Filter (16) — Moderate Contrast").tag(EnlargerFilterMode.orange16)
                        Text("Red Filter (25A) — High Drama & Deep Skies").tag(EnlargerFilterMode.red25A)
                        Text("Green Filter (X1) — Foliage & Flora Separation").tag(EnlargerFilterMode.greenX1)
                        Text("Orthochromatic (Vintage) — Blue/Green Sensitive").tag(EnlargerFilterMode.orthochromatic)
                    }
                    .labelsHidden()
                    .pickerStyle(.menu)
                    
                    Text("Physical optical glass filtration placed in the enlarger light path.")
                        .font(.system(size: 10))
                        .foregroundColor(.secondary.opacity(0.8))
                }
            }
            
            // The Rule of Three Darkroom Controls
            if viewModel.printMedia != .none {
                Divider()
                    .padding(.vertical, 2)
                
                Text("DARKROOM ADJUSTMENTS")
                    .font(.system(size: 10, weight: .bold))
                    .foregroundColor(.secondary)
                
                // 1. Paper Grade / Hardness
                VStack(alignment: .leading, spacing: 2) {
                    SliderRow(
                        label: "Paper Grade",
                        value: $viewModel.printPaperGrade,
                        range: -2.0...2.0,
                        format: "%+.1f",
                        onReset: { viewModel.resetSlider(.printPaperGrade) },
                        onEditingChanged: { viewModel.handleSliderEditing($0, slider: .printPaperGrade) }
                    )
                    HStack {
                        Text("Grade 0 (Soft)")
                        Spacer()
                        Text("Grade 2 (Norm)")
                        Spacer()
                        Text("Grade 4 (Hard)")
                    }
                    .font(.system(size: 9))
                    .foregroundColor(.secondary.opacity(0.7))
                }
                
                // 2. Base Warmth
                VStack(alignment: .leading, spacing: 2) {
                    SliderRow(
                        label: "Base Warmth",
                        value: $viewModel.printBaseWarmth,
                        range: -100.0...100.0,
                        format: "%+.0f",
                        onReset: { viewModel.resetSlider(.printBaseWarmth) },
                        onEditingChanged: { viewModel.handleSliderEditing($0, slider: .printBaseWarmth) }
                    )
                    HStack {
                        Text("Baryta Optical")
                        Spacer()
                        Text("Neutral")
                        Spacer()
                        Text("Natural Cotton")
                    }
                    .font(.system(size: 9))
                    .foregroundColor(.secondary.opacity(0.7))
                }
                
                // 3. Process Depth
                VStack(alignment: .leading, spacing: 2) {
                    SliderRow(
                        label: "Process Depth",
                        value: $viewModel.printProcessDepth,
                        range: 0.0...100.0,
                        format: "%.0f%%",
                        onReset: { viewModel.resetSlider(.printProcessDepth) },
                        onEditingChanged: { viewModel.handleSliderEditing($0, slider: .printProcessDepth) }
                    )
                    HStack {
                        Text(isMonochromeActive ? "Untuned Silver" : "Base Dye")
                        Spacer()
                        Text(isMonochromeActive ? "Archival Toner" : "Full Coupling")
                    }
                    .font(.system(size: 9))
                    .foregroundColor(.secondary.opacity(0.7))
                }
            } else {
                // Passthrough state callout
                VStack(alignment: .leading, spacing: 8) {
                    Text("Pure Digital Capture Passthrough")
                        .font(.caption)
                        .fontWeight(.medium)
                        .foregroundColor(.primary)
                    Text("Select an authentic photographic print medium above to emulate chemical darkroom paper sensitometry, optical enlarger filtration, and archival chemical toners.")
                        .font(.caption2)
                        .foregroundColor(.secondary)
                        .fixedSize(horizontal: false, vertical: true)
                }
                .padding(12)
                .frame(maxWidth: .infinity, alignment: .leading)
                .background(Color(NSColor.controlBackgroundColor).opacity(0.4))
                .cornerRadius(8)
            }
        }
        .padding(.top, 4)
    }
    
    private func archetypeDescription(for media: PrintMediaMode) -> String {
        switch media {
        case .none:
            return "Clean digital display rendering."
        case .chromogenicTypeC:
            return "Classic gallery C-print standard (RA-4). The historical baseline for modern color expectations—intentionally subtle by default, with creamy highlight roll-off and organic CMY dye weight. Shape character with Paper Grade and Warmth."
        case .silverDyeBleach:
            return "High-gloss reversal print with pure azo dyes. Ultra-deep shadow density (Dmax > 3.0), punchy micro-contrast, and glowing gem tones."
        case .fineArtCottonRag:
            return "100% unbleached cotton base with mineral pigment. Velvety matte shadow floor (Dmax ≈ 1.65), natural paper warmth, and glare-free breadth."
        case .barytaSilverGelatin:
            return "Neutral barium sulfate (BaSO4) fiber base. Deep Dmax ≈ 2.2, crisp midtone silver separation, and pure neutral silver blacks."
        case .warmtoneChlorobromide:
            return "Warm fiber base chlorobromide with archival selenium toning. Cool plum-black shadow depths, warm charcoal midtones, and delicate highlights."
        case .platinumPalladium:
            return "Noble metal hand-coated on cotton rag. Delicate long midtone graduation, soft non-clipping highlights, and matte charcoal Dmax ≈ 1.6."
        case .cyanotype:
            return "1842 Sir John Herschel Prussian blue iron-salt process. Rich cyan-blue shadow density and slight warm ivory highlight base."
        }
    }
}
