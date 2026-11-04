//! Role: Attach a Neutral action menu to a compact overflow trigger.

use kivact::state::ItemKey;

use kivact::interaction::MenuEntry;
use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::ui::Button;
use kivact::ui::ButtonVariant;
use kivact::ui::ControlSize;
use kivact::ui::Menu;
use kivact::ui::button;
use kivact::ui::menu;
use kivact::ui::tooltip;
use kivact::view::HandlerResult;
use kivact::view::IntoView;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::OverlayAlign;
use kivact::view::Placement;
use kivact::view::View;
use kivact::view::icon;

/// Icon-only menu trigger paired with the standard themed menu.
pub struct MoreMenu<K: ItemKey> {
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
    K: ItemKey,
    R: HandlerResult,
{
    let label = kivact::i18n::message("soft.more_menu.label", "More options");
    MoreMenu {
        menu: menu(label, entries).on_activate(choose),
        label,
        variant: ButtonVariant::Ghost,
        size: ControlSize::Medium,
        icon: None,
        disabled: false.into_value(),
    }
}

impl<K: ItemKey> MoreMenu<K> {
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
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        self.button.node_mut()
    }
}

impl IntoView for MoreTrigger {
    fn into_view(self) -> View {
        tooltip(self.label).attach(self.button)
    }
}

impl<K: ItemKey> IntoView for MoreMenu<K> {
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
