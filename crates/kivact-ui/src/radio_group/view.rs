//! Role: The radio group component and its option rendering over the keyed list parts.

use crate::control_size::{ControlSize, default_control_size};
use crate::layer::{with_base_recipe_style, with_recipe_style};
use crate::radio_group::look::RadioGroupTheme;
use crate::toggle::look::{CheckControl, CheckableState};
use kivact_core::custom::Caller;
use kivact_core::state::{Callback, IntoValue, Signal, Value, binding};
use kivact_core::style::use_theme;
use kivact_core::view::Variants;
use kivact_core::view::{
    EditPhase, IntoView, Node, Orientation, ReadingDirection, Role, View, for_each, icon, node,
    show, use_direction,
};
use kivact_interaction::{EditSession, ListItem, ListOptions, Selection, use_list};
use kivact_macros::component;
use std::rc::Rc;

/// What an option shows, from its item and its state.
type OptionFn<K> = Rc<dyn Fn(Value<ListItem<K>>, Value<CheckableState>) -> View>;

/// Adds modifiers to the complete canonical target of an option.
type ItemNode<K> = Rc<dyn Fn(Node, Value<ListItem<K>>, Value<CheckableState>) -> Node>;

/// A themed radio group named `label` over `items` that writes the
/// chosen key to `selected`.
///
/// Each option is a `Role::RadioButton` named by its label, showing the
/// theme's indicator (the `dot` icon while chosen), its icon and its label
/// over its description. The arrow keys move the focus and choose (from the
/// right in a right-to-left subtree), Tab enters at the chosen option, a
/// press or Space chooses, and a disabled option is skipped. Each choice is
/// one edit (`.on_edit(..)`, [`EditPhase`]). The label is the group's
/// accessible name; a group always has one, as the options alone do not say
/// what is being chosen.
///
/// `.item(..)` replaces what an option shows beside its indicator, built
/// once per option from its item and its state (both reactive). The option
/// itself, its keys, its semantics, its indicator and its recipes stay the
/// component's; a look without an indicator gives it no size. `.item_node(..)`
/// adds modifiers to the complete canonical target. `.size(..)` gives the
/// size (default `ControlSize::Medium`) to the item and indicator recipes in
/// `CheckableState::size`. `.orientation(..)` lays the options out in a
/// column (the default) or a row.
///
/// ```no_run
/// use kivact::prelude::*;
///
/// fn density() -> impl IntoView {
///     let value = signal(Some("comfortable"));
///     radio_group(
///         "Density",
///         vec![
///             ListItem::new("comfortable", "Comfortable"),
///             ListItem::new("compact", "Compact"),
///         ],
///         value,
///     )
/// }
/// ```
///
/// ```no_run
/// use kivact::prelude::*;
///
/// fn periods() -> impl IntoView {
///     let value = signal(Some("week"));
///     radio_group(
///         "Period",
///         vec![ListItem::new("week", "Week"), ListItem::new("month", "Month")],
///         value,
///     )
///     .item(|item, state| {
///         text(move || {
///             let mark = if state.get().is_checked() { "* " } else { "" };
///             format!("{mark}{}", item.get().label())
///         })
///     })
/// }
/// ```
#[component]
pub fn radio_group<K: kivact_core::state::ItemKey>(
    label: Value<String>,
    items: Value<Vec<ListItem<K>>>,
    selected: Signal<Option<K>>,
    #[prop(default = Orientation::Vertical)] orientation: Orientation,
    size: Option<Value<ControlSize>>,
    on_edit: Option<Callback<EditPhase>>,
    item: Option<Rc<dyn Fn(Value<ListItem<K>>, Value<CheckableState>) -> View>>,
    item_node: Option<Rc<dyn Fn(Node, Value<ListItem<K>>, Value<CheckableState>) -> Node>>,
    #[caller] caller: Caller,
) -> impl Element {
    let root = node().role(Role::RadioGroup).name(label);
    let variants = caller.variants();
    let invalid = caller.invalid();
    let size = size.unwrap_or_else(|| default_control_size().into_value());
    let edit = EditSession::new(move |initial: Option<K>| selected.set(initial));
    if let Some(on_edit) = on_edit {
        edit.on_edit(move |phase| on_edit.call(phase));
    }
    let chosen = binding(selected.into_value(), move |next: Option<K>| {
        edit.once(selected.get(), || selected.set(next));
    });
    let group = use_list(items, Selection::Single(chosen), ListOptions::radio());
    let looks = variants.clone();
    let slot = item;
    let modify = item_node;
    let rows = for_each(
        items,
        |item| item.key().clone(),
        move |item| {
            radio_option(
                group,
                item,
                size,
                invalid,
                &looks,
                slot.as_ref(),
                modify.as_ref(),
            )
        },
    );
    let reversed =
        orientation == Orientation::Horizontal && use_direction() == ReadingDirection::Rtl;
    // The recipe, then the row direction: base layers below the caller's styles.
    let root = with_base_recipe_style(
        root,
        |r: &RadioGroupTheme| r.group_recipe(),
        &variants,
        move || orientation,
    )
    .base_style(if reversed {
        kivact_core::style::style().flex_direction(kivact_core::style::FlexDirection::RowReverse)
    } else {
        kivact_core::style::style()
    });
    root.orientation(orientation).child(rows)
}

/// Render one radio option from its shared navigation and selection part.
fn radio_option<K: kivact_core::state::ItemKey>(
    group: kivact_interaction::ListApi<K>,
    item: Value<ListItem<K>>,
    size: Value<ControlSize>,
    invalid: Option<Value<bool>>,
    looks: &Variants,
    slot: Option<&OptionFn<K>>,
    modify: Option<&ItemNode<K>>,
) -> View {
    let key = item.with_untracked(|item| item.key().clone());
    let member = group.item(key);
    let state = member.state();
    let check = kivact_core::state::memo(move || {
        let state = state.get();
        CheckableState {
            input: state.input,
            control: CheckControl::Radio {
                selected: state.selected,
            },
            size: size.get(),
            invalid: invalid.is_some_and(|invalid| invalid.get()),
        }
    });
    let lines = looks.clone();
    let mark_looks = looks.clone();
    let indicator = with_recipe_style(
        node(),
        |r: &RadioGroupTheme| r.indicator_recipe(),
        looks,
        move || check.get(),
    )
    .child(show(
        move || {
            use_theme().get().ext::<RadioGroupTheme>().has_custom_mark() || check.get().is_checked()
        },
        move || {
            with_recipe_style(
                icon("dot"),
                |r: &RadioGroupTheme| r.mark_recipe(),
                &mark_looks,
                move || check.get(),
            )
        },
    ));
    let content = match slot {
        Some(slot) => slot(item, check),
        None => (
            crate::helpers::item_icon(move || item.get().icon().cloned()),
            crate::helpers::label_and_description(
                move || item.get().label(),
                move || item.get().description(),
                move |line| {
                    with_recipe_style(
                        line,
                        |r: &RadioGroupTheme| r.description_recipe(),
                        &lines,
                        move || check.get(),
                    )
                },
            ),
        )
            .into_view(),
    };
    let target = with_recipe_style(
        node()
            .part(member)
            .role(Role::RadioButton)
            .checked(move || state.get().selected),
        |r: &RadioGroupTheme| r.item_recipe(),
        looks,
        move || check.get(),
    )
    .child(indicator)
    .child(content);
    match modify {
        Some(modify) => modify(target, item, check).into_view(),
        None => target.into_view(),
    }
}
