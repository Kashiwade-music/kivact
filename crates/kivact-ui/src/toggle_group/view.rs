//! Role: The toggle group component: toggles over a keyed selection.

use crate::control_size::{ControlSize, default_control_size};
use crate::layer::with_base_recipe_style;
use crate::toggle_button::look::ToggleButtonState;
use crate::toggle_group::look::{ToggleGroupItemState, ToggleGroupState, ToggleGroupTheme};
use kivact_core::custom::Caller;
use kivact_core::state::{Callback, IntoValue, ItemKey, Signal, Value, binding, memo};
use kivact_core::view::{
    EditPhase, IntoView, Node, Orientation, ReadingDirection, Role, View, for_each, node,
    use_direction,
};
use kivact_interaction::{IntoSelection, ListItem, Selection, ToggleState, use_toggle_button};
use kivact_macros::component;
use std::rc::Rc;

/// The view of one item of a group, given the item and its toggle state.
type ToggleItemSlot<K> = Rc<dyn Fn(Value<ListItem<K>>, Value<ToggleButtonState>) -> View>;

/// The shared inputs of every item of one group.
struct ItemCtx<K: ItemKey> {
    selection: Selection<K>,
    options: Value<Vec<ListItem<K>>>,
    orientation: Value<Orientation>,
    size: Value<ControlSize>,
    spacing: Option<Value<f64>>,
    icon_only: Value<bool>,
    disabled: Option<Value<bool>>,
    on_edit: Option<Callback<EditPhase>>,
    slot: Option<ToggleItemSlot<K>>,
    variants: kivact_core::view::Variants,
    rtl: bool,
}

/// A named group whose toggles edit `selected`. `Option<K>` allows zero or
/// one pressed item; `Vec<K>` allows many. Pressing a selected item releases
/// it. Arrows only move focus; Space, Enter, click and accessibility activation
/// toggle the focused item. Missing keys remain in the caller's selection.
///
/// `item` replaces the content of each item, keeping its behavior and name.
#[component]
pub fn toggle_group<K: ItemKey, S: IntoSelection<K>>(
    label: Value<String>,
    items: Value<Vec<ListItem<K>>>,
    selected: S,
    orientation: Option<Value<Orientation>>,
    size: Option<Value<ControlSize>>,
    spacing: Option<Value<f64>>,
    icon_only: Option<Value<bool>>,
    on_edit: Option<Callback<EditPhase>>,
    item: Option<Rc<dyn Fn(Value<ListItem<K>>, Value<ToggleButtonState>) -> View>>,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    let orientation = orientation.unwrap_or_else(|| Orientation::Horizontal.into_value());
    let size = size.unwrap_or_else(|| default_control_size().into_value());
    let icon_only = icon_only.unwrap_or_else(|| false.into_value());
    let direction = use_direction();
    let ctx = ItemCtx {
        selection: selected.into_selection(),
        options: items,
        orientation,
        size,
        spacing,
        icon_only,
        disabled: caller.disabled(),
        on_edit,
        slot: item,
        variants: variants.clone(),
        rtl: direction == ReadingDirection::Rtl,
    };
    let rows = for_each(
        items,
        |item| item.key().clone(),
        move |item| toggle_item(&ctx, item),
    );
    with_base_recipe_style(
        node()
            .role(Role::Group)
            .name(label)
            .orientation(orientation)
            .roving_focus(orientation),
        |r: &ToggleGroupTheme| r.root_recipe(),
        &variants,
        move || ToggleGroupState {
            orientation: orientation.get(),
            size: size.get(),
            spacing: spacing.map(|spacing| spacing.get().max(0.0)),
            rtl: direction == ReadingDirection::Rtl,
        },
    )
    .child(rows)
}

fn member<K: ItemKey>(selection: Selection<K>, key: K) -> Signal<bool> {
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

/// One toggle item of the group: its behavior, its state and its content.
fn toggle_item<K: ItemKey>(ctx: &ItemCtx<K>, item: Value<ListItem<K>>) -> Node {
    let key = item.with_untracked(|item| item.key().clone());
    let at = key.clone();
    let pressed = member(ctx.selection, key);
    let disabled = ctx.disabled;
    let mut behavior = use_toggle_button(pressed).disabled(move || {
        disabled.is_some_and(|disabled| disabled.get()) || item.get().is_disabled()
    });
    if let Some(observer) = ctx.on_edit {
        behavior = behavior.on_edit(move |phase| observer.call(phase));
    }
    // Only an item with an icon loses its label in an icon-only group;
    // an iconless item keeps its label, so it is not drawn as a square.
    let (icon_only, size) = (ctx.icon_only, ctx.size);
    let hide = memo(move || icon_only.get() && item.get().icon().is_some());
    let part = behavior
        .control()
        .map_state(move |state: ToggleState| ToggleButtonState {
            input: state.input,
            pressed: state.checked.is_checked(),
            size: size.get(),
            leading_icon: item.get().icon().is_some() || item.get().pressed_icon().is_some(),
            label_hidden: hide.get(),
        });
    let state = part.state();
    let content = item_content(ctx.slot.as_ref(), item, state, pressed, hide);
    let (orientation, spacing, options) = (ctx.orientation, ctx.spacing, ctx.options);
    let rtl = ctx.rtl;
    let variants = ctx.variants.clone();
    with_base_recipe_style(
        node().name(move || item.get().label()),
        |r: &ToggleGroupTheme| r.item_recipe(),
        &variants,
        move || ToggleGroupItemState {
            toggle: state.get(),
            layout: ToggleGroupState {
                orientation: orientation.get(),
                size: size.get(),
                spacing: spacing.map(|spacing| spacing.get().max(0.0)),
                rtl,
            },
            first: options.with(|options| options.first().is_some_and(|item| item.key() == &at)),
            last: options.with(|options| options.last().is_some_and(|item| item.key() == &at)),
        },
    )
    .part(part)
    .child(content)
}

fn item_content<K: ItemKey>(
    slot: Option<&ToggleItemSlot<K>>,
    item: Value<ListItem<K>>,
    state: Value<ToggleButtonState>,
    pressed: Signal<bool>,
    hide: Value<bool>,
) -> View {
    match slot {
        Some(slot) => slot(item, state),
        None => (
            crate::helpers::item_icon(move || {
                let item = item.get();
                if pressed.get() {
                    item.pressed_icon().or_else(|| item.icon()).cloned()
                } else {
                    item.icon().cloned()
                }
            }),
            crate::helpers::visible_label((move || item.get().label()).into_value(), Some(hide)),
        )
            .into_view(),
    }
}
