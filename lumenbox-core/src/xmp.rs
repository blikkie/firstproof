use roxmltree::{Document, Node};
use std::path::Path;
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
            edits: EditSettings {
                exposure: 0.0,
                contrast: 0.0,
                temperature: 5000.0,
                tint: 0.0,
                crop_left: 0.0,
                crop_top: 0.0,
                crop_right: 1.0,
                crop_bottom: 1.0,
                crop_angle: 0.0,
            },
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

fn escape_xml(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

/// Generates XMP XML sidecar string.
pub fn generate_xmp(data: &XmpData) -> String {
    let rating_attr = match data.rating {
        Some(r) => format!(" xmp:Rating=\"{}\"", r),
        None => "".to_string(),
    };
    
    let gps_attrs = match (data.latitude, data.longitude) {
        (Some(lat), Some(lon)) => format!(
            " exif:GPSLatitude=\"{}\" exif:GPSLongitude=\"{}\" exif:GPSVersionID=\"2.2.0.0\"",
            format_gps_xmp(lat, true),
            format_gps_xmp(lon, false)
        ),
        _ => "".to_string(),
    };
    
    let title_block = match &data.title {
        Some(t) => format!(
            "   <dc:title>\n    <rdf:Alt>\n     <rdf:li xml:lang=\"x-default\">{}</rdf:li>\n    </rdf:Alt>\n   </dc:title>\n",
            escape_xml(t)
        ),
        None => "".to_string(),
    };
    
    let desc_block = match &data.description {
        Some(d) => format!(
            "   <dc:description>\n    <rdf:Alt>\n     <rdf:li xml:lang=\"x-default\">{}</rdf:li>\n    </rdf:Alt>\n   </dc:description>\n",
            escape_xml(d)
        ),
        None => "".to_string(),
    };
    
    format!(
        r#"<x:xmpmeta xmlns:x="adobe:ns:meta/" x:xmptk="LumenBox Core 0.1.0">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about=""
    xmlns:xmp="http://ns.adobe.com/xap/1.0/"
    xmlns:dc="http://purl.org/dc/elements/1.1/"
    xmlns:exif="http://ns.adobe.com/exif/1.0/"
    xmlns:pm="http://ns.lumenbox.app/pm/1.0/"{rating_attr}{gps_attrs}
    pm:status="{status}"
    pm:exposure="{exposure:.4}"
    pm:contrast="{contrast:.4}"
    pm:temperature="{temperature:.4}"
    pm:tint="{tint:.4}"
    pm:cropLeft="{crop_left:.4}"
    pm:cropTop="{crop_top:.4}"
    pm:cropRight="{crop_right:.4}"
    pm:cropBottom="{crop_bottom:.4}"
    pm:cropAngle="{crop_angle:.4}">
{title_block}{desc_block}  </rdf:Description>
 </rdf:RDF>
</x:xmpmeta>"#,
        status = data.status,
        exposure = data.edits.exposure,
        contrast = data.edits.contrast,
        temperature = data.edits.temperature,
        tint = data.edits.tint,
        crop_left = data.edits.crop_left,
        crop_top = data.edits.crop_top,
        crop_right = data.edits.crop_right,
        crop_bottom = data.edits.crop_bottom,
        crop_angle = data.edits.crop_angle,
        rating_attr = rating_attr,
        gps_attrs = gps_attrs,
        title_block = title_block,
        desc_block = desc_block
    )
}

/// Reads title or description child node out of the dc:title/dc:description blocks.
fn find_rdf_li_val(node: Node) -> Option<String> {
    let rdf_alt = node.first_element_child()?;
    let rdf_li = rdf_alt.first_element_child()?;
    Some(rdf_li.text()?.to_string())
}

/// Parses XMP XML sidecar string.
pub fn parse_xmp(xmp_str: &str) -> std::io::Result<XmpData> {
    let doc = Document::parse(xmp_str)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        
    let root = doc.root_element();
    
    // Find Description node inside rdf:RDF
    let rdf_rdf = root.first_element_child()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Missing rdf:RDF node"))?;
        
    let desc = rdf_rdf.first_element_child()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Missing rdf:Description node"))?;
        
    let mut data = XmpData::default();
    
    // 1. Parse standard attributes
    let raw_rating = desc.attribute(( "http://ns.adobe.com/xap/1.0/", "Rating" ))
        .or_else(|| desc.attribute("Rating"))
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
    
    if let Some(lat_str) = desc.attribute(( "http://ns.adobe.com/exif/1.0/", "GPSLatitude" )) {
        data.latitude = parse_gps_xmp(lat_str);
    }
    if let Some(lon_str) = desc.attribute(( "http://ns.adobe.com/exif/1.0/", "GPSLongitude" )) {
        data.longitude = parse_gps_xmp(lon_str);
    }
    
    // 2. Parse Custom Adjustments (pm:)
    if let Some(status_str) = desc.attribute(( "http://ns.lumenbox.app/pm/1.0/", "status" )) {
        data.status = status_str.to_string();
    }
    if let Some(exp_str) = desc.attribute(( "http://ns.lumenbox.app/pm/1.0/", "exposure" )) {
        if let Ok(val) = exp_str.parse::<f32>() {
            data.edits.exposure = val;
        }
    }
    if let Some(con_str) = desc.attribute(( "http://ns.lumenbox.app/pm/1.0/", "contrast" )) {
        if let Ok(val) = con_str.parse::<f32>() {
            data.edits.contrast = val;
        }
    }
    if let Some(temp_str) = desc.attribute(( "http://ns.lumenbox.app/pm/1.0/", "temperature" )) {
        if let Ok(val) = temp_str.parse::<f32>() {
            data.edits.temperature = val;
        }
    }
    if let Some(tint_str) = desc.attribute(( "http://ns.lumenbox.app/pm/1.0/", "tint" )) {
        if let Ok(val) = tint_str.parse::<f32>() {
            data.edits.tint = val;
        }
    }
    if let Some(cl_str) = desc.attribute(( "http://ns.lumenbox.app/pm/1.0/", "cropLeft" )) {
        if let Ok(val) = cl_str.parse::<f32>() {
            data.edits.crop_left = val;
        }
    }
    if let Some(ct_str) = desc.attribute(( "http://ns.lumenbox.app/pm/1.0/", "cropTop" )) {
        if let Ok(val) = ct_str.parse::<f32>() {
            data.edits.crop_top = val;
        }
    }
    if let Some(cr_str) = desc.attribute(( "http://ns.lumenbox.app/pm/1.0/", "cropRight" )) {
        if let Ok(val) = cr_str.parse::<f32>() {
            data.edits.crop_right = val;
        }
    }
    if let Some(cb_str) = desc.attribute(( "http://ns.lumenbox.app/pm/1.0/", "cropBottom" )) {
        if let Ok(val) = cb_str.parse::<f32>() {
            data.edits.crop_bottom = val;
        }
    }
    if let Some(ca_str) = desc.attribute(( "http://ns.lumenbox.app/pm/1.0/", "cropAngle" )) {
        if let Ok(val) = ca_str.parse::<f32>() {
            data.edits.crop_angle = val;
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

/// Reads XMP from a file path.
pub fn read_xmp_file<P: AsRef<Path>>(path: P) -> std::io::Result<XmpData> {
    let content = std::fs::read_to_string(path)?;
    parse_xmp(&content)
}

/// Writes XMP to a file path.
pub fn write_xmp_file<P: AsRef<Path>>(path: P, data: &XmpData) -> std::io::Result<()> {
    let xml_str = generate_xmp(data);
    std::fs::write(path, xml_str)
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
        data.edits.temperature = 5800.0;
        data.edits.tint = -12.0;
        data.edits.crop_left = 0.05;
        data.edits.crop_top = 0.05;
        data.edits.crop_right = 0.95;
        data.edits.crop_bottom = 0.95;
        data.edits.crop_angle = -0.5;
        
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
        assert_eq!(parsed.edits.temperature, 5800.0);
        assert_eq!(parsed.edits.tint, -12.0);
        assert_eq!(parsed.edits.crop_angle, -0.5);
    }
}
