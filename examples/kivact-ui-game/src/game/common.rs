//! Role: Shared layout and input helpers for Game catalog specimens.

use kivact::interaction::ListItem;
use kivact::prelude::{
    ColorToken, IntoText, IntoValue, IntoView, Key, Node, NodeBuilder, column, grid, node, row,
};
use kivact::style::{Align, Justify, pct, style};
use kivact::style::{GridPlacement, Track};
use kivact::ui::{RadioGroup, radio_group};
use kivact::view::{Handled, Orientation};
use kivact_ui_game as g;

/// A segmented control: a standard radio group in the Game look of slanted
/// segments.
pub(super) fn segmented<K: kivact::state::ItemKey>(
    label: impl IntoText,
    options: impl IntoValue<Vec<ListItem<K>>>,
    value: impl kivact::state::IntoBinding<Option<K>>,
) -> RadioGroup<K> {
    radio_group(label, options, value)
        .orientation(Orientation::Horizontal)
        .variant(kivact::ui::RadioGroupVariant::Segmented)
}

pub(super) fn suppress_search_shortcut(event: kivact::view::KeyEvent) -> Handled {
    if event.key() == Key::Char('/') {
        Handled::Yes
    } else {
        Handled::No
    }
}

pub(super) fn cards(children: impl IntoView, minimum: f64) -> Node {
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

pub(super) fn specimen(
    id: &str,
    name: &str,
    body: impl IntoView,
) -> crate::game::specimens::Specimen<impl kivact::view::Component> {
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
        "INP-05" => "Enter で追加",
        "NAV-01" => "← → で移動",
        "DSP-01" => "last 7 days",
        "FBK-03" => "右下に表示",
        "OVL-01" => "↑↓ / Esc",
        "OVL-04" => "hover / focus",
        _ => "",
    };
    crate::game::specimens::specimen(id.to_owned(), name.to_owned(), body)
        .caption(caption)
        .test_id(id)
}

pub(super) fn line(children: impl IntoView) -> Node {
    row(children).style(style().align_items(Align::Center).gap(12).wrap(true))
}

pub(super) fn list(children: impl IntoView) -> Node {
    column(children).style(style().gap(12).width(pct(100)))
}

pub(super) fn wide<B: NodeBuilder>(view: B) -> B {
    view.style(style().grid_column(GridPlacement::span(2)))
}

pub(super) fn meta(label: impl IntoText) -> Node {
    node()
        .style(
            style()
                .height(26)
                .padding_x(10)
                .justify_content(Justify::Center)
                .background(ColorToken::SurfaceAlt)
                .border(1, ColorToken::Border),
        )
        .child(g::mono(label))
}

pub(super) fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|v| (*v).to_owned()).collect()
}
