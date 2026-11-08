//! Role: Runs the layer filters of an Op::Filter on the CPU.

use kivact_types::geometry::Point;
use kivact_types::render::LayerFilter;

use super::blur::gaussian;
use super::effects::{blur_sigmas, color_matrix, drop_shadow};
use super::graph;
use super::surface::Pixmap;
use crate::ComposeError;

/// `input` filtered by `filter`, of the same size. `px_per_unit` and `origin` map layer space to the pixels.
pub(crate) fn apply(
    filter: &LayerFilter,
    input: &Pixmap,
    px_per_unit: f64,
    origin: Point,
) -> Result<Pixmap, ComposeError> {
    Ok(match filter {
        LayerFilter::Blur { std_dev } => {
            let sigma = std_dev * px_per_unit;
            match blur_sigmas(sigma, sigma) {
                Some((sx, sy)) => gaussian(input, sx, sy),
                None => input.clone(),
            }
        }
        LayerFilter::DropShadow {
            offset,
            std_dev,
            color,
        } => {
            let shifted = (offset.x * px_per_unit, offset.y * px_per_unit);
            let sigma = std_dev * px_per_unit;
            drop_shadow(input, shifted, (sigma, sigma), *color)
        }
        LayerFilter::ColorMatrix(matrix) => color_matrix(input, matrix),
        LayerFilter::Graph(filter_graph) => graph::apply(filter_graph, input, px_per_unit, origin)?,
        // The CPU does not run filter shaders: the input passes through.
        LayerFilter::Shader(_) => input.clone(),
    })
}
