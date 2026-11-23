//! Role: Reusable dashboard, navigation, conversation and scheduling patterns.

use crate::Hue;
use crate::color;
use kivact::component;
use kivact::custom::Caller;
use kivact::i18n::message;
use kivact::state::{Callback, IntoValue, Value};
use kivact::style::color as core;
use kivact::style::{Align, TextAlign, style};
use kivact::ui::{ButtonVariant, button};
use kivact::view::{Element, View, column, icon, row, show, text};

/// A scheduling row with a time, a title, optional description and period,
/// and an optional details action that reports activation to the application.
#[component]
pub fn agenda_item(
    time: Value<String>,
    title: Value<String>,
    #[prop(default = "")] period: Value<String>,
    #[prop(default = Hue::Gray)] tone: Value<Hue>,
    on_details: Option<Callback<()>>,
    #[caller] caller: Caller,
) -> impl Element {
    let disabled = caller.disabled().unwrap_or_else(|| false.into_value());
    let description = caller
        .description()
        .unwrap_or_else(|| String::new().into_value());
    let hue = tone;
    row(())
        .base_style(move || {
            style()
                .padding_y(10)
                .padding_right(12)
                .gap(14)
                .align_items(Align::Center)
                .border(1, color::LINE)
                .border_left(3, hue.get().accent())
                .border_radius(8)
                .background(color::SURFACE)
        })
        .child(
            column((
                text(time).style(
                    style()
                        .font_weight(600)
                        .line_height(1.2)
                        .text_align(TextAlign::End),
                ),
                show(
                    move || !period.get().is_empty(),
                    move || {
                        text(period).style(
                            style()
                                .font_size(11)
                                .color(core::TEXT_MUTED)
                                .text_align(TextAlign::End),
                        )
                    },
                ),
            ))
            .style(style().width(60).font_size(14).text_align(TextAlign::End)),
        )
        .child(
            column((
                text(title).style(style().font_weight(500)),
                text(description).style(style().font_size(12).color(core::TEXT_MUTED)),
            ))
            .style(style().flex_grow(1)),
        )
        .child(on_details.map(|callback| -> View {
            button(message("pop.agenda.details", "Event details"))
                .disabled(disabled)
                .variant(ButtonVariant::Ghost)
                .hide_label(true)
                .icon(icon("chevron-right"))
                .on_press(move || callback.call(()))
                .style(style().size(28, 28))
                .into_view()
        }))
}
