use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

#[derive(uniffi::Record, Clone, Debug, PartialEq)]
pub struct LutRecord {
    pub id: String,
    pub title: String,
    pub file_path: String,
    pub size: u32,
    pub is_builtin: bool,
}

/// A 3D Look-Up Table (LUT) parsed from an Adobe / DaVinci `.cube` file or procedurally generated.
#[derive(Clone, Debug, PartialEq)]
pub struct CubeLut {
    pub title: String,
    pub size: usize,
    pub domain_min: [f32; 3],
    pub domain_max: [f32; 3],
    /// RGB data array of size `size * size * size`.
    /// Data ordering follows the `.cube` standard:
    /// Red index increases fastest, then Green, then Blue:
    /// `index = r + g * size + b * size * size`
    pub data: Vec<[f32; 3]>,
}

impl CubeLut {
    /// Creates a new empty or default identity 3D LUT with dimension `size`.
    pub fn identity(size: usize) -> Self {
        let size = size.max(2);
        let mut data = Vec::with_capacity(size * size * size);
        for b in 0..size {
            let b_val = b as f32 / (size - 1) as f32;
            for g in 0..size {
                let g_val = g as f32 / (size - 1) as f32;
                for r in 0..size {
                    let r_val = r as f32 / (size - 1) as f32;
                    data.push([r_val, g_val, b_val]);
                }
            }
        }
        Self {
            title: "Identity".to_string(),
            size,
            domain_min: [0.0, 0.0, 0.0],
            domain_max: [1.0, 1.0, 1.0],
            data,
        }
    }

    /// Fetches the raw sample at integer grid coordinates (r, g, b).
    #[inline]
    pub fn get_sample(&self, r: usize, g: usize, b: usize) -> [f32; 3] {
        let r = r.min(self.size - 1);
        let g = g.min(self.size - 1);
        let b = b.min(self.size - 1);
        let idx = r + g * self.size + b * self.size * self.size;
        self.data[idx]
    }

    /// Generates tightly-packed RGBA32F float data formatted for Metal 3D textures (`MTLTextureType3D`).
    /// Returns a vector of floats of length `size * size * size * 4`.
    pub fn to_rgba32f(&self) -> Vec<f32> {
        let mut out = Vec::with_capacity(self.data.len() * 4);
        for s in &self.data {
            out.push(s[0]);
            out.push(s[1]);
            out.push(s[2]);
            out.push(1.0);
        }
        out
    }

