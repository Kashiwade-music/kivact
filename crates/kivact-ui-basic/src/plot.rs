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
    AxisScale, LegendPosition, LineDash, PlotAxis, PlotColor, PlotData, PlotModel, PlotRange,
    PlotSeries, PlotSeriesKind, PlotView, TickFormat,
};
pub use paint::draw_plot;
pub use scale::auto_view;
pub use view::{Plot, plot};
