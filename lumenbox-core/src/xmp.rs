use roxmltree::{Document, Node};
use std::path::{Path, PathBuf};
use crate::EditSettings;

#[derive(Clone, Debug)]
pub struct XmpData {
    pub rating: Option<i32>,
    pub status: String, // "Pick", "Reject", "None"
    pub title: Option<String>,
    pub description: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub edits: EditSettings,
}

impl Default for XmpData {
    fn default() -> Self {
        Self {
            rating: None,
            status: "None".to_string(),
            title: None,
            description: None,
            latitude: None,
            longitude: None,
            edits: EditSettings::default(),
        }
    }
}

/// Formats a decimal GPS coordinate into TIFF-style EXIF representation inside XMP.
/// e.g. 34.0522 -> "34,3.1320N"
fn format_gps_xmp(val: f64, is_lat: bool) -> String {
    let abs_val = val.abs();
    let deg = abs_val.floor() as u32;
    let min = (abs_val - deg as f64) * 60.0;
    
    let dir = if is_lat {
        if val >= 0.0 { "N" } else { "S" }
    } else {
        if val >= 0.0 { "E" } else { "W" }
    };
    
    format!("{},{:.4}{}", deg, min, dir)
}

/// Parses TIFF-style EXIF coordinate from XMP back to decimal degree float.
/// e.g. "34,3.1320N" -> 34.0522
fn parse_gps_xmp(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    
    let last_char = s.chars().last()?;
    let has_direction = last_char.is_alphabetic();
    
    let clean_s = if has_direction {
        &s[0..s.len() - 1]
    } else {
        s
    };
    
    let parts: Vec<&str> = clean_s.split(',').collect();
    if parts.is_empty() {
        return None;
    }
    
    let deg = parts[0].parse::<f64>().ok()?;
    let min = if parts.len() > 1 {
        parts[1].parse::<f64>().ok().unwrap_or(0.0)
    } else {
        0.0
    };
    
    let mut val = deg + (min / 60.0);
    
    if has_direction {
        let dir = last_char.to_ascii_uppercase();
        if dir == 'S' || dir == 'W' {
            val = -val;
        }
    }
    
    Some(val)
}

use crate::curve::CurvePointRecord;

fn serialize_curve_points(pts: &[CurvePointRecord]) -> String {
    pts.iter().map(|p| format!("{:.4},{:.4}", p.x, p.y)).collect::<Vec<_>>().join(";")
}

fn parse_curve_points(s: &str) -> Vec<CurvePointRecord> {
    s.split(';')
        .filter_map(|pair| {
            let mut parts = pair.split(',');
            let x = parts.next()?.trim().parse::<f32>().ok()?;
            let y = parts.next()?.trim().parse::<f32>().ok()?;
            Some(CurvePointRecord { x, y })
        })
        .collect()
}

use xmp_writer::{CustomNamespace, Namespace, XmpWriter};

