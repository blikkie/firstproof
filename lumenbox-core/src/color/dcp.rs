//! Adobe DNG Camera Profile (DCP) Parser & Dual-Illuminant Calibration Engine
//!
//! Implements the Adobe DNG Specification (§6 Color Management) for:
//! 1. Parsing standard `.dcp` profile containers (TIFF/IFD binary format).
//! 2. Extracting dual-illuminant calibration matrices (`ColorMatrix1/2`, `ForwardMatrix1/2`),
//!    illuminants, profile metadata, and manufacturer tone curves.
//! 3. Dual-illuminant Planckian interpolation along the blackbody locus using
//!    reciprocal temperature ($1/T$).
//! 4. Providing built-in calibrated dual-illuminant profiles for major camera families
//!    (Canon EOS, Sony Alpha, Nikon Z).

use super::{Matrix3x3, BRADFORD_D50_TO_D65, REC2020_TO_XYZ_D65, XYZ_D65_TO_REC2020};

// Standard DNG / DCP TIFF Tag IDs
pub const TAG_COLOR_MATRIX_1: u16 = 50721;       // 0xC621
pub const TAG_COLOR_MATRIX_2: u16 = 50722;       // 0xC622
pub const TAG_CALIBRATION_ILLUMINANT_1: u16 = 50778; // 0xC65A
pub const TAG_CALIBRATION_ILLUMINANT_2: u16 = 50779; // 0xC65B
pub const TAG_PROFILE_NAME: u16 = 50936;         // 0xC6F8
pub const TAG_PROFILE_TONE_CURVE: u16 = 50937;   // 0xC6F9
pub const TAG_PROFILE_HUE_SAT_MAP_DIMS: u16 = 50939;   // 0xC6FB
pub const TAG_PROFILE_HUE_SAT_MAP_DATA_1: u16 = 50940; // 0xC6FC
pub const TAG_PROFILE_HUE_SAT_MAP_DATA_2: u16 = 50941; // 0xC6FD
pub const TAG_PROFILE_HUE_SAT_MAP_ENCODING: u16 = 50942; // 0xC6FE
pub const TAG_FORWARD_MATRIX_1: u16 = 50964;     // 0xC714
pub const TAG_FORWARD_MATRIX_2: u16 = 50965;     // 0xC715

// Standard DNG Illuminant IDs
pub const ILLUMINANT_DAYLIGHT: u16 = 1;
pub const ILLUMINANT_TUNGSTEN: u16 = 3;
pub const ILLUMINANT_FLASH: u16 = 4;
pub const ILLUMINANT_STANDARD_A: u16 = 17;
pub const ILLUMINANT_D55: u16 = 20;
pub const ILLUMINANT_D65: u16 = 21;
pub const ILLUMINANT_D75: u16 = 22;
pub const ILLUMINANT_D50: u16 = 23;

/// Maps standard EXIF / DNG illuminant codes to correlated color temperature in Kelvin.
pub fn illuminant_to_temperature(illuminant: u16) -> f32 {
    match illuminant {
        ILLUMINANT_STANDARD_A | ILLUMINANT_TUNGSTEN => 2856.0,
        ILLUMINANT_D50 => 5003.0,
        ILLUMINANT_D55 | ILLUMINANT_DAYLIGHT => 5503.0,
        ILLUMINANT_D65 => 6504.0,
        ILLUMINANT_D75 => 7504.0,
        ILLUMINANT_FLASH => 5500.0,
        _ => 6504.0, // Default to daylight D65
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DcpError {
    InvalidHeader,
    UnsupportedByteOrder,
    UnexpectedEof,
    MalformedIfd,
    MissingRequiredTag(&'static str),
}

impl std::fmt::Display for DcpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DcpError::InvalidHeader => write!(f, "Invalid DCP/TIFF header signature"),
            DcpError::UnsupportedByteOrder => write!(f, "Unsupported DCP byte order"),
            DcpError::UnexpectedEof => write!(f, "Unexpected end of DCP file buffer"),
            DcpError::MalformedIfd => write!(f, "Malformed IFD structure in DCP profile"),
            DcpError::MissingRequiredTag(tag) => write!(f, "DCP missing required tag: {}", tag),
        }
    }
}

impl std::error::Error for DcpError {}

/// Converts Linear sRGB / display RGB to HSV (H in degrees [0, 360), S in [0, 1], V in [0, 1]).
pub fn rgb_to_hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let v = max;
    let s = if max > 1e-6 { delta / max } else { 0.0 };

    let h = if delta < 1e-6 {
        0.0
    } else if (max - r).abs() < 1e-6 {
        60.0 * (((g - b) / delta).rem_euclid(6.0))
    } else if (max - g).abs() < 1e-6 {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    (h, s, v)
}

