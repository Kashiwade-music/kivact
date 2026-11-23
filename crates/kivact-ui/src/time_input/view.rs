//! Role: Time field and its wheel dialog.
use crate::helpers::report_unnamed;
use crate::layer::with_recipe_style;
use crate::time_input::look::TimeInputTheme;
use crate::{DatePresentation, button, dialog, text_input};
use kivact_core::custom::{Caller, PartState};
use kivact_core::i18n::{TimeOfDay, TimeStyle, message, use_locale};
use kivact_core::state::{IntoValue, Signal, Value, binding};
use kivact_core::view::{Element, Node, Variants, node, row};
use kivact_interaction::{DateFieldApi, DateFieldOptions, DatePanel, TypedCheck, use_date_field};
use kivact_macros::component;

/// A wall-clock time field with hours and minutes, optionally seconds.
///
/// Edit optional wall-clock time, independently of date and time zone.
/// Name it with `label`. The field's accessible name is `label`, and the
/// title of the wheel dialog. Inside a [`field`](crate::field) the field's
/// label names it.
///
/// Setters: `label`, `picker_label` (the separate picker button's name),
/// `format` (its seconds), `min` and `max` (the accepted wall-clock times),
/// and `presentation` (popup or touch sheet).
#[component]
pub fn time_input(
    value: Signal<Option<TimeOfDay>>,
    label: Option<Value<String>>,
    picker_label: Option<Value<String>>,
    format: Option<Value<TimeStyle>>,
    #[prop(default = None::<TimeOfDay>)] min: Value<Option<TimeOfDay>>,
    #[prop(default = None::<TimeOfDay>)] max: Value<Option<TimeOfDay>>,
    #[prop(default = DatePresentation::Adaptive)] presentation: DatePresentation,
    #[caller] caller: Caller,
) -> impl Element {
    let named = label.is_some() || caller.name().is_some() || !caller.labelled_by().is_empty();
    report_unnamed(named, "time_input", caller.at());
    let seconds: Value<bool> = match format {
        Some(format) => (move || format.get().seconds).into_value(),
        None => false.into_value(),
    };
    let label = label.unwrap_or_else(|| String::new().into_value());
    time_field(
        value,
        label,
        picker_label,
        seconds,
        (min, max),
        presentation,
        &caller.variants(),
    )
}

/// The time field and its wheel dialog. `variants` are the look the field
/// takes: its own and those of the component it is built in (a date-time
/// field passes its own).
pub(crate) fn time_field(
    value: Signal<Option<TimeOfDay>>,
    label: Value<String>,
    picker_label: Option<Value<String>>,
    seconds: Value<bool>,
    bounds: (Value<Option<TimeOfDay>>, Value<Option<TimeOfDay>>),
    presentation: DatePresentation,
    variants: &Variants,
) -> Node {
    let locale = use_locale();
    let state = time_controller(value, seconds, bounds.0, bounds.1);
    let draft = state.editor(move |time| {
        locale.get().format_time(
            time,
            TimeStyle {
                seconds: seconds.get(),
            },
        )
    });
    let input_ref = kivact_core::view::node_ref();
    let button_ref = kivact_core::view::node_ref();
    let return_focus = state.picker().return_focus(button_ref);
    let open = state.picker().open(DatePanel::Time, button_ref);
    let field = text_input(draft)
        .label(label)
        .node_ref(input_ref)
        .with_node(|n| n.expanded(open))
        .placeholder("HH:MM")
        .invalid(move || state.is_invalid().get())
        .forward_keys(vec![kivact_core::view::Key::ArrowDown])
        .on_key(move |event: kivact_core::view::KeyEvent| {
            if event.key() == kivact_core::view::Key::ArrowDown {
                state.picker().show(DatePanel::Time, input_ref);
                kivact_core::view::Handled::Yes
            } else {
                kivact_core::view::Handled::No
            }
        })
        .on_commit(move |text| state.commit(text));
    let toggle = button(picker_label.unwrap_or_else(|| message("time.open", "Choose time")))
        .node_ref(button_ref);
    let picked = binding(value, move |next| value.set(next));
    node().child(with_recipe_style(
        row((
            field,
            dialog(label, open)
                .with_variants(variants)
                .mode(presentation.mode())
                .return_focus(return_focus)
                .hide_title(true)
                .content(move || {
                    kivact_core::view::column((
                        crate::date_parts::wheel::time_panel(picked, seconds),
                        button(message("time.save", "Save"))
                            .on_press(move || state.picker().hide()),
                    ))
                })
                .attach(toggle),
        )),
        |r: &TimeInputTheme| r.root_recipe(),
        variants,
        PartState::default,
    ))
}

fn time_controller(
    value: Signal<Option<TimeOfDay>>,
    seconds: Value<bool>,
    min: Value<Option<TimeOfDay>>,
    max: Value<Option<TimeOfDay>>,
) -> DateFieldApi<TimeOfDay> {
    let key = |time: TimeOfDay| (time.hour, time.minute, time.second);
    use_date_field(
        value,
        DateFieldOptions::new(
            move |text: &str| super::parse::parse_time(text, seconds.get()),
            move |time| {
                min.get().is_none_or(|min| key(time) >= key(min))
                    && max.get().is_none_or(|max| key(time) <= key(max))
            },
        )
        .check(TypedCheck::WhileTyping),
    )
}
