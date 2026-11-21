//! Role: The console's slot contents of the radio group and the tabs: the option's label, description and icon.

use crate::color::TEXT_3;
use crate::fonts::DISPLAY_FAMILY_VAR;
use kivact::interaction::{ListItem, TabState};
use kivact::state::{Value, memo};
use kivact::style::color::{ACCENT, TEXT};
use kivact::style::{style, use_theme};
use kivact::ui::theme::CheckableState;
use kivact::view::{IntoView, View, column, dynamic, show, text};

/// What a card of a [`ChoiceCard`](crate::ChoiceCard) radio group shows: the
/// item's icon, its label in the display type (in the accent while chosen) and
/// its description. A price or a count goes after it in the slot.
pub fn card_label<K: Clone + 'static>(
    item: Value<ListItem<K>>,
    state: Value<CheckableState>,
) -> View {
    (
        icon_view(item),
        text(move || item.with(ListItem::label)).style(move || {
            let theme = use_theme().get();
            style()
                .font_family(theme.get(&DISPLAY_FAMILY_VAR).as_str().unwrap_or_default())
                .font_size(18)
                .font_weight(700)
                .font_style(kivact::style::FontStyle::Italic)
                .color(if state.get().is_checked() {
                    ACCENT
                } else {
                    TEXT
                })
        }),
        description_view(description(item)),
    )
        .into_view()
}

/// What a navigation entry of [`TabsVariant::Navigation`](crate::TabsVariant)
/// shows before its trailing content: the item's icon, its label over its
/// description. A count or a badge goes after it in the slot.
pub fn navigation_label<K: Clone + 'static>(
    item: Value<ListItem<K>>,
    _state: Value<TabState>,
) -> View {
    (
        icon_view(item),
        column((
            text(move || item.with(ListItem::label)),
            description_view(description(item)),
        ))
        .style(style().flex_grow(1.)),
    )
        .into_view()
}

/// The description of `item`, tracked.
fn description<K: Clone + 'static>(item: Value<ListItem<K>>) -> Value<Option<String>> {
    memo(move || item.with(ListItem::description))
}

/// An option's description in the muted small type, left out while it has
/// none or it is empty.
fn description_view(description: Value<Option<String>>) -> View {
    show(
        move || description.with(|text| text.as_ref().is_some_and(|text| !text.is_empty())),
        move || {
            text(move || description.get().unwrap_or_default())
                .style(style().font_size(11.5).color(TEXT_3))
        },
    )
    .into_view()
}

/// An option's icon, as the theme resolves it.
fn icon_view<K: Clone + 'static>(item: Value<ListItem<K>>) -> View {
    dynamic(
        move || item.with(|item| item.icon().cloned()),
        |icon| icon.map(kivact::view::icon),
    )
    .into_view()
}