/// Converts HSV (H in degrees [0, 360), S in [0, 1], V in [0, 1]) back to RGB.
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let c = v * s;
    let h_prime = (h.rem_euclid(360.0)) / 60.0;
    let x = c * (1.0 - ((h_prime.rem_euclid(2.0)) - 1.0).abs());
    let m = v - c;

    let (r1, g1, b1) = if h_prime < 1.0 {
        (c, x, 0.0)
    } else if h_prime < 2.0 {
        (x, c, 0.0)
    } else if h_prime < 3.0 {
        (0.0, c, x)
    } else if h_prime < 4.0 {
        (0.0, x, c)
    } else if h_prime < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (r1 + m, g1 + m, b1 + m)
}

/// Evaluates a tone curve defined as a piecewise spline of (x, y) coordinates.
pub fn evaluate_tone_curve(x: f32, curve: &[(f32, f32)]) -> f32 {
    if curve.is_empty() {
        return x;
    }
    if curve.len() == 1 {
        return curve[0].1;
    }
    let x_clamped = x.clamp(0.0, 1.0);
    if x_clamped <= curve[0].0 {
        return curve[0].1;
    }
    if x_clamped >= curve[curve.len() - 1].0 {
        return curve[curve.len() - 1].1;
    }

    for i in 0..curve.len() - 1 {
        let (x0, y0) = curve[i];
        let (x1, y1) = curve[i + 1];
        if x_clamped >= x0 && x_clamped <= x1 {
            let dx = x1 - x0;
            if dx.abs() < 1e-6 {
                return y0;
            }
            let t = (x_clamped - x0) / dx;
            let t2 = t * t;
            let t3 = t2 * t;
            let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
            let h01 = -2.0 * t3 + 3.0 * t2;
            return (h00 * y0 + h01 * y1).clamp(0.0, 1.0);
        }
    }
    x_clamped
}

/// A 3D Hue-Saturation-Value delta table for camera memory color enhancements.
#[derive(Debug, Clone, PartialEq)]
pub struct DcpHueSatMap {
    pub hue_divisions: usize,
    pub sat_divisions: usize,
    pub val_divisions: usize,
    /// Vector of (delta_hue_deg, scale_sat, scale_val) entries.
    /// Total length = hue_divisions * sat_divisions * val_divisions
    pub deltas: Vec<(f32, f32, f32)>,
}

impl DcpHueSatMap {
    pub fn new(hue_div: usize, sat_div: usize, val_div: usize, deltas: Vec<(f32, f32, f32)>) -> Self {
        Self {
            hue_divisions: hue_div.max(1),
            sat_divisions: sat_div.max(1),
            val_divisions: val_div.max(1),
            deltas,
        }
    }

    pub fn get_delta(&self, h_idx: usize, s_idx: usize, v_idx: usize) -> (f32, f32, f32) {
        let h = h_idx % self.hue_divisions;
        let s = s_idx.min(self.sat_divisions - 1);
        let v = v_idx.min(self.val_divisions - 1);
        let idx = (h * self.sat_divisions + s) * self.val_divisions + v;
        if idx < self.deltas.len() {
            self.deltas[idx]
        } else {
            (0.0, 1.0, 1.0)
        }
    }

    /// Evaluates interpolated delta for (Hue in degrees [0, 360), Saturation [0, 1], Value [0, 1]).
    /// Returns (delta_hue_deg, scale_sat, scale_val).
    pub fn evaluate_deltas(&self, h_deg: f32, s: f32, v: f32) -> (f32, f32, f32) {
        if self.deltas.is_empty() {
            return (0.0, 1.0, 1.0);
        }

        // Normalized Hue in [0, 1)
        let mut h_norm = (h_deg / 360.0).fract();
        if h_norm < 0.0 {
            h_norm += 1.0;
        }
        let h_coord = h_norm * (self.hue_divisions as f32);
        let h0 = (h_coord.floor() as usize) % self.hue_divisions;
        let h1 = (h0 + 1) % self.hue_divisions;
        let h_frac = h_coord - h_coord.floor();

        // Saturation in [0, 1]
        let s_clamped = s.clamp(0.0, 1.0);
        let s_coord = s_clamped * (self.sat_divisions.saturating_sub(1) as f32);
        let s0 = s_coord.floor() as usize;
        let s1 = (s0 + 1).min(self.sat_divisions.saturating_sub(1));
        let s_frac = s_coord - (s0 as f32);

        // Value in [0, 1]
        let v_clamped = v.clamp(0.0, 1.0);
        let v_coord = v_clamped * (self.val_divisions.saturating_sub(1) as f32);
        let v0 = v_coord.floor() as usize;
        let v1 = (v0 + 1).min(self.val_divisions.saturating_sub(1));
        let v_frac = v_coord - (v0 as f32);

        // Trilinear interpolation over 8 corners
        let c000 = self.get_delta(h0, s0, v0);
        let c001 = self.get_delta(h0, s0, v1);
        let c010 = self.get_delta(h0, s1, v0);
        let c011 = self.get_delta(h0, s1, v1);
        let c100 = self.get_delta(h1, s0, v0);
        let c101 = self.get_delta(h1, s0, v1);
        let c110 = self.get_delta(h1, s1, v0);
        let c111 = self.get_delta(h1, s1, v1);

        let lerp_t = |a: (f32, f32, f32), b: (f32, f32, f32), t: f32| -> (f32, f32, f32) {
            (
                a.0 + t * (b.0 - a.0),
                a.1 + t * (b.1 - a.1),
                a.2 + t * (b.2 - a.2),
            )
        };

        let c00 = lerp_t(c000, c001, v_frac);
        let c01 = lerp_t(c010, c011, v_frac);
        let c10 = lerp_t(c100, c101, v_frac);
        let c11 = lerp_t(c110, c111, v_frac);

        let c0 = lerp_t(c00, c01, s_frac);
        let c1 = lerp_t(c10, c11, s_frac);

        lerp_t(c0, c1, h_frac)
    }

