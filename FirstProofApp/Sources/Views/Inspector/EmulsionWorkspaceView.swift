import SwiftUI
import FirstProofBridge

public struct EmulsionWorkspaceView: View {
    @Environment(AppViewModel.self) private var viewModel
    
    public init() {}
    
    private var hasActiveEmulsionEdits: Bool {
        viewModel.emulsionArchetype != .none ||
        viewModel.remjetBacking != .intact ||
        viewModel.emulsionGrainMode != .off ||
        viewModel.grainScale != 1.0
    }
    
    private var isMonochromeActive: Bool {
        switch viewModel.printMedia {
        case .barytaSilverGelatin, .warmtoneChlorobromide, .platinumPalladium, .cyanotype, .bamakoSilverContact:
            return true
        default:
            return viewModel.emulsionArchetype == .mobberleySilver400 ||
                   viewModel.emulsionArchetype == .liguriaCinema80 ||
                   viewModel.emulsionArchetype == .baodingSilver100
        }
    }
    
    private var hasActivePrintEdits: Bool {
        viewModel.printMedia != .none ||
        viewModel.enlargerFilter != .neutralPanchromatic ||
        viewModel.printPaperGrade != 0.0 ||
        viewModel.printProcessDepth != 100.0
    }
    
    public var body: some View {
        @Bindable var viewModel = viewModel
        VStack(alignment: .leading, spacing: 14) {
            // MARK: - Section 1: Upstream Recording Emulsion
            VStack(alignment: .leading, spacing: 10) {
                HStack {
                    Text("RECORDING EMULSION")
                        .font(.caption)
                        .fontWeight(.bold)
                        .foregroundColor(.secondary)
                    Spacer()
                    Button(action: {
                        viewModel.resetEmulsion()
                    }) {
                        Image(systemName: "arrow.counterclockwise")
                            .font(.system(size: 11, weight: .semibold))
                    }
                    .buttonStyle(.bordered)
                    .controlSize(.small)
                    .disabled(!hasActiveEmulsionEdits)
                    .help("Reset Recording Emulsion to Linear Sensor Raw")
                }
                
                // Emulsion Stock Picker
                VStack(alignment: .leading, spacing: 6) {
                    Text("Film Stock Archetype")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    
                    Picker("", selection: Binding<EmulsionArchetype>(
                        get: { viewModel.emulsionArchetype },
                        set: { viewModel.setEmulsionArchetype($0) }
                    )) {
                        Text("None (Linear Sensor Raw)").tag(EmulsionArchetype.none)
                        
                        Divider()
                        
                        Section("Color Negative (C-41 & Cinema)") {
                            Text("Rochester Portrait 400 (C-41)").tag(EmulsionArchetype.rochesterPortrait400)
                            Text("Leverkusen Portrait 160 (C-41)").tag(EmulsionArchetype.leverkusenPortrait160)
                            Text("Ashigara Portrait 400 (4th-Layer C-41)").tag(EmulsionArchetype.ashigaraPortrait400)
                            Text("Rochester Cinema 500 (ECN-2)").tag(EmulsionArchetype.rochesterCinema500)
                            Text("Rochester Daylight 200 (C-41)").tag(EmulsionArchetype.rochesterDaylight200)
                        }
                        
                        Section("Color Reversal / Slide (E-6 & K-14)") {
                            Text("Ashigara Chrome 50 (E-6)").tag(EmulsionArchetype.ashigaraChrome50)
                            Text("Rochester K-14 Chrome 64").tag(EmulsionArchetype.rochesterK14Chrome64)
                        }
                        
                        Section("Monochrome Silver & Diffusion") {
                            Text("Mobberley Silver 400 (Panchromatic)").tag(EmulsionArchetype.mobberleySilver400)
                            Text("Liguria Cinema 80 (Cinema Monochrome)").tag(EmulsionArchetype.liguriaCinema80)
                            Text("Baoding Silver 100 (Halation Bloom)").tag(EmulsionArchetype.baodingSilver100)
                            Text("Cambridge Diffusion 100 (Instant)").tag(EmulsionArchetype.cambridgeDiffusion100)
                        }
                    }
                    .labelsHidden()
                    .pickerStyle(.menu)
                }
                
                // Emulsion Info & Controls when active
                if viewModel.emulsionArchetype != .none {
                    HStack(alignment: .top, spacing: 8) {
                        Image(systemName: "film")
                            .font(.system(size: 11))
                            .foregroundColor(.accentColor)
                            .padding(.top, 1)
                        Text(emulsionDescription(for: viewModel.emulsionArchetype))
                            .font(.caption2)
                            .foregroundColor(.secondary)
                            .lineLimit(nil)
                            .fixedSize(horizontal: false, vertical: true)
                    }
                    .padding(8)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .background(Color(NSColor.controlBackgroundColor).opacity(0.6))
                    .cornerRadius(6)
                    
                    // Film Format / Instant Print Format
                    VStack(alignment: .leading, spacing: 4) {
                        Text(viewModel.emulsionArchetype == .cambridgeDiffusion100 ? "Instant Print Format" : "Film Format Gate")
                            .font(.system(size: 10, weight: .medium))
                            .foregroundColor(.secondary)
                        
                        if viewModel.emulsionArchetype == .cambridgeDiffusion100 {
                            Picker("", selection: Binding<EmulsionFormat>(
                                get: { viewModel.emulsionFormat },
                                set: { viewModel.setEmulsionFormat($0) }
                            )) {
                                Text("Standard Frame (SX-70 / 600 / Packfilm)").tag(EmulsionFormat.format35mm)
                                Text("4x5 Large Format Sheet (Type 59)").tag(EmulsionFormat.format120)
                                Text("8x10 Studio Sheet (Type 809)").tag(EmulsionFormat.format4x5)
                            }
                            .labelsHidden()
                            .pickerStyle(.menu)
                        } else {
                            Picker("", selection: Binding<EmulsionFormat>(
                                get: { viewModel.emulsionFormat },
                                set: { viewModel.setEmulsionFormat($0) }
                            )) {
                                Text("Super 8 (7.3x Magnification)").tag(EmulsionFormat.super8)
                                Text("16mm Cinema (3.4x Magnification)").tag(EmulsionFormat.format16mm)
                                Text("35mm Still Frame (1.0x Baseline)").tag(EmulsionFormat.format35mm)
                                Text("120 Medium Format / 6x7 (0.48x)").tag(EmulsionFormat.format120)
                                Text("4x5 Large Format Sheet (0.28x)").tag(EmulsionFormat.format4x5)
                            }
                            .labelsHidden()
                            .pickerStyle(.menu)
                        }
                    }
                    
                    // Carbon Rem-Jet Anti-Halation Layer (Rochester Cinema 500)
                    if viewModel.emulsionArchetype == .rochesterCinema500 {
                        VStack(alignment: .leading, spacing: 4) {
                            Text("Carbon Rem-Jet Backing")
                                .font(.system(size: 10, weight: .medium))
                                .foregroundColor(.secondary)
                            
                            Picker("", selection: Binding<RemjetBackingMode>(
                                get: { viewModel.remjetBacking },
                                set: { viewModel.setRemjetBacking($0) }
                            )) {
                                Text("Intact (Studio ECN-2)").tag(RemjetBackingMode.intact)
                                Text("Stripped (Halation Scatter)").tag(RemjetBackingMode.stripped)
                            }
                            .labelsHidden()
                            .pickerStyle(.segmented)
                        }
                    }
                }
                
                // Physical Optical Grain & Stochastic Transmutation
                VStack(alignment: .leading, spacing: 6) {
                    Text("Optical Grain Resynthesis")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    
                    Picker("", selection: Binding<EmulsionGrainMode>(
                        get: { viewModel.emulsionGrainMode },
                        set: { viewModel.setEmulsionGrainMode($0) }
                    )) {
                        Text("Off (Sensor Passthrough)").tag(EmulsionGrainMode.off)
                        Text("Optical Transmutation (Sensor Seeded)").tag(EmulsionGrainMode.seedFromSensor)
                        Text("Synthetic Granularity (Procedural)").tag(EmulsionGrainMode.synthesizePhysical)
                    }
                    .labelsHidden()
                    .pickerStyle(.menu)
                }
                
                if viewModel.emulsionGrainMode != .off {
                    SliderRow(
                        label: "Grain Scale",
                        value: $viewModel.grainScale,
                        range: 0.5...2.0,
                        format: "%.2fx",
                        onReset: { viewModel.resetSlider(.grainScale) },
                        onEditingChanged: { viewModel.handleSliderEditing($0, slider: .grainScale) }
                    )
                }
            }
            
            Divider()
                .padding(.vertical, 2)
            
            // MARK: - Section 2: Darkroom Print Media
            VStack(alignment: .leading, spacing: 10) {
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
                    .disabled(!hasActivePrintEdits)
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
                            Text("Bamako Silver Contact (Ferrotype)").tag(PrintMediaMode.bamakoSilverContact)
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
                        Text(printArchetypeDescription(for: viewModel.printMedia))
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
                    
                    // 2. Process Depth
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
        }
        .padding(.top, 4)
    }
    
    private func emulsionDescription(for archetype: EmulsionArchetype) -> String {
        switch archetype {
        case .none:
            return "Linear sensor raw passthrough without film sensitometry."
        case .rochesterPortrait400:
            return "C-41 Color Negative: Gentle S-curve shoulder, warm skin tone fidelity, active DIR cyan-boosted shadow contrast, and pastel highlight separation."
        case .ashigaraChrome50:
            return "E-6 Color Reversal: High saturation, steep D-log H midtones, deep Dmax, and clean neutral shadows."
        case .rochesterK14Chrome64:
            return "K-14 Non-Substantive Reversal: Iconic warm-golden highlight cast, high optical acutance, and punchy archival shadow density."
        case .rochesterCinema500:
            return "ECN-2 Cinema Negative: Soft highlight shoulder, extreme 14+ stop latitude, and low-contrast linear grading foundation."
        case .rochesterDaylight200:
            return "C-41 Golden Daylight Negative: Clean, balanced primary color fidelity, golden daylight warmth, and restrained blue density."
        case .mobberleySilver400:
            return "Panchromatic Silver Halide: Pure monochromatic collapse in linear Rec.2020, linear midtone acutance, and rich shadow micro-contrast."
        case .cambridgeDiffusion100:
            return "Instant Dye Diffusion Reversal: Instant peel-apart chemistry, soft highlight roll-off, reflective TiO2 base tone, and pastel greens."
        case .leverkusenPortrait160:
            return "C-41 Color Negative: Earthy ochre skin tones, restrained saturation, pastel greens, and gentle highlight roll-off."
        case .ashigaraPortrait400:
            return "4th-Layer C-41 Color Negative: Patented cyan-sensitive inter-layer suppressing fluorescent green casts, airy pastels, and luminous skin tones."
        case .liguriaCinema80:
            return "1960s Neorealist Cinema Monochrome: Pure metallic silver high-acutance emulsion with deep Dmax and crisp edge acutance."
        case .baodingSilver100:
            return "Cubic Silver Halide Monochrome: Authentic domestic Chinese formulation with distinct specular halation bloom and gritty micro-contrast."
        }
    }
    
    private func printArchetypeDescription(for media: PrintMediaMode) -> String {
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
        case .bamakoSilverContact:
            return "West African studio contact printing paper (Malick Sidibé / Seydou Keïta). High-silver emulsion ferrotyped onto glass: deep rich Zone III–V shadow separation on deep skin tones, crisp high-key textile acutance, and neutral silver blacks."
        }
    }
}
