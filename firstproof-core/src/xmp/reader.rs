use roxmltree::{Document, Node};
use super::data::{parse_curve_points, parse_gps_xmp, XmpData};

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

/// Reads all li child node values out of an rdf:Bag or rdf:Seq (e.g. dc:subject, lr:hierarchicalSubject).
fn find_rdf_li_vals(node: Node) -> Vec<String> {
    let mut items = Vec::new();
    for child in node.descendants() {
        if child.has_tag_name(( "http://www.w3.org/1999/02/22-rdf-syntax-ns#", "li" )) || child.tag_name().name() == "li" {
            if let Some(text) = child.text() {
                let trimmed = text.trim();
                if !trimmed.is_empty() && !items.contains(&trimmed.to_string()) {
                    items.push(trimmed.to_string());
                }
            }
        }
    }
    items
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
        if ns == "https://firstproof.photo/ns/pm/1.0/" {
            get_prop(&desc, ns, name).or_else(|| get_prop(&desc, "http://ns.lumenbox.app/pm/1.0/", name))
        } else {
            get_prop(&desc, ns, name)
        }
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
    let pm = "https://firstproof.photo/ns/pm/1.0/";
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
    
    // 4. Parse Title/Description/Creator/Copyright/Keywords Children
    for child in desc.children() {
        let tag = child.tag_name().name();
        let ns = child.tag_name().namespace().unwrap_or("");
        let is_dc = ns == "http://purl.org/dc/elements/1.1/" || ns.is_empty();
        let is_photoshop = ns == "http://ns.adobe.com/photoshop/1.0/";
        let is_lr = ns == "http://ns.adobe.com/lightroom/1.0/";

        if is_dc && tag == "title" {
            data.title = find_rdf_li_val(child);
        } else if is_photoshop && tag == "Headline" && data.title.is_none() {
            if let Some(text) = child.text() {
                data.title = Some(text.trim().to_string());
            }
        } else if is_dc && tag == "description" {
            data.description = find_rdf_li_val(child);
        } else if is_dc && tag == "creator" {
            data.creator = find_rdf_li_val(child);
        } else if is_photoshop && tag == "By-line" && data.creator.is_none() {
            if let Some(text) = child.text() {
                data.creator = Some(text.trim().to_string());
            }
        } else if is_dc && tag == "rights" {
            data.copyright = find_rdf_li_val(child);
        } else if (is_dc && tag == "subject") || (is_lr && tag == "hierarchicalSubject") {
            let kws = find_rdf_li_vals(child);
            for kw in kws {
                if !data.keywords.contains(&kw) {
                    data.keywords.push(kw);
                }
            }
        }
    }
    
    if data.title.is_none() {
        if let Some(t) = get("http://ns.adobe.com/photoshop/1.0/", "Headline") {
            data.title = Some(t.to_string());
        }
    }
    if data.creator.is_none() {
        if let Some(c) = get("http://ns.adobe.com/photoshop/1.0/", "By-line") {
            data.creator = Some(c.to_string());
        }
    }
    if data.copyright.is_none() {
        if let Some(r) = get("http://purl.org/dc/elements/1.1/", "rights") {
            data.copyright = Some(r.to_string());
        }
    }
    
    Ok(data)
}