    /// Generates tightly-packed raw little-endian bytes for Metal 3D textures (`MTLTextureType3D`).
    /// Returns a byte vector of length `size * size * size * 4 * 4`.
    pub fn to_rgba32f_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.data.len() * 16);
        for s in &self.data {
            out.extend_from_slice(&s[0].to_le_bytes());
            out.extend_from_slice(&s[1].to_le_bytes());
            out.extend_from_slice(&s[2].to_le_bytes());
            out.extend_from_slice(&1.0f32.to_le_bytes());
        }
        out
    }

    /// Trilinear 3D interpolation.
    /// Input `rgb` is normalized to `[0.0, 1.0]`.
    pub fn interpolate_trilinear(&self, rgb: [f32; 3]) -> [f32; 3] {
        let size_f = self.size as f32;
        let s_minus_1 = size_f - 1.0;

        // Map domain [domain_min, domain_max] -> [0, size - 1]
        let norm_r = ((rgb[0] - self.domain_min[0]) / (self.domain_max[0] - self.domain_min[0]))
            .clamp(0.0, 1.0)
            * s_minus_1;
        let norm_g = ((rgb[1] - self.domain_min[1]) / (self.domain_max[1] - self.domain_min[1]))
            .clamp(0.0, 1.0)
            * s_minus_1;
        let norm_b = ((rgb[2] - self.domain_min[2]) / (self.domain_max[2] - self.domain_min[2]))
            .clamp(0.0, 1.0)
            * s_minus_1;

        let r0 = norm_r.floor() as usize;
        let r1 = (r0 + 1).min(self.size - 1);
        let g0 = norm_g.floor() as usize;
        let g1 = (g0 + 1).min(self.size - 1);
        let b0 = norm_b.floor() as usize;
        let b1 = (b0 + 1).min(self.size - 1);

        let fr = norm_r - r0 as f32;
        let fg = norm_g - g0 as f32;
        let fb = norm_b - b0 as f32;

        let c000 = self.get_sample(r0, g0, b0);
        let c100 = self.get_sample(r1, g0, b0);
        let c010 = self.get_sample(r0, g1, b0);
        let c110 = self.get_sample(r1, g1, b0);
        let c001 = self.get_sample(r0, g0, b1);
        let c101 = self.get_sample(r1, g0, b1);
        let c011 = self.get_sample(r0, g1, b1);
        let c111 = self.get_sample(r1, g1, b1);

        let mut res = [0.0f32; 3];
        for i in 0..3 {
            let c00 = c000[i] * (1.0 - fr) + c100[i] * fr;
            let c10 = c010[i] * (1.0 - fr) + c110[i] * fr;
            let c01 = c001[i] * (1.0 - fr) + c101[i] * fr;
            let c11 = c011[i] * (1.0 - fr) + c111[i] * fr;

            let c0 = c00 * (1.0 - fg) + c10 * fg;
            let c1 = c01 * (1.0 - fg) + c11 * fg;

            res[i] = c0 * (1.0 - fb) + c1 * fb;
        }
        res
    }

    /// Tetrahedral 3D interpolation.
    /// Splits each sub-cube into 6 tetrahedra for superior diagonal color preservation
    /// and absence of hue twisting on saturated gradients.
    pub fn interpolate_tetrahedral(&self, rgb: [f32; 3]) -> [f32; 3] {
        let size_f = self.size as f32;
        let s_minus_1 = size_f - 1.0;

        let norm_r = ((rgb[0] - self.domain_min[0]) / (self.domain_max[0] - self.domain_min[0]))
            .clamp(0.0, 1.0)
            * s_minus_1;
        let norm_g = ((rgb[1] - self.domain_min[1]) / (self.domain_max[1] - self.domain_min[1]))
            .clamp(0.0, 1.0)
            * s_minus_1;
        let norm_b = ((rgb[2] - self.domain_min[2]) / (self.domain_max[2] - self.domain_min[2]))
            .clamp(0.0, 1.0)
            * s_minus_1;

        let r0 = norm_r.floor() as usize;
        let r1 = (r0 + 1).min(self.size - 1);
        let g0 = norm_g.floor() as usize;
        let g1 = (g0 + 1).min(self.size - 1);
        let b0 = norm_b.floor() as usize;
        let b1 = (b0 + 1).min(self.size - 1);

        let fr = norm_r - r0 as f32;
        let fg = norm_g - g0 as f32;
        let fb = norm_b - b0 as f32;

        let c000 = self.get_sample(r0, g0, b0);
        let c111 = self.get_sample(r1, g1, b1);

        let mut res = [0.0f32; 3];

        if fr >= fg && fg >= fb {
            // Case 1: fr >= fg >= fb
            let c100 = self.get_sample(r1, g0, b0);
            let c110 = self.get_sample(r1, g1, b0);
            for i in 0..3 {
                res[i] = (1.0 - fr) * c000[i]
                    + (fr - fg) * c100[i]
                    + (fg - fb) * c110[i]
                    + fb * c111[i];
            }
        } else if fr >= fb && fb >= fg {
            // Case 2: fr >= fb >= fg
            let c100 = self.get_sample(r1, g0, b0);
            let c101 = self.get_sample(r1, g0, b1);
            for i in 0..3 {
                res[i] = (1.0 - fr) * c000[i]
                    + (fr - fb) * c100[i]
                    + (fb - fg) * c101[i]
                    + fg * c111[i];
            }
        } else if fg >= fr && fr >= fb {
            // Case 3: fg >= fr >= fb
            let c010 = self.get_sample(r0, g1, b0);
            let c110 = self.get_sample(r1, g1, b0);
            for i in 0..3 {
                res[i] = (1.0 - fg) * c000[i]
                    + (fg - fr) * c010[i]
                    + (fr - fb) * c110[i]
                    + fb * c111[i];
            }
        } else if fg >= fb && fb >= fr {
            // Case 4: fg >= fb >= fr
            let c010 = self.get_sample(r0, g1, b0);
            let c011 = self.get_sample(r0, g1, b1);
            for i in 0..3 {
                res[i] = (1.0 - fg) * c000[i]
                    + (fg - fb) * c010[i]
                    + (fb - fr) * c011[i]
                    + fr * c111[i];
            }
        } else if fb >= fr && fr >= fg {
            // Case 5: fb >= fr >= fg
            let c001 = self.get_sample(r0, g0, b1);
            let c101 = self.get_sample(r1, g0, b1);
            for i in 0..3 {
                res[i] = (1.0 - fb) * c000[i]
                    + (fb - fr) * c001[i]
                    + (fr - fg) * c101[i]
                    + fg * c111[i];
            }
        } else {
            // Case 6: fb >= fg >= fr
            let c001 = self.get_sample(r0, g0, b1);
            let c011 = self.get_sample(r0, g1, b1);
            for i in 0..3 {
                res[i] = (1.0 - fb) * c000[i]
                    + (fb - fg) * c001[i]
                    + (fg - fr) * c011[i]
                    + fr * c111[i];
            }
        }

        res
    }
}

