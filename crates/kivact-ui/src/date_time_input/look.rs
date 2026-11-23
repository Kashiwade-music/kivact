//! Role: The date time input recipe state, recipe table and default looks.

use crate::recipe::recipe_table;
use kivact_core::custom::PartState;
recipe_table! {
    /// The parts of a date-and-time field.
    DateTimeInputTheme { root(root_recipe): PartState = crate::date_parts::look::input, }
}
