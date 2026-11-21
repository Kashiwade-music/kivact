//! Role: Group arbitrary action controls into one native roving focus scope.
pub(crate) mod look;

use crate::button_group::look::{ButtonGroupMemberState, ButtonGroupState, ButtonGroupTheme};
use crate::control_size::{ControlSize, default_control_size};
use crate::layer::with_base_recipe_style;
use kivact_core::custom::Caller;
use kivact_core::state::{IntoValue, Value, memo, provide, try_use_context};
use kivact_core::view::{
    Children, Node, Orientation, ReadingDirection, Role, Variants, node, row, text, use_direction,
};
use kivact_macros::component;

#[derive(Clone, Copy)]
struct GroupAxis(Value<Orientation>);

#[derive(Clone, Copy)]
pub(crate) struct GroupPosition(pub(crate) Value<ButtonGroupMemberState>);

/// Join controls under an accessible name. Arrows along the axis move focus;
/// Home and End reach the first and last eligible controls. Non-control text
/// and separators do not participate in the keyboard sequence.
#[component]
pub fn button_group(
    label: Value<String>,
    children: Children,
    orientation: Option<Value<Orientation>>,
    size: Option<Value<ControlSize>>,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    let orientation = orientation.unwrap_or_else(|| Orientation::Horizontal.into_value());
    let size = size.unwrap_or_else(|| default_control_size().into_value());
    let direction = use_direction();
    let members = variants.clone();
    let children = provide(GroupAxis(orientation), move || {
        crate::provide_control_size(size.get(), move || {
            let children = children.into_views();
            let count = children.len();
            children
                .into_iter()
                .enumerate()
                .map(move |(index, child)| {
                    let state = memo(move || ButtonGroupMemberState {
                        layout: ButtonGroupState {
                            orientation: orientation.get(),
                            size: size.get(),
                            rtl: direction == ReadingDirection::Rtl,
                        },
                        first: index == 0,
                        last: index + 1 == count,
                    });
                    with_base_recipe_style(
                        node(),
                        |r: &ButtonGroupTheme| r.member_recipe(),
                        &members,
                        move || state.get(),
                    )
                    .child(provide(GroupPosition(state), move || child))
                })
                .collect::<Vec<_>>()
        })
    });
    with_base_recipe_style(
        node()
            .role(Role::Group)
            .name(label)
            .orientation(orientation)
            .roving_focus(orientation),
        |r: &ButtonGroupTheme| r.root_recipe(),
        &variants,
        move || ButtonGroupState {
            orientation: orientation.get(),
            size: size.get(),
            rtl: direction == ReadingDirection::Rtl,
        },
    )
    .child(children)
}

/// A non-focusable line perpendicular to the group's axis.
#[component]
pub fn button_group_separator() -> impl Element {
    cell(node().role(Role::Separator), true)
}

/// A non-focusable prefix or suffix cell in a button group.
#[component]
pub fn button_group_text(label: Value<String>) -> impl Element {
    cell(row(text(label)), false)
}

/// The look of a non-focusable cell (a separator or a text) on the group's
/// axis, read from the group's context.
fn cell(root: Node, separator: bool) -> Node {
    let orientation = try_use_context::<GroupAxis>()
        .map_or_else(|| Orientation::Horizontal.into_value(), |axis| axis.0);
    let state = memo(move || ButtonGroupState {
        orientation: orientation.get(),
        size: default_control_size(),
        rtl: use_direction() == ReadingDirection::Rtl,
    });
    let root = if separator {
        root.orientation(move || {
            if orientation.get() == Orientation::Vertical {
                Orientation::Horizontal
            } else {
                Orientation::Vertical
            }
        })
    } else {
        root
    };
    with_base_recipe_style(
        root,
        if separator {
            |r: &ButtonGroupTheme| r.separator_recipe()
        } else {
            |r: &ButtonGroupTheme| r.text_recipe()
        },
        &Variants::default(),
        move || state.get(),
    )
}
