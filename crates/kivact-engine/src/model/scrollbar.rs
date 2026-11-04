//! Role: Scrollbar geometry shared by paint and pointer handling.
//!
//! The one geometry of a scroll container's bars: painting draws them, hit
//! testing and the thumb drag read the same tracks and thumbs.
//!
//! Invariants:
//! - Only a drawn node with a scrolling axis whose content overflows on
//!   that axis has a bar there; a node without a scrolling axis computes
//!   no geometry at all.
//! - A bar lies over the box's right or bottom edge, as wide as the
//!   node's scrollbar style (its paint with state styles and transitions;
//!   6 px by default); a width that is not finite or not above 0 hides it.
//!   With both bars, each is shortened by the other's width.
//! - A thumb is as long as the visible part of the content is of it, at
//!   least 18 px, and sits along its track as far as the clamped offset
//!   is of the scroll range.
use crate::model::Model;
use kivact_types::{NodeId, Point, Rect, ScrollbarProps};
#[derive(Clone, Copy)]
pub(crate) struct Bar {
    pub(crate) track: Rect,
    pub(crate) thumb: Rect,
    pub(crate) vertical: bool,
    pub(crate) range: f64,
    pub(crate) travel: f64,
}
impl Model {
    pub(crate) fn scrollbar_bars(&self, id: NodeId) -> [Option<Bar>; 2] {
        let Some(n) = self.node(id) else {
            return [None, None];
        };
        let axes = n.scroll_axes();
        if axes == (false, false) {
            return [None, None];
        }
        if !self.tree().drawn(id) {
            return [None, None];
        }
        let props = self.scrollbar_style(id);
        let width = props.width;
        if !width.is_finite() || width <= 0.0 {
            return [None, None];
        }
        let Some(layout) = self.computed_layout(id) else {
            return [None, None];
        };
        let size = layout.size;
        let content = layout.content_size;
        let enabled = (
            axes.0 && content.width > size.width,
            axes.1 && content.height > size.height,
        );
        let make = |vertical: bool| {
            if if vertical { !enabled.1 } else { !enabled.0 } {
                return None;
            }
            let viewport = if vertical { size.height } else { size.width };
            let content = if vertical {
                content.height
            } else {
                content.width
            };
            let length = (viewport
                - if if vertical { enabled.0 } else { enabled.1 } {
                    width
                } else {
                    0.0
                })
            .max(0.0);
            if length <= 0.0 {
                return None;
            }
            let thumb_len = (length * viewport / content).max(18.0).min(length);
            let range = (content - viewport).max(0.0);
            let travel = length - thumb_len;
            let offset = if vertical { n.scroll().y } else { n.scroll().x };
            let shift = travel * offset.clamp(0.0, range) / range;
            let track = if vertical {
                Rect {
                    x: (size.width - width).max(0.0),
                    y: 0.0,
                    width: width.min(size.width),
                    height: length,
                }
            } else {
                Rect {
                    x: 0.0,
                    y: (size.height - width).max(0.0),
                    width: length,
                    height: width.min(size.height),
                }
            };
            let thumb = if vertical {
                Rect {
                    y: shift,
                    height: thumb_len,
                    ..track
                }
            } else {
                Rect {
                    x: shift,
                    width: thumb_len,
                    ..track
                }
            };
            Some(Bar {
                track,
                thumb,
                vertical,
                range,
                travel,
            })
        };
        [make(false), make(true)]
    }
    pub(crate) fn scrollbar_at(&self, id: NodeId, point: Point) -> Option<Bar> {
        self.scrollbar_bars(id)
            .into_iter()
            .flatten()
            .find(|bar| bar.track.contains(point))
    }
    pub(crate) fn scrollbar_style(&self, id: NodeId) -> ScrollbarProps {
        self.effective_paint(id)
            .scrollbar
            .as_deref()
            .copied()
            .unwrap_or_default()
    }
}
