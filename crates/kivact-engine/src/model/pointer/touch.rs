//! Role: Single-touch press-to-scroll arbitration over the native scroll chain.
//!
//! Pending contacts keep normal presses within eight logical pixels. A takeover
//! cancels the capture and the activation once (the `Cancel` is sent at once: it
//! needs no text engine) before scrolling. Multi-touch abandons native
//! arbitration so custom pinch handling keeps the existing pointer path.

use super::state::{Axis, Gesture, TouchCandidate, TouchPan, TouchTarget};
use crate::model::Model;
use kivact_types::{NodeId, Point, PointerId, PointerKind};

impl Model {
    pub(crate) fn reserve_touch_gesture(&mut self, pointer: PointerId) {
        if let Some(track) = self.pointer.state.tracks.get_mut(&pointer.0)
            && let Gesture::Route(route) = &mut track.gesture
        {
            route.touch = None;
        }
    }

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
                match &mut track.gesture {
                    Gesture::Route(route) => route.touch = None,
                    Gesture::Pan(_) => track.gesture = Gesture::default(),
                    Gesture::Scrollbar(_) => {}
                }
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
        self.pointer.state.track_mut(pointer).route_mut().touch = from.map(|from| TouchCandidate {
            start: at,
            target: TouchTarget {
                from,
                origin: hit.expect("invariant: a scroll ancestor requires a hit"),
                modal,
            },
        });
    }

    pub(super) fn touch_move(
        &mut self,
        pointer: PointerId,
        at: Point,
        engine: &mut arvane::Engine,
    ) -> bool {
        match self.pointer.state.track(pointer).gesture.clone() {
            Gesture::Route(route) => {
                let Some(candidate) = route.touch else {
                    return false;
                };
                if !self.touch_target_valid(candidate.target) {
                    self.cancel_pointer_input(pointer);
                    return false;
                }
                let delta = Point {
                    x: candidate.start.x - at.x,
                    y: candidate.start.y - at.y,
                };
                if delta.x.hypot(delta.y) <= 8.0 {
                    return false;
                }
                let axis = if delta.y.abs() >= delta.x.abs() {
                    Axis::Vertical
                } else {
                    Axis::Horizontal
                };
                let initial = match axis {
                    Axis::Vertical => Point { x: 0.0, y: delta.y },
                    Axis::Horizontal => Point { x: delta.x, y: 0.0 },
                };
                let Some((target, offset)) =
                    self.scroll_target(candidate.target.from, initial, None)
                else {
                    return false;
                };
                self.cancel_pointer_input(pointer);
                self.pointer.state.track_mut(pointer).gesture = Gesture::Pan(TouchPan {
                    last: at,
                    target: candidate.target,
                    axis,
                });
                self.set_scroll(target, offset);
            }
            Gesture::Pan(pan) => {
                if !self.touch_target_valid(pan.target) {
                    self.cancel_pointer_input(pointer);
                    return false;
                }
                let mut delta = Point {
                    x: pan.last.x - at.x,
                    y: pan.last.y - at.y,
                };
                match pan.axis {
                    Axis::Vertical => delta.x = 0.0,
                    Axis::Horizontal => delta.y = 0.0,
                }
                self.scroll_from(pan.target.from, delta, None);
                self.pointer.state.track_mut(pointer).gesture =
                    Gesture::Pan(TouchPan { last: at, ..pan });
            }
            Gesture::Scrollbar(_) => return false,
        }
        self.settle(engine);
        true
    }

    fn touch_target_valid(&self, target: TouchTarget) -> bool {
        self.tree.live(target.origin)
            && self.tree.shown(target.origin)
            && self
                .node(target.origin)
                .is_some_and(|node| !node.disabled())
            && self.tree.live(target.from)
            && self.tree.shown(target.from)
            && self.in_scope(target.from)
            && self.modal() == target.modal
            && self.node(target.from).is_some_and(|node| !node.disabled())
    }

    /// A commit invalidated the touch scrolls whose target is gone, hidden,
    /// disabled or out of the modal: each such contact is cancelled now,
    /// before the commit's boundary.
    pub(super) fn revalidate_touch_scrolls(&mut self) {
        let stale: Vec<_> = self
            .pointer
            .state
            .tracks
            .values()
            .filter(|track| match &track.gesture {
                Gesture::Route(route) => route
                    .touch
                    .is_some_and(|candidate| !self.touch_target_valid(candidate.target)),
                Gesture::Pan(pan) => !self.touch_target_valid(pan.target),
                Gesture::Scrollbar(_) => false,
            })
            .map(|track| track.source.id)
            .collect();
        for pointer in stale {
            self.cancel_pointer_input(pointer);
        }
    }
}