/// Parses an Adobe / DaVinci `.cube` file from a reader.
pub fn parse_cube_lut<R: Read>(reader: R) -> Result<CubeLut, String> {
    let buf_reader = BufReader::new(reader);
    let mut title = "Custom LUT".to_string();
    let mut size: Option<usize> = None;
    let mut domain_min = [0.0f32, 0.0, 0.0];
    let mut domain_max = [1.0f32, 1.0, 1.0];
    let mut data = Vec::new();

    for line_res in buf_reader.lines() {
        let line = line_res.map_err(|e| format!("I/O error reading .cube: {}", e))?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0].to_uppercase().as_str() {
            "TITLE" => {
                if parts.len() > 1 {
                    title = parts[1..].join(" ").trim_matches('"').to_string();
                }
            }
            "LUT_3D_SIZE" => {
                if parts.len() > 1 {
                    let s = parts[1]
                        .parse::<usize>()
                        .map_err(|e| format!("Invalid LUT_3D_SIZE '{}': {}", parts[1], e))?;
                    size = Some(s);
                }
            }
            "DOMAIN_MIN" => {
                if parts.len() >= 4 {
                    domain_min = [
                        parts[1].parse().unwrap_or(0.0),
                        parts[2].parse().unwrap_or(0.0),
                        parts[3].parse().unwrap_or(0.0),
                    ];
                }
            }
            "DOMAIN_MAX" => {
                if parts.len() >= 4 {
                    domain_max = [
                        parts[1].parse().unwrap_or(1.0),
                        parts[2].parse().unwrap_or(1.0),
                        parts[3].parse().unwrap_or(1.0),
                    ];
                }
            }
            _ => {
                // Check if this line is an RGB float triplet
                if parts.len() >= 3 {
                    if let (Ok(r), Ok(g), Ok(b)) = (
                        parts[0].parse::<f32>(),
                        parts[1].parse::<f32>(),
                        parts[2].parse::<f32>(),
                    ) {
                        data.push([r, g, b]);
                    }
                }
            }
        }
    }

    let size = size.ok_or_else(|| "Missing LUT_3D_SIZE keyword in .cube file".to_string())?;
    let expected_count = size * size * size;
    if data.len() != expected_count {
        return Err(format!(
            "Expected {} data points for size {}, but parsed {}",
            expected_count,
            size,
            data.len()
        ));
    }

    Ok(CubeLut {
        title,
        size,
        domain_min,
        domain_max,
        data,
    })
}

