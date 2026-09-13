use img_parts::jpeg::Jpeg;
use img_parts::{Bytes, ImageICC};

/// Creates an ICC.1 compliant Display P3 color profile with D65 white point,
/// DCI-P3 color primaries, and standard sRGB parametric transfer curve.
pub fn create_display_p3_profile() -> Result<lcms2::Profile, String> {
    let white_point = lcms2::CIExyY {
        x: 0.3127,
        y: 0.3290,
        Y: 1.0,
    };
    let primaries = lcms2::CIExyYTRIPLE {
        Red: lcms2::CIExyY { x: 0.680, y: 0.320, Y: 1.0 },
        Green: lcms2::CIExyY { x: 0.265, y: 0.690, Y: 1.0 },
        Blue: lcms2::CIExyY { x: 0.150, y: 0.060, Y: 1.0 },
    };
    let curve = lcms2::ToneCurve::new_parametric(
        4,
        &[2.4, 1.0 / 1.055, 0.055 / 1.055, 1.0 / 12.92, 0.04045],
    ).map_err(|e| format!("Failed to create parametric curve: {:?}", e))?;
    let curves = [&curve, &curve, &curve];
    lcms2::Profile::new_rgb(&white_point, &primaries, &curves)
        .map_err(|e| format!("Failed to create Display P3 profile: {:?}", e))
}

/// Injects an ICC profile into a JPEG byte stream as an APP2 segment (ICC.1:2010 specification).
/// Uses zero-copy `img-parts` segment parsing and chunking.
pub fn inject_icc_profile_into_jpeg(jpeg_data: &[u8], icc_profile: &[u8]) -> Vec<u8> {
    match Jpeg::from_bytes(Bytes::copy_from_slice(jpeg_data)) {
        Ok(mut jpeg) => {
            jpeg.set_icc_profile(Some(Bytes::copy_from_slice(icc_profile)));
            let mut out = Vec::with_capacity(jpeg_data.len() + icc_profile.len() + 64);
            if jpeg.encoder().write_to(&mut out).is_ok() {
                out
            } else {
                jpeg_data.to_vec()
            }
        }
        Err(_) => jpeg_data.to_vec(),
    }
}
