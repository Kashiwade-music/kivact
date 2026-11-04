//! Role: Render typed filters in table headers and commit validated drafts.
use super::filter_value::{FilterField, FilterUi};
use super::types::TypedFilters;
use crate::{ButtonVariant, ControlSize, button, checkbox, dialog, select, tag_input, text_input};
use kivact_core::custom::Sent;
use kivact_core::i18n::{CalendarDate, message, message_with};
use kivact_core::state::IntoValue;
use kivact_core::state::{Binding, Machine, Step, Value, binding, machine, memo, signal, var};
use kivact_core::style::{ColorToken, Justify, Length, pct, style};
use kivact_core::view::IntoView;
use kivact_core::view::NodeBuilder;
use kivact_core::view::{NodeRef, OverlayAlign, Placement, View, node_ref};
use kivact_core::view::{column, icon, node, row};
use kivact_interaction::ListItem;
use kivact_interaction::{DialogMode, FilterValue};

#[derive(Clone, Copy)]
pub(super) struct Filters {
    pub(super) values: TypedFilters,
    pub(super) ui: FilterUi,
    pub(super) page: Option<Binding<usize>>,
}

impl Filters {
    fn column(self, index: usize) -> Binding<Option<FilterValue>> {
        binding(
            move || self.values.with(|values| values.get(&index).cloned()),
            move |value| {
                let mut values = self.values.get_untracked();
                if values.get(&index) == value.as_ref() {
                    return;
                }
                if let Some(value) = value {
                    values.insert(index, value);
                } else {
                    values.remove(&index);
                }
                self.values.set(values);
                if let Some(page) = self.page {
                    page.set(0);
                }
            },
        )
    }

    pub(super) fn control(
        self,
        index: usize,
        field: Option<FilterField>,
        title: Value<String>,
        header: Option<NodeRef>,
    ) -> View {
        let value = self.column(index);
        let label = message_with(
            "data_table.filter.by_column",
            "Filter {header}",
            move || [("header", title.get())],
        );
        if self.ui == FilterUi::Popover {
            return field
                .map(|field| popover(field, label, value, header))
                .into_view();
        }
        let compact = self.ui == FilterUi::InlineCompact;
        let size = if compact {
            ControlSize::Small
        } else {
            ControlSize::Medium
        };
        match field {
            Some(field) => editor(field, label, value, size).0,
            None => node()
                .style(style().height(if compact { 28 } else { 32 }))
                .into_view(),
        }
    }
}

fn popover(
    field: FilterField,
    label: Value<String>,
    value: Binding<Option<FilterValue>>,
    header: Option<NodeRef>,
) -> View {
    let open = signal(false);
    let reference = node_ref();
    let focused = reference.focus_visible();
    let hovered = header.map(|header| header.hovered());
    let active = {
        let field = field.clone();
        memo(move || value.with(|value| value.as_ref().is_some_and(|value| field.accepts(value))))
    };
    let trigger = button(label)
        .variant(ButtonVariant::Ghost)
        .hide_label(true)
        .icon(icon("funnel").style(move || {
            let strong = active.get();
            style()
                .size(12, 12)
                .color(if strong {
                    ColorToken::Accent
                } else {
                    ColorToken::TextMuted
                })
                .opacity(
                    if strong || focused.get() || hovered.is_some_and(|hovered| hovered.get()) {
                        1.0
                    } else {
                        0.35
                    },
                )
        }))
        .node_ref(reference)
        .style(style().size(20, 20).min_width(0).min_height(0).padding(0));
    dialog(label, open)
        .mode(DialogMode::Popover)
        .placement(Placement::Below)
        .align(OverlayAlign::Start)
        .width(Length::from(240))
        .hide_title(true)
        .content(move || {
            let draft = signal(value.get_untracked());
            let (control, valid) = editor(
                field.clone(),
                label,
                kivact_core::state::IntoBinding::into_binding(draft),
                ControlSize::Medium,
            );
            let reset = button(message("data_table.filter.reset", "Reset"))
                .variant(ButtonVariant::Ghost)
                .size(ControlSize::Small)
                .on_press(move || {
                    value.set(None);
                    open.set(false);
                });
            let apply = button(message("data_table.filter.apply", "Apply"))
                .size(ControlSize::Small)
                .disabled(move || !valid.get())
                .on_press(move || {
                    value.set(draft.get_untracked());
                    open.set(false);
                });
            column((
                control,
                row((reset, apply)).style(style().justify_content(Justify::SpaceBetween).gap(8)),
            ))
            .style(style().gap(8))
        })
        .attach(trigger)
}