/// Reads and parses a `.cube` file from a filesystem path.
pub fn load_cube_lut_file<P: AsRef<Path>>(path: P) -> Result<CubeLut, String> {
    let p = path.as_ref();
    let file = File::open(p).map_err(|e| format!("Cannot open .cube file {:?}: {}", p, e))?;
    let mut lut = parse_cube_lut(file)?;
    if lut.title == "Custom LUT" {
        if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
            lut.title = stem.to_string();
        }
    }
    Ok(lut)
}

fn dirs_home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

/// Returns the platform-native global 3D LUT directory:
/// - macOS: `~/Library/Application Support/FirstProof/LUTs/`
/// - Linux/CLI: `$XDG_DATA_HOME/firstproof/luts/` or `~/.local/share/firstproof/luts/`
pub fn get_global_lut_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs_home_dir() {
            return home.join("Library/Application Support/FirstProof/LUTs");
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        if let Some(data_home) = std::env::var_os("XDG_DATA_HOME").map(PathBuf::from) {
            return data_home.join("firstproof/luts");
        } else if let Some(home) = dirs_home_dir() {
            return home.join(".local/share/firstproof/luts");
        }
    }
    PathBuf::from("luts")
}

pub fn ensure_global_lut_dir() -> Result<PathBuf, String> {
    let dir = get_global_lut_dir();
    fs::create_dir_all(&dir).map_err(|e| {
        format!("Failed to create global LUT directory '{}': {}", dir.display(), e)
    })?;
    Ok(dir)
}

/// Quickly inspects a `.cube` file header (reading at most 100 lines)
/// returning its (title, size) without allocating or reading large 3D data.
pub fn inspect_cube_file(path: &Path) -> Option<(String, usize)> {
    let file = File::open(path).ok()?;
    let buf_reader = BufReader::new(file);
    let mut title: Option<String> = None;
    let mut size: Option<usize> = None;

    for line_res in buf_reader.lines().take(100) {
        let line = match line_res {
            Ok(l) => l,
            Err(_) => break,
        };
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        match parts[0].to_uppercase().as_str() {
            "TITLE" if parts.len() > 1 => {
                title = Some(parts[1..].join(" ").trim_matches('"').to_string());
            }
            "LUT_3D_SIZE" if parts.len() > 1 => {
                if let Ok(s) = parts[1].parse::<usize>() {
                    size = Some(s);
                }
            }
            _ => {}
        }
        if size.is_some() && title.is_some() {
            break;
        }
    }

    let final_size = size.unwrap_or(33);
    let final_title = title.unwrap_or_else(|| {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Custom LUT")
            .to_string()
    });

    Some((final_title, final_size))
}

/// Discovers all `.cube` LUTs installed in the global library directory.
pub fn discover_global_luts() -> Vec<LutRecord> {
    let dir = match ensure_global_lut_dir() {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };

    let entries = match fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut luts = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            let is_cube = path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("cube"))
                .unwrap_or(false);

            if is_cube {
                if let Some((title, size)) = inspect_cube_file(&path) {
                    let file_name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown.cube")
                        .to_string();
                    luts.push(LutRecord {
                        id: format!("global:{}", file_name),
                        title,
                        file_path: path.to_string_lossy().to_string(),
                        size: size as u32,
                        is_builtin: false,
                    });
                }
            }
        }
    }

    luts.sort_by_key(|a| a.title.to_lowercase());
    luts
}

