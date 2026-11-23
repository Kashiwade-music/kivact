//! Role: The date input recipe state, recipe table and default looks.

use crate::recipe::recipe_table;
use kivact_core::custom::PartState;
use kivact_core::style::{Length, ThemeVar};

/// The height of a date field's wheel row, in logical px.
pub static WHEEL_ROW_HEIGHT_VAR: ThemeVar<f64> = ThemeVar::new("date_input.wheel_row_height", 40.0);
/// [`WHEEL_ROW_HEIGHT_VAR`] as a length for a style.
pub const WHEEL_ROW_HEIGHT: Length = Length::var(&WHEEL_ROW_HEIGHT_VAR);
/// The width of a date field's wheel, in logical px.
pub static WHEEL_WIDTH_VAR: ThemeVar<f64> = ThemeVar::new("date_input.wheel_width", 80.0);
/// [`WHEEL_WIDTH_VAR`] as a length for a style.
pub const WHEEL_WIDTH: Length = Length::var(&WHEEL_WIDTH_VAR);

recipe_table! {
    /// Date and time field layouts and touch wheel surfaces.
    DateInputTheme {
        root(root_recipe): PartState = crate::date_parts::look::input,
        wheel(wheel_recipe): PartState = crate::date_parts::look::wheel,
    }
}
