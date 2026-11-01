//! Role: Vector paint operations.
//!
//! The animated components of a paint (`TransitionProp::Paint`): opacity,
//! a solid background, the border colors (one per side), the canonical
//! transform (its parts: translate, rotate, skew, scale, and its origin),
//! the first two shadows and the filter's amounts. Everything else
//! (gradients, images, shapes, the z-index, the filter's order) takes the
//! target's value at once. The same vector moves a paint whatever changed
//! its target (a signal or style write, the host's hover, press or focus:
//! `model::state_motion`).

use crate::{Edges, Effects, Fill, FilterOp, Len, PaintProps, Point, Rgba, Shadow};

/// Shadows animated (more change at once).
const SHADOWS: usize = 2;
const BG: usize = 1;
const BORDER: usize = BG + 4;
const TRANSFORM: usize = BORDER + 16;
/// The transform's parts (`Transform::components`), then its origin's.
const SHADOW: usize = TRANSFORM + 14;
const FILTER: usize = SHADOW + 8 * SHADOWS;
/// brightness, contrast, saturate, grayscale, sepia, hue rotation, invert,
/// opacity, blur, drop shadow (x, y, blur, r, g, b, a).
const FILTER_LEN: usize = 16;
const LEN: usize = FILTER + FILTER_LEN;

