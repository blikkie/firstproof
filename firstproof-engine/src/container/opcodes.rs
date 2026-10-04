// SPDX-License-Identifier: MIT
// Copyright 2026 First Proof Developers

//! DNG Opcode Processing Engine (Adobe DNG Specification 1.3+).
//!
//! Provides parsing and execution of embedded DNG OpcodeLists:
//! - `OpcodeList1` (Tag 51008): Pre-demosaic Bayer CFA correction (Bad pixel repair, sensor GainMap).
//! - `OpcodeList2` (Tag 51009): Post-demosaic linear RGB correction (Lens shading, flat-field GainMap).
//! - `OpcodeList3` (Tag 51022): Post-geometry rectification (WarpRectilinear, TrimBounds).

use std::fmt;

/// DNG Opcode Identifiers (DNG Specification 1.3.0.0+)
pub const OPCODE_WARP_RECTILINEAR: u32 = 1;
pub const OPCODE_WARP_FISHEYE: u32 = 2;
pub const OPCODE_FIX_VIGNETTE_RADIAL: u32 = 3;
pub const OPCODE_FIX_BAD_PIXELS_CONSTANT: u32 = 4;
pub const OPCODE_FIX_BAD_PIXELS_LIST: u32 = 5;
pub const OPCODE_TRIM_BOUNDS: u32 = 6;
pub const OPCODE_MAP_TABLE: u32 = 7;
pub const OPCODE_MAP_POLYNOMIAL: u32 = 8;
pub const OPCODE_GAIN_MAP: u32 = 9;
pub const OPCODE_DELTA_PER_ROW: u32 = 10;
pub const OPCODE_DELTA_PER_COLUMN: u32 = 11;
pub const OPCODE_SCALE_PER_ROW: u32 = 12;
pub const OPCODE_SCALE_PER_COLUMN: u32 = 13;

/// Opcode Flags
pub const FLAG_NONE: u32 = 0;
pub const FLAG_OPTIONAL: u32 = 1;
pub const FLAG_SKIP_IF_PREVIEW: u32 = 2;

/// Standard DNG Version constants (major.minor.revision.build)
pub const DNG_VERSION_1_3_0_0: u32 = 0x01030000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpcodeError {
    UnexpectedEof,
    InvalidHeader,
    InvalidDataLength { expected: usize, found: usize },
    UnsupportedOpcode { opcode_id: u32 },
}

impl fmt::Display for OpcodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEof => write!(f, "Unexpected end of DNG opcode stream"),
            Self::InvalidHeader => write!(f, "Invalid DNG opcode list header"),
            Self::InvalidDataLength { expected, found } => {
                write!(f, "Invalid opcode payload length: expected {expected}, found {found}")
            }
            Self::UnsupportedOpcode { opcode_id } => {
                write!(f, "Non-optional unsupported opcode ID: {opcode_id}")
            }
        }
    }
}

impl std::error::Error for OpcodeError {}

/// Area specification for spatially-targeted opcodes (GainMap, MapTable, etc.)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DngAreaSpec {
    pub top: i32,
    pub left: i32,
    pub bottom: i32,
    pub right: i32,
    pub plane: u32,
    pub planes: u32,
    pub row_pitch: u32,
    pub col_pitch: u32,
}

impl DngAreaSpec {
    pub const DATA_SIZE: usize = 32;

    pub fn parse(buf: &[u8]) -> Result<Self, OpcodeError> {
        if buf.len() < Self::DATA_SIZE {
            return Err(OpcodeError::UnexpectedEof);
        }
        let top = i32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let left = i32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
        let bottom = i32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]);
        let right = i32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]);
        let plane = u32::from_be_bytes([buf[16], buf[17], buf[18], buf[19]]);
        let planes = u32::from_be_bytes([buf[20], buf[21], buf[22], buf[23]]);
        let row_pitch = u32::from_be_bytes([buf[24], buf[25], buf[26], buf[27]]);
        let col_pitch = u32::from_be_bytes([buf[28], buf[29], buf[30], buf[31]]);

        Ok(Self {
            top,
            left,
            bottom,
            right,
            plane,
            planes: planes.max(1),
            row_pitch: row_pitch.max(1),
            col_pitch: col_pitch.max(1),
        })
    }

    pub fn write_to(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.top.to_be_bytes());
        out.extend_from_slice(&self.left.to_be_bytes());
        out.extend_from_slice(&self.bottom.to_be_bytes());
        out.extend_from_slice(&self.right.to_be_bytes());
        out.extend_from_slice(&self.plane.to_be_bytes());
        out.extend_from_slice(&self.planes.to_be_bytes());
        out.extend_from_slice(&self.row_pitch.to_be_bytes());
        out.extend_from_slice(&self.col_pitch.to_be_bytes());
    }

    #[inline]
    pub fn contains(&self, row: i32, col: i32, plane: u32) -> bool {
        if row < self.top || row >= self.bottom || col < self.left || col >= self.right {
            return false;
        }
        if plane < self.plane || plane >= self.plane + self.planes {
            return false;
        }
        if (row - self.top) % (self.row_pitch as i32) != 0 {
            return false;
        }
        if (col - self.left) % (self.col_pitch as i32) != 0 {
            return false;
        }
        true
    }
}

