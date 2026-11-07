//! Role: Shared CPU effect helpers: blur deviations, drop shadows, color matrices.

use kivact_types::Rgba;
use kivact_types::geometry::Point;
use kivact_types::pixel::ColorMatrix;

use super::blur::gaussian;
use super::surface::{Pixmap, Premul, sample_bilinear};

/// The blur deviations of `sigma_x` and `sigma_y` px, or None when both are below half a pixel (no blur).
pub(crate) fn blur_sigmas(sigma_x: f64, sigma_y: f64) -> Option<(f64, f64)> {
    (sigma_x >= 0.5 || sigma_y >= 0.5).then_some((sigma_x, sigma_y))
}

/// The input's alpha times `color`, blurred by `sigma` px, shifted by `offset` px, with the input over it (design §8.8.2).
pub(crate) fn drop_shadow(
    input: &Pixmap,
    offset: (f64, f64),
    sigma: (f64, f64),
    color: Rgba,
) -> Pixmap {
    let (w, h) = input.size();
    let mut silhouette = Pixmap::new(w, h);
    let tint = [
        (color.r * color.a) as f32,
        (color.g * color.a) as f32,
        (color.b * color.a) as f32,
        color.a as f32,
    ];
    for y in 0..h {
        for x in 0..w {
            let a = input.get(x, y)[3];
            silhouette.set(x, y, tint.map(|v| v * a));
        }
    }
    let blurred = match blur_sigmas(sigma.0, sigma.1) {
        Some((sx, sy)) => gaussian(&silhouette, sx, sy),
        None => silhouette,
    };
    let mut out = Pixmap::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let p = Point::new(f64::from(x) + 0.5 - offset.0, f64::from(y) + 0.5 - offset.1);
            let shadow = sample_bilinear(&blurred, p);
            let top = input.get(x, y);
            let rest = 1.0 - top[3];
            let color: Premul = std::array::from_fn(|i| top[i] + shadow[i] * rest);
            out.set(x, y, color);
        }
    }
    out
}

/// Each pixel unpremultiplied, multiplied by the 4×5 matrix, clamped, and premultiplied again (design §8.8.3).
pub(crate) fn color_matrix(input: &Pixmap, matrix: &ColorMatrix) -> Pixmap {
    let (w, h) = input.size();
    let mut out = Pixmap::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let c = input.get(x, y);
            let a = c[3];
            let straight = if a > 0.0 {
                [c[0] / a, c[1] / a, c[2] / a, a]
            } else {
                [0.0; 4]
            };
            let v = matrix.apply_array(straight.map(f64::from));
            let alpha = v[3] as f32;
            let premul: Premul = [
                v[0] as f32 * alpha,
                v[1] as f32 * alpha,
                v[2] as f32 * alpha,
                alpha,
            ];
            out.set(x, y, premul);
        }
    }
    out
}
