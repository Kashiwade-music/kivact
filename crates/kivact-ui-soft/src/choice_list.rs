//! Role: Labelled rows of checkbox and radio options.

use crate::color as c;
use crate::radius as r;
use kivact::component;
use kivact::custom::Caller;
use kivact::state::ItemKey;
use kivact::state::Signal;
use kivact::view::Element;
use kivact::view::Node;
use kivact::view::column;
use kivact::view::node;
use kivact::view::row;
use kivact::view::text;

use kivact::interaction::ListItem;
use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::state::binding;
use kivact::state::memo;
use kivact::style::Align;
use kivact::style::style;
use kivact::ui::checkbox;
use kivact::ui::radio_group;
use kivact::view::Role;
use kivact::view::for_each;
use kivact::view::node_ref;
use kivact::view::show_some;

/// A labelled group of checkboxes bound to the selected item keys.
#[component]
pub fn checkbox_list<K: ItemKey>(
    label: Value<String>,
    items: Value<Vec<ListItem<K>>>,
    selected: Signal<Vec<K>>,
    #[caller] caller: Caller,
) -> impl Element {
    let disabled = caller.disabled();
    let root = node()
        .with_node(move |node| node.role(Role::Group))
        .name(label);
    let off = memo(move || disabled.as_ref().is_some_and(|disabled| disabled.get()));
    let rows = for_each(
        items,
        |item| item.key().clone(),
        move |item| {
            let key = item.with_untracked(|item| item.key().clone());
            let checked = binding(
                {
                    let key = key.clone();
                    move || selected.with(|checked| checked.contains(&key))
                },
                move |checked| {
                    let mut keys = selected.get_untracked();
                    if checked {
                        if keys.contains(&key) {
                            return;
                        }
                        keys.push(key.clone());
                    } else {
                        keys.retain(|candidate| candidate != &key);
                    }
                    selected.set(keys);
                },
            );
            let label = memo(move || item.get().label());
            let desc = memo(move || item.get().description());
            let disabled = memo(move || off.get() || item.get().is_disabled());
            checkbox_row(label, desc, checked, disabled)
        },
    );
    root.child(column(rows).style(style().gap(4)))
}

/// Build mutually exclusive option rows with the canonical radio-group behavior.
#[component]
pub fn radio_list<K: ItemKey>(
    label: Value<String>,
    items: Value<Vec<ListItem<K>>>,
    selected: Signal<Option<K>>,
) -> impl Element {
    radio_group(label, items, selected)
}

/// A labelled checkbox with supporting text and a row-sized hit area.
///
/// `description` is the supporting text under the label: the checkbox's description and
/// the text in the row, not the root.
#[component]
pub fn checkbox_list_item(
    label: Value<String>,
    description: Value<Option<String>>,
    checked: Signal<bool>,
) -> impl Element {
    checkbox_row(label, description, checked, false.into_value())
}

fn checkbox_row(
    label: Value<String>,
    description: Value<Option<String>>,
    checked: Signal<bool>,
    disabled: Value<bool>,
) -> Node {
    let control_ref = node_ref();
    let details = show_some(description, |description| {
        text(description).style(move || style().font_size(12).color(c::TEXT_SECONDARY))
    });
    let control = checkbox(label, checked)
        .hide_label(true)
        .disabled(disabled)
        .node_ref(control_ref)
        .description(move || description.get().unwrap_or_default());
    row((control, column((text(label), details))))
        .with_node(move |node| node.role(Role::Generic))
        .on_press(move || {
            if !disabled.get_untracked() {
                control_ref.activate();
            }
        })
        .style(move || {
            style()
                .align_items(Align::Center)
                .min_height(32)
                .padding_x(8)
                .padding_y(6)
                .gap(8)
                .border_radius(r::RADIUS_ELEMENT)
                .hover(style().background(c::OVERLAY_HOVER))
        })
}
