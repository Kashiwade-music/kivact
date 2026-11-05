//! Role: Passive uniform content-to-view coordinate transforms.
use crate::{Point, Size};

/// Content-to-view coordinates: `view = content * scale + offset`.
/// Invalid (non-positive or non-finite) scales are treated as 1 by operations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewTransform {
    pub offset: Point,
    pub scale: f64,
}
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
fn valid_scale(scale: f64) -> f64 {
    if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    }
}
