//! Role: Date range field and its two-month popover.
use crate::calendar::{Calendar, calendar_range, calendar_status};
use crate::recipes_field::frame;
use crate::theme_access::tokens;
use kivact::custom::PartState;
use kivact::i18n::{CalendarDate, DateRange, message};
use kivact::interaction::DialogMode;
use kivact::state::Binding;
use kivact::state::IntoBinding;
use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::state::memo;
use kivact::state::signal;
use kivact::style::Align;
use kivact::style::Direction;
use kivact::style::Justify;
use kivact::style::Length;
use kivact::style::pct;
use kivact::style::style;
use kivact::ui::ButtonVariant;
use kivact::ui::button;
use kivact::ui::dialog;
use kivact::view::IntoView;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::ReadingDirection;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::column;
use kivact::view::dynamic;
use kivact::view::icon;
use kivact::view::node;
use kivact::view::node_ref;
use kivact::view::row;
use kivact::view::text;
use kivact::view::use_direction;
use std::rc::Rc;

/// A named range evaluated whenever its preset is rendered or activated.
#[derive(Clone)]
pub struct DateRangePreset {
    label: Value<String>,
    get_range: Rc<dyn Fn() -> DateRange>,
}

impl PartialEq for DateRangePreset {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.get_range, &other.get_range)
    }
}

impl DateRangePreset {
    pub fn new(label: impl IntoText, get_range: impl Fn() -> DateRange + 'static) -> Self {
        Self {
            label: label.into_text_value(),
            get_range: Rc::new(get_range),
        }
    }
}

type DateConstraint = Rc<dyn Fn(CalendarDate) -> bool>;

pub struct DateRangeInput {
    root: Node,
    label: Value<String>,
    value: Binding<Option<DateRange>>,
    configure: Option<Rc<dyn Fn(Calendar) -> Calendar>>,
    presets: Option<Value<Vec<DateRangePreset>>>,
    min: Option<CalendarDate>,
    max: Option<CalendarDate>,
    constraints: Vec<DateConstraint>,
    min_span: Value<u32>,
    max_span: Value<Option<u32>>,
}

pub fn date_range_input(
    label: impl IntoText,
    value: impl IntoBinding<Option<DateRange>>,
) -> DateRangeInput {
    DateRangeInput {
        root: node(),
        label: label.into_text_value(),
        value: value.into_binding(),
        configure: None,
        presets: None,
        min: None,
        max: None,
        constraints: Vec::new(),
        min_span: 1.into_value(),
        max_span: None.into_value(),
    }
}

impl DateRangeInput {
    /// Show named quick selections beside the calendar.
    pub fn presets(mut self, presets: impl IntoValue<Vec<DateRangePreset>>) -> Self {
        self.presets = Some(presets.into_value());
        self
    }

    pub fn min(mut self, min: CalendarDate) -> Self {
        self.min = Some(min);
        self
    }

    pub fn max(mut self, max: CalendarDate) -> Self {
        self.max = Some(max);
        self
    }

    /// Require this predicate to accept both the selected and preset endpoints.
    pub fn date_constraint(mut self, constraint: impl Fn(CalendarDate) -> bool + 'static) -> Self {
        self.constraints.push(Rc::new(constraint));
        self
    }

    pub fn min_range_span(mut self, days: impl IntoValue<u32>) -> Self {
        self.min_span = days.into_value();
        self
    }

    pub fn max_range_span(mut self, days: impl IntoValue<u32>) -> Self {
        let days = days.into_value();
        self.max_span = (move || Some(days.get())).into_value();
        self
    }

    pub fn calendar(mut self, configure: impl Fn(Calendar) -> Calendar + 'static) -> Self {
        self.configure = Some(Rc::new(configure));
        self
    }
}

