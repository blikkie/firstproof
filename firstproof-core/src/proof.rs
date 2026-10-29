//! ColorSync & LittleCMS2 Soft Proofing and ICC Profile Engine for First Proof
//!
//! Provides:
//! 1. Native ColorSync and system ICC profile discovery (`~/Library/ColorSync/Profiles`, etc.)
//! 2. 1-click custom print lab profile ingestion (`import_icc_profile`)
//! 3. High-performance 3D Soft Proofing & Gamut Alarm LUT generation via LittleCMS2
//! 4. Wide-gamut (Display P3) and standard (sRGB) source/display color space handling

use std::fs;
use std::path::{Path, PathBuf};
use crate::CubeLutRecord;

#[derive(uniffi::Record, Clone, Debug, PartialEq)]
pub struct IccProfileRecord {
    pub id: String,
    pub name: String,
    pub description: String,
    pub file_path: String,
    pub color_space: String,
    pub device_class: String,
    pub is_cmyk: bool,
    pub is_built_in: bool,
}

#[derive(uniffi::Enum, Clone, Copy, Debug, PartialEq)]
pub enum ProofIntent {
    RelativeColorimetric,
    Perceptual,
    AbsoluteColorimetric,
}

impl ProofIntent {
    pub fn to_lcms_intent(self) -> lcms2::Intent {
        match self {
            ProofIntent::RelativeColorimetric => lcms2::Intent::RelativeColorimetric,
            ProofIntent::Perceptual => lcms2::Intent::Perceptual,
            ProofIntent::AbsoluteColorimetric => lcms2::Intent::AbsoluteColorimetric,
        }
    }
}

/// Discovers available ICC profiles on the host machine following standard OS conventions.
/// On macOS, scans `~/Library/ColorSync/Profiles/`, `/Library/ColorSync/Profiles/`, and `/System/Library/ColorSync/Profiles/`.
/// Always includes standard baseline profiles (`sRGB`, `Display P3`).
pub fn discover_icc_profiles() -> Vec<IccProfileRecord> {
    let mut profiles = Vec::new();

    // 1. Built-in standard profiles
    profiles.push(IccProfileRecord {
        id: "builtin:srgb".to_string(),
        name: "sRGB IEC61966-2.1".to_string(),
        description: "Standard sRGB (Web / Screen Delivery)".to_string(),
        file_path: "".to_string(),
        color_space: "RGB".to_string(),
        device_class: "Display".to_string(),
        is_cmyk: false,
        is_built_in: true,
    });

    profiles.push(IccProfileRecord {
        id: "builtin:display_p3".to_string(),
        name: "Display P3".to_string(),
        description: "Apple Wide Gamut Display P3".to_string(),
        file_path: "".to_string(),
        color_space: "RGB".to_string(),
        device_class: "Display".to_string(),
        is_cmyk: false,
        is_built_in: true,
    });

    // 2. Platform directories to scan
    let mut search_dirs: Vec<PathBuf> = Vec::new();

    if let Some(home) = dirs_home_dir() {
        search_dirs.push(home.join("Library/ColorSync/Profiles"));
        search_dirs.push(home.join(".local/share/color/icc"));
        search_dirs.push(home.join(".color/icc"));
    }

    search_dirs.push(PathBuf::from("/Library/ColorSync/Profiles"));
    search_dirs.push(PathBuf::from("/Library/ColorSync/Profiles/Displays"));
    search_dirs.push(PathBuf::from("/System/Library/ColorSync/Profiles"));
    search_dirs.push(PathBuf::from("/usr/share/color/icc"));

    let mut seen_paths = std::collections::HashSet::new();

    for dir in search_dirs {
        if !dir.exists() || !dir.is_dir() {
            continue;
        }

        scan_directory_for_icc(&dir, &mut seen_paths, &mut profiles);
    }

    // Sort: built-in first, then alphabetical by description
    profiles.sort_by(|a, b| {
        if a.is_built_in != b.is_built_in {
            b.is_built_in.cmp(&a.is_built_in)
        } else {
            a.description.to_lowercase().cmp(&b.description.to_lowercase())
        }
    });

    profiles
}

