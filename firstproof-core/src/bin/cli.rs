//! First Proof Darkroom CLI (`firstproof-cli`)
//!
//! A fast, headless command-line darkroom engine for developing camera RAWs,
//! inspecting sensor calibration, and verifying optical color science.
//!
//! "Pure capture. Honest color. Zero digital plastic."

use std::env;
use std::path::{Path, PathBuf};
use std::time::Instant;

use firstproof_core::export::export_photo_file_advanced;
use firstproof_core::lensfun::LensfunDatabase;
use firstproof_core::raw::extract_metadata;
use firstproof_core::xmp::read_xmp_file;
use firstproof_core::{
    ColorProfileMode, DemosaicQuality, EditSettings, EmulsionArchetype, EmulsionFormat,
    EmulsionGrainMode, EnlargerFilterMode, ExportColorSpace, ExportRenderingIntent, PrintMediaMode,
    RenderTier,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_main_help() {
    println!(
        r#"First Proof Darkroom CLI v{}
"Pure capture. Honest color. Zero digital plastic."

USAGE:
    firstproof-cli <SUBCOMMAND> [OPTIONS]

SUBCOMMANDS:
    develop    Develop a RAW or image file to 16-bit TIFF or JPEG
    inspect    Inspect camera EXIF, sensor CFA calibration, and XMP edits
    dng        Convert proprietary RAW (.CR2, .NEF, .ARW) to archival DNG
    command    Execute canonical DarkroomCommand JSON headlessly or via socket
    shaders    Transpile canonical WGSL darkroom shaders to Metal and WebGL 2
    help       Print this help message or the help of the given subcommand

EXAMPLES:
    # Develop a RAW with default AgX Vivid tone mapping to 16-bit TIFF
    firstproof-cli develop photo.cr2

    # Develop with custom exposure, profile, and JPEG output
    firstproof-cli develop photo.cr2 --out preview.jpg --exposure +0.5 --profile agx-vivid

    # Convert Canon CR2 to archival DNG with automatic AgX preview & disk verification
    firstproof-cli dng photo.cr2

    # Fast draft preview rendering
    firstproof-cli develop photo.cr2 --out draft.jpg --fast

    # Inspect camera and sensor metadata
    firstproof-cli inspect photo.cr2
"#,
        VERSION
    );
}

fn print_develop_help() {
    println!(
        r#"firstproof-cli develop
Develop a RAW or image file to 16-bit TIFF or high-quality JPEG.

USAGE:
    firstproof-cli develop <INPUT> [OPTIONS]

ARGS:
    <INPUT>                      Path to camera RAW (.cr2, .dng, .nef, .arw) or image file

OPTIONS:
    -o, --out <PATH>             Output path (defaults to <input_stem>_developed.tiff or .jpg)
    -e, --exposure <EV>          Exposure adjustment in EV (-5.0 to +5.0, default: 0.0)
    -c, --contrast <N>           Contrast adjustment (-100 to +100, default: 0.0)
        --highlights <N>         Highlights recovery (-100 to +100, default: 0.0)
        --shadows <N>            Shadows lift (-100 to +100, default: 0.0)
        --whites <N>             Whites clipping (-100 to +100, default: 0.0)
        --blacks <N>             Blacks clipping (-100 to +100, default: 0.0)
        --vibrance <N>           Vibrance (-100 to +100, default: 0.0)
        --saturation <N>         Saturation (-100 to +100, default: 0.0)
    -t, --temp <KELVIN>          White balance temperature in Kelvin (2000-20000)
        --tint <TINT>            White balance green/magenta tint (-100 to +100)
    -p, --profile <NAME>         Tone profile (default: agx-vivid)
                                 [agx-vivid, agx-neutral, camera-standard,
                                  camera-portrait, camera-landscape, camera-vivid]
    -m, --emulsion <STOCK>       Film stock archetype (default: none)
                                 [rochester-portrait-400, ashigara-chrome-50,
                                  rochester-k14-chrome-64, rochester-cinema-500,
                                  cambridge-diffusion-100, mobberley-silver-400,
                                  leverkusen-portrait-160, ashigara-portrait-400,
                                  liguria-cinema-80, baoding-silver-100]
        --format <GATE>          Film format gate: 35mm (default), 120, 4x5, super8, 16mm
        --grain <MODE>           Optical grain mode: off (default), sensor, physical
        --grain-scale <SCALE>    Grain scale multiplier (0.5 to 2.0, default: 1.0)
        --print-media <MEDIA>    Print media: baryta, warmtone, platinum, cyanotype,
                                 bamako, or none (default)
        --filter <COLOR>         Enlarger optical filter: neutral, yellow, orange, red, green, ortho
        --fast                   Fast standard demosaicing tier
        --quality <1-100>        JPEG quality (1-100, default: 95)
        --max-dim <PIXELS>       Constraint long-edge maximum dimension
        --color-space <SPACE>    Output color space: srgb (default), display-p3
        --xmp <PATH>             Explicit XMP sidecar path (auto-detects <INPUT>.xmp)
    -q, --quiet                  Suppress non-error log messages
    -h, --help                   Print help information
"#
    );
}

fn parse_color_profile(s: &str) -> Option<ColorProfileMode> {
    match s.to_lowercase().replace(['-', '_'], "").as_str() {
        "agxvivid" | "vivid" => Some(ColorProfileMode::AgxVivid),
        "agxneutral" | "agxfilmic" | "neutral" | "filmic" => Some(ColorProfileMode::AgxFilmic),
        "agxphotographic" | "photographic" | "photo" => Some(ColorProfileMode::AgxPhotographic),
        "camerastandard" | "standard" => Some(ColorProfileMode::CameraStandard),
        "cameraportrait" | "portrait" => Some(ColorProfileMode::CameraPortrait),
        "cameralandscape" | "landscape" => Some(ColorProfileMode::CameraLandscape),
        "cameravivid" => Some(ColorProfileMode::CameraVivid),
        _ => None,
    }
}

fn parse_emulsion_archetype(s: &str) -> Option<EmulsionArchetype> {
    match s.to_lowercase().replace(['-', '_'], "").as_str() {
        "none" | "bypass" | "off" => Some(EmulsionArchetype::None),
        "rochesterportrait400" | "portra400" | "portra" => {
            Some(EmulsionArchetype::RochesterPortrait400)
        }
        "ashigarachrome50" | "velvia50" | "velvia" => Some(EmulsionArchetype::AshigaraChrome50),
        "rochesterk14chrome64" | "kodachrome64" | "kodachrome" => {
            Some(EmulsionArchetype::RochesterK14Chrome64)
        }
        "rochestercinema500" | "vision3" | "500t" => Some(EmulsionArchetype::RochesterCinema500),
        "rochesterdaylight200" | "gold200" => Some(EmulsionArchetype::RochesterDaylight200),
        "mobberleysilver400" | "hp5" | "hp5plus" => Some(EmulsionArchetype::MobberleySilver400),
        "cambridgediffusion100" | "polaroid" | "sx70" => {
            Some(EmulsionArchetype::CambridgeDiffusion100)
        }
        "leverkusenportrait160" | "agfacolor" => Some(EmulsionArchetype::LeverkusenPortrait160),
        "ashigaraportrait400" | "pro400h" => Some(EmulsionArchetype::AshigaraPortrait400),
        "liguriacinema80" | "ferrania" | "p30" => Some(EmulsionArchetype::LiguriaCinema80),
        "baodingsilver100" | "lucky" | "shd100" => Some(EmulsionArchetype::BaodingSilver100),
        _ => None,
    }
}

fn parse_emulsion_format(s: &str) -> Option<EmulsionFormat> {
    match s.to_lowercase().replace(['-', '_'], "").as_str() {
        "35mm" | "format35mm" | "standard" => Some(EmulsionFormat::Format35mm),
        "120" | "format120" | "medium" | "6x7" => Some(EmulsionFormat::Format120),
        "4x5" | "format4x5" | "large" | "sheet" => Some(EmulsionFormat::Format4x5),
        "super8" | "s8" => Some(EmulsionFormat::Super8),
        "16mm" | "format16mm" => Some(EmulsionFormat::Format16mm),
        _ => None,
    }
}

fn parse_emulsion_grain_mode(s: &str) -> Option<EmulsionGrainMode> {
    match s.to_lowercase().as_str() {
        "off" | "none" => Some(EmulsionGrainMode::Off),
        "sensor" | "seed" | "seedfromsensor" => Some(EmulsionGrainMode::SeedFromSensor),
        "physical" | "synthesize" | "synthesizephysical" => {
            Some(EmulsionGrainMode::SynthesizePhysical)
        }
        _ => None,
    }
}

fn parse_print_media(s: &str) -> Option<PrintMediaMode> {
    match s.to_lowercase().replace(['-', '_'], "").as_str() {
        "none" | "off" => Some(PrintMediaMode::None),
        "baryta" | "barytasilvergelatin" => Some(PrintMediaMode::BarytaSilverGelatin),
        "warmtone" | "warmtonechlorobromide" => Some(PrintMediaMode::WarmtoneChlorobromide),
        "platinum" | "platinumpalladium" => Some(PrintMediaMode::PlatinumPalladium),
        "cyanotype" => Some(PrintMediaMode::Cyanotype),
        "bamako" | "bamakosilvercontact" => Some(PrintMediaMode::BamakoSilverContact),
        _ => None,
    }
}

fn parse_enlarger_filter(s: &str) -> Option<EnlargerFilterMode> {
    match s.to_lowercase().as_str() {
        "neutral" | "panchromatic" => Some(EnlargerFilterMode::NeutralPanchromatic),
        "yellow" | "k2" => Some(EnlargerFilterMode::YellowK2),
        "orange" | "16" => Some(EnlargerFilterMode::Orange16),
        "red" | "25a" => Some(EnlargerFilterMode::Red25A),
        "green" | "x1" => Some(EnlargerFilterMode::GreenX1),
        "ortho" | "orthochromatic" => Some(EnlargerFilterMode::Orthochromatic),
        _ => None,
    }
}

fn cmd_develop(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args[0] == "-h" || args[0] == "--help" {
        print_develop_help();
        return Ok(());
    }

    let input_path = PathBuf::from(&args[0]);
    if !input_path.exists() {
        return Err(format!("Input file not found: {}", input_path.display()));
    }

    let mut output_path: Option<PathBuf> = None;
    let mut explicit_xmp_path: Option<PathBuf> = None;
    let mut is_fast = false;
    let mut jpeg_quality: u8 = 95;
    let mut max_dimension: Option<u32> = None;
    let mut color_space = ExportColorSpace::Srgb;
    let mut is_quiet = false;

    // Start with default edits
    let mut edits = EditSettings::default();

    // Auto-detect adjacent .xmp or input.xmp
    let sidecar_candidates = [
        input_path.with_extension(format!(
            "{}.xmp",
            input_path.extension().and_then(|e| e.to_str()).unwrap_or("")
        )),
        input_path.with_extension("xmp"),
    ];

    let mut loaded_sidecar = false;
    for sc in &sidecar_candidates {
        if sc.exists() {
            if let Ok(xmp) = read_xmp_file(sc) {
                edits = xmp.edits;
                loaded_sidecar = true;
                break;
            }
        }
    }

    // Parse options
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--out" => {
                i += 1;
                if i >= args.len() {
                    return Err("--out requires an argument".into());
                }
                output_path = Some(PathBuf::from(&args[i]));
            }
            "-e" | "--exposure" => {
                i += 1;
                if i >= args.len() {
                    return Err("--exposure requires an argument".into());
                }
                edits.exposure = args[i]
                    .parse::<f32>()
                    .map_err(|_| "Invalid exposure value")?;
            }
            "-c" | "--contrast" => {
                i += 1;
                if i >= args.len() {
                    return Err("--contrast requires an argument".into());
                }
                edits.contrast = args[i]
                    .parse::<f32>()
                    .map_err(|_| "Invalid contrast value")?;
            }
            "--highlights" => {
                i += 1;
                if i >= args.len() {
                    return Err("--highlights requires an argument".into());
                }
                edits.highlights = args[i]
                    .parse::<f32>()
                    .map_err(|_| "Invalid highlights value")?;
            }
            "--shadows" => {
                i += 1;
                if i >= args.len() {
                    return Err("--shadows requires an argument".into());
                }
                edits.shadows = args[i]
                    .parse::<f32>()
                    .map_err(|_| "Invalid shadows value")?;
            }
            "--whites" => {
                i += 1;
                if i >= args.len() {
                    return Err("--whites requires an argument".into());
                }
                edits.whites = args[i].parse::<f32>().map_err(|_| "Invalid whites value")?;
            }
            "--blacks" => {
                i += 1;
                if i >= args.len() {
                    return Err("--blacks requires an argument".into());
                }
                edits.blacks = args[i].parse::<f32>().map_err(|_| "Invalid blacks value")?;
            }
            "--vibrance" => {
                i += 1;
                if i >= args.len() {
                    return Err("--vibrance requires an argument".into());
                }
                edits.vibrance = args[i]
                    .parse::<f32>()
                    .map_err(|_| "Invalid vibrance value")?;
            }
            "--saturation" => {
                i += 1;
                if i >= args.len() {
                    return Err("--saturation requires an argument".into());
                }
                edits.saturation = args[i]
                    .parse::<f32>()
                    .map_err(|_| "Invalid saturation value")?;
            }
            "-t" | "--temp" | "--temperature" => {
                i += 1;
                if i >= args.len() {
                    return Err("--temp requires an argument".into());
                }
                edits.temperature = args[i]
                    .parse::<f32>()
                    .map_err(|_| "Invalid temperature value")?;
            }
            "--tint" => {
                i += 1;
                if i >= args.len() {
                    return Err("--tint requires an argument".into());
                }
                edits.tint = args[i].parse::<f32>().map_err(|_| "Invalid tint value")?;
            }
            "-p" | "--profile" => {
                i += 1;
                if i >= args.len() {
                    return Err("--profile requires an argument".into());
                }
                edits.tone_profile = parse_color_profile(&args[i])
                    .ok_or_else(|| format!("Unknown profile: {}", args[i]))?;
            }
            "-m" | "--emulsion" => {
                i += 1;
                if i >= args.len() {
                    return Err("--emulsion requires an argument".into());
                }
                edits.emulsion_archetype = parse_emulsion_archetype(&args[i])
                    .ok_or_else(|| format!("Unknown emulsion stock: {}", args[i]))?;
            }
            "--format" => {
                i += 1;
                if i >= args.len() {
                    return Err("--format requires an argument".into());
                }
                edits.emulsion_format = parse_emulsion_format(&args[i])
                    .ok_or_else(|| format!("Unknown format gate: {}", args[i]))?;
            }
            "--grain" => {
                i += 1;
                if i >= args.len() {
                    return Err("--grain requires an argument".into());
                }
                edits.emulsion_grain_mode = parse_emulsion_grain_mode(&args[i])
                    .ok_or_else(|| format!("Unknown grain mode: {}", args[i]))?;
            }
            "--grain-scale" => {
                i += 1;
                if i >= args.len() {
                    return Err("--grain-scale requires an argument".into());
                }
                edits.grain_scale = args[i]
                    .parse::<f32>()
                    .map_err(|_| "Invalid grain scale")?;
            }
            "--print-media" => {
                i += 1;
                if i >= args.len() {
                    return Err("--print-media requires an argument".into());
                }
                edits.print_media = parse_print_media(&args[i])
                    .ok_or_else(|| format!("Unknown print media: {}", args[i]))?;
            }
            "--filter" => {
                i += 1;
                if i >= args.len() {
                    return Err("--filter requires an argument".into());
                }
                edits.enlarger_filter = parse_enlarger_filter(&args[i])
                    .ok_or_else(|| format!("Unknown enlarger filter: {}", args[i]))?;
            }
            "--fast" | "--preview" => {
                is_fast = true;
            }
            "--quality" => {
                i += 1;
                if i >= args.len() {
                    return Err("--quality requires an argument".into());
                }
                jpeg_quality = args[i]
                    .parse::<u8>()
                    .map_err(|_| "Invalid quality (1-100)")?
                    .clamp(1, 100);
            }
            "--max-dim" => {
                i += 1;
                if i >= args.len() {
                    return Err("--max-dim requires an argument".into());
                }
                max_dimension = Some(
                    args[i]
                        .parse::<u32>()
                        .map_err(|_| "Invalid max-dim integer")?,
                );
            }
            "--color-space" => {
                i += 1;
                if i >= args.len() {
                    return Err("--color-space requires an argument".into());
                }
                color_space = match args[i].to_lowercase().as_str() {
                    "srgb" => ExportColorSpace::Srgb,
                    "display-p3" | "p3" => ExportColorSpace::DisplayP3,
                    _ => return Err(format!("Unsupported color space: {}", args[i])),
                };
            }
            "--xmp" => {
                i += 1;
                if i >= args.len() {
                    return Err("--xmp requires an argument".into());
                }
                explicit_xmp_path = Some(PathBuf::from(&args[i]));
            }
            "-q" | "--quiet" => {
                is_quiet = true;
            }
            "-h" | "--help" => {
                print_develop_help();
                return Ok(());
            }
            other => {
                return Err(format!("Unknown option: {}", other));
            }
        }
        i += 1;
    }

    // If explicit XMP path passed, override
    if let Some(ref xp) = explicit_xmp_path {
        let xmp = read_xmp_file(xp)
            .map_err(|e| format!("Failed to read XMP sidecar {}: {}", xp.display(), e))?;
        edits = xmp.edits;
        loaded_sidecar = true;
    }

    // Resolve output path if omitted
    let out = match output_path {
        Some(p) => p,
        None => {
            let stem = input_path.file_stem().unwrap_or_default().to_string_lossy();
            let ext = if is_fast { "jpg" } else { "tiff" };
            input_path.with_file_name(format!("{}_developed.{}", stem, ext))
        }
    };

    let demosaic_quality = if is_fast {
        DemosaicQuality::Standard
    } else {
        DemosaicQuality::Archival
    };

    let render_tier = if is_fast {
        RenderTier::Viewport
    } else {
        RenderTier::ArchivalExport
    };

    if !is_quiet {
        println!("============================================================");
        println!(
            "🎞️  First Proof Darkroom CLI: {}",
            input_path.file_name().unwrap_or_default().to_string_lossy()
        );
        println!("============================================================");
        if loaded_sidecar {
            println!("📑 Loaded edits from XMP sidecar");
        }
        println!(
            "🎨 Tone Profile: {:?} | Exposure: {:+.2} EV | Contrast: {:+.0}",
            edits.tone_profile, edits.exposure, edits.contrast
        );
        if edits.emulsion_archetype != EmulsionArchetype::None {
            println!(
                "🧪 Emulsion: {:?} ({:?})",
                edits.emulsion_archetype, edits.emulsion_format
            );
        }
        if is_fast {
            println!("⚡ Mode: Fast Standard Demosaicing");
        } else {
            println!("🏛️  Mode: RCD Archival Demosaicing (16-bit Master)");
        }
        println!("🎯 Output: {}", out.display());
    }

    let t0 = Instant::now();
    export_photo_file_advanced(
        &input_path,
        &out,
        &edits,
        None,
        jpeg_quality,
        max_dimension,
        color_space,
        ExportRenderingIntent::RelativeColorimetric,
        false,
        demosaic_quality,
        render_tier,
    )?;

    let elapsed = t0.elapsed();
    if !is_quiet {
        let size_str = if let Ok(meta) = std::fs::metadata(&out) {
            format!(" ({:.1} MB)", meta.len() as f64 / 1_048_576.0)
        } else {
            String::new()
        };
        println!(
            "✅ Developed successfully in {:.1}ms{}",
            elapsed.as_secs_f64() * 1000.0,
            size_str
        );
    }

    Ok(())
}

