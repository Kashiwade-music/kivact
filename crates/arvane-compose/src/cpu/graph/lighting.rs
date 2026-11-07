// Source: resvg 0.46.0, src/filter/lighting.rs (Copyright 2020 the Resvg Authors).
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Changes: the light sources are in pixels (converted by the caller); the
// input and output are the compositor's premultiplied pixmaps; the Sobel
// formulas and the light equations are those of resvg.

//! Role: feDiffuseLighting and feSpecularLighting on the CPU.

use crate::cpu::surface::Pixmap;

/// A light source in pixel space.
#[derive(Clone, Copy, Debug)]
pub(crate) enum LightPx {
    /// A light at infinity, by azimuth and elevation in degrees.
    Distant {
        /// Azimuth in degrees.
        azimuth: f32,
        /// Elevation in degrees.
        elevation: f32,
    },
    /// A light at a point.
    Point {
        /// The position in px and z.
        position: [f32; 3],
    },
    /// A light at a point aimed at another point.
    Spot {
        /// The position in px and z.
        position: [f32; 3],
        /// The point aimed at, in px and z.
        points_at: [f32; 3],
        /// The specular exponent of the cone.
        exponent: f32,
        /// The limiting cone angle in degrees.
        cone: Option<f32>,
    },
}

/// Which lighting equation applies.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Kind {
    /// Diffuse lighting with its constant.
    Diffuse {
        /// The diffuse constant.
        constant: f32,
    },
    /// Specular lighting with its constant and exponent.
    Specular {
        /// The specular constant.
        constant: f32,
        /// The specular exponent.
        exponent: f32,
    },
}

/// The surface's gradient factor and normal from the Sobel kernels (resvg's `Normal`).
#[derive(Clone, Copy)]
struct Normal {
    /// Per-axis factor of the kernel.
    factor: (f32, f32),
    /// The normal's x and y before the surface scale.
    normal: (f32, f32),
}

impl Normal {
    /// A normal from the factor and the integer Sobel sums.
    fn new(factor_x: f32, factor_y: f32, nx: i32, ny: i32) -> Self {
        Normal {
            factor: (factor_x, factor_y),
            normal: (-nx as f32, -ny as f32),
        }
    }

    /// Whether the normal is zero.
    fn is_flat(&self) -> bool {
        self.normal.0.abs() <= f32::EPSILON * 4.0 && self.normal.1.abs() <= f32::EPSILON * 4.0
    }
}

/// A 3-vector.
#[derive(Clone, Copy, Debug)]
struct V3 {
    /// x.
    x: f32,
    /// y.
    y: f32,
    /// z.
    z: f32,
}

impl V3 {
    /// The dot product.
    fn dot(self, o: V3) -> f32 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }

    /// The length.
    fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    /// The unit vector, or None for a zero vector.
    fn normalized(self) -> Option<V3> {
        let l = self.length();
        (l > f32::EPSILON * 4.0).then(|| V3 {
            x: self.x / l,
            y: self.y / l,
            z: self.z / l,
        })
    }

    /// The difference `self − o`.
    fn sub(self, o: V3) -> V3 {
        V3 {
            x: self.x - o.x,
            y: self.y - o.y,
            z: self.z - o.z,
        }
    }

    /// The sum `self + o`.
    fn add(self, o: V3) -> V3 {
        V3 {
            x: self.x + o.x,
            y: self.y + o.y,
            z: self.z + o.z,
        }
    }
}

