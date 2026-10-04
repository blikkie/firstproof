use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum ColorProfileMode {
    #[default]
    AgxVivid,
    AgxFilmic,
    CameraStandard,
    CameraPortrait,
    CameraLandscape,
    CameraVivid,
}


#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum ExportColorSpace {
    #[default]
    Srgb,
    DisplayP3,
}


#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum ExportRenderingIntent {
    Perceptual,
    #[default]
    RelativeColorimetric,
}

/// Execution tier for rendering: fast interactive 60–120fps viewport canvas
/// vs. high-precision multi-pass offline export pipeline.
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum RenderTier {
    #[default]
    Viewport,
    ArchivalExport,
}

/// RAW CFA Bayer demosaicing precision quality mode.
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum DemosaicQuality {
    #[default]
    Standard,
    Archival,
}


#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum EmulsionGrainMode {
    #[default]
    Off,
    SeedFromSensor,
    SynthesizePhysical,
}

impl EmulsionGrainMode {
    #[inline]
    pub fn id_number(&self) -> i32 {
        match self {
            Self::Off => 0,
            Self::SeedFromSensor => 1,
            Self::SynthesizePhysical => 2,
        }
    }

    pub fn from_id(id: i32) -> Self {
        match id {
            1 => Self::SeedFromSensor,
            2 => Self::SynthesizePhysical,
            _ => Self::Off,
        }
    }
}

#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RemjetBackingMode {
    #[default]
    Intact,
    Stripped,
}

impl RemjetBackingMode {
    #[inline]
    pub fn id_number(&self) -> i32 {
        match self {
            Self::Intact => 0,
            Self::Stripped => 1,
        }
    }

    pub fn from_id(id: i32) -> Self {
        match id {
            1 => Self::Stripped,
            _ => Self::Intact,
        }
    }
}

#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum EmulsionArchetype {
    #[default]
    None,
    RochesterPortrait400,  // C-41 Color Negative (Rochester NY)
    AshigaraChrome50,      // E-6 Color Reversal (Minamiashigara, Japan)
    RochesterK14Chrome64,  // K-14 Non-Substantive Reversal (Rochester NY)
    RochesterCinema500,    // ECN-2 Tungsten Cinema Negative (Rochester NY)
    RochesterDaylight200,  // [Deprecated/Merged into RochesterPortrait400]
    MobberleySilver400,    // Panchromatic Silver Halide B&W Negative (Mobberley, UK)
    CambridgeDiffusion100, // Instant Dye Diffusion Reversal (Cambridge MA)
    LeverkusenPortrait160, // Substantive Ochre/Pastel Color Negative (Leverkusen, DE)
    AshigaraPortrait400,   // 4th Color Layer Cyan-Sensitive Color Negative (Minamiashigara, JP)
    LiguriaCinema80,       // 1960s Italian High-Acutance Cinema B&W (Cairo Montenotte, IT)
    BaodingSilver100,      // Cubic Silver Halide with Halation Bloom B&W (Baoding, CN)
}


impl EmulsionArchetype {
    #[inline]
    pub fn id_number(&self) -> i32 {
        match self {
            Self::None => 0,
            Self::RochesterPortrait400 => 1,
            Self::AshigaraChrome50 => 2,
            Self::RochesterK14Chrome64 => 3,
            Self::RochesterCinema500 => 4,
            Self::RochesterDaylight200 => 5,
            Self::MobberleySilver400 => 6,
            Self::CambridgeDiffusion100 => 7,
            Self::LeverkusenPortrait160 => 8,
            Self::AshigaraPortrait400 => 9,
            Self::LiguriaCinema80 => 10,
            Self::BaodingSilver100 => 11,
        }
    }

    pub fn from_id(id: i32) -> Self {
        match id {
            1 => Self::RochesterPortrait400,
            2 => Self::AshigaraChrome50,
            3 => Self::RochesterK14Chrome64,
            4 => Self::RochesterCinema500,
            // 5 was RochesterDaylight200, now merged into RochesterPortrait400 per spec
            5 => Self::RochesterPortrait400,
            6 => Self::MobberleySilver400,
            7 => Self::CambridgeDiffusion100,
            8 => Self::LeverkusenPortrait160,
            9 => Self::AshigaraPortrait400,
            10 => Self::LiguriaCinema80,
            11 => Self::BaodingSilver100,
            _ => Self::None,
        }
    }

    #[inline]
    pub fn is_monochrome(&self) -> bool {
        matches!(
            self,
            Self::MobberleySilver400 | Self::LiguriaCinema80 | Self::BaodingSilver100
        )
    }

