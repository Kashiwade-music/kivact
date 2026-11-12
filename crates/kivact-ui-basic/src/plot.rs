//! Role: XY plots: the data model, the zoomable plot and its painting and readouts.

mod decimate;
mod heatmap;
mod input;
mod model;
mod paint;
mod scale;
mod ticks;
mod view;

pub use model::{
    AxisScale, LineDash, PlotAxis, PlotData, PlotModel, PlotRange, PlotSeries, PlotSeriesKind,
    PlotView, TickFormat,
};
pub use paint::draw_plot;
pub use view::{Plot, plot};
