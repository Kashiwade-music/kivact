//! Role: SVG filter primitive graphs (CSS filter: url()) as values.
use crate::Rgba;
use crate::geometry::{Insets, Point, Rect};
use crate::pixel::BlendMode;

use crate::pixel::ColorMatrix;

/// A filter graph of SVG filter primitives. `I` is how an Image primitive
/// names its image: `Asset` on the wire (`FilterOp::Graph`), `Pixels` in a
/// composition (`LayerFilter::Graph`).
#[derive(Clone, Debug, PartialEq)]
pub struct FilterGraph<I> {
    /// The primitives in order; the last one's result is the output.
    pub primitives: Vec<Primitive<I>>,
    /// How far the filter region extends beyond the layer bounds; None:
    /// 10% of the bounds' width and height on each side (the SVG default).
    pub margin: Option<Insets>,
    /// Color space of primitives that do not set their own.
    pub color_space: FilterColorSpace,
}

/// The color space a filter primitive computes in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FilterColorSpace {
    /// Linear RGB (the SVG default).
    #[default]
    LinearRgb,
    /// sRGB.
    Srgb,
}

/// The input of a filter primitive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FilterInput {
    /// The layer's source graphic.
    Source,
    /// The source's alpha only.
    SourceAlpha,
    /// The output of an earlier primitive.
    Result(u32),
}

/// One filter primitive with its optional color space.
#[derive(Clone, Debug, PartialEq)]
pub struct Primitive<I> {
    /// What the primitive computes.
    pub op: PrimitiveOp<I>,
    /// Overrides the graph's color space for this primitive.
    pub color_space: Option<FilterColorSpace>,
}

/// The operation of one filter primitive.
#[derive(Clone, Debug, PartialEq)]
pub enum PrimitiveOp<I> {
    /// feBlend.
    Blend {
        top: FilterInput,
        bottom: FilterInput,
        mode: BlendMode,
    },
    /// feColorMatrix.
    ColorMatrix {
        input: FilterInput,
        matrix: ColorMatrix,
    },
    /// feComponentTransfer.
    ComponentTransfer {
        input: FilterInput,
        r: TransferFn,
        g: TransferFn,
        b: TransferFn,
        a: TransferFn,
    },
    /// feComposite.
    Composite {
        top: FilterInput,
        bottom: FilterInput,
        op: CompositeOp,
    },
    /// feConvolveMatrix.
    ConvolveMatrix {
        input: FilterInput,
        order: (u32, u32),
        kernel: Vec<f64>,
        divisor: f64,
        bias: f64,
        target: (u32, u32),
        edge: EdgeMode,
        preserve_alpha: bool,
    },
    /// feDiffuseLighting.
    DiffuseLighting {
        input: FilterInput,
        surface_scale: f64,
        diffuse_constant: f64,
        color: Rgba,
        light: Light,
    },
    /// feSpecularLighting.
    SpecularLighting {
        input: FilterInput,
        surface_scale: f64,
        specular_constant: f64,
        specular_exponent: f64,
        color: Rgba,
        light: Light,
    },
    /// feDisplacementMap.
    DisplacementMap {
        input: FilterInput,
        map: FilterInput,
        scale: f64,
        x: Channel,
        y: Channel,
    },
    /// feDropShadow.
    DropShadow {
        input: FilterInput,
        offset: Point,
        std_dev: (f64, f64),
        color: Rgba,
    },
    /// feFlood.
    Flood { color: Rgba },
    /// feGaussianBlur.
    GaussianBlur {
        input: FilterInput,
        std_dev: (f64, f64),
        edge: EdgeMode,
    },
    /// feImage.
    Image { image: I, rect: Rect },
    /// feMerge.
    Merge { inputs: Vec<FilterInput> },
    /// feMorphology.
    Morphology {
        input: FilterInput,
        op: MorphologyOp,
        radius: (f64, f64),
    },
    /// feOffset.
    Offset { input: FilterInput, offset: Point },
    /// feTile.
    Tile { input: FilterInput, tile: Rect },
    /// feTurbulence.
    Turbulence {
        base_frequency: (f64, f64),
        octaves: u32,
        seed: f64,
        kind: TurbulenceKind,
        stitch: bool,
    },
}

