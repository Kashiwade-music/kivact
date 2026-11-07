//! Role: Per-property transition values: the numbers a paint transition interpolates.
//!
//! Each property is a list of numbers plus the shape they are written into
//! (`PropBasis`). Colors are premultiplied Oklab components (`L·α, a·α, b·α, α`).
use crate::color_space::{ColorSpace, SpaceColor};
use crate::motion::TransitionProp;
use crate::paint::{
    CornerRadius, Corners, ElementTransform, Fill, FilterOp, Len, PaintProps, Shadow,
};
use crate::{Edges, Point, Rgba};

/// One property's interpolation: the shape the numbers are written into
/// and the numbers at both ends. Fixed when a track starts.
#[derive(Clone, Debug, PartialEq)]
pub struct PropInterp {
    /// The non-numeric shape the numbers are written into.
    pub basis: PropBasis,
    /// The numbers at the start.
    pub from: Vec<f64>,
    /// The numbers at the end.
    pub to: Vec<f64>,
}

/// The non-numeric shape of an interpolated value.
#[derive(Clone, Debug, PartialEq)]
pub enum PropBasis {
    /// Numbers written into the property of the same name (Opacity, Transform,
    /// Background, BorderColor, BorderWidth, Radius, TextColor).
    Plain(TransitionProp),
    /// The padded shadow list (as long as the longer side; each entry's inset
    /// and its non-numeric fields fixed); numbers replace dx, dy, blur, spread, color.
    Shadows(Vec<Shadow>),
    /// The filter list being interpolated (function kinds and order from the
    /// non-empty side; Graph and Shader ops as they are); numbers replace each
    /// op's numeric arguments in order. `backdrop` picks the property.
    Filters {
        /// True for `backdrop-filter`.
        backdrop: bool,
        /// The functions in order.
        ops: Vec<FilterOp>,
    },
}

impl PropBasis {
    /// Whether two bases interpolate the same components: the same variant
    /// and prop; for Shadows the same length and inset at each index; for
    /// Filters the same `backdrop`, length, function kind at each index and
    /// `==` Graph and Shader ops. The numbers in the basis are not compared.
    pub fn same_shape(&self, other: &PropBasis) -> bool {
        match (self, other) {
            (PropBasis::Plain(a), PropBasis::Plain(b)) => a == b,
            (PropBasis::Shadows(a), PropBasis::Shadows(b)) => {
                a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.inset == y.inset)
            }
            (
                PropBasis::Filters {
                    backdrop: ba,
                    ops: a,
                },
                PropBasis::Filters {
                    backdrop: bb,
                    ops: b,
                },
            ) => {
                ba == bb
                    && a.len() == b.len()
                    && a.iter().zip(b).all(|(x, y)| same_filter_kind(x, y))
            }
            _ => false,
        }
    }
}

/// The interpolation of `prop` between two paints, or None when they cannot
/// be interpolated (the property switches at once, as CSS discrete interpolation).
/// `All` and `Size` are not paint properties and give None.
pub fn prop_interp(
    prop: TransitionProp,
    from: &PaintProps,
    to: &PaintProps,
    text: (Rgba, Rgba),
) -> Option<PropInterp> {
    use TransitionProp as P;
    match prop {
        P::Opacity => Some(plain(prop, vec![from.opacity], vec![to.opacity])),
        P::Background => {
            let a = solid(from.background.as_ref())?;
            let b = solid(to.background.as_ref())?;
            Some(plain(prop, color4(a).to_vec(), color4(b).to_vec()))
        }
        P::BorderColor => Some(plain(
            prop,
            edge_colors(from.border.map(|b| b.colors)),
            edge_colors(to.border.map(|b| b.colors)),
        )),
        P::BorderWidth => Some(plain(
            prop,
            edge_widths(from.border.map(|b| b.widths)),
            edge_widths(to.border.map(|b| b.widths)),
        )),
        P::Radius => Some(plain(
            prop,
            radius_numbers(&from.radius),
            radius_numbers(&to.radius),
        )),
        P::Transform => Some(plain(
            prop,
            transform_numbers(&from.transform),
            transform_numbers(&to.transform),
        )),
        P::Shadow => shadow_interp(&from.shadows, &to.shadows),
        P::Filter => filter_interp(false, &ops_of(from, false), &ops_of(to, false)),
        P::BackdropFilter => filter_interp(true, &ops_of(from, true), &ops_of(to, true)),
        P::TextColor => Some(plain(
            prop,
            color4(text.0).to_vec(),
            color4(text.1).to_vec(),
        )),
        P::All | P::Size => None,
    }
}

