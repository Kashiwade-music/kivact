//! Role: Key-value metadata arranged in a neutral grid.
use kivact::view::IntoChildren;

use crate::tokens;
use kivact::i18n::message;
use kivact::interaction::use_disclosure;
use kivact::state::Binding;
use kivact::state::IntoBinding;
use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::state::memo;
use kivact::state::signal;
use kivact::style::Align;
use kivact::style::Direction;
use kivact::style::Track;
use kivact::style::style;
use kivact::view::IntoView;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::NodeRef;
use kivact::view::Orientation;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::column;
use kivact::view::grid;
use kivact::view::node;
use kivact::view::node_ref;
use kivact::view::row;
use kivact::view::text;

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
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl MetadataItem {
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }

    fn render(self, position: LabelPosition, width: Value<Option<f64>>) -> View {
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
            .role(Role::Group)
            .labelled_by(label_ref)
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
        if let Some(layout) =
            kivact::state::try_use_context::<MetadataLayout>().map(|value| value.get())
        {
            layout.labels.update(|labels| labels.push(self.label_ref));
            self.render(layout.position, layout.width)
        } else {
            self.render(LabelPosition::Top, None.into_value())
        }
    }
}

#[derive(Clone)]
struct MetadataLayout {
    position: LabelPosition,
    width: Value<Option<f64>>,
    labels: kivact::state::Signal<Vec<NodeRef>>,
}

pub struct MetadataList<C> {
    root: Node,
    items: C,
    columns: Value<MetadataColumns>,
    position: Option<LabelPosition>,
    label_width: Option<f64>,
    max_items: Option<usize>,
    expanded: Option<Binding<bool>>,
    orientation: Value<Orientation>,
    title: Option<View>,
}

pub fn metadata_list<C: IntoChildren>(items: C) -> MetadataList<C> {
    MetadataList {
        root: node(),
        items,
        columns: (MetadataColumns::Single).into_value(),
        position: None,
        label_width: None,
        max_items: None,
        expanded: None,
        orientation: (Orientation::Vertical).into_value(),
        title: None,
    }
}

impl<C> NodeBuilder for MetadataList<C> {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl<C> MetadataList<C> {
    pub fn columns(mut self, columns: impl IntoValue<MetadataColumns>) -> Self {
        self.columns = columns.into_value();
        self
    }

    pub fn label_position(mut self, position: LabelPosition) -> Self {
        self.position = Some(position);
        self
    }

    pub fn label_width(mut self, width: f64) -> Self {
        self.label_width = Some(width.max(0.0));
        self
    }

    pub fn max_items(mut self, max: usize) -> Self {
        self.max_items = Some(max);
        self
    }

    pub fn expanded(mut self, expanded: impl IntoBinding<bool>) -> Self {
        self.expanded = Some(expanded.into_binding());
        self
    }

    pub fn orientation(mut self, orientation: impl IntoValue<Orientation>) -> Self {
        self.orientation = orientation.into_value();
        self
    }

    pub fn title(mut self, title: impl IntoView) -> Self {
        self.title = Some(title.into_view());
        self
    }
}

impl<C: IntoChildren> IntoView for MetadataList<C> {
    fn into_view(self) -> View {
        let Self {
            root,
            items,
            columns,
            position,
            label_width,
            max_items,
            expanded,
            orientation,
            title,
        } = self;
        let horizontal = orientation.get() == Orientation::Horizontal;
        let single = matches!(
            columns.get(),
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
        let labels = signal(Vec::new());
        let width = if position == LabelPosition::Start && label_width.is_none() {
            memo(move || {
                labels.with(|labels: &Vec<NodeRef>| {
                    labels
                        .iter()
                        .filter_map(|label| label.rect().get().map(|rect| rect.width))
                        .reduce(f64::max)
                })
            })
        } else {
            label_width.into_value()
        };
        let children = kivact::state::provide(
            MetadataLayout {
                position,
                width,
                labels,
            },
            move || {
                let mut items = items.into_views();
                let extra = if horizontal {
                    Vec::new()
                } else {
                    max_items
                        .filter(|&limit| limit < items.len())
                        .map_or_else(Vec::new, |limit| items.split_off(limit))
                };
                let render = |items: Vec<View>| {
                    if horizontal {
                        let items = items.into_iter().map(|item| item).collect::<Vec<_>>();
                        return row(items).style(style().wrap(true).gap(16));
                    }
                    let columns = match columns.get() {
                        MetadataColumns::Single => Track::fr(1),
                        MetadataColumns::Count(count) => {
                            Track::repeat(u16::from(count.max(1)), Track::minmax(0, Track::fr(1)))
                        }
                        MetadataColumns::Multi => {
                            Track::repeat_auto_fill(Track::minmax(280, Track::fr(1)))
                        }
                    };
                    let gap = style()
                        .grid_template_columns(columns)
                        .row_gap(8)
                        .column_gap(16);
                    let items = items.into_iter().map(|item| item).collect::<Vec<_>>();
                    grid(items).style(gap)
                };
                let main = render(items);
                let disclosure = (!extra.is_empty()).then(|| {
                    let open = expanded.unwrap_or_else(|| signal(false).into_binding());
                    let show_less = message("soft.metadata_list.show_less", "Show less");
                    let show_more = message("soft.metadata_list.show_more", "Show more");
                    let label = move || {
                        if open.get() {
                            show_less.get()
                        } else {
                            show_more.get()
                        }
                    };
                    let disclosure = use_disclosure(open);
                    let content = render(extra).part(disclosure.content());
                    let trigger = node()
                        .part(disclosure.trigger())
                        .name(label)
                        .base_style(move || {
                            let p = tokens();
                            style()
                                .align_self(Align::Start)
                                .padding_x(8)
                                .padding_y(4)
                                .radius(p.radius.element)
                                .color(p.colors.text_secondary)
                                .hover(style().background(p.colors.overlay_hover))
                                .focus_visible(
                                    style().ring(p.sizes.focus_ring_width, p.colors.focus_ring),
                                )
                        })
                        .child(text(label));
                    column((content, trigger)).style(style().gap(8))
                });
                (main, disclosure)
            },
        );
        root.base_style(style().direction(Direction::Column).gap(8))
            .child((title, children))
            .into_view()
    }
}

kivact::state::constant_value!(MetadataColumns);
