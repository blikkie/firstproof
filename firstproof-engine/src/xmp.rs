use crate::curve::CurvePointRecord;
use crate::modes::{
    ColorProfileMode, EmulsionArchetype, EmulsionFormat, EmulsionGrainMode, EnlargerFilterMode,
    PrintMediaMode,
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
        .element("emulsionGrainMode", pm_ns.clone())
        .value(match data.edits.emulsion_grain_mode {
            EmulsionGrainMode::Off => "Off",
            EmulsionGrainMode::SeedFromSensor => "SeedFromSensor",
            EmulsionGrainMode::SynthesizePhysical => "SynthesizePhysical",
        });
    writer
        .element("grainScale", pm_ns.clone())
        .value(format!("{:.4}", data.edits.grain_scale).as_str());
    writer
        .element("grainSize", pm_ns.clone())
        .value(format!("{:.4}", data.edits.grain_size).as_str());
    writer
        .element("grainAmount", pm_ns.clone())
        .value(format!("{:.4}", data.edits.grain_amount).as_str());

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
            EmulsionArchetype::LeverkusenPortrait160 => "LeverkusenPortrait160",
            EmulsionArchetype::AshigaraPortrait400 => "AshigaraPortrait400",
            EmulsionArchetype::LiguriaCinema80 => "LiguriaCinema80",
            EmulsionArchetype::BaodingSilver100 => "BaodingSilver100",
        });
    writer
        .element("emulsionFormat", pm_ns.clone())
        .value(match data.edits.emulsion_format {
            EmulsionFormat::Format35mm => "Format35mm",
            EmulsionFormat::Format120 => "Format120",
            EmulsionFormat::Format4x5 => "Format4x5",
            EmulsionFormat::Super8 => "Super8",
            EmulsionFormat::Format16mm => "Format16mm",
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
            PrintMediaMode::BamakoSilverContact => "BamakoSilverContact",
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
        .element("curveMaster", pm_ns.clone())
        .value(serialize_curve_points(&data.edits.curves.master).as_str());
    writer
        .element("curveRed", pm_ns.clone())
        .value(serialize_curve_points(&data.edits.curves.red).as_str());
    writer
        .element("curveGreen", pm_ns.clone())
        .value(serialize_curve_points(&data.edits.curves.green).as_str());
    writer
        .element("curveBlue", pm_ns)
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

fn find_desc_node<'a, 'd>(doc: &'a Document<'d>) -> Option<Node<'a, 'd>> {
    doc.descendants().find(|&node| node.is_element()
            && (node.has_tag_name(("http://www.w3.org/1999/02/22-rdf-syntax-ns#", "Description"))
                || node.tag_name().name() == "Description"))
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

pub fn parse_xmp_sidecar(xml: &str) -> Result<XmpData, String> {
    let doc = Document::parse(xml).map_err(|e| e.to_string())?;
    let desc_opt = find_desc_node(&doc);
    let desc = match desc_opt {
        Some(d) => d,
        None => doc.root_element(),
    };

    let mut data = XmpData::default();
    let get = |ns: &str, name: &str| -> Option<&str> {
        if ns == "https://firstproof.photo/ns/pm/1.0/" {
            get_prop(&desc, ns, name).or_else(|| get_prop(&desc, "http://ns.lumenbox.app/pm/1.0/", name))
        } else {
            get_prop(&desc, ns, name)
        }
    };

    // 1. Standard attributes
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

    // 2. First Proof PM adjustments
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
    if let Some(sh_hue_str) = get(pm, "splitToningShadowHue") {
        if let Ok(val) = sh_hue_str.parse::<f32>() {
            data.edits.split_toning_shadow_hue = val;
        }
    }
    if let Some(sh_sat_str) = get(pm, "splitToningShadowSat") {
        if let Ok(val) = sh_sat_str.parse::<f32>() {
            data.edits.split_toning_shadow_sat = val;
        }
    }
    if let Some(hl_hue_str) = get(pm, "splitToningHighlightHue") {
        if let Ok(val) = hl_hue_str.parse::<f32>() {
            data.edits.split_toning_highlight_hue = val;
        }
    }
    if let Some(hl_sat_str) = get(pm, "splitToningHighlightSat") {
        if let Ok(val) = hl_sat_str.parse::<f32>() {
            data.edits.split_toning_highlight_sat = val;
        }
    }
    if let Some(bal_str) = get(pm, "splitToningBalance") {
        if let Ok(val) = bal_str.parse::<f32>() {
            data.edits.split_toning_balance = val;
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
    if let Some(gm_str) = get(pm, "emulsionGrainMode") {
        data.edits.emulsion_grain_mode = match gm_str.trim() {
            "SeedFromSensor" => EmulsionGrainMode::SeedFromSensor,
            "SynthesizePhysical" => EmulsionGrainMode::SynthesizePhysical,
            _ => EmulsionGrainMode::Off,
        };
    }
    if let Some(gs_str) = get(pm, "grainScale") {
        if let Ok(val) = gs_str.parse::<f32>() {
            data.edits.grain_scale = val;
            data.edits.grain_size = val;
        }
    }
    if let Some(gs_str) = get(pm, "grainSize") {
        if let Ok(val) = gs_str.parse::<f32>() {
            data.edits.grain_size = val;
        }
    }
    if let Some(ga_str) = get(pm, "grainAmount") {
        if let Ok(val) = ga_str.parse::<f32>() {
            data.edits.grain_amount = val;
        }
    }
    if let Some(ea_str) = get(pm, "emulsionArchetype") {
        data.edits.emulsion_archetype = match ea_str.trim() {
            "RochesterPortrait400" => EmulsionArchetype::RochesterPortrait400,
            "AshigaraChrome50" => EmulsionArchetype::AshigaraChrome50,
            "RochesterK14Chrome64" => EmulsionArchetype::RochesterK14Chrome64,
            "RochesterCinema500" => EmulsionArchetype::RochesterCinema500,
            // RochesterDaylight200 merged into RochesterPortrait400
            "RochesterDaylight200" => EmulsionArchetype::RochesterPortrait400,
            "MobberleySilver400" => EmulsionArchetype::MobberleySilver400,
            "CambridgeDiffusion100" => EmulsionArchetype::CambridgeDiffusion100,
            "LeverkusenPortrait160" => EmulsionArchetype::LeverkusenPortrait160,
            "AshigaraPortrait400" => EmulsionArchetype::AshigaraPortrait400,
            "LiguriaCinema80" => EmulsionArchetype::LiguriaCinema80,
            "BaodingSilver100" => EmulsionArchetype::BaodingSilver100,
            _ => EmulsionArchetype::None,
        };
    }
    if let Some(ef_str) = get(pm, "emulsionFormat") {
        data.edits.emulsion_format = match ef_str.trim() {
            "Format120" => EmulsionFormat::Format120,
            "Format4x5" => EmulsionFormat::Format4x5,
            "Super8" => EmulsionFormat::Super8,
            "Format16mm" => EmulsionFormat::Format16mm,
            _ => EmulsionFormat::Format35mm,
        };
    }
    if let Some(ee_str) = get(pm, "emulsionExposure") {
        if let Ok(val) = ee_str.parse::<f32>() {
            data.edits.emulsion_exposure = val;
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
            "AgxVivid" => ColorProfileMode::AgxVivid,
            "AgxFilmic" => ColorProfileMode::AgxFilmic,
            "CameraStandard" => ColorProfileMode::CameraStandard,
            "CameraPortrait" => ColorProfileMode::CameraPortrait,
            "CameraLandscape" => ColorProfileMode::CameraLandscape,
            "CameraVivid" => ColorProfileMode::CameraVivid,
            _ => ColorProfileMode::AgxVivid,
        };
    }
    if let Some(pm_str) = get(pm, "printMedia") {
        data.edits.print_media = match pm_str {
            "ChromogenicTypeC" => PrintMediaMode::ChromogenicTypeC,
            "SilverDyeBleach" => PrintMediaMode::SilverDyeBleach,
            "FineArtCottonRag" => PrintMediaMode::FineArtCottonRag,
            "BarytaSilverGelatin" => PrintMediaMode::BarytaSilverGelatin,
            "WarmtoneChlorobromide" => PrintMediaMode::WarmtoneChlorobromide,
            "PlatinumPalladium" => PrintMediaMode::PlatinumPalladium,
            "Cyanotype" => PrintMediaMode::Cyanotype,
            "BamakoSilverContact" => PrintMediaMode::BamakoSilverContact,
            _ => PrintMediaMode::None,
        };
    }
    if let Some(ef_str) = get(pm, "enlargerFilter") {
        data.edits.enlarger_filter = match ef_str {
            "YellowK2" => EnlargerFilterMode::YellowK2,
            "Orange16" => EnlargerFilterMode::Orange16,
            "Red25A" => EnlargerFilterMode::Red25A,
            "GreenX1" => EnlargerFilterMode::GreenX1,
            "Orthochromatic" => EnlargerFilterMode::Orthochromatic,
            _ => EnlargerFilterMode::NeutralPanchromatic,
        };
    }
    if let Some(ppg_str) = get(pm, "printPaperGrade") {
        if let Ok(val) = ppg_str.parse::<f32>() {
            data.edits.print_paper_grade = val;
        }
    }
    if let Some(pbw_str) = get(pm, "printBaseWarmth") {
        if let Ok(val) = pbw_str.parse::<f32>() {
            data.edits.print_base_warmth = val;
        }
    }
    if let Some(ppd_str) = get(pm, "printProcessDepth") {
        if let Ok(val) = ppd_str.parse::<f32>() {
            data.edits.print_process_depth = val;
        }
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

    if let Some(pts_str) = get(pm, "curveMaster").or_else(|| get(pm, "toneCurveMaster")) {
        let pts = parse_curve_points(pts_str);
        if !pts.is_empty() {
            data.edits.curves.master = pts;
        }
    }
    if let Some(pts_str) = get(pm, "curveRed").or_else(|| get(pm, "toneCurveRed")) {
        let pts = parse_curve_points(pts_str);
        if !pts.is_empty() {
            data.edits.curves.red = pts;
        }
    }
    if let Some(pts_str) = get(pm, "curveGreen").or_else(|| get(pm, "toneCurveGreen")) {
        let pts = parse_curve_points(pts_str);
        if !pts.is_empty() {
            data.edits.curves.green = pts;
        }
    }
    if let Some(pts_str) = get(pm, "curveBlue").or_else(|| get(pm, "toneCurveBlue")) {
        let pts = parse_curve_points(pts_str);
        if !pts.is_empty() {
            data.edits.curves.blue = pts;
        }
    }

    // 3. Fallback: Parse Adobe Camera Raw settings (crs:) if native ones are missing
    let crs_ns = "http://ns.adobe.com/camera-raw-settings/1.0/";

    if data.edits.exposure == 0.0 {
        if let Some(exp_str) = desc.attribute((crs_ns, "Exposure2012")).or_else(|| desc.attribute("Exposure2012")) {
            if let Ok(val) = exp_str.parse::<f32>() {
                data.edits.exposure = val;
            }
        }
    }
    if data.edits.contrast == 0.0 {
        if let Some(con_str) = desc.attribute((crs_ns, "Contrast2012")).or_else(|| desc.attribute("Contrast2012")) {
            if let Ok(val) = con_str.parse::<f32>() {
                data.edits.contrast = val;
            }
        }
    }
    if data.edits.highlights == 0.0 {
        if let Some(hl_str) = desc
            .attribute((crs_ns, "Highlights2012"))
            .or_else(|| desc.attribute((crs_ns, "Highlights")))
            .or_else(|| desc.attribute("Highlights2012"))
            .or_else(|| desc.attribute("Highlights"))
        {
            if let Ok(val) = hl_str.parse::<f32>() {
                data.edits.highlights = val;
            }
        }
    }
    if data.edits.shadows == 0.0 {
        if let Some(sh_str) = desc
            .attribute((crs_ns, "Shadows2012"))
            .or_else(|| desc.attribute((crs_ns, "Shadows")))
            .or_else(|| desc.attribute("Shadows2012"))
            .or_else(|| desc.attribute("Shadows"))
        {
            if let Ok(val) = sh_str.parse::<f32>() {
                data.edits.shadows = val;
            }
        }
    }
    if data.edits.whites == 0.0 {
        if let Some(wh_str) = desc
            .attribute((crs_ns, "Whites2012"))
            .or_else(|| desc.attribute((crs_ns, "Whites")))
            .or_else(|| desc.attribute("Whites2012"))
            .or_else(|| desc.attribute("Whites"))
        {
            if let Ok(val) = wh_str.parse::<f32>() {
                data.edits.whites = val;
            }
        }
    }
    if data.edits.blacks == 0.0 {
        if let Some(bl_str) = desc
            .attribute((crs_ns, "Blacks2012"))
            .or_else(|| desc.attribute((crs_ns, "Blacks")))
            .or_else(|| desc.attribute("Blacks2012"))
            .or_else(|| desc.attribute("Blacks"))
        {
            if let Ok(val) = bl_str.parse::<f32>() {
                data.edits.blacks = val;
            }
        }
    }
    if data.edits.vibrance == 0.0 {
        if let Some(vib_str) = desc.attribute((crs_ns, "Vibrance")).or_else(|| desc.attribute("Vibrance")) {
            if let Ok(val) = vib_str.parse::<f32>() {
                data.edits.vibrance = val;
            }
        }
    }
    if data.edits.saturation == 0.0 {
        if let Some(sat_str) = desc.attribute((crs_ns, "Saturation")).or_else(|| desc.attribute("Saturation")) {
            if let Ok(val) = sat_str.parse::<f32>() {
                data.edits.saturation = val;
            }
        }
    }
    if data.edits.split_toning_shadow_sat == 0.0 && data.edits.split_toning_highlight_sat == 0.0 {
        if let Some(sh_hue_str) = desc.attribute((crs_ns, "SplitToningShadowHue")).or_else(|| desc.attribute("SplitToningShadowHue")) {
            if let Ok(val) = sh_hue_str.parse::<f32>() {
                data.edits.split_toning_shadow_hue = val;
            }
        }
        if let Some(sh_sat_str) = desc.attribute((crs_ns, "SplitToningShadowSaturation")).or_else(|| desc.attribute("SplitToningShadowSaturation")) {
            if let Ok(val) = sh_sat_str.parse::<f32>() {
                data.edits.split_toning_shadow_sat = val;
            }
        }
        if let Some(hl_hue_str) = desc.attribute((crs_ns, "SplitToningHighlightHue")).or_else(|| desc.attribute("SplitToningHighlightHue")) {
            if let Ok(val) = hl_hue_str.parse::<f32>() {
                data.edits.split_toning_highlight_hue = val;
            }
        }
        if let Some(hl_sat_str) = desc.attribute((crs_ns, "SplitToningHighlightSaturation")).or_else(|| desc.attribute("SplitToningHighlightSaturation")) {
            if let Ok(val) = hl_sat_str.parse::<f32>() {
                data.edits.split_toning_highlight_sat = val;
            }
        }
        if let Some(bal_str) = desc.attribute((crs_ns, "SplitToningBalance")).or_else(|| desc.attribute("SplitToningBalance")) {
            if let Ok(val) = bal_str.parse::<f32>() {
                data.edits.split_toning_balance = val;
            }
        }
    }
    if data.edits.temperature == 5000.0 {
        if let Some(temp_str) = desc.attribute((crs_ns, "Temperature")).or_else(|| desc.attribute("Temperature")) {
            if let Ok(val) = temp_str.parse::<f32>() {
                data.edits.temperature = val;
            }
        }
    }
    if data.edits.tint == 0.0 {
        if let Some(tint_str) = desc.attribute((crs_ns, "Tint")).or_else(|| desc.attribute("Tint")) {
            if let Ok(val) = tint_str.parse::<f32>() {
                data.edits.tint = val;
            }
        }
    }
    if data.edits.sharpening_amount == 0.0 {
        if let Some(s_str) = desc
            .attribute((crs_ns, "SharpenAmount"))
            .or_else(|| desc.attribute((crs_ns, "Sharpness")))
            .or_else(|| desc.attribute("SharpenAmount"))
            .or_else(|| desc.attribute("Sharpness"))
        {
            if let Ok(val) = s_str.parse::<f32>() {
                data.edits.sharpening_amount = val;
            }
        }
    }
    if data.edits.sharpening_radius == 1.0 {
        if let Some(r_str) = desc.attribute((crs_ns, "SharpenRadius")).or_else(|| desc.attribute("SharpenRadius")) {
            if let Ok(val) = r_str.parse::<f32>() {
                data.edits.sharpening_radius = val;
            }
        }
    }
    if data.edits.sharpening_masking == 0.0 {
        if let Some(m_str) = desc
            .attribute((crs_ns, "SharpenEdgeMasking"))
            .or_else(|| desc.attribute((crs_ns, "Masking")))
            .or_else(|| desc.attribute("SharpenEdgeMasking"))
            .or_else(|| desc.attribute("Masking"))
        {
            if let Ok(val) = m_str.parse::<f32>() {
                data.edits.sharpening_masking = val;
            }
        }
    }
    if data.edits.noise_reduction == 0.0 {
        if let Some(nr_str) = desc.attribute((crs_ns, "LuminanceSmoothing")).or_else(|| desc.attribute("LuminanceSmoothing")) {
            if let Ok(val) = nr_str.parse::<f32>() {
                data.edits.noise_reduction = val;
            }
        }
    }

    // Check crop details in Adobe
    if data.edits.crop_left == 0.0 && data.edits.crop_right == 1.0 {
        let get_attr = |name: &str| desc.attribute((crs_ns, name)).or_else(|| desc.attribute(name));
        if let (Some(l), Some(t), Some(w), Some(h)) = (
            get_attr("CropLeft"),
            get_attr("CropTop"),
            get_attr("CropWidth"),
            get_attr("CropHeight"),
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
        if let Some(ca_str) = desc.attribute((crs_ns, "CropAngle")).or_else(|| desc.attribute("CropAngle")) {
            if let Ok(val) = ca_str.parse::<f32>() {
                data.edits.crop_angle = val;
            }
        }
    }

    if data.edits.perspective_vertical == 0.0 {
        if let Some(pv_str) = desc.attribute((crs_ns, "PerspectiveVertical")).or_else(|| desc.attribute("PerspectiveVertical")) {
            if let Ok(val) = pv_str.parse::<f32>() {
                data.edits.perspective_vertical = val;
            }
        }
    }
    if data.edits.perspective_horizontal == 0.0 {
        if let Some(ph_str) = desc.attribute((crs_ns, "PerspectiveHorizontal")).or_else(|| desc.attribute("PerspectiveHorizontal")) {
            if let Ok(val) = ph_str.parse::<f32>() {
                data.edits.perspective_horizontal = val;
            }
        }
    }
    if let Some(cc_str) = desc.attribute((crs_ns, "CropConstrainToWarp")).or_else(|| desc.attribute("CropConstrainToWarp")) {
        data.edits.crop_constrain = cc_str == "1" || cc_str == "true";
    }

    // Fallback: Parse Orientation hints from third-party editors (Lightroom, Capture One, Darktable, Digikam)
    if data.edits.rotation == 0 && !data.edits.flip_horizontal && !data.edits.flip_vertical {
        let tiff_ns = "http://ns.adobe.com/tiff/1.0/";
        let exif_ns = "http://ns.adobe.com/exif/1.0/";

        let orient_hint = desc
            .attribute((tiff_ns, "Orientation"))
            .or_else(|| desc.attribute((exif_ns, "Orientation")))
            .or_else(|| desc.attribute((crs_ns, "Orientation")))
            .or_else(|| desc.attribute("Orientation"));

        if let Some(orient_str) = orient_hint {
            if let Ok(num) = orient_str.trim().parse::<u32>() {
                if (1..=8).contains(&num) {
                    let rec = crate::preview::decode_exif_orientation(num);
                    data.edits.rotation = rec.rotation;
                    data.edits.flip_horizontal = rec.flip_horizontal;
                    data.edits.flip_vertical = rec.flip_vertical;
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

/// Fallback wrapper returning std::io::Result for compatibility with firstproof-core I/O.
pub fn parse_xmp(xml: &str) -> std::io::Result<XmpData> {
    parse_xmp_sidecar(xml).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

pub use generate_xmp_sidecar as generate_xmp;
