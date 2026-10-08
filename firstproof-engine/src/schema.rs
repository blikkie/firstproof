use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkspaceCategory {
    Light,
    Color,
    Detail,
    Emulsion,
}

#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScaleMode {
    Linear,
    Mired,
}

#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AdjustmentDescriptor {
    pub id: String,
    pub name: String,
    pub category: WorkspaceCategory,
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub default: f64,
    pub scale_mode: ScaleMode,
    pub unit: String,
    pub precision: u32,
}

/// Non-linear Mired mapping:
/// pos(K) = (500.0 - 10^6 / K) / 450.0
pub fn kelvin_to_mired_norm(kelvin: f64) -> f64 {
    let k = kelvin.clamp(2000.0, 20000.0);
    let m = 1_000_000.0 / k;
    ((500.0 - m) / 450.0).clamp(0.0, 1.0)
}

/// Inverse Mired mapping with discrete step quantization:
/// K(pos) = 10^6 / (500.0 - 450.0 * pos)
/// Below 10,000 K: stepped to nearest 50 K.
/// Above 10,000 K: stepped to nearest 200 K.
pub fn mired_norm_to_kelvin(norm: f64) -> f64 {
    let t = norm.clamp(0.0, 1.0);
    let m = 500.0 - 450.0 * t;
    if m <= 0.0 {
        return 20000.0;
    }
    let raw_k = 1_000_000.0 / m;
    let rounded = if raw_k <= 10000.0 {
        (raw_k / 50.0).round() * 50.0
    } else {
        (raw_k / 200.0).round() * 200.0
    };
    rounded.clamp(2000.0, 20000.0)
}

impl AdjustmentDescriptor {
    pub fn normalized_to_value(&self, norm: f64) -> f64 {
        match self.scale_mode {
            ScaleMode::Linear => {
                let n = norm.clamp(0.0, 1.0);
                let raw = self.min + (self.max - self.min) * n;
                if self.step > 0.0 {
                    (raw / self.step).round() * self.step
                } else {
                    raw
                }
            }
            ScaleMode::Mired => mired_norm_to_kelvin(norm),
        }
    }

