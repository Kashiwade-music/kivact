//! Role: Axes: the title, the scale and the tick format of one axis.

/// How an axis maps values to positions.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AxisScale {
    /// Equal steps for equal differences.
    Linear,
    /// Equal steps for equal ratios; values of zero or less are not drawn.
    Log10,
}

/// How tick labels and readouts write their values.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TickFormat {
    /// Four significant digits, the shortest form.
    Plain,
    /// SI prefixes: `1.5k`, `2M`, `10m`, `3µ`.
    Si,
    /// Decibels: whole numbers, or one decimal below 1.
    Decibel,
    /// Degrees, with a `°` suffix.
    Degrees,
}

/// One axis: its title, scale and tick format. Build it with [`PlotAxis::new`]
/// and the setters.
#[derive(Clone, PartialEq, Debug)]
pub struct PlotAxis {
    /// The title drawn beside the axis.
    pub(super) label: String,
    /// The scale.
    pub(super) scale: AxisScale,
    /// How ticks and readouts write values.
    pub(super) format: TickFormat,
}

impl PlotAxis {
    /// A linear axis titled `label`, with plain tick labels.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            scale: AxisScale::Linear,
            format: TickFormat::Plain,
        }
    }

    /// The same axis on `scale`.
    pub fn scale(mut self, scale: AxisScale) -> Self {
        self.scale = scale;
        self
    }

    /// The same axis with its tick labels written in `format`.
    pub fn format(mut self, format: TickFormat) -> Self {
        self.format = format;
        self
    }
}

#[cfg(test)]
mod tests {
    //! Role: Tests for the axis builder.

    use super::*;

    #[test]
    fn an_axis_is_linear_and_plain_until_set() {
        let axis = PlotAxis::new("Hz");
        assert_eq!(axis.scale, AxisScale::Linear);
        assert_eq!(axis.format, TickFormat::Plain);
        let log = axis.scale(AxisScale::Log10).format(TickFormat::Si);
        assert_eq!(log.label, "Hz");
        assert_eq!((log.scale, log.format), (AxisScale::Log10, TickFormat::Si));
    }
}
