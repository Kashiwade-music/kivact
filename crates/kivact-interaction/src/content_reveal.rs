//! Role: Animate a container to and from its measured content height.

use kivact_core::motion::{Animated, Transition, animated};
use kivact_core::style::{Overflow, Style, style};
use kivact_core::view::{IntoValue, NodeRef};
use kivact_core::{Value, memo};

/// An interruptible height reveal driven by a reactive open state.
#[derive(Clone, Copy)]
pub struct ContentReveal {
    shown: Value<bool>,
    progress: Animated<f64>,
}

/// Animate between zero height and the content's natural height.
///
/// Call during setup. The content is initially laid out at its current state.
#[track_caller]
pub fn use_content_reveal(shown: impl IntoValue<bool>, transition: Transition) -> ContentReveal {
    let shown = shown.into_value();
    let progress = animated(move || if shown.get() { 1.0 } else { 0.0 }, transition);
    ContentReveal { shown, progress }
}

impl ContentReveal {
    /// Whether to retain the content through its closing animation.
    pub fn present(self) -> bool {
        self.shown.get() || self.leaving()
    }

    /// Whether the content is closing and should stop accepting interaction.
    pub fn leaving(self) -> bool {
        !self.shown.get() && self.progress.get() > 0.0
    }

    /// Style for the content's container, clipped while partially revealed.
    ///
    /// Call during setup. Attach `measured` to a nonshrinking child whose box
    /// reports the natural content height. At full height the container uses
    /// its natural layout.
    #[track_caller]
    pub fn container_style(self, measured: NodeRef) -> Value<Style> {
        let natural = measured.rect();
        memo(move || {
            let share = self.progress.get().clamp(0.0, 1.0);
            if share < 1.0 {
                style()
                    .overflow(Overflow::Hidden)
                    .height(natural.get().map_or(0.0, |rect| rect.height) * share)
            } else {
                style()
            }
        })
    }
}
