//! Role: Keep a vertical scroll viewport attached to the end of growing content.

use std::cell::Cell;
use std::rc::Rc;

use kivact_core::draw::Point;
use kivact_core::view::{NodeRef, ScrollBehavior};
use kivact_core::{Value, effect_on, memo};

/// Queries and commands for a vertical viewport that follows growing content.
pub struct FollowEnd {
    viewport: NodeRef,
    offset: Value<Point>,
    end: Value<Option<f32>>,
    distance: Value<f32>,
    state: Rc<Cell<FollowState>>,
}

impl FollowEnd {
    /// The distance in pixels from the current offset to the end.
    ///
    /// Before layout establishes an end, this reports zero.
    pub fn distance(&self) -> Value<f32> {
        self.distance
    }

    /// Reattach to the end and bring it into view.
    pub fn jump_to_end(&self) {
        if let Some(end) = self.end.get_untracked() {
            let mut state = self.state.get();
            state.locked = true;
            state.expected = Some(end);
            self.state.set(state);
            self.viewport.scroll_to(
                Point::new(self.offset.get_untracked().x, end),
                ScrollBehavior::Instant,
            );
        }
    }
}

/// Follow content growth until the user scrolls away from the end.
///
/// Call during setup and attach both references to the vertical viewport and its
/// measured content. Scrolling back within `reattach_distance` pixels resumes
/// following. An optional history revision preserves the position on prepend.
/// Update that revision in the same batch as the prepended content.
pub fn use_follow_end(
    viewport: NodeRef,
    content: NodeRef,
    reattach_distance: f32,
    history_revision: Option<Value<u64>>,
) -> FollowEnd {
    let viewport_rect = viewport.rect();
    let content_rect = content.rect();
    let offset = viewport.scroll_offset();
    let geometry = memo(move || {
        content_rect
            .get()
            .zip(viewport_rect.get())
            .map(|(content, viewport)| {
                ((content.height - viewport.height).max(0.0), content.height)
            })
    });
    let end = memo(move || geometry.get().map(|(end, _)| end));
    let distance = memo(move || end.get().map_or(0.0, |end| (end - offset.get().y).max(0.0)));
    let observation = memo(move || {
        (
            geometry.get(),
            offset.get(),
            history_revision.map(|revision| revision.get()),
        )
    });
    let state = Rc::new(Cell::new(FollowState::default()));
    let observer = state.clone();
    effect_on(observation, move |&(geometry, offset, revision)| {
        let Some((end, height)) = geometry else {
            return;
        };
        let mut state = observer.get();
        let target = state.observe(end, height, offset.y, revision, reattach_distance);
        observer.set(state);
        if let Some(y) = target {
            viewport.scroll_to(Point::new(offset.x, y), ScrollBehavior::Instant);
        }
    });
    FollowEnd {
        viewport,
        offset,
        end,
        distance,
        state,
    }
}

#[derive(Clone, Copy)]
struct FollowState {
    locked: bool,
    geometry: Option<(f32, f32)>,
    offset: f32,
    expected: Option<f32>,
    revision: Option<u64>,
    history_pending: bool,
}

impl Default for FollowState {
    fn default() -> Self {
        Self {
            locked: true,
            geometry: None,
            offset: 0.0,
            expected: None,
            revision: None,
            history_pending: false,
        }
    }
}

impl FollowState {
    fn observe(
        &mut self,
        end: f32,
        height: f32,
        offset: f32,
        revision: Option<u64>,
        reattach: f32,
    ) -> Option<f32> {
        const EPSILON: f32 = 0.5;
        let previous_geometry = self.geometry.replace((end, height));
        let previous_offset = std::mem::replace(&mut self.offset, offset);
        let acknowledged = self
            .expected
            .is_some_and(|target| (target - offset).abs() <= EPSILON);
        if acknowledged {
            self.expected = None;
        }
        let previous_revision = std::mem::replace(&mut self.revision, revision);
        let Some((previous_end, previous_height)) = previous_geometry else {
            return self.request(offset, end);
        };
        if revision != previous_revision {
            self.history_pending = true;
        }
        if self.history_pending && height - previous_height > EPSILON {
            self.history_pending = false;
            let anchor = self.expected.unwrap_or(previous_offset);
            return self.request(offset, (anchor + height - previous_height).min(end));
        }
        if (end - previous_end).abs() > EPSILON {
            if end > previous_end && !acknowledged && offset < previous_offset - EPSILON {
                self.locked = false;
            }
            if self.locked {
                return self.request(offset, end);
            }
            self.expected = None;
        } else if !acknowledged && (offset - previous_offset).abs() > EPSILON {
            self.expected = None;
            if end - offset <= reattach {
                self.locked = true;
            } else if offset < previous_offset {
                self.locked = false;
            }
        }
        None
    }

    fn request(&mut self, offset: f32, target: f32) -> Option<f32> {
        if (target - offset).abs() <= 0.5 {
            self.expected = None;
            None
        } else {
            self.expected = Some(target);
            Some(target)
        }
    }
}