    /// Returns the science-calibrated default `(grain_size, grain_amount)` for this emulsion archetype,
    /// derived from published ANSI/ISO 10505 diffuse RMS granularity measurements at D = 1.0.
    #[inline]
    pub fn calibrated_grain_defaults(&self) -> (f32, f32) {
        match self {
            Self::AshigaraChrome50 => (0.55, 0.45),        // RMS 9 (Extremely fine reversal dye clouds)
            Self::RochesterK14Chrome64 => (0.70, 0.60),    // RMS 11 (Non-substantive thin layer)
            Self::LiguriaCinema80 => (0.75, 0.65),         // ISO 80 High-silver cinema B&W
            Self::LeverkusenPortrait160 => (0.80, 0.70),   // RMS 10 (Fine color negative)
            Self::CambridgeDiffusion100 => (0.85, 0.50),   // Instant dye diffusion transfer (DDT)
            Self::AshigaraPortrait400 => (0.95, 0.75),     // RMS 11 (Sigma-crystal 4th layer)
            Self::RochesterPortrait400 => (1.00, 0.80),    // RMS 11 (T-Grain tabular portrait)
            Self::BaodingSilver100 => (1.05, 1.00),        // RMS 14 (Cubic domestic silver)
            Self::RochesterCinema500 => (1.25, 1.10),      // RMS 13 (ECN-2 500T Cinema grain)
            Self::MobberleySilver400 => (1.35, 1.20),      // RMS 14 (Classic British documentary cubic)
            _ => (1.00, 1.00),
        }
    }
}

#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum EmulsionFormat {
    #[default]
    Format35mm,
    Format120,
    Format4x5,
    Super8,
    Format16mm,
}


impl EmulsionFormat {
    #[inline]
    pub fn id_number(&self) -> i32 {
        match self {
            Self::Format35mm => 0,
            Self::Format120 => 1,
            Self::Format4x5 => 2,
            Self::Super8 => 3,
            Self::Format16mm => 4,
        }
    }

    pub fn from_id(id: i32) -> Self {
        match id {
            1 => Self::Format120,
            2 => Self::Format4x5,
            3 => Self::Super8,
            4 => Self::Format16mm,
            _ => Self::Format35mm,
        }
    }

    /// Optical enlargement magnification factor relative to 35mm still frame ($36 \times 24\,\text{mm}$, diag $\approx 43.3\,\text{mm}$).
    #[inline]
    pub fn magnification_factor(&self) -> f32 {
        match self {
            Self::Super8 => 7.3,
            Self::Format16mm => 3.4,
            Self::Format35mm => 1.0,
            Self::Format120 => 0.48,
            Self::Format4x5 => 0.28,
        }
    }
}

#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum PrintMediaMode {
    #[default]
    None,
    ChromogenicTypeC,
    SilverDyeBleach,
    FineArtCottonRag,
    BarytaSilverGelatin,
    WarmtoneChlorobromide,
    PlatinumPalladium,
    Cyanotype,
    BamakoSilverContact,
}


impl PrintMediaMode {
    #[inline]
    pub fn is_monochrome(&self) -> bool {
        matches!(
            self,
            Self::BarytaSilverGelatin
                | Self::WarmtoneChlorobromide
                | Self::PlatinumPalladium
                | Self::Cyanotype
                | Self::BamakoSilverContact
        )
    }

    #[inline]
    pub fn id_number(&self) -> i32 {
        match self {
            Self::None => 0,
            Self::ChromogenicTypeC => 1,
            Self::SilverDyeBleach => 2,
            Self::FineArtCottonRag => 3,
            Self::BarytaSilverGelatin => 4,
            Self::WarmtoneChlorobromide => 5,
            Self::PlatinumPalladium => 6,
            Self::Cyanotype => 7,
            Self::BamakoSilverContact => 8,
        }
    }

    pub fn from_id(id: i32) -> Self {
        match id {
            1 => Self::ChromogenicTypeC,
            2 => Self::SilverDyeBleach,
            3 => Self::FineArtCottonRag,
            4 => Self::BarytaSilverGelatin,
            5 => Self::WarmtoneChlorobromide,
            6 => Self::PlatinumPalladium,
            7 => Self::Cyanotype,
            8 => Self::BamakoSilverContact,
            _ => Self::None,
        }
    }
}

#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum EnlargerFilterMode {
    #[default]
    NeutralPanchromatic,
    YellowK2,
    Orange16,
    Red25A,
    GreenX1,
    Orthochromatic,
}


impl EnlargerFilterMode {
    #[inline]
    pub fn id_number(&self) -> i32 {
        match self {
            Self::NeutralPanchromatic => 0,
            Self::YellowK2 => 1,
            Self::Orange16 => 2,
            Self::Red25A => 3,
            Self::GreenX1 => 4,
            Self::Orthochromatic => 5,
        }
    }

    pub fn from_id(id: i32) -> Self {
        match id {
            1 => Self::YellowK2,
            2 => Self::Orange16,
            3 => Self::Red25A,
            4 => Self::GreenX1,
            5 => Self::Orthochromatic,
            _ => Self::NeutralPanchromatic,
        }
    }

    /// Spectral weights for converting linear RGB into panchromatic/filtered luminance
    #[inline]
    pub fn spectral_weights(&self) -> (f32, f32, f32) {
        match self {
            Self::NeutralPanchromatic => (0.2627, 0.6780, 0.0593),
            Self::YellowK2 => (0.35, 0.60, 0.05),
            Self::Orange16 => (0.55, 0.42, 0.03),
            Self::Red25A => (0.85, 0.15, 0.00),
            Self::GreenX1 => (0.15, 0.80, 0.05),
            Self::Orthochromatic => (0.00, 0.60, 0.40),
        }
    }
}