/// Generates XMP XML sidecar string using type-safe XmpWriter.
pub fn generate_xmp(data: &XmpData) -> String {
    let mut writer = XmpWriter::new();

    // Standard Dublin Core schemas
    if let Some(t) = &data.title {
        writer.title([(None, t.as_str())]);
    }
    if let Some(d) = &data.description {
        writer.description([(None, d.as_str())]);
    }

    // Standard XMP Rating
    if let Some(r) = data.rating {
        writer.element("Rating", Namespace::Xmp).value(r);
    }

    // EXIF namespace
    let exif_ns = Namespace::Custom(Box::new(CustomNamespace::new(
        "Exif",
        "exif",
        "http://ns.adobe.com/exif/1.0/",
    )));
    if let (Some(lat), Some(lon)) = (data.latitude, data.longitude) {
        writer.element("GPSLatitude", exif_ns.clone()).value(format_gps_xmp(lat, true).as_str());
        writer.element("GPSLongitude", exif_ns.clone()).value(format_gps_xmp(lon, false).as_str());
        writer.element("GPSVersionID", exif_ns).value("2.2.0.0");
    }

    // Custom LumenBox PM namespace
    let pm_ns = Namespace::Custom(Box::new(CustomNamespace::new(
        "LumenBox Photo Metadata",
        "pm",
        "http://ns.lumenbox.app/pm/1.0/",
    )));

    writer.element("status", pm_ns.clone()).value(data.status.as_str());
    writer.element("exposure", pm_ns.clone()).value(format!("{:.4}", data.edits.exposure).as_str());
    writer.element("contrast", pm_ns.clone()).value(format!("{:.4}", data.edits.contrast).as_str());
    writer.element("highlights", pm_ns.clone()).value(format!("{:.4}", data.edits.highlights).as_str());
    writer.element("shadows", pm_ns.clone()).value(format!("{:.4}", data.edits.shadows).as_str());
    writer.element("whites", pm_ns.clone()).value(format!("{:.4}", data.edits.whites).as_str());
    writer.element("blacks", pm_ns.clone()).value(format!("{:.4}", data.edits.blacks).as_str());
    writer.element("vibrance", pm_ns.clone()).value(format!("{:.4}", data.edits.vibrance).as_str());
    writer.element("saturation", pm_ns.clone()).value(format!("{:.4}", data.edits.saturation).as_str());
    writer.element("deepBlue", pm_ns.clone()).value(format!("{:.4}", data.edits.deep_blue).as_str());
    writer.element("foliage", pm_ns.clone()).value(format!("{:.4}", data.edits.foliage).as_str());
    writer.element("sharpeningAmount", pm_ns.clone()).value(format!("{:.4}", data.edits.sharpening_amount).as_str());
    writer.element("sharpeningRadius", pm_ns.clone()).value(format!("{:.4}", data.edits.sharpening_radius).as_str());
    writer.element("sharpeningMasking", pm_ns.clone()).value(format!("{:.4}", data.edits.sharpening_masking).as_str());
    writer.element("noiseReduction", pm_ns.clone()).value(format!("{:.4}", data.edits.noise_reduction).as_str());
    writer.element("grainAmount", pm_ns.clone()).value(format!("{:.4}", data.edits.grain_amount).as_str());
    writer.element("grainSize", pm_ns.clone()).value(format!("{:.4}", data.edits.grain_size).as_str());
    writer.element("grainRoughness", pm_ns.clone()).value(format!("{:.4}", data.edits.grain_roughness).as_str());
    writer.element("grainChroma", pm_ns.clone()).value(format!("{:.4}", data.edits.grain_chroma).as_str());
    writer.element("temperature", pm_ns.clone()).value(format!("{:.4}", data.edits.temperature).as_str());
    writer.element("tint", pm_ns.clone()).value(format!("{:.4}", data.edits.tint).as_str());
    writer.element("cropLeft", pm_ns.clone()).value(format!("{:.4}", data.edits.crop_left).as_str());
    writer.element("cropTop", pm_ns.clone()).value(format!("{:.4}", data.edits.crop_top).as_str());
    writer.element("cropRight", pm_ns.clone()).value(format!("{:.4}", data.edits.crop_right).as_str());
    writer.element("cropBottom", pm_ns.clone()).value(format!("{:.4}", data.edits.crop_bottom).as_str());
    writer.element("cropAngle", pm_ns.clone()).value(format!("{:.4}", data.edits.crop_angle).as_str());
    writer.element("perspectiveVertical", pm_ns.clone()).value(format!("{:.4}", data.edits.perspective_vertical).as_str());
    writer.element("perspectiveHorizontal", pm_ns.clone()).value(format!("{:.4}", data.edits.perspective_horizontal).as_str());
    writer.element("cropConstrain", pm_ns.clone()).value(if data.edits.crop_constrain { "true" } else { "false" });
    writer.element("rotation", pm_ns.clone()).value(format!("{}", data.edits.rotation).as_str());
    writer.element("flipHorizontal", pm_ns.clone()).value(if data.edits.flip_horizontal { "true" } else { "false" });
    writer.element("flipVertical", pm_ns.clone()).value(if data.edits.flip_vertical { "true" } else { "false" });

    let tone_profile_str = match data.edits.tone_profile {
        crate::ColorProfileMode::AgxVivid => "AgxVivid",
        crate::ColorProfileMode::AgxFilmic => "AgxFilmic",
        crate::ColorProfileMode::CameraStandard => "CameraStandard",
        crate::ColorProfileMode::CameraPortrait => "CameraPortrait",
        crate::ColorProfileMode::CameraLandscape => "CameraLandscape",
        crate::ColorProfileMode::CameraVivid => "CameraVivid",
    };
    writer.element("toneProfile", pm_ns.clone()).value(tone_profile_str);

    if let Some(name) = &data.edits.lut_name {
        writer.element("lutName", pm_ns.clone()).value(name.as_str());
        writer.element("lutIntensity", pm_ns.clone()).value(format!("{:.4}", data.edits.lut_intensity).as_str());
    }

    if !data.edits.curves.master.is_empty() {
        writer.element("curveMaster", pm_ns.clone()).value(serialize_curve_points(&data.edits.curves.master).as_str());
    }
    if !data.edits.curves.red.is_empty() {
        writer.element("curveRed", pm_ns.clone()).value(serialize_curve_points(&data.edits.curves.red).as_str());
    }
    if !data.edits.curves.green.is_empty() {
        writer.element("curveGreen", pm_ns.clone()).value(serialize_curve_points(&data.edits.curves.green).as_str());
    }
    if !data.edits.curves.blue.is_empty() {
        writer.element("curveBlue", pm_ns.clone()).value(serialize_curve_points(&data.edits.curves.blue).as_str());
    }

    // Adobe Camera Raw Settings (CRS) namespace for compatibility with Lightroom / Camera Raw
    let crs_ns = Namespace::Custom(Box::new(CustomNamespace::new(
        "Camera Raw Settings",
        "crs",
        "http://ns.adobe.com/camera-raw-settings/1.0/",
    )));
    writer.element("PerspectiveVertical", crs_ns.clone()).value(format!("{:.4}", data.edits.perspective_vertical).as_str());
    writer.element("PerspectiveHorizontal", crs_ns.clone()).value(format!("{:.4}", data.edits.perspective_horizontal).as_str());
    writer.element("CropConstrainToWarp", crs_ns.clone()).value(if data.edits.crop_constrain { "1" } else { "0" });
    writer.element("GrainAmount", crs_ns.clone()).value(format!("{:.4}", data.edits.grain_amount).as_str());
    writer.element("GrainSize", crs_ns.clone()).value(format!("{:.4}", data.edits.grain_size).as_str());
    writer.element("GrainFrequency", crs_ns).value(format!("{:.4}", data.edits.grain_roughness).as_str());

    writer.finish(None)
}

