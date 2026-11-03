//! Role: Joined Neutral action buttons with roving keyboard focus.

use std::rc::Rc;

use crate::{Elevation, tokens};
use kivact::control::{Node, NodeBuilder, node};
use kivact::primitive::NodeExt;
use kivact::style::{Align, Direction, style};
use kivact::ui::{Button, ControlSize};
use kivact::view::{
    Handled, IntoText, IntoValue, IntoView, Key, NodeRef, Orientation, PointerPhase,
    ReadingDirection, Role, View, current_direction, node_ref,
};
use kivact::{Signal, Value, signal};

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

/// Static ordered group membership, accepted as a vector or tuple.
pub trait GroupMembers {
    fn into_members(self) -> Vec<GroupMember>;
}

impl<B: Into<GroupMember>> GroupMembers for Vec<B> {
    fn into_members(self) -> Vec<GroupMember> {
        self.into_iter().map(Into::into).collect()
    }
}

macro_rules! tuple_members {
    ($($ty:ident $value:ident),+) => {
        impl<$($ty: Into<GroupMember>),+> GroupMembers for ($($ty,)+) {
            fn into_members(self) -> Vec<GroupMember> {
                let ($($value,)+) = self;
                vec![$($value.into()),+]
            }
        }
    };
}

tuple_members!(A a);
tuple_members!(A a, B b);
tuple_members!(A a, B b, C c);
tuple_members!(A a, B b, C c, D d);
tuple_members!(A a, B b, C c, D d, E e);
tuple_members!(A a, B b, C c, D d, E e, F f);
tuple_members!(A a, B b, C c, D d, E e, F f, G g);
tuple_members!(A a, B b, C c, D d, E e, F f, G g, H h);

/// Joined controls with a shared name, size, elevation, and Tab stop.
pub struct ButtonGroup {
    root: Node,
    members: Vec<GroupMember>,
    orientation: Orientation,
    size: ControlSize,
    elevation: Elevation,
    disabled: Value<bool>,
}

#[track_caller]
pub fn button_group(label: impl IntoText, children: impl GroupMembers) -> ButtonGroup {
    ButtonGroup {
        root: node().role(Role::Group).name(label),
        members: children.into_members(),
        orientation: Orientation::Horizontal,
        size: ControlSize::Medium,
        elevation: Elevation::None,
        disabled: false.into_value(),
    }
}

impl ButtonGroup {
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    pub fn elevation(mut self, elevation: Elevation) -> Self {
        self.elevation = elevation;
        self
    }

    pub fn disabled(mut self, disabled: impl IntoValue<bool>) -> Self {
        self.disabled = disabled.into_value();
        self
    }
}

