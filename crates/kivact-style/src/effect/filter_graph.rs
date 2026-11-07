//! Role: The filter graph builder (SVG filter primitives written as values).
//!
//! A [`FilterGraph`] is built by method calls, each of which adds a
//! primitive and returns a [`FilterInput`] handle to its result. The handles
//! name their graph, so a handle of another graph is refused. `Filter::graph`
//! consumes the builder into an [`AuthoredGraph`] (a `Clone` value with its
//! colors still theme-dependent); style resolution turns that into the
//! engine's `kivact_types::FilterGraph<Asset>`.

use std::sync::atomic::{AtomicU64, Ordering};

use kivact_types as p;
use kivact_types::{
    BlendMode, Channel, CompositeOp, EdgeMode, FilterColorSpace, Insets, Light, MorphologyOp,
    PrimitiveOp, Rgba, TransferFn, TurbulenceKind,
};

use crate::asset::Asset;
use crate::color::Color;
use crate::geometry::Rect;
use crate::theme::value::ThemeColor;

/// The graph ids: a process-wide counter from 1 (0 is "no graph").
static NEXT_GRAPH: AtomicU64 = AtomicU64::new(1);

/// A handle to a primitive's result, or to the element itself. Copy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FilterInput {
    pub(crate) graph: u64,
    pub(crate) input: p::FilterInput,
}

impl FilterInput {
    /// The element's own pixels (belongs to no graph: valid in every graph).
    pub const SOURCE: FilterInput = FilterInput {
        graph: 0,
        input: p::FilterInput::Source,
    };
    /// The element's alpha (belongs to no graph: valid in every graph).
    pub const SOURCE_ALPHA: FilterInput = FilterInput {
        graph: 0,
        input: p::FilterInput::SourceAlpha,
    };
}

/// One primitive as authored: its operation (a color placeholder where the
/// operation has one) and that color, resolved with the theme later.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AuthoredPrimitive {
    op: PrimitiveOp<Asset>,
    color: Option<Color>,
}

/// A filter graph being built. Not Clone: a handle names one graph, so a
/// copy would let handles of one graph slip into another unnoticed.
/// `Filter::graph` consumes it and keeps the built graph (which is Clone).
///
/// ```
/// use kivact::prelude::*;
/// use kivact::draw::{Filter, FilterGraph, FilterInput};
/// use kivact::draw::filter::Channel;
/// // A glitch: displace the element by noise.
/// let mut g = FilterGraph::new();
/// let noise = g.turbulence(0.02, 2, 3.0);
/// g.displacement_map(FilterInput::SOURCE, noise, 12.0, Channel::R, Channel::G);
/// let glitch = Filter::new().graph(g);
/// # let _ = glitch;
/// ```
#[derive(Debug)]
pub struct FilterGraph {
    id: u64,
    prims: Vec<AuthoredPrimitive>,
    margin: Option<Insets>,
    color_space: FilterColorSpace,
    errors: Vec<String>,
}

impl Default for FilterGraph {
    fn default() -> Self {
        FilterGraph::new()
    }
}

impl FilterGraph {
    /// An empty graph (add primitives with the methods).
    pub fn new() -> FilterGraph {
        FilterGraph {
            id: NEXT_GRAPH.fetch_add(1, Ordering::Relaxed),
            prims: Vec::new(),
            margin: None,
            color_space: FilterColorSpace::LinearRgb,
            errors: Vec::new(),
        }
    }

    /// Extend the filter region beyond the element (default: 10% of its size).
    pub fn margin(&mut self, margin: impl Into<Insets>) -> &mut FilterGraph {
        self.margin = Some(margin.into());
        self
    }

    /// Compute in sRGB instead of linear RGB (SVG color-interpolation-filters).
    pub fn srgb(&mut self) -> &mut FilterGraph {
        self.color_space = FilterColorSpace::Srgb;
        self
    }

    /// Add a primitive and return its handle.
    fn push(&mut self, op: PrimitiveOp<Asset>, color: Option<Color>) -> FilterInput {
        let index = self.prims.len() as u32;
        self.prims.push(AuthoredPrimitive { op, color });
        FilterInput {
            graph: self.id,
            input: p::FilterInput::Result(index),
        }
    }

    /// An input of the primitive being added: a handle of this graph, or the
    /// element. A handle of another graph becomes the element and is reported.
    fn input(&mut self, h: FilterInput) -> p::FilterInput {
        if h.graph == 0 {
            return h.input;
        }
        if h.graph != self.id {
            self.errors.push(format!(
                "primitive {}: an input of another filter graph was replaced by the element",
                self.prims.len() + 1
            ));
            return p::FilterInput::Source;
        }
        if let p::FilterInput::Result(n) = h.input
            && n as usize >= self.prims.len()
        {
            self.errors.push(format!(
                "primitive {}: an input names a later result",
                self.prims.len() + 1
            ));
            return p::FilterInput::Source;
        }
        h.input
    }