/// Reads title or description child node out of the dc:title/dc:description blocks.
fn find_rdf_li_val(node: Node) -> Option<String> {
    for child in node.descendants() {
        if child.has_tag_name(( "http://www.w3.org/1999/02/22-rdf-syntax-ns#", "li" )) || child.tag_name().name() == "li" {
            if let Some(text) = child.text() {
                return Some(text.trim().to_string());
            }
        }
    }
    None
}

fn find_desc_node<'a, 'd>(doc: &'a Document<'d>) -> Option<Node<'a, 'd>> {
    for node in doc.descendants() {
        if node.is_element()
            && (node.has_tag_name(( "http://www.w3.org/1999/02/22-rdf-syntax-ns#", "Description" ))
                || node.tag_name().name() == "Description")
        {
            return Some(node);
        }
    }
    None
}

fn get_prop<'a, 'd>(desc: &'a Node<'a, 'd>, ns: &str, name: &str) -> Option<&'a str> {
    if let Some(attr) = desc.attribute((ns, name)).or_else(|| desc.attribute(name)) {
        return Some(attr);
    }
    for child in desc.children() {
        if (child.has_tag_name((ns, name)) || child.tag_name().name() == name) && child.is_element() {
            if let Some(text) = child.text() {
                return Some(text.trim());
            }
        }
    }
    None
}

/// Parses XMP XML sidecar string.
pub fn parse_xmp(xmp_str: &str) -> std::io::Result<XmpData> {
    let doc = Document::parse(xmp_str)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        
    let desc = find_desc_node(&doc)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Missing rdf:Description node"))?;
        
    let mut data = XmpData::default();
    let get = |ns: &str, name: &str| -> Option<&str> {
        get_prop(&desc, ns, name)
    };
    
    // 1. Parse standard attributes
    let raw_rating = get("http://ns.adobe.com/xap/1.0/", "Rating")
        .and_then(|r| r.parse::<i32>().ok());

    if let Some(r) = raw_rating {
        if r == -1 {
            data.status = "reject".to_string();
            data.rating = None;
        } else if r == 0 {
            data.rating = None;
        } else {
            data.rating = Some(r);
        }
    }
    
    if let Some(lat_str) = get("http://ns.adobe.com/exif/1.0/", "GPSLatitude") {
        data.latitude = parse_gps_xmp(lat_str);
    }
    if let Some(lon_str) = get("http://ns.adobe.com/exif/1.0/", "GPSLongitude") {
        data.longitude = parse_gps_xmp(lon_str);
    }
    
    // 2. Parse Custom Adjustments (pm:)
    let pm = "http://ns.lumenbox.app/pm/1.0/";
    if let Some(status_str) = get(pm, "status") {
        data.status = status_str.to_string();
    }
    if let Some(exp_str) = get(pm, "exposure") {
        if let Ok(val) = exp_str.parse::<f32>() {
            data.edits.exposure = val;
        }
    }
    if let Some(con_str) = get(pm, "contrast") {
        if let Ok(val) = con_str.parse::<f32>() {
            data.edits.contrast = val;
        }
    }
    if let Some(hl_str) = get(pm, "highlights") {
        if let Ok(val) = hl_str.parse::<f32>() {
            data.edits.highlights = val;
        }
    }
    if let Some(sh_str) = get(pm, "shadows") {
        if let Ok(val) = sh_str.parse::<f32>() {
            data.edits.shadows = val;
        }
    }
    if let Some(wh_str) = get(pm, "whites") {
        if let Ok(val) = wh_str.parse::<f32>() {
            data.edits.whites = val;
        }
    }
    if let Some(bl_str) = get(pm, "blacks") {
        if let Ok(val) = bl_str.parse::<f32>() {
            data.edits.blacks = val;
        }
    }
    if let Some(vib_str) = get(pm, "vibrance") {
        if let Ok(val) = vib_str.parse::<f32>() {
            data.edits.vibrance = val;
        }
    }
    if let Some(sat_str) = get(pm, "saturation") {
        if let Ok(val) = sat_str.parse::<f32>() {
            data.edits.saturation = val;
        }
    }
    if let Some(db_str) = get(pm, "deepBlue") {
        if let Ok(val) = db_str.parse::<f32>() {
            data.edits.deep_blue = val;
        }
    }
    if let Some(fol_str) = get(pm, "foliage") {
        if let Ok(val) = fol_str.parse::<f32>() {
            data.edits.foliage = val;
        }
    }
    if let Some(s_str) = get(pm, "sharpeningAmount") {
        if let Ok(val) = s_str.parse::<f32>() {
            data.edits.sharpening_amount = val;
        }
    }
    if let Some(r_str) = get(pm, "sharpeningRadius") {
        if let Ok(val) = r_str.parse::<f32>() {
            data.edits.sharpening_radius = val;
        }
    }
    if let Some(m_str) = get(pm, "sharpeningMasking") {
        if let Ok(val) = m_str.parse::<f32>() {
            data.edits.sharpening_masking = val;
        }
    }
    if let Some(nr_str) = get(pm, "noiseReduction") {
        if let Ok(val) = nr_str.parse::<f32>() {
            data.edits.noise_reduction = val;
        }
    }
    if let Some(ga_str) = get(pm, "grainAmount") {
        if let Ok(val) = ga_str.parse::<f32>() {
            data.edits.grain_amount = val;
        }
    }
    if let Some(gs_str) = get(pm, "grainSize") {
        if let Ok(val) = gs_str.parse::<f32>() {
            data.edits.grain_size = val;
        }
    }
    if let Some(gr_str) = get(pm, "grainRoughness") {
        if let Ok(val) = gr_str.parse::<f32>() {
            data.edits.grain_roughness = val;
        }
    }
    if let Some(gc_str) = get(pm, "grainChroma") {
        if let Ok(val) = gc_str.parse::<f32>() {
            data.edits.grain_chroma = val;
        }
    }
    if let Some(temp_str) = get(pm, "temperature") {
        if let Ok(val) = temp_str.parse::<f32>() {
            data.edits.temperature = val;
        }
    }
    if let Some(tint_str) = get(pm, "tint") {
        if let Ok(val) = tint_str.parse::<f32>() {
            data.edits.tint = val;
        }
    }
    if let Some(cl_str) = get(pm, "cropLeft") {
        if let Ok(val) = cl_str.parse::<f32>() {
            data.edits.crop_left = val;
        }
    }
    if let Some(ct_str) = get(pm, "cropTop") {
        if let Ok(val) = ct_str.parse::<f32>() {
            data.edits.crop_top = val;
        }
    }
    if let Some(cr_str) = get(pm, "cropRight") {
        if let Ok(val) = cr_str.parse::<f32>() {
            data.edits.crop_right = val;
        }
    }
    if let Some(cb_str) = get(pm, "cropBottom") {
        if let Ok(val) = cb_str.parse::<f32>() {
            data.edits.crop_bottom = val;
        }
    }
    if let Some(ca_str) = get(pm, "cropAngle") {
        if let Ok(val) = ca_str.parse::<f32>() {
            data.edits.crop_angle = val;
        }
    }
    if let Some(pv_str) = get(pm, "perspectiveVertical") {
        if let Ok(val) = pv_str.parse::<f32>() {
            data.edits.perspective_vertical = val;
        }
    }
    if let Some(ph_str) = get(pm, "perspectiveHorizontal") {
        if let Ok(val) = ph_str.parse::<f32>() {
            data.edits.perspective_horizontal = val;
        }
    }
    if let Some(cc_str) = get(pm, "cropConstrain") {
        data.edits.crop_constrain = cc_str == "true" || cc_str == "1";
    }
    if let Some(rot_str) = get(pm, "rotation") {
        if let Ok(val) = rot_str.parse::<i32>() {
            data.edits.rotation = val;
        }
    }
    if let Some(fh_str) = get(pm, "flipHorizontal") {
        data.edits.flip_horizontal = fh_str == "true" || fh_str == "1";
    }
    if let Some(fv_str) = get(pm, "flipVertical") {
        data.edits.flip_vertical = fv_str == "true" || fv_str == "1";
    }
    if let Some(tp_str) = get(pm, "toneProfile") {
        data.edits.tone_profile = match tp_str {
            "AgxVivid" => crate::ColorProfileMode::AgxVivid,
            "AgxFilmic" => crate::ColorProfileMode::AgxFilmic,
            "CameraStandard" => crate::ColorProfileMode::CameraStandard,
            "CameraPortrait" => crate::ColorProfileMode::CameraPortrait,
            "CameraLandscape" => crate::ColorProfileMode::CameraLandscape,
            "CameraVivid" => crate::ColorProfileMode::CameraVivid,
            _ => crate::ColorProfileMode::AgxVivid,
        };
    }
    if let Some(lut_name_str) = get(pm, "lutName") {
        if !lut_name_str.trim().is_empty() {
            data.edits.lut_name = Some(lut_name_str.to_string());
        }
    }
    if let Some(lut_intensity_str) = get(pm, "lutIntensity") {
        if let Ok(val) = lut_intensity_str.parse::<f32>() {
            data.edits.lut_intensity = val;
        }
    }

    if let Some(pts_str) = get(pm, "curveMaster") {
        let pts = parse_curve_points(pts_str);
        if !pts.is_empty() {
            data.edits.curves.master = pts;
        }
    }
    if let Some(pts_str) = get(pm, "curveRed") {
        let pts = parse_curve_points(pts_str);
        if !pts.is_empty() {
            data.edits.curves.red = pts;
        }
    }
    if let Some(pts_str) = get(pm, "curveGreen") {
        let pts = parse_curve_points(pts_str);
        if !pts.is_empty() {
            data.edits.curves.green = pts;
        }
    }
    if let Some(pts_str) = get(pm, "curveBlue") {
        let pts = parse_curve_points(pts_str);
        if !pts.is_empty() {
            data.edits.curves.blue = pts;
        }
    }
    
    // 3. Fallback: Parse Adobe Camera Raw settings (crs:) if native ones are missing
    let crs_ns = "http://ns.adobe.com/camera-raw-settings/1.0/";
    
    // Check if edits have defaults, if so try reading Adobe settings
    if data.edits.exposure == 0.0 {
        if let Some(exp_str) = desc.attribute(( crs_ns, "Exposure2012" )) {
            if let Ok(val) = exp_str.parse::<f32>() {
                data.edits.exposure = val;
            }
        }
    }
    if data.edits.contrast == 0.0 {
        if let Some(con_str) = desc.attribute(( crs_ns, "Contrast2012" )) {
            if let Ok(val) = con_str.parse::<f32>() {
                data.edits.contrast = val;
            }
        }
    }
    if data.edits.highlights == 0.0 {
        if let Some(hl_str) = desc.attribute(( crs_ns, "Highlights2012" )).or_else(|| desc.attribute(( crs_ns, "Highlights" ))) {
            if let Ok(val) = hl_str.parse::<f32>() {
                data.edits.highlights = val;
            }
        }
    }
    if data.edits.shadows == 0.0 {
        if let Some(sh_str) = desc.attribute(( crs_ns, "Shadows2012" )).or_else(|| desc.attribute(( crs_ns, "Shadows" ))) {
            if let Ok(val) = sh_str.parse::<f32>() {
                data.edits.shadows = val;
            }
        }
    }
    if data.edits.whites == 0.0 {
        if let Some(wh_str) = desc.attribute(( crs_ns, "Whites2012" )).or_else(|| desc.attribute(( crs_ns, "Whites" ))) {
            if let Ok(val) = wh_str.parse::<f32>() {
                data.edits.whites = val;
            }
        }
    }
    if data.edits.blacks == 0.0 {
        if let Some(bl_str) = desc.attribute(( crs_ns, "Blacks2012" )).or_else(|| desc.attribute(( crs_ns, "Blacks" ))) {
            if let Ok(val) = bl_str.parse::<f32>() {
                data.edits.blacks = val;
            }
        }
    }
    if data.edits.vibrance == 0.0 {
        if let Some(vib_str) = desc.attribute(( crs_ns, "Vibrance" )) {
            if let Ok(val) = vib_str.parse::<f32>() {
                data.edits.vibrance = val;
            }
        }
    }
    if data.edits.saturation == 0.0 {
        if let Some(sat_str) = desc.attribute(( crs_ns, "Saturation" )) {
            if let Ok(val) = sat_str.parse::<f32>() {
                data.edits.saturation = val;
            }
        }
    }
    if data.edits.temperature == 5000.0 {
        if let Some(temp_str) = desc.attribute(( crs_ns, "Temperature" )) {
            if let Ok(val) = temp_str.parse::<f32>() {
                data.edits.temperature = val;
            }
        }
    }
    if data.edits.tint == 0.0 {
        if let Some(tint_str) = desc.attribute(( crs_ns, "Tint" )) {
            if let Ok(val) = tint_str.parse::<f32>() {
                data.edits.tint = val;
            }
        }
    }
    if data.edits.sharpening_amount == 0.0 {
        if let Some(s_str) = desc.attribute(( crs_ns, "SharpenAmount" )).or_else(|| desc.attribute(( crs_ns, "Sharpness" ))) {
            if let Ok(val) = s_str.parse::<f32>() {
                data.edits.sharpening_amount = val;
            }
        }
    }
    if data.edits.sharpening_radius == 1.0 {
        if let Some(r_str) = desc.attribute(( crs_ns, "SharpenRadius" )) {
            if let Ok(val) = r_str.parse::<f32>() {
                data.edits.sharpening_radius = val;
            }
        }
    }
    if data.edits.sharpening_masking == 0.0 {
        if let Some(m_str) = desc.attribute(( crs_ns, "SharpenEdgeMasking" )).or_else(|| desc.attribute(( crs_ns, "Masking" ))) {
            if let Ok(val) = m_str.parse::<f32>() {
                data.edits.sharpening_masking = val;
            }
        }
    }
    if data.edits.noise_reduction == 0.0 {
        if let Some(nr_str) = desc.attribute(( crs_ns, "LuminanceSmoothing" )) {
            if let Ok(val) = nr_str.parse::<f32>() {
                data.edits.noise_reduction = val;
            }
        }
    }
    if data.edits.grain_amount == 0.0 {
        if let Some(ga_str) = desc.attribute(( crs_ns, "GrainAmount" )) {
            if let Ok(val) = ga_str.parse::<f32>() {
                data.edits.grain_amount = val;
            }
        }
    }
    if data.edits.grain_size == 1.0 {
        if let Some(gs_str) = desc.attribute(( crs_ns, "GrainSize" )) {
            if let Ok(val) = gs_str.parse::<f32>() {
                data.edits.grain_size = val;
            }
        }
    }
    if data.edits.grain_roughness == 50.0 {
        if let Some(gr_str) = desc.attribute(( crs_ns, "GrainFrequency" )) {
            if let Ok(val) = gr_str.parse::<f32>() {
                data.edits.grain_roughness = val;
            }
        }
    }
    
    // Check crop details in Adobe
    if data.edits.crop_left == 0.0 && data.edits.crop_right == 1.0 {
        if let (Some(l), Some(t), Some(w), Some(h)) = (
            desc.attribute(( crs_ns, "CropLeft" )),
            desc.attribute(( crs_ns, "CropTop" )),
            desc.attribute(( crs_ns, "CropWidth" )),
            desc.attribute(( crs_ns, "CropHeight" ))
        ) {
            let cl = l.parse::<f32>().unwrap_or(0.0);
            let ct = t.parse::<f32>().unwrap_or(0.0);
            let cw = w.parse::<f32>().unwrap_or(1.0);
            let ch = h.parse::<f32>().unwrap_or(1.0);
            data.edits.crop_left = cl;
            data.edits.crop_top = ct;
            data.edits.crop_right = cl + cw;
            data.edits.crop_bottom = ct + ch;
        }
    }
    
    if data.edits.crop_angle == 0.0 {
        if let Some(ca_str) = desc.attribute(( crs_ns, "CropAngle" )) {
            if let Ok(val) = ca_str.parse::<f32>() {
                data.edits.crop_angle = val;
            }
        }
    }
    
    if data.edits.perspective_vertical == 0.0 {
        if let Some(pv_str) = desc.attribute(( crs_ns, "PerspectiveVertical" )) {
            if let Ok(val) = pv_str.parse::<f32>() {
                data.edits.perspective_vertical = val;
            }
        }
    }
    if data.edits.perspective_horizontal == 0.0 {
        if let Some(ph_str) = desc.attribute(( crs_ns, "PerspectiveHorizontal" )) {
            if let Ok(val) = ph_str.parse::<f32>() {
                data.edits.perspective_horizontal = val;
            }
        }
    }
    if let Some(cc_str) = desc.attribute(( crs_ns, "CropConstrainToWarp" )) {
        data.edits.crop_constrain = cc_str == "1" || cc_str == "true";
    }
    
    // Fallback: Parse Orientation hints from third-party editors (Lightroom, Capture One, Darktable, Digikam)
    if data.edits.rotation == 0 && !data.edits.flip_horizontal && !data.edits.flip_vertical {
        let tiff_ns = "http://ns.adobe.com/tiff/1.0/";
        let exif_ns = "http://ns.adobe.com/exif/1.0/";

        let orient_hint = desc.attribute(( tiff_ns, "Orientation" ))
            .or_else(|| desc.attribute(( exif_ns, "Orientation" )))
            .or_else(|| desc.attribute(( crs_ns, "Orientation" )))
            .or_else(|| desc.attribute("Orientation"));

        if let Some(orient_str) = orient_hint {
            if let Ok(num) = orient_str.trim().parse::<u32>() {
                if (1..=8).contains(&num) {
                    let (rot, fh, fv) = crate::raw::decode_exif_orientation(num);
                    data.edits.rotation = rot;
                    data.edits.flip_horizontal = fh;
                    data.edits.flip_vertical = fv;
                }
            } else {
                let trimmed = orient_str.trim();
                if trimmed.eq_ignore_ascii_case("Rotate 90 CW") || trimmed.eq_ignore_ascii_case("Right-top") {
                    data.edits.rotation = 90;
                } else if trimmed.eq_ignore_ascii_case("Rotate 180") || trimmed.eq_ignore_ascii_case("Bottom-right") {
                    data.edits.rotation = 180;
                } else if trimmed.eq_ignore_ascii_case("Rotate 270 CW") || trimmed.eq_ignore_ascii_case("Left-bottom") {
                    data.edits.rotation = 270;
                } else if trimmed.eq_ignore_ascii_case("Horizontal") || trimmed.eq_ignore_ascii_case("Top-left") {
                    data.edits.rotation = 0;
                }
            }
        }
    }
    
    // 4. Parse Title/Description Children
    for child in desc.children() {
        if child.has_tag_name(( "http://purl.org/dc/elements/1.1/", "title" )) {
            data.title = find_rdf_li_val(child);
        } else if child.has_tag_name(( "http://purl.org/dc/elements/1.1/", "description" )) {
            data.description = find_rdf_li_val(child);
        }
    }
    
    Ok(data)
}

/// Resolves the sidecar file path for reading XMP.
/// Supports path directly (if it exists), foo.CR2.xmp (Darktable), and foo.xmp (Lightroom/standard).
pub fn resolve_xmp_read_path<P: AsRef<Path>>(path: P) -> Option<PathBuf> {
    let p = path.as_ref();
    if p.extension().map(|e| e.to_string_lossy().eq_ignore_ascii_case("xmp")).unwrap_or(false) {
        if p.exists() {
            return Some(p.to_path_buf());
        }
    }
    let p_str = p.to_string_lossy();
    let xmp_sidecar = PathBuf::from(format!("{}.xmp", p_str));
    if xmp_sidecar.exists() {
        return Some(xmp_sidecar);
    }
    let xmp_stem = p.with_extension("xmp");
    if xmp_stem.exists() {
        return Some(xmp_stem);
    }
    None
}

/// Resolves the sidecar file path for writing XMP.
/// If an existing sidecar file already exists (e.g. foo.CR2.xmp or foo.xmp), updates that file.
/// Otherwise defaults to foo.xmp.
pub fn resolve_xmp_write_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let p = path.as_ref();
    if p.extension().map(|e| e.to_string_lossy().eq_ignore_ascii_case("xmp")).unwrap_or(false) {
        return p.to_path_buf();
    }
    let p_str = p.to_string_lossy();
    let xmp_sidecar = PathBuf::from(format!("{}.xmp", p_str));
    if xmp_sidecar.exists() {
        return xmp_sidecar;
    }
    p.with_extension("xmp")
}