impl NodeBuilder for ButtonGroup {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct RovingItem {
    pub reference: NodeRef,
    pub disabled: Value<bool>,
}

pub(crate) fn tab_stop(items: &[RovingItem], active: usize) -> Option<usize> {
    if items.get(active).is_some_and(|item| !item.disabled.get()) {
        Some(active)
    } else {
        items.iter().position(|item| !item.disabled.get())
    }
}

pub(crate) fn move_focus(
    items: &[RovingItem],
    active: Signal<usize>,
    key: Key,
    vertical: bool,
    rtl: bool,
) -> Handled {
    let enabled: Vec<_> = items
        .iter()
        .enumerate()
        .filter(|(_, item)| !item.disabled.get())
        .collect();
    let Some(first) = enabled.first() else {
        return Handled::No;
    };
    let Some(last) = enabled.last() else {
        return Handled::No;
    };
    let step = match key {
        Key::Home => Some(first.0),
        Key::End => Some(last.0),
        Key::ArrowDown if vertical => Some(next(&enabled, active.get(), true)),
        Key::ArrowUp if vertical => Some(next(&enabled, active.get(), false)),
        Key::ArrowRight if !vertical => Some(next(&enabled, active.get(), !rtl)),
        Key::ArrowLeft if !vertical => Some(next(&enabled, active.get(), rtl)),
        _ => None,
    };
    if let Some(step) = step {
        active.set(step);
        items[step].reference.focus();
        Handled::Yes
    } else {
        Handled::No
    }
}

fn next(enabled: &[(usize, &RovingItem)], active: usize, forward: bool) -> usize {
    let index = enabled
        .iter()
        .position(|(index, _)| *index == active)
        .unwrap_or(0);
    let count = enabled.len();
    enabled[(index + if forward { 1 } else { count - 1 }) % count].0
}

pub(crate) fn roving_item<T: NodeBuilder>(
    item: T,
    index: usize,
    active: Signal<usize>,
    items: Rc<Vec<RovingItem>>,
) -> T {
    let reference = items[index].reference;
    let disabled = items[index].disabled;
    item.node_ref(reference)
        .focusable(move || tab_stop(&items, active.get()) == Some(index))
        .on_pointer(move |event| {
            if matches!(event.phase, PointerPhase::Down(_)) && !disabled.get() {
                active.set(index);
            }
        })
}

fn corners(vertical: bool, rtl: bool, first: bool, last: bool, r: f32) -> [f32; 4] {
    let (first, last) = (if first { r } else { 0.0 }, if last { r } else { 0.0 });
    match (vertical, rtl) {
        (true, _) => [first, first, last, last],
        (false, true) => [last, first, first, last],
        (false, false) => [first, last, last, first],
    }
}

impl IntoView for ButtonGroup {
    fn into_view(self) -> View {
        let active = signal(0usize);
        let vertical = self.orientation == Orientation::Vertical;
        let rtl = current_direction() == ReadingDirection::Rtl;
        let direction = match (vertical, rtl) {
            (true, _) => Direction::Column,
            (false, true) => Direction::RowReverse,
            _ => Direction::Row,
        };
        let disabled = self.disabled;
        let size = self.size;
        let mut members = self.members;
        let items: Rc<Vec<_>> = Rc::new(
            members
                .iter_mut()
                .map(|member| {
                    let disabled_by_caller = member.button.accessible_disabled();
                    RovingItem {
                        reference: node_ref(),
                        disabled: (move || {
                            disabled.get()
                                || disabled_by_caller
                                    .as_ref()
                                    .is_some_and(|is_disabled| is_disabled())
                        })
                        .into_value(),
                    }
                })
                .collect(),
        );
        let children = members
            .into_iter()
            .enumerate()
            .map(|(index, member)| {
                let first = index == 0;
                let last = index + 1 == items.len();
                let solid = member.solid;
                let button = if member.keep_size {
                    member.button
                } else {
                    member.button.size(size)
                };
                roving_item(
                    button.disabled(items[index].disabled),
                    index,
                    active,
                    items.clone(),
                )
                .style(move || {
                    let t = tokens();
                    let [tl, tr, br, bl] = corners(vertical, rtl, first, last, t.radius.element);
                    let base = style()
                        .radius_corners(tl, tr, br, bl)
                        .pressed(style().scale(1));
                    let divider = if solid {
                        t.colors.on_accent
                    } else {
                        t.colors.border
                    };
                    match (first, vertical, rtl) {
                        (true, _, _) => base,
                        (false, true, _) => base.border_top(1, divider),
                        (false, false, true) => base.border_right(1, divider),
                        (false, false, false) => base.border_left(1, divider),
                    }
                })
                .into_view()
            })
            .collect::<Vec<_>>();
        let elevation = self.elevation;
        self.root
            .disabled(disabled)
            .on_key(move |event| move_focus(&items, active, event.key, vertical, rtl))
            .base_style(move || {
                let t = tokens();
                elevation.apply(
                    style()
                        .direction(direction)
                        .align_items(Align::Stretch)
                        .radius(t.radius.element),
                    &t,
                )
            })
            .child(children)
            .into_view()
    }
}
