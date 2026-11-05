//! Role: Compose keyed toggle buttons over the caller's canonical selection.
use crate::group_types::{ToggleGroupItemState, ToggleGroupState};
use crate::layer::with_base_recipe_style;
use crate::values::{ControlSize, ToggleButtonState};
use crate::variants::{Variant, Variants};
use kivact_core::state::{
    Binding, Callback, IntoText, IntoValue, ItemKey, Value, binding, callback,
};
use kivact_core::view::{
    EditPhase, HandlerResult, IntoView, Node, NodeBuilder, Orientation, ReadingDirection, Role,
    View, ViewFn, for_each, node, use_direction,
};
use kivact_interaction::{IntoSelection, ListItem, Selection};

/// Single or multiple toggles with native roving focus, created by [`toggle_group`].
pub struct ToggleGroup<K: ItemKey> {
    root: Node,
    options: Value<Vec<ListItem<K>>>,
    selection: Selection<K>,
    orientation: Value<Orientation>,
    size: Value<ControlSize>,
    // xtask-allow(state-fields): Independent presentation or construction input, not interaction state.
    spacing: Option<Value<f64>>,
    // xtask-allow(state-fields): Independent presentation or construction input, not interaction state.
    icon_only: Value<bool>,
    // xtask-allow(state-fields): Independent presentation or construction input, not interaction state.
    on_edit: Option<Callback<EditPhase>>,
    // xtask-allow(state-fields): Independent presentation or construction input, not interaction state.
    item: Option<ViewFn<(Value<ListItem<K>>, Value<ToggleButtonState>)>>,
    variants: Variants,
}

