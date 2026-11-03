//! Role: Key-value metadata arranged in a neutral grid.

use crate::tokens;
use kivact::control::{Node, NodeBuilder, column, grid, node, row, text};
use kivact::interaction::use_disclosure;
use kivact::primitive::{NodeExt, part};
use kivact::style::{Align, Direction, Track, style};
use kivact::view::{IntoText, IntoValue, IntoView, NodeRef, Orientation, Role, View, node_ref};
use kivact::{Binding, Value, memo, signal};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MetadataColumns {
    #[default]
    Single,
    Multi,
    Count(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LabelPosition {
    Start,
    Top,
}

pub struct MetadataItem {
    root: Node,
    label: Value<String>,
    value: View,
    icon: Option<View>,
    label_ref: NodeRef,
}

pub fn metadata_item(label: impl IntoText, value: impl IntoView) -> MetadataItem {
    MetadataItem {
        root: node(),
        label: label.into_text_value(),
        value: value.into_view(),
        icon: None,
        label_ref: node_ref(),
    }
}

impl NodeBuilder for MetadataItem {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl MetadataItem {
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }

    fn render(self, position: LabelPosition, width: Value<Option<f32>>) -> View {
        let (root, label, value) = self.parts();
        root.base_style(
            style()
                .direction(if position == LabelPosition::Start {
                    Direction::Row
                } else {
                    Direction::Column
                })
                .gap(if position == LabelPosition::Start {
                    16
                } else {
                    2
                })
                .min_width(0),
        )
        .child((
            row(label).style(move || {
                let mut box_style = style().align_items(Align::Start);
                if position == LabelPosition::Start {
                    box_style = box_style.shrink(0);
                    if let Some(width) = width.get() {
                        box_style = box_style.width(width);
                    }
                }
                box_style
            }),
            value,
        ))
        .into_view()
    }

    fn parts(self) -> (Node, Node, Node) {
        let label_ref = self.label_ref;
        let label = row((
            self.icon,
            text(self.label).style(move || {
                style()
                    .font_size(14)
                    .line_height(20.0 / 14.0)
                    .weight(500)
                    .color(tokens().colors.text_secondary)
            }),
        ))
        .node_ref(label_ref)
        .style(style().gap(4).align_items(Align::Center).shrink(0));
        let value = node()
            .part(part(|state| state).role(Role::Group).labelled_by(label_ref))
            .style(move || {
                style()
                    .min_width(0)
                    .grow(1)
                    .color(tokens().colors.text_primary)
            })
            .child(self.value);
        (self.root, label, value)
    }
}

impl IntoView for MetadataItem {
    fn into_view(self) -> View {
        self.render(LabelPosition::Top, None.into_value())
    }
}

pub trait IntoMetadataItems {
    fn into_metadata_items(self) -> Vec<MetadataItem>;
}

impl IntoMetadataItems for MetadataItem {
    fn into_metadata_items(self) -> Vec<MetadataItem> {
        vec![self]
    }
}

impl IntoMetadataItems for Vec<MetadataItem> {
    fn into_metadata_items(self) -> Vec<MetadataItem> {
        self
    }
}

impl<const N: usize> IntoMetadataItems for [MetadataItem; N] {
    fn into_metadata_items(self) -> Vec<MetadataItem> {
        self.into()
    }
}

macro_rules! metadata_tuples {
    ($($item:ident),+) => {
        impl<$($item: IntoMetadataItems),+> IntoMetadataItems for ($($item,)+) {
            #[allow(non_snake_case, reason = "Tuple type identifiers also bind the tuple values.")]
            fn into_metadata_items(self) -> Vec<MetadataItem> {
                let ($($item,)+) = self;
                let mut items = Vec::new();
                $(items.extend($item.into_metadata_items());)+
                items
            }
        }
    };
}

metadata_tuples!(A);
metadata_tuples!(A, B);
metadata_tuples!(A, B, C);
metadata_tuples!(A, B, C, D);
metadata_tuples!(A, B, C, D, E);
metadata_tuples!(A, B, C, D, E, F);

pub struct MetadataList {
    root: Node,
    items: Vec<MetadataItem>,
    columns: MetadataColumns,
    position: Option<LabelPosition>,
    label_width: Option<f32>,
    max_items: Option<usize>,
    expanded: Option<Binding<bool>>,
    orientation: Orientation,
    title: Option<View>,
}

pub fn metadata_list(items: impl IntoMetadataItems) -> MetadataList {
    MetadataList {
        root: node(),
        items: items.into_metadata_items(),
        columns: MetadataColumns::Single,
        position: None,
        label_width: None,
        max_items: None,
        expanded: None,
        orientation: Orientation::Vertical,
        title: None,
    }
}

impl NodeBuilder for MetadataList {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl MetadataList {
    pub fn columns(mut self, columns: MetadataColumns) -> Self {
        self.columns = columns;
        self
    }

    pub fn label_position(mut self, position: LabelPosition) -> Self {
        self.position = Some(position);
        self
    }

    pub fn label_width(mut self, width: f32) -> Self {
        self.label_width = Some(width.max(0.0));
        self
    }

    pub fn max_items(mut self, max: usize) -> Self {
        self.max_items = Some(max);
        self
    }

    pub fn expanded(mut self, expanded: impl Into<Binding<bool>>) -> Self {
        self.expanded = Some(expanded.into());
        self
    }

    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn title<V: IntoView>(mut self, title: impl FnOnce() -> V) -> Self {
        self.title = Some(title().into_view());
        self
    }
}

impl IntoView for MetadataList {
    fn into_view(self) -> View {
        let Self {
            root,
            mut items,
            columns,
            position,
            label_width,
            max_items,
            expanded,
            orientation,
            title,
        } = self;
        let horizontal = orientation == Orientation::Horizontal;
        let single = matches!(
            columns,
            MetadataColumns::Single | MetadataColumns::Count(0 | 1)
        );
        let position = if horizontal {
            LabelPosition::Top
        } else {
            position.unwrap_or(if single {
                LabelPosition::Start
            } else {
                LabelPosition::Top
            })
        };
        let extra = if horizontal {
            Vec::new()
        } else {
            max_items
                .filter(|&limit| limit < items.len())
                .map_or_else(Vec::new, |limit| items.split_off(limit))
        };
        let label_width = if position == LabelPosition::Start && label_width.is_none() {
            let rects: Vec<_> = items
                .iter()
                .chain(&extra)
                .map(|item| item.label_ref.rect())
                .collect();
            memo(move || {
                rects
                    .iter()
                    .filter_map(|rect| rect.get().map(|rect| rect.width))
                    .reduce(f32::max)
            })
        } else {
            label_width.into_value()
        };
        let render = |items: Vec<MetadataItem>| {
            if horizontal {
                let items = items
                    .into_iter()
                    .map(|item| item.render(position, label_width))
                    .collect::<Vec<_>>();
                return row(items).style(style().wrap(true).gap(16));
            }
            let columns = match columns {
                MetadataColumns::Single => Track::fr(1),
                MetadataColumns::Count(count) => {
                    Track::repeat(u16::from(count.max(1)), Track::minmax(0, Track::fr(1)))
                }
                MetadataColumns::Multi => Track::repeat_auto_fill(Track::minmax(280, Track::fr(1))),
            };
            let gap = style()
                .grid_template_columns(columns)
                .row_gap(8)
                .column_gap(16);
            let items = items
                .into_iter()
                .map(|item| item.render(position, label_width))
                .collect::<Vec<_>>();
            grid(items).style(gap)
        };
        let main = render(items);
        let disclosure = (!extra.is_empty()).then(|| {
            let open = expanded.unwrap_or_else(|| signal(false).into());
            let disclosure = use_disclosure(open);
            let content = render(extra).part(disclosure.content());
            let trigger = node()
                .part(
                    disclosure
                        .trigger()
                        .name(move || if open.get() { "Show less" } else { "Show more" }),
                )
                .base_style(move || {
                    let p = tokens();
                    style()
                        .align_self(Align::Start)
                        .padding_x(8)
                        .padding_y(4)
                        .radius(p.radius.element)
                        .color(p.colors.text_secondary)
                        .hover(style().background(p.colors.overlay_hover))
                        .focus_visible(style().ring(p.sizes.focus_ring_width, p.colors.focus_ring))
                })
                .child(text(
                    move || if open.get() { "Show less" } else { "Show more" },
                ));
            column((content, trigger)).style(style().gap(8))
        });
        root.base_style(style().direction(Direction::Column).gap(8))
            .child((title, main, disclosure))
            .into_view()
    }
}
