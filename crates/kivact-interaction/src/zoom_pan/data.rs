//! Role: Define view transforms, options, and passive gesture snapshots.
use kivact_core::geometry::{Point, Rect, Size};
use kivact_core::state::Value;
use kivact_core::view::{MouseButton, PointerId};

/// Content-to-view coordinates: `view = content * scale + offset`.
/// Invalid (non-positive or non-finite) scales are treated as 1 by operations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewTransform {
    pub offset: Point,
    pub scale: f64,
}
kivact_core::state::constant_value!(ViewTransform);
impl Default for ViewTransform {
    fn default() -> Self {
        Self {
            offset: Point::new(0.0, 0.0),
            scale: 1.0,
        }
    }
}
impl ViewTransform {
    /// Map a surface-local point into content coordinates.
    pub fn to_content(&self, view: Point) -> Point {
        let scale = valid_scale(self.scale);
        Point::new(
            (view.x - self.offset.x) / scale,
            (view.y - self.offset.y) / scale,
        )
    }
    /// Map a content point into surface-local coordinates.
    pub fn to_view(&self, content: Point) -> Point {
        let scale = valid_scale(self.scale);
        Point::new(
            content.x * scale + self.offset.x,
            content.y * scale + self.offset.y,
        )
    }
    /// Uniformly fit the whole content into the view, centered.
    /// Non-positive or non-finite dimensions return the identity transform.
    pub fn fit(content: Size, view: Size) -> Self {
        if ![content.width, content.height, view.width, view.height]
            .into_iter()
            .all(|v| v.is_finite() && v > 0.0)
        {
            return Self::default();
        }
        let scale = (view.width / content.width).min(view.height / content.height);
        Self {
            scale,
            offset: Point::new(
                (view.width - content.width * scale) / 2.0,
                (view.height - content.height * scale) / 2.0,
            ),
        }
    }
}
pub(super) fn valid_scale(scale: f64) -> f64 {
    if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    }
}
pub(super) fn finite(point: Point) -> bool {
    point.x.is_finite() && point.y.is_finite()
}

/// What the setters of [`super::ZoomPanApi`] store; read at the event, so they work in
/// any order.
#[derive(Clone, Copy)]
pub(super) struct ZoomPanOptions {
    pub(super) min: f64,
    pub(super) max: f64,
    pub(super) content: Option<Value<Rect>>,
}

impl Default for ZoomPanOptions {
    fn default() -> Self {
        Self {
            min: 0.01,
            max: 100.0,
            content: None,
        }
    }
}

/// A pan: the pointer that runs it and the transform it started from.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct Pan {
    pub(super) pointer: PointerId,
    pub(super) position: Point,
    pub(super) transform: ViewTransform,
    pub(super) button: MouseButton,
}

/// One finger of a pinch: where it went down and where it is.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct Finger {
    pub(super) pointer: PointerId,
    pub(super) start: Point,
    pub(super) now: Point,
}

/// A two-finger pinch: its fingers and the transform at its start.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct Pinch {
    pub(super) fingers: [Finger; 2],
    pub(super) transform: ViewTransform,
}

fn midpoint(a: Point, b: Point) -> Point {
    Point::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0)
}

fn distance(a: Point, b: Point) -> f64 {
    (a.x - b.x).hypot(a.y - b.y)
}

/// The transform of `pan` with the pointer at `pos`.
pub(super) fn panned(pan: Pan, pos: Point) -> ViewTransform {
    ViewTransform {
        scale: valid_scale(pan.transform.scale),
        offset: Point::new(
            pan.transform.offset.x + pos.x - pan.position.x,
            pan.transform.offset.y + pos.y - pan.position.y,
        ),
    }
}

/// The transform `pinch` makes: the content point under the fingers'
/// starting midpoint follows their midpoint, scaled by how far apart
/// they are now against at the start (clamped to the scale range).
pub(super) fn pinched(pinch: Pinch, options: ZoomPanOptions) -> ViewTransform {
    let ZoomPanOptions { min, max, .. } = options;
    let [a, b] = pinch.fingers;
    let (start, now) = (distance(a.start, b.start), distance(a.now, b.now));
    let factor = if start > 0.0 && now.is_finite() {
        now / start
    } else {
        1.0
    };
    let scale = (valid_scale(pinch.transform.scale) * factor).clamp(min, max);
    let content = pinch.transform.to_content(midpoint(a.start, b.start));
    let anchor = midpoint(a.now, b.now);
    ViewTransform {
        scale,
        offset: Point::new(anchor.x - content.x * scale, anchor.y - content.y * scale),
    }
}
