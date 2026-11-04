//! Role: Labelled rows of checkbox and radio options.
use crate::theme_access::tokens;
use kivact::state::IntoBinding;
use kivact::state::ItemKey;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::column;
use kivact::view::node;
use kivact::view::row;
use kivact::view::text;

use kivact::interaction::ListItem;
use kivact::state::Binding;
use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::state::binding;
use kivact::state::memo;
use kivact::style::Align;
use kivact::style::style;
use kivact::ui::RadioGroup;
use kivact::ui::checkbox;
use kivact::ui::radio_group;
use kivact::view::IntoView;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::for_each;
use kivact::view::node_ref;
use kivact::view::show_some;

pub struct CheckboxList<K: ItemKey> {
    root: Node,
    options: Value<Vec<ListItem<K>>>,
    value: Binding<Vec<K>>,
}

pub fn checkbox_list<K: ItemKey>(
    label: impl IntoText,
    options: impl IntoValue<Vec<ListItem<K>>>,
    value: impl IntoBinding<Vec<K>>,
) -> CheckboxList<K> {
    CheckboxList {
        root: node()
            .with_node(move |node| node.role(Role::Group))
            .name(label),
        options: options.into_value(),
        value: value.into_binding(),
    }
}

pub fn radio_list<K: ItemKey>(
    label: impl IntoText,
    options: impl IntoValue<Vec<ListItem<K>>>,
    value: impl IntoBinding<Option<K>>,
) -> RadioGroup<K> {
    radio_group(label, options, value)
}

impl<K: ItemKey> NodeBuilder for CheckboxList<K> {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl<K: ItemKey> IntoView for CheckboxList<K> {
    fn into_view(self) -> View {
        let Self {
            mut root,
            options,
            value,
        } = self;
        let disabled = root.accessible_disabled();
        let off = memo(move || disabled.as_ref().is_some_and(|disabled| disabled.get()));
        let rows = for_each(
            options,
            |item| item.key().clone(),
            move |item| {
                let key = item.with_untracked(|item| item.key().clone());
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
                let desc = memo(move || item.get().description());
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
    checked: impl IntoBinding<bool>,
) -> Node {
    checkbox_row(
        label.into_text_value(),
        description.into_value(),
        checked.into_binding(),
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
        .with_node(move |node| node.role(Role::Generic))
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
