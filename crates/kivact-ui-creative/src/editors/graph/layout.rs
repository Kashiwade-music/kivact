//! Role: Finite graph coordinates and independent logical view intervals.

use crate::editors::graph::gesture::VisibleKey;
use crate::editors::graph::model::GraphView;
use crate::editors::shared::metrics::{RULER, SIDEBAR};
use kivact::draw::{Point, Rect, Size};

pub(crate) fn axis(range: (f64, f64), fallback: (f64, f64)) -> (f64, f64) {
    if !range.0.is_finite() || !range.1.is_finite() || range.0 >= range.1 {
        return fallback;
    }
    let range = (range.0.clamp(-1e100, 1e100), range.1.clamp(-1e100, 1e100));
    if range.1 > range.0 { range } else { fallback }
}

pub(crate) fn safe_view(view: GraphView) -> GraphView {
    let fallback = GraphView::default();
    GraphView {
        time: axis(view.time, fallback.time),
        value: axis(view.value, fallback.value),
    }
}

pub(crate) fn number(value: f64, fallback: f64) -> f64 {
    if value.is_finite() { value } else { fallback }
}

fn coordinate(value: f64) -> f64 {
    if value.is_nan() {
        0.0
    } else {
        value.clamp(-1e6, 1e6)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Space {
    pub(crate) view: GraphView,
    pub(crate) plot: Rect,
}
impl Space {
    pub(crate) fn new(view: GraphView, size: Size) -> Self {
        Self {
            view: safe_view(view),
            plot: Rect::new(
                SIDEBAR,
                RULER,
                number(size.width - SIDEBAR, 1.0).clamp(1.0, 1e6),
                number(size.height - RULER, 1.0).clamp(1.0, 1e6),
            ),
        }
    }
    pub(crate) fn point(self, time: f64, value: f64) -> Point {
        Point::new(
            coordinate(
                self.plot.x
                    + (time - self.view.time.0) / (self.view.time.1 - self.view.time.0)
                        * self.plot.width,
            ),
            coordinate(
                (self.plot.y + self.plot.height)
                    - (value - self.view.value.0) / (self.view.value.1 - self.view.value.0)
                        * self.plot.height,
            ),
        )
    }
    pub(crate) fn time(self, x: f64) -> f64 {
        self.view.time.0
            + (x - self.plot.x) / self.plot.width * (self.view.time.1 - self.view.time.0)
    }
    pub(crate) fn value(self, y: f64) -> f64 {
        self.view.value.1
            - (y - self.plot.y) / self.plot.height * (self.view.value.1 - self.view.value.0)
    }
    pub(crate) fn delta(self, delta: Point) -> (f64, f64) {
        (
            delta.x / self.plot.width * (self.view.time.1 - self.view.time.0),
            -delta.y / self.plot.height * (self.view.value.1 - self.view.value.0),
        )
    }
    pub(crate) fn key(self, key: &VisibleKey) -> Rect {
        let p = self.point(key.key.time, key.key.value);
        Rect::new(p.x - 6.0, p.y - 6.0, 12.0, 12.0)
    }
    pub(crate) fn pan(self, delta: Point) -> GraphView {
        let (dt, dv) = self.delta(delta);
        safe_view(GraphView {
            time: (self.view.time.0 - dt, self.view.time.1 - dt),
            value: (self.view.value.0 - dv, self.view.value.1 - dv),
        })
    }
    pub(crate) fn zoom(self, at: Point, delta: f64, vertical: bool) -> GraphView {
        let factor = (-delta * 0.001).clamp(-20.0, 20.0).exp();
        let mut next = self.view;
        let (range, anchor) = if vertical {
            (self.view.value, self.value(at.y))
        } else {
            (self.view.time, self.time(at.x))
        };
        let low = anchor - (anchor - range.0) / factor;
        let high = anchor + (range.1 - anchor) / factor;
        if vertical {
            next.value = (low, high);
        } else {
            next.time = (low, high);
        }
        safe_view(next)
    }
}

pub(crate) fn clipped(rect: Rect, plot: Rect) -> Option<Rect> {
    let x = rect.x.max(plot.x);
    let y = rect.y.max(plot.y);
    let right = (rect.x + rect.width).min(plot.x + plot.width);
    let bottom = (rect.y + rect.height).min(plot.y + plot.height);
    (right > x && bottom > y).then_some(Rect::new(x, y, right - x, bottom - y))
}
pub(crate) fn snapped(value: f64, step: f64, snap: bool) -> f64 {
    if snap {
        number((value / step).round() * step, value)
    } else {
        value
    }
}
pub(crate) fn fit(keys: &[VisibleKey], selected: &[u64], step: (f64, f64)) -> Option<GraphView> {
    let chosen: Vec<_> = keys
        .iter()
        .filter(|key| selected.is_empty() || selected.contains(&key.key.id))
        .collect();
    let chosen = if chosen.is_empty() {
        keys.iter().collect()
    } else {
        chosen
    };
    let range = |value: fn(&VisibleKey) -> f64, step: f64| {
        let mut low = f64::INFINITY;
        let mut high = f64::NEG_INFINITY;
        for key in &chosen {
            let v = value(key);
            if v.is_finite() {
                low = low.min(v);
                high = high.max(v);
            }
        }
        if !low.is_finite() {
            return None;
        }
        let padding = if high > low { (high - low) * 0.1 } else { step };
        Some((low - padding, high + padding))
    };
    Some(safe_view(GraphView {
        time: range(|key| key.key.time, step.0)?,
        value: range(|key| key.key.value, step.1)?,
    }))
}
