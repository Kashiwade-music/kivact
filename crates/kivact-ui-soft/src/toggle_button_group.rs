//! Role: Group independently tabbable Neutral toggle buttons.

use std::hash::Hash;
use std::rc::Rc;

use kivact::control::{Node, NodeBuilder, icon, node};
use kivact::primitive::NodeExt;
use kivact::style::{Align, Direction, style};
use kivact::ui::{ControlSize, ListItem, toggle};
use kivact::view::{
    IntoText, IntoValue, IntoView, Orientation, ReadingDirection, Role, View, current_direction,
    for_each, show,
};
use kivact::{Binding, Signal, Value, binding};

/// Caller-owned selection for a toggle group.
pub enum ToggleSelection<K: Clone + PartialEq + 'static> {
    Single(Binding<Option<K>>),
    Multiple(Binding<Vec<K>>),
}

impl<K: Clone + PartialEq + 'static> From<Binding<Option<K>>> for ToggleSelection<K> {
    fn from(binding: Binding<Option<K>>) -> Self {
        Self::Single(binding)
    }
}
impl<K: Clone + PartialEq + 'static> From<Binding<Vec<K>>> for ToggleSelection<K> {
    fn from(binding: Binding<Vec<K>>) -> Self {
        Self::Multiple(binding)
    }
}
impl<K: Clone + PartialEq + 'static> From<Signal<Option<K>>> for ToggleSelection<K> {
    fn from(signal: Signal<Option<K>>) -> Self {
        Self::Single(signal.into())
    }
}
impl<K: Clone + PartialEq + 'static> From<Signal<Vec<K>>> for ToggleSelection<K> {
    fn from(signal: Signal<Vec<K>>) -> Self {
        Self::Multiple(signal.into())
    }
}

impl<K: Clone + PartialEq + 'static> ToggleSelection<K> {
    fn member(&self, key: K) -> Binding<bool> {
        match *self {
            Self::Single(selected) => {
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
            Self::Multiple(selected) => {
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
}

type PressedIcon<K> = Rc<dyn Fn(&K) -> Option<&'static str>>;

/// Separated ghost toggles with single or multiple selection.
pub struct ToggleButtonGroup<K: Clone + Eq + Hash + 'static> {
    root: Node,
    items: Value<Vec<ListItem<K>>>,
    selection: ToggleSelection<K>,
    size: ControlSize,
    orientation: Orientation,
    disabled: Value<bool>,
    icon_only: bool,
    pressed_icon: Option<PressedIcon<K>>,
}

#[track_caller]
pub fn toggle_button_group<K: Clone + Eq + Hash + 'static>(
    label: impl IntoText,
    items: impl IntoValue<Vec<ListItem<K>>>,
    value: impl Into<ToggleSelection<K>>,
) -> ToggleButtonGroup<K> {
    ToggleButtonGroup {
        root: node().role(Role::Group).name(label),
        items: items.into_value(),
        selection: value.into(),
        size: ControlSize::Medium,
        orientation: Orientation::Horizontal,
        disabled: false.into_value(),
        icon_only: false,
        pressed_icon: None,
    }
}

impl<K: Clone + Eq + Hash + 'static> ToggleButtonGroup<K> {
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    pub fn disabled(mut self, disabled: impl IntoValue<bool>) -> Self {
        self.disabled = disabled.into_value();
        self
    }

    pub fn icon_only(mut self, icon_only: bool) -> Self {
        self.icon_only = icon_only;
        self
    }

    pub fn pressed_icon(mut self, icon: impl Fn(&K) -> Option<&'static str> + 'static) -> Self {
        self.pressed_icon = Some(Rc::new(icon));
        self
    }
}

impl<K: Clone + Eq + Hash + 'static> NodeBuilder for ToggleButtonGroup<K> {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl<K: Clone + Eq + Hash + 'static> IntoView for ToggleButtonGroup<K> {
    fn into_view(self) -> View {
        let vertical = self.orientation == Orientation::Vertical;
        let direction = match (vertical, current_direction()) {
            (true, _) => Direction::Column,
            (false, ReadingDirection::Rtl) => Direction::RowReverse,
            _ => Direction::Row,
        };
        let size = self.size;
        let disabled = self.disabled;
        let icon_only = self.icon_only;
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
                move |key, item| {
                    let alternate = pressed_icon.as_ref().and_then(|icon| icon(&key));
                    let pressed = selection.member(key);
                    let control = toggle(move || item.get().label(), pressed)
                        .size(size)
                        .disabled(move || disabled.get() || item.get().is_disabled());
                    let has_icon = item.get().icon_value().is_some() || alternate.is_some();
                    if has_icon {
                        control
                            .icon(show(
                                move || {
                                    item.get().icon_value().is_some()
                                        || (alternate.is_some() && pressed.get())
                                },
                                move || {
                                    icon(move || {
                                        if pressed.get()
                                            && let Some(alternate) = alternate
                                        {
                                            return alternate.into();
                                        }
                                        item.get()
                                            .icon_value()
                                            .cloned()
                                            .unwrap_or_else(|| "".into())
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
