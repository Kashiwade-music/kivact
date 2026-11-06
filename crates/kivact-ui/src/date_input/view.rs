//! Role: Build an editable date and its keyboard-accessible picker.
use super::machine::{
    DateInputState, DateValidation, Draft, Event, OpenedBy, Snapshot, TypedDate, TypedEvent,
    TypedSnapshot,
};
use crate::layer::with_recipe_style;
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
    variants: crate::variants::Variants,
    label: Value<String>,
    picker_label: Option<Value<String>>,
    value: Binding<Option<CalendarDate>>,
    format: Value<DateStyle>,
    custom_format: Option<Rc<dyn Fn(CalendarDate) -> String>>,
    placeholder: Value<String>,
    valid: Option<Binding<bool>>,
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
        variants: crate::variants::Variants::default(),
        label: label.into_text_value(),
        picker_label: None,
        value: value.into_binding(),
        format: DateStyle::Iso.into_value(),
        custom_format: None,
        placeholder: "YYYY-MM-DD".into_text_value(),
        valid: None,
        min: None.into_value(),
        max: None.into_value(),
        week_start: None,
        presentation: DatePresentation::Adaptive,
        configure: None,
    }
}
impl DateInput {
    /// Name the separate picker button independently of the text editor.
    pub fn picker_label(mut self, label: impl IntoText) -> Self {
        self.picker_label = Some(label.into_text_value());
        self
    }

    /// Add a visual variant shared by the field and its picker surface.
    pub fn variant(self, variant: impl crate::Variant) -> Self {
        self.variants.add(variant);
        self
    }
    pub(crate) fn with_variants(self, variants: &crate::variants::Variants) -> Self {
        self.variants.merge(variants);
        self
    }
    /// Date text format, read reactively.
    pub fn format(mut self, format: impl IntoValue<DateStyle>) -> Self {
        self.format = format.into_value();
        self.custom_format = None;
        self
    }
    /// Custom application presentation for an existing date.
    pub fn format_with(mut self, format: impl Fn(CalendarDate) -> String + 'static) -> Self {
        self.custom_format = Some(Rc::new(format));
        self
    }
    /// Text displayed while the optional value is empty.
    pub fn placeholder(mut self, text: impl IntoText) -> Self {
        self.placeholder = text.into_text_value();
        self
    }
    /// Publish whether the current typed draft is a valid optional date.
    pub fn valid(mut self, valid: impl IntoBinding<bool>) -> Self {
        self.valid = Some(valid.into_binding());
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
        self.render(crate::date_trigger::FieldPresentation::Text)
    }
}
impl DateInput {
    /// Open the configured picker from a caller-owned button, without a text editor.
    /// The trigger keeps its handlers, focus identity and accessible name.
    pub fn attach<T: NodeBuilder + IntoView + 'static>(self, trigger: T) -> View {
        self.render(crate::date_trigger::FieldPresentation::trigger(trigger))
    }
    fn render(self, field: crate::date_trigger::FieldPresentation) -> View {
        if let crate::date_trigger::FieldPresentation::Trigger(attach) = field {
            return self.render_trigger(attach);
        }
        let Self {
            mut root,
            variants,
            label,
            picker_label,
            value,
            format,
            custom_format,
            placeholder,
            valid,
            min,
            max,
            week_start,
            presentation,
            configure,
        } = self;
        let external_invalid = root.accessible_invalid();
        let described_by = root.accessible_described_by();
        let locale = use_locale();
        let (state, announcement) = date_controller(value, valid, min, max, locale);
        let open = picker_open(state);
        let typed = typed_controller(value, valid, min, max, locale, state);
        kivact_core::state::effect(move || {
            let value = value.get();
            typed.send(TypedEvent::External(value));
            state.send(Event::External(value));
        });
        let draft = draft_binding(typed, format, custom_format.clone(), locale);
        let error_ref = kivact_core::view::node_ref();
        let error = date_diagnostic(typed.state(), locale, error_ref);
        let input_ref = kivact_core::view::node_ref();
        let button_ref = kivact_core::view::node_ref();
        let return_focus = kivact_core::state::memo(move || match state.get() {
            DateInputState::Open {
                opened_by: OpenedBy::Input,
                ..
            } => input_ref,
            _ => button_ref,
        });
        let field = date_editor(
            EditorInputs {
                label,
                placeholder,
                value,
                custom_format,
                external_invalid,
                described_by,
            },
            typed,
            state,
            draft,
            error_ref,
        )
        .node_ref(input_ref)
        .with_node(|node| node.expanded(open));
        let toggle = button(picker_label.unwrap_or_else(|| message("date.open", "Choose date")))
            .variant(ButtonVariant::Ghost)
            .hide_label(true)
            .icon(icon("calendar"))
            .node_ref(button_ref);
        let content = picker_content(value, state, min, max, week_start, presentation, configure);
        let picker = dialog(message("date.open", "Choose date"), open)
            .with_variants(&variants)
            .mode(presentation.mode())
            .return_focus(return_focus)
            .hide_title(true)
            .content(move || content.call(()))
            .attach(toggle);
        date_layout(root, field, picker, error, announcement, &variants)
    }
}

