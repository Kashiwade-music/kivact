//! Role: The tree recipe state, recipe table and default looks.

use crate::recipe::recipe_table;
use crate::theme::Look;
use kivact_core::custom::PartState;
use kivact_core::style::{Align, FlexDirection, Style, Theme, style};
use kivact_core::style::{Length, ThemeVar, control, radius, space};
use kivact_core::view::{ReadingDirection, use_direction};

/// The width of a tree's expand marker, in logical px.
pub static MARKER_WIDTH_VAR: ThemeVar<f64> = ThemeVar::new("tree.marker_width", 16.0);
/// [`MARKER_WIDTH_VAR`] as a length for a style.
pub const MARKER_WIDTH: Length = Length::var(&MARKER_WIDTH_VAR);

/// The indent of each level of a tree, in logical px.
pub static INDENT_VAR: ThemeVar<f64> = ThemeVar::new("tree.indent", 16.0);
/// [`INDENT_VAR`] as a length for a style.
pub const INDENT: Length = Length::var(&INDENT_VAR);

/// The padding of a tree row, in logical px.
pub static PADDING_VAR: ThemeVar<f64> = ThemeVar::alias("tree.padding", &space::MD_VAR);
/// [`PADDING_VAR`] as a length for a style.
pub const PADDING: Length = Length::var(&PADDING_VAR);

/// The gap inside a tree row, in logical px.
pub static GAP_VAR: ThemeVar<f64> = ThemeVar::alias("tree.gap", &space::XS_VAR);
/// [`GAP_VAR`] as a length for a style.
pub const GAP: Length = Length::var(&GAP_VAR);

/// The minimum height of a tree row, in logical px.
pub static ROW_HEIGHT_VAR: ThemeVar<f64> = ThemeVar::alias("tree.row_height", &control::HEIGHT_VAR);
/// [`ROW_HEIGHT_VAR`] as a length for a style.
pub const ROW_HEIGHT: Length = Length::var(&ROW_HEIGHT_VAR);

/// The corner radius of a tree row, in logical px.
pub static RADIUS_VAR: ThemeVar<f64> = ThemeVar::alias("tree.radius", &radius::SM_VAR);
/// [`RADIUS_VAR`] as a length for a style.
pub const RADIUS: Length = Length::var(&RADIUS_VAR);

pub(crate) fn tree_item(t: &Theme, row: &Look<TreeRowRecipeState>) -> Style {
    let rtl = use_direction() == ReadingDirection::Rtl;
    let padding = t.get(&PADDING_VAR);
    let indent = padding + row.level.saturating_sub(1) as f64 * t.get(&INDENT_VAR);
    style()
        .enter_size(kivact_core::view::Orientation::Vertical)
        .exit_size(kivact_core::view::Orientation::Vertical)
        .flex_direction(if rtl {
            FlexDirection::RowReverse
        } else {
            FlexDirection::Row
        })
        .align_items(Align::Center)
        .min_height(ROW_HEIGHT)
        .padding_left(if rtl { padding } else { indent })
        .padding_right(if rtl { indent } else { padding })
        .gap(GAP)
        .border_radius(RADIUS)
        .hover(style().background(kivact_core::style::color::HOVER))
        .selected(style().background(kivact_core::style::color::SELECTION))
}

pub(crate) fn tree_marker(_: &Theme, _: &Look<TreeRowRecipeState>) -> Style {
    style().width(MARKER_WIDTH).flex_shrink(0)
}

/// The mounted row's interaction state and structural position in its visible tree.
#[derive(Clone, PartialEq)]
pub struct TreeRowRecipeState {
    pub item: kivact_interaction::TreeItemState,
    /// Whether each ancestor is the last visible sibling at its level.
    pub ancestor_last: Vec<bool>,
    pub is_last: bool,
    pub has_branches: bool,
    /// The caller supplied marker content instead of the standard glyph.
    pub custom_marker: bool,
}

impl std::ops::Deref for TreeRowRecipeState {
    type Target = kivact_interaction::TreeItemState;
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

recipe_table! {
    /// The recipes of a [`tree`](super::tree).
    TreeTheme {
        /// The column containing the visible rows.
        root(root_recipe): PartState = crate::look_common::list,
        /// A row with the expansion indicator and its content.
        item(item_recipe): TreeRowRecipeState = tree_item,
        /// The branch indicator or matching leaf spacer.
        marker(marker_recipe): TreeRowRecipeState = tree_marker,
    }
}
