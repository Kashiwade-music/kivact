//! Role: Single-touch press-to-scroll arbitration over the native scroll chain.
//!
//! Pending contacts keep normal presses within eight logical pixels. Takeover
//! cancels capture/activation once before scrolling. Multi-touch abandons native
//! arbitration so custom pinch handling keeps the existing pointer path.

use super::PointerSpace;
use super::state::TouchScroll;
use crate::model::Model;
use kivact_wire::{NodeId, Point, PointerId, PointerKind};

impl Model {
    pub(super) fn touch_candidate(&mut self, pointer: PointerId, hit: Option<NodeId>, at: Point) {
        if self
            .pointer
            .state
            .tracks
            .values()
            .filter(|track| track.source.kind == PointerKind::Touch)
            .count()
            != 1
        {
            for track in self.pointer.state.tracks.values_mut() {
                track.touch_scroll = None;
            }
            return;
        }
        let from = self.tree.chain(hit).into_iter().find(|&id| {
            self.in_scope(id)
                && self
                    .tree
                    .get(id)
                    .is_some_and(|node| node.scroll_axes() != (false, false))
        });
        let modal = self.modal();
        self.pointer.state.track_mut(pointer).touch_scroll = from.map(|from| TouchScroll {
            start: at,
            last: at,
            from,
            origin: hit.expect("invariant: a scroll ancestor requires a hit"),
            modal,
            vertical: None,
        });
    }

    pub(super) fn touch_move(
        &mut self,
        pointer: PointerId,
        at: Point,
        engine: &mut arvane::Engine,
    ) -> bool {
        let Some(mut touch) = self.pointer.state.track(pointer).touch_scroll else {
            return false;
        };
        if !self.touch_scroll_valid(touch) {
            let space = PointerSpace::new(at, engine);
            self.cancel_pointer_input(pointer, &space);
            return false;
        }
        let mut delta = Point {
            x: touch.last.x - at.x,
            y: touch.last.y - at.y,
        };
        if touch.vertical.is_none() {
            delta = Point {
                x: touch.start.x - at.x,
                y: touch.start.y - at.y,
            };
            if delta.x.hypot(delta.y) <= 8.0 {
                return false;
            }
            let vertical = delta.y.abs() >= delta.x.abs();
            let initial = if vertical {
                Point { x: 0.0, y: delta.y }
            } else {
                Point { x: delta.x, y: 0.0 }
            };
            let Some((target, offset)) = self.scroll_target(touch.from, initial, None) else {
                return false;
            };
            let space = PointerSpace::new(at, engine);
            self.cancel_pointer_input(pointer, &space);
            touch.vertical = Some(vertical);
            self.set_scroll(target, offset);
        } else {
            if touch.vertical == Some(true) {
                delta.x = 0.0;
            } else {
                delta.y = 0.0;
            }
            self.scroll_from(touch.from, delta, None);
        }
        touch.last = at;
        self.pointer.state.track_mut(pointer).touch_scroll = Some(touch);
        self.relayout(engine);
        true
    }

    fn touch_scroll_valid(&self, touch: TouchScroll) -> bool {
        self.tree.live(touch.origin)
            && self.tree.shown(touch.origin)
            && self.node(touch.origin).is_some_and(|node| !node.disabled())
            && self.tree.live(touch.from)
            && self.tree.shown(touch.from)
            && self.in_scope(touch.from)
            && self.modal() == touch.modal
            && self.node(touch.from).is_some_and(|node| !node.disabled())
    }

    /// Engine-free tree validation defers only delivery; consume it before
    /// commit boundaries or a later pointer event can release the capture.
    pub(crate) fn flush_touch_cancellations(&mut self, engine: &mut arvane::Engine) {
        let pending: Vec<_> = self
            .pointer
            .state
            .tracks
            .values()
            .filter(|track| track.touch_cancel_pending)
            .map(|track| (track.source.id, track.at))
            .collect();
        for (pointer, at) in pending {
            let space = PointerSpace::new(at, engine);
            self.cancel_pointer_input(pointer, &space);
        }
    }

    pub(super) fn revalidate_touch_scrolls(&mut self) {
        let stale: Vec<_> = self
            .pointer
            .state
            .tracks
            .values()
            .filter(|track| {
                track
                    .touch_scroll
                    .is_some_and(|touch| !self.touch_scroll_valid(touch))
            })
            .map(|track| track.source.id)
            .collect();
        for pointer in stale {
            self.pointer.state.track_mut(pointer).touch_scroll = None;
            self.pointer.state.track_mut(pointer).touch_cancel_pending = true;
            self.pointer.state.track_mut(pointer).press_target = None;
            self.set_chain(pointer, true, Vec::new());
        }
    }
}