/// 2D Bilinear GainMap representation for flat-field and peripheral lens shading calibration.
#[derive(Clone, Debug, PartialEq)]
pub struct DngGainMap {
    pub points_v: u32,
    pub points_h: u32,
    pub spacing_v: f64,
    pub spacing_h: f64,
    pub origin_v: f64,
    pub origin_h: f64,
    pub planes: u32,
    pub map_gain: Vec<f32>,
}

impl DngGainMap {
    pub fn parse(buf: &[u8]) -> Result<Self, OpcodeError> {
        if buf.len() < 44 {
            return Err(OpcodeError::UnexpectedEof);
        }
        let points_v = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let points_h = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
        let spacing_v = f64::from_be_bytes([buf[8], buf[9], buf[10], buf[11], buf[12], buf[13], buf[14], buf[15]]);
        let spacing_h = f64::from_be_bytes([buf[16], buf[17], buf[18], buf[19], buf[20], buf[21], buf[22], buf[23]]);
        let origin_v = f64::from_be_bytes([buf[24], buf[25], buf[26], buf[27], buf[28], buf[29], buf[30], buf[31]]);
        let origin_h = f64::from_be_bytes([buf[32], buf[33], buf[34], buf[35], buf[36], buf[37], buf[38], buf[39]]);
        let planes = u32::from_be_bytes([buf[40], buf[41], buf[42], buf[43]]);

        if points_v == 0 || points_h == 0 || planes == 0 {
            return Err(OpcodeError::InvalidHeader);
        }

        let total_samples = (points_v as usize)
            .checked_mul(points_h as usize)
            .and_then(|v| v.checked_mul(planes as usize))
            .ok_or(OpcodeError::InvalidHeader)?;

        let expected_bytes = 44 + total_samples * 4;
        if buf.len() < expected_bytes {
            return Err(OpcodeError::InvalidDataLength {
                expected: expected_bytes,
                found: buf.len(),
            });
        }

        let mut map_gain = Vec::with_capacity(total_samples);
        let mut off = 44;
        for _ in 0..total_samples {
            let val = f32::from_be_bytes([buf[off], buf[off + 1], buf[off + 2], buf[off + 3]]);
            map_gain.push(val);
            off += 4;
        }

        let spacing_v = if points_v == 1 { 1.0 } else { spacing_v };
        let spacing_h = if points_h == 1 { 1.0 } else { spacing_h };
        let origin_v = if points_v == 1 { 0.0 } else { origin_v };
        let origin_h = if points_h == 1 { 0.0 } else { origin_h };

        Ok(Self {
            points_v,
            points_h,
            spacing_v,
            spacing_h,
            origin_v,
            origin_h,
            planes,
            map_gain,
        })
    }

