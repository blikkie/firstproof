//! Canonical Command Schema & Dispatcher for First Proof
//!
//! Provides a strongly typed, cross-platform command bus for darkroom automation,
//! headless studio scripting, web playground orchestration, and MCP bridge tools.
//!
//! Conforms to First Proof anti-slop principles:
//! "Pure capture. Honest color. Zero digital plastic."

use crate::curve::ToneCurvesRecord;
use crate::modes::{
    EmulsionArchetype, EmulsionFormat, EmulsionGrainMode, EnlargerFilterMode, PrintMediaMode,
    RemjetBackingMode,
};
use crate::schema::WorkspaceCategory;
use crate::settings::EditSettings;
use serde::{Deserialize, Serialize};

/// Non-destructive delta adjustments applied to an [`EditSettings`] state.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct EditDelta {
    pub exposure: Option<f32>,
    pub contrast: Option<f32>,
    pub highlights: Option<f32>,
    pub shadows: Option<f32>,
    pub whites: Option<f32>,
    pub blacks: Option<f32>,
    pub vibrance: Option<f32>,
    pub saturation: Option<f32>,
    pub deep_blue: Option<f32>,
    pub foliage: Option<f32>,
    pub temperature: Option<f32>,
    pub tint: Option<f32>,
    pub rotation: Option<i32>,
    pub crop: Option<[f32; 4]>,
    pub crop_left: Option<f32>,
    pub crop_top: Option<f32>,
    pub crop_right: Option<f32>,
    pub crop_bottom: Option<f32>,
    pub crop_angle: Option<f32>,
    pub perspective_vertical: Option<f32>,
    pub perspective_horizontal: Option<f32>,
    pub crop_constrain: Option<bool>,
    pub flip_horizontal: Option<bool>,
    pub flip_vertical: Option<bool>,
    pub lut_name: Option<Option<String>>,
    pub lut_intensity: Option<f32>,
    pub sharpening_amount: Option<f32>,
    pub sharpening_radius: Option<f32>,
    pub sharpening_masking: Option<f32>,
    pub noise_reduction: Option<f32>,
    pub emulsion_grain_mode: Option<EmulsionGrainMode>,
    pub grain_scale: Option<f32>,
    pub grain_size: Option<f32>,
    pub grain_amount: Option<f32>,
    pub emulsion_archetype: Option<EmulsionArchetype>,
    pub emulsion_format: Option<EmulsionFormat>,
    pub emulsion_exposure: Option<f32>,
    pub remjet_backing: Option<RemjetBackingMode>,
    pub curves: Option<ToneCurvesRecord>,
    pub split_toning_shadow_hue: Option<f32>,
    pub split_toning_shadow_sat: Option<f32>,
    pub split_toning_highlight_hue: Option<f32>,
    pub split_toning_highlight_sat: Option<f32>,
    pub split_toning_balance: Option<f32>,
    pub print_media: Option<PrintMediaMode>,
    pub enlarger_filter: Option<EnlargerFilterMode>,
    pub print_paper_grade: Option<f32>,
    pub print_base_warmth: Option<f32>,
    pub print_process_depth: Option<f32>,
    pub defringe_amount: Option<f32>,
    pub lens_correction_enabled: Option<bool>,
    pub tca_correction_enabled: Option<bool>,
    pub vignetting_correction_enabled: Option<bool>,
}

