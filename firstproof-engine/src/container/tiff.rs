// SPDX-License-Identifier: MIT
// Copyright 2026 First Proof Developers

//! Safe, zero-allocation-friendly TIFF & IFD container parser operating directly on byte slices.

use std::collections::{HashMap, HashSet};

/// Supported TIFF Data Types (per TIFF 6.0 Specification)
pub const TYPE_BYTE: u16 = 1;
pub const TYPE_ASCII: u16 = 2;
pub const TYPE_SHORT: u16 = 3;
pub const TYPE_LONG: u16 = 4;
pub const TYPE_RATIONAL: u16 = 5;
pub const TYPE_SBYTE: u16 = 6;
pub const TYPE_UNDEFINED: u16 = 7;
pub const TYPE_SSHORT: u16 = 8;
pub const TYPE_SLONG: u16 = 9;
pub const TYPE_SRATIONAL: u16 = 10;
pub const TYPE_FLOAT: u16 = 11;
pub const TYPE_DOUBLE: u16 = 12;

/// A raw 12-byte IFD Directory Entry
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IfdEntry {
    pub tag: u16,
    pub typ: u16,
    pub count: u32,
    pub offset_or_val: u32,
}

impl IfdEntry {
    /// Computes the total byte size of this entry's data payload.
    pub fn byte_size(&self) -> usize {
        let type_size = match self.typ {
            TYPE_BYTE | TYPE_ASCII | TYPE_SBYTE | TYPE_UNDEFINED => 1,
            TYPE_SHORT | TYPE_SSHORT => 2,
            TYPE_LONG | TYPE_SLONG | TYPE_FLOAT => 4,
            TYPE_RATIONAL | TYPE_SRATIONAL | TYPE_DOUBLE => 8,
            _ => 1,
        };
        (self.count as usize).saturating_mul(type_size)
    }

