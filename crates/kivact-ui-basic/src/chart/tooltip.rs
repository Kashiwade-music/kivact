//! Role: The hover tooltip card and the legend of a chart.

use super::model::{ChartConfig, Hit, series_color};
use crate::ColorName;

use crate::typography::MONO;
use kivact::geometry::{Point, Size};
use kivact::prelude::Element;
use kivact::style::{Align, Direction, Justify, Position, style};
use kivact::view::{IntoView, Role};
use kivact::view::{Node, column, node, row, text};
use std::rc::Rc;

/// The distance between the pointer and the card.
const OFFSET: f64 = 14.0;

/// The tooltip card for `hit` (chart.tsx `ChartTooltipContent`), placed beside the pointer `at` in a plot of `size`.
///
/// The card is built once per hit; it is placed at the pointer's side and
/// flipped to its left when the pointer is near the right edge.
pub(super) fn tooltip(
    hit: &Hit,
    at: Point,
    config: &Rc<ChartConfig>,
    size: Size,
    format: &dyn Fn(f64) -> String,
) -> Node {
    let rows = hit.rows.iter().map(|hit_row| {
        let series = hit_row.series;
        let config = config.clone();
        row((
            node().style(move || {
                style()
                    .size(10, 10)
                    .shrink(0)
                    .radius(2)
                    .background(series_color(
                        &config,
                        series,
                        &kivact::style::use_tokens::<crate::Tokens>(),
                    ))
            }),
            row((
                text(hit_row.label.clone())
                    .style(style().color(ColorName::MutedForeground).line_height(1.0)),
                text(format(hit_row.value)).style(
                    style()
                        .font_family(MONO)
                        .weight(500)
                        .tabular_nums()
                        .line_height(1.0)
                        .color(ColorName::Foreground),
                ),
            ))
            .style(
                style()
                    .grow(1)
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
            .map(|title| text(title).style(style().weight(500).line_height(1.0))),
        column(rows.collect::<Vec<_>>()).style(style().gap(6)),
    ))
    .with_node(|root| root.role(Role::Group))
    // The pointer passes through the card to the plot (`pointer-events-none`).
    .hit_testable(false)
    .style(move || {
        let p = kivact::style::use_tokens::<crate::Tokens>();
        let top = (at.y + OFFSET).min(size.height - height).max(0.0);
        // Laid out at the plot's top left corner (flipped: ending there),
        // and moved to the pointer.
        let placed = style().position(Position::Absolute).top(0);
        let placed = if at.x > size.width * 0.55 {
            placed.right(size.width).translate(at.x - OFFSET, top)
        } else {
            placed.left(0).translate(at.x + OFFSET, top)
        };
        let c = kivact::style::use_theme().get().colors().shadow;
        placed
            .z_index(10)
            .min_width(128)
            .gap(6)
            .padding_x(10)
            .padding_y(6)
            .background(p.background)
            .color(p.foreground)
            .border(1, p.border.with_alpha(0.5))
            .radius(0)
            .font_size(12)
            .shadow(kivact::draw::Shadow::new(0, 20, 25, c).spread(-5))
            .shadow(kivact::draw::Shadow::new(0, 8, 10, c).spread(-6))
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
                        .shrink(0)
                        .radius(2)
                        .background(series_color(
                            &colors,
                            index,
                            &kivact::style::use_tokens::<crate::Tokens>(),
                        ))
                }),
                text(series.label.clone()).style(style().color(ColorName::Foreground)),
            ))
            .style(style().gap(6).align_items(Align::Center))
            .into_view()
        })
        .collect::<Vec<_>>();
    row(items).style(
        style()
            .direction(Direction::Row)
            .justify_content(Justify::Center)
            .align_items(Align::Center)
            .gap(16)
            .padding_top(12)
            .font_size(12)
            .line_height(16.0 / 12.0),
    )
}
