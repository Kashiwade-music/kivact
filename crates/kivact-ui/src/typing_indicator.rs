//! Role: Present a localized typing status with a theme-controlled animation.

pub(crate) mod look;

use crate::layer::{with_base_recipe_style, with_recipe_style};
use crate::typing_indicator::look::{TypingIndicatorState, TypingIndicatorTheme};
use kivact_core::custom::Caller;
use kivact_core::motion::{Animated, Easing, Repeat, Transition};
use kivact_core::view::{LiveRegion, Role, View, node};
use kivact_macros::component;

/// Show that a participant is typing; remove or hide it when typing ends.
///
/// A polite status named "Typing", whose presence is controlled by its
/// caller. `avatar` fills the optional participant avatar slot.
#[component]
pub fn typing_indicator(avatar: Option<View>, #[caller] caller: Caller) -> impl Element {
    let variants = caller.variants();
    let duration = kivact_core::style::use_theme()
        .with(|t| t.get(&crate::typing_indicator::look::PERIOD_MS_VAR));
    let phase = Animated::from(
        0.0,
        std::f64::consts::TAU,
        Transition::tween(duration, Easing::Linear).repeat(Repeat::Forever),
    );
    let state = move || TypingIndicatorState { phase: phase.get() };
    let dots = with_recipe_style(
        node(),
        |r: &TypingIndicatorTheme| r.dots_recipe(),
        &variants,
        state,
    );
    let avatar = avatar.map(|v| {
        with_recipe_style(
            node().child(v),
            |r: &TypingIndicatorTheme| r.avatar_recipe(),
            &variants,
            state,
        )
    });
    with_base_recipe_style(
        node()
            .role(Role::Status)
            .live_region(LiveRegion::Polite)
            .name(kivact_core::i18n::message(
                "typing_indicator.label",
                "Typing",
            )),
        |r: &TypingIndicatorTheme| r.root_recipe(),
        &variants,
        state,
    )
    .child((avatar, dots))
}