fn dirs_home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn scan_directory_for_icc(
    dir: &Path,
    seen_paths: &mut std::collections::HashSet<PathBuf>,
    out: &mut Vec<IccProfileRecord>,
) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Recurse one level if appropriate (e.g. Displays)
            if path.file_name().and_then(|n| n.to_str()) == Some("Displays") {
                scan_directory_for_icc(&path, seen_paths, out);
            }
            continue;
        }

        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
        if ext != "icc" && ext != "icm" {
            continue;
        }

        let canonical = path.canonicalize().unwrap_or_else(|_| path.clone());
        if seen_paths.contains(&canonical) {
            continue;
        }
        seen_paths.insert(canonical);

        if let Some(record) = inspect_icc_file(&path) {
            out.push(record);
        }
    }
}

/// Parses metadata from an on-disk ICC profile using LittleCMS2.
pub fn inspect_icc_file(path: &Path) -> Option<IccProfileRecord> {
    let profile = lcms2::Profile::new_file(path).ok()?;
    let file_name = path.file_name()?.to_string_lossy().to_string();

    let desc = profile
        .info(lcms2::InfoType::Description, lcms2::Locale::none())
        .unwrap_or_else(|| {
            path.file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| file_name.clone())
        });

    let cs = profile.color_space();
    let cs_str = match cs {
        lcms2::ColorSpaceSignature::RgbData => "RGB",
        lcms2::ColorSpaceSignature::CmykData => "CMYK",
        lcms2::ColorSpaceSignature::GrayData => "Gray",
        lcms2::ColorSpaceSignature::LabData => "Lab",
        lcms2::ColorSpaceSignature::XYZData => "XYZ",
        _ => "Other",
    };

    let dev_cls = profile.device_class();
    let dev_str = match dev_cls {
        lcms2::ProfileClassSignature::OutputClass => "Output",
        lcms2::ProfileClassSignature::DisplayClass => "Display",
        lcms2::ProfileClassSignature::InputClass => "Input",
        lcms2::ProfileClassSignature::LinkClass => "DeviceLink",
        lcms2::ProfileClassSignature::AbstractClass => "Abstract",
        lcms2::ProfileClassSignature::ColorSpaceClass => "ColorSpace",
        lcms2::ProfileClassSignature::NamedColorClass => "NamedColor",
        _ => "Other",
    };

    let is_cmyk = cs == lcms2::ColorSpaceSignature::CmykData;

    Some(IccProfileRecord {
        id: path.to_string_lossy().to_string(),
        name: file_name,
        description: desc,
        file_path: path.to_string_lossy().to_string(),
        color_space: cs_str.to_string(),
        device_class: dev_str.to_string(),
        is_cmyk,
        is_built_in: false,
    })
}

/// Copies an external ICC profile into the standard user ColorSync directory (`~/Library/ColorSync/Profiles/`),
/// making it permanently accessible across application sessions.
pub fn import_icc_profile(source_path: &str) -> Result<IccProfileRecord, String> {
    let src = Path::new(source_path);
    if !src.exists() {
        return Err(format!("Source ICC profile file does not exist: {}", source_path));
    }

    // Validate the profile with LittleCMS2
    let _record = inspect_icc_file(src).ok_or_else(|| {
        format!("File is not a valid ICC profile readable by LittleCMS2: {}", source_path)
    })?;

    let home = dirs_home_dir().ok_or_else(|| "Could not determine user home directory".to_string())?;
    
    #[cfg(target_os = "macos")]
    let target_dir = home.join("Library/ColorSync/Profiles");

    #[cfg(not(target_os = "macos"))]
    let target_dir = home.join(".local/share/color/icc");

    fs::create_dir_all(&target_dir).map_err(|e| {
        format!("Failed to create destination ICC directory '{}': {}", target_dir.display(), e)
    })?;

    let file_name = src
        .file_name()
        .ok_or_else(|| "Invalid file name".to_string())?;
    let dest = target_dir.join(file_name);

    fs::copy(src, &dest).map_err(|e| {
        format!("Failed to copy ICC profile to '{}': {}", dest.display(), e)
    })?;

    inspect_icc_file(&dest).ok_or_else(|| {
        format!("Failed to inspect imported ICC profile at '{}'", dest.display())
    })
}