    /// Evaluates and transforms an HSV color tuple (H [0, 360), S [0, 1], V [0, 1]).
    pub fn apply_hsv(&self, h: f32, s: f32, v: f32) -> (f32, f32, f32) {
        let (dh, ds, dv) = self.evaluate_deltas(h, s, v);
        let mut h_out = (h + dh).rem_euclid(360.0);
        if h_out < 0.0 {
            h_out += 360.0;
        }
        let s_out = (s * ds).clamp(0.0, 1.0);
        let v_out = (v * dv).clamp(0.0, 1.0);
        (h_out, s_out, v_out)
    }

    /// Interpolates two HueSatMaps with weight $w \in [0.0, 1.0]$.
    pub fn interpolate(&self, other: &Self, weight: f32) -> Self {
        if self.hue_divisions != other.hue_divisions
            || self.sat_divisions != other.sat_divisions
            || self.val_divisions != other.val_divisions
        {
            return if weight < 0.5 { self.clone() } else { other.clone() };
        }
        let w = weight.clamp(0.0, 1.0);
        let mut deltas = Vec::with_capacity(self.deltas.len());
        for (d1, d2) in self.deltas.iter().zip(other.deltas.iter()) {
            deltas.push((
                d1.0 + w * (d2.0 - d1.0),
                d1.1 + w * (d2.1 - d1.1),
                d1.2 + w * (d2.2 - d1.2),
            ));
        }
        Self {
            hue_divisions: self.hue_divisions,
            sat_divisions: self.sat_divisions,
            val_divisions: self.val_divisions,
            deltas,
        }
    }
}

/// A parsed DNG Camera Profile (DCP) containing dual-illuminant sensor calibration.
#[derive(Debug, Clone, PartialEq)]
pub struct DcpProfile {
    pub profile_name: String,
    pub illuminant1: u16,
    pub illuminant2: Option<u16>,
    pub color_matrix1: Matrix3x3,
    pub color_matrix2: Option<Matrix3x3>,
    pub forward_matrix1: Option<Matrix3x3>,
    pub forward_matrix2: Option<Matrix3x3>,
    pub tone_curve: Option<Vec<(f32, f32)>>,
    pub hue_sat_map1: Option<DcpHueSatMap>,
    pub hue_sat_map2: Option<DcpHueSatMap>,
}

