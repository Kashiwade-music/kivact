//! Role: Compose Soft bulk selection and status on canonical controls.
use crate::color as c;
use crate::radius as r;
use crate::variants::RowStatusDot;

use kivact::Prop;
use kivact::component;
use kivact::state::IntoText;
use kivact::state::PropValue;
use kivact::state::Signal;
use kivact::state::Value;
use kivact::style::Align;
use kivact::style::FlexDirection;
use kivact::style::style;
use kivact::view::Element;
use kivact::view::View;
use kivact::view::text;

/// Bulk table actions shown while the external selection contains keys.
///
/// The clear action empties `selection`. The toolbar hides while nothing is selected.
#[component]
pub fn table_selection_toolbar<K: PropValue>(
    selection: Signal<Vec<K>>,
    #[prop(default = "Unselect All".into_text_value())] clear_label: Value<String>,
    selection_label: Option<Rc<dyn Fn(Value<usize>) -> View>>,
    actions: Option<View>,
) -> impl Element {
    let root = kivact::ui::toolbar("Bulk actions");
    let count = kivact::state::memo(move || selection.with(Vec::len));
    let count_label = match selection_label {
        Some(render) => render(count),
        None => text(move || format!("{} selected", count.get())).into_view(),
    };
    let clear = kivact::ui::button(clear_label)
        .variant(kivact::ui::ButtonVariant::Ghost)
        .size(kivact::ui::ControlSize::Small)
        .on_press(move || selection.set(Vec::new()));
    root.hidden(move || count.get() == 0)
        .with_node(move |node| {
            node.base_style(move || {
                style()
                    .flex_direction(FlexDirection::Row)
                    .align_items(Align::Center)
                    .padding_x(8)
                    .padding_y(4)
                    .gap(8)
                    .border_radius(r::RADIUS_ELEMENT)
                    .background(c::BG_MUTED)
                    .font_size(14)
                    .color(c::TEXT_PRIMARY)
            })
        })
        .start((count_label, clear))
        .end((actions,))
}

/// The semantic colors of a table's status marker.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum RowStatusTone {
    #[default]
    /// Use neutral status ink.
    Neutral,
    /// Use successful status ink.
    Success,
    /// Use warning status ink.
    Warning,
    /// Use error status ink.
    Error,
}

/// Show a 1-based row number for a table's index lane, end-aligned in the
/// secondary text color. Constant or reactive.
#[component]
pub fn table_row_index(index: Value<usize>) -> impl Element {
    text(move || index.get().to_string()).base_style(move || {
        style()
            .font_size(12)
            .text_align(kivact::style::TextAlign::End)
            .color(c::TEXT_SECONDARY)
    })
}

/// Show a named semantic status marker without adding another status owner.
#[component]
pub fn table_row_status(label: Value<String>, tone: Value<RowStatusTone>) -> impl Element {
    kivact::ui::badge("")
        .tone(move || match tone.get() {
            RowStatusTone::Neutral => kivact::interaction::Tone::Neutral,
            RowStatusTone::Success => kivact::interaction::Tone::Success,
            RowStatusTone::Warning => kivact::interaction::Tone::Warning,
            RowStatusTone::Error => kivact::interaction::Tone::Danger,
        })
        .variant(RowStatusDot)
        .name(label)
        .with_node(|node| node.role(kivact::view::Role::Image))
}
