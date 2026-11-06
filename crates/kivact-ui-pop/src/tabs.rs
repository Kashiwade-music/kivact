//! Role: The Pop content of a tab or segment: icon, label and count badge.

use kivact::interaction::ListItem;
use kivact::interaction::TabState;
use kivact::state::{Value, memo};
use kivact::style::{Align, style};
use kivact::view::{IntoView, View, dynamic, show};
use kivact::view::{row, text};

/// What a Pop tab shows: the item's icon, its label and, while it has a
/// [`description`](ListItem::description), that text as a count badge.
///
/// The slot of [`kivact::ui::tabs`] in Pop's look; the kit's theme
/// gives the tabs their underline. The default radio-group slot draws segments.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_pop as ui;
///
/// fn sections() -> impl IntoView {
///     let selected = signal("skills");
///     tabs(
///         "Profile sections",
///         vec![
///             ListItem::new("overview", "Overview").with_icon("home"),
///             ListItem::new("skills", "Skills").with_description("4"),
///         ],
///         selected,
///     )
///     .item(ui::tab_label)
///     .content(|key| text(format!("{key} panel")))
/// }
/// ```
pub fn tab_label<K: Clone + 'static>(item: Value<ListItem<K>>, _state: Value<TabState>) -> View {
    item_label(item)
}

/// The icon, label and count badge of `item`.
pub(crate) fn item_label<K: Clone + 'static>(item: Value<ListItem<K>>) -> View {
    let count = memo(move || item.with(ListItem::description));
    row((
        dynamic(
            move || item.with(|item| item.icon().cloned()),
            |icon| icon.map(kivact::view::icon),
        ),
        text(move || item.with(ListItem::label)),
        show(
            move || count.get().is_some(),
            move || {
                kivact::ui::badge(move || count.get().unwrap_or_default())
                    .variant(crate::BadgeVariant::Count)
            },
        ),
    ))
    .style(style().gap(6).align_items(Align::Center))
    .into_view()
}