/// Copies an external `.cube` file into the persistent global LUT directory,
/// verifying its format and making it permanently accessible across sessions.
pub fn import_global_lut(source_path: &str) -> Result<LutRecord, String> {
    let src = Path::new(source_path);
    if !src.exists() {
        return Err(format!("Source LUT file does not exist: {}", source_path));
    }

    let (title, size) = inspect_cube_file(src).ok_or_else(|| {
        format!("File is not a valid Adobe/DaVinci .cube LUT: {}", source_path)
    })?;

    let dest_dir = ensure_global_lut_dir()?;
    let original_name = src
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "Invalid file name".to_string())?;

    // Sanitize file name
    let safe_name: String = original_name
        .chars()
        .map(|c| if c == '/' || c == '\\' || c == ':' { '_' } else { c })
        .collect();

    let dest_path = dest_dir.join(&safe_name);
    fs::copy(src, &dest_path).map_err(|e| {
        format!("Failed to copy LUT to '{}': {}", dest_path.display(), e)
    })?;

    Ok(LutRecord {
        id: format!("global:{}", safe_name),
        title,
        file_path: dest_path.to_string_lossy().to_string(),
        size: size as u32,
        is_builtin: false,
    })
}

/// Deletes a custom LUT from the global LUT directory.
pub fn delete_global_lut(id_or_path: &str) -> Result<bool, String> {
    let dest_dir = ensure_global_lut_dir()?;
    let target_path = if let Some(file_name) = id_or_path.strip_prefix("global:") {
        dest_dir.join(file_name)
    } else {
        let p = Path::new(id_or_path);
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            dest_dir.join(id_or_path)
        }
    };

    if !target_path.exists() {
        return Err(format!("LUT file does not exist: {}", target_path.display()));
    }

    // Security check: verify destination path containment
    let canonical_dest = dest_dir.canonicalize().map_err(|e| e.to_string())?;
    let canonical_target = target_path.canonicalize().map_err(|e| e.to_string())?;
    if !canonical_target.starts_with(&canonical_dest) {
        return Err("Cannot delete LUT outside the global LUT library".to_string());
    }

    fs::remove_file(&canonical_target).map_err(|e| {
        format!("Failed to remove LUT file '{}': {}", canonical_target.display(), e)
    })?;

    Ok(true)
}

/// Renames a custom LUT in the global directory, updating both the filename and internal TITLE header.
pub fn rename_global_lut(id_or_path: &str, new_title: &str) -> Result<LutRecord, String> {
    let dest_dir = ensure_global_lut_dir()?;
    let current_path = if let Some(file_name) = id_or_path.strip_prefix("global:") {
        dest_dir.join(file_name)
    } else {
        let p = Path::new(id_or_path);
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            dest_dir.join(id_or_path)
        }
    };

    if !current_path.exists() {
        return Err(format!("LUT file does not exist: {}", current_path.display()));
    }

    let canonical_dest = dest_dir.canonicalize().map_err(|e| e.to_string())?;
    let canonical_target = current_path.canonicalize().map_err(|e| e.to_string())?;
    if !canonical_target.starts_with(&canonical_dest) {
        return Err("Cannot rename LUT outside the global LUT library".to_string());
    }

    let content = fs::read_to_string(&canonical_target)
        .map_err(|e| format!("Cannot read LUT: {}", e))?;

    let mut lines: Vec<String> = Vec::new();
    let mut title_found = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.to_uppercase().starts_with("TITLE") {
            lines.push(format!("TITLE \"{}\"", new_title));
            title_found = true;
        } else {
            lines.push(line.to_string());
        }
    }

    if !title_found {
        lines.insert(0, format!("TITLE \"{}\"", new_title));
    }

    let sanitized_title: String = new_title
        .chars()
        .map(|c| if c == '/' || c == '\\' || c == ':' { '_' } else { c })
        .collect();
    let new_file_name = format!("{}.cube", sanitized_title);
    let new_path = dest_dir.join(&new_file_name);

    let updated_content = lines.join("\n");
    fs::write(&new_path, updated_content)
        .map_err(|e| format!("Failed to write updated LUT: {}", e))?;

    if new_path != canonical_target {
        let _ = fs::remove_file(&canonical_target);
    }

    let (_, size) = inspect_cube_file(&new_path).unwrap_or((new_title.to_string(), 33));

    Ok(LutRecord {
        id: format!("global:{}", new_file_name),
        title: new_title.to_string(),
        file_path: new_path.to_string_lossy().to_string(),
        size: size as u32,
        is_builtin: false,
    })
}

