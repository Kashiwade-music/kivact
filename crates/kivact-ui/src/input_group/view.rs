//! Role: The input group component, its member rendering and the addon helper.

use crate::input_group::look::InputGroupTheme;
use crate::layer::{with_base_recipe_style, with_recipe_style};
use crate::{ControlSize, provide_control_size};
use kivact_core::custom::Caller;
use kivact_core::state::{IntoText, IntoValue, Value};
use kivact_core::view::Variants;
use kivact_core::view::{
    Children, Element, IntoView, Node, Role, View, node, node_ref, row, text, use_direction,
};
use kivact_macros::component;

use crate::input_group::look::InputGroupMemberKind;
use crate::input_group::look::InputGroupMemberState;
use crate::input_group::look::InputGroupState;

/// Group controls under a visible label. The controls are built by the
/// caller, and the group gives its size to them as the inherited control size.
///
/// Setters: `size` sets the inherited control size of the group's controls
/// (default medium); `start` and `end` are the leading and trailing inline
/// adornments; `block_start` and `block_end` are the adornments above and
/// below the controls.
#[component]
pub fn input_group(
    label: Value<String>,
    children: Children,
    size: Option<Value<ControlSize>>,
    start: Option<View>,
    end: Option<View>,
    block_start: Option<View>,
    block_end: Option<View>,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    let size = size.unwrap_or_else(|| ControlSize::Medium.into_value());
    let direction = use_direction();
    let reference = node_ref();
    let focused = reference.focus_within();
    let invalid = caller.invalid();
    let part = kivact_core::custom::part(|input| input);
    let input = part.state();
    let (has_start, has_end) = (start.is_some(), end.is_some());
    let (has_block_start, has_block_end) = (block_start.is_some(), block_end.is_some());
    let state = kivact_core::state::memo(move || InputGroupState {
        size: size.get(),
        direction,
        input: input.get(),
        focused: focused.get(),
        invalid: invalid.is_some_and(|invalid| invalid.get()),
        has_start,
        has_end,
        has_block_start,
        has_block_end,
    });
    let render = GroupRender {
        state,
        variants: variants.clone(),
        primary: node_ref(),
    };
    let inline = render.clone();
    let members = provide_control_size(size.get(), move || {
        crate::input_group::context::provide_state(state, move || {
            inline.members(start, children.into_views(), end)
        })
    });
    let label_ref = node_ref();
    let label_view = with_recipe_style(
        text(label).node_ref(label_ref),
        |r: &InputGroupTheme| r.label_recipe(),
        &variants,
        move || state.get(),
    );
    let top = block_start.map(|view| render.block(view, true));
    let bottom = block_end.map(|view| render.block(view, false));
    with_base_recipe_style(
        node().node_ref(reference).part(part),
        |r: &InputGroupTheme| r.root_recipe(),
        &variants,
        move || state.get(),
    )
    .role(Role::Group)
    .name(label)
    .with_node(move |node| node.labelled_by(label_ref))
    .child((label_view, top, members, bottom))
}

#[derive(Clone)]
struct GroupRender {
    state: Value<InputGroupState>,
    variants: Variants,
    primary: kivact_core::view::NodeRef,
}

impl GroupRender {
    fn members(self, start: Option<View>, controls: Vec<View>, end: Option<View>) -> View {
        let variants = self.variants.clone();
        let offset = usize::from(start.is_some());
        let count = offset + controls.len() + usize::from(end.is_some());
        let start = start.map(|view| self.member(view, InputGroupMemberKind::Start, 0, count));
        let controls = controls
            .into_iter()
            .enumerate()
            .map(|(index, view)| {
                self.member(view, InputGroupMemberKind::Control, index + offset, count)
            })
            .collect::<Vec<_>>();
        let controls = node()
            .node_ref(self.primary)
            .style(kivact_core::style::style().display(kivact_core::style::Display::Contents))
            .child(controls);
        let end = end.map(|view| self.member(view, InputGroupMemberKind::End, count - 1, count));
        let state = self.state;
        with_recipe_style(
            row((start, controls, end)),
            |r: &InputGroupTheme| r.members_recipe(),
            &variants,
            move || state.get(),
        )
        .into_view()
    }

    fn member(&self, view: View, kind: InputGroupMemberKind, index: usize, count: usize) -> Node {
        let variants = self.variants.clone();
        let reference = node_ref();
        let focused = reference.focus_within();
        let state = self.state;
        let root = if kind == InputGroupMemberKind::Control {
            node().child(view)
        } else {
            adornment(view, self.primary)
        };
        with_recipe_style(
            root.node_ref(reference),
            |r: &InputGroupTheme| r.member_recipe(),
            &variants,
            move || InputGroupMemberState {
                group: state.get(),
                kind,
                first: index == 0,
                last: index + 1 == count,
                focused: focused.get(),
            },
        )
    }

    fn block(&self, view: View, top: bool) -> Node {
        let variants = self.variants.clone();
        let state = self.state;
        with_recipe_style(
            adornment(view, self.primary),
            if top {
                |r: &InputGroupTheme| r.block_start_recipe()
            } else {
                |r: &InputGroupTheme| r.block_end_recipe()
            },
            &variants,
            move || state.get(),
        )
    }
}

fn adornment(view: View, primary: kivact_core::view::NodeRef) -> Node {
    let reference = node_ref();
    let focused = reference.focus_within();
    node()
        .role(Role::Generic)
        .node_ref(reference)
        .child(view)
        .on_press(move || {
            if !focused.get_untracked() {
                primary.activate();
            }
        })
}

/// Add noninteractive text adjacent to a grouped input.
pub fn input_addon(label: impl IntoText) -> impl kivact_core::view::Element {
    let size = crate::control_size::default_control_size();
    let direction = use_direction();
    with_recipe_style(
        node().child(text(label)),
        |r: &InputGroupTheme| r.addon_recipe(),
        &Variants::default(),
        move || InputGroupState {
            size,
            direction,
            ..InputGroupState::default()
        },
    )
}
