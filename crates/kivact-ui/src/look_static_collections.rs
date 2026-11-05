//! Role: Default theme recipes for static tables and item lists.
use crate::UiTokens;
use kivact_core::custom::PartState;
use kivact_core::style::{Align, ColorToken, Direction, Style, Theme, pct, style};
pub(super) fn table(t: &Theme, _: &PartState) -> Style {
    style()
        .direction(Direction::Column)
        .width(pct(100))
        .min_width(0)
        .font_size(
            t.ext::<UiTokens>()
                .table
                .font_size
                .unwrap_or(t.text().size_small),
        )
}
pub(super) fn group(_: &Theme, _: &PartState) -> Style {
    style().direction(Direction::Column)
}
pub(super) fn footer(_: &Theme, _: &PartState) -> Style {
    style()
        .direction(Direction::Column)
        .background(ColorToken::SurfaceAlt)
}
pub(super) fn row(_: &Theme, _: &PartState) -> Style {
    style()
        .direction(Direction::Row)
        .width(pct(100))
        .align_items(Align::Center)
        .border_bottom(1, ColorToken::Border)
        .selected(style().background(ColorToken::Selection))
}
pub(super) fn cell(t: &Theme, _: &PartState) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .min_width(0)
        .padding(
            t.ext::<UiTokens>()
                .table
                .cell_padding
                .unwrap_or(t.space().sm),
        )
        .color(ColorToken::Text)
}
pub(super) fn head(t: &Theme, s: &PartState) -> Style {
    cell(t, s)
        .min_height(
            t.ext::<UiTokens>()
                .table
                .header_height
                .unwrap_or(t.control_height()),
        )
        .weight(600)
}
pub(super) fn list(t: &Theme, _: &PartState) -> Style {
    style()
        .direction(Direction::Column)
        .width(pct(100))
        .gap(t.ext::<UiTokens>().item_list.gap.unwrap_or(t.space().xs))
}
pub(super) fn item(t: &Theme, _: &crate::collection_values::ItemRowState) -> Style {
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
pub(super) fn content(_: &Theme, _: &crate::collection_values::ItemRowState) -> Style {
    style().direction(Direction::Column).grow(1).min_width(0)
}
pub(super) fn title(_: &Theme, _: &crate::collection_values::ItemRowState) -> Style {
    style().weight(500)
}
pub(super) fn description(t: &Theme, _: &crate::collection_values::ItemRowState) -> Style {
    style()
        .font_size(
            t.ext::<UiTokens>()
                .item_list
                .description_size
                .unwrap_or(t.text().size_small),
        )
        .color(ColorToken::TextMuted)
}
pub(super) fn media(_: &Theme, _: &crate::collection_values::ItemRowState) -> Style {
    style().shrink(0).align_self(Align::Center)
}
pub(super) fn actions(_: &Theme, _: &crate::collection_values::ItemRowState) -> Style {
    style()
        .direction(Direction::Row)
        .shrink(0)
        .align_items(Align::Center)
}
pub(super) fn band(_: &Theme, _: &crate::collection_values::ItemRowState) -> Style {
    style().direction(Direction::Row).width(pct(100))
}

pub(super) fn item_body(t: &Theme, _: &crate::collection_values::ItemRowState) -> Style {
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

pub(super) fn list_header(t: &Theme, _: &PartState) -> Style {
    style().font_size(t.text().size_small).weight(600)
}
pub(super) fn marker(_: &Theme, _: &crate::collection_values::ItemRowState) -> Style {
    style().shrink(0).min_width(16).direction(Direction::Row)
}
