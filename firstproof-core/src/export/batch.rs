use rayon::prelude::*;
use crate::{BatchExportItemRecord, BatchExportResultRecord};
use super::encoders::export_photo_file;

/// Dispatches multiple photo exports concurrently across all available CPU cores using Rayon.
pub fn export_batch_photos(
    items: Vec<BatchExportItemRecord>,
) -> Vec<BatchExportResultRecord> {
    items
        .into_par_iter()
        .map(|item| {
            let dist_tuple = item.lens_coeffs.as_ref().and_then(|c| {
                if c.has_distortion {
                    Some((c.distortion.as_slice(), c.distortion_model.as_str()))
                } else {
                    None
                }
            });

            match export_photo_file(
                &item.input_path,
                &item.output_path,
                &item.edits,
                dist_tuple,
                item.quality as u8,
                item.max_dimension,
                item.color_space,
                item.rendering_intent,
                item.ultra_hdr,
            ) {
                Ok(()) => BatchExportResultRecord {
                    input_path: item.input_path,
                    output_path: item.output_path,
                    success: true,
                    error_message: None,
                },
                Err(e) => BatchExportResultRecord {
                    input_path: item.input_path,
                    output_path: item.output_path,
                    success: false,
                    error_message: Some(e),
                },
            }
        })
        .collect()
}