/// The operator of feComposite.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CompositeOp {
    /// Source over destination.
    Over,
    /// Source in destination.
    In,
    /// Source out destination.
    Out,
    /// Source atop destination.
    Atop,
    /// Exclusive or.
    Xor,
    /// Sum of both.
    Lighter,
    /// The arithmetic combination k1·i1·i2 + k2·i1 + k3·i2 + k4.
    Arithmetic { k1: f64, k2: f64, k3: f64, k4: f64 },
}

/// One channel's transfer function of feComponentTransfer.
#[derive(Clone, Debug, PartialEq)]
pub enum TransferFn {
    /// Unchanged.
    Identity,
    /// Piecewise linear over the table.
    Table(Vec<f64>),
    /// Stepped over the table.
    Discrete(Vec<f64>),
    /// slope · c + intercept.
    Linear { slope: f64, intercept: f64 },
    /// amplitude · c^exponent + offset.
    Gamma {
        amplitude: f64,
        exponent: f64,
        offset: f64,
    },
}

/// A light source of the lighting primitives.
#[derive(Clone, Debug, PartialEq)]
pub enum Light {
    /// A light at infinity.
    Distant {
        azimuth_deg: f64,
        elevation_deg: f64,
    },
    /// A light at a point.
    Point { position: [f64; 3] },
    /// A light at a point aimed at another point.
    Spot {
        position: [f64; 3],
        points_at: [f64; 3],
        exponent: f64,
        cone_deg: Option<f64>,
    },
}

/// How a convolution or blur treats pixels beyond the edge.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum EdgeMode {
    /// Repeat the edge pixels.
    Duplicate,
    /// Wrap around.
    Wrap,
    /// Transparent beyond the edge.
    #[default]
    None,
}

/// A color channel, for displacement maps.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Channel {
    /// Red.
    R,
    /// Green.
    G,
    /// Blue.
    B,
    /// Alpha.
    A,
}

/// The operator of feMorphology.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MorphologyOp {
    /// Minimum over the radius.
    Erode,
    /// Maximum over the radius.
    Dilate,
}

/// The noise type of feTurbulence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TurbulenceKind {
    /// Fractal noise.
    FractalNoise,
    /// Turbulence (absolute values of the noise).
    Turbulence,
}

impl<I> FilterGraph<I> {
    /// The same graph with each Image's `image` mapped by `f`.
    pub fn map_images<J>(&self, mut f: impl FnMut(&I) -> J) -> FilterGraph<J> {
        FilterGraph {
            primitives: self
                .primitives
                .iter()
                .map(|p| Primitive {
                    op: map_op(&p.op, &mut f),
                    color_space: p.color_space,
                })
                .collect(),
            margin: self.margin,
            color_space: self.color_space,
        }
    }

    /// Check the limits of the graph, with the same limits for the CPU and
    /// the GPU. Errors name the primitive index.
    pub fn validate(&self) -> Result<(), String> {
        if self.primitives.is_empty() {
            return Err("filter graph has no primitives".into());
        }
        if self.primitives.len() > 64 {
            return Err("filter graph has more than 64 primitives".into());
        }
        for (i, p) in self.primitives.iter().enumerate() {
            validate_op(i, &p.op)?;
        }
        Ok(())
    }
}

fn map_op<I, J>(op: &PrimitiveOp<I>, f: &mut impl FnMut(&I) -> J) -> PrimitiveOp<J> {
    use PrimitiveOp as P;
    match op {
        P::Image { image, rect } => P::Image {
            image: f(image),
            rect: *rect,
        },
        P::Blend { top, bottom, mode } => P::Blend {
            top: *top,
            bottom: *bottom,
            mode: *mode,
        },
        P::ColorMatrix { input, matrix } => P::ColorMatrix {
            input: *input,
            matrix: *matrix,
        },
        P::ComponentTransfer { input, r, g, b, a } => P::ComponentTransfer {
            input: *input,
            r: r.clone(),
            g: g.clone(),
            b: b.clone(),
            a: a.clone(),
        },
        P::Composite { top, bottom, op } => P::Composite {
            top: *top,
            bottom: *bottom,
            op: *op,
        },
        P::ConvolveMatrix {
            input,
            order,
            kernel,
            divisor,
            bias,
            target,
            edge,
            preserve_alpha,
        } => P::ConvolveMatrix {
            input: *input,
            order: *order,
            kernel: kernel.clone(),
            divisor: *divisor,
            bias: *bias,
            target: *target,
            edge: *edge,
            preserve_alpha: *preserve_alpha,
        },
        P::DiffuseLighting {
            input,
            surface_scale,
            diffuse_constant,
            color,
            light,
        } => P::DiffuseLighting {
            input: *input,
            surface_scale: *surface_scale,
            diffuse_constant: *diffuse_constant,
            color: *color,
            light: light.clone(),
        },
        P::SpecularLighting {
            input,
            surface_scale,
            specular_constant,
            specular_exponent,
            color,
            light,
        } => P::SpecularLighting {
            input: *input,
            surface_scale: *surface_scale,
            specular_constant: *specular_constant,
            specular_exponent: *specular_exponent,
            color: *color,
            light: light.clone(),
        },
        other => map_op_rest(other),
    }
}