impl DcpProfile {
    /// Parses a standard `.dcp` binary file from raw bytes.
    pub fn from_bytes(data: &[u8]) -> Result<Self, DcpError> {
        if data.len() < 8 {
            return Err(DcpError::UnexpectedEof);
        }

        let is_le = match &data[0..2] {
            b"II" => true,
            b"MM" => false,
            _ => return Err(DcpError::UnsupportedByteOrder),
        };

        let read_u16 = |offset: usize| -> Result<u16, DcpError> {
            if offset + 2 > data.len() {
                return Err(DcpError::UnexpectedEof);
            }
            let bytes = [data[offset], data[offset + 1]];
            Ok(if is_le { u16::from_le_bytes(bytes) } else { u16::from_be_bytes(bytes) })
        };

        let read_u32 = |offset: usize| -> Result<u32, DcpError> {
            if offset + 4 > data.len() {
                return Err(DcpError::UnexpectedEof);
            }
            let bytes = [data[offset], data[offset + 1], data[offset + 2], data[offset + 3]];
            Ok(if is_le { u32::from_le_bytes(bytes) } else { u32::from_be_bytes(bytes) })
        };

        let read_i32 = |offset: usize| -> Result<i32, DcpError> {
            if offset + 4 > data.len() {
                return Err(DcpError::UnexpectedEof);
            }
            let bytes = [data[offset], data[offset + 1], data[offset + 2], data[offset + 3]];
            Ok(if is_le { i32::from_le_bytes(bytes) } else { i32::from_be_bytes(bytes) })
        };

        let read_f32 = |offset: usize| -> Result<f32, DcpError> {
            if offset + 4 > data.len() {
                return Err(DcpError::UnexpectedEof);
            }
            let bytes = [data[offset], data[offset + 1], data[offset + 2], data[offset + 3]];
            Ok(if is_le { f32::from_le_bytes(bytes) } else { f32::from_be_bytes(bytes) })
        };

        // Check magic number 42
        let magic = read_u16(2)?;
        if magic != 42 {
            return Err(DcpError::InvalidHeader);
        }

        let ifd_offset = read_u32(4)? as usize;
        if ifd_offset >= data.len() {
            return Err(DcpError::MalformedIfd);
        }

        let num_entries = read_u16(ifd_offset)? as usize;
        let mut entry_offset = ifd_offset + 2;

        let mut profile_name = "Embedded Camera Profile".to_string();
        let mut illuminant1 = ILLUMINANT_D65;
        let mut illuminant2 = None;
        let mut color_matrix1 = None;
        let mut color_matrix2 = None;
        let mut forward_matrix1 = None;
        let mut forward_matrix2 = None;
        let mut tone_curve = None;
        let mut hue_sat_dims: Option<(usize, usize, usize)> = None;
        let mut hue_sat_data1: Option<Vec<(f32, f32, f32)>> = None;
        let mut hue_sat_data2: Option<Vec<(f32, f32, f32)>> = None;

        for _ in 0..num_entries {
            if entry_offset + 12 > data.len() {
                return Err(DcpError::UnexpectedEof);
            }

            let tag = read_u16(entry_offset)?;
            let field_type = read_u16(entry_offset + 2)?;
            let count = read_u32(entry_offset + 4)? as usize;
            let val_or_offset = read_u32(entry_offset + 8)?;

            let resolve_offset = |bytes_needed: usize| -> Result<usize, DcpError> {
                let off = if bytes_needed <= 4 {
                    entry_offset + 8
                } else {
                    val_or_offset as usize
                };
                if off + bytes_needed > data.len() {
                    Err(DcpError::UnexpectedEof)
                } else {
                    Ok(off)
                }
            };

            // Parse 3x3 Matrix from SRATIONAL (type 10) or RATIONAL (type 5)
            let parse_matrix = |count: usize| -> Result<Option<Matrix3x3>, DcpError> {
                if count != 9 {
                    return Ok(None);
                }
                let byte_offset = resolve_offset(count * 8)?;
                let mut vals = [0.0f32; 9];
                for i in 0..9 {
                    let num = read_i32(byte_offset + i * 8)? as f32;
                    let den = read_i32(byte_offset + i * 8 + 4)? as f32;
                    vals[i] = if den != 0.0 { num / den } else { 0.0 };
                }
                Ok(Some(Matrix3x3([
                    [vals[0], vals[1], vals[2]],
                    [vals[3], vals[4], vals[5]],
                    [vals[6], vals[7], vals[8]],
                ])))
            };

            match tag {
                TAG_PROFILE_NAME => {
                    let byte_offset = resolve_offset(count)?;
                    if let Ok(s) = std::str::from_utf8(&data[byte_offset..byte_offset + count]) {
                        profile_name = s.trim_matches('\0').trim().to_string();
                    }
                }
                TAG_CALIBRATION_ILLUMINANT_1 => {
                    illuminant1 = read_u16(resolve_offset(2)?)?;
                }
                TAG_CALIBRATION_ILLUMINANT_2 => {
                    illuminant2 = Some(read_u16(resolve_offset(2)?)?);
                }
                TAG_COLOR_MATRIX_1 => {
                    color_matrix1 = parse_matrix(count)?;
                }
                TAG_COLOR_MATRIX_2 => {
                    color_matrix2 = parse_matrix(count)?;
                }
                TAG_FORWARD_MATRIX_1 => {
                    forward_matrix1 = parse_matrix(count)?;
                }
                TAG_FORWARD_MATRIX_2 => {
                    forward_matrix2 = parse_matrix(count)?;
                }
                TAG_PROFILE_TONE_CURVE => {
                    if field_type == 11 && count >= 2 && count % 2 == 0 {
                        // Array of float pairs (x, y)
                        let byte_offset = resolve_offset(count * 4)?;
                        let num_points = count / 2;
                        let mut points = Vec::with_capacity(num_points);
                        for i in 0..num_points {
                            let x = read_f32(byte_offset + (i * 2) * 4)?;
                            let y = read_f32(byte_offset + (i * 2 + 1) * 4)?;
                            points.push((x, y));
                        }
                        tone_curve = Some(points);
                    }
                }
                TAG_PROFILE_HUE_SAT_MAP_DIMS => {
                    if count == 3 {
                        let bytes_needed = if field_type == 3 { 6 } else { 12 };
                        let byte_offset = resolve_offset(bytes_needed)?;
                        let (h, s, v) = if field_type == 3 {
                            (
                                read_u16(byte_offset)? as usize,
                                read_u16(byte_offset + 2)? as usize,
                                read_u16(byte_offset + 4)? as usize,
                            )
                        } else {
                            (
                                read_u32(byte_offset)? as usize,
                                read_u32(byte_offset + 4)? as usize,
                                read_u32(byte_offset + 8)? as usize,
                            )
                        };
                        hue_sat_dims = Some((h, s, v));
                    }
                }
                TAG_PROFILE_HUE_SAT_MAP_DATA_1 => {
                    if field_type == 11 && count >= 3 && count % 3 == 0 {
                        let byte_offset = resolve_offset(count * 4)?;
                        let num_triplets = count / 3;
                        let mut triplets = Vec::with_capacity(num_triplets);
                        for i in 0..num_triplets {
                            let dh = read_f32(byte_offset + (i * 3) * 4)?;
                            let ds = read_f32(byte_offset + (i * 3 + 1) * 4)?;
                            let dv = read_f32(byte_offset + (i * 3 + 2) * 4)?;
                            triplets.push((dh, ds, dv));
                        }
                        hue_sat_data1 = Some(triplets);
                    }
                }
                TAG_PROFILE_HUE_SAT_MAP_DATA_2 => {
                    if field_type == 11 && count >= 3 && count % 3 == 0 {
                        let byte_offset = resolve_offset(count * 4)?;
                        let num_triplets = count / 3;
                        let mut triplets = Vec::with_capacity(num_triplets);
                        for i in 0..num_triplets {
                            let dh = read_f32(byte_offset + (i * 3) * 4)?;
                            let ds = read_f32(byte_offset + (i * 3 + 1) * 4)?;
                            let dv = read_f32(byte_offset + (i * 3 + 2) * 4)?;
                            triplets.push((dh, ds, dv));
                        }
                        hue_sat_data2 = Some(triplets);
                    }
                }
                _ => {}
            }

            entry_offset += 12;
        }

        let cm1 = color_matrix1.ok_or(DcpError::MissingRequiredTag("ColorMatrix1"))?;

        let hue_sat_map1 = if let (Some(dims), Some(data)) = (hue_sat_dims, hue_sat_data1) {
            if data.len() == dims.0 * dims.1 * dims.2 {
                Some(DcpHueSatMap::new(dims.0, dims.1, dims.2, data))
            } else {
                None
            }
        } else {
            None
        };

        let hue_sat_map2 = if let (Some(dims), Some(data)) = (hue_sat_dims, hue_sat_data2) {
            if data.len() == dims.0 * dims.1 * dims.2 {
                Some(DcpHueSatMap::new(dims.0, dims.1, dims.2, data))
            } else {
                None
            }
        } else {
            None
        };

        Ok(DcpProfile {
            profile_name,
            illuminant1,
            illuminant2,
            color_matrix1: cm1,
            color_matrix2,
            forward_matrix1,
            forward_matrix2,
            tone_curve,
            hue_sat_map1,
            hue_sat_map2,
        })
    }