impl DateInput {
    fn render_trigger(self, attach: crate::date_trigger::AttachTrigger) -> View {
        let Self {
            root,
            variants,
            label,
            value,
            valid,
            min,
            max,
            week_start,
            presentation,
            configure,
            ..
        } = self;
        let open = kivact_core::state::signal(false);
        let focus = attach.focus;
        let content = move || {
            let selected = binding(value, move |next| {
                value.set(next);
                if let Some(valid) = valid {
                    valid.set(true);
                }
                if !presentation.sheet() {
                    open.set(false);
                    focus.restore_focus();
                }
            });
            let mut picker = calendar(selected).min(min).max(max).autofocus(true);
            if let Some(day) = week_start {
                picker = picker.week_start(day);
            }
            if let Some(configure) = &configure {
                picker = configure(picker);
            }
            if presentation.sheet() {
                kivact_core::view::column((
                    crate::date_sheet::panel(selected, picker),
                    button(message("date.save", "Save")).on_press(move || {
                        open.set(false);
                        focus.restore_focus();
                    }),
                ))
                .into_view()
            } else {
                picker.into_view()
            }
        };
        root.child(
            attach.view(
                dialog(label, open)
                    .with_variants(&variants)
                    .mode(presentation.mode())
                    .hide_title(true)
                    .content(content),
            ),
        )
        .into_view()
    }
}

fn checked_invalid(state: &TypedDate) -> bool {
    matches!(state, TypedDate::Editing { text, validation: DateValidation::Checked } if !text.trim().is_empty())
}
fn date_diagnostic(
    typed: Value<TypedDate>,
    locale: Value<kivact_core::i18n::Locale>,
    reference: kivact_core::view::NodeRef,
) -> Node {
    let format = message("date.error.format", "Use YYYY-MM-DD");
    let missing = message("date.error.invalid", "This date does not exist");
    let bounds = message("date.error.bounds", "Date is outside the allowed range");
    kivact_core::view::text(move || match typed.get() {
        TypedDate::Editing {
            text,
            validation: DateValidation::Checked,
        } if !text.trim().is_empty() => {
            if locale.get().parse_date(&text).is_some() {
                bounds.get()
            } else if iso_shaped(&text) {
                missing.get()
            } else {
                format.get()
            }
        }
        _ => String::new(),
    })
    .node_ref(reference)
    .role(kivact_core::view::Role::Status)
    .live_region(kivact_core::view::LiveRegion::Polite)
    .hidden(move || !checked_invalid(&typed.get()))
    .style(
        kivact_core::style::style()
            .font_size(12)
            .color(kivact_core::style::ColorToken::Danger),
    )
}
fn iso_shaped(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() == 10
        && matches!(bytes[4], b'-' | b'/')
        && bytes[7] == bytes[4]
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit())
}

/// Own committed date editing and picker visibility in the canonical controller.
fn date_controller(
    value: Binding<Option<CalendarDate>>,
    valid: Option<Binding<bool>>,
    min: Value<Option<CalendarDate>>,
    max: Value<Option<CalendarDate>>,
    locale: Value<kivact_core::i18n::Locale>,
) -> (kivact_core::state::MachineHandle<Snapshot>, Value<String>) {
    let announcement = kivact_core::state::signal(String::new());
    let chosen = message("calendar.chosen", "Date selected");
    let initial = value.get_untracked().map_or(Draft::Empty, Draft::Valid);
    let controller = machine(
        DateInputState::Closed { draft: initial },
        move || Snapshot {
            locale: locale.get(),
            min: min.get(),
            max: max.get(),
        },
        move |_, next| {
            value.set(next);
            if let Some(date) = next {
                announcement.set(format!("{}: {date}", chosen.get_untracked()));
            }
            if let Some(valid) = valid {
                valid.set(true);
            }
        },
    );
    (controller, announcement.into_value())
}
/// Bind picker visibility to the controller's open/close transitions.
fn picker_open(state: kivact_core::state::MachineHandle<Snapshot>) -> Binding<bool> {
    binding(
        move || state.get().is_open(),
        move |open| {
            if open {
                state.send(Event::Open(OpenedBy::Button))
            } else {
                state.send(Event::Close)
            }
        },
    )
}
/// Own the pending text and its checked validation through the typed-date machine.
fn typed_controller(
    value: Binding<Option<CalendarDate>>,
    valid: Option<Binding<bool>>,
    min: Value<Option<CalendarDate>>,
    max: Value<Option<CalendarDate>>,
    locale: Value<kivact_core::i18n::Locale>,
    state: kivact_core::state::MachineHandle<Snapshot>,
) -> kivact_core::state::MachineHandle<TypedSnapshot> {
    machine(
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
        move |_, effect| match effect {
            super::machine::TypedEffect::Commit(text) => state.send(Event::Commit(text)),
            super::machine::TypedEffect::Validity(next) => {
                if let Some(valid) = valid {
                    valid.set(next);
                }
            }
        },
    )
}
/// Compose the configured calendar or date-wheel presentation.
fn date_picker(
    selected: Binding<Option<CalendarDate>>,
    min: Value<Option<CalendarDate>>,
    max: Value<Option<CalendarDate>>,
    week_start: Option<Value<Weekday>>,
    presentation: DatePresentation,
    configure: Option<&Rc<dyn Fn(Calendar) -> Calendar>>,
) -> View {
    let mut picker = calendar(selected).min(min).max(max).autofocus(true);
    if let Some(day) = week_start {
        picker = picker.week_start(day);
    }
    if let Some(configure) = configure {
        picker = configure(picker);
    }
    if presentation.sheet() {
        crate::date_sheet::panel(selected, picker).into_view()
    } else {
        picker.into_view()
    }
}