fn editor(
    field: FilterField,
    label: Value<String>,
    value: Binding<Option<FilterValue>>,
    size: ControlSize,
) -> (View, Value<bool>) {
    let view = match field {
        FilterField::Choice(options) => {
            let all = ListItem::new(None, message("data_table.filter.all", "All"));
            let options = std::iter::once(all)
                .chain(
                    options
                        .into_iter()
                        .map(|(key, label)| ListItem::new(Some(key), label)),
                )
                .collect::<Vec<_>>();
            let choice = binding(
                move || {
                    Some(value.with(|value| match value {
                        Some(FilterValue::Choice(key)) => Some(key.clone()),
                        _ => None,
                    }))
                },
                move |key: Option<Option<String>>| {
                    value.set(key.flatten().map(FilterValue::Choice))
                },
            );
            select(label, options, choice)
                .keep_trigger_focus(true)
                .size(size)
                .style(style().width(pct(100)))
                .into_view()
        }
        FilterField::Choices(options) => {
            let controls = options.into_iter().map(|(key, label)| {
                let observed = key.clone();
                let checked = binding(
                    move || value.with(|value| matches!(value, Some(FilterValue::Choices(keys)) if keys.contains(&observed))),
                    move |checked| {
                        let mut keys = match value.get_untracked() {
                            Some(FilterValue::Choices(keys)) => keys,
                            _ => Vec::new(),
                        };
                        if checked && !keys.contains(&key) {
                            keys.push(key.clone());
                        } else if !checked {
                            keys.retain(|existing| existing != &key);
                        }
                        value.set((!keys.is_empty()).then_some(FilterValue::Choices(keys)));
                    },
                );
                checkbox(label, checked)
            }).collect::<Vec<_>>();
            column(controls).style(style().gap(4)).into_view()
        }
        FilterField::Tokens => {
            let tags = memo(move || {
                value.with(|value| match value {
                    Some(FilterValue::Tokens(tags)) => tags.clone(),
                    _ => Vec::new(),
                })
            });
            tag_input(label, tags)
                .on_add(move |tag| {
                    let mut next = tags.get_untracked();
                    if !next.contains(&tag) {
                        next.push(tag);
                        value.set(Some(FilterValue::Tokens(next)));
                    }
                })
                .on_remove(move |index| {
                    let mut next = tags.get_untracked();
                    if index < next.len() {
                        next.remove(index);
                        value.set((!next.is_empty()).then_some(FilterValue::Tokens(next)));
                    }
                })
                .into_view()
        }
        field => return text_editor(field, label, value, size),
    };
    (view, true.into_value())
}

#[derive(Clone, PartialEq)]
enum FilterDraft {
    Valid {
        text: String,
        value: Option<FilterValue>,
    },
    Invalid {
        text: String,
    },
}
impl FilterDraft {
    fn text(&self) -> &str {
        match self {
            Self::Valid { text, .. } | Self::Invalid { text } => text,
        }
    }
}
enum FilterEvent {
    External(Option<FilterValue>),
    Typed(String),
}
struct FilterInput {
    field: FilterField,
    sent: Sent<Option<FilterValue>>,
}
impl Machine for FilterInput {
    type State = FilterDraft;
    type Event = FilterEvent;
    type Effect = Option<FilterValue>;
    fn transition(
        &self,
        _state: &FilterDraft,
        event: FilterEvent,
    ) -> Step<FilterDraft, FilterEvent, Option<FilterValue>> {
        match event {
            FilterEvent::External(value) if self.sent.get() == Some(&value) => Step::stay(),
            FilterEvent::External(value) => Step::to(FilterDraft::Valid {
                text: format_value(value.as_ref()),
                value,
            }),
            FilterEvent::Typed(text) => match parse_value(&self.field, &text) {
                Ok(value) => Step::to(FilterDraft::Valid {
                    text,
                    value: value.clone(),
                })
                .with(value),
                Err(()) => Step::to(FilterDraft::Invalid { text }),
            },
        }
    }
}
fn text_editor(
    field: FilterField,
    label: Value<String>,
    value: Binding<Option<FilterValue>>,
    size: ControlSize,
) -> (View, Value<bool>) {
    let initial = value.get_untracked();
    let mut last = Sent::new();
    last.update(initial.clone());
    let sent = var(last);
    let input = field.clone();
    let draft = machine(
        FilterDraft::Valid {
            text: format_value(initial.as_ref()),
            value: initial,
        },
        move || FilterInput {
            field: input.clone(),
            sent: sent.get(),
        },
        move |_, next| {
            sent.update(|sent| {
                sent.update(next.clone());
            });
            value.set(next);
        },
    );
    draft.feed(value.into_value(), FilterEvent::External);
    let valid = memo(move || matches!(draft.get(), FilterDraft::Valid { .. }));
    let hint = match field {
        FilterField::Date => "YYYY-MM-DD",
        FilterField::Time => "HH:MM",
        _ => "",
    };
    let draft = binding(
        move || draft.get().text().to_owned(),
        move |text| draft.send(FilterEvent::Typed(text)),
    );
    let view = text_input(label, draft)
        .size(size)
        .placeholder(hint)
        .invalid(move || !valid.get())
        .style(style().width(pct(100)))
        .into_view();
    (view, valid)
}

fn format_value(value: Option<&FilterValue>) -> String {
    match value {
        Some(FilterValue::Text(text)) => text.clone(),
        Some(FilterValue::Number(number)) => number.to_string(),
        Some(FilterValue::Date(date)) => {
            format!("{:04}-{:02}-{:02}", date.year(), date.month(), date.day())
        }
        Some(FilterValue::Time { hour, minute }) => format!("{hour:02}:{minute:02}"),
        _ => String::new(),
    }
}

fn parse_value(field: &FilterField, raw: &str) -> Result<Option<FilterValue>, ()> {
    let text = raw.trim();
    if text.is_empty() {
        return Ok(None);
    }
    let value = match field {
        FilterField::Text => FilterValue::Text(text.to_owned()),
        FilterField::Number { .. } => FilterValue::Number(text.parse().map_err(|_| ())?),
        FilterField::Date => {
            let (year, rest) = text.split_once('-').ok_or(())?;
            let (month, day) = rest.split_once('-').ok_or(())?;
            let date = CalendarDate::checked(
                year.parse().map_err(|_| ())?,
                month.parse().map_err(|_| ())?,
                day.parse().map_err(|_| ())?,
            )
            .ok_or(())?;
            FilterValue::Date(date)
        }
        FilterField::Time => {
            let (hour, minute) = text.split_once(':').ok_or(())?;
            FilterValue::Time {
                hour: hour.parse().map_err(|_| ())?,
                minute: minute.parse().map_err(|_| ())?,
            }
        }
        _ => return Err(()),
    };
    if field.accepts(&value) {
        Ok(Some(value))
    } else {
        Err(())
    }
}
