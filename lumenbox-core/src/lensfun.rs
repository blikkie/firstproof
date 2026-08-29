use roxmltree::Document;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct Camera {
    pub maker: String,
    pub model: String,
    pub mount: String,
    pub crop_factor: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DistortionCalib {
    pub model: String, // "poly3" or "ptlens"
    pub focal_length: f32,
    pub coefficients: [f32; 4], // [k1, k2, k3, k4] or [a, b, c, d]
}

#[derive(Debug, Clone, PartialEq)]
pub struct TcaCalib {
    pub focal_length: f32,
    pub coefficients: [f32; 2], // [kr, kb] (red & blue scale relative to green)
}

#[derive(Debug, Clone, PartialEq)]
pub struct VignettingCalib {
    pub focal_length: f32,
    pub aperture: f32,
    pub distance: f32,
    pub coefficients: [f32; 3], // [k1, k2, k3]
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lens {
    pub maker: String,
    pub model: String,
    pub mounts: Vec<String>,
    pub crop_factor: f32,
    pub min_focal: f32,
    pub max_focal: f32,
    pub min_aperture: f32,
    pub max_aperture: f32,
    pub distortion_calib: Vec<DistortionCalib>,
    pub tca_calib: Vec<TcaCalib>,
    pub vignetting_calib: Vec<VignettingCalib>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LensCorrectionCoefficients {
    pub distortion: [f32; 4],
    pub tca: [f32; 2],
    pub vignetting: [f32; 3],
    pub distortion_model: String,
    pub has_distortion: bool,
    pub has_tca: bool,
    pub has_vignetting: bool,
}

impl Default for LensCorrectionCoefficients {
    fn default() -> Self {
        Self {
            distortion: [0.0, 0.0, 0.0, 0.0],
            tca: [1.0, 1.0],
            vignetting: [0.0, 0.0, 0.0],
            distortion_model: "poly3".to_string(),
            has_distortion: false,
            has_tca: false,
            has_vignetting: false,
        }
    }
}

#[derive(Debug, Default)]
pub struct LensfunDatabase {
    pub cameras: Vec<Camera>,
    pub lenses: Vec<Lens>,
}

impl LensfunDatabase {
    pub fn new() -> Self {
        let mut db = Self::default();
        let _ = db.load_xml_string(DEFAULT_LENSFUN_XML);
        
        for dir in &[
            "/opt/homebrew/share/lensfun/version_1",
            "/usr/local/share/lensfun/version_1",
            "/usr/share/lensfun/version_1",
        ] {
            let _ = db.load_directory(dir);
        }
        
        db
    }

    /// Loads all `.xml` files from a given directory path into the database.
    pub fn load_directory<P: AsRef<Path>>(&mut self, dir_path: P) -> Result<u32, String> {
        let dir_path = dir_path.as_ref();
        if !dir_path.exists() || !dir_path.is_dir() {
            return Err(format!("Directory not found: {}", dir_path.display()));
        }

        let entries = fs::read_dir(dir_path).map_err(|e| e.to_string())?;
        let mut loaded_files = 0u32;

        for entry in entries.filter_map(std::result::Result::ok) {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "xml") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if self.load_xml_string(&content).is_ok() {
                        loaded_files += 1;
                    }
                }
            }
        }

        Ok(loaded_files)
    }

    /// Parses XML string database content and appends cameras and lenses to memory.
    pub fn load_xml_string(&mut self, xml_content: &str) -> Result<(), String> {
        let doc = Document::parse(xml_content).map_err(|e| e.to_string())?;
        let root = doc.root_element();

        if root.tag_name().name() != "lensdatabase" {
            return Err("Root element must be <lensdatabase>".to_string());
        }

        for child in root.children() {
            if !child.is_element() {
                continue;
            }

            match child.tag_name().name() {
                "camera" => {
                    if let Some(cam) = parse_camera_element(&child) {
                        self.cameras.push(cam);
                    }
                }
                "lens" => {
                    if let Some(lens) = parse_lens_element(&child) {
                        self.lenses.push(lens);
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Finds a matching camera by make and model using fuzzy string normalization.
    pub fn find_camera(&self, make: &str, model: &str) -> Option<&Camera> {
        let norm_make = normalize_str(make);
        let norm_model = normalize_str(model);

        self.cameras.iter().find(|c| {
            let c_make = normalize_str(&c.maker);
            let c_model = normalize_str(&c.model);

            (c_make.contains(&norm_make) || norm_make.contains(&c_make))
                && (c_model.contains(&norm_model) || norm_model.contains(&c_model))
        })
    }

    /// Finds a matching lens given camera information, lens string, and optional focal length.
    pub fn find_lens(
        &self,
        camera_make: &str,
        camera_model: &str,
        lens_model: &str,
        focal_length: Option<f32>,
    ) -> Option<&Lens> {
        let norm_lens = normalize_str(lens_model);
        if norm_lens.is_empty() {
            return None;
        }

        let mut candidates: Vec<&Lens> = self
            .lenses
            .iter()
            .filter(|l| {
                let l_model = normalize_str(&l.model);
                l_model.contains(&norm_lens) || norm_lens.contains(&l_model)
            })
            .collect();

        // If no match found by lens_model, attempt matching camera maker & focal length bounds
        if candidates.is_empty() {
            if let Some(cam) = self.find_camera(camera_make, camera_model) {
                let cam_mount = normalize_str(&cam.mount);
                candidates = self
                    .lenses
                    .iter()
                    .filter(|l| {
                        l.mounts
                            .iter()
                            .any(|m| normalize_str(m) == cam_mount)
                    })
                    .collect();
            }
        }

        if let Some(focal) = focal_length {
            candidates
                .into_iter()
                .find(|l| focal >= l.min_focal - 0.5 && focal <= l.max_focal + 0.5)
        } else {
            candidates.into_iter().next()
        }
    }

    /// Resolves correction coefficients for distortion, TCA, and vignetting for a given shot.
    pub fn resolve_coefficients(
        &self,
        camera_make: &str,
        camera_model: &str,
        lens_model: &str,
        focal_length: f32,
        aperture: Option<f32>,
    ) -> LensCorrectionCoefficients {
        let lens = match self.find_lens(camera_make, camera_model, lens_model, Some(focal_length)) {
            Some(l) => l,
            None => return LensCorrectionCoefficients::default(),
        };

        let mut res = LensCorrectionCoefficients::default();

        // 1. Resolve Distortion (closest focal length)
        if !lens.distortion_calib.is_empty() {
            if let Some(dist) = lens
                .distortion_calib
                .iter()
                .min_by(|a, b| {
                    (a.focal_length - focal_length)
                        .abs()
                        .partial_cmp(&(b.focal_length - focal_length).abs())
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
            {
                res.distortion = dist.coefficients;
                res.distortion_model = dist.model.clone();
                res.has_distortion = true;
            }
        }

        // 2. Resolve TCA (closest focal length)
        if !lens.tca_calib.is_empty() {
            if let Some(tca) = lens
                .tca_calib
                .iter()
                .min_by(|a, b| {
                    (a.focal_length - focal_length)
                        .abs()
                        .partial_cmp(&(b.focal_length - focal_length).abs())
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
            {
                res.tca = tca.coefficients;
                res.has_tca = true;
            }
        }

        // 3. Resolve Vignetting (closest focal length & aperture)
        if !lens.vignetting_calib.is_empty() {
            let target_aperture = aperture.unwrap_or(2.8);
            if let Some(vig) = lens
                .vignetting_calib
                .iter()
                .min_by(|a, b| {
                    let diff_a = (a.focal_length - focal_length).abs() * 2.0
                        + (a.aperture - target_aperture).abs();
                    let diff_b = (b.focal_length - focal_length).abs() * 2.0
                        + (b.aperture - target_aperture).abs();
                    diff_a
                        .partial_cmp(&diff_b)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
            {
                res.vignetting = vig.coefficients;
                res.has_vignetting = true;
            }
        }

        res
    }
}

fn parse_camera_element(node: &roxmltree::Node) -> Option<Camera> {
    let mut maker = String::new();
    let mut model = String::new();
    let mut mount = String::new();
    let mut crop_factor = 1.0f32;

    for child in node.children() {
        if !child.is_element() {
            continue;
        }
        let text = child.text().unwrap_or("").trim().to_string();
        match child.tag_name().name() {
            "maker" => maker = text,
            "model" => model = text,
            "mount" => mount = text,
            "cropfactor" => crop_factor = text.parse::<f32>().unwrap_or(1.0),
            _ => {}
        }
    }

    if maker.is_empty() || model.is_empty() {
        None
    } else {
        Some(Camera {
            maker,
            model,
            mount,
            crop_factor,
        })
    }
}

fn parse_lens_element(node: &roxmltree::Node) -> Option<Lens> {
    let mut maker = String::new();
    let mut model = String::new();
    let mut mounts = Vec::new();
    let mut crop_factor = 1.0f32;
    let mut min_focal = 0.0f32;
    let mut max_focal = 0.0f32;
    let min_aperture = 1.0f32;
    let max_aperture = 22.0f32;

    let mut distortion_calib = Vec::new();
    let mut tca_calib = Vec::new();
    let mut vignetting_calib = Vec::new();

    for child in node.children() {
        if !child.is_element() {
            continue;
        }
        let name = child.tag_name().name();
        let text = child.text().unwrap_or("").trim().to_string();

        match name {
            "maker" => maker = text,
            "model" => model = text,
            "mount" => mounts.push(text),
            "cropfactor" => crop_factor = text.parse::<f32>().unwrap_or(1.0),
            "calibration" => {
                for calib_node in child.children().filter(|n| n.is_element()) {
                    match calib_node.tag_name().name() {
                        "distortion" => {
                            if let Some(d) = parse_distortion(&calib_node) {
                                distortion_calib.push(d);
                            }
                        }
                        "tca" => {
                            if let Some(t) = parse_tca(&calib_node) {
                                tca_calib.push(t);
                            }
                        }
                        "vignetting" => {
                            if let Some(v) = parse_vignetting(&calib_node) {
                                vignetting_calib.push(v);
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    if min_focal == 0.0 && max_focal == 0.0 {
        if let Some(first_dist) = distortion_calib.first() {
            min_focal = first_dist.focal_length;
            max_focal = first_dist.focal_length;
        }
    }

    if model.is_empty() {
        None
    } else {
        Some(Lens {
            maker,
            model,
            mounts,
            crop_factor,
            min_focal,
            max_focal,
            min_aperture,
            max_aperture,
            distortion_calib,
            tca_calib,
            vignetting_calib,
        })
    }
}

fn parse_distortion(node: &roxmltree::Node) -> Option<DistortionCalib> {
    let model = node.attribute("model").unwrap_or("poly3").to_string();
    let focal_length = node.attribute("focal").and_then(|f| f.parse::<f32>().ok())?;

    let k1 = node.attribute("k1").and_then(|v| v.parse::<f32>().ok()).unwrap_or(0.0);
    let k2 = node.attribute("k2").and_then(|v| v.parse::<f32>().ok()).unwrap_or(0.0);
    let k3 = node.attribute("k3").and_then(|v| v.parse::<f32>().ok()).unwrap_or(0.0);
    let k4 = node.attribute("k4").and_then(|v| v.parse::<f32>().ok()).unwrap_or(0.0);

    let a = node.attribute("a").and_then(|v| v.parse::<f32>().ok()).unwrap_or(k1);
    let b = node.attribute("b").and_then(|v| v.parse::<f32>().ok()).unwrap_or(k2);
    let c = node.attribute("c").and_then(|v| v.parse::<f32>().ok()).unwrap_or(k3);
    let d = node.attribute("d").and_then(|v| v.parse::<f32>().ok()).unwrap_or(k4);

    Some(DistortionCalib {
        model,
        focal_length,
        coefficients: [a, b, c, d],
    })
}

fn parse_tca(node: &roxmltree::Node) -> Option<TcaCalib> {
    let focal_length = node.attribute("focal").and_then(|f| f.parse::<f32>().ok())?;
    let kr = node.attribute("kr").and_then(|v| v.parse::<f32>().ok()).unwrap_or(1.0);
    let kb = node.attribute("kb").and_then(|v| v.parse::<f32>().ok()).unwrap_or(1.0);

    Some(TcaCalib {
        focal_length,
        coefficients: [kr, kb],
    })
}

fn parse_vignetting(node: &roxmltree::Node) -> Option<VignettingCalib> {
    let focal_length = node.attribute("focal").and_then(|f| f.parse::<f32>().ok())?;
    let aperture = node.attribute("aperture").and_then(|a| a.parse::<f32>().ok()).unwrap_or(2.8);
    let distance = node.attribute("distance").and_then(|d| d.parse::<f32>().ok()).unwrap_or(1000.0);

    let k1 = node.attribute("k1").and_then(|v| v.parse::<f32>().ok()).unwrap_or(0.0);
    let k2 = node.attribute("k2").and_then(|v| v.parse::<f32>().ok()).unwrap_or(0.0);
    let k3 = node.attribute("k3").and_then(|v| v.parse::<f32>().ok()).unwrap_or(0.0);

    Some(VignettingCalib {
        focal_length,
        aperture,
        distance,
        coefficients: [k1, k2, k3],
    })
}

fn normalize_str(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

const DEFAULT_LENSFUN_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<lensdatabase version="1">
    <camera>
        <maker>Canon</maker>
        <model>Canon EOS 60D</model>
        <model>Canon EOS 60D / Kiss X4 / Rebel T2i</model>
        <mount>Canon EF-S</mount>
        <mount>Canon EF</mount>
        <cropfactor>1.6</cropfactor>
    </camera>
    <camera>
        <maker>Canon</maker>
        <model>Canon EOS 5D Mark IV</model>
        <mount>Canon EF</mount>
        <cropfactor>1.0</cropfactor>
    </camera>
    <camera>
        <maker>Panasonic</maker>
        <model>Lumix LX3</model>
        <mount>fixed</mount>
        <cropfactor>4.7</cropfactor>
    </camera>
    <camera>
        <maker>Sony</maker>
        <model>ILCE-7M3</model>
        <mount>Sony E</mount>
        <cropfactor>1.0</cropfactor>
    </camera>
    <lens>
        <maker>Canon</maker>
        <model>Canon EF 28mm f/2.8</model>
        <model>EF28mm f/2.8</model>
        <mount>Canon EF</mount>
        <mount>Canon EF-S</mount>
        <cropfactor>1.0</cropfactor>
        <calibration>
            <distortion model="poly3" focal="28" k1="-0.024" k2="0.006" />
            <tca focal="28" kr="1.0003" kb="0.9997" />
            <vignetting focal="28" aperture="2.8" distance="10" k1="-0.4" k2="0.1" k3="0.0" />
        </calibration>
    </lens>
    <lens>
        <maker>Canon</maker>
        <model>Canon EF 50mm f/1.8 II</model>
        <model>EF50mm f/1.8 II</model>
        <mount>Canon EF</mount>
        <mount>Canon EF-S</mount>
        <cropfactor>1.0</cropfactor>
        <calibration>
            <distortion model="poly3" focal="50" k1="-0.012" k2="0.003" />
            <tca focal="50" kr="1.0001" kb="0.9999" />
            <vignetting focal="50" aperture="1.8" distance="10" k1="-0.6" k2="0.2" k3="0.0" />
        </calibration>
    </lens>
    <lens>
        <maker>Canon</maker>
        <model>Canon EF 24-70mm f/2.8L II USM</model>
        <model>EF24-70mm f/2.8L II USM</model>
        <mount>Canon EF</mount>
        <cropfactor>1.0</cropfactor>
        <calibration>
            <distortion model="poly3" focal="24" k1="-0.035" k2="0.002" />
            <distortion model="poly3" focal="70" k1="0.015" k2="-0.001" />
            <tca focal="24" kr="1.0002" kb="0.9998" />
            <vignetting focal="24" aperture="2.8" distance="10" k1="-0.5" k2="0.1" k3="0.0" />
        </calibration>
    </lens>
</lensdatabase>"#;

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_LENSFUN_XML: &str = DEFAULT_LENSFUN_XML;

    #[test]
    fn test_parse_xml_and_matching() {
        let db = LensfunDatabase::new();

        assert!(db.cameras.len() >= 4);
        assert!(db.lenses.len() >= 3);

        let cam = db.find_camera("Canon", "5D Mark IV").expect("Camera should match");
        assert_eq!(cam.maker, "Canon");

        let lens = db
            .find_lens("Canon", "Canon EOS 5D Mark IV", "EF 24-70mm f/2.8L II USM", Some(24.0))
            .expect("Lens should match");
        assert_eq!(lens.maker, "Canon");

        let coeffs = db.resolve_coefficients("Canon", "5D Mark IV", "24-70mm", 24.0, Some(2.8));
        assert!(coeffs.has_distortion);
        assert_eq!(coeffs.distortion[0], -0.035);
        assert!(coeffs.has_tca);
        assert_eq!(coeffs.tca[0], 1.0002);
        assert!(coeffs.has_vignetting);
        assert_eq!(coeffs.vignetting[0], -0.5);
        
        // Test user's Canon EOS 60D + EF28mm f/2.8 setup
        let coeffs_60d = db.resolve_coefficients("Canon", "Canon EOS 60D", "EF28mm f/2.8", 28.0, Some(2.8));
        assert!(coeffs_60d.has_distortion);
        assert_eq!(coeffs_60d.distortion[0], -0.024);
    }
}
