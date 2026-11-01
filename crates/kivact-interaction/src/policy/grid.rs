//! Role: Two-dimensional grid moves from keys, independent of a UI runtime.

use kivact_core::view::{Key, Modifiers};

/// One keyboard move between the cells of a grid of rows and columns.
///
/// [`from_key`](Self::from_key) maps the conventional grid keys; callers
/// decide what a move means for their data. A [`Table`](crate::Table) applies
/// it with [`apply`](Self::apply) to bounded rows and columns, and a
/// [calendar grid](crate::use_calendar_grid) maps it onto dates, where moving past the edge of a month continues into
/// the next one.
///
/// ```
/// # mod kivact {
/// #     //! Role: Supply facade paths for this documentation example.
/// #     pub use kivact_core::view;
/// #     pub extern crate kivact_interaction as interaction;
/// # }
/// use kivact::interaction::GridMove;
/// use kivact::view::{Key, Modifiers};
/// let ctrl = Modifiers { ctrl: true, ..Modifiers::default() };
/// assert_eq!(GridMove::from_key(Key::Home, ctrl), Some(GridMove::First));
/// assert_eq!(GridMove::from_key(Key::PageDown, ctrl), None);
/// // Three rows of four columns; moves stop at the edges.
/// assert_eq!(GridMove::Right.apply((1, 3), 3, 4), Some((1, 3)));
/// assert_eq!(GridMove::Down.apply((1, 2), 3, 4), Some((2, 2)));
/// assert_eq!(GridMove::RowStart.apply((2, 3), 3, 4), Some((2, 0)));
/// assert_eq!(GridMove::Last.apply((0, 0), 3, 4), Some((2, 3)));
/// assert_eq!(GridMove::Down.apply((0, 0), 0, 4), None);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridMove {
    /// ArrowLeft: the previous cell in the row.
    Left,
    /// ArrowRight: the next cell in the row.
    Right,
    /// ArrowUp: the same column in the previous row.
    Up,
    /// ArrowDown: the same column in the next row.
    Down,
    /// Home: the first cell in the row.
    RowStart,
    /// End: the last cell in the row.
    RowEnd,
    /// Ctrl+Home or Cmd+Home: the first cell of the first row.
    First,
    /// Ctrl+End or Cmd+End: the last cell of the last row.
    Last,
}

impl GridMove {
    /// The move a key requests, if any.
    ///
    /// Shift and Alt do not change the move, so a caller can give them its own
    /// meaning (a table extends its row selection with Shift+ArrowUp/Down).
    /// Ctrl or Meta turns Home and End into [`First`](Self::First) and
    /// [`Last`](Self::Last).
    pub fn from_key(key: Key, modifiers: Modifiers) -> Option<Self> {
        let command = modifiers.ctrl || modifiers.meta;
        Some(match key {
            Key::ArrowLeft => Self::Left,
            Key::ArrowRight => Self::Right,
            Key::ArrowUp => Self::Up,
            Key::ArrowDown => Self::Down,
            Key::Home if command => Self::First,
            Key::End if command => Self::Last,
            Key::Home => Self::RowStart,
            Key::End => Self::RowEnd,
            _ => return None,
        })
    }

    /// The cell `(row, column)` this move reaches from `at` in a grid of
    /// `rows` by `columns`, stopping at the edges.
    ///
    /// A position outside the grid is clamped into it first. An empty grid has
    /// no cell, so the result is `None`.
    pub fn apply(self, at: (usize, usize), rows: usize, columns: usize) -> Option<(usize, usize)> {
        let (last_row, last_column) = (rows.checked_sub(1)?, columns.checked_sub(1)?);
        let (row, column) = (at.0.min(last_row), at.1.min(last_column));
        Some(match self {
            Self::Left => (row, column.saturating_sub(1)),
            Self::Right => (row, (column + 1).min(last_column)),
            Self::Up => (row.saturating_sub(1), column),
            Self::Down => ((row + 1).min(last_row), column),
            Self::RowStart => (row, 0),
            Self::RowEnd => (row, last_column),
            Self::First => (0, 0),
            Self::Last => (last_row, last_column),
        })
    }
}