fn cmd_inspect(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args[0] == "-h" || args[0] == "--help" {
        println!("USAGE: firstproof-cli inspect <INPUT_PATH>");
        return Ok(());
    }

    let path = Path::new(&args[0]);
    if !path.exists() {
        return Err(format!("File not found: {}", path.display()));
    }

    println!("============================================================");
    println!(
        "🔍 First Proof Photo Inspector: {}",
        path.file_name().unwrap_or_default().to_string_lossy()
    );
    println!("============================================================");

    // 1. EXIF Metadata
    let meta = extract_metadata(path).ok();
    if let Some(ref m) = meta {
        println!(
            "📷 Camera:     {} {}",
            m.camera_make.as_deref().unwrap_or("Unknown"),
            m.camera_model.as_deref().unwrap_or("Unknown")
        );
        println!(
            "🔭 Lens:       {}",
            m.lens_model.as_deref().unwrap_or("(Unknown)")
        );
        println!(
            "📏 Exposure:   {:.1}mm | f/{:.1} | {}s | ISO {}",
            m.focal_length.unwrap_or(0.0),
            m.aperture.unwrap_or(0.0),
            m.shutter_speed.as_deref().unwrap_or("-"),
            m.iso.unwrap_or(0)
        );
        if let Some(ts) = m.taken_time {
            println!("📅 Timestamp:  {} (Unix epoch)", ts);
        }
    } else {
        println!("⚠️  Unable to extract standard EXIF tags");
    }

    // 2. Raw Sensor CFA Calibration
    if let Ok(frame) = firstproof_core::raw_decoder::RawDecoder::decode(
        &firstproof_core::raw_decoder::RawlerDecoder,
        path,
    ) {
        println!("\n🔬 Sensor & CFA Geometry:");
        println!(
            "   Dimensions: {} x {} (Active: {} x {} at offset {}, {})",
            frame.full_width,
            frame.full_height,
            frame.active_width,
            frame.active_height,
            frame.active_x,
            frame.active_y
        );
        println!("   CFA Mosaic: {:?}", frame.cfa_pattern);
        println!("   Black Levels: {:?}", frame.black_levels);
        println!("   White Levels: {:?}", frame.white_levels);
        println!("   As-Shot WB:   {:?}", frame.as_shot_wb());
        println!(
            "   As-Shot Temp: {:.1} K",
            frame.as_shot_temperature()
        );
        println!(
            "   Baseline Exp: {:.3} EV (gain: {:.3}x)",
            frame.baseline_exposure,
            2.0f32.powf(frame.baseline_exposure)
        );
    }

    // 3. Lensfun Database Lookup
    let ldb = LensfunDatabase::new();
    if let Some(ref m) = meta {
        let make = m.camera_make.as_deref().unwrap_or("");
        let model = m.camera_model.as_deref().unwrap_or("");
        let lens = m.lens_model.as_deref().unwrap_or("");
        let focal = m.focal_length.unwrap_or(0.0);
        let coeffs = ldb.resolve_coefficients(make, model, lens, focal, m.aperture);
        if coeffs.has_distortion || coeffs.has_tca || coeffs.has_vignetting {
            println!("\n🔍 Lens Calibration (Lensfun DB):");
            println!("   Distortion Model: {}", coeffs.distortion_model);
            println!("   Coefficients:     {:?}", coeffs.distortion);
        } else {
            println!("\nℹ️  No Lensfun distortion profile found in database");
        }
    }

    // 4. XMP Sidecar
    match read_xmp_file(path) {
        Ok(xmp) => {
            let e = &xmp.edits;
            println!("\n📑 Active XMP Edits Sidecar:");
            let rating_str = match xmp.rating {
                Some(r) => format!("{} ★", r),
                None => "Unrated".to_string(),
            };
            println!("   Rating:     {}", rating_str);
            println!("   Pick/Reject: {}", xmp.status);
            println!(
                "   White Balance: {:.0} K (Tint: {:+.0})",
                e.temperature, e.tint
            );
            println!(
                "   Exposure: {:+.2} EV | Contrast: {:+.0}",
                e.exposure, e.contrast
            );
            println!(
                "   Tone Recovery: H:{:+.0} S:{:+.0} W:{:+.0} B:{:+.0}",
                e.highlights, e.shadows, e.whites, e.blacks
            );
            println!("   Tone Profile:  {:?}", e.tone_profile);
            if e.emulsion_archetype != EmulsionArchetype::None {
                println!(
                    "   Emulsion:      {:?} ({:?})",
                    e.emulsion_archetype, e.emulsion_format
                );
            }
        }
        Err(_) => {
            println!("\nℹ️  No adjacent XMP sidecar (.xmp) found");
        }
    }

    Ok(())
}