/// Reads XMP from a file path or photo path by resolving sidecars.
pub fn read_xmp_file<P: AsRef<Path>>(path: P) -> std::io::Result<XmpData> {
    let target = resolve_xmp_read_path(&path).unwrap_or_else(|| path.as_ref().to_path_buf());
    let content = std::fs::read_to_string(target)?;
    parse_xmp(&content)
}

/// Writes XMP to a file path or photo path by resolving sidecars.
pub fn write_xmp_file<P: AsRef<Path>>(path: P, data: &XmpData) -> std::io::Result<()> {
    let target = resolve_xmp_write_path(&path);
    let xml_str = generate_xmp(data);
    std::fs::write(target, xml_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xmp_roundtrip() {
        let mut data = XmpData::default();
        data.rating = Some(4);
        data.status = "Pick".to_string();
        data.title = Some("San Francisco Bridge".to_string());
        data.description = Some("Taken in high fog.".to_string());
        data.latitude = Some(37.8199);
        data.longitude = Some(-122.4783);
        
        data.edits.exposure = 0.85;
        data.edits.contrast = 25.0;
        data.edits.highlights = -40.0;
        data.edits.shadows = 35.0;
        data.edits.whites = 15.0;
        data.edits.blacks = -20.0;
        data.edits.vibrance = 25.0;
        data.edits.saturation = -10.0;
        data.edits.deep_blue = 40.0;
        data.edits.foliage = 30.0;
        data.edits.sharpening_amount = 45.0;
        data.edits.sharpening_radius = 1.5;
        data.edits.sharpening_masking = 25.0;
        data.edits.noise_reduction = 10.0;
        data.edits.temperature = 5800.0;
        data.edits.tint = -12.0;
        data.edits.crop_left = 0.05;
        data.edits.crop_top = 0.05;
        data.edits.crop_right = 0.95;
        data.edits.crop_bottom = 0.95;
        data.edits.crop_angle = -0.5;
        data.edits.perspective_vertical = -15.0;
        data.edits.perspective_horizontal = 8.5;
        data.edits.crop_constrain = true;
        data.edits.rotation = 90;
        data.edits.flip_horizontal = true;
        data.edits.flip_vertical = false;
        
        let path = "test_image_metadata.xmp";
        write_xmp_file(path, &data).unwrap();
        
        let parsed = read_xmp_file(path).unwrap();
        let _ = std::fs::remove_file(path);
        
        assert_eq!(parsed.rating, Some(4));
        assert_eq!(parsed.status, "Pick");
        assert_eq!(parsed.title.as_deref(), Some("San Francisco Bridge"));
        assert_eq!(parsed.description.as_deref(), Some("Taken in high fog."));
        
        // Assert float coordinates within rounding tolerance
        assert!((parsed.latitude.unwrap() - 37.8199).abs() < 0.0001);
        assert!((parsed.longitude.unwrap() - -122.4783).abs() < 0.0001);
        
        assert_eq!(parsed.edits.exposure, 0.85);
        assert_eq!(parsed.edits.contrast, 25.0);
        assert_eq!(parsed.edits.highlights, -40.0);
        assert_eq!(parsed.edits.shadows, 35.0);
        assert_eq!(parsed.edits.whites, 15.0);
        assert_eq!(parsed.edits.blacks, -20.0);
        assert_eq!(parsed.edits.vibrance, 25.0);
        assert_eq!(parsed.edits.saturation, -10.0);
        assert_eq!(parsed.edits.deep_blue, 40.0);
        assert_eq!(parsed.edits.foliage, 30.0);
        assert_eq!(parsed.edits.sharpening_amount, 45.0);
        assert_eq!(parsed.edits.sharpening_radius, 1.5);
        assert_eq!(parsed.edits.sharpening_masking, 25.0);
        assert_eq!(parsed.edits.noise_reduction, 10.0);
        assert_eq!(parsed.edits.temperature, 5800.0);
        assert_eq!(parsed.edits.tint, -12.0);
        assert_eq!(parsed.edits.crop_angle, -0.5);
        assert_eq!(parsed.edits.perspective_vertical, -15.0);
        assert_eq!(parsed.edits.perspective_horizontal, 8.5);
        assert_eq!(parsed.edits.crop_constrain, true);
        assert_eq!(parsed.edits.rotation, 90);
        assert_eq!(parsed.edits.flip_horizontal, true);
        assert_eq!(parsed.edits.flip_vertical, false);
    }

    #[test]
    fn test_xmp_adobe_perspective_ingestion() {
        let adobe_xmp = r#"<x:xmpmeta xmlns:x="adobe:ns:meta/">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about=""
    xmlns:crs="http://ns.adobe.com/camera-raw-settings/1.0/"
    crs:PerspectiveVertical="+25.0"
    crs:PerspectiveHorizontal="-12.5"
    crs:CropConstrainToWarp="1">
  </rdf:Description>
 </rdf:RDF>
</x:xmpmeta>"#;
        let parsed = parse_xmp(adobe_xmp).unwrap();
        assert_eq!(parsed.edits.perspective_vertical, 25.0);
        assert_eq!(parsed.edits.perspective_horizontal, -12.5);
        assert_eq!(parsed.edits.crop_constrain, true);
    }

    #[test]
    fn test_xmp_adobe_tone_recovery_ingestion() {
        let adobe_xmp = r#"<x:xmpmeta xmlns:x="adobe:ns:meta/">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about=""
    xmlns:crs="http://ns.adobe.com/camera-raw-settings/1.0/"
    crs:Highlights2012="-65"
    crs:Shadows2012="+45"
    crs:Whites2012="+20"
    crs:Blacks2012="-15">
  </rdf:Description>
 </rdf:RDF>
</x:xmpmeta>"#;
        let parsed = parse_xmp(adobe_xmp).unwrap();
        assert_eq!(parsed.edits.highlights, -65.0);
        assert_eq!(parsed.edits.shadows, 45.0);
        assert_eq!(parsed.edits.whites, 20.0);
        assert_eq!(parsed.edits.blacks, -15.0);
    }

    #[test]
    fn test_xmp_adobe_color_sliders_ingestion() {
        let adobe_xmp = r#"<x:xmpmeta xmlns:x="adobe:ns:meta/">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about=""
    xmlns:crs="http://ns.adobe.com/camera-raw-settings/1.0/"
    crs:Vibrance="35"
    crs:Saturation="-15">
  </rdf:Description>
 </rdf:RDF>
</x:xmpmeta>"#;
        let parsed = parse_xmp(adobe_xmp).unwrap();
        assert_eq!(parsed.edits.vibrance, 35.0);
        assert_eq!(parsed.edits.saturation, -15.0);
    }

    #[test]
    fn test_xmp_adobe_detail_sliders_ingestion() {
        let adobe_xmp = r#"<x:xmpmeta xmlns:x="adobe:ns:meta/">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about=""
    xmlns:crs="http://ns.adobe.com/camera-raw-settings/1.0/"
    crs:SharpenAmount="40"
    crs:SharpenRadius="1.2"
    crs:SharpenEdgeMasking="20"
    crs:LuminanceSmoothing="15">
  </rdf:Description>
 </rdf:RDF>
</x:xmpmeta>"#;
        let parsed = parse_xmp(adobe_xmp).unwrap();
        assert_eq!(parsed.edits.sharpening_amount, 40.0);
        assert_eq!(parsed.edits.sharpening_radius, 1.2);
        assert_eq!(parsed.edits.sharpening_masking, 20.0);
        assert_eq!(parsed.edits.noise_reduction, 15.0);
    }

    #[test]
    fn test_xmp_sidecar_resolution_from_image_path() {
        let raw_photo = "test_photo.CR2";
        let expected_xmp = "test_photo.xmp";
        
        let mut data = XmpData::default();
        data.edits.exposure = 1.25;
        data.edits.temperature = 6200.0;
        data.status = "Pick".to_string();
        
        // Writing with RAW image path creates test_photo.xmp
        write_xmp_file(raw_photo, &data).unwrap();
        assert!(std::path::Path::new(expected_xmp).exists());
        
        // Reading with RAW image path automatically finds test_photo.xmp
        let read_back = read_xmp_file(raw_photo).unwrap();
        assert_eq!(read_back.edits.exposure, 1.25);
        assert_eq!(read_back.edits.temperature, 6200.0);
        assert_eq!(read_back.status, "Pick");
        
        let _ = std::fs::remove_file(expected_xmp);
    }

    #[test]
    fn test_xmp_third_party_orientation_hints() {
        // 1. Adobe Lightroom / Camera Raw with tiff:Orientation="6"
        let lr_xmp = r#"<x:xmpmeta xmlns:x="adobe:ns:meta/">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about=""
    xmlns:tiff="http://ns.adobe.com/tiff/1.0/"
    xmlns:crs="http://ns.adobe.com/camera-raw-settings/1.0/"
    tiff:Orientation="6"
    crs:Orientation="Rotate 90 CW">
  </rdf:Description>
 </rdf:RDF>
</x:xmpmeta>"#;
        let parsed_lr = parse_xmp(lr_xmp).unwrap();
        assert_eq!(parsed_lr.edits.rotation, 90);
        assert_eq!(parsed_lr.edits.flip_horizontal, false);
        assert_eq!(parsed_lr.edits.flip_vertical, false);

        // 2. Capture One / Darktable with tiff:Orientation="8" (270 deg)
        let c1_xmp = r#"<x:xmpmeta xmlns:x="adobe:ns:meta/">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about=""
    xmlns:tiff="http://ns.adobe.com/tiff/1.0/"
    tiff:Orientation="8">
  </rdf:Description>
 </rdf:RDF>
</x:xmpmeta>"#;
        let parsed_c1 = parse_xmp(c1_xmp).unwrap();
        assert_eq!(parsed_c1.edits.rotation, 270);
        assert_eq!(parsed_c1.edits.flip_horizontal, false);

        // 3. Front camera / scanner mirrored portrait: orientation 5 (Transpose: 270 deg + flip H)
        let mirror_xmp = r#"<x:xmpmeta xmlns:x="adobe:ns:meta/">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about=""
    xmlns:exif="http://ns.adobe.com/exif/1.0/"
    exif:Orientation="5">
  </rdf:Description>
 </rdf:RDF>
</x:xmpmeta>"#;
        let parsed_mirror = parse_xmp(mirror_xmp).unwrap();
        assert_eq!(parsed_mirror.edits.rotation, 270);
        assert_eq!(parsed_mirror.edits.flip_horizontal, true);
    }

    #[test]
    fn test_xmp_curves_roundtrip() {
        let mut data = XmpData::default();
        data.edits.curves.master = vec![
            CurvePointRecord { x: 0.0, y: 0.05 },
            CurvePointRecord { x: 0.25, y: 0.20 },
            CurvePointRecord { x: 0.75, y: 0.85 },
            CurvePointRecord { x: 1.0, y: 0.98 },
        ];
        data.edits.curves.red = vec![
            CurvePointRecord { x: 0.0, y: 0.0 },
            CurvePointRecord { x: 0.5, y: 0.6 },
            CurvePointRecord { x: 1.0, y: 1.0 },
        ];

        let xml = generate_xmp(&data);
        assert!(xml.contains("curveMaster"));
        assert!(xml.contains("curveRed"));

        let parsed = parse_xmp(&xml).unwrap();
        assert_eq!(parsed.edits.curves.master.len(), 4);
        assert!((parsed.edits.curves.master[0].y - 0.05).abs() < 1e-3);
        assert!((parsed.edits.curves.master[2].y - 0.85).abs() < 1e-3);
        assert_eq!(parsed.edits.curves.red.len(), 3);
        assert!((parsed.edits.curves.red[1].y - 0.6).abs() < 1e-3);
        assert_eq!(parsed.edits.curves.green.len(), 2);
        assert_eq!(parsed.edits.curves.blue.len(), 2);
    }
}