type DateFormat = Rc<dyn Fn(CalendarDate) -> String>;
// xtask-allow(state-fields): Independent text-editor construction inputs and caller-owned semantic references; typed-date lifecycle belongs to its machine.
struct EditorInputs {
    label: Value<String>,
    placeholder: Value<String>,
    value: Binding<Option<CalendarDate>>,
    custom_format: Option<DateFormat>,
    external_invalid: Option<Value<bool>>,
    described_by: Vec<kivact_core::view::NodeRef>,
}
/// Render the date editor with its canonical typed draft, diagnostics and picker commands.
fn date_editor(
    inputs: EditorInputs,
    typed: kivact_core::state::MachineHandle<TypedSnapshot>,
    state: kivact_core::state::MachineHandle<Snapshot>,
    draft: Binding<String>,
    error_ref: kivact_core::view::NodeRef,
) -> crate::TextInput {
    let EditorInputs {
        label,
        placeholder,
        value,
        custom_format,
        external_invalid,
        described_by,
    } = inputs;
    let mut field = text_input(label, draft)
        .with_node(|node| node.described_by(error_ref))
        .placeholder(placeholder)
        .invalid(move || {
            let draft = typed.get();
            let editing = matches!(
                draft,
                TypedDate::Editing {
                    validation: DateValidation::Unchecked,
                    ..
                }
            );
            (!editing && external_invalid.is_some_and(|invalid| invalid.get()))
                || checked_invalid(&draft)
        })
        .forward_keys(vec![Key::ArrowDown])
        .on_key(move |event: KeyEvent| {
            if event.key() == Key::ArrowDown {
                state.send(Event::Open(OpenedBy::Input));
                Handled::Yes
            } else {
                Handled::No
            }
        })
        .on_draft_commit(move |text| {
            if let Some(date) = value.get_untracked().filter(|date| {
                custom_format
                    .as_ref()
                    .is_some_and(|format| format(*date) == text)
            }) {
                state.send(Event::Pick(date));
            } else {
                typed.send(TypedEvent::Commit(text));
            }
        });
    for reference in described_by {
        field = field.with_node(|node| node.described_by(reference));
    }
    field
}

/// Display the accepted date in its configured format while preserving pending typed text.
fn draft_binding(
    typed: kivact_core::state::MachineHandle<TypedSnapshot>,
    format: Value<DateStyle>,
    display: Option<DateFormat>,
    locale: Value<kivact_core::i18n::Locale>,
) -> Binding<String> {
    binding(
        move || match typed.get() {
            TypedDate::Following(date) => display.as_ref().map_or_else(
                || locale.get().format_date(date, format.get()),
                |format| format(date),
            ),
            TypedDate::Editing { text, .. } => text,
        },
        move |text: String| typed.send(TypedEvent::Edit(text)),
    )
}

fn picker_content(
    value: Binding<Option<CalendarDate>>,
    state: kivact_core::state::MachineHandle<Snapshot>,
    min: Value<Option<CalendarDate>>,
    max: Value<Option<CalendarDate>>,
    week_start: Option<Value<Weekday>>,
    presentation: DatePresentation,
    configure: Option<Rc<dyn Fn(Calendar) -> Calendar>>,
) -> kivact_core::view::ViewFn {
    kivact_core::view::ViewFn::from(move |()| {
        let selected = binding(value, move |next| match next {
            Some(date) => state.send(if presentation.sheet() {
                Event::SheetPick(date)
            } else {
                Event::Pick(date)
            }),
            None => state.send(Event::Clear),
        });
        let picker = date_picker(
            selected,
            min,
            max,
            week_start,
            presentation,
            configure.as_ref(),
        );
        if presentation.sheet() {
            kivact_core::view::column((
                picker,
                button(message("date.save", "Save")).on_press(move || state.send(Event::Close)),
            ))
            .into_view()
        } else {
            picker
        }
    })
}

fn date_layout(
    root: Node,
    field: crate::TextInput,
    picker: View,
    error: Node,
    announcement: Value<String>,
    variants: &crate::variants::Variants,
) -> View {
    root.child(with_recipe_style(
        row((field, picker)),
        |r| r.date_input_recipes().root_recipe(),
        variants,
        PartState::default,
    ))
    .child(error)
    .child(
        kivact_core::view::node()
            .role(kivact_core::view::Role::Status)
            .name(announcement)
            .live_region(kivact_core::view::LiveRegion::Polite),
    )
    .into_view()
}
