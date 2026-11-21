//! Role: Removable chips with application-owned additions.

use kivact::component;
use kivact::i18n::{message, message_with};
use kivact::state::{Callback, Value, callback};
use kivact::style::color as core;
use kivact::style::{Align, BorderStyle, style};
use kivact::ui::{ButtonVariant, ControlSize, button};
use kivact::view::{Element, for_index, icon, row, text};

/// A chip with a label and a remove button. `on_remove` runs when the
/// remove button is invoked.
#[component]
pub fn removable_chip(label: Value<String>, on_remove: Option<Callback<()>>) -> impl Element {
    let remove = on_remove.unwrap_or_else(|| callback(|_| ()));
    row(())
        .child((
            text(label),
            button(message_with(
                "pop.chip.remove",
                "Remove {label}",
                move || [("label", label.get())],
            ))
            .variant(ButtonVariant::Ghost)
            .hide_label(true)
            .icon(icon("x"))
            .style(style().size(16, 16))
            .on_press(remove),
        ))
        .style(
            style()
                .gap(4)
                .height(26)
                .padding_x(10)
                .align_items(Align::Center)
                .background(core::SURFACE_ALT)
                .border_radius(6)
                .font_size(13),
        )
}

/// A chip collection that reports requests to its owner: `on_add` when the
/// add button is invoked, `on_remove` with the index of a removed chip.
#[component]
pub fn chip_list(
    tags: Value<Vec<String>>,
    on_add: Option<Callback<()>>,
    on_remove: Option<Callback<usize>>,
) -> impl Element {
    let remove = on_remove.unwrap_or_else(|| callback(|_: usize| ()));
    let add = on_add.unwrap_or_else(|| callback(|_| ()));
    row(()).style(style().gap(6).flex_wrap(true)).child((
        for_index(tags, move |index, label| {
            removable_chip(label).on_remove(move || remove.call(index))
        }),
        button(message("pop.chip.add", "Add"))
            .icon(icon("plus"))
            .size(ControlSize::Small)
            .variant(ButtonVariant::Ghost)
            .on_press(move || add.call(()))
            .style(
                style()
                    .height(26)
                    .border(1, core::BORDER)
                    .border_style(BorderStyle::Dashed),
            ),
    ))
}
