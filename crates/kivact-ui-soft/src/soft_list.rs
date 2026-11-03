//! Role: Compose static lists and Neutral item rows.

use crate::tokens;
use kivact::control::{Node, NodeBuilder, column, node, text};
use kivact::primitive::{NodeExt, part};
use kivact::style::{Align, Cursor, Direction, Justify, TextOverflow, style};
use kivact::view::{HandlerResult, IntoText, IntoValue, IntoView, NodeRef, Role, View, node_ref};
use kivact::{Callback, Value};

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

type Slot = Box<dyn FnOnce() -> View>;

fn slot<V: IntoView>(content: impl FnOnce() -> V + 'static) -> Slot {
    Box::new(move || content().into_view())
}

pub struct Item {
    root: Node,
    label: Value<String>,
    label_view: Option<Slot>,
    description: Option<Value<String>>,
    description_view: Option<Slot>,
    start: Option<Slot>,
    end: Option<Slot>,
    marker: Option<Slot>,
    density: ItemDensity,
    align: ItemAlign,
    layout: ItemLayout,
    label_lines: u32,
    description_lines: u32,
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
        density: ItemDensity::Balanced,
        align: ItemAlign::Center,
        layout: ItemLayout::Stacked,
        label_lines: 1,
        description_lines: 1,
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
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl Item {
    pub fn label_view<V: IntoView>(mut self, content: impl FnOnce() -> V + 'static) -> Self {
        self.label_view = Some(slot(content));
        self
    }

    pub fn description(mut self, description: impl IntoText) -> Self {
        self.description = Some(description.into_text_value());
        self
    }

    pub fn description_view<V: IntoView>(mut self, content: impl FnOnce() -> V + 'static) -> Self {
        self.description_view = Some(slot(content));
        self
    }

    pub fn start<V: IntoView>(mut self, content: impl FnOnce() -> V + 'static) -> Self {
        self.start = Some(slot(content));
        self
    }

    pub fn end<V: IntoView>(mut self, content: impl FnOnce() -> V + 'static) -> Self {
        self.end = Some(slot(content));
        self
    }

    pub fn marker<V: IntoView>(mut self, content: impl FnOnce() -> V + 'static) -> Self {
        self.marker = Some(slot(content));
        self
    }

    pub fn align(mut self, align: ItemAlign) -> Self {
        self.align = align;
        self
    }

    pub fn density(mut self, density: ItemDensity) -> Self {
        self.density = density;
        self
    }

    pub fn layout(mut self, layout: ItemLayout) -> Self {
        self.layout = layout;
        self
    }

    pub fn label_lines(mut self, lines: u32) -> Self {
        self.label_lines = lines.max(1);
        self
    }

    pub fn description_lines(mut self, lines: u32) -> Self {
        self.description_lines = lines.max(1);
        self
    }

    pub fn on_press<R: HandlerResult>(mut self, press: impl Fn() -> R + 'static) -> Self {
        self.on_press = Some(kivact::view::callback(move |_| press()));
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

impl IntoView for Item {
    fn into_view(self) -> View {
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
        let mut attachment = part(|state| state)
            .role(role)
            .name(label)
            .disabled(disabled)
            .selected(selected);
        if description.is_some() || description_view.is_some() {
            attachment = attachment.described_by(description_ref);
        }
        if let Some(target) = delegate {
            attachment = attachment.on_press(move || target.activate());
        } else if let Some(on_press) = on_press {
            attachment = attachment
                .focusable(true)
                .on_press(move || on_press.call(()));
        }
        let label = match label_view {
            Some(content) => content(),
            None => text(label)
                .style(move || {
                    let p = tokens();
                    style()
                        .font_size(14)
                        .line_height(20.0 / 14.0)
                        .color(p.colors.text_primary)
                        .line_clamp(label_lines)
                        .text_overflow(TextOverflow::Ellipsis)
                })
                .into_view(),
        };
        let description = match description_view {
            Some(content) => Some(content()),
            None => description.map(|value| {
                text(value)
                    .style(move || {
                        let p = tokens();
                        style()
                            .font_size(12)
                            .line_height(20.0 / 12.0)
                            .color(p.colors.text_secondary)
                            .line_clamp(description_lines)
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
                    .direction(match layout {
                        ItemLayout::Stacked => Direction::Column,
                        ItemLayout::Inline => Direction::Row,
                    })
                    .gap(if layout == ItemLayout::Inline { 8 } else { 0 }),
            )
            .child((
                label,
                description.map(|content| node().node_ref(description_ref).child(content)),
            ));
        root.part(attachment)
            .base_style(move || {
                let p = tokens();
                let mut row = style()
                    .direction(Direction::Row)
                    .min_width(0)
                    .padding_x(12)
                    .padding_y(density.padding())
                    .gap(8)
                    .radius(p.radius.element)
                    .align_items(if align == ItemAlign::Center {
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
                        .focus_visible(style().ring(p.sizes.focus_ring_width, p.colors.focus_ring))
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
            .child((
                marker.map(|content| content()),
                start.map(|content| content()),
                content,
                end.map(|content| content()),
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
pub trait IntoListItems {
    fn into_list_items(self) -> Vec<Item>;
}

impl IntoListItems for Item {
    fn into_list_items(self) -> Vec<Item> {
        vec![self]
    }
}

impl IntoListItems for Vec<Item> {
    fn into_list_items(self) -> Vec<Item> {
        self
    }
}

impl<const N: usize> IntoListItems for [Item; N] {
    fn into_list_items(self) -> Vec<Item> {
        self.into()
    }
}

macro_rules! list_tuples {
    ($($item:ident),+) => {
        impl<$($item: IntoListItems),+> IntoListItems for ($($item,)+) {
            #[allow(non_snake_case, reason = "Tuple type identifiers also bind the tuple values.")]
            fn into_list_items(self) -> Vec<Item> {
                let ($($item,)+) = self;
                let mut items = Vec::new();
                $(items.extend($item.into_list_items());)+
                items
            }
        }
    };
}

list_tuples!(A);
list_tuples!(A, B);
list_tuples!(A, B, C);
list_tuples!(A, B, C, D);
list_tuples!(A, B, C, D, E);
list_tuples!(A, B, C, D, E, F);

pub struct List {
    root: Node,
    items: Vec<Item>,
    header: Option<Slot>,
    density: ListDensity,
    dividers: bool,
    edge: ListEdge,
    marker: ListMarker,
    start: u32,
}

pub fn list(items: impl IntoListItems) -> List {
    List {
        root: node(),
        items: items.into_list_items(),
        header: None,
        density: ListDensity::Balanced,
        dividers: false,
        edge: ListEdge::Padded,
        marker: ListMarker::None,
        start: 1,
    }
}

impl NodeBuilder for List {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl List {
    pub fn header<V: IntoView>(mut self, content: impl FnOnce() -> V + 'static) -> Self {
        self.header = Some(slot(content));
        self
    }

    pub fn density(mut self, density: ListDensity) -> Self {
        self.density = density;
        self
    }

    pub fn dividers(mut self, dividers: bool) -> Self {
        self.dividers = dividers;
        self
    }

    pub fn edge(mut self, edge: ListEdge) -> Self {
        self.edge = edge;
        self
    }

    pub fn marker(mut self, marker: ListMarker) -> Self {
        self.marker = marker;
        self
    }

    pub fn start(mut self, start: u32) -> Self {
        self.start = start;
        self
    }
}

impl IntoView for List {
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
        let count = items.len();
        let marker_width = (start as u64 + count.saturating_sub(1) as u64)
            .to_string()
            .len()
            .max(1) as f32
            * 8.0
            + 8.0;
        let header_ref = node_ref();
        let mut semantics = part(|state| state).role(Role::List);
        if header.is_some() {
            semantics = semantics.labelled_by(header_ref);
        }
        let rows = items
            .into_iter()
            .enumerate()
            .map(|(index, mut item)| {
                item.list_item = true;
                item.density = density;
                if marker != ListMarker::None {
                    let glyph = match marker {
                        ListMarker::None => String::new(),
                        ListMarker::Disc => "•".to_owned(),
                        ListMarker::Circle => "◦".to_owned(),
                        ListMarker::Decimal => format!("{}.", start as u64 + index as u64),
                    };
                    item = item.marker(move || {
                        node()
                            .style(
                                style()
                                    .direction(Direction::Row)
                                    .justify_content(Justify::End)
                                    .width(marker_width)
                                    .shrink(0),
                            )
                            .child(text(glyph))
                    });
                }
                item.style(move || {
                    let mut row = style();
                    if matches!(edge, ListEdge::Flush) {
                        row = row.padding_x(0);
                    }
                    if matches!(edge, ListEdge::Bleed) {
                        row = row.radius(0);
                    }
                    if dividers && index + 1 < count {
                        row = row.border_bottom(1, tokens().colors.border);
                    }
                    row
                })
                .into_view()
            })
            .collect::<Vec<_>>();
        column((
            header.map(|content| node().node_ref(header_ref).child(content())),
            root.part(semantics)
                .base_style(
                    style()
                        .direction(Direction::Column)
                        .gap(if dividers { 0 } else { 2 }),
                )
                .child(rows),
        ))
        .into_view()
    }
}