/// Copies the primitives whose parameters do not depend on the image type.
/// `map_op` dispatches only the remaining variants here.
fn map_op_rest<I, J>(op: &PrimitiveOp<I>) -> PrimitiveOp<J> {
    use PrimitiveOp as P;
    match op {
        P::DisplacementMap {
            input,
            map,
            scale,
            x,
            y,
        } => P::DisplacementMap {
            input: *input,
            map: *map,
            scale: *scale,
            x: *x,
            y: *y,
        },
        P::DropShadow {
            input,
            offset,
            std_dev,
            color,
        } => P::DropShadow {
            input: *input,
            offset: *offset,
            std_dev: *std_dev,
            color: *color,
        },
        P::Flood { color } => P::Flood { color: *color },
        P::GaussianBlur {
            input,
            std_dev,
            edge,
        } => P::GaussianBlur {
            input: *input,
            std_dev: *std_dev,
            edge: *edge,
        },
        P::Merge { inputs } => P::Merge {
            inputs: inputs.clone(),
        },
        P::Morphology { input, op, radius } => P::Morphology {
            input: *input,
            op: *op,
            radius: *radius,
        },
        P::Offset { input, offset } => P::Offset {
            input: *input,
            offset: *offset,
        },
        P::Tile { input, tile } => P::Tile {
            input: *input,
            tile: *tile,
        },
        P::Turbulence {
            base_frequency,
            octaves,
            seed,
            kind,
            stitch,
        } => P::Turbulence {
            base_frequency: *base_frequency,
            octaves: *octaves,
            seed: *seed,
            kind: *kind,
            stitch: *stitch,
        },
        P::Image { .. } | P::Blend { .. } | P::ColorMatrix { .. } => {
            unreachable!("map_op dispatches these variants before map_op_rest")
        }
        P::ComponentTransfer { .. }
        | P::Composite { .. }
        | P::ConvolveMatrix { .. }
        | P::DiffuseLighting { .. }
        | P::SpecularLighting { .. } => {
            unreachable!("map_op dispatches these variants before map_op_rest")
        }
    }
}

fn check_input(i: usize, input: FilterInput) -> Result<(), String> {
    match input {
        FilterInput::Result(r) if r as usize >= i => Err(format!(
            "primitive {i}: Result({r}) must name an earlier primitive"
        )),
        _ => Ok(()),
    }
}

fn check_finite(i: usize, values: &[f64]) -> Result<(), String> {
    if values.iter().all(|v| v.is_finite()) {
        Ok(())
    } else {
        Err(format!("primitive {i}: a number is not finite"))
    }
}

fn check_non_negative(i: usize, values: &[f64]) -> Result<(), String> {
    check_finite(i, values)?;
    if values.iter().all(|v| *v >= 0.0) {
        Ok(())
    } else {
        Err(format!("primitive {i}: a number is negative"))
    }
}

fn validate_op<I>(i: usize, op: &PrimitiveOp<I>) -> Result<(), String> {
    use PrimitiveOp as P;
    match op {
        P::Blend { top, bottom, .. } | P::Composite { top, bottom, .. } => {
            check_input(i, *top)?;
            check_input(i, *bottom)
        }
        P::ColorMatrix { input, .. } => check_input(i, *input),
        P::Flood { .. } => Ok(()),
        P::Offset { input, offset } => {
            check_input(i, *input)?;
            check_finite(i, &[offset.x, offset.y])
        }
        P::Tile { input, tile } => {
            check_input(i, *input)?;
            check_finite(i, &[tile.x, tile.y, tile.width, tile.height])
        }
        P::Image { rect, .. } => check_finite(i, &[rect.x, rect.y, rect.width, rect.height]),
        P::Morphology { input, radius, .. } => {
            check_input(i, *input)?;
            check_non_negative(i, &[radius.0, radius.1])
        }
        P::DropShadow {
            input,
            offset,
            std_dev,
            ..
        } => {
            check_input(i, *input)?;
            check_finite(i, &[offset.x, offset.y])?;
            check_non_negative(i, &[std_dev.0, std_dev.1])
        }
        P::GaussianBlur { input, std_dev, .. } => {
            check_input(i, *input)?;
            check_non_negative(i, &[std_dev.0, std_dev.1])
        }
        P::Merge { inputs } => {
            if inputs.is_empty() || inputs.len() > 16 {
                return Err(format!("primitive {i}: Merge needs 1 to 16 inputs"));
            }
            inputs.iter().try_for_each(|input| check_input(i, *input))
        }
        other => validate_op_rest(i, other),
    }
}