impl EditDelta {
    /// Applies all defined adjustments in this delta onto an existing [`EditSettings`] record.
    pub fn apply_to(&self, settings: &mut EditSettings) {
        if let Some(v) = self.exposure { settings.exposure = v; }
        if let Some(v) = self.contrast { settings.contrast = v; }
        if let Some(v) = self.highlights { settings.highlights = v; }
        if let Some(v) = self.shadows { settings.shadows = v; }
        if let Some(v) = self.whites { settings.whites = v; }
        if let Some(v) = self.blacks { settings.blacks = v; }
        if let Some(v) = self.vibrance { settings.vibrance = v; }
        if let Some(v) = self.saturation { settings.saturation = v; }
        if let Some(v) = self.deep_blue { settings.deep_blue = v; }
        if let Some(v) = self.foliage { settings.foliage = v; }
        if let Some(v) = self.temperature { settings.temperature = v; }
        if let Some(v) = self.tint { settings.tint = v; }
        if let Some(v) = self.rotation { settings.rotation = v; }
        if let Some(c) = self.crop {
            settings.crop_left = c[0];
            settings.crop_top = c[1];
            settings.crop_right = c[2];
            settings.crop_bottom = c[3];
        }
        if let Some(v) = self.crop_left { settings.crop_left = v; }
        if let Some(v) = self.crop_top { settings.crop_top = v; }
        if let Some(v) = self.crop_right { settings.crop_right = v; }
        if let Some(v) = self.crop_bottom { settings.crop_bottom = v; }
        if let Some(v) = self.crop_angle { settings.crop_angle = v; }
        if let Some(v) = self.perspective_vertical { settings.perspective_vertical = v; }
        if let Some(v) = self.perspective_horizontal { settings.perspective_horizontal = v; }
        if let Some(v) = self.crop_constrain { settings.crop_constrain = v; }
        if let Some(v) = self.flip_horizontal { settings.flip_horizontal = v; }
        if let Some(v) = self.flip_vertical { settings.flip_vertical = v; }
        if let Some(ref v) = self.lut_name { settings.lut_name = v.clone(); }
        if let Some(v) = self.lut_intensity { settings.lut_intensity = v; }
        if let Some(v) = self.sharpening_amount { settings.sharpening_amount = v; }
        if let Some(v) = self.sharpening_radius { settings.sharpening_radius = v; }
        if let Some(v) = self.sharpening_masking { settings.sharpening_masking = v; }
        if let Some(v) = self.noise_reduction { settings.noise_reduction = v; }
        if let Some(v) = self.emulsion_grain_mode { settings.emulsion_grain_mode = v; }
        if let Some(v) = self.grain_scale { settings.grain_scale = v; }
        if let Some(v) = self.grain_size { settings.grain_size = v; }
        if let Some(v) = self.grain_amount { settings.grain_amount = v; }
        if let Some(v) = self.emulsion_archetype { settings.emulsion_archetype = v; }
        if let Some(v) = self.emulsion_format { settings.emulsion_format = v; }
        if let Some(v) = self.emulsion_exposure { settings.emulsion_exposure = v; }
        if let Some(v) = self.remjet_backing { settings.remjet_backing = v; }
        if let Some(ref v) = self.curves { settings.curves = v.clone(); }
        if let Some(v) = self.split_toning_shadow_hue { settings.split_toning_shadow_hue = v; }
        if let Some(v) = self.split_toning_shadow_sat { settings.split_toning_shadow_sat = v; }
        if let Some(v) = self.split_toning_highlight_hue { settings.split_toning_highlight_hue = v; }
        if let Some(v) = self.split_toning_highlight_sat { settings.split_toning_highlight_sat = v; }
        if let Some(v) = self.split_toning_balance { settings.split_toning_balance = v; }
        if let Some(v) = self.print_media { settings.print_media = v; }
        if let Some(v) = self.enlarger_filter { settings.enlarger_filter = v; }
        if let Some(v) = self.print_paper_grade { settings.print_paper_grade = v; }
        if let Some(v) = self.print_base_warmth { settings.print_base_warmth = v; }
        if let Some(v) = self.print_process_depth { settings.print_process_depth = v; }
        if let Some(v) = self.defringe_amount { settings.defringe_amount = v; }
        if let Some(v) = self.lens_correction_enabled { settings.lens_correction_enabled = v; }
        if let Some(v) = self.tca_correction_enabled { settings.tca_correction_enabled = v; }
        if let Some(v) = self.vignetting_correction_enabled { settings.vignetting_correction_enabled = v; }
    }
}