    pub fn value_to_normalized(&self, val: f64) -> f64 {
        match self.scale_mode {
            ScaleMode::Linear => {
                if (self.max - self.min).abs() < 1e-9 {
                    0.0
                } else {
                    ((val - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
                }
            }
            ScaleMode::Mired => kelvin_to_mired_norm(val),
        }
    }

    pub fn format_value(&self, val: f64) -> String {
        match self.scale_mode {
            ScaleMode::Mired => format!("{:.0} K", val),
            ScaleMode::Linear => {
                let prefix = if self.unit == "EV" && val > 0.0 { "+" } else { "" };
                let unit_suffix = if self.unit.is_empty() {
                    String::new()
                } else if self.unit == "%" {
                    "%".to_string()
                } else {
                    format!(" {}", self.unit)
                };

                if self.precision == 0 {
                    format!("{}{:.0}{}", prefix, val, unit_suffix)
                } else {
                    format!("{}{:.*}{}", prefix, self.precision as usize, val, unit_suffix)
                }
            }
        }
    }
}

pub fn all_adjustments() -> Vec<AdjustmentDescriptor> {
    vec![
        // Light
        AdjustmentDescriptor {
            id: "exposure".into(),
            name: "Exposure".into(),
            category: WorkspaceCategory::Light,
            min: -3.0,
            max: 3.0,
            step: 0.05,
            default: 0.0,
            scale_mode: ScaleMode::Linear,
            unit: "EV".into(),
            precision: 2,
        },
        AdjustmentDescriptor {
            id: "contrast".into(),
            name: "Contrast".into(),
            category: WorkspaceCategory::Light,
            min: -100.0,
            max: 100.0,
            step: 1.0,
            default: 0.0,
            scale_mode: ScaleMode::Linear,
            unit: "".into(),
            precision: 0,
        },
        AdjustmentDescriptor {
            id: "highlights".into(),
            name: "Highlights".into(),
            category: WorkspaceCategory::Light,
            min: -100.0,
            max: 100.0,
            step: 1.0,
            default: 0.0,
            scale_mode: ScaleMode::Linear,
            unit: "".into(),
            precision: 0,
        },
        AdjustmentDescriptor {
            id: "shadows".into(),
            name: "Shadows".into(),
            category: WorkspaceCategory::Light,
            min: -100.0,
            max: 100.0,
            step: 1.0,
            default: 0.0,
            scale_mode: ScaleMode::Linear,
            unit: "".into(),
            precision: 0,
        },
        AdjustmentDescriptor {
            id: "whites".into(),
            name: "Whites".into(),
            category: WorkspaceCategory::Light,
            min: -100.0,
            max: 100.0,
            step: 1.0,
            default: 0.0,
            scale_mode: ScaleMode::Linear,
            unit: "".into(),
            precision: 0,
        },
        AdjustmentDescriptor {
            id: "blacks".into(),
            name: "Blacks".into(),
            category: WorkspaceCategory::Light,
            min: -100.0,
            max: 100.0,
            step: 1.0,
            default: 0.0,
            scale_mode: ScaleMode::Linear,
            unit: "".into(),
            precision: 0,
        },
        // Color
        AdjustmentDescriptor {
            id: "temperature".into(),
            name: "Warmth (Temp)".into(),
            category: WorkspaceCategory::Color,
            min: 2000.0,
            max: 20000.0,
            step: 50.0,
            default: 5500.0,
            scale_mode: ScaleMode::Mired,
            unit: "K".into(),
            precision: 0,
        },
        AdjustmentDescriptor {
            id: "tint".into(),
            name: "Tint".into(),
            category: WorkspaceCategory::Color,
            min: -200.0,
            max: 200.0,
            step: 1.0,
            default: 0.0,
            scale_mode: ScaleMode::Linear,
            unit: "".into(),
            precision: 0,
        },
        AdjustmentDescriptor {
            id: "vibrance".into(),
            name: "Vibrance".into(),
            category: WorkspaceCategory::Color,
            min: -100.0,
            max: 100.0,
            step: 1.0,
            default: 0.0,
            scale_mode: ScaleMode::Linear,
            unit: "".into(),
            precision: 0,
        },
        AdjustmentDescriptor {
            id: "saturation".into(),
            name: "Saturation".into(),
            category: WorkspaceCategory::Color,
            min: -100.0,
            max: 100.0,
            step: 1.0,
            default: 0.0,
            scale_mode: ScaleMode::Linear,
            unit: "".into(),
            precision: 0,
        },
        AdjustmentDescriptor {
            id: "deep_blue".into(),
            name: "Deep Blue".into(),
            category: WorkspaceCategory::Color,
            min: -100.0,
            max: 100.0,
            step: 1.0,
            default: 0.0,
            scale_mode: ScaleMode::Linear,
            unit: "".into(),
            precision: 0,
        },
        AdjustmentDescriptor {
            id: "foliage".into(),
            name: "Foliage Green".into(),
            category: WorkspaceCategory::Color,
            min: -100.0,
            max: 100.0,
            step: 1.0,
            default: 0.0,
            scale_mode: ScaleMode::Linear,
            unit: "".into(),
            precision: 0,
        },
        // Detail
        AdjustmentDescriptor {
            id: "sharpen_amount".into(),
            name: "Sharpening".into(),
            category: WorkspaceCategory::Detail,
            min: 0.0,
            max: 100.0,
            step: 1.0,
            default: 0.0,
            scale_mode: ScaleMode::Linear,
            unit: "".into(),
            precision: 0,
        },
        AdjustmentDescriptor {
            id: "sharpen_radius".into(),
            name: "Radius".into(),
            category: WorkspaceCategory::Detail,
            min: 0.5,
            max: 3.0,
            step: 0.1,
            default: 1.0,
            scale_mode: ScaleMode::Linear,
            unit: "px".into(),
            precision: 1,
        },
        AdjustmentDescriptor {
            id: "sharpen_masking".into(),
            name: "Masking".into(),
            category: WorkspaceCategory::Detail,
            min: 0.0,
            max: 100.0,
            step: 1.0,
            default: 0.0,
            scale_mode: ScaleMode::Linear,
            unit: "".into(),
            precision: 0,
        },
        AdjustmentDescriptor {
            id: "noise_reduction".into(),
            name: "Luminance Noise".into(),
            category: WorkspaceCategory::Detail,
            min: 0.0,
            max: 100.0,
            step: 1.0,
            default: 0.0,
            scale_mode: ScaleMode::Linear,
            unit: "".into(),
            precision: 0,
        },
        // Emulsion
        AdjustmentDescriptor {
            id: "emulsion_exp".into(),
            name: "Emulsion Exposure".into(),
            category: WorkspaceCategory::Emulsion,
            min: -2.0,
            max: 2.0,
            step: 0.1,
            default: 0.0,
            scale_mode: ScaleMode::Linear,
            unit: "EV".into(),
            precision: 2,
        },
        AdjustmentDescriptor {
            id: "paper_grade".into(),
            name: "Paper Contrast Grade".into(),
            category: WorkspaceCategory::Emulsion,
            min: -2.0,
            max: 2.0,
            step: 0.1,
            default: 0.0,
            scale_mode: ScaleMode::Linear,
            unit: "".into(),
            precision: 1,
        },
        AdjustmentDescriptor {
            id: "process_depth".into(),
            name: "Process Depth".into(),
            category: WorkspaceCategory::Emulsion,
            min: 0.0,
            max: 100.0,
            step: 1.0,
            default: 100.0,
            scale_mode: ScaleMode::Linear,
            unit: "%".into(),
            precision: 0,
        },
    ]
}

pub fn get_adjustment(id: &str) -> Option<AdjustmentDescriptor> {
    all_adjustments().into_iter().find(|adj| adj.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mired_conversion_bounds() {
        let n_2000 = kelvin_to_mired_norm(2000.0);
        let n_20000 = kelvin_to_mired_norm(20000.0);
        assert!((n_2000 - 0.0).abs() < 1e-4);
        assert!((n_20000 - 1.0).abs() < 1e-4);

        let k_0 = mired_norm_to_kelvin(0.0);
        let k_1 = mired_norm_to_kelvin(1.0);
        assert_eq!(k_0, 2000.0);
        assert_eq!(k_1, 20000.0);
    }

    #[test]
    fn test_mired_midpoint_behavior() {
        // Daylight 5500K
        let norm = kelvin_to_mired_norm(5500.0);
        let k = mired_norm_to_kelvin(norm);
        assert_eq!(k, 5500.0);
    }

    #[test]
    fn test_descriptor_formatting() {
        let exp = get_adjustment("exposure").unwrap();
        assert_eq!(exp.format_value(0.5), "+0.50 EV");
        assert_eq!(exp.format_value(-1.25), "-1.25 EV");
        assert_eq!(exp.format_value(0.0), "0.00 EV");

        let temp = get_adjustment("temperature").unwrap();
        assert_eq!(temp.format_value(5500.0), "5500 K");

        let depth = get_adjustment("process_depth").unwrap();
        assert_eq!(depth.format_value(100.0), "100%");
    }
}