fn print_dng_help() {
    println!(
        r#"firstproof-cli dng
Convert proprietary camera RAW (.CR2, .NEF, .ARW) to archival Adobe DNG 1.4+.

USAGE:
    firstproof-cli dng <INPUT> [OPTIONS]

OPTIONS:
    -o, --out <PATH>         Explicit output DNG path (defaults to <INPUT_STEM>.dng)
        --delete-original    Safely delete original RAW file after verified conversion
        --no-verify          Skip independent disk read-back verification
    -q, --quiet              Suppress non-error log messages
    -h, --help               Print help information

EXAMPLES:
    # Convert Canon CR2 to archival DNG with automatic AgX preview & verification
    firstproof-cli dng photo.cr2

    # Convert with custom output path
    firstproof-cli dng photo.cr2 --out /archive/photo.dng

    # Convert and safely unlink original CR2
    firstproof-cli dng photo.cr2 --delete-original
"#
    );
}

fn cmd_dng(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        print_dng_help();
        return Err("No input RAW file specified".into());
    }

    let mut input_path: Option<PathBuf> = None;
    let mut output_path: Option<PathBuf> = None;
    let mut delete_original = false;
    let mut verify = true;
    let mut is_quiet = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--out" => {
                i += 1;
                if i >= args.len() {
                    return Err("--out requires an argument".into());
                }
                output_path = Some(PathBuf::from(&args[i]));
            }
            "--delete-original" => {
                delete_original = true;
            }
            "--no-verify" => {
                verify = false;
            }
            "-q" | "--quiet" => {
                is_quiet = true;
            }
            "-h" | "--help" => {
                print_dng_help();
                return Ok(());
            }
            arg if !arg.starts_with('-') => {
                if input_path.is_none() {
                    input_path = Some(PathBuf::from(arg));
                } else {
                    return Err(format!("Unexpected argument: {}", arg));
                }
            }
            unknown => {
                return Err(format!("Unknown option: {}", unknown));
            }
        }
        i += 1;
    }

    let input_path = input_path.ok_or_else(|| "No input RAW file specified".to_string())?;

    let options = firstproof_core::dng::DngConvertOptions {
        output_path,
        delete_original,
        verify,
        quiet: is_quiet,
    };

    if !is_quiet {
        println!("============================================================");
        println!(
            "📦 Archival DNG Converter: {}",
            input_path.file_name().unwrap_or_default().to_string_lossy()
        );
        println!("============================================================");
        println!("🔒 Lossless Bayer CFA Compression (JPEG-92)");
        println!("🎨 First Proof AgX Filmic Embedded Preview (Zero Plastic)");
        println!("🛡️  Post-Write Disk Read-Back Verification: {}", if verify { "Active" } else { "Disabled" });
        if delete_original {
            println!("⚠️  Original RAW will be unlinked only after verification succeeds");
        }
    }

    let report = firstproof_core::dng::convert_raw_to_dng(&input_path, &options)?;

    if !is_quiet {
        println!("------------------------------------------------------------");
        println!("✅ Conversion Succeeded in {:.1}ms", report.elapsed_ms);
        println!("📁 Destination: {}", report.dng_path.display());
        println!(
            "💾 Size: {:.2} MB -> {:.2} MB ({:+.1}% space saved)",
            report.source_size_bytes as f64 / 1_048_576.0,
            report.dng_size_bytes as f64 / 1_048_576.0,
            report.compression_ratio
        );
        println!("🔑 CFA SHA-256 (Source): {}", report.source_cfa_sha256);
        println!("🔑 CFA SHA-256 (DNG):    {}", report.disk_cfa_sha256);
        println!("✨ Cryptographic Bit-for-Bit Parity: 100% IDENTICAL");
        if report.deleted_original {
            println!("🗑️  Source file safely unlinked: {}", report.source_path.display());
        }
        if !report.sidecars_migrated.is_empty() {
            for sc in &report.sidecars_migrated {
                println!("📑 Sidecar migrated: {}", sc.display());
            }
        }
        println!("============================================================");
    }

    Ok(())
}