/// Validates the primitives with tables, kernels, or lighting parameters.
/// `validate_op` dispatches only the remaining variants here.
fn validate_op_rest<I>(i: usize, op: &PrimitiveOp<I>) -> Result<(), String> {
    use PrimitiveOp as P;
    match op {
        P::ComponentTransfer { input, r, g, b, a } => {
            check_input(i, *input)?;
            [r, g, b, a]
                .into_iter()
                .try_for_each(|t| check_transfer(i, t))
        }
        P::ConvolveMatrix {
            input,
            order,
            kernel,
            divisor,
            target,
            ..
        } => {
            check_input(i, *input)?;
            check_convolve(i, *order, kernel, *divisor, *target)
        }
        P::DiffuseLighting {
            input,
            surface_scale,
            diffuse_constant,
            ..
        } => {
            check_input(i, *input)?;
            check_finite(i, &[*surface_scale])?;
            check_non_negative(i, &[*diffuse_constant])
        }
        P::SpecularLighting {
            input,
            surface_scale,
            specular_constant,
            specular_exponent,
            ..
        } => {
            check_input(i, *input)?;
            check_finite(i, &[*surface_scale])?;
            check_non_negative(i, &[*specular_constant])?;
            if (1.0..=128.0).contains(specular_exponent) {
                Ok(())
            } else {
                Err(format!("primitive {i}: specular_exponent must be 1 to 128"))
            }
        }
        P::DisplacementMap {
            input, map, scale, ..
        } => {
            check_input(i, *input)?;
            check_input(i, *map)?;
            check_finite(i, &[*scale])
        }
        P::Turbulence {
            base_frequency,
            octaves,
            seed,
            ..
        } => {
            check_non_negative(i, &[base_frequency.0, base_frequency.1])?;
            check_finite(i, &[*seed])?;
            if *octaves <= 16 {
                Ok(())
            } else {
                Err(format!("primitive {i}: octaves must be 0 to 16"))
            }
        }
        P::Blend { .. }
        | P::Composite { .. }
        | P::ColorMatrix { .. }
        | P::Flood { .. }
        | P::Offset { .. }
        | P::Tile { .. }
        | P::Image { .. }
        | P::Morphology { .. }
        | P::DropShadow { .. }
        | P::GaussianBlur { .. }
        | P::Merge { .. } => {
            unreachable!("validate_op dispatches these variants before validate_op_rest")
        }
    }
}

fn check_transfer(i: usize, t: &TransferFn) -> Result<(), String> {
    match t {
        TransferFn::Table(v) | TransferFn::Discrete(v) => {
            if v.is_empty() || v.len() > 256 {
                return Err(format!(
                    "primitive {i}: transfer tables need 1 to 256 values"
                ));
            }
            check_finite(i, v)
        }
        _ => Ok(()),
    }
}

fn check_convolve(
    i: usize,
    order: (u32, u32),
    kernel: &[f64],
    divisor: f64,
    target: (u32, u32),
) -> Result<(), String> {
    let valid_order = (1..=16).contains(&order.0) && (1..=16).contains(&order.1);
    if !valid_order {
        return Err(format!(
            "primitive {i}: ConvolveMatrix order must be 1 to 16"
        ));
    }
    if kernel.len() != (order.0 * order.1) as usize {
        return Err(format!(
            "primitive {i}: kernel length must be order.0 times order.1"
        ));
    }
    if target.0 >= order.0 || target.1 >= order.1 {
        return Err(format!("primitive {i}: target must lie inside the order"));
    }
    check_finite(i, kernel)?;
    check_finite(i, &[divisor])
}
