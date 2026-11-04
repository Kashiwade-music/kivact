//! Role: Adapter-neutral color values (text styles are in `text`).
//! This module contains plain data usable on any thread.

/// Straight-alpha color, components in `0.0..=1.0`.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
/// Rgba.
pub struct Rgba {
    /// R.
    pub r: f32,
    /// G.
    pub g: f32,
    /// B.
    pub b: f32,
    /// A.
    pub a: f32,
}

impl Rgba {
    /// Fully transparent black.
    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    /// An RGB hex color and an alpha clamped to `0..=1`.
    pub fn from_hex(hex: u32, alpha: f32) -> Self {
        Self {
            r: ((hex >> 16) & 0xff) as f32 / 255.0,
            g: ((hex >> 8) & 0xff) as f32 / 255.0,
            b: (hex & 0xff) as f32 / 255.0,
            a: alpha.clamp(0.0, 1.0),
        }
    }

    /// With alpha mul.
    pub fn with_alpha_mul(self, factor: f32) -> Self {
        Rgba {
            a: (self.a * factor).clamp(0.0, 1.0),
            ..self
        }
    }
}

/// Convert premultiplied RGBA8 pixels to straight alpha in place.
///
/// RGB channels are rounded to nearest with integer arithmetic and clamped to
/// 255. Alpha is preserved; zero-alpha pixels become transparent black. A
/// trailing incomplete pixel is left unchanged, matching frame and GPU readback.
pub fn unpremultiply(rgba: &mut [u8]) {
    for pixel in rgba.as_chunks_mut::<4>().0 {
        let alpha = u32::from(pixel[3]);
        match alpha {
            0 => pixel[..3].fill(0),
            255 => {}
            _ => {
                for channel in &mut pixel[..3] {
                    *channel = ((u32::from(*channel) * 255 + alpha / 2) / alpha).min(255) as u8;
                }
            }
        }
    }
}

impl Rgba {
    /// Clamp straight-alpha components to zero through one; non-finite is zero.
    pub fn sanitize(self) -> Self {
        let unit = |v: f32| {
            if v.is_finite() {
                v.clamp(0.0, 1.0)
            } else {
                0.0
            }
        };
        Self {
            r: unit(self.r),
            g: unit(self.g),
            b: unit(self.b),
            a: unit(self.a),
        }
    }
}