    /// Add a primitive whose inputs are checked.
    fn add(&mut self, op: impl FnOnce(&mut Self) -> PrimitiveOp<Asset>) -> FilterInput {
        let op = op(self);
        self.push(op, None)
    }

    /// A Gaussian blur of standard deviation `std_dev` px.
    pub fn blur(&mut self, input: FilterInput, std_dev: impl Into<f64>) -> FilterInput {
        let s: f64 = std_dev.into();
        self.blur_xy(input, s, s)
    }

    /// A Gaussian blur with separate deviations along x and y.
    pub fn blur_xy(
        &mut self,
        input: FilterInput,
        x: impl Into<f64>,
        y: impl Into<f64>,
    ) -> FilterInput {
        let (x, y) = (x.into(), y.into());
        self.add(|g| PrimitiveOp::GaussianBlur {
            input: g.input(input),
            std_dev: (x, y),
            edge: EdgeMode::None,
        })
    }

    /// Two inputs blended by `mode` (`top` over `bottom`).
    pub fn blend(&mut self, top: FilterInput, bottom: FilterInput, mode: BlendMode) -> FilterInput {
        self.add(|g| PrimitiveOp::Blend {
            top: g.input(top),
            bottom: g.input(bottom),
            mode,
        })
    }

    /// A 4×5 color matrix (row by row, as SVG `feColorMatrix`).
    pub fn color_matrix(&mut self, input: FilterInput, matrix: [f64; 20]) -> FilterInput {
        self.add(|g| PrimitiveOp::ColorMatrix {
            input: g.input(input),
            matrix: p::ColorMatrix(matrix),
        })
    }

    /// The color matrix of `saturate(amount)`.
    pub fn saturate(&mut self, input: FilterInput, amount: impl Into<f64>) -> FilterInput {
        let s: f64 = amount.into();
        self.color_matrix(
            input,
            [
                0.213 + 0.787 * s,
                0.715 - 0.715 * s,
                0.072 - 0.072 * s,
                0.0,
                0.0,
                0.213 - 0.213 * s,
                0.715 + 0.285 * s,
                0.072 - 0.072 * s,
                0.0,
                0.0,
                0.213 - 0.213 * s,
                0.715 - 0.715 * s,
                0.072 + 0.928 * s,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
            ],
        )
    }

    /// The color matrix of `hue-rotate(deg)`.
    pub fn hue_rotate(&mut self, input: FilterInput, deg: impl Into<f64>) -> FilterInput {
        let r: f64 = deg.into();
        let (c, s) = (r.to_radians().cos(), r.to_radians().sin());
        self.color_matrix(
            input,
            [
                0.213 + c * 0.787 - s * 0.213,
                0.715 - c * 0.715 - s * 0.715,
                0.072 - c * 0.072 + s * 0.928,
                0.0,
                0.0,
                0.213 - c * 0.213 + s * 0.143,
                0.715 + c * 0.285 + s * 0.140,
                0.072 - c * 0.072 - s * 0.283,
                0.0,
                0.0,
                0.213 - c * 0.213 - s * 0.787,
                0.715 - c * 0.715 + s * 0.715,
                0.072 + c * 0.928 + s * 0.072,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
            ],
        )
    }

    /// The luminance as alpha (`feColorMatrix type="luminanceToAlpha"`).
    pub fn luminance_to_alpha(&mut self, input: FilterInput) -> FilterInput {
        self.color_matrix(
            input,
            [
                0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, //
                0.2125, 0.7154, 0.0721, 0.0, 0.0,
            ],
        )
    }

    /// A transfer function per channel (`feComponentTransfer`).
    pub fn component_transfer(
        &mut self,
        input: FilterInput,
        r: TransferFn,
        g: TransferFn,
        b: TransferFn,
        a: TransferFn,
    ) -> FilterInput {
        self.add(|s| PrimitiveOp::ComponentTransfer {
            input: s.input(input),
            r,
            g,
            b,
            a,
        })
    }

    /// Porter–Duff composition of `top` onto `bottom` (`feComposite`).
    pub fn composite(
        &mut self,
        top: FilterInput,
        bottom: FilterInput,
        op: CompositeOp,
    ) -> FilterInput {
        self.add(|g| PrimitiveOp::Composite {
            top: g.input(top),
            bottom: g.input(bottom),
            op,
        })
    }

