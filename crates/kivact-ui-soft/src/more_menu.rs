//! Role: Attach a Neutral action menu to a compact overflow trigger.

use std::hash::Hash;

use kivact::Value;
use kivact::control::{Node, NodeBuilder, icon};
use kivact::ui::{Button, ButtonVariant, ControlSize, Menu, MenuEntry, button, menu, tooltip};
use kivact::view::{HandlerResult, IntoText, IntoValue, IntoView, OverlayAlign, Placement, View};

/// Icon-only menu trigger paired with the standard themed menu.
pub struct MoreMenu<K: Clone + Eq + Hash + 'static> {
    menu: Menu<K>,
    label: Value<String>,
    variant: ButtonVariant,
    size: ControlSize,
    icon: Option<View>,
    disabled: Value<bool>,
}

#[track_caller]
pub fn more_menu<K, R>(
    entries: impl IntoValue<Vec<MenuEntry<K>>>,
    choose: impl Fn(K) -> R + 'static,
) -> MoreMenu<K>
where
    K: Clone + Eq + Hash + 'static,
    R: HandlerResult,
{
    let label = kivact::i18n::message("soft.more_menu.label", "More options");
    MoreMenu {
        menu: menu(label, entries, choose),
        label,
        variant: ButtonVariant::Ghost,
        size: ControlSize::Medium,
        icon: None,
        disabled: false.into_value(),
    }
}

impl<K: Clone + Eq + Hash + 'static> MoreMenu<K> {
    pub fn label(mut self, label: impl IntoText) -> Self {
        self.label = label.into_text_value();
        self
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }

    pub fn disabled(mut self, disabled: impl IntoValue<bool>) -> Self {
        self.disabled = disabled.into_value();
        self
    }

    pub fn placement(mut self, placement: Placement) -> Self {
        self.menu = self.menu.placement(placement);
        self
    }

    pub fn align(mut self, align: OverlayAlign) -> Self {
        self.menu = self.menu.align(align);
        self
    }
}

struct MoreTrigger {
    button: Button,
    label: Value<String>,
}

impl NodeBuilder for MoreTrigger {
    fn node_mut(&mut self) -> &mut Node {
        self.button.node_mut()
    }
}

impl IntoView for MoreTrigger {
    fn into_view(self) -> View {
        tooltip(self.label).attach(self.button)
    }
}

impl<K: Clone + Eq + Hash + 'static> IntoView for MoreMenu<K> {
    fn into_view(self) -> View {
        let trigger = button(self.label)
            .icon(self.icon.unwrap_or_else(|| icon("ellipsis").into_view()))
            .hide_label(true)
            .variant(self.variant)
            .size(self.size)
            .disabled(self.disabled);
        self.menu.attach(MoreTrigger {
            button: trigger,
            label: self.label,
        })
    }
}
