//! Role: Single-touch press-to-scroll arbitration over the native scroll chain.
//!
//! Pending contacts keep normal presses within eight logical pixels. A takeover
//! cancels the capture and the activation once (the `Cancel` is sent at once: it
//! needs no text engine) before scrolling. Multi-touch abandons native
//! arbitration so custom pinch handling keeps the existing pointer path.

use super::state::{Axis, TouchCandidate, TouchGesture, TouchPan, TouchTarget, TouchTrack, Track};
use crate::model::Model;
use kivact_types::{NodeId, Point, PointerId, PointerKind};

impl Model {
    pub(crate) fn reserve_touch_gesture(&mut self, pointer: PointerId) {
        if let Some(Track::Touch(track)) = self.pointer.state.tracks.get_mut(&pointer.0)
            && let TouchGesture::Pending { route, .. } = &mut track.gesture
        {
            track.gesture = TouchGesture::Route(std::mem::take(route));
        }
    }

    pub(super) fn touch_candidate(&mut self, pointer: PointerId, hit: Option<NodeId>, at: Point) {
        if self
            .pointer
            .state
            .tracks
            .values()
            .filter(|track| track.source().kind == PointerKind::Touch)
            .count()
            != 1
        {
            for track in self.pointer.state.tracks.values_mut() {
                if let Track::Touch(track) = track {
                    track.gesture = match std::mem::take(&mut track.gesture) {
                        TouchGesture::Pending { route, .. } | TouchGesture::Route(route) => {
                            TouchGesture::Route(route)
                        }
                        TouchGesture::Pan(_) => TouchGesture::default(),
                    };
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
        if let Some(from) = from
            && let Track::Touch(track) = self.pointer.state.track_mut(pointer)
        {
            let route = match std::mem::take(&mut track.gesture) {
                TouchGesture::Route(route) | TouchGesture::Pending { route, .. } => route,
                TouchGesture::Pan(_) => unreachable!("invariant: a new touch press is routed"),
            };
            track.gesture = TouchGesture::Pending {
                route,
                candidate: TouchCandidate {
                    start: at,
                    target: TouchTarget {
                        from,
                        origin: hit.expect("invariant: a scroll ancestor requires a hit"),
                        modal,
                    },
                },
            };
        }
    }

    pub(super) fn touch_move(
        &mut self,
        pointer: PointerId,
        at: Point,
        engine: &mut arvane::Engine,
    ) -> bool {
        let Track::Touch(track) = self.pointer.state.track(pointer) else {
            return false;
        };
        match track.gesture.clone() {
            TouchGesture::Pending { candidate, .. } => {
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
                let Track::Touch(track) = self.pointer.state.track_mut(pointer) else {
                    unreachable!("invariant: touch pan belongs to a touch")
                };
                track.gesture = TouchGesture::Pan(TouchPan {
                    samples: {
                        let mut samples =
                            kivact_types::DragVelocity::new(std::time::Duration::from_millis(100));
                        samples.reset(track.source.time_ns, at);
                        samples
                    },
                    scroller: target,
                    last: at,
                    target: candidate.target,
                    axis,
                });
                self.set_scroll(target, offset);
            }
            TouchGesture::Pan(mut pan) => {
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
                if let Some((target, offset)) = self.scroll_target(pan.target.from, delta, None) {
                    self.set_scroll(target, offset);
                    pan.scroller = target;
                }
                let time = self.pointer.state.track(pointer).source().time_ns;
                pan.samples.record(time, at);
                let Track::Touch(track) = self.pointer.state.track_mut(pointer) else {
                    unreachable!("invariant: touch pan belongs to a touch")
                };
                track.gesture = TouchGesture::Pan(TouchPan { last: at, ..pan });
            }
            TouchGesture::Route(_) => return false,
        }
        self.settle(engine);
        true
    }

    /// Start inertia from the last 100 ms of touch positions.
    pub(crate) fn release_touch_scroll(&mut self, pointer: PointerId) {
        let Track::Touch(track) = self.pointer.state.track(pointer) else {
            return;
        };
        let TouchGesture::Pan(pan) = &track.gesture else {
            return;
        };
        let sampled = pan.samples.pixels_per_ms(track.source.time_ns);
        let mut velocity = Point {
            x: -sampled.x,
            y: -sampled.y,
        };
        match pan.axis {
            Axis::Vertical => velocity.x = 0.0,
            Axis::Horizontal => velocity.y = 0.0,
        }
        self.start_scroll_fling(pan.scroller, velocity);
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
            .filter(|track| match track {
                Track::Touch(TouchTrack {
                    gesture: TouchGesture::Pending { candidate, .. },
                    ..
                }) => !self.touch_target_valid(candidate.target),
                Track::Touch(TouchTrack {
                    gesture: TouchGesture::Pan(pan),
                    ..
                }) => !self.touch_target_valid(pan.target),
                _ => false,
            })
            .map(|track| track.source().id)
            .collect();
        for pointer in stale {
            self.cancel_pointer_input(pointer);
        }
    }
}
