//! Role: The splitter recipe state, tokens, recipe table and default looks.

use crate::recipe::recipe_table;
use crate::slider::look::AxisState;
use kivact_core::style::{ColorToken, Cursor, Direction, Justify, Style, Theme, style};
use kivact_core::view::Orientation;
/// The handle: a hairline in a hit area wide enough to grab.
pub(crate) fn splitter_handle(_: &Theme, state: &AxisState) -> Style {
    let s = style()
        .shrink(0)
        .direction(Direction::Row)
        .justify_content(Justify::Center)
        .paint(|d, size| {
            if size.width <= size.height {
                d.fill_rect(
                    kivact_core::geometry::Rect::new(
                        (size.width - 1.0) / 2.0,
                        0.0,
                        1.0,
                        size.height,
                    ),
                    ColorToken::Border,
                );
            } else {
                d.fill_rect(
                    kivact_core::geometry::Rect::new(
                        0.0,
                        (size.height - 1.0) / 2.0,
                        size.width,
                        1.0,
                    ),
                    ColorToken::Border,
                );
            }
        })
        .hover(style().background(ColorToken::Hover))
        .pressed(style().background(ColorToken::Pressed));
    match state.orientation {
        Orientation::Vertical => s.height(6).cursor(Cursor::NsResize),
        _ => s.width(6).cursor(Cursor::EwResize),
    }
}

recipe_table! {
    /// The recipes of a [`splitter`](super::splitter).
    SplitterRecipes {
        /// The handle between the panes.
        handle(handle_recipe): AxisState = splitter_handle,
    }
}
