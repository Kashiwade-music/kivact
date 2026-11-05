//! Role: Date and wall-clock wheels share scalar keyboard, pointer and assistive editing.
use crate::layer::with_recipe_style;
use crate::variants::Variants;
use kivact_core::custom::PartState;
use kivact_core::i18n::{CalendarDate, TimeOfDay, message};
use kivact_core::state::{Binding, IntoBinding, Value, binding, notifier, signal};
use kivact_core::view::{EditPhase, Handled, Node, NodeBuilder, Orientation, Role, WheelEvent, column, node, row, text};
use kivact_interaction::{ScalarRange, use_scalar};
/// A single discrete wheel; the scalar owns drag cancellation and bounds.
pub(crate) fn wheel(label: Value<String>, value: Binding<f64>, min: f64, max: f64) -> Node {
    let release = notifier();
    let range = ScalarRange::new(min, max).expect("invariant: picker bounds are ordered").with_step(1.0).expect("invariant: one is a positive step");
    let scalar = use_scalar(value, range).on_edit(move |phase| { if matches!(phase, EditPhase::Commit | EditPhase::Cancel) { release.notify(); } });
    let control = node().part(scalar.input()).name(label).focusable(true)
        .child(text(move || { release.track(); format!("{:02}", scalar.value().get() as u32) }))
        .on_wheel(move |event: WheelEvent| {
            let delta = event.delta().y;
            if !delta.is_finite() || delta == 0.0 { return Handled::No; }
            scalar.set(value.get_untracked() + delta.signum()); release.notify(); Handled::Yes
        });
    with_recipe_style(node().part(scalar.track(Orientation::Vertical)).with_node(|n| n.role(Role::Group)).child(control), |r| r.date_input_recipes().wheel_recipe(), &Variants::default(), PartState::default)
}
pub(crate) fn date_panel(value: Binding<Option<CalendarDate>>, picker: crate::Calendar) -> Node {
    let date = signal(value.get_untracked().unwrap_or_else(CalendarDate::today));
    let year = binding(move || f64::from(date.get().year()), move |year| { let current = date.get_untracked(); date.set(current.add_months((year as i32 - current.year()) * 12)); });
    let month = binding(move || f64::from(date.get().month()), move |month| { let current = date.get_untracked(); date.set(current.add_months(month as i32 - i32::from(current.month()))); });
    let day = binding(move || f64::from(date.get().day()), move |day| { let current = date.get_untracked(); if let Some(next) = CalendarDate::checked(current.year(), current.month(), day as u8) { date.set(next); } });
    column((row((wheel(message("date.year", "Year"), year, 1.0, 9999.0), wheel(message("date.month", "Month"), month, 1.0, 12.0), wheel(message("date.day", "Day"), day, 1.0, f64::from(date.get_untracked().month_of().days())))), picker, crate::button(message("date.apply", "Apply date")).on_press(move || value.set(Some(date.get_untracked())))))
}
pub(crate) fn time_panel(value: Binding<Option<TimeOfDay>>, seconds: Value<bool>) -> Node {
    let time = signal(value.get_untracked().unwrap_or(TimeOfDay { hour: 0, minute: 0, second: 0 }));
    let hour = binding(move || f64::from(time.get().hour), move |hour| { let mut next = time.get_untracked(); next.hour = hour as u8; time.set(next); });
    let minute = binding(move || f64::from(time.get().minute), move |minute| { let mut next = time.get_untracked(); next.minute = minute as u8; time.set(next); });
    let second = binding(move || f64::from(time.get().second), move |second| { let mut next = time.get_untracked(); next.second = second as u8; time.set(next); });
    let wheels = kivact_core::view::dynamic(move || seconds.get(), move |show| {
        let mut wheels = vec![wheel(message("time.hour", "Hour"), hour, 0.0, 23.0), wheel(message("time.minute", "Minute"), minute, 0.0, 59.0)];
        if show { wheels.push(wheel(message("time.second", "Second"), second, 0.0, 59.0)); } row(wheels)
    });
    column((wheels, crate::button(message("time.apply", "Apply time")).on_press(move || value.set(Some(time.get_untracked())))))
}
