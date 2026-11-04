//! Role: Reusable dashboard, navigation, conversation and scheduling patterns.

use crate::{Hue, Tokens};
use kivact::i18n::message;
use kivact::view::{Node, NodeBuilder, column, grid, icon, node, row, text};

use kivact::state::{Callback, Value};
use kivact::state::{IntoText, IntoValue};
use kivact::style::use_theme;
use kivact::style::{Align, Color, ColorToken, Direction, Justify, Track, style};
use kivact::ui::{ButtonVariant, button};
use kivact::view::{IntoView, View, show};

#[track_caller]
pub fn breadcrumb(items: impl IntoValue<Vec<String>>) -> impl kivact::view::Component {
    let items = items.into_value();
    row(kivact::view::dynamic(items, |items| {
        let count = items.len();
        let mut nodes = Vec::new();
        for (i, part) in items.into_iter().enumerate() {
            if i > 0 {
                nodes.push(icon("chevron-right").into_view());
            }
            nodes.push(
                text(part)
                    .style(style().color(if i + 1 == count {
                        ColorToken::Text
                    } else {
                        ColorToken::TextMuted
                    }))
                    .into_view(),
            );
        }
        nodes
    }))
    .style(style().gap(6).align_items(Align::Center).wrap(true))
}

/// Diamond stepper. Created by [`stepper`].
pub struct Stepper {
    root: Node,
    labels: Value<Vec<String>>,
    notes: Value<Vec<String>>,
    current: Value<usize>,
}

/// Diamond stepper over `labels`. The application supplies and changes its
/// current phase. [`Stepper::notes`] adds per-phase dates or status notes.
#[track_caller]
pub fn stepper(labels: impl IntoValue<Vec<String>>, current: impl IntoValue<usize>) -> Stepper {
    let current = current.into_value();
    let labels = labels.into_value();
    Stepper {
        root: grid(())
            .style(move || {
                style()
                    .grid_template_columns(Track::repeat(
                        labels.with(Vec::len) as u16,
                        Track::fr(1),
                    ))
                    .gap(0)
            })
            .draw(move |d, size| {
                let count = labels.with(Vec::len);
                let width = size.width / count.max(1) as f64;
                for index in 1..count {
                    d.line(
                        kivact::geometry::Point::new((index as f64 - 0.5) * width, 15.0),
                        kivact::geometry::Point::new((index as f64 + 0.5) * width, 15.0),
                        2.0,
                        if index <= current.get() {
                            ColorToken::Accent
                        } else {
                            ColorToken::Border
                        },
                    );
                }
            }),
        labels,
        notes: Vec::new().into_value(),
        current,
    }
}
impl Stepper {
    /// Per-phase notes, such as dates or statuses; a missing note is empty.
    pub fn notes(mut self, notes: impl IntoValue<Vec<String>>) -> Self {
        self.notes = notes.into_value();
        self
    }
}
impl NodeBuilder for Stepper {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Stepper {
    fn into_view(self) -> View {
        let Self {
            root,
            labels,
            notes,
            current,
        } = self;
        root.child(kivact::view::for_index(labels, move |i, label| {
            column((
                node()
                    .style(
                        style()
                            .size(30, 30)
                            .radius(6)
                            .rotate(45)
                            .direction(Direction::Row)
                            .align_items(Align::Center)
                            .justify_content(Justify::Center)
                            .margin_bottom(6),
                    )
                    .style(move || {
                        let t = &use_theme().get();
                        let p = t.ext::<Tokens>();
                        if i < current.get() {
                            style()
                                .background(p.blue)
                                .border(1, p.blue)
                                .color(Color::WHITE)
                        } else if i == current.get() {
                            style()
                                .background(p.surface)
                                .border(1, p.blue)
                                .color(p.blue)
                                .ring(4, p.hue(Hue::Blue).0)
                        } else {
                            style()
                                .background(p.surface)
                                .border(1, p.line_strong)
                                .color(p.text_muted)
                        }
                    })
                    .child(
                        text(move || {
                            if i < current.get() {
                                "✓".into()
                            } else {
                                (i + 1).to_string()
                            }
                        })
                        .style(style().rotate(-45).font_size(13).weight(600)),
                    ),
                text(label).style(style().font_size(13).weight(500)),
                text(move || notes.with(|notes| notes.get(i).cloned().unwrap_or_default()))
                    .style(style().font_size(12).color(ColorToken::TextMuted)),
            ))
            .style(style().align_items(Align::Center).gap(2))
            .into_view()
        }))
        .into_view()
    }
}

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
impl NodeBuilder for AgendaItem {
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
