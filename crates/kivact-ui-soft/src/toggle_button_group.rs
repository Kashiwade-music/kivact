//! Role: Group independently tabbable Neutral toggle buttons.
use kivact::interaction::IntoSelection;
use kivact::interaction::Selection;

use kivact::state::ItemKey;
use std::rc::Rc;

use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::icon;
use kivact::view::node;

use kivact::interaction::ListItem;
use kivact::state::Binding;
use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::state::binding;
use kivact::style::Align;
use kivact::style::Direction;
use kivact::style::style;
use kivact::ui::ControlSize;
use kivact::ui::toggle_button;
use kivact::view::IntoView;
use kivact::view::Orientation;
use kivact::view::ReadingDirection;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::for_each;
use kivact::view::show;
use kivact::view::use_direction;

fn selection_member<K: ItemKey>(selection: &Selection<K>, key: K) -> Binding<bool> {
    match *selection {
        Selection::Single(selected) => {
            let read = key.clone();
            binding(
                move || selected.get().as_ref() == Some(&read),
                move |on| {
                    if on {
                        selected.set(Some(key.clone()));
                    } else if selected.get().as_ref() == Some(&key) {
                        selected.set(None);
                    }
                },
            )
        }
        Selection::Multiple(selected) => {
            let read = key.clone();
            binding(
                move || selected.get().contains(&read),
                move |on| {
                    let mut values = selected.get();
                    if on {
                        if !values.contains(&key) {
                            values.push(key.clone());
                        }
                    } else {
                        values.retain(|value| value != &key);
                    }
                    selected.set(values);
                },
            )
        }
    }
}

type PressedIcon<K> = Rc<dyn Fn(&K) -> Option<&'static str>>;

/// Separated ghost toggles with single or multiple selection.
pub struct ToggleButtonGroup<K: ItemKey> {
    root: Node,
    items: Value<Vec<ListItem<K>>>,
    selection: Selection<K>,
    size: Value<ControlSize>,
    orientation: Value<Orientation>,
    disabled: Value<bool>,
    icon_only: Value<bool>,
    pressed_icon: Option<PressedIcon<K>>,
}

#[track_caller]
pub fn toggle_button_group<K: ItemKey>(
    label: impl IntoText,
    items: impl IntoValue<Vec<ListItem<K>>>,
    value: impl IntoSelection<K>,
) -> ToggleButtonGroup<K> {
    ToggleButtonGroup {
        root: node().role(Role::Group).name(label),
        items: items.into_value(),
        selection: value.into_selection(),
        size: (ControlSize::Medium).into_value(),
        orientation: (Orientation::Horizontal).into_value(),
        disabled: false.into_value(),
        icon_only: (false).into_value(),
        pressed_icon: None,
    }
}

impl<K: ItemKey> ToggleButtonGroup<K> {
    pub fn orientation(mut self, orientation: impl IntoValue<Orientation>) -> Self {
        self.orientation = orientation.into_value();
        self
    }

    pub fn size(mut self, size: impl IntoValue<ControlSize>) -> Self {
        self.size = size.into_value();
        self
    }

    pub fn disabled(mut self, disabled: impl IntoValue<bool>) -> Self {
        self.disabled = disabled.into_value();
        self
    }

    pub fn icon_only(mut self, icon_only: impl IntoValue<bool>) -> Self {
        self.icon_only = icon_only.into_value();
        self
    }

    pub fn pressed_icon(mut self, icon: impl Fn(&K) -> Option<&'static str> + 'static) -> Self {
        self.pressed_icon = Some(Rc::new(icon));
        self
    }
}

impl<K: ItemKey> NodeBuilder for ToggleButtonGroup<K> {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl<K: ItemKey> IntoView for ToggleButtonGroup<K> {
    fn into_view(self) -> View {
        let vertical = self.orientation.get() == Orientation::Vertical;
        let direction = match (vertical, use_direction().get()) {
            (true, _) => Direction::Column,
            (false, ReadingDirection::Rtl) => Direction::RowReverse,
            _ => Direction::Row,
        };
        let size = self.size.get();
        let disabled = self.disabled;
        let icon_only = self.icon_only.get();
        let selection = self.selection;
        let pressed_icon = self.pressed_icon;
        self.root
            .base_style(
                style()
                    .direction(direction)
                    .gap(4)
                    .align_items(if vertical {
                        Align::Stretch
                    } else {
                        Align::Center
                    }),
            )
            .child(for_each(
                self.items,
                |item| item.key().clone(),
                move |item| {
                    let key = item.with_untracked(|item| item.key().clone());
                    let alternate = pressed_icon.as_ref().and_then(|icon| icon(&key));
                    let pressed = selection_member(&selection, key);
                    let control = toggle_button(move || item.get().label(), pressed)
                        .size(size)
                        .disabled(move || disabled || item.get().is_disabled());
                    let has_icon = item.get().icon().is_some() || alternate.is_some();
                    if has_icon {
                        control
                            .icon(show(
                                move || {
                                    item.get().icon().is_some()
                                        || (alternate.is_some() && pressed.get())
                                },
                                move || {
                                    icon(move || {
                                        if pressed.get()
                                            && let Some(alternate) = alternate
                                        {
                                            return alternate.into();
                                        }
                                        item.get().icon().cloned().unwrap_or_else(|| "".into())
                                    })
                                },
                            ))
                            .hide_label(icon_only)
                    } else {
                        control
                    }
                },
            ))
            .into_view()
    }
}
