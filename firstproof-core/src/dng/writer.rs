// SPDX-License-Identifier: MIT
// Copyright 2026 First Proof Developers

//! Archival DNG authoring engine.
//!
//! Writes Adobe DNG 1.4+ containers with lossless Bayer mosaic compression,
//! embedded First Proof authentic AgX filmic previews (zero camera-JPEG plastic),
//! and full camera calibration/EXIF preservation.

use std::io::{Seek, Write};
use std::path::Path;
use std::sync::Arc;

use image::{DynamicImage, RgbImage};
use rawler::decoders::{RawDecodeParams, WellKnownIFD};
use rawler::dng::writer::DngWriter;
use rawler::dng::{CropMode, DngCompression, DngPhotometricConversion, DNG_VERSION_V1_4};
use rawler::formats::tiff::Entry;
use rawler::rawsource::RawSource;
use rawler::tags::{DngTag, ExifTag, TiffCommonTag};
use rawler::RawImageData;
use sha2::{Digest, Sha256};

use crate::color::Color3;
use crate::export::decode::f16_to_f32;
use crate::export::decode_raw_preview_binned_buffer;

/// Computes the cryptographic SHA-256 digest of a raw CFA sensor photosite array.
pub fn hash_cfa_data(data: &RawImageData) -> String {
    let mut hasher = Sha256::new();
    match data {
        RawImageData::Integer(v) => {
            for &val in v {
                hasher.update(val.to_le_bytes());
            }
        }
        RawImageData::Float(v) => {
            for &val in v {
                hasher.update(val.to_le_bytes());
            }
        }
    }
    format!("{:x}", hasher.finalize())
}

/// Generates an authentic First Proof AgX filmic preview image.
///
/// Uses sub-15ms 2×2 Bayer box-binning into Linear Rec.2020 space, followed by
/// AgX filmic tone mapping to sRGB. Completely bypasses proprietary camera-JPEG rendering.
pub fn generate_agx_preview<P: AsRef<Path>>(input_path: P) -> Option<DynamicImage> {
    let (bin_w, bin_h, rgba_f16) = decode_raw_preview_binned_buffer(input_path).ok()?;
    if bin_w == 0 || bin_h == 0 || rgba_f16.is_empty() {
        return None;
    }

    let mut rgb8 = Vec::with_capacity((bin_w * bin_h * 3) as usize);
    let pixel_count = (bin_w * bin_h) as usize;

    for i in 0..pixel_count {
        let offset = i * 8;
        if offset + 5 >= rgba_f16.len() {
            break;
        }
        let r_f16 = u16::from_le_bytes([rgba_f16[offset], rgba_f16[offset + 1]]);
        let g_f16 = u16::from_le_bytes([rgba_f16[offset + 2], rgba_f16[offset + 3]]);
        let b_f16 = u16::from_le_bytes([rgba_f16[offset + 4], rgba_f16[offset + 5]]);

        let r_rec = f16_to_f32(r_f16);
        let g_rec = f16_to_f32(g_f16);
        let b_rec = f16_to_f32(b_f16);

        let mapped = crate::color::agx_tone_map_vivid(Color3::new(r_rec, g_rec, b_rec), 1.0);
        rgb8.push((mapped.r().clamp(0.0, 1.0) * 255.0).round() as u8);
        rgb8.push((mapped.g().clamp(0.0, 1.0) * 255.0).round() as u8);
        rgb8.push((mapped.b().clamp(0.0, 1.0) * 255.0).round() as u8);
    }

    RgbImage::from_raw(bin_w, bin_h, rgb8).map(DynamicImage::ImageRgb8)
}

/// Parameters for writing an archival DNG file.
pub struct WriteDngParams {
    pub software_name: String,
    pub xmp_packet: Option<Vec<u8>>,
}

impl Default for WriteDngParams {
    fn default() -> Self {
        Self {
            software_name: "First Proof Darkroom".to_string(),
            xmp_packet: None,
        }
    }
}