/// Write `values` laid out in `basis` into `out` (a copy of the target paint):
/// the property is replaced by the basis filled with the values.
pub fn apply_components(basis: &PropBasis, values: &[f64], out: &mut PaintProps, text: &mut Rgba) {
    match basis {
        PropBasis::Plain(prop) => apply_plain(*prop, values, out, text),
        PropBasis::Shadows(list) => {
            out.shadows = list
                .iter()
                .enumerate()
                .map(|(i, s)| shadow_from(s, &values[8 * i..8 * i + 8]))
                .collect();
        }
        PropBasis::Filters { backdrop, ops } => apply_filters(*backdrop, ops, values, out),
    }
}

fn apply_plain(prop: TransitionProp, v: &[f64], out: &mut PaintProps, text: &mut Rgba) {
    use TransitionProp as P;
    match prop {
        P::Opacity => out.opacity = v[0].clamp(0.0, 1.0),
        P::Background => out.background = Some(Fill::Solid(rgba4(&v[0..4]))),
        P::BorderColor => {
            if let Some(b) = out.border.as_mut() {
                b.colors = Edges {
                    top: rgba4(&v[0..4]),
                    right: rgba4(&v[4..8]),
                    bottom: rgba4(&v[8..12]),
                    left: rgba4(&v[12..16]),
                };
            }
        }
        P::BorderWidth => {
            if let Some(b) = out.border.as_mut() {
                b.widths = Edges {
                    top: v[0],
                    right: v[1],
                    bottom: v[2],
                    left: v[3],
                };
            }
        }
        P::Radius => out.radius = corners_from(v),
        P::Transform => out.transform = transform_from(v),
        P::TextColor => *text = rgba4(&v[0..4]),
        _ => {}
    }
}

fn apply_filters(backdrop: bool, ops: &[FilterOp], v: &[f64], out: &mut PaintProps) {
    let mut at = 0;
    let filled: Vec<FilterOp> = ops.iter().map(|op| filter_fill(op, v, &mut at)).collect();
    if filled.is_empty() && out.effects.is_none() {
        return;
    }
    let effects = out.effects.get_or_insert_with(Box::default);
    if backdrop {
        effects.backdrop = filled;
    } else {
        effects.filter = filled;
    }
}

fn plain(prop: TransitionProp, from: Vec<f64>, to: Vec<f64>) -> PropInterp {
    PropInterp {
        basis: PropBasis::Plain(prop),
        from,
        to,
    }
}

/// The premultiplied Oklab components of a color.
fn color4(c: Rgba) -> [f64; 4] {
    let s = SpaceColor::from_rgba(c).convert(ColorSpace::Oklab);
    let a = s.alpha;
    [s.channels[0] * a, s.channels[1] * a, s.channels[2] * a, a]
}

/// The color of premultiplied Oklab components, in sRGB.
fn rgba4(v: &[f64]) -> Rgba {
    let a = v[3].clamp(0.0, 1.0);
    if a <= 0.0 {
        return Rgba::TRANSPARENT;
    }
    let channels = [v[0] / a, v[1] / a, v[2] / a];
    SpaceColor::new(ColorSpace::Oklab, channels, a).to_srgb()
}

/// The color of a background; no background is transparent, a layered one
/// cannot be interpolated.
fn solid(fill: Option<&Fill>) -> Option<Rgba> {
    match fill {
        None => Some(Rgba::TRANSPARENT),
        Some(Fill::Solid(c)) => Some(*c),
        Some(Fill::Layers(_)) => None,
    }
}

fn edge_colors(border: Option<Edges<Rgba>>) -> Vec<f64> {
    let e = border.unwrap_or(Edges::all(Rgba::TRANSPARENT));
    [e.top, e.right, e.bottom, e.left]
        .into_iter()
        .flat_map(color4)
        .collect()
}

fn edge_widths(widths: Option<Edges<f64>>) -> Vec<f64> {
    let e = widths.unwrap_or(Edges::all(0.0));
    vec![e.top, e.right, e.bottom, e.left]
}

fn radius_numbers(c: &Corners) -> Vec<f64> {
    c.each()
        .iter()
        .flat_map(|r| [r.x.px, r.x.pct, r.y.px, r.y.pct])
        .collect()
}

