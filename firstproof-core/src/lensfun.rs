use roxmltree::Document;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Camera {
    pub maker: String,
    pub model: String,
    pub mount: String,
    pub crop_factor: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DistortionCalib {
    pub model: String, // "poly3" or "ptlens"
    pub focal_length: f32,
    pub coefficients: [f32; 4], // [k1, k2, k3, k4] or [a, b, c, d]
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TcaCalib {
    pub focal_length: f32,
    pub coefficients: [f32; 2], // [kr, kb] (red & blue scale relative to green)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VignettingCalib {
    pub focal_length: f32,
    pub aperture: f32,
    pub distance: f32,
    pub coefficients: [f32; 3], // [k1, k2, k3]
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(uniffi::Record, Debug, Clone, PartialEq)]
pub struct LensCorrectionCoefficients {
    pub distortion: Vec<f32>,
    pub tca: Vec<f32>,
    pub vignetting: Vec<f32>,
    pub distortion_model: String,
    pub has_distortion: bool,
    pub has_tca: bool,
    pub has_vignetting: bool,
}

impl Default for LensCorrectionCoefficients {
    fn default() -> Self {
        Self {
            distortion: vec![0.0, 0.0, 0.0, 0.0],
            tca: vec![1.0, 1.0],
            vignetting: vec![0.0, 0.0, 0.0],
            distortion_model: "poly3".to_string(),
            has_distortion: false,
            has_tca: false,
            has_vignetting: false,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LensfunDatabase {
    pub cameras: Vec<Camera>,
    pub lenses: Vec<Lens>,
}

impl LensfunDatabase {
    pub fn new() -> Self {
        // 1. Check application bundle relative to current_exe() (e.g. macOS App Bundle Contents/Resources)
        if let Ok(exe_path) = std::env::current_exe() {
            // E.g. /path/to/FirstProof.app/Contents/MacOS/FirstProofApp -> Contents/Resources/lensfun.json
            if let Some(contents_dir) = exe_path.parent().and_then(|p| p.parent()) {
                let bundle_json = contents_dir.join("Resources/lensfun.json");
                if let Ok(loaded) = Self::from_json_file(&bundle_json) {
                    if !loaded.cameras.is_empty() || !loaded.lenses.is_empty() {
                        return loaded;
                    }
                }
            }
            // Check ancestors for workspace builds (e.g. target/debug, dist/)
            for ancestor in exe_path.ancestors().take(6) {
                let candidate = ancestor.join("assets/lensfun/lensfun.json");
                if let Ok(loaded) = Self::from_json_file(&candidate) {
                    if !loaded.cameras.is_empty() || !loaded.lenses.is_empty() {
                        return loaded;
                    }
                }
            }
        }

        // 2. Check FIRSTPROOF_LENSFUN_PATH environment variable
        if let Ok(env_path) = std::env::var("FIRSTPROOF_LENSFUN_PATH") {
            if let Ok(loaded) = Self::from_json_file(&env_path) {
                if !loaded.cameras.is_empty() || !loaded.lenses.is_empty() {
                    return loaded;
                }
            }
        }

        // 3. Check for bundled pre-indexed JSON database relative to CWD (instant startup: <2ms)
        for json_path in &[
            "Contents/Resources/lensfun.json",
            "../Resources/lensfun.json",
            "assets/lensfun/lensfun.json",
            "../assets/lensfun/lensfun.json",
        ] {
            if let Ok(loaded) = Self::from_json_file(json_path) {
                if !loaded.cameras.is_empty() || !loaded.lenses.is_empty() {
                    return loaded;
                }
            }
        }

        let mut db = Self::default();

        // 4. Check for directory of bundled XML files (e.g. system Lensfun installations)
        for dir in &[
            "Contents/Resources/lensfun",
            "assets/lensfun/db",
            "../assets/lensfun/db",
            "/opt/homebrew/share/lensfun/version_1",
            "/usr/local/share/lensfun/version_1",
            "/usr/share/lensfun/version_1",
        ] {
            if let Ok(count) = db.load_directory(dir) {
                if count > 0 {
                    return db;
                }
            }
        }

        db
    }

    /// Deserializes a pre-indexed JSON database file.
    pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let file = fs::File::open(path).map_err(|e| e.to_string())?;
        let reader = std::io::BufReader::new(file);
        serde_json::from_reader(reader).map_err(|e| e.to_string())
    }

    /// Serializes the in-memory database to a JSON file.
    pub fn save_json_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let file = fs::File::create(path).map_err(|e| e.to_string())?;
        let writer = std::io::BufWriter::new(file);
        serde_json::to_writer(writer, self).map_err(|e| e.to_string())
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
            if path.is_file() && path.extension().is_some_and(|ext| ext == "xml") {
                if let Ok(content) = fs::read_to_string(&path) {
                    match self.load_xml_string(&content) {
                        Ok(()) => loaded_files += 1,
                        Err(e) => eprintln!("Failed to parse {}: {}", path.display(), e),
                    }
                }
            }
        }

        Ok(loaded_files)
    }

    /// Parses XML string database content and appends cameras and lenses to memory.
    pub fn load_xml_string(&mut self, xml_content: &str) -> Result<(), String> {
        let opt = roxmltree::ParsingOptions {
            allow_dtd: true,
            ..Default::default()
        };
        let doc = Document::parse_with_options(xml_content, opt).map_err(|e| e.to_string())?;
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
                res.distortion = dist.coefficients.to_vec();
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
                res.tca = tca.coefficients.to_vec();
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
                res.vignetting = vig.coefficients.to_vec();
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
        let mut min_f = f32::MAX;
        let mut max_f = f32::MIN;
        let mut has_focal = false;

        for f in distortion_calib
            .iter()
            .map(|d| d.focal_length)
            .chain(tca_calib.iter().map(|t| t.focal_length))
            .chain(vignetting_calib.iter().map(|v| v.focal_length))
        {
            if f > 0.0 {
                min_f = min_f.min(f);
                max_f = max_f.max(f);
                has_focal = true;
            }
        }

        if has_focal {
            min_focal = min_f;
            max_focal = max_f;
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
    let mut out = String::with_capacity(s.len() + 8);
    let mut prev_is_alphabetic = false;
    let mut prev_is_digit = false;

    for c in s.chars() {
        if c.is_alphabetic() {
            if prev_is_digit {
                out.push(' ');
            }
            out.push(c.to_ascii_lowercase());
            prev_is_alphabetic = true;
            prev_is_digit = false;
        } else if c.is_ascii_digit() {
            if prev_is_alphabetic {
                out.push(' ');
            }
            out.push(c);
            prev_is_alphabetic = false;
            prev_is_digit = true;
        } else {
            // Punctuation, dashes, slashes, whitespace are token separators
            if !out.ends_with(' ') && !out.is_empty() {
                out.push(' ');
            }
            prev_is_alphabetic = false;
            prev_is_digit = false;
        }
    }

    out.split_whitespace().collect::<Vec<&str>>().join(" ")
}



#[cfg(test)]
mod tests {
    use super::*;

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

        let coeffs = db.resolve_coefficients("Canon", "5D Mark IV", "EF 24-70mm f/2.8L II USM", 24.0, Some(2.8));
        assert!(coeffs.has_distortion);
        assert_eq!(coeffs.distortion[0], 0.02964);
        assert_eq!(coeffs.distortion_model, "ptlens");
        assert!(coeffs.has_tca);
        assert!(coeffs.has_vignetting);
        
        // Test user's Canon EOS 60D + EF28mm f/2.8 setup
        let coeffs_60d = db.resolve_coefficients("Canon", "Canon EOS 60D", "EF28mm f/2.8", 28.0, Some(2.8));
        assert!(coeffs_60d.has_distortion);
        assert!((coeffs_60d.distortion[0] - 0.007228).abs() < 1e-4);
        assert_eq!(coeffs_60d.distortion_model, "ptlens");
    }
}