impl NodeBuilder for DateRangeInput {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for DateRangeInput {
    fn into_view(self) -> View {
        let Self {
            mut root,
            label,
            value,
            configure,
            presets,
            min,
            max,
            constraints,
            min_span,
            max_span,
        } = self;
        let disabled = root.accessible_disabled();
        let disabled = memo(move || disabled.as_ref().is_some_and(|read| read()));
        let open = signal(false);
        let announcement = signal(String::new());
        let chosen = message("soft.calendar.range_chosen", "Date range selected");
        let trigger_ref = node_ref();
        let placeholder = message("soft.date_range.placeholder", "Select date range");
        let shown = move || {
            value
                .get()
                .map(|range| format!("{} – {}", range.start(), range.end()))
                .unwrap_or_else(|| placeholder.get())
        };
        let trigger = button(label)
            .hide_label(true)
            .disabled(disabled)
            .variant(ButtonVariant::Ghost)
            .child(icon("calendar"))
            .child(text(shown))
            .node_ref(trigger_ref)
            .style(move || {
                let t = tokens();
                frame(&t, &PartState::default(), false)
                    .height(t.sizes.control_md)
                    .direction(Direction::Row)
                    .align_items(Align::Center)
                    .gap(8)
                    .padding_x(8)
                    .color(if value.get().is_some() {
                        t.colors.text_primary
                    } else {
                        t.colors.text_secondary
                    })
            });
        let picker = dialog(label, open)
            .mode(DialogMode::Popover)
            .hide_title(true)
            .width(Length::Auto)
            .content(move || {
                let finish = move || {
                    if let Some(range) = value.get_untracked() {
                        announcement.set(format!(
                            "{}: {} – {}",
                            chosen.get_untracked(),
                            range.start(),
                            range.end()
                        ));
                    }
                    open.set(false);
                    trigger_ref.restore_focus();
                };
                let calendar = calendar_range(value)
                    .number_of_months(2)
                    .bounds(min, max)
                    .range_span(min_span, max_span)
                    .autofocus(presets.is_none_or(|presets| presets.with(Vec::is_empty)))
                    .on_commit(finish);
                let calendar = match &configure {
                    Some(configure) => configure(calendar),
                    None => calendar,
                }
                .gate_disabled(disabled)
                .with_constraints({
                    let constraints = constraints.clone();
                    move |date| constraints.iter().all(|accept| accept(date))
                });
                let Some(presets) = presets else {
                    return calendar.into_view();
                };
                let validate = Rc::new(calendar.range_validator());
                let rtl = use_direction().get() == ReadingDirection::Rtl;
                let preset_label = message("soft.date_range_input.presets", "Preset date ranges");
                let sidebar = dynamic(
                    move || presets.get(),
                    move |items| {
                        if items.is_empty() {
                            return None;
                        }
                        let buttons = items
                            .into_iter()
                            .map(|preset| {
                                let range = memo({
                                    let get_range = preset.get_range.clone();
                                    move || get_range()
                                });
                                let current = memo(move || value.get() == Some(range.get()));
                                let enabled = memo({
                                    let validate = validate.clone();
                                    move || !disabled.get() && validate(range.get())
                                });
                                let validate = validate.clone();
                                button(preset.label)
                                    .variant(ButtonVariant::Ghost)
                                    .with_node(move |node| node.selected(current))
                                    .disabled(move || !enabled.get())
                                    .on_press(move || {
                                        let range = (preset.get_range)();
                                        if !disabled.get_untracked() && validate(range) {
                                            value.set(Some(range));
                                            finish();
                                        }
                                    })
                                    .style(move || {
                                        let t = tokens();
                                        let c = &t.colors;
                                        let s = style()
                                            .width(pct(100))
                                            .min_height(28)
                                            .padding_x(8)
                                            .padding_y(4)
                                            .radius(t.radius.element)
                                            .weight(400)
                                            .font_size(14)
                                            .line_height(20.0 / 14.0)
                                            .justify_content(if rtl {
                                                Justify::End
                                            } else {
                                                Justify::Start
                                            })
                                            .color(if enabled.get() {
                                                c.text_primary
                                            } else {
                                                c.text_disabled
                                            });
                                        if current.get() && enabled.get() {
                                            s.background(c.accent_muted)
                                                .color(c.accent)
                                                .hover(style().background(c.accent_muted))
                                        } else {
                                            s
                                        }
                                    })
                                    .into_view()
                            })
                            .collect::<Vec<_>>();
                        Some(column(buttons).role(Role::Group).name(preset_label).style(
                            move || {
                                let t = tokens();
                                let s = style().min_width(140).padding(12).gap(4);
                                if rtl {
                                    s.border_left(1, t.colors.border_emphasized)
                                } else {
                                    s.border_right(1, t.colors.border_emphasized)
                                }
                            },
                        ))
                    },
                );
                row((sidebar, calendar))
                    .style(style().direction(if rtl {
                        Direction::RowReverse
                    } else {
                        Direction::Row
                    }))
                    .into_view()
            })
            .attach(trigger);
        root.child(calendar_status(announcement))
            .child(picker)
            .into_view()
    }
}