/// Canonical, strongly typed darkroom command enum.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum DarkroomCommand {
    /// Retrieve current darkroom state (selected photo, active edits, canvas mode, lens calibration).
    GetState,
    /// Sample pixel RGB color values at normalized [u, v] coordinates in [0.0 .. 1.0].
    SamplePixel { points: Vec<[f64; 2]> },
    /// Inspect metadata (EXIF, CFA layout, camera matrices, XMP sidecar) for a photo.
    InspectMetadata {
        #[serde(default)]
        path: Option<String>,
    },
    /// Switch selected photo in filmstrip / detail view.
    SelectPhoto { path: String },
    /// Switch darkroom canvas mode ("grid", "detail", "compare").
    SetCanvasMode { mode: String },
    /// Set absolute exposure in EV (-5.0 .. +5.0).
    SetExposure { ev: f32 },
    /// Set white balance temperature in Kelvin (2000 .. 20000).
    SetTemperature { kelvin: f64 },
    /// Set white balance tint (-100.0 .. +100.0).
    SetTint { tint: f32 },
    /// Set curated fine-art film emulsion archetype.
    SetEmulsion {
        archetype: EmulsionArchetype,
        #[serde(default)]
        format: Option<EmulsionFormat>,
        #[serde(default)]
        exposure: Option<f32>,
    },
    /// Set physical print media archetype and paper grading.
    SetPrintMedia {
        media: PrintMediaMode,
        #[serde(default)]
        paper_grade: Option<f32>,
        #[serde(default)]
        base_warmth: Option<f32>,
        #[serde(default)]
        process_depth: Option<f32>,
    },
    /// Apply non-destructive delta adjustments to current photo edits.
    ApplyEditDelta { delta: EditDelta },
    /// Reset active edits (either an individual workspace category or entire photo).
    ResetWorkspace {
        #[serde(default)]
        category: Option<WorkspaceCategory>,
    },
    /// Capture current viewport canvas rendering to an image file.
    CaptureCanvas { output_path: String },
    /// Develop a RAW or image to 16-bit TIFF or JPEG with specified or active edits.
    DevelopPhoto {
        input: String,
        #[serde(default)]
        output: Option<String>,
        #[serde(default)]
        settings: Option<EditSettings>,
        #[serde(default)]
        fast: Option<bool>,
    },
    /// Convert proprietary camera RAW (.CR2, .NEF, .ARW) to archival DNG with validation.
    ConvertToDng {
        input: String,
        #[serde(default)]
        output: Option<String>,
    },
    /// Perform automated dual-engine mathematical parity audit on a photo.
    AuditParity {
        #[serde(default)]
        photo_path: Option<String>,
    },
}

/// Parses an emulsion archetype name from flexible kebab-case, snake_case, or lowercase string.
pub fn parse_emulsion_archetype_str(s: &str) -> Option<EmulsionArchetype> {
    match s.to_lowercase().replace(['-', '_'], "").as_str() {
        "none" | "bypass" | "off" => Some(EmulsionArchetype::None),
        "rochesterportrait400" | "portra400" | "portra" => Some(EmulsionArchetype::RochesterPortrait400),
        "ashigarachrome50" | "velvia50" | "velvia" => Some(EmulsionArchetype::AshigaraChrome50),
        "rochesterk14chrome64" | "kodachrome64" | "kodachrome" => Some(EmulsionArchetype::RochesterK14Chrome64),
        "rochestercinema500" | "vision3" | "500t" => Some(EmulsionArchetype::RochesterCinema500),
        "rochesterdaylight200" | "gold200" => Some(EmulsionArchetype::RochesterDaylight200),
        "mobberleysilver400" | "hp5" | "hp5plus" => Some(EmulsionArchetype::MobberleySilver400),
        "cambridgediffusion100" | "polaroid" | "sx70" => Some(EmulsionArchetype::CambridgeDiffusion100),
        "leverkusenportrait160" | "agfacolor" => Some(EmulsionArchetype::LeverkusenPortrait160),
        "ashigaraportrait400" | "pro400h" => Some(EmulsionArchetype::AshigaraPortrait400),
        "liguriacinema80" | "ferrania" | "p30" => Some(EmulsionArchetype::LiguriaCinema80),
        "baodingsilver100" | "lucky" | "shd100" => Some(EmulsionArchetype::BaodingSilver100),
        _ => None,
    }
}

/// Parses an emulsion format gate from flexible string representation.
pub fn parse_emulsion_format_str(s: &str) -> Option<EmulsionFormat> {
    match s.to_lowercase().replace(['-', '_'], "").as_str() {
        "35mm" | "format35mm" | "standard" => Some(EmulsionFormat::Format35mm),
        "120" | "format120" | "medium" | "6x7" => Some(EmulsionFormat::Format120),
        "4x5" | "format4x5" | "large" | "sheet" => Some(EmulsionFormat::Format4x5),
        "super8" | "s8" => Some(EmulsionFormat::Super8),
        "16mm" | "format16mm" => Some(EmulsionFormat::Format16mm),
        _ => None,
    }
}

