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

/// Computes CIEDE2000 color difference ($\Delta E_{00}$) between two Lab colors.
/// Standard parametric weights $k_L = k_C = k_H = 1.0$.
pub fn ciede2000(lab1: Lab, lab2: Lab) -> f32 {
    let l1 = lab1.l;
    let a1 = lab1.a;
    let b1 = lab1.b;

    let l2 = lab2.l;
    let a2 = lab2.a;
    let b2 = lab2.b;

    let c1 = (a1 * a1 + b1 * b1).sqrt();
    let c2 = (a2 * a2 + b2 * b2).sqrt();
    let c_bar = (c1 + c2) * 0.5;

    let c_bar_7 = c_bar.powi(7);
    let g = 0.5 * (1.0 - (c_bar_7 / (c_bar_7 + 25.0f32.powi(7))).sqrt());

    let a1_prime = (1.0 + g) * a1;
    let a2_prime = (1.0 + g) * a2;

    let c1_prime = (a1_prime * a1_prime + b1 * b1).sqrt();
    let c2_prime = (a2_prime * a2_prime + b2 * b2).sqrt();

    let h1_prime = {
        let mut deg = b1.atan2(a1_prime).to_degrees();
        if deg < 0.0 { deg += 360.0; }
        deg
    };

    let h2_prime = {
        let mut deg = b2.atan2(a2_prime).to_degrees();
        if deg < 0.0 { deg += 360.0; }
        deg
    };

    let delta_l_prime = l2 - l1;
    let delta_c_prime = c2_prime - c1_prime;

    let delta_h_prime = if c1_prime * c2_prime == 0.0 {
        0.0
    } else if (h2_prime - h1_prime).abs() <= 180.0 {
        h2_prime - h1_prime
    } else if h2_prime - h1_prime > 180.0 {
        h2_prime - h1_prime - 360.0
    } else {
        h2_prime - h1_prime + 360.0
    };

    let delta_big_h_prime = 2.0 * (c1_prime * c2_prime).sqrt() * (delta_h_prime.to_radians() * 0.5).sin();

    let l_bar_prime = (l1 + l2) * 0.5;
    let c_bar_prime = (c1_prime + c2_prime) * 0.5;

    let h_bar_prime = if c1_prime * c2_prime == 0.0 {
        h1_prime + h2_prime
    } else if (h1_prime - h2_prime).abs() <= 180.0 {
        (h1_prime + h2_prime) * 0.5
    } else if h1_prime + h2_prime < 360.0 {
        (h1_prime + h2_prime + 360.0) * 0.5
    } else {
        (h1_prime + h2_prime - 360.0) * 0.5
    };

    let t = 1.0
        - 0.17 * (h_bar_prime - 30.0).to_radians().cos()
        + 0.24 * (2.0 * h_bar_prime).to_radians().cos()
        + 0.32 * (3.0 * h_bar_prime + 6.0).to_radians().cos()
        - 0.20 * (4.0 * h_bar_prime - 63.0).to_radians().cos();

    let delta_theta = 30.0 * (-(((h_bar_prime - 275.0) / 25.0).powi(2))).exp();
    let c_bar_prime_7 = c_bar_prime.powi(7);
    let r_c = 2.0 * (c_bar_prime_7 / (c_bar_prime_7 + 25.0f32.powi(7))).sqrt();

    let l_diff_sq = (l_bar_prime - 50.0).powi(2);
    let s_l = 1.0 + (0.015 * l_diff_sq) / (20.0 + l_diff_sq).sqrt();
    let s_c = 1.0 + 0.045 * c_bar_prime;
    let s_h = 1.0 + 0.015 * c_bar_prime * t;

    let r_t = -(2.0 * delta_theta.to_radians()).sin() * r_c;

    let term_l = delta_l_prime / s_l;
    let term_c = delta_c_prime / s_c;
    let term_h = delta_big_h_prime / s_h;

    (term_l * term_l + term_c * term_c + term_h * term_h + r_t * term_c * term_h).sqrt()
}
