//! Role: The date range input recipe state, tokens, recipe table and default looks.

use crate::recipe::recipe_table;
use kivact_core::custom::PartState;
recipe_table! {
    /// The parts of a date-range field.
    DateRangeInputRecipes { root(root_recipe): PartState = crate::date_parts::look::input, }
}
