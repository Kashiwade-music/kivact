//! Role: Joined Neutral action buttons with roving keyboard focus.
use kivact::view::IntoChildren;

use crate::Elevation;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::node;

use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::state::Signal;
use kivact::state::Value;
use kivact::state::signal;
use kivact::style::Align;
use kivact::style::Direction;
use kivact::style::style;
use kivact::ui::Button;
use kivact::ui::ControlSize;
use kivact::view::IntoView;
use kivact::view::Orientation;
use kivact::view::ReadingDirection;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::use_direction;

/// A button-group member with optional explicit divider and size behavior.
pub struct GroupMember {
    button: Button,
    solid: bool,
    keep_size: bool,
}

impl From<Button> for GroupMember {
    fn from(button: Button) -> Self {
        Self {
            button,
            solid: false,
            keep_size: false,
        }
    }
}

impl GroupMember {
    /// Choose the on-accent divider color for a solid primary or danger button.
    pub fn solid(mut self, solid: bool) -> Self {
        self.solid = solid;
        self
    }

    /// Keep the button's own configured size.
    pub fn keep_size(mut self, keep: bool) -> Self {
        self.keep_size = keep;
        self
    }
}

/// Joined controls with a shared name, size, elevation, and Tab stop.
pub struct ButtonGroup<C> {
    root: Node,
    members: C,
    orientation: Value<Orientation>,
    size: Value<ControlSize>,
    elevation: Value<Elevation>,
    disabled: Value<bool>,
}

#[track_caller]
pub fn button_group<C: IntoChildren>(label: impl IntoText, children: C) -> ButtonGroup<C> {
    ButtonGroup {
        root: node()
            .with_node(move |node| node.role(Role::Group))
            .name(label),
        members: children,
        orientation: (Orientation::Horizontal).into_value(),
        size: (ControlSize::Medium).into_value(),
        elevation: (Elevation::None).into_value(),
        disabled: false.into_value(),
    }
}

impl<C> ButtonGroup<C> {
    pub fn orientation(mut self, orientation: impl IntoValue<Orientation>) -> Self {
        self.orientation = orientation.into_value();
        self
    }

    pub fn size(mut self, size: impl IntoValue<ControlSize>) -> Self {
        self.size = size.into_value();
        self
    }

    pub fn elevation(mut self, elevation: impl IntoValue<Elevation>) -> Self {
        self.elevation = elevation.into_value();
        self
    }

    pub fn disabled(mut self, disabled: impl IntoValue<bool>) -> Self {
        self.disabled = disabled.into_value();
        self
    }
}

impl<C> NodeBuilder for ButtonGroup<C> {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

fn corners(vertical: bool, rtl: bool, first: bool, last: bool, r: f64) -> [f64; 4] {
    let (first, last) = (if first { r } else { 0.0 }, if last { r } else { 0.0 });
    match (vertical, rtl) {
        (true, _) => [first, first, last, last],
        (false, true) => [last, first, first, last],
        (false, false) => [first, last, last, first],
    }
}

impl<C: IntoChildren> IntoView for ButtonGroup<C> {
    fn into_view(self) -> View {
        let vertical = self.orientation.get() == Orientation::Vertical;
        let rtl = use_direction().get() == ReadingDirection::Rtl;
        let direction = match (vertical, rtl) {
            (true, _) => Direction::Column,
            (false, true) => Direction::RowReverse,
            _ => Direction::Row,
        };
        let disabled = self.disabled;
        let size = self.size.get();
        let children = kivact::ui::provide_control_size(size, move || {
            let count = signal(0usize);
            let next = signal(0usize);
            kivact::state::provide(
                MemberLayout {
                    size,
                    vertical,
                    rtl,
                    count,
                    next,
                },
                move || {
                    let children = self.members.into_views();
                    count.set(children.len());
                    let count = children.len();
                    children
                        .into_iter()
                        .enumerate()
                        .map(|(index, child)| {
                            node().child(child).style(move || {
                                let t = kivact::style::use_tokens::<crate::Tokens>();
                                let [tl, tr, br, bl] = corners(
                                    vertical,
                                    rtl,
                                    index == 0,
                                    index + 1 == count,
                                    t.radius.element,
                                );
                                style().radius_corners(tl, tr, br, bl).clip(true)
                            })
                        })
                        .collect::<Vec<_>>()
                },
            )
        });
        let elevation = self.elevation.get();
        self.root
            .disabled(disabled)
            .roving_focus(self.orientation.get())
            .with_node(move |node| {
                node.base_style(move || {
                    let t = kivact::style::use_tokens::<crate::Tokens>();
                    elevation.apply(
                        style()
                            .direction(direction)
                            .align_items(Align::Stretch)
                            .radius(t.radius.element),
                        &t,
                    )
                })
            })
            .child(children)
            .into_view()
    }
}

#[derive(Clone, Copy)]
struct MemberLayout {
    size: ControlSize,
    vertical: bool,
    rtl: bool,
    count: Signal<usize>,
    next: Signal<usize>,
}
impl IntoView for GroupMember {
    fn into_view(self) -> View {
        let Some(layout) =
            kivact::state::try_use_context::<MemberLayout>().map(|value| value.get())
        else {
            return self.button.into_view();
        };
        let index = layout.next.get_untracked();
        layout.next.set(index + 1);
        let solid = self.solid;
        let button = if self.keep_size {
            self.button
        } else {
            self.button.default_size(layout.size)
        };
        button
            .style(move || {
                let t = kivact::style::use_tokens::<crate::Tokens>();
                let first = index == 0;
                let last = index + 1 == layout.count.get();
                let [tl, tr, br, bl] =
                    corners(layout.vertical, layout.rtl, first, last, t.radius.element);
                let base = style()
                    .radius_corners(tl, tr, br, bl)
                    .pressed(style().scale(1));
                let divider = if solid {
                    t.colors.on_accent
                } else {
                    t.colors.border
                };
                match (first, layout.vertical, layout.rtl) {
                    (true, _, _) => base,
                    (false, true, _) => base.border_top(1, divider),
                    (false, false, true) => base.border_right(1, divider),
                    (false, false, false) => base.border_left(1, divider),
                }
            })
            .into_view()
    }
}
