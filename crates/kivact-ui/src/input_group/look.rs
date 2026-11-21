//! Role: The input group recipe state, tokens, recipe table and default looks.

use crate::control_size::ControlSize;
use crate::recipe::recipe_table;
use crate::variants::Look;
use kivact_core::style::{Align, FlexDirection, Justify, Style, Theme, style};
use kivact_core::view::ReadingDirection;
pub(crate) fn group_root(t: &Theme, _: &Look<InputGroupState>) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .gap(t.get(&kivact_core::style::space::SM_VAR))
        .min_width(0)
}

pub(crate) fn group_label(t: &Theme, _: &Look<InputGroupState>) -> Style {
    style()
        .font_size(t.get(&kivact_core::style::font::SIZE_VAR))
        .font_weight(t.get(&kivact_core::style::font::WEIGHT_STRONG_VAR))
}

pub(crate) fn group_members(_: &Theme, s: &Look<InputGroupState>) -> Style {
    style()
        .flex_direction(if s.direction == kivact_core::view::ReadingDirection::Rtl {
            FlexDirection::RowReverse
        } else {
            FlexDirection::Row
        })
        .align_items(Align::Stretch)
        .min_width(0)
}

pub(crate) fn group_addon(t: &Theme, s: &Look<InputGroupState>) -> Style {
    style()
        .height(crate::look_common::height(t, s.size))
        .padding_x(t.get(&kivact_core::style::space::SM_VAR))
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .border(1, kivact_core::style::color::BORDER)
        .background(kivact_core::style::color::SURFACE_ALT)
        .color(kivact_core::style::color::TEXT)
        .font_size(t.get(&kivact_core::style::font::SIZE_VAR))
}

pub(crate) fn group_member(t: &Theme, s: &Look<InputGroupMemberState>) -> Style {
    let rtl = s.group.direction == kivact_core::view::ReadingDirection::Rtl;
    let (left, right) = if rtl {
        (s.last, s.first)
    } else {
        (s.first, s.last)
    };
    let radius = t.get(&kivact_core::style::radius::MD_VAR);
    let base = style()
        .height(crate::look_common::height(t, s.group.size))
        .border_radius_corners(
            if left { radius } else { 0.0 },
            if right { radius } else { 0.0 },
            if right { radius } else { 0.0 },
            if left { radius } else { 0.0 },
        );
    let base = if s.first {
        base
    } else if rtl {
        base.margin_right(-1)
    } else {
        base.margin_left(-1)
    };
    if s.focused { base.z_index(1) } else { base }
}

pub(crate) fn group_block(_: &Theme, _: &Look<InputGroupState>) -> Style {
    style()
}

/// Input group size and reading direction passed to its recipes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InputGroupState {
    pub input: kivact_core::custom::PartState,
    pub focused: bool,
    pub invalid: bool,
    pub has_start: bool,
    pub has_end: bool,
    pub has_block_start: bool,
    pub has_block_end: bool,
    pub size: ControlSize,
    pub direction: ReadingDirection,
}

/// A member's position, inherited size, and actual focus-within observation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InputGroupMemberState {
    pub kind: InputGroupMemberKind,
    pub group: InputGroupState,
    pub first: bool,
    pub last: bool,
    pub focused: bool,
}

/// The source slot of one actual inline input-group member.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputGroupMemberKind {
    Start,
    Control,
    End,
}

impl Default for InputGroupState {
    fn default() -> Self {
        Self {
            input: Default::default(),
            focused: false,
            invalid: false,
            has_start: false,
            has_end: false,
            has_block_start: false,
            has_block_end: false,
            size: ControlSize::Medium,
            direction: ReadingDirection::Ltr,
        }
    }
}

recipe_table! {
    /// Recipes for a labelled group of controls and its text adornments.
    InputGroupRecipes {
        root(root_recipe): InputGroupState = group_root,
        label(label_recipe): InputGroupState = group_label,
        members(members_recipe): InputGroupState = group_members,
        member(member_recipe): InputGroupMemberState = group_member,
        addon(addon_recipe): InputGroupState = group_addon,
        block_start(block_start_recipe): InputGroupState = group_block,
        block_end(block_end_recipe): InputGroupState = group_block,
    }
}
