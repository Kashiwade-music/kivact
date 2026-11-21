//! Role: Attach a Neutral action menu to a compact overflow trigger.

use kivact::component;
use kivact::state::Callback;
use kivact::state::ItemKey;
use kivact::state::Value;

use kivact::interaction::MenuEntry;
use kivact::ui::ButtonVariant;
use kivact::ui::ControlSize;
use kivact::ui::button;
use kivact::ui::menu;
use kivact::ui::tooltip;
use kivact::view::Element;
use kivact::view::OverlayAlign;
use kivact::view::Placement;
use kivact::view::View;
use kivact::view::node;

/// Icon-only menu trigger paired with the standard themed menu.
///
/// `choose` runs with the key of the chosen entry.
#[component]
pub fn more_menu<K: ItemKey>(
    items: Value<Vec<MenuEntry<K>>>,
    choose: Callback<K>,
    #[prop(default = kivact::i18n::message("soft.more_menu.label", "More options"))] label: Value<
        String,
    >,
    #[prop(default = ButtonVariant::Ghost)] variant: ButtonVariant,
    size: Option<Value<ControlSize>>,
    icon: Option<View>,
    placement: Option<Placement>,
    align: Option<OverlayAlign>,
) -> impl Element {
    let mut overflow = menu(label, items).on_activate(move |key| choose.call(key));
    if let Some(placement) = placement {
        overflow = overflow.placement(placement);
    }
    if let Some(align) = align {
        overflow = overflow.align(align);
    }
    let mut trigger = button(label)
        .icon(icon.unwrap_or_else(|| kivact::view::icon("ellipsis").into_view()))
        .hide_label(true)
        .variant(variant);
    if let Some(size) = size {
        trigger = trigger.size(size);
    }
    node().child(overflow.attach(tooltip(label, trigger)))
}