fn corners_from(v: &[f64]) -> Corners {
    let corner = |i: usize| CornerRadius {
        x: Len {
            px: v[4 * i],
            pct: v[4 * i + 1],
        },
        y: Len {
            px: v[4 * i + 2],
            pct: v[4 * i + 3],
        },
    };
    Corners {
        top_left: corner(0),
        top_right: corner(1),
        bottom_right: corner(2),
        bottom_left: corner(3),
    }
}

/// 17 transform parts, then the origin's four lengths and its z.
fn transform_numbers(t: &ElementTransform) -> Vec<f64> {
    let mut v = Vec::with_capacity(22);
    v.extend(t.translate);
    v.extend(t.per_size);
    v.extend(t.rotate);
    v.extend(t.skew);
    v.extend(t.scale);
    v.push(t.perspective_inverse);
    v.extend([
        t.origin[0].px,
        t.origin[0].pct,
        t.origin[1].px,
        t.origin[1].pct,
    ]);
    v.push(t.origin_z);
    v
}

fn transform_from(v: &[f64]) -> ElementTransform {
    let arr3 = |at: usize| [v[at], v[at + 1], v[at + 2]];
    ElementTransform {
        translate: arr3(0),
        per_size: [v[3], v[4], v[5], v[6]],
        rotate: arr3(7),
        skew: arr3(10),
        scale: arr3(13),
        perspective_inverse: v[16],
        origin: [
            Len {
                px: v[17],
                pct: v[18],
            },
            Len {
                px: v[19],
                pct: v[20],
            },
        ],
        origin_z: v[21],
    }
}

fn shadow_numbers(s: &Shadow) -> Vec<f64> {
    let mut v = vec![s.offset.x, s.offset.y, s.blur, s.spread];
    v.extend(color4(s.color));
    v
}

fn shadow_from(basis: &Shadow, v: &[f64]) -> Shadow {
    Shadow {
        offset: Point { x: v[0], y: v[1] },
        blur: v[2].max(0.0),
        spread: v[3],
        color: rgba4(&v[4..8]),
        inset: basis.inset,
    }
}

/// A shadow that stands in for a missing one: transparent, zero, with the
/// other side's inset.
fn padding(inset: bool) -> Shadow {
    Shadow {
        color: Rgba::TRANSPARENT,
        inset,
        ..Shadow::default()
    }
}

fn shadow_interp(a: &[Shadow], b: &[Shadow]) -> Option<PropInterp> {
    let n = a.len().max(b.len());
    let mut basis = Vec::with_capacity(n);
    let mut from = Vec::with_capacity(8 * n);
    let mut to = Vec::with_capacity(8 * n);
    for i in 0..n {
        let (x, y) = (a.get(i), b.get(i));
        let inset = match (x, y) {
            (Some(x), Some(y)) if x.inset != y.inset => return None,
            (Some(x), _) => x.inset,
            (None, Some(y)) => y.inset,
            (None, None) => false,
        };
        let entry = x.or(y).copied().unwrap_or_else(|| padding(inset));
        basis.push(entry);
        from.extend(shadow_numbers(
            &x.copied().unwrap_or_else(|| padding(inset)),
        ));
        to.extend(shadow_numbers(
            &y.copied().unwrap_or_else(|| padding(inset)),
        ));
    }
    Some(PropInterp {
        basis: PropBasis::Shadows(basis),
        from,
        to,
    })
}

/// The functions of `p`'s filter (or backdrop) list.
fn ops_of(p: &PaintProps, backdrop: bool) -> Vec<FilterOp> {
    match &p.effects {
        None => Vec::new(),
        Some(e) if backdrop => e.backdrop.clone(),
        Some(e) => e.filter.clone(),
    }
}

fn same_filter_kind(x: &FilterOp, y: &FilterOp) -> bool {
    match (x, y) {
        (FilterOp::Graph(_), _) | (FilterOp::Shader(_), _) => x == y,
        _ => std::mem::discriminant(x) == std::mem::discriminant(y),
    }
}

fn amount(op: &FilterOp) -> f64 {
    match op {
        FilterOp::Brightness(a)
        | FilterOp::Contrast(a)
        | FilterOp::Saturate(a)
        | FilterOp::Grayscale(a)
        | FilterOp::Sepia(a)
        | FilterOp::HueRotate(a)
        | FilterOp::Invert(a)
        | FilterOp::Opacity(a)
        | FilterOp::Blur(a) => *a,
        _ => 0.0,
    }
}

