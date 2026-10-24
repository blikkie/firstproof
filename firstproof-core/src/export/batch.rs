use rayon::prelude::*;
use crate::{BatchExportItemRecord, BatchExportResultRecord};
use super::encoders::export_photo_file_advanced;

/// Dispatches multiple photo exports concurrently across all available CPU cores using Rayon.
pub fn export_batch_photos(
    items: Vec<BatchExportItemRecord>,
) -> Vec<BatchExportResultRecord> {
    items
        .into_par_iter()
        .map(|item| {
            let demosaic_quality = item.demosaic_quality.unwrap_or_default();
            let render_tier = item.render_tier.unwrap_or(crate::RenderTier::ArchivalExport);

            match export_photo_file_advanced(
                &item.input_path,
                &item.output_path,
                &item.edits,
                item.lens_coeffs.as_ref(),
                item.quality as u8,
                item.max_dimension,
                item.color_space,
                item.rendering_intent,
                item.ultra_hdr,
                demosaic_quality,
                render_tier,
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
