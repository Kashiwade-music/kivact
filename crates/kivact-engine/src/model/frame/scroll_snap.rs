//! Role: Settle native touch scrolling on child alignment stops.
use crate::model::Model;
use kivact_types::{NodeId, Point, Position, ScrollSnap, nearest_snap};
impl Model {
    pub(crate) fn settle_scroll_snap(&mut self, id: NodeId) {
        let Some(node) = self.node(id) else { return };
        let snap = node
            .layout()
            .map_or(ScrollSnap::None, |layout| layout.scroll_snap);
        if snap == ScrollSnap::None {
            return;
        }
        let axes = node.scroll_axes();
        let at = node.scroll();
        let Some(viewport) = self.computed_layout(id) else {
            return;
        };
        let max = Point {
            x: (viewport.content_size.width - viewport.size.width).max(0.0),
            y: (viewport.content_size.height - viewport.size.height).max(0.0),
        };
        let align = match snap {
            ScrollSnap::None | ScrollSnap::Start => 0.0,
            ScrollSnap::Center => 0.5,
            ScrollSnap::End => 1.0,
        };
        let mut x = Vec::new();
        let mut y = Vec::new();
        for child in self.tree.flat_children(id) {
            let Some(node) = self.node(child) else {
                continue;
            };
            if node.hidden()
                || node.layout().is_some_and(|layout| {
                    matches!(layout.position, Position::Absolute | Position::Fixed)
                })
            {
                continue;
            }
            let Some(layout) = self.computed_layout(child) else {
                continue;
            };
            x.push(
                (layout.location.x + align * (layout.size.width - viewport.size.width))
                    .clamp(0.0, max.x),
            );
            y.push(
                (layout.location.y + align * (layout.size.height - viewport.size.height))
                    .clamp(0.0, max.y),
            );
        }
        x.sort_by(f64::total_cmp);
        x.dedup();
        y.sort_by(f64::total_cmp);
        y.dedup();
        let target = Point {
            x: if axes.0 {
                nearest_snap(at.x, &x).map_or(at.x, |index| x[index])
            } else {
                at.x
            },
            y: if axes.1 {
                nearest_snap(at.y, &y).map_or(at.y, |index| y[index])
            } else {
                at.y
            },
        };
        self.set_scroll(id, target);
    }
}
