//! Role: Shared layout and input helpers of the game catalog pages.

use kivact::interaction::ListItem;
use kivact::prelude::{Element, IntoView, Key, column, grid, node, row};
use kivact::state::{IntoText, IntoValue};
use kivact::style::color;
use kivact::style::{Align, Justify, pct, style};
use kivact::style::{GridPlacement, Track};
use kivact::ui::{RadioGroup, radio_group};
use kivact::view::Node;
use kivact::view::{Handled, Orientation};
use kivact_ui_game as g;

/// A segmented control: a standard radio group in the Game look of slanted
/// segments.
pub(crate) fn segmented<K: kivact::state::ItemKey>(
    label: impl IntoText,
    options: impl IntoValue<Vec<ListItem<K>>>,
    value: kivact::state::Signal<Option<K>>,
) -> RadioGroup<K> {
    radio_group(label, options, value)
        .orientation(Orientation::Horizontal)
        .variant(kivact::ui::RadioGroupVariant::Segmented)
}

pub(crate) fn suppress_search_shortcut(event: kivact::view::KeyEvent) -> Handled {
    if event.key() == Key::Char('/') {
        Handled::Yes
    } else {
        Handled::No
    }
}

pub(crate) fn cards(children: impl IntoView, minimum: f64) -> Node {
    grid(children).style(
        style()
            .grid_template_columns(Track::repeat_auto_fill(Track::minmax(
                minimum,
                Track::fr(1),
            )))
            .gap(16)
            .align_items(Align::Stretch),
    )
}

pub(crate) fn specimen(id: &str, name: &str, body: impl IntoView) -> impl kivact::view::Element {
    let body = node()
        .child(body)
        .style(style().min_height(if matches!(id, "OVL-01" | "OVL-05") {
            56
        } else {
            0
        }));
    let caption = match id {
        "TKN-01" => "click → copy",
        "BTN-03" => "click: loading",
        "BTN-05" => "native dense",
        "BTN-06" => "hero action",
        "INP-05" => "Enter to add",
        "NAV-01" => "← → to move",
        "DSP-01" => "last 7 days",
        "FBK-03" => "Shown at the bottom right",
        "OVL-01" => "↑↓ / Esc",
        "OVL-04" => "hover / focus",
        _ => "",
    };
    crate::support::specimens::specimen_frame(id.to_owned(), name.to_owned(), body.into_view())
        .caption(caption.to_owned())
        .test_id(id)
}

pub(crate) fn line(children: impl IntoView) -> Node {
    row(children).style(style().align_items(Align::Center).gap(12).flex_wrap(true))
}

pub(crate) fn list(children: impl IntoView) -> Node {
    column(children).style(style().gap(12).width(pct(100)))
}

pub(crate) fn wide<B: Element>(view: B) -> B {
    view.style(style().grid_column(GridPlacement::span(2)))
}

pub(crate) fn meta(label: impl IntoText) -> Node {
    node()
        .style(
            style()
                .height(26)
                .padding_x(10)
                .justify_content(Justify::Center)
                .background(color::SURFACE_ALT)
                .border(1, color::BORDER),
        )
        .child(g::mono(label))
}

pub(crate) fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|v| (*v).to_owned()).collect()
}

pub(crate) fn choice_options(items: &[(&str, &str)]) -> Vec<ListItem<String>> {
    items
        .iter()
        .map(|(key, label)| ListItem::new((*key).to_owned(), *label))
        .collect()
}

/// The theme's display family, which the headings and numbers of the console use.
pub(crate) fn display_family() -> String {
    kivact::style::use_theme()
        .get()
        .get(&g::fonts::DISPLAY_FAMILY_VAR)
        .as_str()
        .unwrap_or_default()
        .to_owned()
}