/// Resolves and loads a 3D LUT by file path, file name, or title from the global LUT library.
pub fn load_global_or_cube_lut(name_or_path: &str) -> Result<CubeLut, String> {
    // 1. Direct path check
    let p = Path::new(name_or_path);
    if p.exists() && p.is_file() {
        return load_cube_lut_file(p);
    }

    // 2. Direct name in global LUT directory
    let dir = get_global_lut_dir();
    let in_dir = dir.join(name_or_path);
    if in_dir.exists() && in_dir.is_file() {
        return load_cube_lut_file(&in_dir);
    }

    // 3. Name with .cube in global LUT directory
    let in_dir_cube = dir.join(format!("{}.cube", name_or_path));
    if in_dir_cube.exists() && in_dir_cube.is_file() {
        return load_cube_lut_file(&in_dir_cube);
    }

    // 4. By TITLE or id in global directory
    let discovered = discover_global_luts();
    for lut in discovered {
        if lut.title == name_or_path || lut.id == name_or_path {
            return load_cube_lut_file(&lut.file_path);
        }
    }

    Err(format!("Could not resolve 3D LUT '{}'", name_or_path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_lut() {
        let lut = CubeLut::identity(17);
        assert_eq!(lut.size, 17);
        assert_eq!(lut.data.len(), 17 * 17 * 17);

        // Sampling origin should be [0, 0, 0]
        let s0 = lut.interpolate_tetrahedral([0.0, 0.0, 0.0]);
        assert!((s0[0] - 0.0).abs() < 1e-5);
        assert!((s0[1] - 0.0).abs() < 1e-5);
        assert!((s0[2] - 0.0).abs() < 1e-5);

        // Sampling mid should be [0.5, 0.5, 0.5]
        let s_mid = lut.interpolate_tetrahedral([0.5, 0.5, 0.5]);
        assert!((s_mid[0] - 0.5).abs() < 0.01);
        assert!((s_mid[1] - 0.5).abs() < 0.01);
        assert!((s_mid[2] - 0.5).abs() < 0.01);

        // Sampling 1 should be [1, 1, 1]
        let s1 = lut.interpolate_tetrahedral([1.0, 1.0, 1.0]);
        assert!((s1[0] - 1.0).abs() < 1e-5);
        assert!((s1[1] - 1.0).abs() < 1e-5);
        assert!((s1[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_parse_cube_format() {
        let cube_text = r#"
# Simple test .cube
TITLE "Test Cube"
LUT_3D_SIZE 2
DOMAIN_MIN 0.0 0.0 0.0
DOMAIN_MAX 1.0 1.0 1.0

0.0 0.0 0.0
1.0 0.0 0.0
0.0 1.0 0.0
1.0 1.0 0.0
0.0 0.0 1.0
1.0 0.0 1.0
0.0 1.0 1.0
1.0 1.0 1.0
"#;
        let lut = parse_cube_lut(cube_text.as_bytes()).unwrap();
        assert_eq!(lut.title, "Test Cube");
        assert_eq!(lut.size, 2);
        assert_eq!(lut.data.len(), 8);
        assert_eq!(lut.data[0], [0.0, 0.0, 0.0]);
        assert_eq!(lut.data[7], [1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_trilinear_and_tetrahedral_parity() {
        let mut lut = CubeLut::identity(17);
        for sample in &mut lut.data {
            sample[0] = sample[0].powf(1.2);
            sample[1] = sample[1].powf(1.1);
            sample[2] = sample[2].powf(0.9);
        }
        let test_points = [
            [0.2, 0.3, 0.4],
            [0.5, 0.5, 0.5],
            [0.8, 0.1, 0.9],
            [0.0, 1.0, 0.0],
        ];

        for pt in &test_points {
            let tri = lut.interpolate_trilinear(*pt);
            let tetra = lut.interpolate_tetrahedral(*pt);

            for ch in 0..3 {
                // Trilinear and tetrahedral should closely track each other
                assert!((tri[ch] - tetra[ch]).abs() < 0.05);
            }
        }
    }
}