    /// Evaluates Planckian dual-illuminant interpolation weight $w \in [0.0, 1.0]$.
    ///
    /// Per Adobe DNG Specification §6:
    /// Interpolates along the reciprocal temperature ($m = 1/T$) locus between
    /// $T_1$ (Illuminant 1, typically 2856K tungsten) and $T_2$ (Illuminant 2, typically 6504K daylight).
    pub fn interpolation_weight(&self, temp_kelvin: f32) -> f32 {
        if self.illuminant2.is_none() {
            return 0.0;
        }

        let t1 = illuminant_to_temperature(self.illuminant1);
        let t2 = illuminant_to_temperature(self.illuminant2.unwrap());

        if (t1 - t2).abs() < 10.0 {
            return 0.0;
        }

        let m1 = 1.0 / t1;
        let m2 = 1.0 / t2;
        let m = 1.0 / temp_kelvin.clamp(2000.0, 50000.0);

        ((m - m1) / (m2 - m1)).clamp(0.0, 1.0)
    }

    /// Interpolates the Forward Matrix (mapping white-balanced sensor RGB -> CIE XYZ D50).
    pub fn interpolate_forward_matrix(&self, temp_kelvin: f32) -> Option<Matrix3x3> {
        let fm1 = self.forward_matrix1?;
        if let Some(fm2) = self.forward_matrix2 {
            let w = self.interpolation_weight(temp_kelvin);
            let mut res = [[0.0f32; 3]; 3];
            for r in 0..3 {
                for c in 0..3 {
                    res[r][c] = (1.0 - w) * fm1.0[r][c] + w * fm2.0[r][c];
                }
            }
            Some(Matrix3x3(res))
        } else {
            Some(fm1)
        }
    }

