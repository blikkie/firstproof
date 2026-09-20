use crate::curve::CurvePointRecord;
use crate::modes::{
    ColorProfileMode, EmulsionArchetype, EmulsionFormat, EnlargerFilterMode, PrintMediaMode,
};
use crate::settings::EditSettings;
use roxmltree::{Document, Node};
use serde::{Deserialize, Serialize};
use xmp_writer::{CustomNamespace, Namespace, XmpWriter};

#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct XmpData {
    pub rating: Option<i32>,
    pub status: String, // "Pick", "Reject", "None"
    pub title: Option<String>,
    pub description: Option<String>,
    pub creator: Option<String>,
    pub copyright: Option<String>,
    pub keywords: Vec<String>,
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
            creator: None,
            copyright: None,
            keywords: Vec::new(),
            latitude: None,
            longitude: None,
            edits: EditSettings::default(),
        }
    }
}

pub fn format_gps_xmp(val: f64, is_lat: bool) -> String {
    let abs_val = val.abs();
    let deg = abs_val.floor() as u32;
    let min = (abs_val - deg as f64) * 60.0;
    let dir = if is_lat {
        if val >= 0.0 {
            "N"
        } else {
            "S"
        }
    } else {
        if val >= 0.0 {
            "E"
        } else {
            "W"
        }
    };
    format!("{},{:.4}{}", deg, min, dir)
}

pub fn parse_gps_xmp(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let last_char = s.chars().last()?;
    let is_negative = match last_char {
        'S' | 's' | 'W' | 'w' => true,
        'N' | 'n' | 'E' | 'e' => false,
        _ => return None,
    };
    let coord_part = &s[..s.len() - 1];
    let parts: Vec<&str> = coord_part.split(',').collect();
    if parts.is_empty() {
        return None;
    }
    let deg: f64 = parts[0].trim().parse().ok()?;
    let min: f64 = if parts.len() > 1 {
        parts[1].trim().parse().ok()?
    } else {
        0.0
    };
    let sec: f64 = if parts.len() > 2 {
        parts[2].trim().parse().ok()?
    } else {
        0.0
    };
    let decimal = deg + (min / 60.0) + (sec / 3600.0);
    if is_negative {
        Some(-decimal)
    } else {
        Some(decimal)
    }
}

pub fn serialize_curve_points(pts: &[CurvePointRecord]) -> String {
    pts.iter()
        .map(|p| format!("{:.4},{:.4}", p.x, p.y))
        .collect::<Vec<_>>()
        .join(";")
}

pub fn parse_curve_points(s: &str) -> Vec<CurvePointRecord> {
    let mut out = Vec::new();
    for token in s.split(';') {
        let parts: Vec<&str> = token.split(',').collect();
        if parts.len() == 2 {
            if let (Ok(x), Ok(y)) = (
                parts[0].trim().parse::<f32>(),
                parts[1].trim().parse::<f32>(),
            ) {
                out.push(CurvePointRecord { x, y });
            }
        }
    }
    if out.is_empty() {
        vec![
            CurvePointRecord { x: 0.0, y: 0.0 },
            CurvePointRecord { x: 1.0, y: 1.0 },
        ]
    } else {
        out
    }
}

