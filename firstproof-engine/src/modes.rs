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
pub enum EmulsionArchetype {
    #[default]
    None,
    RochesterPortrait400,  // C-41 Color Negative (Rochester NY)
    AshigaraChrome50,      // E-6 Color Reversal (Minamiashigara, Japan)
    RochesterK14Chrome64,  // K-14 Non-Substantive Reversal (Rochester NY)
    RochesterCinema500,    // ECN-2 Tungsten Cinema Negative (Rochester NY)
    RochesterDaylight200,  // C-41 Golden Daylight Color Negative (Rochester NY)
    MobberleySilver400,    // Panchromatic Silver Halide B&W Negative (Mobberley, UK)
    CambridgeDiffusion100, // Instant Dye Diffusion Reversal (Cambridge MA)
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
        }
    }

    pub fn from_id(id: i32) -> Self {
        match id {
            1 => Self::RochesterPortrait400,
            2 => Self::AshigaraChrome50,
            3 => Self::RochesterK14Chrome64,
            4 => Self::RochesterCinema500,
            5 => Self::RochesterDaylight200,
            6 => Self::MobberleySilver400,
            7 => Self::CambridgeDiffusion100,
            _ => Self::None,
        }
    }

    #[inline]
    pub fn is_monochrome(&self) -> bool {
        matches!(self, Self::MobberleySilver400)
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