    pub fn write_to(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.points_v.to_be_bytes());
        out.extend_from_slice(&self.points_h.to_be_bytes());
        out.extend_from_slice(&self.spacing_v.to_be_bytes());
        out.extend_from_slice(&self.spacing_h.to_be_bytes());
        out.extend_from_slice(&self.origin_v.to_be_bytes());
        out.extend_from_slice(&self.origin_h.to_be_bytes());
        out.extend_from_slice(&self.planes.to_be_bytes());
        for &val in &self.map_gain {
            out.extend_from_slice(&val.to_be_bytes());
        }
    }

    /// Evaluates the interpolated gain factor at (row, col) for the given color plane.
    /// Bounds defines the enclosing area `(top, left, bottom, right)`.
    #[inline]
    pub fn interpolate(
        &self,
        row: i32,
        col: i32,
        plane: u32,
        area_top: i32,
        area_left: i32,
        area_bottom: i32,
        area_right: i32,
    ) -> f32 {
        let h = (area_bottom - area_top) as f64;
        let w = (area_right - area_left) as f64;
        if h <= 0.0 || w <= 0.0 || self.points_v == 0 || self.points_h == 0 {
            return 1.0;
        }

        let y_norm = (row as f64 + 0.5 - area_top as f64) / h;
        let x_norm = (col as f64 + 0.5 - area_left as f64) / w;

        let row_idx_f = (y_norm - self.origin_v) / self.spacing_v;
        let col_idx_f = (x_norm - self.origin_h) / self.spacing_h;

        let (r1, r2, fract_r) = if row_idx_f <= 0.0 {
            (0usize, 0usize, 0.0f32)
        } else if row_idx_f >= (self.points_v - 1) as f64 {
            let last = (self.points_v - 1) as usize;
            (last, last, 0.0f32)
        } else {
            let r1 = row_idx_f.floor() as usize;
            (r1, r1 + 1, (row_idx_f - r1 as f64) as f32)
        };

        let (c1, c2, fract_c) = if col_idx_f <= 0.0 {
            (0usize, 0usize, 0.0f32)
        } else if col_idx_f >= (self.points_h - 1) as f64 {
            let last = (self.points_h - 1) as usize;
            (last, last, 0.0f32)
        } else {
            let c1 = col_idx_f.floor() as usize;
            (c1, c1 + 1, (col_idx_f - c1 as f64) as f32)
        };

        let p = (plane as usize).min((self.planes as usize).saturating_sub(1));
        let stride_plane = 1;
        let stride_col = self.planes as usize;
        let stride_row = (self.points_h as usize) * stride_col;

        let idx00 = r1 * stride_row + c1 * stride_col + p * stride_plane;
        let idx01 = r1 * stride_row + c2 * stride_col + p * stride_plane;
        let idx10 = r2 * stride_row + c1 * stride_col + p * stride_plane;
        let idx11 = r2 * stride_row + c2 * stride_col + p * stride_plane;

        let g00 = self.map_gain.get(idx00).copied().unwrap_or(1.0);
        let g01 = self.map_gain.get(idx01).copied().unwrap_or(1.0);
        let g10 = self.map_gain.get(idx10).copied().unwrap_or(1.0);
        let g11 = self.map_gain.get(idx11).copied().unwrap_or(1.0);

        let g_top = (1.0 - fract_c) * g00 + fract_c * g01;
        let g_bot = (1.0 - fract_c) * g10 + fract_c * g11;

        (1.0 - fract_r) * g_top + fract_r * g_bot
    }
}

/// Defective single pixel coordinate
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DngBadPoint {
    pub row: i32,
    pub col: i32,
}

/// Defective rectangle coordinate (single dead column or row)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DngBadRect {
    pub top: i32,
    pub left: i32,
    pub bottom: i32,
    pub right: i32,
}

/// Strongly-typed DNG Opcode variant
#[derive(Clone, Debug, PartialEq)]
pub enum DngOpcode {
    FixBadPixelsConstant {
        min_version: u32,
        flags: u32,
        constant: u32,
        bayer_phase: u32,
    },
    FixBadPixelsList {
        min_version: u32,
        flags: u32,
        bayer_phase: u32,
        bad_points: Vec<DngBadPoint>,
        bad_rects: Vec<DngBadRect>,
    },
    GainMap {
        min_version: u32,
        flags: u32,
        area: DngAreaSpec,
        map: DngGainMap,
    },
    WarpRectilinear {
        min_version: u32,
        flags: u32,
        planes: u32,
        coefficients: [f64; 4],
        center: [f64; 2],
    },
    Unknown {
        opcode_id: u32,
        min_version: u32,
        flags: u32,
        data: Vec<u8>,
    },
}

impl DngOpcode {
    pub fn opcode_id(&self) -> u32 {
        match self {
            Self::FixBadPixelsConstant { .. } => OPCODE_FIX_BAD_PIXELS_CONSTANT,
            Self::FixBadPixelsList { .. } => OPCODE_FIX_BAD_PIXELS_LIST,
            Self::GainMap { .. } => OPCODE_GAIN_MAP,
            Self::WarpRectilinear { .. } => OPCODE_WARP_RECTILINEAR,
            Self::Unknown { opcode_id, .. } => *opcode_id,
        }
    }

    pub fn min_version(&self) -> u32 {
        match self {
            Self::FixBadPixelsConstant { min_version, .. } => *min_version,
            Self::FixBadPixelsList { min_version, .. } => *min_version,
            Self::GainMap { min_version, .. } => *min_version,
            Self::WarpRectilinear { min_version, .. } => *min_version,
            Self::Unknown { min_version, .. } => *min_version,
        }
    }

