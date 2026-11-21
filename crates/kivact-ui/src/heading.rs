//! Role: Build semantic headings with theme-owned typography.

pub(crate) mod look;

use crate::heading::look::{HeadingState, HeadingTheme};
use crate::layer::with_base_recipe_style;
use kivact_core::custom::Caller;
use kivact_core::state::Value;
use kivact_core::view::{Role, text};
use kivact_macros::component;

/// Show `value` as a heading, clamping `level` to 1 through 6.
#[component]
pub fn heading(value: Value<String>, level: u8, #[caller] caller: Caller) -> impl Element {
    let variants = caller.variants();
    let level = level.clamp(1, 6);
    with_base_recipe_style(
        text(value)
            .role(Role::Heading)
            .level(u32::from(level))
            .name(value),
        |r: &HeadingTheme| r.root_recipe(),
        &variants,
        move || HeadingState { level },
    )
}