/// Writes an archival DNG file from a source RAW file.
///
/// Returns the SHA-256 checksum of the source CFA mosaic written into the container.
pub fn write_archival_dng<P: AsRef<Path>, W: Write + Seek>(
    source_raw_path: P,
    output_writer: &mut W,
    params: &WriteDngParams,
) -> Result<String, String> {
    let source_raw_path = source_raw_path.as_ref();
    let rawsource = Arc::new(RawSource::new(source_raw_path).map_err(|e| e.to_string())?);
    let decoder = rawler::get_decoder(&rawsource).map_err(|e| e.to_string())?;
    let raw_params = RawDecodeParams { image_index: 0 };

    let rawimage = decoder
        .raw_image(&rawsource, &raw_params, false)
        .map_err(|e| format!("Failed to decode RAW image: {}", e))?;
    let metadata = decoder
        .raw_metadata(&rawsource, &raw_params)
        .map_err(|e| format!("Failed to decode RAW metadata: {}", e))?;

    // Compute cryptographic SHA-256 of the uncompressed, unscaled source CFA mosaic
    let source_cfa_hash = hash_cfa_data(&rawimage.data);

    let mut dng = DngWriter::new(output_writer, DNG_VERSION_V1_4)
        .map_err(|e| format!("Failed to initialize DNG writer: {:?}", e))?;

    // 1. SubIFD 0: Raw Bayer CFA Mosaic (Lossless JPEG-92 compressed, square layout)
    let mut raw_sub = dng.subframe(0);
    raw_sub
        .raw_image(
            &rawimage,
            CropMode::Best,
            DngCompression::Lossless,
            DngPhotometricConversion::Original,
            1, // standard JPEG predictor
        )
        .map_err(|e| format!("Failed to write RAW CFA subframe: {:?}", e))?;

    if let Ok(Some(dng_raw_ifd)) = decoder.ifd(WellKnownIFD::VirtualDngRawTags) {
        raw_sub.ifd_mut().copy(dng_raw_ifd.value_iter());
    }
    raw_sub
        .finalize()
        .map_err(|e| format!("Failed to finalize RAW CFA subframe: {:?}", e))?;

    // 2. Previews: IFD 0 Thumbnail & SubIFD 1 High-Res AgX Retina Preview
    if let Some(preview_img) = generate_agx_preview(source_raw_path) {
        // SubIFD 1: High-Res Retina Preview (JPEG Q=85, up to 2560px long edge)
        let mut preview_sub = dng.subframe(1);
        if let Err(err) = preview_sub.preview(&preview_img, 0.85) {
            log::warn!("Failed to write SubIFD preview: {:?}", err);
        } else {
            let _ = preview_sub.finalize();
        }

        // IFD 0: Root Thumbnail (uncompressed RGB, scaled to ~240px)
        if let Err(err) = dng.thumbnail(&preview_img) {
            log::warn!("Failed to write IFD0 thumbnail: {:?}", err);
        }
    }

    // 3. Metadata & Camera Calibration
    dng.load_base_tags(&rawimage)
        .map_err(|e| format!("Failed to load base tags: {:?}", e))?;
    dng.load_metadata(&metadata)
        .map_err(|e| format!("Failed to load metadata: {:?}", e))?;

    if !dng.root_ifd().contains(ExifTag::Orientation) {
        dng.root_ifd_mut()
            .add_tag(ExifTag::Orientation, rawimage.orientation.to_u16());
    }

    // Copy Virtual DNG root tags (matrices, baseline exposure)
    if let Ok(Some(dng_root_ifd)) = decoder.ifd(WellKnownIFD::VirtualDngRootTags) {
        dng.root_ifd_mut().copy(dng_root_ifd.value_iter());
    }

    // Remove unsafe makernotes
    if let Ok(Some(Entry {
        value: rawler::formats::tiff::Value::Short(v),
        ..
    })) = decoder
        .ifd(WellKnownIFD::VirtualDngRootTags)
        .map(|opt| opt.and_then(|ifd| ifd.get_entry(DngTag::MakerNoteSafety).cloned()))
    {
        if v.first().copied().unwrap_or(0) == 0 {
            dng.exif_ifd_mut().remove_tag(ExifTag::MakerNotes);
        }
    }

    // 4. XMP Packet Embedding
    if let Some(ref packet) = params.xmp_packet {
        dng.xpacket(packet)
            .map_err(|e| format!("Failed to write XMP packet: {:?}", e))?;
    } else if let Ok(Some(xpacket)) = decoder.xpacket(&rawsource, &raw_params) {
        let _ = dng.xpacket(&xpacket);
    }

    // Software attribution
    dng.root_ifd_mut()
        .add_tag(TiffCommonTag::Software, &params.software_name);
    dng.root_ifd_mut().add_tag(
        ExifTag::ModifyDate,
        chrono::Local::now().format("%Y:%m:%d %H:%M:%S").to_string(),
    );

    dng.close()
        .map_err(|e| format!("Failed to close DNG file: {:?}", e))?;

    Ok(source_cfa_hash)
}