pub fn generate_xmp_sidecar(data: &XmpData) -> String {
    let mut writer = XmpWriter::new();

    if let Some(t) = &data.title {
        writer.title([(None, t.as_str())]);
    }
    if let Some(d) = &data.description {
        writer.description([(None, d.as_str())]);
    }
    if let Some(c) = &data.creator {
        writer.creator([c.as_str()]);
    }
    if let Some(r) = &data.copyright {
        writer.rights([(None, r.as_str())]);
    }
    if !data.keywords.is_empty() {
        writer.subject(data.keywords.iter().map(|s| s.as_str()));
    }
    if let Some(r) = data.rating {
        writer.element("Rating", Namespace::Xmp).value(r);
    }
    let exif_ns = Namespace::Custom(Box::new(CustomNamespace::new(
        "Exif",
        "exif",
        "http://ns.adobe.com/exif/1.0/",
    )));
    if let (Some(lat), Some(lon)) = (data.latitude, data.longitude) {
        writer
            .element("GPSLatitude", exif_ns.clone())
            .value(format_gps_xmp(lat, true).as_str());
        writer
            .element("GPSLongitude", exif_ns.clone())
            .value(format_gps_xmp(lon, false).as_str());
        writer.element("GPSVersionID", exif_ns).value("2.2.0.0");
    }

    let pm_ns = Namespace::Custom(Box::new(CustomNamespace::new(
        "First Proof Photo Metadata",
        "pm",
        "https://firstproof.photo/ns/pm/1.0/",
    )));

    writer
        .element("status", pm_ns.clone())
        .value(data.status.as_str());
    writer
        .element("exposure", pm_ns.clone())
        .value(format!("{:.4}", data.edits.exposure).as_str());
    writer
        .element("contrast", pm_ns.clone())
        .value(format!("{:.4}", data.edits.contrast).as_str());
    writer
        .element("highlights", pm_ns.clone())
        .value(format!("{:.4}", data.edits.highlights).as_str());
    writer
        .element("shadows", pm_ns.clone())
        .value(format!("{:.4}", data.edits.shadows).as_str());
    writer
        .element("whites", pm_ns.clone())
        .value(format!("{:.4}", data.edits.whites).as_str());
    writer
        .element("blacks", pm_ns.clone())
        .value(format!("{:.4}", data.edits.blacks).as_str());
    writer
        .element("vibrance", pm_ns.clone())
        .value(format!("{:.4}", data.edits.vibrance).as_str());
    writer
        .element("saturation", pm_ns.clone())
        .value(format!("{:.4}", data.edits.saturation).as_str());
    writer
        .element("deepBlue", pm_ns.clone())
        .value(format!("{:.4}", data.edits.deep_blue).as_str());
    writer
        .element("foliage", pm_ns.clone())
        .value(format!("{:.4}", data.edits.foliage).as_str());
    writer
        .element("temperature", pm_ns.clone())
        .value(format!("{:.4}", data.edits.temperature).as_str());
    writer
        .element("tint", pm_ns.clone())
        .value(format!("{:.4}", data.edits.tint).as_str());
    writer
        .element("cropLeft", pm_ns.clone())
        .value(format!("{:.4}", data.edits.crop_left).as_str());
    writer
        .element("cropTop", pm_ns.clone())
        .value(format!("{:.4}", data.edits.crop_top).as_str());
    writer
        .element("cropRight", pm_ns.clone())
        .value(format!("{:.4}", data.edits.crop_right).as_str());
    writer
        .element("cropBottom", pm_ns.clone())
        .value(format!("{:.4}", data.edits.crop_bottom).as_str());
    writer
        .element("cropAngle", pm_ns.clone())
        .value(format!("{:.4}", data.edits.crop_angle).as_str());
    writer
        .element("perspectiveVertical", pm_ns.clone())
        .value(format!("{:.4}", data.edits.perspective_vertical).as_str());
    writer
        .element("perspectiveHorizontal", pm_ns.clone())
        .value(format!("{:.4}", data.edits.perspective_horizontal).as_str());
    writer
        .element("cropConstrain", pm_ns.clone())
        .value(if data.edits.crop_constrain {
            "true"
        } else {
            "false"
        });
    writer
        .element("rotation", pm_ns.clone())
        .value(format!("{}", data.edits.rotation).as_str());
    writer
        .element("flipHorizontal", pm_ns.clone())
        .value(if data.edits.flip_horizontal {
            "true"
        } else {
            "false"
        });
    writer
        .element("flipVertical", pm_ns.clone())
        .value(if data.edits.flip_vertical {
            "true"
        } else {
            "false"
        });
    writer
        .element("toneProfile", pm_ns.clone())
        .value(match data.edits.tone_profile {
            ColorProfileMode::AgxVivid => "AgxVivid",
            ColorProfileMode::AgxFilmic => "AgxFilmic",
            ColorProfileMode::CameraStandard => "CameraStandard",
            ColorProfileMode::CameraPortrait => "CameraPortrait",
            ColorProfileMode::CameraLandscape => "CameraLandscape",
            ColorProfileMode::CameraVivid => "CameraVivid",
        });
    if let Some(lut) = &data.edits.lut_name {
        writer.element("lutName", pm_ns.clone()).value(lut.as_str());
    }
    writer
        .element("lutIntensity", pm_ns.clone())
        .value(format!("{:.4}", data.edits.lut_intensity).as_str());
    writer
        .element("sharpeningAmount", pm_ns.clone())
        .value(format!("{:.4}", data.edits.sharpening_amount).as_str());
    writer
        .element("sharpeningRadius", pm_ns.clone())
        .value(format!("{:.4}", data.edits.sharpening_radius).as_str());
    writer
        .element("sharpeningMasking", pm_ns.clone())
        .value(format!("{:.4}", data.edits.sharpening_masking).as_str());
    writer
        .element("noiseReduction", pm_ns.clone())
        .value(format!("{:.4}", data.edits.noise_reduction).as_str());

    writer
        .element("emulsionArchetype", pm_ns.clone())
        .value(match data.edits.emulsion_archetype {
            EmulsionArchetype::None => "None",
            EmulsionArchetype::RochesterPortrait400 => "RochesterPortrait400",
            EmulsionArchetype::AshigaraChrome50 => "AshigaraChrome50",
            EmulsionArchetype::RochesterK14Chrome64 => "RochesterK14Chrome64",
            EmulsionArchetype::RochesterCinema500 => "RochesterCinema500",
            EmulsionArchetype::RochesterDaylight200 => "RochesterDaylight200",
            EmulsionArchetype::MobberleySilver400 => "MobberleySilver400",
            EmulsionArchetype::CambridgeDiffusion100 => "CambridgeDiffusion100",
        });
    writer
        .element("emulsionFormat", pm_ns.clone())
        .value(match data.edits.emulsion_format {
            EmulsionFormat::Format35mm => "Format35mm",
            EmulsionFormat::Format120 => "Format120",
            EmulsionFormat::Format4x5 => "Format4x5",
        });
    writer
        .element("emulsionExposure", pm_ns.clone())
        .value(format!("{:.4}", data.edits.emulsion_exposure).as_str());

    writer
        .element("splitToningShadowHue", pm_ns.clone())
        .value(format!("{:.4}", data.edits.split_toning_shadow_hue).as_str());
    writer
        .element("splitToningShadowSat", pm_ns.clone())
        .value(format!("{:.4}", data.edits.split_toning_shadow_sat).as_str());
    writer
        .element("splitToningHighlightHue", pm_ns.clone())
        .value(format!("{:.4}", data.edits.split_toning_highlight_hue).as_str());
    writer
        .element("splitToningHighlightSat", pm_ns.clone())
        .value(format!("{:.4}", data.edits.split_toning_highlight_sat).as_str());
    writer
        .element("splitToningBalance", pm_ns.clone())
        .value(format!("{:.4}", data.edits.split_toning_balance).as_str());

    writer
        .element("printMedia", pm_ns.clone())
        .value(match data.edits.print_media {
            PrintMediaMode::None => "None",
            PrintMediaMode::ChromogenicTypeC => "ChromogenicTypeC",
            PrintMediaMode::SilverDyeBleach => "SilverDyeBleach",
            PrintMediaMode::FineArtCottonRag => "FineArtCottonRag",
            PrintMediaMode::BarytaSilverGelatin => "BarytaSilverGelatin",
            PrintMediaMode::WarmtoneChlorobromide => "WarmtoneChlorobromide",
            PrintMediaMode::PlatinumPalladium => "PlatinumPalladium",
            PrintMediaMode::Cyanotype => "Cyanotype",
        });
    writer
        .element("enlargerFilter", pm_ns.clone())
        .value(match data.edits.enlarger_filter {
            EnlargerFilterMode::NeutralPanchromatic => "NeutralPanchromatic",
            EnlargerFilterMode::YellowK2 => "YellowK2",
            EnlargerFilterMode::Orange16 => "Orange16",
            EnlargerFilterMode::Red25A => "Red25A",
            EnlargerFilterMode::GreenX1 => "GreenX1",
            EnlargerFilterMode::Orthochromatic => "Orthochromatic",
        });
    writer
        .element("printPaperGrade", pm_ns.clone())
        .value(format!("{:.4}", data.edits.print_paper_grade).as_str());
    writer
        .element("printBaseWarmth", pm_ns.clone())
        .value(format!("{:.4}", data.edits.print_base_warmth).as_str());
    writer
        .element("printProcessDepth", pm_ns.clone())
        .value(format!("{:.4}", data.edits.print_process_depth).as_str());

    writer
        .element("toneCurveMaster", pm_ns.clone())
        .value(serialize_curve_points(&data.edits.curves.master).as_str());
    writer
        .element("toneCurveRed", pm_ns.clone())
        .value(serialize_curve_points(&data.edits.curves.red).as_str());
    writer
        .element("toneCurveGreen", pm_ns.clone())
        .value(serialize_curve_points(&data.edits.curves.green).as_str());
    writer
        .element("toneCurveBlue", pm_ns)
        .value(serialize_curve_points(&data.edits.curves.blue).as_str());

    writer.finish(None)
}

