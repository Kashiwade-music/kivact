//! Role: Time item and lane rectangles for painting and hit testing.
use crate::editors::shared::metrics::{RULER, SIDEBAR};
use crate::editors::shared::model::Lane;
use crate::editors::time::model::TimeItem;
use kivact::draw::{Point, Rect};
pub(crate) fn item_rect<T: TimeItem>(
    item: &T,
    lanes: &[Lane],
    height: f64,
    zoom: f64,
    pan: Point,
) -> Option<Rect> {
    if !item.valid() {
        return None;
    }
    let row = lanes.iter().position(|lane| lane.id == item.lane())?;
    let bar = item.duration() > 0.0;
    let x = SIDEBAR + (item.time() * zoom) + pan.x;
    let y = RULER + row as f64 * height;
    Some(if bar {
        Rect::new(x, y + 3.0, item.duration() * zoom, height - 7.0)
    } else {
        Rect::new(x - 4.0, y + (height - 8.0) * 0.5, 8.0, 8.0)
    })
}

/// Shared row geometry for painting, region hit testing and embedded cells.
pub(crate) fn lane_rect(index: usize, height: f64) -> Rect {
    Rect::new(0.0, RULER + index as f64 * height, SIDEBAR, height)
}