/// Creates an in-memory linear RGB profile (gamma 1.0) with D65 illuminant for sRGB (Rec. 709) or Display P3 primaries.
/// Preserves the Linear Optical Domain Invariant for LittleCMS2 soft proofing transforms.
fn create_linear_rgb_profile(is_p3: bool) -> Result<lcms2::Profile, String> {
    let white_point = lcms2::CIExyY {
        x: 0.3127,
        y: 0.3290,
        Y: 1.0,
    };
    let primaries = if is_p3 {
        lcms2::CIExyYTRIPLE {
            Red: lcms2::CIExyY { x: 0.680, y: 0.320, Y: 1.0 },
            Green: lcms2::CIExyY { x: 0.265, y: 0.690, Y: 1.0 },
            Blue: lcms2::CIExyY { x: 0.150, y: 0.060, Y: 1.0 },
        }
    } else {
        // sRGB primaries (Rec. 709)
        lcms2::CIExyYTRIPLE {
            Red: lcms2::CIExyY { x: 0.640, y: 0.330, Y: 1.0 },
            Green: lcms2::CIExyY { x: 0.300, y: 0.600, Y: 1.0 },
            Blue: lcms2::CIExyY { x: 0.150, y: 0.060, Y: 1.0 },
        }
    };
    let linear_curve = lcms2::ToneCurve::new(1.0);
    let curves = [&linear_curve, &linear_curve, &linear_curve];
    lcms2::Profile::new_rgb(&white_point, &primaries, &curves)
        .map_err(|e| format!("Failed to create linear RGB profile: {:?}", e))
}

