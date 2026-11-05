//! Role: Build an editable date and its keyboard-accessible picker.
use super::machine::{
    DateInputState, DateValidation, Draft, Event, OpenedBy, Snapshot, TypedDate, TypedEvent,
    TypedSnapshot,
};
use crate::layer::with_recipe_style;
use crate::variants::Variants;
use crate::{ButtonVariant, Calendar, button, calendar, dialog, text_input};
use kivact_core::custom::PartState;
use kivact_core::geometry::Edge;
use kivact_core::i18n::{CalendarDate, DateStyle, Weekday, message, use_locale};
use kivact_core::state::{Binding, IntoBinding, IntoText, IntoValue, Value, binding, machine};
use kivact_core::view::{
    Handled, IntoView, Key, KeyEvent, Node, NodeBuilder, View, icon, node, row,
};
use kivact_interaction::DialogMode;
use std::rc::Rc;
/// The surface used for date and time picking.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DatePresentation {
    #[default]
    Adaptive,
    Desktop,
    BottomSheet,
}
impl DatePresentation {
    pub(crate) fn sheet(self) -> bool {
        match self {
            Self::Adaptive => kivact_core::platform::capabilities().get().touch,
            Self::Desktop => false,
            Self::BottomSheet => true,
        }
    }
    pub(crate) fn mode(self) -> DialogMode {
        if self.sheet() {
            DialogMode::Drawer(Edge::Bottom)
        } else {
            DialogMode::Popover
        }
    }
}
/// An editable locale-sensitive date, created by [`date_input`].
pub struct DateInput {
    root: Node,
    label: Value<String>,
    value: Binding<Option<CalendarDate>>,
    format: Value<DateStyle>,
    min: Value<Option<CalendarDate>>,
    max: Value<Option<CalendarDate>>,
    week_start: Option<Value<Weekday>>,
    presentation: DatePresentation,
    configure: Option<Rc<dyn Fn(Calendar) -> Calendar>>,
}
/// Edit an optional date. Enter commits valid text; Arrow Down opens the picker.
pub fn date_input(
    label: impl IntoText,
    value: impl IntoBinding<Option<CalendarDate>>,
) -> DateInput {
    DateInput {
        root: node(),
        label: label.into_text_value(),
        value: value.into_binding(),
        format: DateStyle::Iso.into_value(),
        min: None.into_value(),
        max: None.into_value(),
        week_start: None,
        presentation: DatePresentation::Adaptive,
        configure: None,
    }
}
impl DateInput {
    /// Date text format, read reactively.
    pub fn format(mut self, format: impl IntoValue<DateStyle>) -> Self {
        self.format = format.into_value();
        self
    }
    /// Earliest accepted date.
    pub fn min(mut self, min: impl IntoValue<Option<CalendarDate>>) -> Self {
        self.min = min.into_value();
        self
    }
    /// Latest accepted date.
    pub fn max(mut self, max: impl IntoValue<Option<CalendarDate>>) -> Self {
        self.max = max.into_value();
        self
    }
    /// First column of the calendar.
    pub fn week_start(mut self, day: impl IntoValue<Weekday>) -> Self {
        self.week_start = Some(day.into_value());
        self
    }
    /// Choose the popup surface.
    pub fn presentation(mut self, presentation: DatePresentation) -> Self {
        self.presentation = presentation;
        self
    }
    /// Configure every calendar option, including range-independent constraints and marks.
    pub fn calendar(mut self, configure: impl Fn(Calendar) -> Calendar + 'static) -> Self {
        self.configure = Some(Rc::new(configure));
        self
    }
}
impl NodeBuilder for DateInput {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for DateInput {
    fn into_view(self) -> View {
        let Self {
            root,
            label,
            value,
            format,
            min,
            max,
            week_start,
            presentation,
            configure,
        } = self;
        let locale = use_locale();
        let initial = value.get_untracked().map_or(Draft::Empty, Draft::Valid);
        let state = machine(
            DateInputState::Closed { draft: initial },
            move || Snapshot {
                locale: locale.get(),
                min: min.get(),
                max: max.get(),
            },
            move |_, next| value.set(next),
        );
        let open = binding(
            move || state.get().is_open(),
            move |open| {
                if open {
                    state.send(Event::Open(OpenedBy::Button))
                } else {
                    state.send(Event::Close)
                }
            },
        );
        let typed = machine(
            value.get_untracked().map_or(
                TypedDate::Editing {
                    text: String::new(),
                    validation: DateValidation::Unchecked,
                },
                TypedDate::Following,
            ),
            move || TypedSnapshot {
                locale: locale.get(),
                min: min.get(),
                max: max.get(),
            },
            move |_, text| state.send(Event::Commit(text)),
        );
        kivact_core::state::effect(move || {
            let value = value.get();
            typed.send(TypedEvent::External(value));
            state.send(Event::External(value));
        });
        let draft = binding(
            move || match typed.get() {
                TypedDate::Following(date) => locale.get().format_date(date, format.get()),
                TypedDate::Editing { text, .. } => text,
            },
            move |text: String| typed.send(TypedEvent::Edit(text)),
        );
        let field = text_input(label, draft)
            .placeholder("YYYY-MM-DD")
            .invalid(move || matches!(typed.get(), TypedDate::Editing { ref text, validation: DateValidation::Checked } if !text.trim().is_empty() || matches!(state.get().draft(), Draft::Invalid(_))))
            .forward_keys(vec![Key::ArrowDown])
            .on_key(move |event: KeyEvent| { if event.key() == Key::ArrowDown { state.send(Event::Open(OpenedBy::Input)); Handled::Yes } else { Handled::No } })
            .on_draft_commit(move |text| typed.send(TypedEvent::Commit(text)));
        let toggle = button(message("date.open", "Choose date"))
            .variant(ButtonVariant::Ghost)
            .hide_label(true)
            .icon(icon("calendar"))
            .on_press(move || state.send(Event::Open(OpenedBy::Button)));
        let trigger = with_recipe_style(
            row((field, toggle)),
            |r| r.date_input_recipes().root_recipe(),
            &Variants::default(),
            PartState::default,
        );
        let content = move || {
            let selected = binding(value, move |next| match next {
                Some(date) => state.send(Event::Pick(date)),
                None => state.send(Event::Clear),
            });
            let mut picker = calendar(selected).min(min).max(max).autofocus(true);
            if let Some(day) = week_start {
                picker = picker.week_start(day);
            }
            if let Some(configure) = &configure {
                picker = configure(picker);
            }
            if presentation.sheet() {
                crate::date_wheel::date_panel(selected, picker).into_view()
            } else {
                picker.into_view()
            }
        };
        root.child(
            dialog(label, open)
                .mode(presentation.mode())
                .hide_title(true)
                .content(content)
                .attach(trigger),
        )
        .into_view()
    }
}