    /// A convolution of `input` with a kernel of `order` (columns, rows):
    /// the divisor is the kernel sum (1 if 0), the target is its center, and
    /// the edges duplicate.
    pub fn convolve(
        &mut self,
        input: FilterInput,
        order: (u32, u32),
        kernel: impl IntoIterator<Item = f64>,
    ) -> FilterInput {
        let kernel: Vec<f64> = kernel.into_iter().collect();
        let sum: f64 = kernel.iter().sum();
        let divisor = if sum == 0.0 { 1.0 } else { sum };
        let target = (order.0 / 2, order.1 / 2);
        self.add(|g| PrimitiveOp::ConvolveMatrix {
            input: g.input(input),
            order,
            kernel,
            divisor,
            bias: 0.0,
            target,
            edge: EdgeMode::Duplicate,
            preserve_alpha: false,
        })
    }

    /// Diffuse lighting of the element's alpha as a surface, lit by `light`.
    pub fn diffuse_lighting(
        &mut self,
        input: FilterInput,
        light: Light,
        color: impl ThemeColor,
    ) -> FilterInput {
        let input = self.input(input);
        self.push(
            PrimitiveOp::DiffuseLighting {
                input,
                surface_scale: 1.0,
                diffuse_constant: 1.0,
                color: Rgba::TRANSPARENT,
                light,
            },
            Some(Color::of(color)),
        )
    }

    /// Specular lighting of the element's alpha as a surface, lit by `light`.
    pub fn specular_lighting(
        &mut self,
        input: FilterInput,
        light: Light,
        color: impl ThemeColor,
        exponent: impl Into<f64>,
    ) -> FilterInput {
        let input = self.input(input);
        let exponent: f64 = exponent.into();
        self.push(
            PrimitiveOp::SpecularLighting {
                input,
                surface_scale: 1.0,
                specular_constant: 1.0,
                specular_exponent: exponent,
                color: Rgba::TRANSPARENT,
                light,
            },
            Some(Color::of(color)),
        )
    }

    /// Displace `input` by the channels `x` and `y` of `map`, scaled by `scale`.
    pub fn displacement_map(
        &mut self,
        input: FilterInput,
        map: FilterInput,
        scale: impl Into<f64>,
        x: Channel,
        y: Channel,
    ) -> FilterInput {
        let scale: f64 = scale.into();
        self.add(|g| PrimitiveOp::DisplacementMap {
            input: g.input(input),
            map: g.input(map),
            scale,
            x,
            y,
        })
    }

    /// A blurred, colored copy of the silhouette, offset by `(x, y)`.
    pub fn drop_shadow(
        &mut self,
        input: FilterInput,
        x: impl Into<f64>,
        y: impl Into<f64>,
        std_dev: impl Into<f64>,
        color: impl ThemeColor,
    ) -> FilterInput {
        let (x, y, s) = (x.into(), y.into(), std_dev.into());
        let input = self.input(input);
        self.push(
            PrimitiveOp::DropShadow {
                input,
                offset: p::Point { x, y },
                std_dev: (s, s),
                color: Rgba::TRANSPARENT,
            },
            Some(Color::of(color)),
        )
    }

    /// The color, everywhere the filter region is.
    pub fn flood(&mut self, color: impl ThemeColor) -> FilterInput {
        self.push(
            PrimitiveOp::Flood {
                color: Rgba::TRANSPARENT,
            },
            Some(Color::of(color)),
        )
    }

    /// `color` where `input` has alpha (feFlood and feComposite `in`): the
    /// result is the composite, the flood is the one before it.
    pub fn flood_in(&mut self, input: FilterInput, color: impl ThemeColor) -> FilterInput {
        let flood = self.flood(color);
        let bottom = self.input(input);
        let top = self.input(flood);
        self.add(|_| PrimitiveOp::Composite {
            top,
            bottom,
            op: CompositeOp::In,
        })
    }

    /// A picture drawn into `rect` (feImage).
    pub fn image(&mut self, image: impl Into<Asset>, rect: Rect) -> FilterInput {
        let image: Asset = image.into();
        self.add(|_| PrimitiveOp::Image { image, rect })
    }

    /// The inputs stacked in order, the first at the bottom (feMerge).
    pub fn merge(&mut self, inputs: impl IntoIterator<Item = FilterInput>) -> FilterInput {
        let inputs: Vec<FilterInput> = inputs.into_iter().collect();
        let inputs: Vec<p::FilterInput> = inputs.into_iter().map(|h| self.input(h)).collect();
        self.add(|_| PrimitiveOp::Merge { inputs })
    }

