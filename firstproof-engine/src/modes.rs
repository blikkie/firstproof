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
}

#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum EmulsionFormat {
    #[default]
    Format35mm,
    Format120,
    Format4x5,
}


impl EmulsionFormat {
    #[inline]
    pub fn id_number(&self) -> i32 {
        match self {
            Self::Format35mm => 0,
            Self::Format120 => 1,
            Self::Format4x5 => 2,
        }
    }

    pub fn from_id(id: i32) -> Self {
        match id {
            1 => Self::Format120,
            2 => Self::Format4x5,
            _ => Self::Format35mm,
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