    pub fn flags(&self) -> u32 {
        match self {
            Self::FixBadPixelsConstant { flags, .. } => *flags,
            Self::FixBadPixelsList { flags, .. } => *flags,
            Self::GainMap { flags, .. } => *flags,
            Self::WarpRectilinear { flags, .. } => *flags,
            Self::Unknown { flags, .. } => *flags,
        }
    }

    pub fn is_optional(&self) -> bool {
        (self.flags() & FLAG_OPTIONAL) != 0
    }
}

/// A sequential list of DNG opcodes (`OpcodeList1`, `OpcodeList2`, or `OpcodeList3`)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DngOpcodeList {
    pub opcodes: Vec<DngOpcode>,
}

impl DngOpcodeList {
    pub fn new() -> Self {
        Self { opcodes: Vec::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.opcodes.is_empty()
    }

    pub fn len(&self) -> usize {
        self.opcodes.len()
    }

    /// Parses an OpcodeList from raw big-endian byte stream per DNG Specification.
    pub fn parse(buf: &[u8]) -> Result<Self, OpcodeError> {
        if buf.is_empty() {
            return Ok(Self::new());
        }
        if buf.len() < 4 {
            return Err(OpcodeError::UnexpectedEof);
        }

        let count = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
        let mut opcodes = Vec::with_capacity(count);
        let mut off = 4;

        for _ in 0..count {
            if off + 16 > buf.len() {
                return Err(OpcodeError::UnexpectedEof);
            }
            let opcode_id = u32::from_be_bytes([buf[off], buf[off + 1], buf[off + 2], buf[off + 3]]);
            let min_version = u32::from_be_bytes([buf[off + 4], buf[off + 5], buf[off + 6], buf[off + 7]]);
            let flags = u32::from_be_bytes([buf[off + 8], buf[off + 9], buf[off + 10], buf[off + 11]]);
            let byte_count = u32::from_be_bytes([buf[off + 12], buf[off + 13], buf[off + 14], buf[off + 15]]) as usize;
            off += 16;

            if off + byte_count > buf.len() {
                return Err(OpcodeError::InvalidDataLength {
                    expected: off + byte_count,
                    found: buf.len(),
                });
            }

            let data = &buf[off..off + byte_count];
            off += byte_count;

            match opcode_id {
                OPCODE_FIX_BAD_PIXELS_CONSTANT => {
                    if byte_count >= 8 {
                        let constant = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
                        let bayer_phase = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
                        opcodes.push(DngOpcode::FixBadPixelsConstant {
                            min_version,
                            flags,
                            constant,
                            bayer_phase,
                        });
                    } else if (flags & FLAG_OPTIONAL) == 0 {
                        return Err(OpcodeError::InvalidDataLength {
                            expected: 8,
                            found: byte_count,
                        });
                    }
                }
                OPCODE_FIX_BAD_PIXELS_LIST => {
                    if byte_count >= 12 {
                        let bayer_phase = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
                        let p_count = u32::from_be_bytes([data[4], data[5], data[6], data[7]]) as usize;
                        let r_count = u32::from_be_bytes([data[8], data[9], data[10], data[11]]) as usize;

                        let expected = 12 + p_count * 8 + r_count * 16;
                        if byte_count >= expected {
                            let mut bad_points = Vec::with_capacity(p_count);
                            let mut p_off = 12;
                            for _ in 0..p_count {
                                let row = i32::from_be_bytes([data[p_off], data[p_off + 1], data[p_off + 2], data[p_off + 3]]);
                                let col = i32::from_be_bytes([data[p_off + 4], data[p_off + 5], data[p_off + 6], data[p_off + 7]]);
                                bad_points.push(DngBadPoint { row, col });
                                p_off += 8;
                            }

                            let mut bad_rects = Vec::with_capacity(r_count);
                            for _ in 0..r_count {
                                let top = i32::from_be_bytes([data[p_off], data[p_off + 1], data[p_off + 2], data[p_off + 3]]);
                                let left = i32::from_be_bytes([data[p_off + 4], data[p_off + 5], data[p_off + 6], data[p_off + 7]]);
                                let bottom = i32::from_be_bytes([data[p_off + 8], data[p_off + 9], data[p_off + 10], data[p_off + 11]]);
                                let right = i32::from_be_bytes([data[p_off + 12], data[p_off + 13], data[p_off + 14], data[p_off + 15]]);
                                bad_rects.push(DngBadRect { top, left, bottom, right });
                                p_off += 16;
                            }

                            opcodes.push(DngOpcode::FixBadPixelsList {
                                min_version,
                                flags,
                                bayer_phase,
                                bad_points,
                                bad_rects,
                            });
                        } else if (flags & FLAG_OPTIONAL) == 0 {
                            return Err(OpcodeError::InvalidDataLength {
                                expected,
                                found: byte_count,
                            });
                        }
                    } else if (flags & FLAG_OPTIONAL) == 0 {
                        return Err(OpcodeError::InvalidDataLength {
                            expected: 12,
                            found: byte_count,
                        });
                    }
                }
                OPCODE_GAIN_MAP => {
                    if byte_count >= DngAreaSpec::DATA_SIZE + 44 {
                        let area = DngAreaSpec::parse(&data[..DngAreaSpec::DATA_SIZE])?;
                        let map = DngGainMap::parse(&data[DngAreaSpec::DATA_SIZE..])?;
                        opcodes.push(DngOpcode::GainMap {
                            min_version,
                            flags,
                            area,
                            map,
                        });
                    } else if (flags & FLAG_OPTIONAL) == 0 {
                        return Err(OpcodeError::InvalidDataLength {
                            expected: DngAreaSpec::DATA_SIZE + 44,
                            found: byte_count,
                        });
                    }
                }
                OPCODE_WARP_RECTILINEAR => {
                    // WarpRectilinear: planes (4 bytes), coefficients (k0, k1, k2, k3 = 32 bytes), center (cx, cy = 16 bytes)
                    if byte_count >= 52 {
                        let planes = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
                        let k0 = f64::from_be_bytes([data[4], data[5], data[6], data[7], data[8], data[9], data[10], data[11]]);
                        let k1 = f64::from_be_bytes([data[12], data[13], data[14], data[15], data[16], data[17], data[18], data[19]]);
                        let k2 = f64::from_be_bytes([data[20], data[21], data[22], data[23], data[24], data[25], data[26], data[27]]);
                        let k3 = f64::from_be_bytes([data[28], data[29], data[30], data[31], data[32], data[33], data[34], data[35]]);
                        let cx = f64::from_be_bytes([data[36], data[37], data[38], data[39], data[40], data[41], data[42], data[43]]);
                        let cy = f64::from_be_bytes([data[44], data[45], data[46], data[47], data[48], data[49], data[50], data[51]]);
                        opcodes.push(DngOpcode::WarpRectilinear {
                            min_version,
                            flags,
                            planes,
                            coefficients: [k0, k1, k2, k3],
                            center: [cx, cy],
                        });
                    } else if (flags & FLAG_OPTIONAL) == 0 {
                        return Err(OpcodeError::InvalidDataLength {
                            expected: 52,
                            found: byte_count,
                        });
                    }
                }
                _ => {
                    // Unrecognized or other opcodes (MapTable, DeltaPerRow, etc.)
                    if (flags & FLAG_OPTIONAL) != 0 {
                        opcodes.push(DngOpcode::Unknown {
                            opcode_id,
                            min_version,
                            flags,
                            data: data.to_vec(),
                        });
                    } else {
                        return Err(OpcodeError::UnsupportedOpcode { opcode_id });
                    }
                }
            }
        }

        Ok(Self { opcodes })
    }

    /// Serializes the OpcodeList into standard big-endian DNG bytecode.
    pub fn serialize(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&(self.opcodes.len() as u32).to_be_bytes());

        for op in &self.opcodes {
            out.extend_from_slice(&op.opcode_id().to_be_bytes());
            out.extend_from_slice(&op.min_version().to_be_bytes());
            out.extend_from_slice(&op.flags().to_be_bytes());

            match op {
                DngOpcode::FixBadPixelsConstant { constant, bayer_phase, .. } => {
                    out.extend_from_slice(&8u32.to_be_bytes());
                    out.extend_from_slice(&constant.to_be_bytes());
                    out.extend_from_slice(&bayer_phase.to_be_bytes());
                }
                DngOpcode::FixBadPixelsList { bayer_phase, bad_points, bad_rects, .. } => {
                    let len = (12 + bad_points.len() * 8 + bad_rects.len() * 16) as u32;
                    out.extend_from_slice(&len.to_be_bytes());
                    out.extend_from_slice(&bayer_phase.to_be_bytes());
                    out.extend_from_slice(&(bad_points.len() as u32).to_be_bytes());
                    out.extend_from_slice(&(bad_rects.len() as u32).to_be_bytes());
                    for pt in bad_points {
                        out.extend_from_slice(&pt.row.to_be_bytes());
                        out.extend_from_slice(&pt.col.to_be_bytes());
                    }
                    for r in bad_rects {
                        out.extend_from_slice(&r.top.to_be_bytes());
                        out.extend_from_slice(&r.left.to_be_bytes());
                        out.extend_from_slice(&r.bottom.to_be_bytes());
                        out.extend_from_slice(&r.right.to_be_bytes());
                    }
                }
                DngOpcode::GainMap { area, map, .. } => {
                    let mut payload = Vec::new();
                    area.write_to(&mut payload);
                    map.write_to(&mut payload);
                    out.extend_from_slice(&(payload.len() as u32).to_be_bytes());
                    out.extend_from_slice(&payload);
                }
                DngOpcode::WarpRectilinear { planes, coefficients, center, .. } => {
                    out.extend_from_slice(&52u32.to_be_bytes());
                    out.extend_from_slice(&planes.to_be_bytes());
                    for &c in coefficients {
                        out.extend_from_slice(&c.to_be_bytes());
                    }
                    out.extend_from_slice(&center[0].to_be_bytes());
                    out.extend_from_slice(&center[1].to_be_bytes());
                }
                DngOpcode::Unknown { data, .. } => {
                    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
                    out.extend_from_slice(data);
                }
            }
        }

        out
    }
}