fn cmd_bundle_lensfun(args: &[String]) -> Result<(), String> {
    if args.len() < 2 {
        return Err("Usage: firstproof-cli bundle-lensfun <INPUT_XML_DIR> <OUTPUT_JSON>".to_string());
    }
    let input_dir = Path::new(&args[0]);
    let output_json = Path::new(&args[1]);

    println!("==> Compiling Lensfun database from {:?}...", input_dir);
    let mut db = LensfunDatabase::default();
    let loaded = db.load_directory(input_dir)?;
    println!("    Parsed {} XML files ({} cameras, {} lenses)", loaded, db.cameras.len(), db.lenses.len());

    if let Some(parent) = output_json.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    db.save_json_file(output_json)?;
    let meta = std::fs::metadata(output_json).map_err(|e| e.to_string())?;
    println!("    ✅ Wrote compiled database to {:?} ({:.2} MB)", output_json, meta.len() as f64 / 1_048_576.0);
    Ok(())
}

fn print_shaders_help() {
    println!(
        r#"firstproof-cli shaders
Transpile canonical WGSL darkroom shaders to Metal and WebGL 2 GLSL.

USAGE:
    firstproof-cli shaders [OPTIONS]

OPTIONS:
    --check          Verify on-disk shaders match transpiled output without writing
    --msl <PATH>     Target path for Metal Shading Language output
                     (default: FirstProofApp/Sources/Shaders/Shaders.metal)
    --glsl <PATH>    Target path for WebGL 2 TypeScript GLSL output
                     (default: web/src/shaders/GeneratedGLSL.ts)
    --wgsl <PATH>    Target path for WebGPU TypeScript WGSL output
                     (default: web/src/shaders/CanonicalWGSL.ts)
    -h, --help       Print this help message
"#
    );
}