    /// Resolves the slice containing this entry's data payload.
    pub fn data_slice<'a>(&self, buf: &'a [u8]) -> Option<&'a [u8]> {
        let size = self.byte_size();
        if size == 0 {
            return None;
        }
        if size <= 4 {
            // Value is embedded directly in the offset_or_val field (first `size` bytes)
            // Note: In TIFF, small values are left-aligned in the 4-byte offset field!
            // When parsed via read_u32, little-endian vs big-endian ordering is preserved.
            // But for raw slice access, the position depends on where this entry was in memory.
            // Instead, we extract directly from offset_or_val bytes:
            None // Callers should use specialized typed readers (as_u32, as_f32) for size <= 4
        } else {
            let start = self.offset_or_val as usize;
            let end = start.checked_add(size)?;
            if end <= buf.len() {
                Some(&buf[start..end])
            } else {
                None
            }
        }
    }

    /// Resolves the raw byte payload of this entry (handling both embedded and offset data).
    pub fn raw_bytes(&self, buf: &[u8], is_le: bool) -> Option<Vec<u8>> {
        let size = self.byte_size();
        if size == 0 {
            return Some(Vec::new());
        }
        if size <= 4 {
            let bytes = if is_le {
                self.offset_or_val.to_le_bytes()
            } else {
                self.offset_or_val.to_be_bytes()
            };
            Some(bytes[..size].to_vec())
        } else {
            self.data_slice(buf).map(|s| s.to_vec())
        }
    }

    /// Reads single unsigned integer value (SHORT or LONG).
    pub fn as_u32(&self, buf: &[u8], is_le: bool) -> Option<u32> {
        if self.count == 0 {
            return None;
        }
        match self.typ {
            TYPE_SHORT => {
                if self.count == 1 {
                    // Stored in low 16 bits of offset_or_val if LE
                    Some(self.offset_or_val as u16 as u32)
                } else {
                    let slice = self.data_slice(buf)?;
                    if slice.len() >= 2 {
                        Some(read_u16(slice, 0, is_le)? as u32)
                    } else {
                        None
                    }
                }
            }
            TYPE_LONG => {
                if self.count == 1 {
                    Some(self.offset_or_val)
                } else {
                    let slice = self.data_slice(buf)?;
                    read_u32(slice, 0, is_le)
                }
            }
            TYPE_BYTE => Some(self.offset_or_val as u8 as u32),
            _ => None,
        }
    }

    /// Reads multiple unsigned integers (SHORT or LONG array).
    pub fn as_u32_vec(&self, buf: &[u8], is_le: bool) -> Option<Vec<u32>> {
        let count = self.count as usize;
        if count == 0 || count > 100_000 {
            return None;
        }
        let mut out = Vec::with_capacity(count);

        if self.typ == TYPE_SHORT {
            if count == 1 {
                out.push(self.offset_or_val as u16 as u32);
            } else if count == 2 && self.byte_size() <= 4 {
                // Two shorts packed in offset_or_val
                let val0 = (self.offset_or_val & 0xFFFF) as u16 as u32;
                let val1 = ((self.offset_or_val >> 16) & 0xFFFF) as u16 as u32;
                out.push(val0);
                out.push(val1);
            } else {
                let slice = self.data_slice(buf)?;
                for i in 0..count {
                    out.push(read_u16(slice, i * 2, is_le)? as u32);
                }
            }
        } else if self.typ == TYPE_LONG {
            if count == 1 {
                out.push(self.offset_or_val);
            } else {
                let slice = self.data_slice(buf)?;
                for i in 0..count {
                    out.push(read_u32(slice, i * 4, is_le)?);
                }
            }
        } else if self.typ == TYPE_BYTE {
            if count <= 4 {
                let bytes = if is_le {
                    self.offset_or_val.to_le_bytes()
                } else {
                    self.offset_or_val.to_be_bytes()
                };
                for i in 0..count {
                    out.push(bytes[i] as u32);
                }
            } else {
                let slice = self.data_slice(buf)?;
                for &b in slice.iter().take(count) {
                    out.push(b as u32);
                }
            }
        } else {
            return None;
        }
        Some(out)
    }

    /// Reads floating point scalar (FLOAT, DOUBLE, RATIONAL, SRATIONAL, SHORT, LONG).
    pub fn as_f32(&self, buf: &[u8], is_le: bool) -> Option<f32> {
        if self.count == 0 {
            return None;
        }
        match self.typ {
            TYPE_RATIONAL => {
                let slice = self.data_slice(buf)?;
                if slice.len() >= 8 {
                    let num = read_u32(slice, 0, is_le)? as f64;
                    let den = read_u32(slice, 4, is_le)? as f64;
                    if den != 0.0 {
                        Some((num / den) as f32)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            TYPE_SRATIONAL => {
                let slice = self.data_slice(buf)?;
                if slice.len() >= 8 {
                    let num = read_i32(slice, 0, is_le)? as f64;
                    let den = read_i32(slice, 4, is_le)? as f64;
                    if den != 0.0 {
                        Some((num / den) as f32)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            TYPE_FLOAT => {
                if self.count == 1 {
                    let bytes = if is_le {
                        self.offset_or_val.to_le_bytes()
                    } else {
                        self.offset_or_val.to_be_bytes()
                    };
                    Some(f32::from_ne_bytes(bytes))
                } else {
                    let slice = self.data_slice(buf)?;
                    if slice.len() >= 4 {
                        let bytes = [slice[0], slice[1], slice[2], slice[3]];
                        Some(if is_le { f32::from_le_bytes(bytes) } else { f32::from_be_bytes(bytes) })
                    } else {
                        None
                    }
                }
            }
            TYPE_DOUBLE => {
                let slice = self.data_slice(buf)?;
                if slice.len() >= 8 {
                    let mut b = [0u8; 8];
                    b.copy_from_slice(&slice[0..8]);
                    let d = if is_le { f64::from_le_bytes(b) } else { f64::from_be_bytes(b) };
                    Some(d as f32)
                } else {
                    None
                }
            }
            TYPE_SHORT | TYPE_LONG => self.as_u32(buf, is_le).map(|v| v as f32),
            _ => None,
        }
    }

    /// Reads floating point array (RATIONAL, SRATIONAL, FLOAT, DOUBLE, SHORT).
    pub fn as_f32_vec(&self, buf: &[u8], is_le: bool) -> Option<Vec<f32>> {
        let count = self.count as usize;
        if count == 0 || count > 100_000 {
            return None;
        }

        if self.typ == TYPE_RATIONAL {
            let slice = self.data_slice(buf)?;
            let mut out = Vec::with_capacity(count);
            for i in 0..count {
                let num = read_u32(slice, i * 8, is_le)? as f64;
                let den = read_u32(slice, i * 8 + 4, is_le)? as f64;
                out.push(if den != 0.0 { (num / den) as f32 } else { 0.0 });
            }
            Some(out)
        } else if self.typ == TYPE_SRATIONAL {
            let slice = self.data_slice(buf)?;
            let mut out = Vec::with_capacity(count);
            for i in 0..count {
                let num = read_i32(slice, i * 8, is_le)? as f64;
                let den = read_i32(slice, i * 8 + 4, is_le)? as f64;
                out.push(if den != 0.0 { (num / den) as f32 } else { 0.0 });
            }
            Some(out)
        } else if self.typ == TYPE_FLOAT {
            let slice = self.data_slice(buf)?;
            let mut out = Vec::with_capacity(count);
            for i in 0..count {
                let off = i * 4;
                if off + 4 <= slice.len() {
                    let b = [slice[off], slice[off+1], slice[off+2], slice[off+3]];
                    out.push(if is_le { f32::from_le_bytes(b) } else { f32::from_be_bytes(b) });
                }
            }
            Some(out)
        } else if self.typ == TYPE_DOUBLE {
            let slice = self.data_slice(buf)?;
            let mut out = Vec::with_capacity(count);
            for i in 0..count {
                let off = i * 8;
                if off + 8 <= slice.len() {
                    let mut b = [0u8; 8];
                    b.copy_from_slice(&slice[off..off+8]);
                    let d = if is_le { f64::from_le_bytes(b) } else { f64::from_be_bytes(b) };
                    out.push(d as f32);
                }
            }
            Some(out)
        } else if self.typ == TYPE_SHORT || self.typ == TYPE_LONG {
            self.as_u32_vec(buf, is_le).map(|v| v.into_iter().map(|x| x as f32).collect())
        } else {
            None
        }
    }

    /// Reads 3x3 matrix from 9-element RATIONAL or SRATIONAL array.
    pub fn as_matrix3x3(&self, buf: &[u8], is_le: bool) -> Option<[[f32; 3]; 3]> {
        if self.count != 9 {
            return None;
        }
        let vals = self.as_f32_vec(buf, is_le)?;
        if vals.len() == 9 {
            Some([
                [vals[0], vals[1], vals[2]],
                [vals[3], vals[4], vals[5]],
                [vals[6], vals[7], vals[8]],
            ])
        } else {
            None
        }
    }

    /// Reads ASCII string (stripping trailing null bytes).
    pub fn as_string(&self, buf: &[u8]) -> Option<String> {
        if self.typ != TYPE_ASCII {
            return None;
        }
        let count = self.count as usize;
        if count == 0 {
            return None;
        }
        let raw = if count <= 4 {
            let bytes = self.offset_or_val.to_le_bytes();
            bytes[..count].to_vec()
        } else {
            self.data_slice(buf)?.to_vec()
        };
        // Trim trailing null characters and whitespace
        let s = String::from_utf8_lossy(&raw);
        let trimmed = s.trim_end_matches('\0').trim().to_string();
        if !trimmed.is_empty() {
            Some(trimmed)
        } else {
            None
        }
    }
}

/// A parsed IFD block containing indexed entries
#[derive(Clone, Debug, Default)]
pub struct ParsedIfd {
    pub offset: u32,
    pub entries: HashMap<u16, IfdEntry>,
    pub next_ifd_offset: u32,
}

impl ParsedIfd {
    pub fn get(&self, tag: u16) -> Option<&IfdEntry> {
        self.entries.get(&tag)
    }
}

/// A parsed TIFF container holding IFD chains and SubIFDs
#[derive(Clone, Debug)]
pub struct TiffContainer {
    pub is_little_endian: bool,
    pub root_ifds: Vec<ParsedIfd>,
    pub sub_ifds: Vec<ParsedIfd>,
    pub exif_ifds: Vec<ParsedIfd>,
}

impl TiffContainer {
    /// Attempts to parse a TIFF container from a raw byte buffer (header or full file).
    pub fn parse(buf: &[u8]) -> Option<Self> {
        if buf.len() < 8 {
            return None;
        }
        let is_le = match &buf[0..2] {
            b"II" => true,
            b"MM" => false,
            _ => return None,
        };

        let magic = read_u16(buf, 2, is_le)?;
        // Standard TIFF (42), Panasonic RW2 (0x55), Olympus ORF (0x4F52)
        if magic != 42 && magic != 0x55 && magic != 0x4F52 {
            return None;
        }

        let first_ifd_offset = read_u32(buf, 4, is_le)?;
        if first_ifd_offset == 0 || (first_ifd_offset as usize) >= buf.len() {
            return None;
        }

        let mut visited = HashSet::new();
        let mut root_ifds = Vec::new();
        let mut sub_ifds = Vec::new();
        let mut exif_ifds = Vec::new();

        // 1. Walk main IFD chain
        let mut curr_offset = first_ifd_offset;
        while curr_offset > 0 && (curr_offset as usize) + 2 <= buf.len() {
            if !visited.insert(curr_offset) {
                break; // Cycle detected
            }
            if let Some(ifd) = parse_ifd(buf, curr_offset, is_le) {
                curr_offset = ifd.next_ifd_offset;
                root_ifds.push(ifd);
            } else {
                break;
            }
        }

        // 2. Discover SubIFDs (Tag 330) and ExifIFDs (Tag 34665)
        for ifd in &root_ifds {
            // SubIFDs
            if let Some(sub_entry) = ifd.get(super::tags::TAG_SUB_IFDS) {
                if let Some(offsets) = sub_entry.as_u32_vec(buf, is_le) {
                    for off in offsets {
                        if off > 0 && (off as usize) + 2 <= buf.len() && visited.insert(off) {
                            if let Some(sub) = parse_ifd(buf, off, is_le) {
                                sub_ifds.push(sub);
                            }
                        }
                    }
                }
            }

            // ExifIFD
            if let Some(exif_entry) = ifd.get(super::tags::TAG_EXIF_IFD_POINTER) {
                if let Some(off) = exif_entry.as_u32(buf, is_le) {
                    if off > 0 && (off as usize) + 2 <= buf.len() && visited.insert(off) {
                        if let Some(exif) = parse_ifd(buf, off, is_le) {
                            exif_ifds.push(exif);
                        }
                    }
                }
            }
        }

        Some(Self {
            is_little_endian: is_le,
            root_ifds,
            sub_ifds,
            exif_ifds,
        })
    }

    /// Finds the first entry matching `tag` across all IFDs (IFD0, SubIFDs, ExifIFD).
    pub fn find_entry(&self, tag: u16) -> Option<&IfdEntry> {
        for ifd in &self.root_ifds {
            if let Some(e) = ifd.get(tag) {
                return Some(e);
            }
        }
        for ifd in &self.sub_ifds {
            if let Some(e) = ifd.get(tag) {
                return Some(e);
            }
        }
        for ifd in &self.exif_ifds {
            if let Some(e) = ifd.get(tag) {
                return Some(e);
            }
        }
        None
    }
}

/// Parses a single IFD at `offset`.
fn parse_ifd(buf: &[u8], offset: u32, is_le: bool) -> Option<ParsedIfd> {
    let start = offset as usize;
    if start + 2 > buf.len() {
        return None;
    }
    let num_entries = read_u16(buf, start, is_le)? as usize;
    let entries_size = num_entries.saturating_mul(12);
    if start + 2 + entries_size > buf.len() {
        return None;
    }

    let mut entries = HashMap::with_capacity(num_entries);
    let mut curr = start + 2;

    for _ in 0..num_entries {
        let tag = read_u16(buf, curr, is_le)?;
        let typ = read_u16(buf, curr + 2, is_le)?;
        let count = read_u32(buf, curr + 4, is_le)?;
        let offset_or_val = read_u32(buf, curr + 8, is_le)?;

        entries.insert(
            tag,
            IfdEntry {
                tag,
                typ,
                count,
                offset_or_val,
            },
        );
        curr += 12;
    }

    let next_ifd_offset = if curr + 4 <= buf.len() {
        read_u32(buf, curr, is_le).unwrap_or(0)
    } else {
        0
    };

    Some(ParsedIfd {
        offset,
        entries,
        next_ifd_offset,
    })
}

#[inline]
pub(crate) fn read_u16(buf: &[u8], off: usize, is_le: bool) -> Option<u16> {
    if off + 2 <= buf.len() {
        let b = [buf[off], buf[off + 1]];
        Some(if is_le { u16::from_le_bytes(b) } else { u16::from_be_bytes(b) })
    } else {
        None
    }
}

#[inline]
pub(crate) fn read_u32(buf: &[u8], off: usize, is_le: bool) -> Option<u32> {
    if off + 4 <= buf.len() {
        let b = [buf[off], buf[off + 1], buf[off + 2], buf[off + 3]];
        Some(if is_le { u32::from_le_bytes(b) } else { u32::from_be_bytes(b) })
    } else {
        None
    }
}

#[inline]
pub(crate) fn read_i32(buf: &[u8], off: usize, is_le: bool) -> Option<i32> {
    if off + 4 <= buf.len() {
        let b = [buf[off], buf[off + 1], buf[off + 2], buf[off + 3]];
        Some(if is_le { i32::from_le_bytes(b) } else { i32::from_be_bytes(b) })
    } else {
        None
    }
}
