//! Role: Present a localized typing status with a theme-controlled animation.
use crate::TypingIndicatorState;
use crate::variants::{Variant, Variants};
use kivact_core::motion::{Animated, Easing, Repeat, Transition};
use kivact_core::view::{IntoView, LiveRegion, Node, NodeBuilder, Role, View, node};
/// A polite status whose presence is controlled by its caller.
pub struct TypingIndicator {
    root: Node,
    avatar: Option<View>,
    variants: Variants,
}
/// Show that a participant is typing; remove or hide it when typing ends.
pub fn typing_indicator() -> TypingIndicator {
    TypingIndicator {
        root: node()
            .role(Role::Status)
            .live_region(LiveRegion::Polite)
            .name(kivact_core::i18n::message(
                "typing_indicator.label",
                "Typing",
            )),
        avatar: None,
        variants: Variants::default(),
    }
}
impl TypingIndicator {
    /// Fill the optional participant avatar slot.
    pub fn avatar(mut self, avatar: impl IntoView) -> Self {
        self.avatar = Some(avatar.into_view());
        self
    }
    /// Add an appearance read by the theme recipe.
    pub fn variant(mut self, variant: impl Variant) -> Self {
        self.variants.add(variant);
        self
    }
}
impl NodeBuilder for TypingIndicator {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for TypingIndicator {
    fn into_view(self) -> View {
        let duration = kivact_core::style::use_tokens::<crate::UiTokens>()
            .chat
            .typing_period_ms;
        let phase = Animated::from(
            0.0,
            std::f64::consts::TAU,
            Transition::tween(duration as f64, Easing::Linear).repeat(Repeat::Forever),
        );
        let state = move || TypingIndicatorState { phase: phase.get() };
        let dots = crate::layer::with_recipe_style(
            node(),
            |r| r.typing_indicator_recipes().dots_recipe(),
            &self.variants,
            state,
        );
        let avatar = self.avatar.map(|v| {
            crate::layer::with_recipe_style(
                node().child(v),
                |r| r.typing_indicator_recipes().avatar_recipe(),
                &self.variants,
                state,
            )
        });
        crate::layer::with_base_recipe_style(
            self.root,
            |r| r.typing_indicator_recipes().root_recipe(),
            &self.variants,
            state,
        )
        .child((avatar, dots))
        .into_view()
    }
}