fn cmd_shaders(args: &[String]) -> Result<(), String> {
    let mut check_only = false;
    let mut msl_path = PathBuf::from("FirstProofApp/Sources/Shaders/Shaders.metal");
    let mut glsl_path = PathBuf::from("web/src/shaders/GeneratedGLSL.ts");
    let mut wgsl_path = PathBuf::from("web/src/shaders/CanonicalWGSL.ts");

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--check" => check_only = true,
            "--msl" if i + 1 < args.len() => {
                msl_path = PathBuf::from(&args[i + 1]);
                i += 1;
            }
            "--glsl" if i + 1 < args.len() => {
                glsl_path = PathBuf::from(&args[i + 1]);
                i += 1;
            }
            "--wgsl" if i + 1 < args.len() => {
                wgsl_path = PathBuf::from(&args[i + 1]);
                i += 1;
            }
            "-h" | "--help" => {
                print_shaders_help();
                return Ok(());
            }
            other => return Err(format!("Unknown option for shaders command: {}", other)),
        }
        i += 1;
    }

    println!("🎨 Transpiling canonical WGSL shaders with Naga...");
    let msl_content = format!(
        "// AUTO-GENERATED from canonical WGSL by firstproof-engine (Story 4.8)\n// DO NOT EDIT DIRECTLY. Edit firstproof-engine/shaders/*.wgsl instead.\n\n{}",
        firstproof_engine::shader::compile_to_msl().map_err(|e| e.to_string())?
    );

    let raw_glsl = firstproof_engine::shader::compile_to_glsl().map_err(|e| e.to_string())?;
    let glsl_content = format!(
        r#"// AUTO-GENERATED from canonical WGSL by firstproof-engine (Story 4.8)
// DO NOT EDIT DIRECTLY. Edit firstproof-engine/shaders/*.wgsl instead.

export const FRAGMENT_SHADER_SOURCE = `{}`;

export const SHADER_UNIFORM_BLOCK_NAME = 'ShaderUniforms_block_0Fragment';
export const SHADER_UNIFORM_SIZE_BYTES = 320;

export const SHADER_SAMPLER_NAMES = {{
  texture: '_group_0_binding_1_fs',
  lut3d: '_group_0_binding_3_fs',
  curveMaster: '_group_0_binding_5_fs',
  curveRgb: '_group_0_binding_6_fs',
  gainMap: '_group_0_binding_8_fs',
}} as const;
"#,
        raw_glsl
    );

    let raw_wgsl = firstproof_engine::shader::canonical_wgsl();
    let wgsl_content = format!(
        r#"// AUTO-GENERATED from canonical WGSL by firstproof-engine (Story 4.8)
// DO NOT EDIT DIRECTLY. Edit firstproof-engine/shaders/*.wgsl instead.

export const CANONICAL_DARKROOM_WGSL = `{}`;
"#,
        raw_wgsl
    );

    let targets = [
        ("MSL (.metal)", &msl_path, &msl_content),
        ("WebGL 2 (GLSL TS)", &glsl_path, &glsl_content),
        ("WebGPU (WGSL TS)", &wgsl_path, &wgsl_content),
    ];

    if check_only {
        for (label, path, expected) in &targets {
            if !path.exists() {
                return Err(format!("Shader check failed: target file {:?} does not exist.", path));
            }
            let actual = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
            if actual != **expected {
                return Err(format!(
                    "Shader check failed: {:?} is out of sync with canonical WGSL. Run 'firstproof-cli shaders' to regenerate.",
                    path
                ));
            }
            println!("  ✅ {} {:?} matches canonical source", label, path);
        }
        println!("✨ All generated shaders are in bit-level sync with canonical WGSL.");
    } else {
        for (label, path, content) in &targets {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            std::fs::write(path, content).map_err(|e| e.to_string())?;
            println!("  ✅ Generated {} -> {:?}", label, path);
        }
        println!("✨ Successfully transpiled canonical WGSL to Metal, WebGL 2 GLSL, and WebGPU WGSL.");
    }

    Ok(())
}

