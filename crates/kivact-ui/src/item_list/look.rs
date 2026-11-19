//! Role: The item list recipe state, tokens, recipe table and default looks.
use crate::recipe::recipe_table;
use crate::tokens::UiTokens;
use kivact_core::custom::PartState;
use kivact_core::style::{Align, ColorToken, Direction, Style, Theme, pct, style};
pub(crate) fn item_list_root(t: &Theme, _: &PartState) -> Style {
    style()
        .direction(Direction::Column)
        .width(pct(100))
        .gap(t.ext::<UiTokens>().item_list.gap.unwrap_or(t.space().xs))
}

pub(crate) fn item(t: &Theme, _: &ItemRowState) -> Style {
    style()
        .direction(Direction::Column)
        .padding(
            t.ext::<UiTokens>()
                .item_list
                .padding
                .unwrap_or(t.space().sm),
        )
        .gap(
            t.ext::<UiTokens>()
                .item_list
                .row_gap
                .unwrap_or(t.space().sm),
        )
        .selected(style().background(ColorToken::Selection))
        .focus_visible(style().ring(t.focus_ring(), ColorToken::Focus))
}

pub(crate) fn content(_: &Theme, _: &ItemRowState) -> Style {
    style().direction(Direction::Column).grow(1).min_width(0)
}

pub(crate) fn title(_: &Theme, _: &ItemRowState) -> Style {
    style().weight(500)
}

pub(crate) fn description(t: &Theme, _: &ItemRowState) -> Style {
    style()
        .font_size(
            t.ext::<UiTokens>()
                .item_list
                .description_size
                .unwrap_or(t.text().size_small),
        )
        .color(ColorToken::TextMuted)
}

pub(crate) fn media(_: &Theme, _: &ItemRowState) -> Style {
    style().shrink(0).align_self(Align::Center)
}

pub(crate) fn actions(_: &Theme, _: &ItemRowState) -> Style {
    style()
        .direction(Direction::Row)
        .shrink(0)
        .align_items(Align::Center)
}

pub(crate) fn band(_: &Theme, _: &ItemRowState) -> Style {
    style().direction(Direction::Row).width(pct(100))
}

pub(crate) fn item_body(t: &Theme, _: &ItemRowState) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(
            t.ext::<UiTokens>()
                .item_list
                .row_gap
                .unwrap_or(t.space().sm),
        )
}

pub(crate) fn list_header(t: &Theme, _: &PartState) -> Style {
    style().font_size(t.text().size_small).weight(600)
}

pub(crate) fn marker(_: &Theme, _: &ItemRowState) -> Style {
    style().shrink(0).min_width(16).direction(Direction::Row)
}

/// Visual overrides for static lists and item rows.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ItemListTokens {
    pub gap: Option<f64>,
    pub padding: Option<f64>,
    pub row_gap: Option<f64>,
    pub description_size: Option<f64>,
    pub title_lines: Option<u32>,
    pub description_lines: Option<u32>,
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
    ItemListRecipes {
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
}

/// Actual zero-based position and collection size; this record owns no ordering state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemPosition {
    pub index: usize,
    pub count: usize,
}

/// Structural marker used by a semantic static list.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ItemListMarker {
    #[default]
    None,
    Disc,
    Circle,
    Decimal,
}

kivact_core::state::constant_value!(ItemListMarker);
