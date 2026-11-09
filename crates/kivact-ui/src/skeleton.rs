//! Role: Render loading placeholders and pause their clock for reduced motion.
use crate::variants::{Variant, Variants};
use crate::{SkeletonShape, SkeletonState};
use kivact_core::motion::{Animated, Easing, Repeat, Stagger, Transition};
use kivact_core::state::{IntoValue, Value, effect};
use kivact_core::view::{IntoView, Node, NodeBuilder, View, column, for_index, node};
pub struct Skeleton {
    root: Node,
    lines: Value<usize>,
    shape: Value<SkeletonShape>,
    variants: Variants,
    index: usize,
}
pub fn skeleton() -> Skeleton {
    Skeleton {
        root: column(()),
        lines: 1usize.into_value(),
        shape: SkeletonShape::default().into_value(),
        variants: Variants::default(),
        index: 0,
    }
}
impl Skeleton {
    /// Offset this placeholder within a staggered loading group.
    pub fn index(mut self, index: usize) -> Self {
        self.index = index;
        self
    }
    pub fn lines(mut self, lines: impl IntoValue<usize>) -> Self {
        self.lines = lines.into_value();
        self
    }
    pub fn shape(mut self, shape: impl IntoValue<SkeletonShape>) -> Self {
        self.shape = shape.into_value();
        self
    }
    pub fn variant(mut self, variant: impl Variant) -> Self {
        self.variants.add(variant);
        self
    }
}
impl NodeBuilder for Skeleton {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Skeleton {
    fn into_view(self) -> View {
        let tokens = kivact_core::style::use_tokens::<crate::UiTokens>();
        let period = tokens.indicators.period_ms;
        // The placeholder is the last of the group 0..=index, so a linear stagger
        // from the first item gives `index * stagger_ms`.
        let stagger = Stagger::new(tokens.indicators.stagger_ms);
        let delay = tokens.indicators.delay_ms + stagger.delay(self.index, self.index + 1);
        let transition = tokens
            .indicators
            .skeleton_transition
            .clone()
            .unwrap_or_else(|| Transition::tween(period as f64, Easing::EaseInOut));
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
        let lines = self.lines;
        let shape = self.shape;
        let variants = self.variants;
        self.root
            .base_style(move || {
                kivact_core::style::style().gap(
                    kivact_core::style::use_tokens::<crate::UiTokens>()
                        .indicators
                        .gap,
                )
            })
            .child(for_index(
                move || (0..lines.get()).collect::<Vec<_>>(),
                move |_, _| {
                    crate::layer::with_base_recipe_style(
                        node(),
                        |r| r.skeleton_recipe(),
                        &variants,
                        move || SkeletonState {
                            shape: shape.get(),
                            pulse: if reduced.get() { 1.0 } else { pulse.get() },
                        },
                    )
                },
            ))
            .into_view()
    }
}