fn print_command_help() {
    println!(
        r#"firstproof-cli command
Execute canonical DarkroomCommand JSON requests headlessly or forward them to a running FirstProofApp instance.

USAGE:
    firstproof-cli command [OPTIONS] [COMMAND_JSON]

ARGS:
    <COMMAND_JSON>               JSON string containing DarkroomCommand (e.g. '{{"command":"get_state"}}')

OPTIONS:
    -c, --connect                Forward command to live AppKit instance via Unix domain socket
        --socket <PATH>          Socket path (default: /tmp/firstproof-debug.sock)
        --stdin                  Read commands line-by-line from standard input
    -h, --help                   Print help information

EXAMPLES:
    # Inspect photo metadata headlessly
    firstproof-cli command '{{"command":"inspect_metadata","path":"photo.cr2"}}'

    # Query live AppKit viewport state
    firstproof-cli command --connect '{{"command":"get_state"}}'

    # Select photo in active desktop app
    firstproof-cli command --connect '{{"command":"select_photo","path":"/path/to/img.cr2"}}'

    # Stream batch commands via stdin
    cat commands.jsonl | firstproof-cli command
"#
    );
}

#[cfg(unix)]
fn send_to_socket(socket_path: &str, json_cmd: &str) -> Result<String, String> {
    use std::io::{Read, Write};
    use std::os::unix::net::UnixStream;
    use std::time::Duration;

    let mut stream = UnixStream::connect(socket_path)
        .map_err(|e| format!("Failed to connect to socket at '{}': {}. Is FirstProofApp running?", socket_path, e))?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| e.to_string())?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| e.to_string())?;

    stream.write_all(json_cmd.as_bytes())
        .map_err(|e| format!("Failed to write to socket: {}", e))?;
    stream.write_all(b"\n")
        .map_err(|e| format!("Failed to write newline to socket: {}", e))?;

    let mut response = String::new();
    stream.read_to_string(&mut response)
        .map_err(|e| format!("Failed to read response from socket: {}", e))?;

    Ok(response)
}

