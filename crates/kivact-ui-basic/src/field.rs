//! Role: Field-set, grouping and separator layouts of field.tsx.

use kivact::state::Value;
use kivact::view::{Element, Node, column, node, text};

use kivact::state::IntoText;
use kivact::style::{Direction, Justify, Position, pct, style};
use kivact::view::{IntoView, Role, View};

/// The look of a [`FieldSet`] legend.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FieldLegendVariant {
    /// `text-sm`.
    #[default]
    Legend,
    /// `text-xs`, like a label.
    Label,
}

/// A group of fields with an optional legend and description (`FieldSet`).
///
/// Created by [`field_set`]; a `Role::Group` named by the legend.
pub struct FieldSet {
    root: Node,
    children: View,
    legend: Option<Value<String>>,
    variant: FieldLegendVariant,
    description: Option<Value<String>>,
}

/// A field set around `children`.
#[track_caller]
pub fn field_set(children: impl IntoView) -> FieldSet {
    FieldSet {
        root: node().with_node(|root| root.role(Role::Group)).style(
            style()
                .direction(Direction::Column)
                .width(pct(100))
                .shrink(0),
        ),
        children: children.into_view(),
        legend: None,
        variant: FieldLegendVariant::Legend,
        description: None,
    }
}

impl FieldSet {
    /// The legend text.
    pub fn legend(mut self, text: impl IntoText) -> Self {
        self.legend = Some(text.into_text_value());
        self
    }

    /// The legend look (default [`FieldLegendVariant::Legend`]).
    pub fn legend_variant(mut self, variant: FieldLegendVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Helper text under the legend.
    pub fn description(mut self, text: impl IntoText) -> Self {
        self.description = Some(text.into_text_value());
        self
    }
}

impl Element for FieldSet {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for FieldSet {
    fn into_view(self) -> View {
        let FieldSet {
            root,
            children,
            legend,
            variant,
            description,
        } = self;
        let mut root = root;
        if let Some(legend) = legend {
            let size = match variant {
                FieldLegendVariant::Legend => 14,
                FieldLegendVariant::Label => 12,
            };
            if root.accessible_name().is_none() {
                root = root.name(legend);
            }
            root = root.child(text(legend).style(move || {
                style()
                    .font_size(size)
                    .line_height(if size == 14 { 20.0 / 14.0 } else { 16.0 / 12.0 })
                    .weight(500)
                    .margin_bottom(10)
            }));
        }
        if let Some(desc) = description {
            root = root.child(text(desc).style(move || {
                style()
                    .font_size(12)
                    .line_height(1.625)
                    .margin_top(if legend.is_some() { -6 } else { 0 })
                    .margin_bottom(16)
                    .color(kivact::style::use_tokens::<crate::Tokens>().muted_foreground)
            }));
        }
        root.child(column(children).style(style().gap(16)))
            .into_view()
    }
}

/// A column of fields spaced 20 px (`FieldGroup`).
#[track_caller]
pub fn field_group(children: impl IntoView) -> impl kivact::view::Element {
    column(children).style(style().width(pct(100)).gap(20))
}

/// A column that stacks a label and its description tightly (`FieldContent`).
#[track_caller]
pub fn field_content(children: impl IntoView) -> impl kivact::view::Element {
    column(children).style(style().grow(1).min_width(0).gap(2).line_height(1.375))
}

/// A line between fields, optionally with centered text (`FieldSeparator`).
pub struct FieldSeparator {
    root: Node,
    text: Option<Value<String>>,
}

/// A separator line; add `.text(..)` for a label on the line.
#[track_caller]
pub fn field_separator() -> FieldSeparator {
    FieldSeparator {
        root: node().style(
            style()
                .direction(Direction::Row)
                .justify_content(Justify::Center)
                .position(Position::Relative)
                .width(pct(100))
                .height(20)
                .margin_y(-8)
                .shrink(0),
        ),
        text: None,
    }
}

impl FieldSeparator {
    /// Text centered on the line.
    pub fn text(mut self, text: impl IntoText) -> Self {
        self.text = Some(text.into_text_value());
        self
    }
}

impl Element for FieldSeparator {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for FieldSeparator {
    fn into_view(self) -> View {
        let line = node().with_node(|root| root.role(Role::Separator)).style(
            style()
                .position(Position::Absolute)
                .left(0)
                .right(0)
                .top(10)
                .height(1)
                .background(crate::ColorName::Border),
        );
        let mut root = self.root.child(line);
        if let Some(label) = self.text {
            root = root.child(
                text(label).style(
                    style()
                        .font_size(12)
                        .padding_x(8)
                        .background(crate::ColorName::Background)
                        .color(crate::ColorName::MutedForeground)
                        .line_height(20.0 / 12.0),
                ),
            );
        }
        root.into_view()
    }
}

impl kivact::ui::Variant for FieldLegendVariant {}
