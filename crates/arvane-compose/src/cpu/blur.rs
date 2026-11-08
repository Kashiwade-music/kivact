//! Role: Gaussian blur of premultiplied pixels by repeated halving.

use super::surface::{Pixmap, Premul};

/// Halvings allowed for one axis.
const MAX_LEVELS: u32 = 6;

/// A premultiplied float plane used while a blur runs.
#[derive(Clone, Debug)]
struct Plane {
    /// Width in px.
    w: usize,
    /// Height in px.
    h: usize,
    /// Row-major texels.
    px: Vec<Premul>,
}

impl Plane {
    /// The plane of `src`.
    fn from_pixmap(src: &Pixmap) -> Self {
        let (w, h) = src.size();
        let (w, h) = (w as usize, h as usize);
        let mut px = Vec::with_capacity(w * h);
        for y in 0..h {
            for x in 0..w {
                px.push(src.get(x as u32, y as u32));
            }
        }
        Plane { w, h, px }
    }

    /// The pixmap of this plane, quantized.
    fn to_pixmap(&self) -> Pixmap {
        let mut out = Pixmap::new(self.w as u32, self.h as u32);
        for y in 0..self.h {
            for x in 0..self.w {
                out.set(x as u32, y as u32, self.px[y * self.w + x]);
            }
        }
        out
    }

    /// The texel at (`x`, `y`), zero outside the plane.
    fn at(&self, x: isize, y: isize) -> Premul {
        if x < 0 || y < 0 || x as usize >= self.w || y as usize >= self.h {
            return [0.0; 4];
        }
        self.px[y as usize * self.w + x as usize]
    }

    /// Halve the width: each texel is the mean of two texels (zero outside).
    fn shrink_x(&self) -> Plane {
        let w = self.w.div_ceil(2);
        let mut px = Vec::with_capacity(w * self.h);
        for y in 0..self.h {
            for x in 0..w {
                let a = self.at(2 * x as isize, y as isize);
                let b = self.at(2 * x as isize + 1, y as isize);
                px.push(std::array::from_fn(|i| (a[i] + b[i]) * 0.5));
            }
        }
        Plane { w, h: self.h, px }
    }

    /// Halve the height: each texel is the mean of two texels (zero outside).
    fn shrink_y(&self) -> Plane {
        let h = self.h.div_ceil(2);
        let mut px = Vec::with_capacity(self.w * h);
        for y in 0..h {
            for x in 0..self.w {
                let a = self.at(x as isize, 2 * y as isize);
                let b = self.at(x as isize, 2 * y as isize + 1);
                px.push(std::array::from_fn(|i| (a[i] + b[i]) * 0.5));
            }
        }
        Plane { w: self.w, h, px }
    }

    /// Double the width back to `w`, bilinear at ((x + 0.5) / 2 − 0.5) in the halved texels.
    fn expand_x(&self, w: usize) -> Plane {
        let mut px = Vec::with_capacity(w * self.h);
        for y in 0..self.h {
            for x in 0..w {
                let u = (x as f32 + 0.5) / 2.0 - 0.5;
                let i0 = u.floor();
                let t = u - i0;
                let i0 = i0 as isize;
                let p0 = self.at(i0, y as isize);
                let p1 = self.at(i0 + 1, y as isize);
                px.push(std::array::from_fn(|i| (1.0 - t) * p0[i] + t * p1[i]));
            }
        }
        Plane { w, h: self.h, px }
    }

    /// Double the height back to `h`, bilinear as `expand_x`.
    fn expand_y(&self, h: usize) -> Plane {
        let mut px = Vec::with_capacity(self.w * h);
        for y in 0..h {
            let v = (y as f32 + 0.5) / 2.0 - 0.5;
            let j0 = v.floor();
            let t = v - j0;
            for x in 0..self.w {
                let p0 = self.at(x as isize, j0 as isize);
                let p1 = self.at(x as isize, j0 as isize + 1);
                px.push(std::array::from_fn(|i| (1.0 - t) * p0[i] + t * p1[i]));
            }
        }
        Plane { w: self.w, h, px }
    }

    /// Convolve along x with the normalized Gaussian of standard deviation `sigma` (px).
    fn convolve_x(&self, sigma: f64) -> Plane {
        let weights = kernel(sigma);
        let r = (weights.len() / 2) as isize;
        let mut px = Vec::with_capacity(self.w * self.h);
        for y in 0..self.h {
            for x in 0..self.w {
                let mut acc = [0.0_f32; 4];
                for (k, wt) in weights.iter().enumerate() {
                    let p = self.at(x as isize + k as isize - r, y as isize);
                    for c in 0..4 {
                        acc[c] += wt * p[c];
                    }
                }
                px.push(acc);
            }
        }
        Plane {
            w: self.w,
            h: self.h,
            px,
        }
    }

    /// Convolve along y, as `convolve_x`.
    fn convolve_y(&self, sigma: f64) -> Plane {
        let weights = kernel(sigma);
        let r = (weights.len() / 2) as isize;
        let mut px = Vec::with_capacity(self.w * self.h);
        for y in 0..self.h {
            for x in 0..self.w {
                let mut acc = [0.0_f32; 4];
                for (k, wt) in weights.iter().enumerate() {
                    let p = self.at(x as isize, y as isize + k as isize - r);
                    for c in 0..4 {
                        acc[c] += wt * p[c];
                    }
                }
                px.push(acc);
            }
        }
        Plane {
            w: self.w,
            h: self.h,
            px,
        }
    }
}

/// The number of halvings for a standard deviation `sigma` (px): while sigma / 2^k > 4, up to six.
fn levels(sigma: f64) -> u32 {
    let mut k = 0;
    while k < MAX_LEVELS && sigma / f64::from(1_u32 << k) > 4.0 {
        k += 1;
    }
    k
}

/// The normalized kernel w_i = exp(−i² / (2σ²)) over [−r, r] with r = ⌈3σ⌉.
fn kernel(sigma: f64) -> Vec<f32> {
    let r = (3.0 * sigma).ceil() as i64;
    let raw: Vec<f64> = (-r..=r)
        .map(|i| {
            let i = i as f64;
            (-(i * i) / (2.0 * sigma * sigma)).exp()
        })
        .collect();
    let total: f64 = raw.iter().sum();
    raw.into_iter().map(|w| (w / total) as f32).collect()
}

/// Gaussian blur of `src` with standard deviations `sigma_x` and `sigma_y` (px).
///
/// Each axis is halved `levels` times, convolved at the reduced size, then
/// doubled back. An axis below 0.5 px is left as it is.
pub(crate) fn gaussian(src: &Pixmap, sigma_x: f64, sigma_y: f64) -> Pixmap {
    let (kx, ky) = (levels(sigma_x), levels(sigma_y));
    let mut plane = Plane::from_pixmap(src);
    let mut widths = Vec::with_capacity(kx as usize);
    let mut heights = Vec::with_capacity(ky as usize);
    for _ in 0..kx {
        widths.push(plane.w);
        plane = plane.shrink_x();
    }
    for _ in 0..ky {
        heights.push(plane.h);
        plane = plane.shrink_y();
    }
    if sigma_x >= 0.5 {
        plane = plane.convolve_x(sigma_x / f64::from(1_u32 << kx));
    }
    if sigma_y >= 0.5 {
        plane = plane.convolve_y(sigma_y / f64::from(1_u32 << ky));
    }
    for &w in widths.iter().rev() {
        plane = plane.expand_x(w);
    }
    for &h in heights.iter().rev() {
        plane = plane.expand_y(h);
    }
    plane.to_pixmap()
}