#[cfg(not(unix))]
fn send_to_socket(_socket_path: &str, _json_cmd: &str) -> Result<String, String> {
    Err("Unix domain sockets are not supported on this platform".to_string())
}

fn cmd_command(args: &[String]) -> Result<(), String> {
    use std::io::IsTerminal;

    let mut connect = false;
    let mut socket_path = "/tmp/firstproof-debug.sock".to_string();
    let mut read_stdin = false;
    let mut command_json: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_command_help();
                return Ok(());
            }
            "-c" | "--connect" => {
                connect = true;
                i += 1;
            }
            "--socket" => {
                i += 1;
                if i >= args.len() {
                    return Err("Missing value for --socket".to_string());
                }
                socket_path = args[i].clone();
                connect = true;
                i += 1;
            }
            "--stdin" => {
                read_stdin = true;
                i += 1;
            }
            arg if !arg.starts_with('-') && command_json.is_none() => {
                command_json = Some(arg.to_string());
                i += 1;
            }
            other => {
                return Err(format!("Unknown option: {}", other));
            }
        }
    }

    if read_stdin || (command_json.is_none() && !std::io::stdin().is_terminal()) {
        use std::io::BufRead;
        let stdin = std::io::stdin();
        for line in stdin.lock().lines() {
            let line = line.map_err(|e| e.to_string())?;
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let resp = if connect {
                send_to_socket(&socket_path, trimmed)?
            } else {
                firstproof_core::execute_darkroom_command_headless(trimmed.to_string())
            };
            println!("{}", resp.trim());
        }
        return Ok(());
    }

    let cmd_str = command_json.ok_or_else(|| {
        "Missing command JSON argument or --stdin. Use --help for usage.".to_string()
    })?;

    let resp = if connect {
        send_to_socket(&socket_path, &cmd_str)?
    } else {
        firstproof_core::execute_darkroom_command_headless(cmd_str)
    };

    println!("{}", resp.trim());
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_main_help();
        std::process::exit(1);
    }

    let subcommand = args[1].as_str();
    let rest = &args[2..];

    let result = match subcommand {
        "develop" | "dev" => cmd_develop(rest),
        "inspect" | "info" => cmd_inspect(rest),
        "dng" | "convert" => cmd_dng(rest),
        "command" | "cmd" => cmd_command(rest),
        "bundle-lensfun" => cmd_bundle_lensfun(rest),
        "shaders" | "transpile-shaders" => cmd_shaders(rest),
        "help" | "--help" | "-h" => {
            if !rest.is_empty() {
                match rest[0].as_str() {
                    "develop" | "dev" => print_develop_help(),
                    "dng" | "convert" => print_dng_help(),
                    "command" | "cmd" => print_command_help(),
                    "shaders" => print_shaders_help(),
                    _ => print_main_help(),
                }
            } else {
                print_main_help();
            }
            Ok(())
        }
        "version" | "--version" | "-v" => {
            println!("firstproof-cli v{}", VERSION);
            Ok(())
        }
        other => {
            eprintln!("Error: unknown subcommand '{}'\n", other);
            print_main_help();
            std::process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("\n❌ Error: {}", e);
        std::process::exit(1);
    }
}
