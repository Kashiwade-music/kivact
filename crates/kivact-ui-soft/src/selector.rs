//! Role: Single and multiple option selection and custom selector popovers.
use crate::recipes_field::frame;
use crate::theme_access::tokens;
use crate::typeahead::Typeahead;
use kivact::control::{NodeBuilder, icon, scroll, text};
use kivact::interaction::ListSelection;
use kivact::primitive::PartState;
use kivact::style::{Align, Direction, Length, style};
use kivact::ui::{ButtonVariant, DialogKind, ListItem, Select, button, dialog, select};
use kivact::view::{IntoText, IntoValue, IntoView, View};
use kivact::{Binding, signal};
use std::hash::Hash;

pub type Selector<K> = Select<K>;
pub type MultiSelector<K> = Typeahead<K>;

pub fn selector<K: Clone + Eq + Hash + 'static>(
    label: impl IntoText,
    options: impl IntoValue<Vec<ListItem<K>>>,
    value: impl Into<Binding<Option<K>>>,
) -> Selector<K> {
    select(label, options, value).open_on_arrow(true)
}

pub fn multi_selector<K: Clone + Eq + Hash + 'static>(
    label: impl IntoText,
    options: impl IntoValue<Vec<ListItem<K>>>,
    value: impl Into<Binding<Vec<K>>>,
) -> MultiSelector<K> {
    let value = value.into();
    let mut field = Typeahead::new(label, options, ListSelection::Multiple(value));
    field.chips = Some(value);
    field
}

pub fn complex_selector<V: IntoView>(
    label: impl IntoText,
    value_label: impl IntoText,
    content: impl Fn() -> V + 'static,
) -> View {
    let label = label.into_text_value();
    let value_label = value_label.into_text_value();
    let open = signal(false);
    let trigger = button(label)
        .variant(ButtonVariant::Ghost)
        .hide_label(true)
        .child(text(value_label))
        .trailing_icon(icon("chevron-down"))
        .expanded(open)
        .on_press(move || open.set(!open.get_untracked()))
        .style(move || {
            let t = tokens();
            frame(&t, &PartState::default(), false)
                .height(32)
                .padding_x(12)
                .direction(Direction::Row)
                .align_items(Align::Center)
                .gap(8)
        });
    dialog(label, open)
        .kind(DialogKind::Popover)
        .hide_title(true)
        .width(Length::Auto)
        .content(move || scroll(content()).style(style().max_height(480).padding(12)))
        .attach(trigger)
}
