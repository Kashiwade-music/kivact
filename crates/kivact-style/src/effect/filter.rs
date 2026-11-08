//! Role: CSS filter functions (`Filter`) and their resolution to engine ops.

use std::sync::Arc;

use kivact_runtime::UiError;
use kivact_types::FilterOp;

use crate::color::Color;
use crate::geometry::Point;
use crate::shader::ShaderFilter;
use crate::theme::value::{Theme, ThemeColor};

use super::filter_graph::{AuthoredGraph, FilterGraph};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum FilterFn {
    Brightness(f64),
    Contrast(f64),
    Saturate(f64),
    Grayscale(f64),
    Sepia(f64),
    HueRotate(f64),
    Invert(f64),
    Opacity(f64),
    Blur(f64),
    DropShadow {
        offset: Point,
        blur: f64,
        color: Color,
    },
    /// An SVG filter graph (`Filter::graph`).
    Graph(Arc<AuthoredGraph>),
    /// A filter shader (`Filter::shader`).
    Shader(Arc<ShaderFilter>),
}

/// A list of CSS filter functions, applied in order (CSS `filter`,
/// `backdrop-filter`). Amounts are CSS's: `1` leaves the image as it is
/// for brightness, contrast, saturate and opacity, `0` for grayscale,
/// sepia and invert; angles are degrees, lengths px.
///
/// ```
/// use kivact::prelude::*;
/// use kivact::style::{rgba};
/// use kivact::draw::Filter;
/// // CSS `filter: brightness(1.12) drop-shadow(0 0 4px rgba(90,169,255,.5))`
/// let s = style().filter(
///     Filter::new()
///         .brightness(1.12)
///         .drop_shadow(0, 0, 4, rgba(0x5aa9ff, 0.5)),
/// );
/// # let _ = s;
/// ```
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Filter {
    pub(crate) ops: Vec<FilterFn>,
}

macro_rules! filter_fn {
    ($(#[$m:meta])* $name:ident, $variant:ident) => {
        $(#[$m])*
        pub fn $name(mut self, amount: impl Into<f64>) -> Filter {
            self.ops.push(FilterFn::$variant(amount.into()));
            self
        }
    };
}

impl Filter {
    /// No filter functions (add them with the methods).
    pub fn new() -> Filter {
        Filter::default()
    }

    filter_fn!(
        /// CSS `brightness(amount)`: 0 black, 1 unchanged.
        brightness, Brightness);
    filter_fn!(
        /// CSS `contrast(amount)`: 0 gray, 1 unchanged.
        contrast, Contrast);
    filter_fn!(
        /// CSS `saturate(amount)`: 0 gray, 1 unchanged, more is more vivid.
        saturate, Saturate);
    filter_fn!(
        /// CSS `grayscale(amount)`: 0 unchanged, 1 gray.
        grayscale, Grayscale);
    filter_fn!(
        /// CSS `sepia(amount)`: 0 unchanged, 1 sepia.
        sepia, Sepia);
    filter_fn!(
        /// CSS `hue-rotate(deg)`.
        hue_rotate, HueRotate);
    filter_fn!(
        /// CSS `invert(amount)`: 0 unchanged, 1 inverted.
        invert, Invert);
    filter_fn!(
        /// CSS `opacity(amount)`: 1 unchanged, 0 transparent.
        opacity, Opacity);
    filter_fn!(
        /// CSS `blur(px)`: a gaussian of standard deviation `px`.
        blur, Blur);

    /// CSS `drop-shadow(x y blur color)`: a blurred copy of the
    /// silhouette beneath (it follows the drawn shape, unlike a box
    /// shadow).
    pub fn drop_shadow(
        mut self,
        x: impl Into<f64>,
        y: impl Into<f64>,
        blur: impl Into<f64>,
        color: impl ThemeColor,
    ) -> Filter {
        self.ops.push(FilterFn::DropShadow {
            offset: Point::new(x.into(), y.into()),
            blur: blur.into(),
            color: Color::of(color),
        });
        self
    }

    /// An SVG filter graph (CSS `url(#filter)`), built with [`FilterGraph`].
    /// Its problems are reported when the style is compiled, and the graph
    /// is then dropped.
    pub fn graph(mut self, graph: FilterGraph) -> Filter {
        self.ops.push(FilterFn::Graph(Arc::new(graph.built())));
        self
    }

    /// A filter shader (see [`ShaderFilter`]), applied in its place in the list.
    pub fn shader(mut self, shader: ShaderFilter) -> Filter {
        self.ops.push(FilterFn::Shader(Arc::new(shader)));
        self
    }
}

/// The problems the filter's graphs and shaders have, for reporting.
pub(crate) fn filter_problems(f: &Filter, theme: &Theme) -> Vec<UiError> {
    f.ops
        .iter()
        .filter_map(|op| match op {
            FilterFn::Graph(g) => g
                .problem(theme)
                .map(|e| UiError::msg(format!("filter graph: {e}"))),
            FilterFn::Shader(s) => s.problems().first().cloned(),
            _ => None,
        })
        .collect()
}

/// A filter resolved against `theme` as the engine applies it.
pub(crate) fn filter_ops(f: &Filter, theme: &Theme) -> Vec<FilterOp> {
    f.ops
        .iter()
        .filter_map(|op| match op {
            FilterFn::Brightness(a) => Some(FilterOp::Brightness(*a)),
            FilterFn::Contrast(a) => Some(FilterOp::Contrast(*a)),
            FilterFn::Saturate(a) => Some(FilterOp::Saturate(*a)),
            FilterFn::Grayscale(a) => Some(FilterOp::Grayscale(*a)),
            FilterFn::Sepia(a) => Some(FilterOp::Sepia(*a)),
            FilterFn::HueRotate(a) => Some(FilterOp::HueRotate(*a)),
            FilterFn::Invert(a) => Some(FilterOp::Invert(*a)),
            FilterFn::Opacity(a) => Some(FilterOp::Opacity(*a)),
            FilterFn::Blur(a) => Some(FilterOp::Blur(*a)),
            FilterFn::DropShadow {
                offset,
                blur,
                color,
            } => Some(FilterOp::DropShadow {
                offset: *offset,
                blur: *blur,
                color: color.resolve(theme),
            }),
            // A graph with a problem is dropped (reported by `filter_problems`).
            FilterFn::Graph(g) => g
                .resolve(theme)
                .ok()
                .map(|graph| FilterOp::Graph(Arc::new(graph))),
            FilterFn::Shader(s) => Some(FilterOp::Shader(Arc::new(s.spec(theme)))),
        })
        .collect()
}
