//! Role: Join independently framed form controls in one labelled row.
use crate::recipes_field::height;
use crate::theme_access::tokens;
use kivact::Value;
use kivact::control::{Node, NodeBuilder, column, node, row, text};
use kivact::primitive::NodeExt;
use kivact::style::{Align, Direction, Style, style};
use kivact::ui::ControlSize;
use kivact::view::{IntoText, IntoView, ReadingDirection, Role, View, current_direction, node_ref};

/// Controls that can have their shared edges styled by an input group.
pub trait InputGroupItems {
    fn joined(self, size: ControlSize, rtl: bool) -> Vec<View>;
}

fn join<V: NodeBuilder + IntoView>(
    child: V,
    size: ControlSize,
    rtl: bool,
    first: bool,
    last: bool,
) -> View {
    let (left, right) = if rtl { (last, first) } else { (first, last) };
    child
        .style(move || {
            let t = tokens();
            let left = if left { t.radius.element } else { 0.0 };
            let right = if right { t.radius.element } else { 0.0 };
            let s = style()
                .height(height(&t, size))
                .radius_corners(left, right, right, left);
            let s = if first {
                s
            } else if rtl {
                s.margin_right(-1)
            } else {
                s.margin_left(-1)
            };
            s.focus(style().z_index(1))
        })
        .into_view()
}

impl<V: NodeBuilder + IntoView> InputGroupItems for Vec<V> {
    fn joined(self, size: ControlSize, rtl: bool) -> Vec<View> {
        let count = self.len();
        self.into_iter()
            .enumerate()
            .map(|(i, child)| join(child, size, rtl, i == 0, i + 1 == count))
            .collect()
    }
}

macro_rules! group_items {
    ($first:ident; $($middle:ident),*; $last:ident) => {
        impl<$first: NodeBuilder + IntoView, $($middle: NodeBuilder + IntoView,)* $last: NodeBuilder + IntoView>
            InputGroupItems for ($first, $($middle,)* $last)
        {
            #[allow(non_snake_case, reason = "Tuple type identifiers also bind the tuple values.")]
            fn joined(self, size: ControlSize, rtl: bool) -> Vec<View> {
                let ($first, $($middle,)* $last) = self;
                vec![join($first, size, rtl, true, false), $(join($middle, size, rtl, false, false),)* join($last, size, rtl, false, true)]
            }
        }
    };
}

group_items!(A;; B);
group_items!(A; B; C);
group_items!(A; B, C; D);

pub struct InputGroup<C> {
    root: Node,
    label: Value<String>,
    controls: C,
    size: ControlSize,
}

pub fn input_group<C: InputGroupItems>(label: impl IntoText, controls: C) -> InputGroup<C> {
    InputGroup {
        root: node(),
        label: label.into_text_value(),
        controls,
        size: ControlSize::Medium,
    }
}

impl<C> InputGroup<C> {
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
}

impl<C> NodeBuilder for InputGroup<C> {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl<C: InputGroupItems + 'static> IntoView for InputGroup<C> {
    fn into_view(self) -> View {
        let Self {
            root,
            label,
            controls,
            size,
        } = self;
        let label_ref = node_ref();
        let rtl = current_direction() == ReadingDirection::Rtl;
        let members = row(controls.joined(size, rtl))
            .role(Role::Group)
            .name(label)
            .labelled_by(label_ref)
            .style(
                style()
                    .direction(if rtl {
                        Direction::RowReverse
                    } else {
                        Direction::Row
                    })
                    .align_items(Align::Stretch),
            );
        root.child(
            column((
                text(label).node_ref(label_ref).style(move || {
                    let t = tokens();
                    style()
                        .font_size(t.typography.body.size_px)
                        .weight(t.typography.weight_medium)
                        .color(t.colors.text_primary)
                }),
                members,
            ))
            .style(style().gap(4)),
        )
        .into_view()
    }
}

pub fn input_group_text(label: impl IntoText) -> Node {
    node().child(text(label)).style(move || {
        let t = tokens();
        Style::default()
            .padding_x(8)
            .align_items(Align::Center)
            .justify_content(kivact::style::Justify::Center)
            .border(1, t.colors.border_emphasized)
            .background(t.colors.accent_muted)
            .color(t.colors.text_primary)
            .font_size(14)
    })
}
