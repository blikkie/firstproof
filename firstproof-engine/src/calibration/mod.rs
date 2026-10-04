// SPDX-License-Identifier: MIT
// Copyright 2026 First Proof Developers

//! Camera calibration registry and sensor database.

pub mod models;
pub mod registry;

pub use models::CameraCalibration;
pub use registry::{CameraCalibrationRegistry, CAMERA_CALIBRATIONS};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_and_fuzzy_model_lookups() {
        // Exact match (Canon EOS 60D verified against _MG_2651.dng Tag 50732 = 125/100)
        let eos60d = CameraCalibrationRegistry::lookup("Canon", "Canon EOS 60D")
            .expect("Canon EOS 60D should be found");
        assert_eq!(eos60d.clean_name, "Canon EOS 60D");
        assert_eq!(eos60d.baseline_exposure, 0.00);
        assert_eq!(eos60d.baseline_sharpness, 1.25);
        assert_eq!(eos60d.black_levels, [2045.0, 2045.0, 2050.0, 2050.0]);

        // Model prefix stripped match
        let a7m3 = CameraCalibrationRegistry::lookup("Sony", "ILCE-7M3")
            .expect("Sony ILCE-7M3 should be found");
        assert_eq!(a7m3.clean_name, "Sony Alpha 7 III");
        assert_eq!(a7m3.baseline_exposure, 0.50);
        assert_eq!(a7m3.baseline_sharpness, 1.00);

        // Nikon Z 6
        let z6 = CameraCalibrationRegistry::lookup("Nikon", "Z 6")
            .expect("Nikon Z 6 should be found");
        assert_eq!(z6.clean_name, "Nikon Z 6");
        assert_eq!(z6.baseline_exposure, 0.50);
        assert_eq!(z6.baseline_sharpness, 1.00);

        // Nikon D850
        let d850 = CameraCalibrationRegistry::lookup("Nikon", "D850")
            .expect("Nikon D850 should be found");
        assert_eq!(d850.baseline_sharpness, 1.00);
    }

    #[test]
    fn test_family_fallbacks_for_unlisted_models() {
        let rebel = CameraCalibrationRegistry::lookup_or_fallback("Canon", "EOS Rebel T7i");
        assert_eq!(rebel.clean_name, "Canon EOS Standard");
        assert_eq!(rebel.baseline_exposure, 0.00);
        assert_eq!(rebel.baseline_sharpness, 1.00);

        let sony_a6400 = CameraCalibrationRegistry::lookup_or_fallback("Sony", "ILCE-6400");
        assert_eq!(sony_a6400.clean_name, "Sony Alpha Standard");
        assert_eq!(sony_a6400.baseline_exposure, 0.50);
        assert_eq!(sony_a6400.baseline_sharpness, 1.00);

        let fuji_xt5 = CameraCalibrationRegistry::lookup_or_fallback("Fujifilm", "X-T5");
        assert_eq!(fuji_xt5.clean_name, "Fujifilm X Standard");
        assert_eq!(fuji_xt5.baseline_exposure, 0.72);
        assert_eq!(fuji_xt5.baseline_sharpness, 1.00);

        let leica_m11 = CameraCalibrationRegistry::lookup_or_fallback("Leica", "M11");
        assert_eq!(leica_m11.clean_name, "Leica Standard");
        assert_eq!(leica_m11.baseline_sharpness, 1.00);

        let unknown = CameraCalibrationRegistry::lookup_or_fallback("UnknownCorp", "FutureCam 9000");
        assert_eq!(unknown.clean_name, "Generic Rec.2020 Standard");
        assert_eq!(unknown.baseline_exposure, 0.0);
        assert_eq!(unknown.baseline_sharpness, 1.00);
    }
}
