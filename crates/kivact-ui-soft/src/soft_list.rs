//! Role: Static lists and neutral item rows.
use kivact::view::IntoChildren;

use kivact::state::ItemKey;
use kivact::state::PropValue;
use std::collections::HashMap;
use std::rc::Rc;

use crate::tokens;
use kivact::state::Callback;
use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::state::memo;
use kivact::style::Align;
use kivact::style::Cursor;
use kivact::style::Direction;
use kivact::style::Justify;
use kivact::style::TextOverflow;
use kivact::style::style;
use kivact::view::IntoView;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::NodeRef;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::column;
use kivact::view::for_each;
use kivact::view::node;
use kivact::view::node_ref;
use kivact::view::text;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ItemDensity {
    Compact,
    #[default]
    Balanced,
    Spacious,
}

impl ItemDensity {
    pub(crate) fn padding(self) -> i32 {
        match self {
            Self::Compact => 4,
            Self::Balanced => 8,
            Self::Spacious => 12,
        }
    }
}

pub type ListDensity = ItemDensity;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ItemAlign {
    #[default]
    Center,
    Start,
    FirstLine,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ItemLayout {
    #[default]
    Stacked,
    Inline,
}

pub struct Item {
    root: Node,
    label: Value<String>,
    label_view: Option<View>,
    description: Option<Value<String>>,
    description_view: Option<View>,
    start: Option<View>,
    end: Option<View>,
    marker: Option<View>,
    density: Value<ItemDensity>,
    align: Value<ItemAlign>,
    layout: Value<ItemLayout>,
    label_lines: Value<u32>,
    description_lines: Value<u32>,
    on_press: Option<Callback<()>>,
    delegate: Option<NodeRef>,
    disabled: Value<bool>,
    selected: Value<bool>,
    highlighted: Value<bool>,
    list_item: bool,
}

pub fn item(label: impl IntoText) -> Item {
    Item {
        root: node(),
        label: label.into_text_value(),
        label_view: None,
        description: None,
        description_view: None,
        start: None,
        end: None,
        marker: None,
        density: (ItemDensity::Balanced).into_value(),
        align: (ItemAlign::Center).into_value(),
        layout: (ItemLayout::Stacked).into_value(),
        label_lines: 1.into_value(),
        description_lines: 1.into_value(),
        on_press: None,
        delegate: None,
        disabled: false.into_value(),
        selected: false.into_value(),
        highlighted: false.into_value(),
        list_item: false,
    }
}

pub fn list_item(label: impl IntoText) -> Item {
    Item {
        list_item: true,
        ..item(label)
    }
}

impl NodeBuilder for Item {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl Item {
    pub fn label_view(mut self, content: impl IntoView) -> Self {
        self.label_view = Some(content.into_view());
        self
    }

    pub fn description(mut self, description: impl IntoText) -> Self {
        self.description = Some(description.into_text_value());
        self
    }

    pub fn description_view(mut self, content: impl IntoView) -> Self {
        self.description_view = Some(content.into_view());
        self
    }

    pub fn start(mut self, content: impl IntoView) -> Self {
        self.start = Some(content.into_view());
        self
    }

    pub fn end(mut self, content: impl IntoView) -> Self {
        self.end = Some(content.into_view());
        self
    }

    pub fn marker(mut self, content: impl IntoView) -> Self {
        self.marker = Some(content.into_view());
        self
    }

    pub fn align(mut self, align: impl IntoValue<ItemAlign>) -> Self {
        self.align = align.into_value();
        self
    }

    pub fn density(mut self, density: impl IntoValue<ItemDensity>) -> Self {
        self.density = density.into_value();
        self
    }

    pub fn layout(mut self, layout: impl IntoValue<ItemLayout>) -> Self {
        self.layout = layout.into_value();
        self
    }

    pub fn label_lines(mut self, lines: impl IntoValue<u32>) -> Self {
        let lines = lines.into_value();
        self.label_lines = (move || lines.get().max(1)).into_value();
        self
    }

    pub fn description_lines(mut self, lines: impl IntoValue<u32>) -> Self {
        let lines = lines.into_value();
        self.description_lines = (move || lines.get().max(1)).into_value();
        self
    }

    pub fn on_press(mut self, press: impl Into<Callback<()>>) -> Self {
        let press: Callback<()> = press.into();
        self.on_press = Some(kivact::state::callback(move |_| press.call(())));
        self
    }

    pub fn delegate_to(mut self, target: NodeRef) -> Self {
        self.delegate = Some(target);
        self
    }

    pub fn disabled(mut self, value: impl IntoValue<bool>) -> Self {
        self.disabled = value.into_value();
        self
    }

    pub fn selected(mut self, value: impl IntoValue<bool>) -> Self {
        self.selected = value.into_value();
        self
    }

    pub fn highlighted(mut self, value: impl IntoValue<bool>) -> Self {
        self.highlighted = value.into_value();
        self
    }
}

impl Item {
    fn render(self) -> View {
        let Self {
            root,
            label,
            label_view,
            description,
            description_view,
            start,
            end,
            marker,
            density,
            align,
            layout,
            label_lines,
            description_lines,
            on_press,
            disabled,
            selected,
            highlighted,
            list_item,
            delegate,
        } = self;
        let actionable = on_press.is_some() || delegate.is_some();
        let role = if list_item {
            Role::ListItem
        } else if actionable {
            Role::Button
        } else {
            Role::Group
        };
        let description_ref = node_ref();
        let mut attached_root = root
            .with_node(move |node| node.role(role))
            .name(label)
            .disabled(disabled)
            .selected(selected);
        if description.is_some() || description_view.is_some() {
            attached_root = attached_root.with_node(move |node| node.described_by(description_ref));
        }
        if let Some(target) = delegate {
            attached_root = attached_root.on_press(move || target.activate());
        } else if let Some(on_press) = on_press {
            attached_root = attached_root
                .focusable(true)
                .on_press(move || on_press.call(()));
        }
        let label = match label_view {
            Some(content) => content,
            None => text(label)
                .style(move || {
                    let p = tokens();
                    style()
                        .font_size(14)
                        .line_height(20.0 / 14.0)
                        .color(p.colors.text_primary)
                        .line_clamp(label_lines.get())
                        .text_overflow(TextOverflow::Ellipsis)
                })
                .into_view(),
        };
        let description = match description_view {
            Some(content) => Some(content),
            None => description.map(|value| {
                text(value)
                    .style(move || {
                        let p = tokens();
                        style()
                            .font_size(12)
                            .line_height(20.0 / 12.0)
                            .color(p.colors.text_secondary)
                            .line_clamp(description_lines.get())
                            .text_overflow(TextOverflow::Ellipsis)
                    })
                    .into_view()
            }),
        };
        let content = node()
            .style(
                style()
                    .grow(1)
                    .min_width(0)
                    .direction(match layout.get() {
                        ItemLayout::Stacked => Direction::Column,
                        ItemLayout::Inline => Direction::Row,
                    })
                    .gap(if layout.get() == ItemLayout::Inline {
                        8
                    } else {
                        0
                    }),
            )
            .child((
                label,
                description.map(|content| node().node_ref(description_ref).child(content)),
            ));
        attached_root
            .with_node(move |node| {
                node.base_style(move || {
                    let p = tokens();
                    let mut row = style()
                        .direction(Direction::Row)
                        .min_width(0)
                        .padding_x(12)
                        .padding_y(density.get().padding())
                        .gap(8)
                        .radius(p.radius.element)
                        .align_items(if align.get() == ItemAlign::Center {
                            Align::Center
                        } else {
                            Align::Start
                        });
                    if selected.get() {
                        row = row.background(p.colors.accent_muted);
                    } else if highlighted.get() {
                        row = row.background(p.colors.overlay_hover);
                    }
                    if actionable {
                        row = row
                            .cursor(Cursor::Pointer)
                            .hover(style().background(p.colors.overlay_hover))
                            .pressed(style().background(p.colors.overlay_pressed))
                            .focus_visible(
                                style().ring(p.sizes.focus_ring_width, p.colors.focus_ring),
                            )
                            .transition(
                                kivact::style::TransitionProp::Paint,
                                kivact::motion::Transition::tween(
                                    p.motion.fast as f64,
                                    p.motion.ease_standard,
                                ),
                            );
                    }
                    row
                })
            })
            .child((
                marker.map(|content| content),
                start.map(|content| content),
                content,
                end.map(|content| content),
            ))
            .into_view()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ListEdge {
    #[default]
    Padded,
    Flush,
    Outset,
    Bleed,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ListMarker {
    #[default]
    None,
    Disc,
    Circle,
    Decimal,
}

/// Conversion implemented for individual list rows, tuples of rows, and vectors.
enum ListRows<C> {
    Static(C),
    Reactive(kivact::view::ViewFn<ListAppearance>),
}

#[derive(Clone, Copy)]
struct ListAppearance {
    density: Value<ListDensity>,
    dividers: Value<bool>,
    edge: Value<ListEdge>,
    marker: Value<ListMarker>,
    start: Value<u32>,
}

pub struct List<C> {
    root: Node,
    items: ListRows<C>,
    header: Option<View>,
    density: Value<ListDensity>,
    dividers: Value<bool>,
    edge: Value<ListEdge>,
    marker: Value<ListMarker>,
    start: Value<u32>,
}

pub fn list<C: IntoChildren>(items: C) -> List<C> {
    List {
        root: node(),
        items: ListRows::Static(items),
        header: None,
        density: (ListDensity::Balanced).into_value(),
        dividers: (false).into_value(),
        edge: (ListEdge::Padded).into_value(),
        marker: (ListMarker::None).into_value(),
        start: 1.into_value(),
    }
}

/// Render a static-semantics list from a keyed reactive source.
pub fn list_items<T, K>(
    items: impl IntoValue<Vec<T>>,
    key: impl Fn(&T) -> K + 'static,
    render: impl Fn(Value<T>) -> Item + 'static,
) -> List<()>
where
    T: PropValue,
    K: ItemKey,
{
    let items = items.into_value();
    let key = Rc::new(key);
    let render = Rc::new(render);
    List {
        items: ListRows::Reactive(kivact::view::ViewFn::from(
            move |appearance: ListAppearance| {
                let key = key.clone();
                let row_key = key.clone();
                let render = render.clone();
                let order_key = key.clone();
                let order = memo(move || {
                    items.with(|items| {
                        items
                            .iter()
                            .enumerate()
                            .map(|(index, item)| (order_key(item), index))
                            .collect::<HashMap<_, _>>()
                    })
                });
                for_each(
                    items,
                    move |item| key(item),
                    move |item| {
                        let key = item.with_untracked(|item| row_key(item));
                        let position = memo(move || {
                            order.with(|order| {
                                (order.get(&key).copied().unwrap_or_default(), order.len())
                            })
                        });
                        appearance.decorate(render(item), position)
                    },
                )
            },
        )),
        ..list(())
    }
}

impl<C> NodeBuilder for List<C> {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl<C> List<C> {
    pub fn header(mut self, content: impl IntoView) -> Self {
        self.header = Some(content.into_view());
        self
    }

    pub fn density(mut self, density: impl IntoValue<ListDensity>) -> Self {
        self.density = density.into_value();
        self
    }

    pub fn dividers(mut self, dividers: impl IntoValue<bool>) -> Self {
        self.dividers = dividers.into_value();
        self
    }

    pub fn edge(mut self, edge: impl IntoValue<ListEdge>) -> Self {
        self.edge = edge.into_value();
        self
    }

    pub fn marker(mut self, marker: impl IntoValue<ListMarker>) -> Self {
        self.marker = marker.into_value();
        self
    }

    pub fn start(mut self, start: impl IntoValue<u32>) -> Self {
        self.start = start.into_value();
        self
    }
}

impl<C: IntoChildren> IntoView for List<C> {
    fn into_view(self) -> View {
        let Self {
            root,
            items,
            header,
            density,
            dividers,
            edge,
            marker,
            start,
        } = self;
        let appearance = ListAppearance {
            density,
            dividers,
            edge,
            marker,
            start,
        };
        let header_ref = node_ref();
        let mut root = root.with_node(move |node| node.role(Role::List));
        if header.is_some() {
            root = root.with_node(move |node| node.labelled_by(header_ref));
        }
        let rows = match items {
            ListRows::Static(items) => {
                let next = kivact::state::signal(0usize);
                let count = kivact::state::signal(0usize);
                kivact::state::provide(
                    ListLayout {
                        appearance,
                        next,
                        count,
                    },
                    move || {
                        let children = items.into_views();
                        count.set(children.len());
                        children
                    },
                )
            }
            ListRows::Reactive(render) => render.call(appearance),
        };
        column((
            header.map(|content| node().node_ref(header_ref).child(content)),
            root.with_node(move |node| {
                node.base_style(move || {
                    style()
                        .direction(Direction::Column)
                        .gap(if dividers.get() { 0 } else { 2 })
                })
            })
            .child(rows),
        ))
        .into_view()
    }
}

impl ListAppearance {
    fn decorate(self, mut item: Item, position: Value<(usize, usize)>) -> View {
        item.list_item = true;
        item.density = self.density;
        {
            let fallback = item.marker.take();
            let marker = self.marker;
            let start = self.start;
            let generated = node()
                .style(move || {
                    let (_, count) = position.get();
                    let width = if marker.get() == ListMarker::Decimal {
                        let digits = (u64::from(start.get()) + count.saturating_sub(1) as u64)
                            .to_string()
                            .len();
                        digits as f64 * 8.0 + 8.0
                    } else {
                        16.0
                    };
                    style()
                        .direction(Direction::Row)
                        .justify_content(Justify::End)
                        .width(width)
                        .shrink(0)
                })
                .child(text(move || match marker.get() {
                    ListMarker::None => String::new(),
                    ListMarker::Disc => "•".to_owned(),
                    ListMarker::Circle => "◦".to_owned(),
                    ListMarker::Decimal => {
                        format!("{}.", u64::from(start.get()) + position.get().0 as u64)
                    }
                }));
            item = item.marker((
                node()
                    .hidden(move || marker.get() != ListMarker::None)
                    .child(fallback),
                generated.hidden(move || marker.get() == ListMarker::None),
            ));
        }
        item.style(move || {
            let mut row = style();
            if self.edge.get() == ListEdge::Flush {
                row = row.padding_x(0);
            }
            if self.edge.get() == ListEdge::Bleed {
                row = row.radius(0);
            }
            let (index, count) = position.get();
            if self.dividers.get() && index + 1 < count {
                row = row.border_bottom(1, tokens().colors.border);
            }
            row
        })
        .render()
    }
}

#[derive(Clone)]
struct ListLayout {
    appearance: ListAppearance,
    next: kivact::state::Signal<usize>,
    count: kivact::state::Signal<usize>,
}
impl IntoView for Item {
    fn into_view(self) -> View {
        if let Some(layout) =
            kivact::state::try_use_context::<ListLayout>().map(|value| value.get())
        {
            let index = layout.next.get_untracked();
            layout.next.set(index + 1);
            let position = memo(move || (index, layout.count.get()));
            layout.appearance.decorate(self, position)
        } else {
            self.render()
        }
    }
}

kivact::state::constant_value!(ItemAlign, ItemDensity, ItemLayout, ListEdge, ListMarker);