fn find_rdf_li_val(node: Node) -> Option<String> {
    for child in node.descendants() {
        if child.has_tag_name(("http://www.w3.org/1999/02/22-rdf-syntax-ns#", "li"))
            || child.tag_name().name() == "li"
        {
            if let Some(text) = child.text() {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
    }
    if let Some(text) = node.text() {
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    None
}

fn find_rdf_li_vals(node: Node) -> Vec<String> {
    let mut items = Vec::new();
    for child in node.descendants() {
        if child.has_tag_name(("http://www.w3.org/1999/02/22-rdf-syntax-ns#", "li"))
            || child.tag_name().name() == "li"
        {
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

pub fn parse_xmp_sidecar(xml: &str) -> Result<XmpData, String> {
    let doc = Document::parse(xml).map_err(|e| e.to_string())?;
    let mut data = XmpData::default();

    for node in doc.descendants() {
        if node.is_element() {
            let name = node.tag_name().name();
            let text = node.text().unwrap_or("").trim();
            match name {
                "Rating" => {
                    if let Ok(r) = text.parse::<i32>() {
                        data.rating = Some(r);
                    }
                }
                "status" => data.status = text.to_string(),
                "title" => {
                    if let Some(t) = find_rdf_li_val(node) {
                        data.title = Some(t);
                    }
                }
                "description" => {
                    if let Some(d) = find_rdf_li_val(node) {
                        data.description = Some(d);
                    }
                }
                "creator" => {
                    if let Some(c) = find_rdf_li_val(node) {
                        data.creator = Some(c);
                    }
                }
                "rights" => {
                    if let Some(r) = find_rdf_li_val(node) {
                        data.copyright = Some(r);
                    }
                }
                "subject" => {
                    for kw in find_rdf_li_vals(node) {
                        if !data.keywords.contains(&kw) {
                            data.keywords.push(kw);
                        }
                    }
                }
                "GPSLatitude" => {
                    data.latitude = parse_gps_xmp(text);
                }
                "GPSLongitude" => {
                    data.longitude = parse_gps_xmp(text);
                }
                "exposure" => {
                    if let Ok(v) = text.parse::<f32>() {
                        data.edits.exposure = v;
                    }
                }
                "contrast" => {
                    if let Ok(v) = text.parse::<f32>() {
                        data.edits.contrast = v;
                    }
                }
                "highlights" => {
                    if let Ok(v) = text.parse::<f32>() {
                        data.edits.highlights = v;
                    }
                }
                "shadows" => {
                    if let Ok(v) = text.parse::<f32>() {
                        data.edits.shadows = v;
                    }
                }
                "whites" => {
                    if let Ok(v) = text.parse::<f32>() {
                        data.edits.whites = v;
                    }
                }
                "blacks" => {
                    if let Ok(v) = text.parse::<f32>() {
                        data.edits.blacks = v;
                    }
                }
                "vibrance" => {
                    if let Ok(v) = text.parse::<f32>() {
                        data.edits.vibrance = v;
                    }
                }
                "saturation" => {
                    if let Ok(v) = text.parse::<f32>() {
                        data.edits.saturation = v;
                    }
                }
                "deepBlue" => {
                    if let Ok(v) = text.parse::<f32>() {
                        data.edits.deep_blue = v;
                    }
                }
                "foliage" => {
                    if let Ok(v) = text.parse::<f32>() {
                        data.edits.foliage = v;
                    }
                }
                "temperature" => {
                    if let Ok(v) = text.parse::<f32>() {
                        data.edits.temperature = v;
                    }
                }
                "tint" => {
                    if let Ok(v) = text.parse::<f32>() {
                        data.edits.tint = v;
                    }
                }
                "cropLeft" => {
                    if let Ok(v) = text.parse::<f32>() {
                        data.edits.crop_left = v;
                    }
                }
                "cropTop" => {
                    if let Ok(v) = text.parse::<f32>() {
                        data.edits.crop_top = v;
                    }
                }
                "cropRight" => {
                    if let Ok(v) = text.parse::<f32>() {
                        data.edits.crop_right = v;
                    }
                }
                "cropBottom" => {
                    if let Ok(v) = text.parse::<f32>() {
                        data.edits.crop_bottom = v;
                    }
                }
                "cropAngle" => {
                    if let Ok(v) = text.parse::<f32>() {
                        data.edits.crop_angle = v;
                    }
                }
                "perspectiveVertical" => {
                    if let Ok(v) = text.parse::<f32>() {
                        data.edits.perspective_vertical = v;
                    }
                }
                "perspectiveHorizontal" => {
                    if let Ok(v) = text.parse::<f32>() {
                        data.edits.perspective_horizontal = v;
                    }
                }
                "cropConstrain" => {
                    data.edits.crop_constrain = text == "true" || text == "1";
                }
                "rotation" => {
                    if let Ok(v) = text.parse::<i32>() {
                        data.edits.rotation = v;
                    }
                }
                "flipHorizontal" => data.edits.flip_horizontal = text == "true" || text == "1",
                "flipVertical" => data.edits.flip_vertical = text == "true" || text == "1",
                "toneProfile" => {
                    data.edits.tone_profile = match text {
                        "AgxVivid" => ColorProfileMode::AgxVivid,
                        "AgxFilmic" => ColorProfileMode::AgxFilmic,
                        "CameraStandard" => ColorProfileMode::CameraStandard,
                        "CameraPortrait" => ColorProfileMode::CameraPortrait,
                        "CameraLandscape" => ColorProfileMode::CameraLandscape,
                        "CameraVivid" => ColorProfileMode::CameraVivid,
                        _ => ColorProfileMode::AgxVivid,
                    };
                }
                "lutName" => {
                    if !text.is_empty() {
                        data.edits.lut_name = Some(text.to_string());
                    }
                }
                "lutIntensity" => {
                    if let Ok(v) = text.parse::<f32>() {
                        data.edits.lut_intensity = v;
                    }
                }
                "sharpeningAmount" => {
                    if let Ok(v) = text.parse::<f32>() {
                        data.edits.sharpening_amount = v;
                    }
                }
                "sharpeningRadius" => {
                    if let Ok(v) = text.parse::<f32>() {
                        data.edits.sharpening_radius = v;
                    }
                }
                "sharpeningMasking" => {
                    if let Ok(v) = text.parse::<f32>() {
                        data.edits.sharpening_masking = v;
                    }
                }
                "noiseReduction" => {
                    if let Ok(v) = text.parse::<f32>() {
                        data.edits.noise_reduction = v;
                    }
                }
                "emulsionArchetype" => {
                    data.edits.emulsion_archetype = match text {
                        "RochesterPortrait400" => EmulsionArchetype::RochesterPortrait400,
                        "AshigaraChrome50" => EmulsionArchetype::AshigaraChrome50,
                        "RochesterK14Chrome64" => EmulsionArchetype::RochesterK14Chrome64,
                        "RochesterCinema500" => EmulsionArchetype::RochesterCinema500,
                        "RochesterDaylight200" => EmulsionArchetype::RochesterDaylight200,
                        "MobberleySilver400" => EmulsionArchetype::MobberleySilver400,
                        "CambridgeDiffusion100" => EmulsionArchetype::CambridgeDiffusion100,
                        _ => EmulsionArchetype::None,
                    };
                }
                "emulsionFormat" => {
                    data.edits.emulsion_format = match text {
                        "Format120" => EmulsionFormat::Format120,
                        "Format4x5" => EmulsionFormat::Format4x5,
                        _ => EmulsionFormat::Format35mm,
                    };
                }
                "emulsionExposure" => {
                    if let Ok(v) = text.parse::<f32>() {
                        data.edits.emulsion_exposure = v;
                    }
                }
                "splitToningShadowHue" => {
                    if let Ok(v) = text.parse::<f32>() {
                        data.edits.split_toning_shadow_hue = v;
                    }
                }
                "splitToningShadowSat" => {
                    if let Ok(v) = text.parse::<f32>() {
                        data.edits.split_toning_shadow_sat = v;
                    }
                }
                "splitToningHighlightHue" => {
                    if let Ok(v) = text.parse::<f32>() {
                        data.edits.split_toning_highlight_hue = v;
                    }
                }
                "splitToningHighlightSat" => {
                    if let Ok(v) = text.parse::<f32>() {
                        data.edits.split_toning_highlight_sat = v;
                    }
                }
                "splitToningBalance" => {
                    if let Ok(v) = text.parse::<f32>() {
                        data.edits.split_toning_balance = v;
                    }
                }
                "printMedia" => {
                    data.edits.print_media = match text {
                        "ChromogenicTypeC" => PrintMediaMode::ChromogenicTypeC,
                        "SilverDyeBleach" => PrintMediaMode::SilverDyeBleach,
                        "FineArtCottonRag" => PrintMediaMode::FineArtCottonRag,
                        "BarytaSilverGelatin" => PrintMediaMode::BarytaSilverGelatin,
                        "WarmtoneChlorobromide" => PrintMediaMode::WarmtoneChlorobromide,
                        "PlatinumPalladium" => PrintMediaMode::PlatinumPalladium,
                        "Cyanotype" => PrintMediaMode::Cyanotype,
                        _ => PrintMediaMode::None,
                    };
                }
                "enlargerFilter" => {
                    data.edits.enlarger_filter = match text {
                        "YellowK2" => EnlargerFilterMode::YellowK2,
                        "Orange16" => EnlargerFilterMode::Orange16,
                        "Red25A" => EnlargerFilterMode::Red25A,
                        "GreenX1" => EnlargerFilterMode::GreenX1,
                        "Orthochromatic" => EnlargerFilterMode::Orthochromatic,
                        _ => EnlargerFilterMode::NeutralPanchromatic,
                    };
                }
                "printPaperGrade" => {
                    if let Ok(v) = text.parse::<f32>() {
                        data.edits.print_paper_grade = v;
                    }
                }
                "printBaseWarmth" => {
                    if let Ok(v) = text.parse::<f32>() {
                        data.edits.print_base_warmth = v;
                    }
                }
                "printProcessDepth" => {
                    if let Ok(v) = text.parse::<f32>() {
                        data.edits.print_process_depth = v;
                    }
                }
                "toneCurveMaster" => data.edits.curves.master = parse_curve_points(text),
                "toneCurveRed" => data.edits.curves.red = parse_curve_points(text),
                "toneCurveGreen" => data.edits.curves.green = parse_curve_points(text),
                "toneCurveBlue" => data.edits.curves.blue = parse_curve_points(text),
                _ => {}
            }
        }
    }

    Ok(data)
}