/// The filter amounts that change nothing.
const IDENTITY: [f32; FILTER_LEN] = [
    1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaintVec([f32; LEN]);

fn rgba(c: Rgba) -> [f32; 4] {
    [c.r, c.g, c.b, c.a]
}

fn color(v: &[f32]) -> Rgba {
    Rgba::new(v[0], v[1], v[2], v[3].clamp(0.0, 1.0))
}

/// The filter of `p` as amounts in the canonical order (a function given
/// twice composes: amounts multiply, angles and blurs add).
fn filter_amounts(p: &PaintProps) -> [f32; FILTER_LEN] {
    let mut v = IDENTITY;
    for op in p.effects.iter().flat_map(|e| &e.filter) {
        match *op {
            FilterOp::Brightness(a) => v[0] *= a,
            FilterOp::Contrast(a) => v[1] *= a,
            FilterOp::Saturate(a) => v[2] *= a,
            FilterOp::Grayscale(a) => v[3] = a,
            FilterOp::Sepia(a) => v[4] = a,
            FilterOp::HueRotate(a) => v[5] += a,
            FilterOp::Invert(a) => v[6] = a,
            FilterOp::Opacity(a) => v[7] *= a,
            FilterOp::Blur(a) => v[8] += a,
            FilterOp::DropShadow {
                offset,
                blur,
                color,
            } => {
                v[9..12].copy_from_slice(&[offset.x, offset.y, blur]);
                v[12..16].copy_from_slice(&rgba(color));
            }
        }
    }
    v
}

/// The filter of the amounts `v`, in the canonical order, leaving out the
/// functions that change nothing.
fn filter_of(v: &[f32]) -> Vec<FilterOp> {
    let mut out = Vec::new();
    let mut push = |i: usize, op: fn(f32) -> FilterOp| {
        if (v[i] - IDENTITY[i]).abs() > 1e-4 {
            out.push(op(v[i]));
        }
    };
    push(0, FilterOp::Brightness);
    push(1, FilterOp::Contrast);
    push(2, FilterOp::Saturate);
    push(3, FilterOp::Grayscale);
    push(4, FilterOp::Sepia);
    push(5, FilterOp::HueRotate);
    push(6, FilterOp::Invert);
    push(7, FilterOp::Opacity);
    push(8, FilterOp::Blur);
    if v[15] > 1e-3 {
        out.push(FilterOp::DropShadow {
            offset: Point::new(v[9], v[10]),
            blur: v[11].max(0.0),
            color: color(&v[12..16]),
        });
    }
    out
}

impl PaintVec {
    pub fn of(p: &PaintProps) -> PaintVec {
        let mut v = [0.0; LEN];
        v[0] = p.opacity;
        if let Some(Fill::Solid(c)) = &p.background {
            v[BG..BORDER].copy_from_slice(&rgba(*c));
        }
        if let Some(b) = &p.border {
            let c = b.colors;
            for (i, side) in [c.top, c.right, c.bottom, c.left].into_iter().enumerate() {
                v[BORDER + 4 * i..BORDER + 4 * i + 4].copy_from_slice(&rgba(side));
            }
        }
        let t = p.transform;
        v[TRANSFORM..TRANSFORM + 10].copy_from_slice(&[
            t.translate.x,
            t.translate.y,
            t.per_size[0],
            t.per_size[1],
            t.per_size[2],
            t.per_size[3],
            t.rotate,
            t.skew_x,
            t.scale.x,
            t.scale.y,
        ]);
        let [x, y] = t.origin;
        v[TRANSFORM + 10..SHADOW].copy_from_slice(&[x.px, x.pct, y.px, y.pct]);
        for (i, s) in p.shadows.iter().take(SHADOWS).enumerate() {
            let at = SHADOW + 8 * i;
            v[at..at + 4].copy_from_slice(&[s.offset.x, s.offset.y, s.blur, s.spread]);
            v[at + 4..at + 8].copy_from_slice(&rgba(s.color));
        }
        v[FILTER..].copy_from_slice(&filter_amounts(p));
        PaintVec(v)
    }

    /// Put the animated components into `p` (the target paint).
    pub fn apply(&self, p: &mut PaintProps) {
        let v = &self.0;
        p.opacity = v[0].clamp(0.0, 1.0);
        let bg = color(&v[BG..BORDER]);
        match &mut p.background {
            Some(Fill::Solid(c)) => *c = bg,
            None if bg.a > 0.001 => p.background = Some(Fill::Solid(bg)),
            _ => {}
        }
        if let Some(b) = &mut p.border {
            let side = |i: usize| color(&v[BORDER + 4 * i..BORDER + 4 * i + 4]);
            b.colors = Edges {
                top: side(0),
                right: side(1),
                bottom: side(2),
                left: side(3),
            };
        }
        let parts: [f32; 10] = std::array::from_fn(|i| v[TRANSFORM + i]);
        let o = &v[TRANSFORM + 10..SHADOW];
        p.transform = crate::ElementTransform {
            translate: Point::new(parts[0], parts[1]),
            per_size: [parts[2], parts[3], parts[4], parts[5]],
            rotate: parts[6],
            skew_x: parts[7],
            scale: Point::new(parts[8], parts[9]),
            origin: [
                Len {
                    px: o[0],
                    pct: o[1],
                },
                Len {
                    px: o[2],
                    pct: o[3],
                },
            ],
        };
        for i in 0..SHADOWS {
            let at = SHADOW + 8 * i;
            let c = color(&v[at + 4..at + 8]);
            if p.shadows.len() == i && c.a > 0.001 {
                p.shadows.push(Shadow::default());
            }
            if let Some(s) = p.shadows.get_mut(i) {
                s.offset = Point::new(v[at], v[at + 1]);
                s.blur = v[at + 2].max(0.0);
                s.spread = v[at + 3];
                s.color = c;
            }
        }
        // The target's own filter, unless the amounts are on their way.
        let amounts = &v[FILTER..];
        let target = filter_amounts(p);
        if amounts
            .iter()
            .zip(target)
            .any(|(a, b)| (a - b).abs() > 1e-4)
        {
            p.effects.get_or_insert_with(Box::<Effects>::default).filter = filter_of(amounts);
        }
    }
}

impl PaintVec {
    pub const COMPONENTS: usize = LEN;
    pub fn components(&self) -> &[f32] {
        &self.0
    }
    pub fn from_components(c: &[f32]) -> Self {
        let mut v = [0.0; LEN];
        for (d, s) in v.iter_mut().zip(c) {
            *d = *s;
        }
        PaintVec(v)
    }
}
