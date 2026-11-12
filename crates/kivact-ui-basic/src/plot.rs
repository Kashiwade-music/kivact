//! Role: XY plots: the data model, the zoomable plot and its painting and readouts.

mod axis;
mod data;
mod decimate;
mod heatmap;
mod input;
mod model;
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
