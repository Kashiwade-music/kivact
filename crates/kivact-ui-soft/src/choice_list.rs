//! Role: Labelled rows of checkbox and radio options.
use crate::theme_access::tokens;
use kivact::control::{Node, NodeBuilder, column, node, row, text};
use kivact::primitive::NodeExt;
use kivact::style::{Align, style};
use kivact::ui::{ListItem, RadioGroup, checkbox, radio_group};
use kivact::view::{IntoText, IntoValue, IntoView, Role, View, for_each, node_ref, show_some};
use kivact::{Binding, Value, binding, memo};
use std::hash::Hash;

pub struct CheckboxList<K: Clone + Eq + Hash + 'static> {
    root: Node,
    options: Value<Vec<ListItem<K>>>,
    value: Binding<Vec<K>>,
}

pub fn checkbox_list<K: Clone + Eq + Hash + 'static>(
    label: impl IntoText,
    options: impl IntoValue<Vec<ListItem<K>>>,
    value: impl Into<Binding<Vec<K>>>,
) -> CheckboxList<K> {
    CheckboxList {
        root: node().role(Role::Group).name(label),
        options: options.into_value(),
        value: value.into(),
    }
}

pub fn radio_list<K: Clone + Eq + Hash + 'static>(
    label: impl IntoText,
    options: impl IntoValue<Vec<ListItem<K>>>,
    value: impl Into<Binding<Option<K>>>,
) -> RadioGroup<K> {
    radio_group(label, options, value)
}

impl<K: Clone + Eq + Hash + 'static> NodeBuilder for CheckboxList<K> {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl<K: Clone + Eq + Hash + 'static> IntoView for CheckboxList<K> {
    fn into_view(self) -> View {
        let Self {
            mut root,
            options,
            value,
        } = self;
        let disabled = root.accessible_disabled();
        let off = memo(move || disabled.as_ref().is_some_and(|disabled| disabled()));
        let rows = for_each(
            options,
            |item| item.key().clone(),
            move |key, item| {
                let checked = binding(
                    {
                        let key = key.clone();
                        move || value.with(|checked| checked.contains(&key))
                    },
                    move |checked| {
                        let mut keys = value.get_untracked();
                        if checked {
                            if keys.contains(&key) {
                                return;
                            }
                            keys.push(key.clone());
                        } else {
                            keys.retain(|candidate| candidate != &key);
                        }
                        value.set(keys);
                    },
                );
                let label = memo(move || item.get().label());
                let desc = memo(move || item.get().description_value());
                let disabled = memo(move || off.get() || item.get().is_disabled());
                checkbox_row(label, desc, checked, disabled)
            },
        );
        root.child(column(rows).style(style().gap(4))).into_view()
    }
}

/// A labelled checkbox with supporting text and a row-sized hit area.
pub fn checkbox_list_item(
    label: impl IntoText,
    description: impl IntoValue<Option<String>>,
    checked: impl Into<Binding<bool>>,
) -> Node {
    checkbox_row(
        label.into_text_value(),
        description.into_value(),
        checked.into(),
        false.into_value(),
    )
}

fn checkbox_row(
    label: Value<String>,
    description: Value<Option<String>>,
    checked: Binding<bool>,
    disabled: Value<bool>,
) -> Node {
    let control_ref = node_ref();
    let details = show_some(description, |description| {
        text(description).style(move || style().font_size(12).color(tokens().colors.text_secondary))
    });
    let control = checkbox(label, checked)
        .hide_label(true)
        .disabled(disabled)
        .node_ref(control_ref)
        .description(move || description.get().unwrap_or_default());
    row((control, column((text(label), details))))
        .role(Role::Generic)
        .on_press(move || {
            if !disabled.get_untracked() {
                control_ref.activate();
            }
        })
        .style(move || {
            let t = tokens();
            style()
                .align_items(Align::Center)
                .min_height(32)
                .padding_x(8)
                .padding_y(6)
                .gap(8)
                .radius(t.radius.element)
                .hover(style().background(t.colors.overlay_hover))
        })
}
