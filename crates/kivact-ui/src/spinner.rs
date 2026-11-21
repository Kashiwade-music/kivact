//! Role: Display a localized loading status with one animation.
pub(crate) mod look;

use crate::ControlSize;
use crate::spinner::look::SpinnerTheme;
use crate::theme::SpinnerState;
use kivact_core::custom::Caller;
use kivact_core::motion::{Animated, Easing, Repeat, Transition};
use kivact_core::state::Value;
use kivact_core::view::{Element, Role, View, node};
use kivact_macros::component;

/// A small animated glyph announced as loading, with an optional label.
#[component]
pub fn spinner(
    #[prop(default = crate::control_size::default_control_size())] size: Value<ControlSize>,
    label_content: Option<View>,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    let theme = kivact_core::style::use_theme().get();
    let period_ms = theme.get(&crate::spinner::look::PERIOD_MS_VAR);
    let reduced_period = *theme
        .ext::<crate::spinner::look::SpinnerTheme>()
        .reduced_period_value();
    let glyph = kivact_core::view::dynamic(
        move || {
            let reduced = kivact_core::platform::is_motion_reduced().get();
            let essential = reduced && reduced_period.is_some();
            let period = if reduced {
                reduced_period.unwrap_or(period_ms)
            } else {
                period_ms
            };
            (period, essential)
        },
        move |(period, essential)| {
            let transition = Transition::tween(period, Easing::Linear).repeat(Repeat::Forever);
            let transition = if essential {
                transition.essential()
            } else {
                transition
            };
            let turn = Animated::from(0.0, 360.0, transition);
            crate::layer::with_base_recipe_style(
                node(),
                |r: &SpinnerTheme| r.root_recipe(),
                &variants,
                move || SpinnerState {
                    size: size.get(),
                    turn: turn.get(),
                },
            )
        },
    );
    node()
        .role(Role::Status)
        .name(kivact_core::i18n::message("spinner.loading", "Loading"))
        .with_node(|node| {
            node.base_style(move || {
                kivact_core::style::style()
                    .flex_direction(kivact_core::style::FlexDirection::Column)
                    .align_items(kivact_core::style::Align::Center)
                    .gap(crate::spinner::look::GAP)
                    .flex_shrink(0)
            })
        })
        .child(glyph)
        .child(label_content)
}
