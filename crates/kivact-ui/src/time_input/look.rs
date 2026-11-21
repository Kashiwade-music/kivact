//! Role: The time input recipe state, tokens, recipe table and default looks.

use crate::recipe::recipe_table;
use kivact_core::custom::PartState;
recipe_table! {
    /// The parts of a wall-clock field.
    TimeInputTheme {
        root(root_recipe): PartState = crate::date_parts::look::input,
        wheel(wheel_recipe): PartState = crate::date_parts::look::wheel,
    }
}
