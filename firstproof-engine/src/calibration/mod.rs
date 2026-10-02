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
        // Exact match
        let eos60d = CameraCalibrationRegistry::lookup("Canon", "Canon EOS 60D")
            .expect("Canon EOS 60D should be found");
        assert_eq!(eos60d.clean_name, "Canon EOS 60D");
        assert_eq!(eos60d.baseline_exposure, 0.00);
        assert_eq!(eos60d.black_levels, [2045.0, 2045.0, 2050.0, 2050.0]);

        // Model prefix stripped match
        let a7m3 = CameraCalibrationRegistry::lookup("Sony", "ILCE-7M3")
            .expect("Sony ILCE-7M3 should be found");
        assert_eq!(a7m3.clean_name, "Sony Alpha 7 III");
        assert_eq!(a7m3.baseline_exposure, 0.50);

        // Nikon Z 6
        let z6 = CameraCalibrationRegistry::lookup("Nikon", "Z 6")
            .expect("Nikon Z 6 should be found");
        assert_eq!(z6.clean_name, "Nikon Z 6");
        assert_eq!(z6.baseline_exposure, 0.50);
    }

    #[test]
    fn test_family_fallbacks_for_unlisted_models() {
        let rebel = CameraCalibrationRegistry::lookup_or_fallback("Canon", "EOS Rebel T7i");
        assert_eq!(rebel.clean_name, "Canon EOS Standard");
        assert_eq!(rebel.baseline_exposure, 0.00);

        let sony_a6400 = CameraCalibrationRegistry::lookup_or_fallback("Sony", "ILCE-6400");
        assert_eq!(sony_a6400.clean_name, "Sony Alpha Standard");
        assert_eq!(sony_a6400.baseline_exposure, 0.50);

        let fuji_xt5 = CameraCalibrationRegistry::lookup_or_fallback("Fujifilm", "X-T5");
        assert_eq!(fuji_xt5.clean_name, "Fujifilm X Standard");
        assert_eq!(fuji_xt5.baseline_exposure, 0.72);

        let unknown = CameraCalibrationRegistry::lookup_or_fallback("UnknownCorp", "FutureCam 9000");
        assert_eq!(unknown.clean_name, "Generic Rec.2020 Standard");
        assert_eq!(unknown.baseline_exposure, 0.0);
    }
}