/// Parses a print media archetype from flexible string representation.
pub fn parse_print_media_str(s: &str) -> Option<PrintMediaMode> {
    match s.to_lowercase().replace(['-', '_'], "").as_str() {
        "none" | "off" => Some(PrintMediaMode::None),
        "chromogenic" | "typec" | "ra4" | "chromogenictypec" => Some(PrintMediaMode::ChromogenicTypeC),
        "silverdyebleach" | "ilfochrome" | "azo" => Some(PrintMediaMode::SilverDyeBleach),
        "baryta" | "barytasilvergelatin" => Some(PrintMediaMode::BarytaSilverGelatin),
        "warmtone" | "warmtonechlorobromide" => Some(PrintMediaMode::WarmtoneChlorobromide),
        "platinum" | "platinumpalladium" => Some(PrintMediaMode::PlatinumPalladium),
        "cyanotype" => Some(PrintMediaMode::Cyanotype),
        "bamako" | "bamakosilvercontact" => Some(PrintMediaMode::BamakoSilverContact),
        _ => None,
    }
}

impl DarkroomCommand {
    /// Parses a JSON string into a [`DarkroomCommand`], transparently accepting both
    /// the canonical tagged format (`{"command": "get_state"}`) and JSON-RPC
    /// format (`{"method": "get_state", "params": {...}}`).
    pub fn from_json(json_str: &str) -> Result<Self, String> {
        let mut val: serde_json::Value =
            serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON: {}", e))?;

        // 1. Direct deserialization if "command" key is present (with string normalization)
        if let Some(cmd_name) = val.get("command").and_then(|v| v.as_str()) {
            if cmd_name == "set_emulsion" {
                if let Some(arch_str) = val.get("archetype").and_then(|v| v.as_str()) {
                    if let Some(arch) = parse_emulsion_archetype_str(arch_str) {
                        val["archetype"] = serde_json::to_value(arch).unwrap();
                    }
                }
                if let Some(fmt_str) = val.get("format").and_then(|v| v.as_str()) {
                    if let Some(fmt) = parse_emulsion_format_str(fmt_str) {
                        val["format"] = serde_json::to_value(fmt).unwrap();
                    }
                }
            } else if cmd_name == "set_print_media" {
                if let Some(media_str) = val.get("media").and_then(|v| v.as_str()) {
                    if let Some(media) = parse_print_media_str(media_str) {
                        val["media"] = serde_json::to_value(media).unwrap();
                    }
                }
            } else if cmd_name == "apply_edit_delta" {
                if let Some(delta_val) = val.get_mut("delta").and_then(|v| v.as_object_mut()) {
                    if let Some(arch_str) = delta_val.get("emulsion_archetype").and_then(|v| v.as_str()) {
                        if let Some(arch) = parse_emulsion_archetype_str(arch_str) {
                            delta_val.insert("emulsion_archetype".to_string(), serde_json::to_value(arch).unwrap());
                        }
                    }
                    if let Some(fmt_str) = delta_val.get("emulsion_format").and_then(|v| v.as_str()) {
                        if let Some(fmt) = parse_emulsion_format_str(fmt_str) {
                            delta_val.insert("emulsion_format".to_string(), serde_json::to_value(fmt).unwrap());
                        }
                    }
                    if let Some(media_str) = delta_val.get("print_media").and_then(|v| v.as_str()) {
                        if let Some(media) = parse_print_media_str(media_str) {
                            delta_val.insert("print_media".to_string(), serde_json::to_value(media).unwrap());
                        }
                    }
                }
            }
            return serde_json::from_value(val).map_err(|e| format!("Command decode error: {}", e));
        }

        // 2. JSON-RPC format parsing ("method" and optional "params")
        if let Some(method_val) = val.get("method").and_then(|v| v.as_str()) {
            let empty_params = serde_json::Value::Object(serde_json::Map::new());
            let params = val.get("params").unwrap_or(&empty_params);

            return match method_val {
                "get_state" => Ok(DarkroomCommand::GetState),
                "open_detail_view" | "select_photo" => {
                    let path = params
                        .get("photo_path")
                        .or_else(|| params.get("path"))
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| "Missing 'photo_path' or 'path' param".to_string())?;
                    Ok(DarkroomCommand::SelectPhoto {
                        path: path.to_string(),
                    })
                }
                "set_canvas_mode" => {
                    let mode = params
                        .get("mode")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| "Missing 'mode' param".to_string())?;
                    Ok(DarkroomCommand::SetCanvasMode {
                        mode: mode.to_string(),
                    })
                }
                "sample_viewport" | "sample_pixel" => {
                    let pts = params
                        .get("points")
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or_default();
                    Ok(DarkroomCommand::SamplePixel { points: pts })
                }
                "capture_canvas" => {
                    let out = params
                        .get("output_path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("/tmp/firstproof_viewport.png")
                        .to_string();
                    Ok(DarkroomCommand::CaptureCanvas { output_path: out })
                }
                "inspect_metadata" => {
                    let path = params
                        .get("path")
                        .or_else(|| params.get("photo_path"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    Ok(DarkroomCommand::InspectMetadata { path })
                }
                "set_exposure" => {
                    let ev = params
                        .get("ev")
                        .or_else(|| params.get("exposure"))
                        .and_then(|v| v.as_f64())
                        .ok_or_else(|| "Missing 'ev' or 'exposure' param".to_string())?
                        as f32;
                    Ok(DarkroomCommand::SetExposure { ev })
                }
                "set_temperature" => {
                    let k = params
                        .get("kelvin")
                        .or_else(|| params.get("temperature"))
                        .and_then(|v| v.as_f64())
                        .ok_or_else(|| "Missing 'kelvin' param".to_string())?;
                    Ok(DarkroomCommand::SetTemperature { kelvin: k })
                }
                "set_tint" => {
                    let t = params
                        .get("tint")
                        .and_then(|v| v.as_f64())
                        .ok_or_else(|| "Missing 'tint' param".to_string())?
                        as f32;
                    Ok(DarkroomCommand::SetTint { tint: t })
                }
                "set_emulsion" => {
                    let arch_val = params.get("archetype");
                    let arch: EmulsionArchetype = if let Some(a_str) = arch_val.and_then(|v| v.as_str()) {
                        serde_json::from_value(serde_json::Value::String(a_str.to_string()))
                            .map_err(|e| format!("Invalid emulsion archetype: {}", e))?
                    } else if let Some(a_int) = arch_val.and_then(|v| v.as_i64()) {
                        match a_int {
                            1 => EmulsionArchetype::RochesterPortrait400,
                            2 => EmulsionArchetype::AshigaraChrome50,
                            3 => EmulsionArchetype::RochesterK14Chrome64,
                            4 => EmulsionArchetype::RochesterCinema500,
                            5 => EmulsionArchetype::RochesterDaylight200,
                            6 => EmulsionArchetype::MobberleySilver400,
                            7 => EmulsionArchetype::CambridgeDiffusion100,
                            _ => EmulsionArchetype::None,
                        }
                    } else {
                        EmulsionArchetype::None
                    };

                    let fmt = params.get("format").and_then(|v| {
                        if let Some(s) = v.as_str() {
                            serde_json::from_value(serde_json::Value::String(s.to_string())).ok()
                        } else if let Some(i) = v.as_i64() {
                            match i {
                                1 => Some(EmulsionFormat::Format120),
                                2 => Some(EmulsionFormat::Format4x5),
                                _ => Some(EmulsionFormat::Format35mm),
                            }
                        } else {
                            None
                        }
                    });

                    let exp = params.get("exposure").and_then(|v| v.as_f64()).map(|f| f as f32);

                    Ok(DarkroomCommand::SetEmulsion {
                        archetype: arch,
                        format: fmt,
                        exposure: exp,
                    })
                }
                "apply_edit_delta" => {
                    let delta_val = params.get("delta").unwrap_or(params);
                    let delta: EditDelta = serde_json::from_value(delta_val.clone())
                        .map_err(|e| format!("Invalid edit delta: {}", e))?;
                    Ok(DarkroomCommand::ApplyEditDelta { delta })
                }
                "reset_workspace" => {
                    let cat = params.get("category").and_then(|v| {
                        if let Some(s) = v.as_str() {
                            match s.to_lowercase().as_str() {
                                "light" => Some(WorkspaceCategory::Light),
                                "color" => Some(WorkspaceCategory::Color),
                                "detail" => Some(WorkspaceCategory::Detail),
                                "emulsion" => Some(WorkspaceCategory::Emulsion),
                                _ => None,
                            }
                        } else {
                            None
                        }
                    });
                    Ok(DarkroomCommand::ResetWorkspace { category: cat })
                }
                "develop_photo" => {
                    let input = params
                        .get("input")
                        .or_else(|| params.get("photo_path"))
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| "Missing 'input' param".to_string())?
                        .to_string();
                    let output = params.get("output").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let fast = params.get("fast").and_then(|v| v.as_bool());
                    let settings = params.get("settings").and_then(|v| serde_json::from_value(v.clone()).ok());
                    Ok(DarkroomCommand::DevelopPhoto {
                        input,
                        output,
                        settings,
                        fast,
                    })
                }
                "convert_to_dng" => {
                    let input = params
                        .get("input")
                        .or_else(|| params.get("photo_path"))
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| "Missing 'input' param".to_string())?
                        .to_string();
                    let output = params.get("output").and_then(|v| v.as_str()).map(|s| s.to_string());
                    Ok(DarkroomCommand::ConvertToDng { input, output })
                }
                "audit_parity" => {
                    let path = params
                        .get("photo_path")
                        .or_else(|| params.get("path"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    Ok(DarkroomCommand::AuditParity { photo_path: path })
                }
                other => Err(format!("Unknown JSON-RPC method: '{}'", other)),
            };
        }

        Err("Payload does not contain 'command' or 'method' discriminator".to_string())
    }
}

/// Uniform command response envelope matching native AppKit, CLI, and Web targets.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommandResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(flatten)]
    pub payload: serde_json::Map<String, serde_json::Value>,
}