/// A named group whose toggles edit `selection`. `Option<K>` allows zero or
/// one pressed item; `Vec<K>` allows many. Pressing a selected item releases
/// it. Arrows only move focus; Space, Enter, click and accessibility activation
/// toggle the focused item. Missing keys remain in the caller's selection.
#[track_caller]
pub fn toggle_group<K: ItemKey>(
    label: impl IntoText,
    options: impl IntoValue<Vec<ListItem<K>>>,
    selection: impl IntoSelection<K>,
) -> ToggleGroup<K> {
    ToggleGroup {
        root: node().role(Role::Group).name(label),
        options: options.into_value(),
        selection: selection.into_selection(),
        orientation: Orientation::Horizontal.into_value(),
        size: crate::values::default_control_size().into_value(),
        spacing: None,
        icon_only: false.into_value(),
        on_edit: None,
        item: None,
        variants: Variants::default(),
    }
}
impl<K: ItemKey> ToggleGroup<K> {
    /// The constant or reactive layout and keyboard navigation axis.
    pub fn orientation(mut self, orientation: impl IntoValue<Orientation>) -> Self {
        self.orientation = orientation.into_value();
        self
    }
    /// Size of all toggle items.
    pub fn size(mut self, size: impl IntoValue<ControlSize>) -> Self {
        self.size = size.into_value();
        self
    }
    /// Space in four-pixel units; zero joins the controls.
    pub fn spacing(mut self, spacing: impl IntoValue<f64>) -> Self {
        self.spacing = Some(spacing.into_value());
        self
    }
    /// Hide visible labels while retaining accessible names.
    pub fn icon_only(mut self, icon_only: impl IntoValue<bool>) -> Self {
        self.icon_only = icon_only.into_value();
        self
    }
    /// Observe each item's edit phases.
    pub fn on_edit<R: HandlerResult>(
        mut self,
        observer: impl Fn(EditPhase) -> R + 'static,
    ) -> Self {
        self.on_edit = Some(callback(observer));
        self
    }
    /// Replace the content of each item, retaining its behavior and name.
    pub fn item<V: IntoView>(
        mut self,
        content: impl Fn(Value<ListItem<K>>, Value<ToggleButtonState>) -> V + 'static,
    ) -> Self {
        self.item = Some(ViewFn::from(move |(item, state)| content(item, state)));
        self
    }
    /// Add a theme presentation variant shared by the group and its items.
    pub fn variant(self, variant: impl Variant) -> Self {
        self.variants.add(variant);
        self
    }
}
impl<K: ItemKey> NodeBuilder for ToggleGroup<K> {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

fn member<K: ItemKey>(selection: Selection<K>, key: K) -> Binding<bool> {
    let read = key.clone();
    match selection {
        Selection::Single(value) => binding(
            move || value.get().as_ref() == Some(&read),
            move |on| {
                if on {
                    value.set(Some(key.clone()));
                } else if value.get().as_ref() == Some(&key) {
                    value.set(None);
                }
            },
        ),
        Selection::Multiple(value) => binding(
            move || value.get().contains(&read),
            move |on| {
                let mut next = value.get();
                if on {
                    if !next.contains(&key) {
                        next.push(key.clone());
                    }
                } else {
                    next.retain(|item| item != &key);
                }
                value.set(next);
            },
        ),
    }
}
impl<K: ItemKey> IntoView for ToggleGroup<K> {
    fn into_view(mut self) -> View {
        let orientation = self.orientation;
        let size = self.size;
        let spacing = self.spacing;
        let icon_only = self.icon_only;
        let selection = self.selection;
        let options = self.options;
        let direction = use_direction();
        let slot = self.item;
        let on_edit = self.on_edit;
        let disabled = self.root.accessible_disabled();
        let variants = self.variants.clone();
        let rows = for_each(
            self.options,
            |item| item.key().clone(),
            move |item| {
                let key = item.with_untracked(|item| item.key().clone());
                let at = key.clone();
                let pressed = member(selection, key);
                let mut behavior =
                    kivact_interaction::use_toggle_button(pressed).disabled(move || {
                        disabled.is_some_and(|disabled| disabled.get()) || item.get().is_disabled()
                    });
                if let Some(observer) = on_edit {
                    behavior = behavior.on_edit(move |phase| observer.call(phase));
                }
                let part =
                    behavior
                        .control()
                        .map_state(move |state: kivact_interaction::ToggleState| {
                            ToggleButtonState {
                                input: state.input,
                                pressed: state.checked.is_checked(),
                                size: size.get(),
                                leading_icon: item.get().icon().is_some()
                                    || item.get().pressed_icon().is_some(),
                                icon_only: icon_only.get(),
                            }
                        });
                let state = part.state();
                let content = match &slot {
                    Some(slot) => slot.call((item, state)),
                    None => (
                        crate::helpers::item_icon(move || {
                            let item = item.get();
                            if pressed.get() {
                                item.pressed_icon().or_else(|| item.icon()).cloned()
                            } else {
                                item.icon().cloned()
                            }
                        }),
                        crate::helpers::visible_label(
                            (move || item.get().label()).into_value(),
                            Some(icon_only),
                        ),
                    )
                        .into_view(),
                };
                with_base_recipe_style(
                    node().name(move || item.get().label()),
                    |r| r.toggle_group_recipes().item_recipe(),
                    &variants,
                    move || ToggleGroupItemState {
                        toggle: state.get(),
                        layout: ToggleGroupState {
                            orientation: orientation.get(),
                            size: size.get(),
                            spacing: spacing.map(|spacing| spacing.get().max(0.0)),
                            rtl: direction.get() == ReadingDirection::Rtl,
                        },
                        first: options
                            .with(|options| options.first().is_some_and(|item| item.key() == &at)),
                        last: options
                            .with(|options| options.last().is_some_and(|item| item.key() == &at)),
                    },
                )
                .part(part)
                .child(content)
            },
        );
        let direction = use_direction();
        with_base_recipe_style(
            self.root.orientation(orientation).roving_focus(orientation),
            |r| r.toggle_group_recipes().root_recipe(),
            &self.variants,
            move || ToggleGroupState {
                orientation: orientation.get(),
                size: size.get(),
                spacing: spacing.map(|spacing| spacing.get().max(0.0)),
                rtl: direction.get() == ReadingDirection::Rtl,
            },
        )
        .child(rows)
        .into_view()
    }
}
