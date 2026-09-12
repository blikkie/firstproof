use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

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

// MARK: - Procedural Film Simulation Generators

/// Procedural generator for built-in creative film simulations.
/// All algorithms are pure math based on sensitometry curves, split-toning, and color matrix transforms.
/// Free of third-party IP or copyright encumbrances.
pub struct FilmSimulationGenerator;

impl FilmSimulationGenerator {
    /// Applies an S-curve contrast transfer function centered at `mid`.
    #[inline]
    fn s_curve(x: f32, contrast: f32, mid: f32) -> f32 {
        let centered = x - mid;
        let curved = centered * (1.0 + contrast * (1.0 - centered.abs().min(1.0)));
        (curved + mid).clamp(0.0, 1.0)
    }

    /// Converts sRGB RGB to HSV.
    #[inline]
    fn rgb_to_hsv(c: [f32; 3]) -> [f32; 3] {
        let r = c[0];
        let g = c[1];
        let b = c[2];
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let h = if delta <= 1e-6 {
            0.0
        } else if (max - r).abs() < 1e-6 {
            60.0 * (((g - b) / delta) % 6.0)
        } else if (max - g).abs() < 1e-6 {
            60.0 * (((b - r) / delta) + 2.0)
        } else {
            60.0 * (((r - g) / delta) + 4.0)
        };
        let h = if h < 0.0 { h + 360.0 } else { h };
        let s = if max <= 1e-6 { 0.0 } else { delta / max };
        let v = max;
        [h, s, v]
    }

