//! Role: The stepper recipe state, recipe table and default looks.

use crate::recipe::recipe_table;
use crate::variants::Look;
use kivact_core::style::{Align, FlexDirection, Style, Theme, style};
use kivact_core::style::{Length, ThemeVar};
use kivact_core::view::Orientation;

/// The gap between the parts of a step, in logical px.
pub static GAP_VAR: ThemeVar<f64> = ThemeVar::new("stepper.gap", 8.0);
/// [`GAP_VAR`] as a length for a style.
pub const GAP: Length = Length::var(&GAP_VAR);

/// The padding of a stepper, in logical px.
pub static PADDING_VAR: ThemeVar<f64> = ThemeVar::new("stepper.padding", 4.0);
/// [`PADDING_VAR`] as a length for a style.
pub const PADDING: Length = Length::var(&PADDING_VAR);

/// The minimum width of a step, in logical px.
pub static MIN_STEP_WIDTH_VAR: ThemeVar<f64> = ThemeVar::new("stepper.min_step_width", 96.0);
/// [`MIN_STEP_WIDTH_VAR`] as a length for a style.
pub const MIN_STEP_WIDTH: Length = Length::var(&MIN_STEP_WIDTH_VAR);

/// The gap around a step's connector, in logical px.
pub static CONNECTOR_GAP_VAR: ThemeVar<f64> = ThemeVar::new("stepper.connector_gap", 8.0);
/// [`CONNECTOR_GAP_VAR`] as a length for a style.
pub const CONNECTOR_GAP: Length = Length::var(&CONNECTOR_GAP_VAR);

pub(crate) fn stepper_root(t: &Theme, _: &Look<StepperState>) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .gap(GAP)
        .padding(PADDING)
        .font_size(t.get(&kivact_core::style::font::SIZE_VAR))
        .color(kivact_core::style::color::TEXT)
}

pub(crate) fn stepper_item(t: &Theme, look: &Look<StepperItemState>) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .min_width(if look.collapsed {
            0.0
        } else {
            t.get(&MIN_STEP_WIDTH_VAR)
        })
        .gap(CONNECTOR_GAP)
        .color(if look.index == look.current {
            t.get(&kivact_core::style::color::ACCENT_VAR)
        } else {
            t.get(&kivact_core::style::color::TEXT_VAR)
        })
        .opacity(if look.disabled {
            t.get(&kivact_core::style::control::DISABLED_OPACITY_VAR)
        } else {
            1.0
        })
}

pub(crate) fn stepper_layout(_: &Theme, _: &Look<StepperItemState>) -> Style {
    style().flex_direction(FlexDirection::Row)
}

pub(crate) fn stepper_body(_: &Theme, _: &Look<StepperItemState>) -> Style {
    style().flex_direction(FlexDirection::Column)
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
    StepperTheme {
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
