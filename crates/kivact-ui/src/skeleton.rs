//! Role: Render loading placeholders and pause their clock for reduced motion.

pub(crate) mod look;

pub use look::SkeletonShape;

use crate::skeleton::look::{SkeletonShape as Shape, SkeletonState, SkeletonTheme};
use kivact_core::custom::Caller;
use kivact_core::motion::{Animated, Easing, Repeat, Stagger, Transition};
use kivact_core::state::{Value, effect};
use kivact_core::view::{column, for_index, node};
use kivact_macros::component;

/// A loading placeholder that pulses. `index` offsets it within a staggered group.
#[component]
pub fn skeleton(
    #[prop(default = 1usize)] lines: Value<usize>,
    #[prop(default = Shape::default())] shape: Value<Shape>,
    #[prop(default = 0usize)] index: usize,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    let theme = kivact_core::style::use_theme().get();
    let period = theme.get(&crate::skeleton::look::PERIOD_MS_VAR);
    // The placeholder is the last of the group 0..=index, so a linear stagger
    // from the first item gives `index * stagger_ms`.
    let stagger = Stagger::new(theme.get(&crate::skeleton::look::STAGGER_MS_VAR));
    let delay = theme.get(&crate::skeleton::look::DELAY_MS_VAR) + stagger.delay(index, index + 1);
    let transition = theme
        .ext::<crate::skeleton::look::SkeletonTheme>()
        .transition_value()
        .clone()
        .unwrap_or_else(|| Transition::tween(period, Easing::EaseInOut));
    let pulse = Animated::from(
        1.0,
        0.5,
        transition.delay(delay).repeat(Repeat::Forever).alternate(),
    );
    let reduced = kivact_core::platform::is_motion_reduced();
    effect(move || {
        if reduced.get() {
            pulse.pause();
        } else {
            pulse.resume();
        }
    });
    column(())
        .base_style(move || kivact_core::style::style().gap(crate::skeleton::look::GAP))
        .child(for_index(
            move || (0..lines.get()).collect::<Vec<_>>(),
            move |_, _| {
                crate::layer::with_base_recipe_style(
                    node(),
                    |r: &SkeletonTheme| r.root_recipe(),
                    &variants,
                    move || SkeletonState {
                        shape: shape.get(),
                        pulse: if reduced.get() { 1.0 } else { pulse.get() },
                    },
                )
            },
        ))
}
