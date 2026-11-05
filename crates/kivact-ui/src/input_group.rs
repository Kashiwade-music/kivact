//! Role: Group labelled controls and their inline or block adornments.
use super::layer::{with_base_recipe_style, with_recipe_style};
use super::variants::{Variant, Variants};
use super::{ControlSize, provide_control_size};
use kivact_core::state::{IntoText, IntoValue, Value};
use kivact_core::view::{
    IntoChildren, IntoView, Node, NodeBuilder, Role, View, column, node, row, text,
};

/// Input group size passed to all of its recipes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InputGroupState {
    pub size: ControlSize,
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
    let mut group = input_group(label, ());
    let root = group.root;
    InputGroup {
        root,
        label: group.label,
        controls: Controls::Scoped(Box::new(init)),
        size: group.size,
        start: group.start.take(),
        end: group.end.take(),
        block_start: group.block_start.take(),
        block_end: group.block_end.take(),
        variants: group.variants,
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
        let state = move || InputGroupState { size: size.get() };
        let controls = match self.controls {
            Controls::Built(controls) => row(controls.into_views()).into_view(),
            Controls::Scoped(init) => {
                provide_control_size(size.get(), move || row(init().into_views()))
            }
        };
        let members = with_recipe_style(
            row((self.start, controls, self.end)),
            |r| r.input_group_recipes().members_recipe(),
            &self.variants,
            state,
        );
        let label = with_recipe_style(
            text(self.label),
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
        .child(column((label, self.block_start, members, self.block_end)))
        .into_view()
    }
}
/// Add noninteractive text adjacent to a grouped input.
pub fn input_addon(label: impl IntoText) -> impl kivact_core::view::Component {
    let size = super::values::default_control_size();
    with_recipe_style(
        node().child(text(label)),
        |r| r.input_group_recipes().addon_recipe(),
        &Variants::default(),
        move || InputGroupState { size },
    )
}