/// Helper function to check if photosite at (row, col) with given Bayer phase is Green.
/// Formula from Adobe DNG SDK: ((row + col + bayerPhase + (bayerPhase >> 1)) & 1) == 0.
#[inline]
pub fn is_bayer_green(row: i32, col: i32, bayer_phase: u32) -> bool {
    let term = (row as u32).wrapping_add(col as u32).wrapping_add(bayer_phase).wrapping_add(bayer_phase >> 1);
    (term & 1) == 0
}

/// Applies DNG OpcodeList1 (pre-demosaic / CFA raw stage) to a raw Bayer sensor buffer.
/// Executes bad pixel in-painting and radial GainMap vignetting correction directly in-place.
pub fn apply_opcode_list_1_to_raw(
    sensor_data: &mut [u16],
    width: usize,
    height: usize,
    black_levels: [f32; 4],
    white_levels: [f32; 4],
    opcodes: &DngOpcodeList,
) {
    if sensor_data.is_empty() || width == 0 || height == 0 {
        return;
    }

    for opcode in &opcodes.opcodes {
        match opcode {
            DngOpcode::FixBadPixelsConstant { constant, bayer_phase, .. } => {
                let bad_val = *constant as u16;
                let w = width as i32;
                let h = height as i32;

                // Process interior pixels first
                for row in 0..h {
                    for col in 0..w {
                        let idx = (row as usize) * width + (col as usize);
                        if sensor_data[idx] == bad_val {
                            let mut sum = 0u32;
                            let mut count = 0u32;

                            if is_bayer_green(row, col, *bayer_phase) {
                                // 4 diagonal neighbors for green
                                let deltas = [(-1, -1), (-1, 1), (1, -1), (1, 1)];
                                for (dr, dc) in deltas {
                                    let nr = row + dr;
                                    let nc = col + dc;
                                    if nr >= 0 && nr < h && nc >= 0 && nc < w {
                                        let n_val = sensor_data[(nr as usize) * width + (nc as usize)];
                                        if n_val != bad_val {
                                            sum += n_val as u32;
                                            count += 1;
                                        }
                                    }
                                }
                            } else {
                                // 4 rectilinear distance-2 neighbors for same color (Red or Blue)
                                let deltas = [(-2, 0), (2, 0), (0, -2), (0, 2)];
                                for (dr, dc) in deltas {
                                    let nr = row + dr;
                                    let nc = col + dc;
                                    if nr >= 0 && nr < h && nc >= 0 && nc < w {
                                        let n_val = sensor_data[(nr as usize) * width + (nc as usize)];
                                        if n_val != bad_val {
                                            sum += n_val as u32;
                                            count += 1;
                                        }
                                    }
                                }
                            }

                            if count > 0 {
                                sensor_data[idx] = ((sum + (count >> 1)) / count) as u16;
                            }
                        }
                    }
                }
            }
            DngOpcode::FixBadPixelsList { bayer_phase, bad_points, bad_rects, .. } => {
                let w = width as i32;
                let h = height as i32;

                // 1. Repair discrete bad points
                for pt in bad_points {
                    let row = pt.row;
                    let col = pt.col;
                    if row >= 0 && row < h && col >= 0 && col < w {
                        let idx = (row as usize) * width + (col as usize);
                        let mut sum = 0u32;
                        let mut count = 0u32;

                        if is_bayer_green(row, col, *bayer_phase) {
                            let deltas = [(-1, -1), (-1, 1), (1, -1), (1, 1)];
                            for (dr, dc) in deltas {
                                let nr = row + dr;
                                let nc = col + dc;
                                if nr >= 0 && nr < h && nc >= 0 && nc < w {
                                    sum += sensor_data[(nr as usize) * width + (nc as usize)] as u32;
                                    count += 1;
                                }
                            }
                        } else {
                            let deltas = [(-2, 0), (2, 0), (0, -2), (0, 2)];
                            for (dr, dc) in deltas {
                                let nr = row + dr;
                                let nc = col + dc;
                                if nr >= 0 && nr < h && nc >= 0 && nc < w {
                                    sum += sensor_data[(nr as usize) * width + (nc as usize)] as u32;
                                    count += 1;
                                }
                            }
                        }

                        if count > 0 {
                            sensor_data[idx] = ((sum + (count >> 1)) / count) as u16;
                        }
                    }
                }

                // 2. Repair bad rectangles (dead columns/rows)
                for rect in bad_rects {
                    let t = rect.top.max(0);
                    let b = rect.bottom.min(h);
                    let l = rect.left.max(0);
                    let r = rect.right.min(w);

                    for row in t..b {
                        for col in l..r {
                            let idx = (row as usize) * width + (col as usize);
                            let is_col_defect = (rect.right - rect.left) <= 2;
                            let mut sum = 0u32;
                            let mut count = 0u32;

                            if is_col_defect {
                                // Interpolate horizontally from outside the defect
                                for dc in [-2, 2] {
                                    let nc = col + dc;
                                    if nc >= 0 && nc < w && (nc < rect.left || nc >= rect.right) {
                                        sum += sensor_data[(row as usize) * width + (nc as usize)] as u32;
                                        count += 1;
                                    }
                                }
                            } else {
                                // Interpolate vertically
                                for dr in [-2, 2] {
                                    let nr = row + dr;
                                    if nr >= 0 && nr < h && (nr < rect.top || nr >= rect.bottom) {
                                        sum += sensor_data[(nr as usize) * width + (col as usize)] as u32;
                                        count += 1;
                                    }
                                }
                            }

                            if count > 0 {
                                sensor_data[idx] = ((sum + (count >> 1)) / count) as u16;
                            }
                        }
                    }
                }
            }
            DngOpcode::GainMap { area, map, .. } => {
                let w = width as i32;
                let h = height as i32;

                let start_row = area.top.max(0);
                let end_row = area.bottom.min(h);
                let start_col = area.left.max(0);
                let end_col = area.right.min(w);

                for row in start_row..end_row {
                    if (row - area.top) % (area.row_pitch as i32) != 0 {
                        continue;
                    }
                    for col in start_col..end_col {
                        if (col - area.left) % (area.col_pitch as i32) != 0 {
                            continue;
                        }

                        let cfa_idx = ((row as usize) % 2) * 2 + ((col as usize) % 2);
                        let black = black_levels[cfa_idx];
                        let white = white_levels[cfa_idx];

                        let gain = map.interpolate(
                            row,
                            col,
                            0, // Single plane or CFA channel
                            area.top,
                            area.left,
                            area.bottom,
                            area.right,
                        );

                        let idx = (row as usize) * width + (col as usize);
                        let val = sensor_data[idx] as f32;
                        let val_lin = (val - black).max(0.0) * gain;
                        sensor_data[idx] = (val_lin + black).clamp(0.0, white).round() as u16;
                    }
                }
            }
            _ => {}
        }
    }
}

