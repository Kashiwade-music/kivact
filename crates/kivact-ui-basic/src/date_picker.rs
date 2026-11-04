//! Role: Date pickers: an outline button that opens a popover with a calendar.

use crate::button::{Look, button_style};
use crate::calendar::{Calendar, Names, calendar, calendar_range};
use crate::parts::pressable;
use crate::theme_access::tokens;
use kivact::i18n::{CalendarDate, DateRange, interpolate, message};
use kivact::state::IntoBinding;
use kivact::state::{Binding, Value, memo, signal};
use kivact::state::{IntoText, IntoValue};
use kivact::style::{Color, Justify, pct, style};
use kivact::ui::ControlSize;
use kivact::view::icon;
use kivact::view::{IntoView, Role, View, node_ref};
use kivact::view::{Node, NodeBuilder, node, text};
use std::rc::Rc;

type Format = Rc<dyn Fn(CalendarDate) -> String>;

/// Shared look and behavior of the single and the range picker.
struct Picker {
    root: Node,
    label: Value<String>,
    placeholder: Option<Value<String>>,
    format: Option<Format>,
}

impl Picker {
    fn new(label: impl IntoText) -> Self {
        let label = label.into_text_value();
        let (root, state) = pressable(
            node().with_node(|root| root.role(Role::Button)).name(label),
            Role::Button,
            None,
        );
        // The width is the content's: give the trigger a `.style(..)` width to
        // fix it (the shadcn demo uses 280 px).
        let root = root.style(move || {
            let p = tokens();
            button_style(&p, Look::Outline, ControlSize::Medium, &state.get())
                .justify_content(Justify::Start)
                .weight(400)
                .shrink(0)
                .max_width(pct(100))
        });
        Picker {
            root,
            label,
            placeholder: None,
            format: None,
        }
    }

    /// The trigger and the popover around the calendar `make` builds.
    fn view(
        self,
        shown: impl Fn(&Rc<Names>, &Format) -> Value<Option<String>> + 'static,
        make: impl Fn() -> Calendar + 'static,
    ) -> View {
        let Picker {
            root,
            label,
            placeholder,
            format,
        } = self;
        let names = Rc::new(Names::new());
        // The date format is a message template, `{month} {day}, {year}` in English.
        let template = message("basic.date_picker.format", "{month} {day}, {year}");
        let format: Format = format.unwrap_or_else(|| {
            let names = names.clone();
            Rc::new(move |d: CalendarDate| {
                interpolate(
                    &template.get(),
                    &[
                        ("month", names.month(d.month()).get().as_str()),
                        ("day", d.day().to_string().as_str()),
                        ("year", d.year().to_string().as_str()),
                    ],
                )
            })
        });
        let text_value = shown(&names, &format);
        let placeholder = placeholder.unwrap_or_else(|| {
            kivact::i18n::message("basic.date_picker.placeholder", "Pick a date")
        });
        let open = signal(false);
        let trigger_ref = node_ref();
        let trigger = root.node_ref(trigger_ref).child(icon("calendar")).child(
            text(move || text_value.get().unwrap_or_else(|| placeholder.get())).style(move || {
                let p = tokens();
                style().color(if text_value.get().is_some() {
                    p.foreground
                } else {
                    p.muted_foreground
                })
            }),
        );
        // popover.tsx with `w-auto p-0`, at the start of the button.
        kivact::ui::dialog(label, open)
            .mode(kivact::interaction::DialogMode::Popover)
            .hide_title(true)
            .width(kivact::style::Length::Auto)
            .variant(crate::recipes_dialog::Flush)
            .content(move || {
                make()
                    .autofocus(true)
                    .on_commit(move || {
                        open.set(false);
                        // A press on a day leaves no focus inside the panel to restore.
                        trigger_ref.restore_focus();
                    })
                    .style(style().background(Color::TRANSPARENT))
            })
            .attach(trigger)
    }
}

/// A button that shows a date and opens a calendar (`DatePicker`).
///
/// Created by [`date_picker`]. The popover closes after a day is chosen and
/// gives the focus back to the button. The button is as wide as its text; give
/// it a width with `.style(..)`. The default format is the message
/// `date_picker.format` (`{month} {day}, {year}`).
pub struct DatePicker {
    picker: Picker,
    value: Binding<Option<CalendarDate>>,
    calendar: Option<Rc<dyn Fn(Calendar) -> Calendar>>,
}

