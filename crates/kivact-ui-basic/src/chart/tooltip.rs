//! Role: The hover tooltip card and the legend of a chart.

use super::model::{ChartConfig, Hit, series_color};
use crate::color::{BACKGROUND, BORDER_VAR, FOREGROUND, MUTED_FOREGROUND};
use crate::recipe_parts::MONO;
use kivact::geometry::{Point, Rect, Size};
use kivact::prelude::Element;
use kivact::style::{Align, FlexDirection, Justify, Position, Shadow, style};
use kivact::view::{IntoView, Role};
use kivact::view::{Node, column, node, row, text};
use std::rc::Rc;

/// The distance between the pointer and the card.
const OFFSET: f64 = 14.0;

/// The tooltip card for `hit` (chart.tsx `ChartTooltipContent`), placed beside the pointer `at` in the plot canvas whose bounds are `bounds`.
///
/// The card is built once per hit; a pointer move only translates it. It is
/// placed at the pointer's side and flipped to its left when the pointer is
/// near the right edge. The bounds are read where the card positions itself.
pub(super) fn tooltip(
    hit: &Hit,
    at: kivact::state::Value<Point>,
    config: &Rc<ChartConfig>,
    bounds: kivact::state::Value<Option<Rect>>,
    format: &dyn Fn(f64) -> String,
) -> Node {
    let rows = hit.rows.iter().map(|hit_row| {
        let series = hit_row.series;
        let config = config.clone();
        row((
            node().style(move || {
                style()
                    .size(10, 10)
                    .flex_shrink(0.0)
                    .border_radius(2.0)
                    .background(series_color(&config, series))
            }),
            row((
                text(hit_row.label.clone()).style(style().color(MUTED_FOREGROUND).line_height(1.0)),
                text(format(hit_row.value)).style(
                    style()
                        .font_family(MONO)
                        .font_weight(500)
                        .tabular_nums()
                        .line_height(1.0)
                        .color(FOREGROUND),
                ),
            ))
            .style(
                style()
                    .flex_grow(1.0)
                    .gap(16)
                    .align_items(Align::Center)
                    .justify_content(Justify::SpaceBetween),
            ),
        ))
        .style(style().gap(8).align_items(Align::Center))
        .into_view()
    });
    let height = 14.0
        + hit.title.as_ref().map_or(0.0, |_| 22.0)
        + hit.rows.len() as f64 * 16.0
        + hit.rows.len().saturating_sub(1) as f64 * 6.0;
    column((
        hit.title
            .clone()
            .map(|title| text(title).style(style().font_weight(500).line_height(1.0))),
        column(rows.collect::<Vec<_>>()).style(style().gap(6)),
    ))
    .with_node(|root| root.role(Role::Group))
    // The pointer passes through the card to the plot (`pointer-events-none`).
    .hit_testable(false)
    .style(move || {
        let at = at.get();
        let size = bounds.get().map_or(Size::new(0.0, 0.0), |rect| {
            Size::new(rect.width, rect.height)
        });
        let top = (at.y + OFFSET).min(size.height - height).max(0.0);
        // Laid out at the plot's top left corner (flipped: ending there),
        // and moved to the pointer.
        let placed = style().position(Position::Absolute).top(0);
        let placed = if at.x > size.width * 0.55 {
            placed.right(size.width).translate(at.x - OFFSET, top)
        } else {
            placed.left(0).translate(at.x + OFFSET, top)
        };
        let theme = kivact::style::use_theme().get();
        let c = theme.get(&kivact::style::color::SHADOW_VAR);
        placed
            .z_index(10)
            .min_width(128)
            .gap(6)
            .padding_x(10)
            .padding_y(6)
            .background(BACKGROUND)
            .color(FOREGROUND)
            .border(1, theme.get(&BORDER_VAR).with_alpha(0.5))
            .border_radius(0.0)
            .font_size(12)
            .shadow(Shadow::new(0.0, 20.0, 25.0, c).spread(-5.0))
            .shadow(Shadow::new(0.0, 8.0, 10.0, c).spread(-6.0))
    })
}

/// The legend under the plot (chart.tsx `ChartLegendContent`): a swatch and the label of each series.
pub(super) fn legend(config: &Rc<ChartConfig>) -> Node {
    let items = config
        .series
        .iter()
        .enumerate()
        .map(|(index, series)| {
            let colors = config.clone();
            row((
                node().style(move || {
                    style()
                        .size(8, 8)
                        .flex_shrink(0.0)
                        .border_radius(2.0)
                        .background(series_color(&colors, index))
                }),
                text(series.label.clone()).style(style().color(FOREGROUND)),
            ))
            .style(style().gap(6).align_items(Align::Center))
            .into_view()
        })
        .collect::<Vec<_>>();
    row(items).style(
        style()
            .flex_direction(FlexDirection::Row)
            .justify_content(Justify::Center)
            .align_items(Align::Center)
            .gap(16)
            .padding_top(12)
            .font_size(12)
            .line_height(16.0 / 12.0),
    )
}
