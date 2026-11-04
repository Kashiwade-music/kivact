//! Role: Keep a vertical scroll viewport attached to the end of growing content.

use super::machine::{Effect, Event, Machine, Sample, State};
use kivact_types::Revision;

use kivact_core::geometry::Point;
use kivact_core::state::{MachineHandle, Value, effect, machine, memo};
use kivact_core::view::{NodeRef, ScrollBehavior};

/// Queries and commands for a vertical viewport that follows growing content.
pub struct FollowEndApi {
    end: Value<Option<f64>>,
    distance: Value<f64>,
    state: MachineHandle<Machine>,
}

impl FollowEndApi {
    /// The distance in pixels from the current offset to the end.
    ///
    /// Before layout establishes an end, this reports zero.
    pub fn distance(&self) -> Value<f64> {
        self.distance
    }

    /// Reattach to the end and bring it into view.
    pub fn jump_to_end(&self) {
        if let Some(end) = self.end.get_untracked() {
            self.state.send(Event::Jump(end));
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
    reattach_distance: impl Into<f64>,
    history_revision: Option<Value<u64>>,
) -> FollowEndApi {
    let reattach_distance = reattach_distance.into();
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
    let state = machine(
        State::default(),
        move || Machine {
            reattach: reattach_distance,
        },
        move |_, command| match command {
            Effect::Scroll(y) => viewport.scroll_to(
                Point::new(offset.get_untracked().x, y),
                ScrollBehavior::Instant,
            ),
        },
    );
    effect(move || {
        let Some((end, height)) = geometry.get() else {
            return;
        };
        state.send(Event::Observe(Sample {
            end,
            height,
            offset: offset.get().y,
            revision: history_revision.map(|revision| Revision(revision.get())),
        }));
    });
    FollowEndApi {
        end,
        distance,
        state,
    }
}
