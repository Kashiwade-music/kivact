//! Role: Compose a semantic badge with icons and optional activation.
pub(crate) mod look;

use crate::badge::look::BadgeTheme;
use crate::layer::{with_base_recipe_style, with_recipe_style};
use crate::theme::BadgeState;
use kivact_core::custom::Caller;
use kivact_core::state::Value;
use kivact_core::view::{View, row, text};
use kivact_interaction::Tone;
use kivact_macros::component;

/// A semantic badge: its label, with an optional leading and trailing icon.
/// A badge with an `on_press` handler is activatable.
#[component]
pub fn badge(
    label: Value<String>,
    #[prop(default = Tone::Neutral)] tone: Value<Tone>,
    icon: Option<View>,
    trailing_icon: Option<View>,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    let actionable = caller.has_press();
    let leading_icon = icon.is_some();
    let has_trailing = trailing_icon.is_some();
    let part = kivact_core::custom::part(move |input| BadgeState {
        tone: tone.get(),
        input,
        actionable,
        leading_icon,
        trailing_icon: has_trailing,
    });
    let state = part.state();
    let mut root = row(()).name(label).part(part);
    if actionable {
        root = root.part(kivact_interaction::use_button().control());
    }
    with_base_recipe_style(
        root,
        |r: &BadgeTheme| r.root_recipe(),
        &variants,
        move || state.get(),
    )
    .child(icon)
    .child(with_recipe_style(
        text(label),
        |r: &BadgeTheme| r.label_recipe(),
        &variants,
        move || state.get(),
    ))
    .child(trailing_icon)
}
