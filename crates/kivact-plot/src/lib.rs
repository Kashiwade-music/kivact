//! Role: Interactive XY plots and heatmaps on the public Kivact facade.
//!
//! [`plot`] draws a [`PlotModel`]: lines, points, stems, histograms and
//! heatmaps with linear or logarithmic axes. Dragging pans, the wheel zooms
//! and a double click returns to the automatic range, each as one edit.
//! Colors are theme variables ([`color`]): any kit may set them, and
//! without a kit they follow the core colors. A plot is exported to PNG or
//! SVG through `kivact::draw::Offscreen`.

mod axis;
pub mod color;
mod data;
mod decimate;
mod heatmap;
mod input;
mod model;
pub mod nice;
mod paint;
mod scale;
mod series;
mod ticks;
mod view;

pub use axis::{AxisScale, PlotAxis, TickFormat};
pub use data::PlotData;
pub use model::{PlotModel, PlotRange, PlotView};
pub use series::{LineDash, PlotSeries};
pub use view::{Plot, plot};
