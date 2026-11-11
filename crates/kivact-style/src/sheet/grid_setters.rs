//! Role: The `Style` setters for CSS grid tracks and placement.

#[cfg(doc)]
use crate::grid::Track;
use crate::grid::{GridAutoFlow, GridPlacement, Tracks};
use kivact_types::Align;

use super::fields::Style;

macro_rules! grid_setter {
    ($(#[$m:meta])* $name:ident, $field:ident, $t:ty, $conv:expr) => {
        $(#[$m])*
        pub fn $name(mut self, v: $t) -> Self {
            let set = self
                .flow
                .as_deref()
                .and_then(|flow| flow.grid.as_deref())
                .is_some_and(|g| g.$field.is_some());
            if set {
                self.duplicates += 1;
            }
            #[allow(clippy::redundant_closure_call, reason = "`$conv` may be a closure literal called in place")]
            {
                self.flow_mut().grid.get_or_insert_with(Default::default).$field = Some(($conv)(v.into()));
            }
            self
        }
    };
}

impl Style {
    grid_setter!(
        /// CSS `grid-template-columns`: the grid's columns, a [`Track`] or a
        /// tuple of them: `(248, Track::fr(1))`,
        /// `Track::repeat_auto_fill(Track::minmax(330, Track::fr(1)))`,
        /// `(Track::minmax(70, pct(38)), Track::fr(1), 20)`. For a `grid`
        /// box.
        grid_template_columns, columns, impl Into<Tracks>, |v: Tracks| v);
    grid_setter!(
        /// CSS `grid-template-rows`.
        grid_template_rows, rows, impl Into<Tracks>, |v: Tracks| v);
    grid_setter!(
        /// CSS `grid-auto-columns`: the size of the columns auto-placed
        /// items make beyond the template (cycling through the list).
        grid_auto_columns, auto_columns, impl Into<Tracks>, |v: Tracks| v);
    grid_setter!(
        /// CSS `grid-auto-rows`: the size of the rows beyond the template
        /// (`grid_auto_rows(120)` makes every new row 120 px high).
        grid_auto_rows, auto_rows, impl Into<Tracks>, |v: Tracks| v);
    grid_setter!(
        /// CSS `grid-auto-flow`.
        grid_auto_flow, flow, GridAutoFlow, |v| v);
    grid_setter!(
        /// CSS `grid-column` of a grid item: `GridPlacement::span(4)`,
        /// `(1, -1)` (the whole row), `2`, `(2, GridPlacement::span(3))`.
        grid_column, column, impl Into<GridPlacement>, |v: GridPlacement| v);
    grid_setter!(
        /// CSS `grid-row` of a grid item.
        grid_row, row, impl Into<GridPlacement>, |v: GridPlacement| v);
    grid_setter!(
        /// CSS `justify-items`: how a grid's items sit in their areas along
        /// the row (default: stretch).
        justify_items, justify_items, Align, |v| v);
    grid_setter!(
        /// CSS `justify-self`: how this grid item sits in its area along
        /// the row.
        justify_self, justify_self, Align, |v| v);

    /// CSS `place-items`: `align_items` and `justify_items` at once
    /// (`place_items(Align::Center)` centers a grid's items in their
    /// areas).
    pub fn place_items(self, a: Align) -> Self {
        self.align_items(a).justify_items(a)
    }
}
