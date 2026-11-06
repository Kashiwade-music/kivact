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
    button: Button,
}

#[track_caller]
/// Create an overflow menu whose chosen entry key is passed to `choose`.
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
        button: button(label)
            .icon(icon("ellipsis"))
            .hide_label(true)
            .variant(ButtonVariant::Ghost),
    }
}

impl<K: ItemKey> MoreMenu<K> {
    /// Set the trigger's accessible name and tooltip text.
    pub fn label(mut self, label: impl IntoText) -> Self {
        self.label = label.into_text_value();
        self.button = self.button.name(self.label);
        self
    }

    /// Set the visual variant of the overflow trigger button.
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.button = self.button.variant(variant);
        self
    }

    /// Set the trigger button size, including reactive size changes.
    pub fn size(mut self, size: impl IntoValue<ControlSize>) -> Self {
        self.button = self.button.size(size);
        self
    }

    /// Mount replacement trigger icon content once.
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.button = self.button.icon(icon);
        self
    }

    /// Place the menu on the chosen side of its trigger.
    pub fn placement(mut self, placement: Placement) -> Self {
        self.menu = self.menu.placement(placement);
        self
    }

    /// Align the menu with the start, center or end of its trigger.
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
        let trigger = self.button;
        self.menu.attach(MoreTrigger {
            button: trigger,
            label: self.label,
        })
    }
}

impl<K: ItemKey> NodeBuilder for MoreMenu<K> {
    fn node_mut(&mut self) -> &mut Node {
        self.button.node_mut()
    }
}
