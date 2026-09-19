use crate::curve::CurvePointRecord;
use roxmltree::{Document, Node};
use serde::{Deserialize, Serialize};
use xmp_writer::{CustomNamespace, Namespace, XmpWriter};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WebEditSettings {
    pub exposure: f32,
    pub contrast: f32,
    pub highlights: f32,
    pub shadows: f32,
    pub whites: f32,
    pub blacks: f32,
    pub vibrance: f32,
    pub saturation: f32,
    pub temperature: f32,
    pub tint: f32,
    pub crop_left: f32,
    pub crop_top: f32,
    pub crop_right: f32,
    pub crop_bottom: f32,
    pub crop_angle: f32,
    pub rotation: i32,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
    pub curve_master: Vec<CurvePointRecord>,
    pub curve_red: Vec<CurvePointRecord>,
    pub curve_green: Vec<CurvePointRecord>,
    pub curve_blue: Vec<CurvePointRecord>,
    pub emulsion_archetype: String,
    pub emulsion_format: String,
    pub emulsion_exposure: f32,
}

impl Default for WebEditSettings {
    fn default() -> Self {
        Self {
            exposure: 0.0,
            contrast: 0.0,
            highlights: 0.0,
            shadows: 0.0,
            whites: 0.0,
            blacks: 0.0,
            vibrance: 0.0,
            saturation: 0.0,
            temperature: 5500.0,
            tint: 0.0,
            crop_left: 0.0,
            crop_top: 0.0,
            crop_right: 1.0,
            crop_bottom: 1.0,
            crop_angle: 0.0,
            rotation: 0,
            flip_horizontal: false,
            flip_vertical: false,
            curve_master: vec![
                CurvePointRecord { x: 0.0, y: 0.0 },
                CurvePointRecord { x: 1.0, y: 1.0 },
            ],
            curve_red: vec![
                CurvePointRecord { x: 0.0, y: 0.0 },
                CurvePointRecord { x: 1.0, y: 1.0 },
            ],
            curve_green: vec![
                CurvePointRecord { x: 0.0, y: 0.0 },
                CurvePointRecord { x: 1.0, y: 1.0 },
            ],
            curve_blue: vec![
                CurvePointRecord { x: 0.0, y: 0.0 },
                CurvePointRecord { x: 1.0, y: 1.0 },
            ],
            emulsion_archetype: "None".to_string(),
            emulsion_format: "Format35mm".to_string(),
            emulsion_exposure: 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WebXmpData {
    pub rating: Option<i32>,
    pub status: String, // "Pick", "Reject", "None"
    pub title: Option<String>,
    pub description: Option<String>,
    pub creator: Option<String>,
    pub copyright: Option<String>,
    pub keywords: Vec<String>,
    pub edits: WebEditSettings,
}

impl Default for WebXmpData {
    fn default() -> Self {
        Self {
            rating: None,
            status: "None".to_string(),
            title: None,
            description: None,
            creator: None,
            copyright: None,
            keywords: Vec::new(),
            edits: WebEditSettings::default(),
        }
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

pub fn generate_xmp_sidecar(data: &WebXmpData) -> String {
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
        .element("emulsionArchetype", pm_ns.clone())
        .value(data.edits.emulsion_archetype.as_str());
    writer
        .element("emulsionFormat", pm_ns.clone())
        .value(data.edits.emulsion_format.as_str());
    writer
        .element("emulsionExposure", pm_ns.clone())
        .value(format!("{:.4}", data.edits.emulsion_exposure).as_str());

    writer
        .element("toneCurveMaster", pm_ns.clone())
        .value(serialize_curve_points(&data.edits.curve_master).as_str());
    writer
        .element("toneCurveRed", pm_ns.clone())
        .value(serialize_curve_points(&data.edits.curve_red).as_str());
    writer
        .element("toneCurveGreen", pm_ns.clone())
        .value(serialize_curve_points(&data.edits.curve_green).as_str());
    writer
        .element("toneCurveBlue", pm_ns)
        .value(serialize_curve_points(&data.edits.curve_blue).as_str());

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

pub fn parse_xmp_sidecar(xml: &str) -> Result<WebXmpData, String> {
    let doc = Document::parse(xml).map_err(|e| e.to_string())?;
    let mut data = WebXmpData::default();

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
                "rotation" => {
                    if let Ok(v) = text.parse::<i32>() {
                        data.edits.rotation = v;
                    }
                }
                "flipHorizontal" => data.edits.flip_horizontal = text == "true",
                "flipVertical" => data.edits.flip_vertical = text == "true",
                "emulsionArchetype" => data.edits.emulsion_archetype = text.to_string(),
                "emulsionFormat" => data.edits.emulsion_format = text.to_string(),
                "emulsionExposure" => {
                    if let Ok(v) = text.parse::<f32>() {
                        data.edits.emulsion_exposure = v;
                    }
                }
                "toneCurveMaster" => data.edits.curve_master = parse_curve_points(text),
                "toneCurveRed" => data.edits.curve_red = parse_curve_points(text),
                "toneCurveGreen" => data.edits.curve_green = parse_curve_points(text),
                "toneCurveBlue" => data.edits.curve_blue = parse_curve_points(text),
                _ => {}
            }
        }
    }

    Ok(data)
}