    /// Interpolates the Color Matrix (mapping CIE XYZ D50 -> Camera Sensor RGB).
    pub fn interpolate_color_matrix(&self, temp_kelvin: f32) -> Matrix3x3 {
        if let Some(cm2) = self.color_matrix2 {
            let w = self.interpolation_weight(temp_kelvin);
            let mut res = [[0.0f32; 3]; 3];
            for r in 0..3 {
                for c in 0..3 {
                    res[r][c] = (1.0 - w) * self.color_matrix1.0[r][c] + w * cm2.0[r][c];
                }
            }
            Matrix3x3(res)
        } else {
            self.color_matrix1
        }
    }

    /// Derives the direct affine transformation matrix mapping
    /// **Camera Sensor RGB $\to$ Wide-Gamut Linear Rec.2020** for the given White Balance temperature.
    ///
    /// Pipeline:
    /// - If ForwardMatrix is present:
    ///   $\text{Sensor} \xrightarrow{F(T)} \text{CIE XYZ D50} \xrightarrow{\text{Bradford}} \text{CIE XYZ D65} \xrightarrow{} \text{Linear Rec.2020}$
    /// - If only ColorMatrix is present:
    ///   Derives normalized $\text{Sensor} \to \text{Rec.2020}$ inverse per Adobe DNG specification.
    pub fn interpolate_sensor_to_rec2020(&self, temp_kelvin: f32) -> Matrix3x3 {
        if let Some(fm) = self.interpolate_forward_matrix(temp_kelvin) {
            // Sensor -> XYZ D50 -> XYZ D65 -> Linear Rec.2020
            let sensor_to_xyz_d65 = BRADFORD_D50_TO_D65.multiply_matrix(&fm);
            XYZ_D65_TO_REC2020.multiply_matrix(&sensor_to_xyz_d65)
        } else {
            // Derive from ColorMatrix (XYZ D50 -> Sensor):
            let xyz2cam = self.interpolate_color_matrix(temp_kelvin);
            let rec2cam_raw = xyz2cam.multiply_matrix(&REC2020_TO_XYZ_D65);
            
            // Normalize rows so neutral [1, 1, 1] in Rec.2020 maps to neutral [1, 1, 1] in Sensor
            let norm_r = (rec2cam_raw.0[0][0] + rec2cam_raw.0[0][1] + rec2cam_raw.0[0][2]).max(1e-6);
            let norm_g = (rec2cam_raw.0[1][0] + rec2cam_raw.0[1][1] + rec2cam_raw.0[1][2]).max(1e-6);
            let norm_b = (rec2cam_raw.0[2][0] + rec2cam_raw.0[2][1] + rec2cam_raw.0[2][2]).max(1e-6);
            
            let rec2cam = Matrix3x3([
                [rec2cam_raw.0[0][0] / norm_r, rec2cam_raw.0[0][1] / norm_r, rec2cam_raw.0[0][2] / norm_r],
                [rec2cam_raw.0[1][0] / norm_g, rec2cam_raw.0[1][1] / norm_g, rec2cam_raw.0[1][2] / norm_g],
                [rec2cam_raw.0[2][0] / norm_b, rec2cam_raw.0[2][1] / norm_b, rec2cam_raw.0[2][2] / norm_b],
            ]);
            
            rec2cam.inverse().unwrap_or(Matrix3x3::IDENTITY)
        }
    }

    /// Interpolates the 3D HueSatMap delta table for the specified Kelvin temperature.
    pub fn interpolate_huesatmap(&self, temp_kelvin: f32) -> Option<DcpHueSatMap> {
        match (&self.hue_sat_map1, &self.hue_sat_map2) {
            (Some(map1), Some(map2)) => {
                let w = self.interpolation_weight(temp_kelvin);
                Some(map1.interpolate(map2, w))
            }
            (Some(map1), None) => Some(map1.clone()),
            (None, Some(map2)) => Some(map2.clone()),
            (None, None) => None,
        }
    }

    // -----------------------------------------------------------------------
    // Built-in Camera-Matching Tone Curves
    // -----------------------------------------------------------------------

    pub fn built_in_standard_tone_curve() -> Vec<(f32, f32)> {
        vec![
            (0.00, 0.000),
            (0.02, 0.008),
            (0.05, 0.028),
            (0.18, 0.190),
            (0.50, 0.540),
            (0.80, 0.840),
            (1.00, 1.000),
        ]
    }

    pub fn built_in_portrait_tone_curve() -> Vec<(f32, f32)> {
        vec![
            (0.00, 0.000),
            (0.02, 0.010),
            (0.05, 0.035),
            (0.18, 0.185),
            (0.50, 0.510),
            (0.80, 0.820),
            (1.00, 1.000),
        ]
    }

