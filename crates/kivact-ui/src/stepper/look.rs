//! Role: The stepper recipe state, tokens, recipe table and default looks.

use crate::recipe::recipe_table;
use crate::variants::Look;
use kivact_core::style::{Align, ColorToken, Direction, Style, Theme, style};
use kivact_core::view::Orientation;
pub(crate) fn stepper_root(t: &Theme, _: &Look<StepperState>) -> Style {
    let p = &t.ext::<crate::tokens::UiTokens>().stepper;
    style()
        .direction(Direction::Row)
        .gap(p.gap)
        .padding(p.padding)
        .font_size(t.text().size)
        .color(ColorToken::Text)
}

pub(crate) fn stepper_item(t: &Theme, look: &Look<StepperItemState>) -> Style {
    let p = &t.ext::<crate::tokens::UiTokens>().stepper;
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .min_width(if look.collapsed {
            0.0
        } else {
            p.min_step_width
        })
        .gap(p.connector_gap)
        .color(if look.index == look.current {
            t.colors().accent
        } else {
            t.colors().text
        })
        .opacity(if look.disabled {
            t.disabled_opacity()
        } else {
            1.0
        })
}

pub(crate) fn stepper_layout(_: &Theme, _: &Look<StepperItemState>) -> Style {
    style().direction(Direction::Row)
}

pub(crate) fn stepper_body(_: &Theme, _: &Look<StepperItemState>) -> Style {
    style().direction(Direction::Column)
}

pub(crate) fn stepper_decoration(_: &Theme, _: &Look<StepperItemState>) -> Style {
    style().size(0, 0)
}

/// Recipe state for an individual step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StepperItemState {
    pub count: usize,
    pub orientation: Orientation,
    pub index: usize,
    pub current: usize,
    pub status: StepStatus,
    pub indicator: StepIndicatorKind,
    pub interactive: bool,
    pub collapsed: bool,
    pub disabled: bool,
    pub optional: bool,
}

/// Actual responsive layout projected from the mounted root and caller data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StepperState {
    pub current: usize,
    pub count: usize,
    pub orientation: Orientation,
    pub collapsed: bool,
}

recipe_table! {
    /// Progress list and individual step recipes.
    StepperRecipes {
        root(root_recipe): StepperState = stepper_root,
        previous(previous_recipe): StepperState = crate::look_common::part,
        next(next_recipe): StepperState = crate::look_common::part,
        item(item_recipe): StepperItemState = stepper_item,
        layout(layout_recipe): StepperItemState = stepper_layout,
        track(track_recipe): StepperItemState = stepper_layout,
        leading(leading_recipe): StepperItemState = stepper_decoration,
        trailing(trailing_recipe): StepperItemState = stepper_decoration,
        progress(progress_recipe): StepperItemState = stepper_decoration,
        body(body_recipe): StepperItemState = stepper_body,
        indicator(indicator_recipe): StepperItemState = crate::look_common::part,
        label(label_recipe): StepperItemState = crate::look_common::part,
        description(description_recipe): StepperItemState = crate::look_common::part,
        content(content_recipe): StepperItemState = crate::look_common::part,
        end_content(end_content_recipe): StepperItemState = crate::look_common::part,
    }
}

/// Semantic status attached to a step.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StepStatus {
    #[default]
    Default,
    Accent,
    Success,
    Warning,
    Error,
}

/// The actual indicator content policy selected on one step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepIndicatorKind {
    Auto,
    Number,
    None,
    Custom,
}