/// Generates a 3D Proofing Look-Up Table (default size 33×33×33) using LittleCMS2.
///
/// Output layout:
/// - `size * size * size * 4` 32-bit floats packed as little-endian bytes (`MTLTextureType3D` / WebGL 3D texture).
/// - Channels `(R, G, B)` store the simulated proofed color in the user's display color space.
/// - Channel `A` stores `1.0` for in-gamut colors and `0.0` for out-of-gamut colors.
pub fn generate_soft_proof_lut(
    profile_id_or_path: &str,
    intent: ProofIntent,
    source_is_p3: bool,
    display_is_p3: bool,
    size: u32,
) -> Result<CubeLutRecord, String> {
    let size = (size as usize).clamp(17, 65);
    let total_points = size * size * size;

    // Distinctive alarm code for gamut check: [0xFFFF, 0x0000, 0xFFFF] (saturated magenta in 16-bit)
    let mut alarm_codes = [0u16; 16];
    alarm_codes[0] = 0xFFFF; // R
    alarm_codes[1] = 0x0000; // G
    alarm_codes[2] = 0xFFFF; // B
    #[allow(deprecated)]
    lcms2::Transform::<u8, u8>::set_global_alarm_codes(alarm_codes);

    // 1. Source Profile (Rendered Picture Space - Linear Optical Domain)
    let in_prof = create_linear_rgb_profile(source_is_p3)?;

    // 2. Display Profile (User Monitor Space - Linear Optical Domain)
    let out_display = create_linear_rgb_profile(display_is_p3)?;

    // 3. Proofing Profile (Simulation Target)
    let (proof_prof, profile_title) = if profile_id_or_path == "builtin:srgb" || profile_id_or_path == "sRGB" {
        (lcms2::Profile::new_srgb(), "sRGB IEC61966-2.1".to_string())
    } else if profile_id_or_path == "builtin:display_p3" || profile_id_or_path == "Display P3" {
        let p = crate::export::icc::create_display_p3_profile()
            .map_err(|e| format!("Failed to create Display P3 proof profile: {}", e))?;
        (p, "Display P3".to_string())
    } else {
        let p = lcms2::Profile::new_file(profile_id_or_path)
            .map_err(|e| format!("Failed to open ICC profile '{}': {}", profile_id_or_path, e))?;
        let name = p
            .info(lcms2::InfoType::Description, lcms2::Locale::none())
            .unwrap_or_else(|| profile_id_or_path.to_string());
        (p, name)
    };

    let lcms_intent = intent.to_lcms_intent();

    // Configure proofing flags
    let mut proof_flags = lcms2::Flags::SOFT_PROOFING;
    if intent == ProofIntent::RelativeColorimetric {
        proof_flags = proof_flags | lcms2::Flags::BLACKPOINT_COMPENSATION;
    } else if intent == ProofIntent::AbsoluteColorimetric {
        // Absolute colorimetric simulates paper white substrate reflection
        proof_flags = proof_flags | lcms2::Flags::NO_WHITE_ON_WHITE_FIXUP;
    }

    let proof_transform = lcms2::Transform::new_proofing(
        &in_prof,
        lcms2::PixelFormat::RGB_8,
        &out_display,
        lcms2::PixelFormat::RGB_8,
        &proof_prof,
        lcms_intent,
        lcms_intent,
        proof_flags,
    )
    .map_err(|e| format!("Failed to create LittleCMS2 proofing transform: {:?}", e))?;

    let gamut_flags = proof_flags | lcms2::Flags::GAMUT_CHECK;
    let gamut_transform = lcms2::Transform::new_proofing(
        &in_prof,
        lcms2::PixelFormat::RGB_8,
        &out_display,
        lcms2::PixelFormat::RGB_8,
        &proof_prof,
        lcms_intent,
        lcms_intent,
        gamut_flags,
    )
    .map_err(|e| format!("Failed to create LittleCMS2 gamut check transform: {:?}", e))?;

    // Generate regular 3D grid in RGB
    let mut grid_in = Vec::with_capacity(total_points * 3);
    for b in 0..size {
        let b_val = (b * 255 / (size - 1)) as u8;
        for g in 0..size {
            let g_val = (g * 255 / (size - 1)) as u8;
            for r in 0..size {
                let r_val = (r * 255 / (size - 1)) as u8;
                grid_in.push(r_val);
                grid_in.push(g_val);
                grid_in.push(b_val);
            }
        }
    }

    let mut proofed_out = vec![0u8; total_points * 3];
    let mut gamut_out = vec![0u8; total_points * 3];

    proof_transform.transform_pixels(&grid_in, &mut proofed_out);
    gamut_transform.transform_pixels(&grid_in, &mut gamut_out);

    // Combine into tightly-packed RGBA32F little-endian bytes
    let mut rgba32f = Vec::with_capacity(total_points * 16);
    for i in 0..total_points {
        let pr = proofed_out[i * 3] as f32 / 255.0;
        let pg = proofed_out[i * 3 + 1] as f32 / 255.0;
        let pb = proofed_out[i * 3 + 2] as f32 / 255.0;

        let gr = gamut_out[i * 3];
        let gg = gamut_out[i * 3 + 1];
        let gb = gamut_out[i * 3 + 2];

        // LittleCMS alarm codes: set to pure magenta [255, 0, 255] via set_global_alarm_codes.
        // For in-gamut pixels, gamut_transform outputs the proofed color (pr, pg, pb).
        // Guard against genuine in-gamut magenta by verifying proofed color wasn't already magenta.
        let is_alarm = (gr == 255 && gg == 0 && gb == 255)
            && !(proofed_out[i * 3] == 255 && proofed_out[i * 3 + 1] == 0 && proofed_out[i * 3 + 2] == 255);
        let alpha = if is_alarm { 0.0f32 } else { 1.0f32 };

        rgba32f.extend_from_slice(&pr.to_le_bytes());
        rgba32f.extend_from_slice(&pg.to_le_bytes());
        rgba32f.extend_from_slice(&pb.to_le_bytes());
        rgba32f.extend_from_slice(&alpha.to_le_bytes());
    }

    Ok(CubeLutRecord {
        title: format!("Proof: {}", profile_title),
        size: size as u32,
        data_rgba32f: rgba32f,
    })
}