/// Lights `input`'s alpha surface with `kind`, `surface_scale`, the lighting `color` (0..=255 per channel).
pub(crate) fn apply(
    kind: Kind,
    surface_scale: f32,
    color: [f32; 3],
    light: LightPx,
    input: &Pixmap,
) -> Pixmap {
    let (w, h) = input.size();
    let mut out = Pixmap::new(w, h);
    if w < 3 || h < 3 {
        return out;
    }
    let alpha = |x: i64, y: i64| -> i32 {
        i32::from(input.data[(y as usize * w as usize + x as usize) * 4 + 3])
    };
    let mut light_vector = match light {
        LightPx::Distant { azimuth, elevation } => {
            let (az, el) = (azimuth.to_radians(), elevation.to_radians());
            V3 {
                x: az.cos() * el.cos(),
                y: az.sin() * el.cos(),
                z: el.sin(),
            }
        }
        _ => V3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
    };
    for y in 0..h {
        for x in 0..w {
            let normal = sobel(&alpha, i64::from(x), i64::from(y), w, h);
            if let Some(pos) = light_position(light) {
                let nz = alpha(i64::from(x), i64::from(y)) as f32 / 255.0 * surface_scale;
                let v = V3 {
                    x: pos[0],
                    y: pos[1],
                    z: pos[2],
                }
                .sub(V3 {
                    x: x as f32,
                    y: y as f32,
                    z: nz,
                });
                light_vector = v.normalized().unwrap_or(v);
            }
            let light_color = light_color(light, color, light_vector);
            let factor = light_factor(kind, surface_scale, normal, light_vector);
            let compute = |c: f32| ((c * factor).clamp(0.0, 255.0) + 0.5) as u8;
            let r = compute(light_color[0]);
            let g = compute(light_color[1]);
            let b = compute(light_color[2]);
            let a = match kind {
                Kind::Diffuse { .. } => 255,
                Kind::Specular { .. } => r.max(g).max(b),
            };
            let i = (y as usize * w as usize + x as usize) * 4;
            out.data[i..i + 4].copy_from_slice(&[r, g, b, a]);
        }
    }
    out
}

/// The position of a point or spot light in px and z, None for a distant light.
fn light_position(light: LightPx) -> Option<[f32; 3]> {
    match light {
        LightPx::Distant { .. } => None,
        LightPx::Point { position } | LightPx::Spot { position, .. } => Some(position),
    }
}

/// The light's color at the surface, from the lighting color and the spot cone.
fn light_color(light: LightPx, color: [f32; 3], light_vector: V3) -> [f32; 3] {
    let LightPx::Spot {
        position,
        points_at,
        exponent,
        cone,
    } = light
    else {
        return color;
    };
    let origin = V3 {
        x: position[0],
        y: position[1],
        z: position[2],
    };
    let target = V3 {
        x: points_at[0],
        y: points_at[1],
        z: points_at[2],
    };
    let direction = target.sub(origin);
    let direction = direction.normalized().unwrap_or(direction);
    let minus_l_dot_s = -light_vector.dot(direction);
    if minus_l_dot_s <= 0.0 {
        return [0.0; 3];
    }
    if let Some(angle) = cone
        && minus_l_dot_s < angle.to_radians().cos()
    {
        return [0.0; 3];
    }
    let factor = minus_l_dot_s.powf(exponent);
    color.map(|c| ((c * factor).clamp(0.0, 255.0) + 0.5).floor())
}

/// The lighting factor of the surface normal against the light vector.
fn light_factor(kind: Kind, surface_scale: f32, normal: Normal, light_vector: V3) -> f32 {
    match kind {
        Kind::Diffuse { constant } => {
            let k = if normal.is_flat() {
                light_vector.z
            } else {
                let n = surface_normal(surface_scale, normal);
                n.dot(light_vector) / n.length()
            };
            constant * k
        }
        Kind::Specular { constant, exponent } => {
            let h = light_vector.add(V3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            });
            let h_length = h.length();
            if h_length <= f32::EPSILON * 4.0 {
                return 0.0;
            }
            let n_dot_h = if normal.is_flat() {
                h.z / h_length
            } else {
                let n = surface_normal(surface_scale, normal);
                n.dot(h) / n.length() / h_length
            };
            let k = if (exponent - 1.0).abs() <= f32::EPSILON * 4.0 {
                n_dot_h
            } else {
                n_dot_h.powf(exponent)
            };
            constant * k
        }
    }
}

/// The unscaled surface vector (nx · factor.x, ny · factor.y, 1) with the surface scale applied.
fn surface_normal(surface_scale: f32, normal: Normal) -> V3 {
    let s = surface_scale / 255.0;
    V3 {
        x: normal.normal.0 * s * normal.factor.0,
        y: normal.normal.1 * s * normal.factor.1,
        z: 1.0,
    }
}