impl CommandResponse {
    /// Creates a successful response with optional data payload.
    pub fn ok(payload: serde_json::Value) -> Self {
        let mut map = match payload {
            serde_json::Value::Object(m) => m,
            serde_json::Value::Null => serde_json::Map::new(),
            other => {
                let mut m = serde_json::Map::new();
                m.insert("data".to_string(), other);
                m
            }
        };
        map.remove("status");
        Self {
            status: "ok".to_string(),
            message: None,
            payload: map,
        }
    }

    /// Creates an error response with explanatory message.
    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            status: "error".to_string(),
            message: Some(msg.into()),
            payload: serde_json::Map::new(),
        }
    }

    /// Serializes the response to a JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            "{\"status\":\"error\",\"message\":\"Serialization failed\"}".to_string()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tagged_command_deserialization() {
        let json = r#"{"command":"set_exposure","ev":1.5}"#;
        let cmd = DarkroomCommand::from_json(json).expect("parse tagged command");
        assert_eq!(cmd, DarkroomCommand::SetExposure { ev: 1.5 });
    }

    #[test]
    fn test_json_rpc_method_deserialization() {
        let json = r#"{"method":"open_detail_view","params":{"photo_path":"/tmp/test.cr2"}}"#;
        let cmd = DarkroomCommand::from_json(json).expect("parse rpc command");
        assert_eq!(
            cmd,
            DarkroomCommand::SelectPhoto {
                path: "/tmp/test.cr2".to_string()
            }
        );
    }

    #[test]
    fn test_edit_delta_application() {
        let mut edits = EditSettings::default();
        assert_eq!(edits.exposure, 0.0);
        assert_eq!(edits.temperature, 5500.0);

        let delta = EditDelta {
            exposure: Some(0.75),
            temperature: Some(6200.0),
            contrast: Some(15.0),
            ..Default::default()
        };
        delta.apply_to(&mut edits);

        assert_eq!(edits.exposure, 0.75);
        assert_eq!(edits.temperature, 6200.0);
        assert_eq!(edits.contrast, 15.0);
        assert_eq!(edits.highlights, 0.0); // unchanged
    }

    #[test]
    fn test_command_response_serialization() {
        let res = CommandResponse::ok(serde_json::json!({
            "selected_photo": "/tmp/test.cr2",
            "canvas_mode": "detail"
        }));
        let json = res.to_json();
        assert!(json.contains("\"status\":\"ok\""));
        assert!(json.contains("\"selected_photo\":\"/tmp/test.cr2\""));
        assert!(json.contains("\"canvas_mode\":\"detail\""));

        let err = CommandResponse::error("Photo not found");
        let err_json = err.to_json();
        assert!(err_json.contains("\"status\":\"error\""));
        assert!(err_json.contains("\"message\":\"Photo not found\""));
    }
}