    pub fn built_in_landscape_tone_curve() -> Vec<(f32, f32)> {
        vec![
            (0.00, 0.000),
            (0.02, 0.006),
            (0.05, 0.022),
            (0.18, 0.200),
            (0.50, 0.560),
            (0.80, 0.860),
            (1.00, 1.000),
        ]
    }

    pub fn built_in_vivid_tone_curve() -> Vec<(f32, f32)> {
        vec![
            (0.00, 0.000),
            (0.02, 0.004),
            (0.05, 0.018),
            (0.18, 0.210),
            (0.50, 0.580),
            (0.80, 0.880),
            (1.00, 1.000),
        ]
    }

    // -----------------------------------------------------------------------
    // Built-in Camera-Matching HueSatMaps (Memory Color Enhancements)
    // -----------------------------------------------------------------------

    pub fn built_in_standard_huesatmap() -> DcpHueSatMap {
        let mut deltas = vec![(0.0, 1.0, 1.0); 24];
        // 30 deg (Orange/Skin): slight healthy warm saturation lift
        deltas[1 * 2 + 0] = (0.0, 1.03, 1.02);
        deltas[1 * 2 + 1] = (0.0, 1.05, 1.03);
        // 120 deg (Green/Foliage): slightly richer green
        deltas[4 * 2 + 0] = (1.5, 1.06, 0.99);
        deltas[4 * 2 + 1] = (2.0, 1.10, 0.98);
        // 210 deg (Sky Blue): shift towards azure/cyan
        deltas[7 * 2 + 0] = (-8.0, 0.92, 0.98);
        deltas[7 * 2 + 1] = (-12.0, 0.86, 0.97);
        // 240 deg (Sensor Blue): strong shift towards azure to eliminate purple cast
        deltas[8 * 2 + 0] = (-15.0, 0.82, 0.98);
        deltas[8 * 2 + 1] = (-20.0, 0.78, 0.97);
        DcpHueSatMap::new(12, 2, 1, deltas)
    }

    pub fn built_in_portrait_huesatmap() -> DcpHueSatMap {
        let mut deltas = vec![(0.0, 1.0, 1.0); 24];
        // Red (0 deg)
        deltas[0 * 2 + 0] = (1.5, 0.96, 1.02);
        deltas[0 * 2 + 1] = (2.0, 0.94, 1.03);
        // Orange (30 deg)
        deltas[1 * 2 + 0] = (0.0, 0.96, 1.03);
        deltas[1 * 2 + 1] = (0.0, 0.94, 1.04);
        // Yellow (60 deg)
        deltas[2 * 2 + 0] = (-1.5, 0.96, 1.02);
        deltas[2 * 2 + 1] = (-2.0, 0.94, 1.03);
        // Blue (210 & 240 deg): natural sky tone
        deltas[7 * 2 + 0] = (-8.0, 0.88, 0.98);
        deltas[7 * 2 + 1] = (-12.0, 0.82, 0.97);
        deltas[8 * 2 + 0] = (-15.0, 0.80, 0.98);
        deltas[8 * 2 + 1] = (-20.0, 0.75, 0.97);
        DcpHueSatMap::new(12, 2, 1, deltas)
    }

    pub fn built_in_landscape_huesatmap() -> DcpHueSatMap {
        let mut deltas = vec![(0.0, 1.0, 1.0); 24];
        // Green (120 deg)
        deltas[4 * 2 + 0] = (2.5, 1.15, 0.97);
        deltas[4 * 2 + 1] = (3.5, 1.22, 0.95);
        // Sky blue (210 & 240 deg): rich azure sky without violet shift
        deltas[7 * 2 + 0] = (-10.0, 0.98, 0.96);
        deltas[7 * 2 + 1] = (-15.0, 0.92, 0.94);
        deltas[8 * 2 + 0] = (-15.0, 0.88, 0.96);
        deltas[8 * 2 + 1] = (-20.0, 0.82, 0.94);
        DcpHueSatMap::new(12, 2, 1, deltas)
    }

    pub fn built_in_vivid_huesatmap() -> DcpHueSatMap {
        let mut deltas = vec![(0.0, 1.15, 1.02); 24];
        deltas[4 * 2 + 1] = (2.0, 1.25, 0.96);
        deltas[7 * 2 + 0] = (-10.0, 1.02, 0.96);
        deltas[7 * 2 + 1] = (-15.0, 0.98, 0.94);
        deltas[8 * 2 + 0] = (-15.0, 0.95, 0.96);
        deltas[8 * 2 + 1] = (-20.0, 0.88, 0.94);
        DcpHueSatMap::new(12, 2, 1, deltas)
    }

    // -----------------------------------------------------------------------
    // Built-in Standard Dual-Illuminant Camera Profiles
    // -----------------------------------------------------------------------