    /// Converts HSV back to sRGB.
    #[inline]
    fn hsv_to_rgb(hsv: [f32; 3]) -> [f32; 3] {
        let h = hsv[0] / 60.0;
        let s = hsv[1];
        let v = hsv[2];
        let c = v * s;
        let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
        let m = v - c;

        let (r1, g1, b1) = if h < 1.0 {
            (c, x, 0.0)
        } else if h < 2.0 {
            (x, c, 0.0)
        } else if h < 3.0 {
            (0.0, c, x)
        } else if h < 4.0 {
            (0.0, x, c)
        } else if h < 5.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };
        [(r1 + m).clamp(0.0, 1.0), (g1 + m).clamp(0.0, 1.0), (b1 + m).clamp(0.0, 1.0)]
    }

    /// Generates "Film - Classic Chrome":
    /// Muted overall saturation, rich deep shadows, subtle warm highlights and cooler shadow bias.
    pub fn classic_chrome(size: usize) -> CubeLut {
        let mut lut = CubeLut::identity(size);
        lut.title = "Film - Classic Chrome".to_string();

        for sample in &mut lut.data {
            let r = sample[0];
            let g = sample[1];
            let b = sample[2];

            let mut hsv = Self::rgb_to_hsv([r, g, b]);
            // Desaturate slightly across the spectrum, especially greens/cyans
            let sat_scale = if hsv[0] > 70.0 && hsv[0] < 180.0 {
                0.72 // Mute foliage & greens
            } else if hsv[0] >= 180.0 && hsv[0] < 260.0 {
                0.80 // Mute skies slightly
            } else {
                0.85
            };
            hsv[1] = (hsv[1] * sat_scale).clamp(0.0, 1.0);

            let desat_rgb = Self::hsv_to_rgb(hsv);
            let lum = 0.2126 * desat_rgb[0] + 0.7152 * desat_rgb[1] + 0.0722 * desat_rgb[2];

            // Punchy contrast in mid-low tones
            let cr = Self::s_curve(desat_rgb[0], 0.25, 0.45);
            let cg = Self::s_curve(desat_rgb[1], 0.25, 0.45);
            let cb = Self::s_curve(desat_rgb[2], 0.25, 0.45);

            // Subtle split tone: warm highlights, cool shadows
            let shadow_factor = (1.0 - lum).powf(1.8);
            let highlight_factor = lum.powf(2.0);

            let out_r = cr + highlight_factor * 0.03 - shadow_factor * 0.02;
            let out_g = cg + highlight_factor * 0.01;
            let out_b = cb - highlight_factor * 0.02 + shadow_factor * 0.03;

            *sample = [out_r.clamp(0.0, 1.0), out_g.clamp(0.0, 1.0), out_b.clamp(0.0, 1.0)];
        }
        lut
    }

    /// Generates "Film - Warm Portra":
    /// Smooth highlight roll-off, warm golden skin tone protection, gentle shadow lift.
    pub fn warm_portra(size: usize) -> CubeLut {
        let mut lut = CubeLut::identity(size);
        lut.title = "Film - Warm Portra".to_string();

        for sample in &mut lut.data {
            let r = sample[0];
            let g = sample[1];
            let b = sample[2];

            let mut hsv = Self::rgb_to_hsv([r, g, b]);
            // Warm skin tones (hue 20 to 45 deg)
            let is_skin = (hsv[0] >= 10.0 && hsv[0] <= 55.0) as i32 as f32;
            if is_skin > 0.0 {
                // Protect & gently warm skin saturation
                hsv[1] = (hsv[1] * 1.05).clamp(0.0, 1.0);
            } else {
                hsv[1] = (hsv[1] * 0.95).clamp(0.0, 1.0);
            }

            let rgb_adj = Self::hsv_to_rgb(hsv);
            let lum = 0.2126 * rgb_adj[0] + 0.7152 * rgb_adj[1] + 0.0722 * rgb_adj[2];

            // Soft highlight compression curve & gentle shadow lift
            let lift = 0.02 * (1.0 - lum).powf(2.0);
            let roll_off = 1.0 - (1.0 - lum).powf(1.1);

            let out_r = (rgb_adj[0] * 0.98 + roll_off * 0.04 + lift * 1.2).clamp(0.0, 1.0);
            let out_g = (rgb_adj[1] * 0.99 + roll_off * 0.02 + lift * 1.0).clamp(0.0, 1.0);
            let out_b = (rgb_adj[2] * 0.95 + lift * 0.7).clamp(0.0, 1.0);

            *sample = [out_r, out_g, out_b];
        }
        lut
    }

    /// Generates "Film - Cine Teal & Orange":
    /// Classic complementary cinematic color grade: amber/warm skin and midtones, teal/cyan shadows.
    pub fn cine_teal_orange(size: usize) -> CubeLut {
        let mut lut = CubeLut::identity(size);
        lut.title = "Film - Cine Teal & Orange".to_string();

        for sample in &mut lut.data {
            let r = sample[0];
            let g = sample[1];
            let b = sample[2];

            let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;

            // S-curve for deep film contrast
            let cr = Self::s_curve(r, 0.22, 0.48);
            let cg = Self::s_curve(g, 0.20, 0.48);
            let cb = Self::s_curve(b, 0.22, 0.48);

            // Shadow teal push (cyan-blue)
            let shadow_weight = (1.0 - lum).powf(1.6) * 0.12;
            // Highlight orange push (amber-gold)
            let highlight_weight = lum.powf(1.4) * 0.10;

            let out_r = (cr - shadow_weight * 0.4 + highlight_weight * 1.0).clamp(0.0, 1.0);
            let out_g = (cg + shadow_weight * 0.5 + highlight_weight * 0.4).clamp(0.0, 1.0);
            let out_b = (cb + shadow_weight * 1.0 - highlight_weight * 0.6).clamp(0.0, 1.0);

            *sample = [out_r, out_g, out_b];
        }
        lut
    }

    /// Generates "Film - B&W High Contrast (Tri-X)":
    /// Panchromatic luminance weighting, aggressive S-curve, crushed deep blacks, crisp silver highlights.
    pub fn tri_x_bw(size: usize) -> CubeLut {
        let mut lut = CubeLut::identity(size);
        lut.title = "Film - B&W High Contrast (Tri-X)".to_string();

        for sample in &mut lut.data {
            let r = sample[0];
            let g = sample[1];
            let b = sample[2];

            // Panchromatic spectral sensitivity (enhanced red and green contrast)
            let panchro_lum = 0.32 * r + 0.56 * g + 0.12 * b;

            // Aggressive film S-curve
            let curved = Self::s_curve(panchro_lum, 0.45, 0.50);

            *sample = [curved, curved, curved];
        }
        lut
    }

    /// Generates "Film - Vintage Fade":
    /// Lifted matte black point, compressed dynamic range, warm cream tint, nostalgic pastel feel.
    pub fn vintage_fade(size: usize) -> CubeLut {
        let mut lut = CubeLut::identity(size);
        lut.title = "Film - Vintage Fade".to_string();

        for sample in &mut lut.data {
            let r = sample[0];
            let g = sample[1];
            let b = sample[2];

            let mut hsv = Self::rgb_to_hsv([r, g, b]);
            hsv[1] = (hsv[1] * 0.82).clamp(0.0, 1.0); // Soft pastel desaturation

            let rgb_adj = Self::hsv_to_rgb(hsv);
            let lum = 0.2126 * rgb_adj[0] + 0.7152 * rgb_adj[1] + 0.0722 * rgb_adj[2];

            // Lift black point to 0.08 (matte shadow wash)
            let min_black = 0.08;
            let max_white = 0.94;
            let compressed_lum = min_black + lum * (max_white - min_black);

            let gain = compressed_lum / lum.max(1e-5);
            let out_r = (rgb_adj[0] * gain * 1.03 + 0.02).clamp(0.0, 1.0);
            let out_g = (rgb_adj[1] * gain * 1.00 + 0.01).clamp(0.0, 1.0);
            let out_b = (rgb_adj[2] * gain * 0.94).clamp(0.0, 1.0);

            *sample = [out_r, out_g, out_b];
        }
        lut
    }

    /// Returns the collection of all 5 built-in film simulation LUTs.
    pub fn all_builtin(size: usize) -> Vec<CubeLut> {
        vec![
            Self::classic_chrome(size),
            Self::warm_portra(size),
            Self::cine_teal_orange(size),
            Self::tri_x_bw(size),
            Self::vintage_fade(size),
        ]
    }

    /// Returns the titles of all 5 built-in film simulations without generating LUT data.
    pub fn builtin_titles() -> Vec<String> {
        vec![
            "Film - Classic Chrome".to_string(),
            "Film - Warm Portra".to_string(),
            "Film - Cine Teal & Orange".to_string(),
            "Film - B&W High Contrast (Tri-X)".to_string(),
            "Film - Vintage Fade".to_string(),
        ]
    }

    /// Generates a single built-in simulation by title.
    pub fn generate_by_title(title: &str, size: usize) -> Option<CubeLut> {
        match title {
            "Film - Classic Chrome" => Some(Self::classic_chrome(size)),
            "Film - Warm Portra" => Some(Self::warm_portra(size)),
            "Film - Cine Teal & Orange" => Some(Self::cine_teal_orange(size)),
            "Film - B&W High Contrast (Tri-X)" => Some(Self::tri_x_bw(size)),
            "Film - Vintage Fade" => Some(Self::vintage_fade(size)),
            _ => None,
        }
    }
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
        let lut = FilmSimulationGenerator::classic_chrome(17);
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

    #[test]
    fn test_builtin_simulations_generate() {
        let all = FilmSimulationGenerator::all_builtin(17);
        assert_eq!(all.len(), 5);
        for lut in all {
            assert_eq!(lut.data.len(), 17 * 17 * 17);
            let rgba = lut.to_rgba32f();
            assert_eq!(rgba.len(), 17 * 17 * 17 * 4);
        }
    }
}