/// The Sobel normal at (`x`, `y`), with the kernel of its place on the edge or in the interior.
fn sobel(a: &impl Fn(i64, i64) -> i32, x: i64, y: i64, w: u32, h: u32) -> Normal {
    let (wi, hi) = (i64::from(w), i64::from(h));
    let third = 1.0 / 3.0;
    let half = 1.0 / 2.0;
    let quarter = 1.0 / 4.0;
    let two_thirds = 2.0 / 3.0;
    let left_edge = x == 0;
    let right_edge = x == wi - 1;
    let top_edge = y == 0;
    let bottom_edge = y == hi - 1;
    match (left_edge, right_edge, top_edge, bottom_edge) {
        (true, _, true, _) => {
            let (c, r, b, br) = (a(0, 0), a(1, 0), a(0, 1), a(1, 1));
            Normal::new(
                two_thirds,
                two_thirds,
                -2 * c + 2 * r - b + br,
                -2 * c - r + 2 * b + br,
            )
        }
        (_, true, true, _) => {
            let (l, c, bl, b) = (a(x - 1, 0), a(x, 0), a(x - 1, 1), a(x, 1));
            Normal::new(
                two_thirds,
                two_thirds,
                -2 * l + 2 * c - bl + b,
                -l - 2 * c + bl + 2 * b,
            )
        }
        (true, _, _, true) => {
            let (t, tr, c, r) = (a(0, hi - 2), a(1, hi - 2), a(0, hi - 1), a(1, hi - 1));
            Normal::new(
                two_thirds,
                two_thirds,
                -t + tr - 2 * c + 2 * r,
                -2 * t - tr + 2 * c + r,
            )
        }
        (_, true, _, true) => {
            let (tl, t, l, c) = (
                a(wi - 2, hi - 2),
                a(wi - 1, hi - 2),
                a(wi - 2, hi - 1),
                a(wi - 1, hi - 1),
            );
            Normal::new(
                two_thirds,
                two_thirds,
                -tl + t - 2 * l + 2 * c,
                -tl - 2 * t + l + 2 * c,
            )
        }
        (_, _, true, _) => {
            let (l, c, r) = (a(x - 1, 0), a(x, 0), a(x + 1, 0));
            let (bl, b, br) = (a(x - 1, 1), a(x, 1), a(x + 1, 1));
            Normal::new(
                third,
                half,
                -2 * l + 2 * r - bl + br,
                -l - 2 * c - r + bl + 2 * b + br,
            )
        }
        (_, _, _, true) => {
            let (tl, t, tr) = (a(x - 1, hi - 2), a(x, hi - 2), a(x + 1, hi - 2));
            let (l, c, r) = (a(x - 1, hi - 1), a(x, hi - 1), a(x + 1, hi - 1));
            Normal::new(
                third,
                half,
                -tl + tr - 2 * l + 2 * r,
                -tl - 2 * t - tr + l + 2 * c + r,
            )
        }
        (true, _, _, _) => {
            let (t, tr, c, r) = (a(0, y - 1), a(1, y - 1), a(0, y), a(1, y));
            let (b, br) = (a(0, y + 1), a(1, y + 1));
            Normal::new(
                half,
                third,
                -t + tr - 2 * c + 2 * r - b + br,
                -2 * t - tr + 2 * b + br,
            )
        }
        (_, true, _, _) => {
            let (tl, t, l, c) = (
                a(wi - 2, y - 1),
                a(wi - 1, y - 1),
                a(wi - 2, y),
                a(wi - 1, y),
            );
            let (bl, b) = (a(wi - 2, y + 1), a(wi - 1, y + 1));
            Normal::new(
                half,
                third,
                -tl + t - 2 * l + 2 * c - bl + b,
                -tl - 2 * t + bl + 2 * b,
            )
        }
        _ => {
            let (tl, t, tr) = (a(x - 1, y - 1), a(x, y - 1), a(x + 1, y - 1));
            let (l, r) = (a(x - 1, y), a(x + 1, y));
            let (bl, b, br) = (a(x - 1, y + 1), a(x, y + 1), a(x + 1, y + 1));
            Normal::new(
                quarter,
                quarter,
                -tl + tr - 2 * l + 2 * r - bl + br,
                -tl - 2 * t - tr + bl + 2 * b + br,
            )
        }
    }
}