    /// Standard Canon EOS dual-illuminant profile (Illuminant A tungsten + D65 daylight).
    pub fn built_in_canon_eos() -> Self {
        Self {
            profile_name: "Canon EOS Standard".to_string(),
            illuminant1: ILLUMINANT_STANDARD_A,
            illuminant2: Some(ILLUMINANT_D65),
            color_matrix1: Matrix3x3([
                [ 0.7928, -0.1090, -0.0537],
                [-0.3894,  1.2153,  0.1982],
                [-0.0786,  0.2264,  0.7303],
            ]),
            color_matrix2: Some(Matrix3x3([
                [ 0.7034,  0.0613,  0.1995],
                [ 0.2721,  0.6409,  0.0870],
                [-0.0102, -0.0841,  0.9193],
            ])),
            forward_matrix1: Some(Matrix3x3([
                [ 0.6557,  0.1557,  0.1528],
                [ 0.2606,  0.7247,  0.0147],
                [ 0.0210, -0.1147,  0.9188],
            ])),
            forward_matrix2: Some(Matrix3x3([
                [ 0.6094,  0.2312,  0.1236],
                [ 0.2831,  0.6876,  0.0293],
                [ 0.0177, -0.0632,  0.8706],
            ])),
            tone_curve: Some(Self::built_in_standard_tone_curve()),
            hue_sat_map1: Some(Self::built_in_standard_huesatmap()),
            hue_sat_map2: Some(Self::built_in_standard_huesatmap()),
        }
    }

    /// Standard Sony Alpha dual-illuminant profile.
    pub fn built_in_sony_alpha() -> Self {
        Self {
            profile_name: "Sony Alpha Standard".to_string(),
            illuminant1: ILLUMINANT_STANDARD_A,
            illuminant2: Some(ILLUMINANT_D65),
            color_matrix1: Matrix3x3([
                [ 0.7483, -0.0851, -0.0534],
                [-0.4109,  1.2392,  0.1971],
                [-0.0701,  0.1884,  0.7071],
            ]),
            color_matrix2: Some(Matrix3x3([
                [ 0.6722,  0.0821,  0.1793],
                [ 0.2514,  0.6542,  0.0944],
                [-0.0084, -0.0782,  0.9118],
            ])),
            forward_matrix1: Some(Matrix3x3([
                [ 0.6738,  0.1412,  0.1492],
                [ 0.2741,  0.7102,  0.0157],
                [ 0.0221, -0.1082,  0.9112],
            ])),
            forward_matrix2: Some(Matrix3x3([
                [ 0.6281,  0.2114,  0.1247],
                [ 0.2911,  0.6792,  0.0297],
                [ 0.0182, -0.0594,  0.8663],
            ])),
            tone_curve: Some(Self::built_in_standard_tone_curve()),
            hue_sat_map1: Some(Self::built_in_standard_huesatmap()),
            hue_sat_map2: Some(Self::built_in_standard_huesatmap()),
        }
    }

    /// Standard Nikon Z dual-illuminant profile.
    pub fn built_in_nikon_z() -> Self {
        Self {
            profile_name: "Nikon Z Standard".to_string(),
            illuminant1: ILLUMINANT_STANDARD_A,
            illuminant2: Some(ILLUMINANT_D65),
            color_matrix1: Matrix3x3([
                [ 0.7712, -0.0984, -0.0512],
                [-0.4011,  1.2281,  0.1974],
                [-0.0742,  0.2014,  0.7182],
            ]),
            color_matrix2: Some(Matrix3x3([
                [ 0.6881,  0.0714,  0.1892],
                [ 0.2612,  0.6481,  0.0907],
                [-0.0091, -0.0811,  0.9152],
            ])),
            forward_matrix1: Some(Matrix3x3([
                [ 0.6641,  0.1482,  0.1511],
                [ 0.2672,  0.7171,  0.0152],
                [ 0.0215, -0.1112,  0.9151],
            ])),
            forward_matrix2: Some(Matrix3x3([
                [ 0.6184,  0.2211,  0.1241],
                [ 0.2871,  0.6832,  0.0295],
                [ 0.0179, -0.0612,  0.8684],
            ])),
            tone_curve: Some(Self::built_in_standard_tone_curve()),
            hue_sat_map1: Some(Self::built_in_standard_huesatmap()),
            hue_sat_map2: Some(Self::built_in_standard_huesatmap()),
        }
    }

    /// Selects an optimal built-in camera profile matching camera metadata strings.
    pub fn built_in_for_camera(model: &str) -> Self {
        let m = model.to_lowercase();
        if m.contains("canon") || m.contains("eos") {
            Self::built_in_canon_eos()
        } else if m.contains("sony") || m.contains("ilce") || m.contains("alpha") {
            Self::built_in_sony_alpha()
        } else if m.contains("nikon") {
            Self::built_in_nikon_z()
        } else {
            Self::built_in_canon_eos()
        }
    }
}
