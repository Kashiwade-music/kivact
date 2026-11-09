//! Role: Group labelled controls and their inline or block adornments.
use super::layer::{with_base_recipe_style, with_recipe_style};
use super::variants::{Variant, Variants};
use super::{ControlSize, provide_control_size};
use kivact_core::state::{IntoText, IntoValue, Value};
use kivact_core::view::{
    IntoChildren, IntoView, Node, NodeBuilder, Role, View, node, node_ref, row, text, use_direction,
};

use crate::component_values::{InputGroupMemberKind, InputGroupMemberState, InputGroupState};
enum Controls<C> {
    Built(C),
    Scoped(Box<dyn FnOnce() -> C>),
}
/// A labelled group of adjacent inputs and adornments.
pub struct InputGroup<C> {
    root: Node,
    label: Value<String>,
    controls: Controls<C>,
    size: Value<ControlSize>,
    // xtask-allow(state-fields): builder option - start
    start: Option<View>,
    // xtask-allow(state-fields): builder option - end
    end: Option<View>,
    // xtask-allow(state-fields): builder option - block start
    block_start: Option<View>,
    // xtask-allow(state-fields): builder option - block end
    block_end: Option<View>,
    variants: Variants,
}
/// Group eagerly built controls under a visible label.
pub fn input_group<C: IntoChildren>(label: impl IntoText, children: C) -> InputGroup<C> {
    InputGroup {
        root: node(),
        label: label.into_text_value(),
        controls: Controls::Built(children),
        size: ControlSize::Medium.into_value(),
        start: None,
        end: None,
        block_start: None,
        block_end: None,
        variants: Variants::default(),
    }
}
/// Construct controls once inside the group's inherited control-size scope.
pub fn input_group_with<C: IntoChildren>(
    label: impl IntoText,
    init: impl FnOnce() -> C + 'static,
) -> InputGroup<C> {
    InputGroup {
        root: node(),
        label: label.into_text_value(),
        controls: Controls::Scoped(Box::new(init)),
        size: ControlSize::Medium.into_value(),
        start: None,
        end: None,
        block_start: None,
        block_end: None,
        variants: Variants::default(),
    }
}
impl<C> InputGroup<C> {
    /// Set inherited control sizing for scoped initializers.
    pub fn size(mut self, value: impl IntoValue<ControlSize>) -> Self {
        self.size = value.into_value();
        self
    }
    /// Set the leading inline adornment.
    pub fn start(mut self, value: impl IntoView) -> Self {
        self.start = Some(value.into_view());
        self
    }
    /// Set the trailing inline adornment.
    pub fn end(mut self, value: impl IntoView) -> Self {
        self.end = Some(value.into_view());
        self
    }
    /// Set the adornment above the controls.
    pub fn block_start(mut self, value: impl IntoView) -> Self {
        self.block_start = Some(value.into_view());
        self
    }
    /// Set the adornment below the controls.
    pub fn block_end(mut self, value: impl IntoView) -> Self {
        self.block_end = Some(value.into_view());
        self
    }
    /// Add a theme-defined appearance.
    pub fn variant(mut self, value: impl Variant) -> Self {
        self.variants.add(value);
        self
    }
}
impl<C> NodeBuilder for InputGroup<C> {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl<C: IntoChildren + 'static> IntoView for InputGroup<C> {
    fn into_view(mut self) -> View {
        let size = self.size;
        let direction = use_direction();
        let reference = node_ref();
        let focused = reference.focus_within();
        let invalid = self.root.accessible_invalid();
        let part = kivact_core::custom::part(|input| input);
        let input = part.state();
        let (has_start, has_end) = (self.start.is_some(), self.end.is_some());
        let (has_block_start, has_block_end) =
            (self.block_start.is_some(), self.block_end.is_some());
        let state = kivact_core::state::memo(move || InputGroupState {
            size: size.get(),
            direction: direction.get(),
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
            variants: self.variants.clone(),
            primary: node_ref(),
        };
        let inline = render.clone();
        let members = move |controls: C| {
            crate::input_group_context::provide_state(state, move || {
                inline.members(self.start, controls.into_views(), self.end)
            })
        };
        let members = match self.controls {
            Controls::Built(controls) => members(controls),
            Controls::Scoped(init) => provide_control_size(size.get(), move || members(init())),
        };
        let label_ref = node_ref();
        let label = with_recipe_style(
            text(self.label).node_ref(label_ref),
            |r| r.input_group_recipes().label_recipe(),
            &self.variants,
            move || state.get(),
        );
        let top = self.block_start.map(|view| render.block(view, true));
        let bottom = self.block_end.map(|view| render.block(view, false));
        with_base_recipe_style(
            self.root.node_ref(reference).part(part),
            |r| r.input_group_recipes().root_recipe(),
            &self.variants,
            move || state.get(),
        )
        .role(Role::Group)
        .name(self.label)
        .with_node(move |node| node.labelled_by(label_ref))
        .child((label, top, members, bottom))
        .into_view()
    }
}
#[derive(Clone)]
struct GroupRender {
    state: Value<InputGroupState>,
    variants: Variants,
    primary: kivact_core::view::NodeRef,
}
impl GroupRender {
    fn members(self, start: Option<View>, controls: Vec<View>, end: Option<View>) -> View {
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
            |r| r.input_group_recipes().members_recipe(),
            &self.variants,
            move || state.get(),
        )
        .into_view()
    }
    fn member(&self, view: View, kind: InputGroupMemberKind, index: usize, count: usize) -> Node {
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
            |r| r.input_group_recipes().member_recipe(),
            &self.variants,
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
        let state = self.state;
        with_recipe_style(
            adornment(view, self.primary),
            if top {
                |r| r.input_group_recipes().block_start_recipe()
            } else {
                |r| r.input_group_recipes().block_end_recipe()
            },
            &self.variants,
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
pub fn input_addon(label: impl IntoText) -> impl kivact_core::view::Component {
    let size = super::values::default_control_size();
    let direction = use_direction();
    with_recipe_style(
        node().child(text(label)),
        |r| r.input_group_recipes().addon_recipe(),
        &Variants::default(),
        move || InputGroupState {
            size,
            direction: direction.get(),
            ..InputGroupState::default()
        },
    )
}
