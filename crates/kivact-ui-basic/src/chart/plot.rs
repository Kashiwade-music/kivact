//! Role: Chart options, and painting and hit-testing dispatched by chart kind.

use super::model::{ChartConfig, ChartKind, ChartRow};
use super::model::{Frame, Hit};
use kivact::draw::Draw;
use kivact::geometry::{Point, Size};
use kivact::state::Value;

/// The options a chart paints and hit-tests with.
#[derive(Clone, Copy)]
pub(super) struct Options {
    pub(super) kind: Value<ChartKind>,
    pub(super) horizontal: Value<bool>,
    pub(super) grid: Value<bool>,
    pub(super) axis: Value<bool>,
}

impl Options {
    pub(super) fn frame<'a>(
        self,
        config: &'a ChartConfig,
        rows: &'a [ChartRow],
        total_label: &'a str,
        format: &'a dyn Fn(f64) -> String,
    ) -> Frame<'a> {
        Frame {
            config,
            rows,
            kind: self.kind.get(),
            horizontal: self.horizontal.get(),
            grid: self.grid.get(),
            axis: self.axis.get(),
            total_label,
            format,
        }
    }
}

pub(super) fn paint(d: &mut Draw, size: Size, frame: &Frame, hit: Option<&Hit>) {
    match frame.kind {
        ChartKind::Bar | ChartKind::StackedBar | ChartKind::Line | ChartKind::Area => {
            super::cartesian::paint(d, size, frame, hit);
        }
        ChartKind::Pie | ChartKind::Donut | ChartKind::Radial => {
            super::pie::paint(d, size, frame, hit)
        }
        ChartKind::Radar => super::radar::paint(d, size, frame, hit),
    }
}

pub(super) fn hit_test(size: Size, frame: &Frame, at: Point) -> Option<Hit> {
    match frame.kind {
        ChartKind::Bar | ChartKind::StackedBar | ChartKind::Line | ChartKind::Area => {
            super::cartesian::hit(size, frame, at)
        }
        ChartKind::Pie | ChartKind::Donut | ChartKind::Radial => super::pie::hit(size, frame, at),
        ChartKind::Radar => super::radar::hit(size, frame, at),
    }
}
