//! Role: The item list recipe state, recipe table and default looks.
use crate::recipe::recipe_table;
use crate::theme::Look;
use kivact_core::custom::PartState;
use kivact_core::style::{Align, FlexDirection, Style, Theme, pct, style};
use kivact_core::style::{Length, ThemeVar, font, space};
use kivact_macros::Prop;

/// The gap between the items of a list, in logical px.
pub static GAP_VAR: ThemeVar<f64> = ThemeVar::alias("item_list.gap", &space::XS_VAR);
/// [`GAP_VAR`] as a length for a style.
pub const GAP: Length = Length::var(&GAP_VAR);

/// The padding of an item row, in logical px.
pub static PADDING_VAR: ThemeVar<f64> = ThemeVar::alias("item_list.padding", &space::SM_VAR);
/// [`PADDING_VAR`] as a length for a style.
pub const PADDING: Length = Length::var(&PADDING_VAR);

/// The gap between the parts of an item row, in logical px.
pub static ROW_GAP_VAR: ThemeVar<f64> = ThemeVar::alias("item_list.row_gap", &space::SM_VAR);
/// [`ROW_GAP_VAR`] as a length for a style.
pub const ROW_GAP: Length = Length::var(&ROW_GAP_VAR);

/// The font size of an item's description, in logical px.
pub static DESCRIPTION_SIZE_VAR: ThemeVar<f64> =
    ThemeVar::alias("item_list.description_size", &font::SIZE_SMALL_VAR);
/// [`DESCRIPTION_SIZE_VAR`] as a length for a style.
pub const DESCRIPTION_SIZE: Length = Length::var(&DESCRIPTION_SIZE_VAR);

pub(crate) fn item_list_root(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .width(pct(100))
        .gap(GAP)
}

pub(crate) fn item(t: &Theme, _: &Look<ItemRowState>) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .padding(PADDING)
        .gap(ROW_GAP)
        .selected(style().background(kivact_core::style::color::SELECTION))
        .focus_visible(style().outline(t.focus_ring(), kivact_core::style::color::FOCUS))
}

pub(crate) fn content(_: &Theme, _: &Look<ItemRowState>) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .flex_grow(1)
        .min_width(0)
}

pub(crate) fn title(_: &Theme, _: &Look<ItemRowState>) -> Style {
    style().font_weight(500)
}

pub(crate) fn description(_: &Theme, _: &Look<ItemRowState>) -> Style {
    style()
        .font_size(DESCRIPTION_SIZE)
        .color(kivact_core::style::color::TEXT_MUTED)
}

pub(crate) fn media(_: &Theme, _: &Look<ItemRowState>) -> Style {
    style().flex_shrink(0).align_self(Align::Center)
}

pub(crate) fn actions(_: &Theme, _: &Look<ItemRowState>) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .flex_shrink(0)
        .align_items(Align::Center)
}

pub(crate) fn band(_: &Theme, _: &Look<ItemRowState>) -> Style {
    style().flex_direction(FlexDirection::Row).width(pct(100))
}

pub(crate) fn item_body(_: &Theme, _: &Look<ItemRowState>) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(ROW_GAP)
}

pub(crate) fn list_header(t: &Theme, _: &Look<PartState>) -> Style {
    style()
        .font_size(t.get(&kivact_core::style::font::SIZE_SMALL_VAR))
        .font_weight(600)
}

pub(crate) fn marker(_: &Theme, _: &Look<ItemRowState>) -> Style {
    style()
        .flex_shrink(0)
        .min_width(16)
        .flex_direction(FlexDirection::Row)
}

/// Actual item-row input and independent construction facts exposed to recipes.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ItemRowState {
    pub input: kivact_core::custom::PartState,
    pub has_description: bool,
    pub actionable: bool,
    pub position: Option<ItemPosition>,
    pub marker: ItemListMarker,
    pub start: u32,
    pub title_lines: Option<u32>,
    pub description_lines: Option<u32>,
}

impl std::ops::Deref for ItemRowState {
    type Target = kivact_core::custom::PartState;
    fn deref(&self) -> &Self::Target {
        &self.input
    }
}

recipe_table! {
    /// Static list and item-row part recipes.
    ItemListTheme {
        list_header(list_header_recipe): PartState = list_header,
        marker(marker_recipe): ItemRowState = marker,
        root(root_recipe): PartState = item_list_root,
        item(item_recipe): ItemRowState = item,
        body(body_recipe): ItemRowState = item_body,
        content(content_recipe): ItemRowState = content,
        title(title_recipe): ItemRowState = title,
        description(description_recipe): ItemRowState = description,
        media(media_recipe): ItemRowState = media,
        actions(actions_recipe): ItemRowState = actions,
        header(header_recipe): ItemRowState = band,
        footer(footer_recipe): ItemRowState = band,
    }
    settings {
        /// The lines a row's title may wrap to; `None` lets it wrap freely.
        title_lines(title_lines_value): Option<u32> = None,
        /// The lines a row's description may wrap to; `None` lets it wrap freely.
        description_lines(description_lines_value): Option<u32> = None,
    }
}

/// Actual zero-based position and collection size; this record owns no ordering state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemPosition {
    pub index: usize,
    pub count: usize,
}

/// Structural marker used by a semantic static list.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum ItemListMarker {
    #[default]
    None,
    Disc,
    Circle,
    Decimal,
}
