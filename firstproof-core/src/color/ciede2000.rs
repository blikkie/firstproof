//! CIELAB and CIEDE2000 Color Difference Metric ($\Delta E_{00}$)
//!
//! Implements ISO/CIE 11664-6:2014 standard for perceptual color difference,
//! providing an objective metric to verify that color adjustments do not cause
//! perceptible shifts to neutral tones, skin tones, or memory colors.

use super::Color3;

/// CIELAB color representation ($L^*, a^*, b^*$)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lab {
    pub l: f32, // Luminance [0.0, 100.0]
    pub a: f32, // Green (-) to Red (+)
    pub b: f32, // Blue (-) to Yellow (+)
}

impl Lab {
    pub fn new(l: f32, a: f32, b: f32) -> Self {
        Lab { l, a, b }
    }

    /// Computes CIELAB hue angle in degrees [0, 360)
    pub fn hue_angle_deg(&self) -> f32 {
        let mut deg = self.b.atan2(self.a).to_degrees();
        if deg < 0.0 {
            deg += 360.0;
        }
        deg
    }

    /// Computes chroma $C^* = \sqrt{a^{*2} + b^{*2}}$
    pub fn chroma(&self) -> f32 {
        (self.a * self.a + self.b * self.b).sqrt()
    }
}

/// D65 standard illuminant reference white point (2-degree observer)
pub const D65_WHITE: Color3 = Color3([0.95047, 1.00000, 1.08883]);

/// Converts CIE XYZ (D65 normalized where Y=1.0 is 100 nits/diffuse white) to CIELAB
pub fn xyz_d65_to_lab(xyz: Color3) -> Lab {
    let f = |t: f32| -> f32 {
        let delta = 6.0 / 29.0;
        if t > delta * delta * delta {
            t.cbrt()
        } else {
            t / (3.0 * delta * delta) + 4.0 / 29.0
        }
    };

    let fx = f(xyz.r() / D65_WHITE.r());
    let fy = f(xyz.g() / D65_WHITE.g());
    let fz = f(xyz.b() / D65_WHITE.b());

    let l = 116.0 * fy - 16.0;
    let a = 500.0 * (fx - fy);
    let b = 200.0 * (fy - fz);

    Lab::new(l, a, b)
}

/// Converts linear sRGB [0, 1] to CIELAB (D65)
pub fn linear_srgb_to_lab(srgb: Color3) -> Lab {
    // sRGB to XYZ (D65)
    let r = srgb.r();
    let g = srgb.g();
    let b = srgb.b();

    let x = 0.4124564 * r + 0.3575761 * g + 0.1804375 * b;
    let y = 0.2126729 * r + 0.7151522 * g + 0.0721750 * b;
    let z = 0.0193339 * r + 0.1191920 * g + 0.9503041 * b;

    xyz_d65_to_lab(Color3([x, y, z]))
}

use palette::color_difference::Ciede2000;
use palette::white_point::D65;
use palette::Lab as PaletteLab;

impl From<Lab> for PaletteLab<D65, f32> {
    #[inline]
    fn from(l: Lab) -> Self {
        PaletteLab::new(l.l, l.a, l.b)
    }
}

impl From<PaletteLab<D65, f32>> for Lab {
    #[inline]
    fn from(p: PaletteLab<D65, f32>) -> Self {
        Lab::new(p.l, p.a, p.b)
    }
}

/// Computes CIEDE2000 color difference ($\Delta E_{00}$) between two Lab colors via `palette`.
/// Standard parametric weights $k_L = k_C = k_H = 1.0$.
pub fn ciede2000(lab1: Lab, lab2: Lab) -> f32 {
    let p1: PaletteLab<D65, f32> = lab1.into();
    let p2: PaletteLab<D65, f32> = lab2.into();
    p1.difference(p2)
}

