//! Role: Join independently framed form controls in one labelled row.
use crate::recipes_field::height;
use crate::theme_access::tokens;
use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::state::provide;
use kivact::state::try_use_context;
use kivact::style::Align;
use kivact::style::Direction;
use kivact::style::Style;
use kivact::style::style;
use kivact::ui::ControlSize;
use kivact::ui::provide_control_size;
use kivact::view::IntoChildren;
use kivact::view::IntoView;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::ReadingDirection;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::column;
use kivact::view::node;
use kivact::view::node_ref;
use kivact::view::row;
use kivact::view::text;
use kivact::view::use_direction;

#[derive(Clone, Copy)]
struct InheritMemberSize(bool);

fn join<V: NodeBuilder + IntoView>(
    child: V,
    size: ControlSize,
    rtl: bool,
    first: bool,
    last: bool,
) -> View {
    let (left, right) = if rtl { (last, first) } else { (first, last) };
    let reference = node_ref();
    let focus_within = reference.focus_within();
    let size_style = move || style().height(height(&tokens(), size));
    let child = if try_use_context::<InheritMemberSize>()
        .map(|value| value.get())
        .is_some_and(|setting| setting.0)
    {
        child.with_node(move |node| node.base_style(size_style))
    } else {
        child.style(size_style)
    };
    child
        .node_ref(reference)
        .style(move || {
            let t = tokens();
            let left = if left { t.radius.element } else { 0.0 };
            let right = if right { t.radius.element } else { 0.0 };
            let s = style().radius_corners(left, right, right, left);
            let s = if first {
                s
            } else if rtl {
                s.margin_right(-1)
            } else {
                s.margin_left(-1)
            };
            if focus_within.get() { s.z_index(1) } else { s }
        })
        .into_view()
}

pub struct InputGroup<C> {
    root: Node,
    label: Value<String>,
    controls: Controls<C>,
    size: Value<ControlSize>,
}

enum Controls<C> {
    Built(C),
    Scoped(Box<dyn FnOnce() -> C>),
}

pub fn input_group<C: IntoChildren>(label: impl IntoText, controls: C) -> InputGroup<C> {
    InputGroup {
        root: node(),
        label: label.into_text_value(),
        controls: Controls::Built(controls),
        size: (ControlSize::Medium).into_value(),
    }
}

/// Build the controls in the group's size scope. A control with its own size
/// uses that size. Unstyled members still receive the group's default height.
pub fn input_group_with<C: IntoChildren>(
    label: impl IntoText,
    controls: impl FnOnce() -> C + 'static,
) -> InputGroup<C> {
    InputGroup {
        root: node(),
        label: label.into_text_value(),
        controls: Controls::Scoped(Box::new(controls)),
        size: (ControlSize::Medium).into_value(),
    }
}

impl<C> InputGroup<C> {
    pub fn size(mut self, size: impl IntoValue<ControlSize>) -> Self {
        self.size = size.into_value();
        self
    }
}

impl<C> NodeBuilder for InputGroup<C> {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl<C: IntoChildren + 'static> IntoView for InputGroup<C> {
    fn into_view(self) -> View {
        let Self {
            root,
            label,
            controls,
            size,
        } = self;
        let label_ref = node_ref();
        let rtl = use_direction().get() == ReadingDirection::Rtl;
        let members = move |controls: C| {
            row({
                let children = controls.into_views();
                let count = children.len();
                children
                    .into_iter()
                    .enumerate()
                    .map(|(index, child)| {
                        join(
                            node().child(child),
                            size.get(),
                            rtl,
                            index == 0,
                            index + 1 == count,
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .with_node(move |node| node.role(Role::Group))
            .name(label)
            .with_node(move |node| node.labelled_by(label_ref))
            .style(
                style()
                    .direction(if rtl {
                        Direction::RowReverse
                    } else {
                        Direction::Row
                    })
                    .align_items(Align::Stretch),
            )
        };
        let members = match controls {
            Controls::Built(controls) => {
                provide(InheritMemberSize(false), move || members(controls))
            }
            Controls::Scoped(controls) => provide_control_size(size.get(), move || {
                provide(InheritMemberSize(true), move || members(controls()))
            }),
        };
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

pub fn input_group_text(label: impl IntoText) -> impl kivact::view::Component {
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
