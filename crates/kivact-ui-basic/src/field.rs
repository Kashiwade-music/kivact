//! Role: Field-set, grouping and separator layouts of field.tsx.

use crate::MUTED_FOREGROUND;
use crate::variants::FieldLegendVariant;
use kivact::component;
use kivact::custom::Caller;
use kivact::state::Value;
use kivact::style::{FlexDirection, Justify, Position, pct, style};
use kivact::view::{Element, Role, View, column, node, text};

/// A group of fields with an optional legend and description (`FieldSet`).
///
/// Created by [`field_set`]; a `Role::Group` named by the legend. Give it
/// its description with `.description(..)`.
#[component]
pub fn field_set(
    children: View,
    legend: Option<Value<String>>,
    #[prop(default = FieldLegendVariant::Legend)] legend_variant: FieldLegendVariant,
    #[caller] caller: Caller,
) -> impl Element {
    let has_legend = legend.is_some();
    let mut root = node().with_node(|root| root.role(Role::Group)).style(
        style()
            .flex_direction(FlexDirection::Column)
            .width(pct(100))
            .flex_shrink(0.0),
    );
    if let Some(legend) = legend {
        let size = match legend_variant {
            FieldLegendVariant::Legend => 14,
            FieldLegendVariant::Label => 12,
        };
        root = root.name(legend);
        root = root.child(text(legend).style(move || {
            style()
                .font_size(size)
                .line_height(if size == 14 { 20.0 / 14.0 } else { 16.0 / 12.0 })
                .font_weight(500)
                .margin_bottom(10)
        }));
    }
    if let Some(desc) = caller.description() {
        root = root.child(text(desc).style(move || {
            style()
                .font_size(12)
                .line_height(1.625)
                .margin_top(if has_legend { -6 } else { 0 })
                .margin_bottom(16)
                .color(MUTED_FOREGROUND)
        }));
    }
    root.child(column(children).style(style().gap(16)))
}

/// A column of fields spaced 20 px (`FieldGroup`).
#[component]
pub fn field_group(children: View) -> impl Element {
    column(children).style(style().width(pct(100)).gap(20))
}

/// A column that stacks a label and its description tightly (`FieldContent`).
#[component]
pub fn field_content(children: View) -> impl Element {
    column(children).style(
        style()
            .flex_grow(1.0)
            .min_width(0)
            .gap(2)
            .line_height(1.375),
    )
}

/// A separator line (`FieldSeparator`), optionally with centered text: add
/// `.text(..)` for a label on the line.
#[component]
pub fn field_separator(text: Option<Value<String>>) -> impl Element {
    let line = node().with_node(|root| root.role(Role::Separator)).style(
        style()
            .position(Position::Absolute)
            .left(0)
            .right(0)
            .top(10)
            .height(1)
            .background(crate::BORDER),
    );
    let mut root = node()
        .style(
            style()
                .flex_direction(FlexDirection::Row)
                .justify_content(Justify::Center)
                .position(Position::Relative)
                .width(pct(100))
                .height(20)
                .margin_y(-8)
                .flex_shrink(0.0),
        )
        .child(line);
    if let Some(label) = text {
        root = root.child(
            kivact::view::text(label).style(
                style()
                    .font_size(12)
                    .padding_x(8)
                    .background(crate::BACKGROUND)
                    .color(MUTED_FOREGROUND)
                    .line_height(20.0 / 12.0),
            ),
        );
    }
    root
}