    /// Shrink the opaque parts of `input` by `radius` px (feMorphology erode).
    pub fn erode(&mut self, input: FilterInput, radius: impl Into<f64>) -> FilterInput {
        let r: f64 = radius.into();
        self.add(|g| PrimitiveOp::Morphology {
            input: g.input(input),
            op: MorphologyOp::Erode,
            radius: (r, r),
        })
    }

    /// Grow the opaque parts of `input` by `radius` px (feMorphology dilate).
    pub fn dilate(&mut self, input: FilterInput, radius: impl Into<f64>) -> FilterInput {
        let r: f64 = radius.into();
        self.add(|g| PrimitiveOp::Morphology {
            input: g.input(input),
            op: MorphologyOp::Dilate,
            radius: (r, r),
        })
    }

    /// `input` moved by `(x, y)` px (feOffset).
    pub fn offset(
        &mut self,
        input: FilterInput,
        x: impl Into<f64>,
        y: impl Into<f64>,
    ) -> FilterInput {
        let (x, y) = (x.into(), y.into());
        self.add(|g| PrimitiveOp::Offset {
            input: g.input(input),
            offset: p::Point { x, y },
        })
    }

    /// `input` repeated in every `tile` (feTile).
    pub fn tile(&mut self, input: FilterInput, tile: Rect) -> FilterInput {
        self.add(|g| PrimitiveOp::Tile {
            input: g.input(input),
            tile,
        })
    }

    /// Turbulence noise of `base_frequency`, `octaves` and `seed` (feTurbulence).
    pub fn turbulence(
        &mut self,
        base_frequency: impl Into<f64>,
        octaves: u32,
        seed: impl Into<f64>,
    ) -> FilterInput {
        self.noise(TurbulenceKind::Turbulence, base_frequency, octaves, seed)
    }

    /// Fractal noise of `base_frequency`, `octaves` and `seed` (feTurbulence fractalNoise).
    pub fn fractal_noise(
        &mut self,
        base_frequency: impl Into<f64>,
        octaves: u32,
        seed: impl Into<f64>,
    ) -> FilterInput {
        self.noise(TurbulenceKind::FractalNoise, base_frequency, octaves, seed)
    }

    fn noise(
        &mut self,
        kind: TurbulenceKind,
        base_frequency: impl Into<f64>,
        octaves: u32,
        seed: impl Into<f64>,
    ) -> FilterInput {
        let f: f64 = base_frequency.into();
        let seed: f64 = seed.into();
        self.add(|_| PrimitiveOp::Turbulence {
            base_frequency: (f, f),
            octaves,
            seed,
            kind,
            stitch: false,
        })
    }
}

/// A built filter graph: its primitives with their colors still to resolve.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AuthoredGraph {
    prims: Vec<AuthoredPrimitive>,
    margin: Option<Insets>,
    color_space: FilterColorSpace,
    errors: Vec<String>,
}

impl FilterGraph {
    /// The graph as a value, with the problems the builder found.
    pub(crate) fn built(self) -> AuthoredGraph {
        AuthoredGraph {
            prims: self.prims,
            margin: self.margin,
            color_space: self.color_space,
            errors: self.errors,
        }
    }
}

impl AuthoredGraph {
    /// The engine's graph under `theme`, or the first problem it has (the
    /// builder's problems come first).
    pub(crate) fn resolve(
        &self,
        theme: &crate::theme::value::Theme,
    ) -> Result<p::FilterGraph<Asset>, String> {
        if let Some(e) = self.errors.first() {
            return Err(e.clone());
        }
        let primitives = self
            .prims
            .iter()
            .map(|prim| {
                let mut op = prim.op.clone();
                if let Some(c) = prim.color {
                    let c = c.resolve(theme);
                    match &mut op {
                        PrimitiveOp::DiffuseLighting { color, .. }
                        | PrimitiveOp::SpecularLighting { color, .. }
                        | PrimitiveOp::DropShadow { color, .. }
                        | PrimitiveOp::Flood { color } => *color = c,
                        _ => {}
                    }
                }
                p::Primitive {
                    op,
                    color_space: None,
                }
            })
            .collect();
        let graph = p::FilterGraph {
            primitives,
            margin: self.margin,
            color_space: self.color_space,
        };
        graph.validate().map(|()| graph)
    }

    /// The problems that keep this graph from resolving (for reporting).
    pub(crate) fn problem(&self, theme: &crate::theme::value::Theme) -> Option<String> {
        self.resolve(theme).err()
    }
}
