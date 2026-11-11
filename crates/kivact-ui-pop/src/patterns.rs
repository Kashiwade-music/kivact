//! Role: Reusable dashboard, navigation, conversation and scheduling patterns.

use crate::{Hue, Tokens};
use kivact::i18n::message;
use kivact::view::{Element, Node, column, icon, row, text};

use kivact::state::{Callback, Value};
use kivact::state::{IntoText, IntoValue};
use kivact::style::use_theme;
use kivact::style::{Align, ColorToken, style};
use kivact::ui::{ButtonVariant, button};
use kivact::view::{IntoView, View, show};

/// A scheduling row with application-provided period and optional details action.
pub struct AgendaItem {
    root: Node,
    time: Value<String>,
    title: Value<String>,
    description: Value<String>,
    period: Value<String>,
    on_details: Option<Callback<()>>,
    hue: Value<Hue>,
}
impl AgendaItem {
    /// Supporting text; empty by default.
    pub fn description(mut self, value: impl IntoText) -> Self {
        self.description = value.into_text_value();
        self
    }
    /// Border hue; neutral gray by default.
    pub fn tone(mut self, value: impl IntoValue<Hue>) -> Self {
        self.hue = value.into_value();
        self
    }

    /// Application-provided period text, omitted when empty.
    pub fn period(mut self, period: impl IntoText) -> Self {
        self.period = period.into_text_value();
        self
    }
    /// Adds a details button that reports activation to the application.
    pub fn on_details(mut self, f: impl Into<kivact::state::Callback<()>>) -> Self {
        self.on_details = Some(f.into());
        self
    }
}
impl Element for AgendaItem {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
#[track_caller]
pub fn agenda_item(time: impl IntoText, title: impl IntoText) -> AgendaItem {
    AgendaItem {
        root: row(()),
        time: time.into_text_value(),
        title: title.into_text_value(),
        description: String::new().into_value(),
        period: String::new().into_value(),
        on_details: None,
        hue: Hue::Gray.into_value(),
    }
}

impl IntoView for AgendaItem {
    fn into_view(self) -> View {
        let Self {
            mut root,
            time,
            title,
            description,
            period,
            on_details,
            hue,
        } = self;
        let disabled = root
            .accessible_disabled()
            .unwrap_or_else(|| false.into_value());
        root.base_style(move || {
            let t = &use_theme().get();
            let p = t.ext::<Tokens>();
            style()
                .padding_y(10)
                .padding_right(12)
                .gap(14)
                .align_items(Align::Center)
                .border(1, p.line)
                .border_left(3, p.accent(hue.get()))
                .radius(8)
                .background(p.surface)
        })
        .child(
            column((
                text(time).style(
                    style()
                        .weight(600)
                        .line_height(1.2)
                        .text_align(kivact::style::TextAlign::End),
                ),
                show(
                    move || !period.get().is_empty(),
                    move || {
                        text(period).style(
                            style()
                                .font_size(11)
                                .color(ColorToken::TextMuted)
                                .text_align(kivact::style::TextAlign::End),
                        )
                    },
                ),
            ))
            .style(
                style()
                    .width(60)
                    .font_size(14)
                    .text_align(kivact::style::TextAlign::End),
            ),
        )
        .child(
            column((
                text(title).style(style().weight(500)),
                text(description).style(style().font_size(12).color(ColorToken::TextMuted)),
            ))
            .style(style().grow(1)),
        )
        .child(on_details.map(|callback| {
            button(message("pop.agenda.details", "Event details"))
                .disabled(disabled)
                .variant(ButtonVariant::Ghost)
                .hide_label(true)
                .icon(icon("chevron-right"))
                .on_press(move || callback.call(()))
                .style(style().size(28, 28))
        }))
        .into_view()
    }
}
