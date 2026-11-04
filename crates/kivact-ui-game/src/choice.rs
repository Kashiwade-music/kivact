//! Role: Game looks of the radio group and tabs, and the slots that fill them.

use crate::Color;
use kivact::interaction::ListItem;
use kivact::interaction::TabState;
use kivact::state::{Value, memo};
use kivact::style::{style, use_theme};
use kivact::ui::CheckableState;
use kivact::view::{IntoView, View, dynamic, show};
use kivact::view::{column, text};

use crate::Tokens;

/// Slanted single-selection segments: the Game look of a standard radio
/// group, `kivact::ui::radio_group(..).orientation(Orientation::Horizontal)
/// .variant(Segmented)`. A segmented control is a
/// radio group, not tabs; another theme draws the plain radio group.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Segmented;

impl kivact::ui::Variant for Segmented {}

/// Selectable plan cards: the Game look of a standard radio group,
/// `kivact::ui::radio_group(..).variant(ChoiceCard).item(card_label)`. A
/// card shows its item's description under the label; a price goes beside
/// [`card_label`] in the slot. Another theme draws the plain radio group.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ChoiceCard;

impl kivact::ui::Variant for ChoiceCard {}

/// Vertical navigation with a blue selection edge: the Game look of
/// standard tabs, `kivact::ui::tabs(..).orientation(Orientation::Vertical).variant(Navigation)
/// .item(navigation_label)`. A count goes beside [`navigation_label`] in the
/// slot. Another theme draws the plain tabs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Navigation;

impl kivact::ui::Variant for Navigation {}

/// Boxed tabs: the tabs sit as segments in a shaded box, the selected one
/// raised on the surface with an accent edge, in place of the Game theme's
/// underlined tabs: `kivact::ui::tabs(..).variant(Boxed)`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Boxed;

impl kivact::ui::Variant for Boxed {}

/// What a [`ChoiceCard`] shows: the item's icon, its label in the display
/// type (in the accent while chosen) and its description. Put a price or a
/// count after it in the slot.
///
/// ```no_run
/// use kivact::interaction::{ListItem};
/// use kivact::prelude::*;
/// use kivact_ui_game as g;
///
/// fn plans() -> impl IntoView {
///     let plan = signal(Some("team"));
///     radio_group(
///         "Plan",
///         vec![ListItem::new("solo", "SOLO"), ListItem::new("team", "TEAM").with_description("50 seats")],
///         plan,
///     )
///     .variant(g::ChoiceCard)
///     .item(|item, state| (g::card_label(item, state), text("¥39,800/month")))
/// }
/// ```
pub fn card_label<K: Clone + 'static>(
    item: Value<ListItem<K>>,
    state: Value<CheckableState>,
) -> View {
    (
        icon_view(item),
        text(move || item.with(ListItem::label)).style(move || {
            let theme = &use_theme().get();
            style()
                .font_family(theme.ext::<Tokens>().display_family.as_str())
                .font_size(18)
                .weight(700)
                .italic(true)
                .color(if state.get().checked.is_checked() {
                    theme.colors().accent
                } else {
                    theme.colors().text
                })
        }),
        description_view(description(item)),
    )
        .into_view()
}

/// What a [`Navigation`] entry shows before its trailing content: the item's
/// icon, its label over its description. Put a count or a badge after it in
/// the slot.
///
/// ```no_run
/// use kivact::interaction::{ListItem};
/// use kivact::prelude::*;
/// use kivact_ui_game as g;
///
/// fn sections() -> impl IntoView {
///     let section = signal("inbox");
///     tabs("Sections", vec![ListItem::new("inbox", "Inbox"), ListItem::new("alerts", "Alerts")], section)
///         .orientation(Orientation::Vertical)
///         .variant(g::Navigation)
///         .item(|item, state| (g::navigation_label(item, state), g::badge("3", kivact::interaction::Tone::Info)))
/// }
/// ```
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
        .style(style().grow(1)),
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
                .style(style().font_size(11.5).color(Color::Text3))
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
