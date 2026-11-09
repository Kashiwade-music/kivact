//! Role: Passive layout snapshots and tokens shared by action group recipes.
use crate::values::{ControlSize, ToggleButtonState};
use kivact_core::style::Color;
use kivact_core::view::Orientation;
/// Passive button group metrics and colors.
#[derive(Clone, Debug, PartialEq)]
pub struct ButtonGroupTokens {
    pub gap: f64,
    pub joined: bool,
    pub radius: f64,
    pub divider: Color,
    pub solid_divider: Color,
    pub text_background: Color,
    pub text_color: Color,
}
impl Default for ButtonGroupTokens {
    fn default() -> Self {
        Self {
            gap: 0.0,
            joined: true,
            radius: 0.0,
            divider: Color::TRANSPARENT,
            solid_divider: Color::TRANSPARENT,
            text_background: Color::TRANSPARENT,
            text_color: Color::BLACK,
        }
    }
}

/// Layout supplied to group and auxiliary cell recipes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ButtonGroupState {
    pub orientation: Orientation,
    pub size: ControlSize,
    // xtask-allow(state-fields): independent: a derived position or construction input; every combination is valid
    pub rtl: bool,
}

/// Position of a child within a joined group, derived from the child list.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ButtonGroupMemberState {
    pub layout: ButtonGroupState,
    // xtask-allow(state-fields): independent: a derived position or construction input; every combination is valid
    pub first: bool,
    // xtask-allow(state-fields): independent: a derived position or construction input; every combination is valid
    pub last: bool,
}

/// Passive spacing for toggle groups.
#[derive(Clone, Debug, PartialEq)]
pub struct ToggleGroupTokens {
    pub gap: f64,
}
impl Default for ToggleGroupTokens {
    fn default() -> Self {
        Self { gap: 4.0 }
    }
}

/// Layout supplied to the group recipe.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToggleGroupState {
    pub orientation: Orientation,
    pub size: ControlSize,
    // xtask-allow(state-fields): option: absent means the component's default
    pub spacing: Option<f64>,
    // xtask-allow(state-fields): independent: a derived position or construction input; every combination is valid
    pub rtl: bool,
}

/// A toggle's actual state together with its derived position and layout.
#[derive(Clone, Debug, PartialEq)]
pub struct ToggleGroupItemState {
    pub toggle: ToggleButtonState,
    pub layout: ToggleGroupState,
    // xtask-allow(state-fields): independent: a derived position or construction input; every combination is valid
    pub first: bool,
    // xtask-allow(state-fields): independent: a derived position or construction input; every combination is valid
    pub last: bool,
}
impl std::ops::Deref for ToggleGroupItemState {
    type Target = ToggleButtonState;
    fn deref(&self) -> &Self::Target {
        &self.toggle
    }
}
