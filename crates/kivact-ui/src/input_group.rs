//! Role: Group labelled controls and their inline or block adornments.
use super::layer::{with_base_recipe_style, with_recipe_style};
use super::variants::{Variant, Variants};
use super::{ControlSize, provide_control_size};
use kivact_core::state::{IntoText, IntoValue, Value};
use kivact_core::view::{
    IntoChildren, IntoView, Node, NodeBuilder, ReadingDirection, Role, View, node, node_ref, row,
    text, use_direction,
};

/// Input group size and reading direction passed to its recipes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InputGroupState {
    pub size: ControlSize,
    pub direction: ReadingDirection,
}
// xtask-allow(state-fields): Read-only projection of member position and NodeRef focus-within, never independent behavior state.
/// A member's position, inherited size, and actual focus-within observation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InputGroupMemberState {
    pub group: InputGroupState,
    pub first: bool,
    pub last: bool,
    pub focused: bool,
}
enum Controls<C> {
    Built(C),
    Scoped(Box<dyn FnOnce() -> C>),
}
// xtask-allow(state-fields): Builder-only optional adornments and scoped initializer; no coupled runtime state.
/// A labelled group of adjacent inputs and adornments.
pub struct InputGroup<C> {
    root: Node,
    label: Value<String>,
    controls: Controls<C>,
    size: Value<ControlSize>,
    start: Option<View>,
    end: Option<View>,
    block_start: Option<View>,
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
    pub fn variant(self, value: impl Variant) -> Self {
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
    fn into_view(self) -> View {
        let size = self.size;
        let direction = use_direction();
        let state = move || InputGroupState {
            size: size.get(),
            direction: direction.get(),
        };
        let variants = self.variants.clone();
        let members = move |controls: C| {
            let views: Vec<_> = self
                .start
                .into_iter()
                .chain(controls.into_views())
                .chain(self.end)
                .collect();
            let count = views.len();
            let children = views
                .into_iter()
                .enumerate()
                .map(|(index, view)| {
                    let reference = node_ref();
                    let focused = reference.focus_within();
                    with_recipe_style(
                        node().node_ref(reference).child(view),
                        |r| r.input_group_recipes().member_recipe(),
                        &variants,
                        move || InputGroupMemberState {
                            group: state(),
                            first: index == 0,
                            last: index + 1 == count,
                            focused: focused.get(),
                        },
                    )
                })
                .collect::<Vec<_>>();
            with_recipe_style(
                row(children),
                |r| r.input_group_recipes().members_recipe(),
                &variants,
                state,
            )
        };
        let members = match self.controls {
            Controls::Built(controls) => members(controls).into_view(),
            Controls::Scoped(init) => provide_control_size(size.get(), move || members(init())),
        };
        let label_ref = node_ref();
        let label = with_recipe_style(
            text(self.label).node_ref(label_ref),
            |r| r.input_group_recipes().label_recipe(),
            &self.variants,
            state,
        );
        with_base_recipe_style(
            self.root,
            |r| r.input_group_recipes().root_recipe(),
            &self.variants,
            state,
        )
        .role(Role::Group)
        .name(self.label)
        .with_node(move |node| node.labelled_by(label_ref))
        .child((label, self.block_start, members, self.block_end))
        .into_view()
    }
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
        },
    )
}