fn filter_numbers(op: &FilterOp) -> Vec<f64> {
    match op {
        FilterOp::DropShadow {
            offset,
            blur,
            color,
        } => {
            let mut v = vec![offset.x, offset.y, *blur];
            v.extend(color4(*color));
            v
        }
        FilterOp::Graph(_) | FilterOp::Shader(_) => Vec::new(),
        other => vec![amount(other)],
    }
}

/// The CSS initial value of the function kind of `op`; None for Graph and Shader.
fn initial_like(op: &FilterOp) -> Option<FilterOp> {
    Some(match op {
        FilterOp::Brightness(_) => FilterOp::Brightness(1.0),
        FilterOp::Contrast(_) => FilterOp::Contrast(1.0),
        FilterOp::Saturate(_) => FilterOp::Saturate(1.0),
        FilterOp::Opacity(_) => FilterOp::Opacity(1.0),
        FilterOp::Grayscale(_) => FilterOp::Grayscale(0.0),
        FilterOp::Sepia(_) => FilterOp::Sepia(0.0),
        FilterOp::Invert(_) => FilterOp::Invert(0.0),
        FilterOp::HueRotate(_) => FilterOp::HueRotate(0.0),
        FilterOp::Blur(_) => FilterOp::Blur(0.0),
        FilterOp::DropShadow { .. } => FilterOp::DropShadow {
            offset: Point::default(),
            blur: 0.0,
            color: Rgba::TRANSPARENT,
        },
        FilterOp::Graph(_) | FilterOp::Shader(_) => return None,
    })
}

fn with_amount(op: &FilterOp, x: f64) -> FilterOp {
    match op {
        FilterOp::Brightness(_) => FilterOp::Brightness(x),
        FilterOp::Contrast(_) => FilterOp::Contrast(x),
        FilterOp::Saturate(_) => FilterOp::Saturate(x),
        FilterOp::Opacity(_) => FilterOp::Opacity(x),
        FilterOp::Grayscale(_) => FilterOp::Grayscale(x),
        FilterOp::Sepia(_) => FilterOp::Sepia(x),
        FilterOp::Invert(_) => FilterOp::Invert(x),
        FilterOp::HueRotate(_) => FilterOp::HueRotate(x),
        FilterOp::Blur(_) => FilterOp::Blur(x),
        other => other.clone(),
    }
}

/// One function with its numbers read from `v` at `*at`, advancing `*at`.
fn filter_fill(op: &FilterOp, v: &[f64], at: &mut usize) -> FilterOp {
    let i = *at;
    match op {
        FilterOp::DropShadow { .. } => {
            *at += 7;
            FilterOp::DropShadow {
                offset: Point {
                    x: v[i],
                    y: v[i + 1],
                },
                blur: v[i + 2].max(0.0),
                color: rgba4(&v[i + 3..i + 7]),
            }
        }
        FilterOp::Graph(_) | FilterOp::Shader(_) => op.clone(),
        _ => {
            *at += 1;
            with_amount(op, v[i])
        }
    }
}

/// The function list of one side padded with the initial values of the other side.
fn filter_interp(backdrop: bool, a: &[FilterOp], b: &[FilterOp]) -> Option<PropInterp> {
    let (basis, from, to) = if a.is_empty() && b.is_empty() {
        (Vec::new(), Vec::new(), Vec::new())
    } else if a.len() == b.len() && !a.is_empty() {
        if !a.iter().zip(b).all(|(x, y)| same_filter_kind(x, y)) {
            return None;
        }
        let from = a.iter().flat_map(filter_numbers).collect();
        let to = b.iter().flat_map(filter_numbers).collect();
        (a.to_vec(), from, to)
    } else if a.is_empty() {
        let init = initial_numbers(b)?;
        (
            b.to_vec(),
            init,
            b.iter().flat_map(filter_numbers).collect(),
        )
    } else if b.is_empty() {
        let init = initial_numbers(a)?;
        (
            a.to_vec(),
            a.iter().flat_map(filter_numbers).collect(),
            init,
        )
    } else {
        return None;
    };
    Some(PropInterp {
        basis: PropBasis::Filters {
            backdrop,
            ops: basis,
        },
        from,
        to,
    })
}

/// The numbers of the initial values of the kinds in `ops`; None when one is Graph or Shader.
fn initial_numbers(ops: &[FilterOp]) -> Option<Vec<f64>> {
    let mut out = Vec::new();
    for op in ops {
        out.extend(filter_numbers(&initial_like(op)?));
    }
    Some(out)
}