/// Applies DNG OpcodeList2 (post-demosaic linear RGB stage) to a demosaiced RGB float buffer.
/// `rgb_data` is interleaved linear RGB: `[r0, g0, b0, r1, g1, b1, ...]`.
pub fn apply_opcode_list_2_to_rgb(
    rgb_data: &mut [f32],
    width: usize,
    height: usize,
    opcodes: &DngOpcodeList,
) {
    if rgb_data.is_empty() || width == 0 || height == 0 {
        return;
    }

    for opcode in &opcodes.opcodes {
        if let DngOpcode::GainMap { area, map, .. } = opcode {
            let w = width as i32;
            let h = height as i32;

            let start_row = area.top.max(0);
            let end_row = area.bottom.min(h);
            let start_col = area.left.max(0);
            let end_col = area.right.min(w);

            for row in start_row..end_row {
                if (row - area.top) % (area.row_pitch as i32) != 0 {
                    continue;
                }
                for col in start_col..end_col {
                    if (col - area.left) % (area.col_pitch as i32) != 0 {
                        continue;
                    }

                    let pixel_idx = ((row as usize) * width + (col as usize)) * 3;

                    for plane in 0..3 {
                        let gain = map.interpolate(
                            row,
                            col,
                            plane,
                            area.top,
                            area.left,
                            area.bottom,
                            area.right,
                        );
                        if pixel_idx + (plane as usize) < rgb_data.len() {
                            rgb_data[pixel_idx + (plane as usize)] *= gain;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_opcode_list() {
        let list = DngOpcodeList::parse(&[]).expect("Empty parse");
        assert!(list.is_empty());
        assert_eq!(list.serialize(), 0u32.to_be_bytes());
    }

    #[test]
    fn test_fix_bad_pixels_constant_roundtrip() {
        let opcode = DngOpcode::FixBadPixelsConstant {
            min_version: DNG_VERSION_1_3_0_0,
            flags: FLAG_NONE,
            constant: 0,
            bayer_phase: 0,
        };
        let mut list = DngOpcodeList::new();
        list.opcodes.push(opcode.clone());

        let bytes = list.serialize();
        let parsed = DngOpcodeList::parse(&bytes).expect("Parse serialized opcode list");

        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed.opcodes[0], opcode);
    }

    #[test]
    fn test_gain_map_interpolation() {
        let area = DngAreaSpec {
            top: 0,
            left: 0,
            bottom: 100,
            right: 100,
            plane: 0,
            planes: 1,
            row_pitch: 1,
            col_pitch: 1,
        };
        // 2x2 Gain map: center/corners
        let map = DngGainMap {
            points_v: 2,
            points_h: 2,
            spacing_v: 1.0,
            spacing_h: 1.0,
            origin_v: 0.0,
            origin_h: 0.0,
            planes: 1,
            map_gain: vec![
                1.5, 2.0, // Top row: (0,0)=1.5, (0,1)=2.0
                1.0, 1.2, // Bottom row: (1,0)=1.0, (1,1)=1.2
            ],
        };

        // Corner 0,0
        let g00 = map.interpolate(0, 0, 0, area.top, area.left, area.bottom, area.right);
        assert!((g00 - 1.5).abs() < 0.02, "Expected corner ~1.5, got {g00}");

        // Corner 99, 99
        let g11 = map.interpolate(99, 99, 0, area.top, area.left, area.bottom, area.right);
        assert!((g11 - 1.2).abs() < 0.02, "Expected corner ~1.2, got {g11}");

        // Midpoint
        let g_mid = map.interpolate(50, 50, 0, area.top, area.left, area.bottom, area.right);
        // Average of corners = (1.5 + 2.0 + 1.0 + 1.2) / 4 = 1.425
        assert!((g_mid - 1.425).abs() < 0.05, "Expected mid ~1.425, got {g_mid}");
    }

    #[test]
    fn test_bad_pixel_repair_green_and_red() {
        let mut raw = vec![1000u16; 16]; // 4x4 image
        // In RGGB:
        // (0,0)=R (0,1)=G (0,2)=R (0,3)=G
        // (1,0)=G (1,1)=B (1,2)=G (1,3)=B
        // (2,0)=R (2,1)=G (2,2)=R (2,3)=G
        // (3,0)=G (3,1)=B (3,2)=G (3,3)=B

        // Set one green pixel to dead (0)
        raw[1 * 4 + 2] = 0; // (1, 2) is Green
        // Set one red pixel to dead (0)
        raw[2 * 4 + 2] = 0; // (2, 2) is Red

        let mut list = DngOpcodeList::new();
        list.opcodes.push(DngOpcode::FixBadPixelsConstant {
            min_version: DNG_VERSION_1_3_0_0,
            flags: FLAG_NONE,
            constant: 0,
            bayer_phase: 0,
        });

        apply_opcode_list_1_to_raw(&mut raw, 4, 4, [0.0; 4], [65535.0; 4], &list);

        assert_eq!(raw[1 * 4 + 2], 1000, "Green dead pixel should be restored to 1000");
        assert_eq!(raw[2 * 4 + 2], 1000, "Red dead pixel should be restored to 1000");
    }
}