/// A date picker named `label` over `value`.
///
/// ```no_run
/// use kivact::i18n::CalendarDate;
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn birthday() -> impl IntoView {
///     let value = signal(None::<CalendarDate>);
///     basic::date_picker("Date of birth", value).placeholder("Pick a date")
/// }
/// ```
#[track_caller]
pub fn date_picker(
    label: impl IntoText,
    value: impl kivact::state::IntoBinding<Option<CalendarDate>>,
) -> DatePicker {
    DatePicker {
        picker: Picker::new(label),
        value: value.into_binding(),
        calendar: None,
    }
}

impl DatePicker {
    /// Text shown while no date is chosen (default `Pick a date`).
    pub fn placeholder(mut self, text: impl IntoText) -> Self {
        self.picker.placeholder = Some(text.into_text_value());
        self
    }

    /// How the chosen date is written (default `October 3, 2026`).
    pub fn format(mut self, f: impl Fn(CalendarDate) -> String + 'static) -> Self {
        self.picker.format = Some(Rc::new(f));
        self
    }

    /// Set up the calendar that opens, such as its disabled dates.
    pub fn calendar(mut self, f: impl Fn(Calendar) -> Calendar + 'static) -> Self {
        self.calendar = Some(Rc::new(f));
        self
    }
}

impl NodeBuilder for DatePicker {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.picker.root
    }
}

impl IntoView for DatePicker {
    fn into_view(self) -> View {
        let DatePicker {
            picker,
            value,
            calendar: configure,
        } = self;
        let current = value.into_value();
        picker.view(
            move |_, format| {
                let format = format.clone();
                memo(move || current.get().map(&*format))
            },
            move || {
                let base = calendar(value);
                match &configure {
                    Some(configure) => configure(base),
                    None => base,
                }
            },
        )
    }
}

/// A button that shows a range of dates and opens a calendar
/// (`DatePicker` with a range).
///
/// Created by [`date_range_picker`]; the popover closes after the second
/// click. The two ends are joined by the message `date_picker.range`
/// (`{start} - {end}`).
pub struct DateRangePicker {
    picker: Picker,
    value: Binding<Option<DateRange>>,
    calendar: Option<Rc<dyn Fn(Calendar) -> Calendar>>,
}

/// A range picker named `label` over `value`.
#[track_caller]
pub fn date_range_picker(
    label: impl IntoText,
    value: impl kivact::state::IntoBinding<Option<DateRange>>,
) -> DateRangePicker {
    DateRangePicker {
        picker: Picker::new(label),
        value: value.into_binding(),
        calendar: None,
    }
}

impl DateRangePicker {
    /// Text shown while no range is chosen (default `Pick a date`).
    pub fn placeholder(mut self, text: impl IntoText) -> Self {
        self.picker.placeholder = Some(text.into_text_value());
        self
    }

    /// How each end is written (default `October 3, 2026`); the range reads
    /// `start - end`.
    pub fn format(mut self, f: impl Fn(CalendarDate) -> String + 'static) -> Self {
        self.picker.format = Some(Rc::new(f));
        self
    }

    /// Set up the calendar that opens, such as the number of months.
    pub fn calendar(mut self, f: impl Fn(Calendar) -> Calendar + 'static) -> Self {
        self.calendar = Some(Rc::new(f));
        self
    }
}

impl NodeBuilder for DateRangePicker {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.picker.root
    }
}

impl IntoView for DateRangePicker {
    fn into_view(self) -> View {
        let DateRangePicker {
            picker,
            value,
            calendar: configure,
        } = self;
        let current = value.into_value();
        let separator = message("basic.date_picker.range", "{start} - {end}");
        picker.view(
            move |_, format| {
                let format = format.clone();
                memo(move || {
                    current.get().map(|r| {
                        if r.start() == r.end() {
                            format(r.start())
                        } else {
                            interpolate(
                                &separator.get(),
                                &[
                                    ("start", format(r.start()).as_str()),
                                    ("end", format(r.end()).as_str()),
                                ],
                            )
                        }
                    })
                })
            },
            move || {
                let base = calendar_range(value);
                match &configure {
                    Some(configure) => configure(base),
                    None => base,
                }
            },
        )
    }
}
