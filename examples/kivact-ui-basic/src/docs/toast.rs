//! Catalog page of Toast: pushed notifications, variants, actions and durations.

use catalog_shell::{Example, Page};
use kivact::interaction::{Tone, use_toaster};
use kivact::prelude::*;
use kivact_ui_basic as basic;
use std::time::Duration;

pub(crate) fn page() -> Page {
    Page::new("toast", "Toast", "Components", include_str!("toast.rs"))
        .description("A notice that briefly reports the result of an action. Put the standard toast_region(use_toaster()).hold_all_on_hover(true) once at the root of the app, and push with use_toaster().push(Toast::new(title).tone(tone)). The body is the second line, the length is duration, and the action is added as a method of Toast; the icon is drawn by the kit's theme for each tone. With several, the new one stacks slightly behind the others; pointing at them or focusing them opens the list with spacing. While it is open, the timers of all the notices stop, and Escape closes it.")
        .usage("use kivact::prelude::*;\nuse kivact::ui::{ControlSize};
use kivact::interaction::{Toast, Tone, use_toaster};\nuse kivact_ui_basic as basic;\nuse std::time::Duration;")
        .example(Example::new(
            "default",
            "Default",
            "A notice with a title and a body. It disappears after 5 seconds and stacks at the bottom right. The toast_region sits at the root of this catalog.",
            default,
        ))
        .example(Example::new(
            "variants",
            "Variants",
            "tone chooses Neutral, Success, Info, Warning or Danger. Warning and Danger are read out more emphatically to assistive technology. Loading passes a spinner as the icon and sets duration to None, so it stays until it is closed.",
            variants,
        ))
        .example(Example::new(
            "action",
            "With action",
            "action adds a button. Pressing it calls the callback and closes the notice. Make the callback in the setup and pass it from the handler.",
            with_action,
        ))
        .example(Example::new(
            "duration",
            "Duration",
            "duration changes how long it shows. None keeps it until it is closed, and the returned ToastId closes it.",
            duration,
        ))
}

// example: default
fn default() -> impl IntoView {
    button("Show Toast")
        .variant(ButtonVariant::Outline)
        .on_press(|| {
            use_toaster().push(
                Toast::new("Event has been created").body("Sunday, December 03, 2023 at 9:00 AM"),
            );
        })
}
// end example

// example: variants
fn variants() -> impl IntoView {
    let show = |label: &'static str, tone: Tone| {
        button(label)
            .variant(ButtonVariant::Outline)
            .size(ControlSize::Small)
            .on_press(move || {
                use_toaster().push(Toast::new(format!("{label} toast")).tone(tone));
            })
    };
    row((
        show("Default", Tone::Neutral),
        show("Success", Tone::Success),
        show("Info", Tone::Info),
        show("Warning", Tone::Warning),
        show("Error", Tone::Danger),
        button("Loading")
            .variant(ButtonVariant::Outline)
            .size(ControlSize::Small)
            .on_press(|| {
                use_toaster().push(
                    Toast::new("Loading toast")
                        .icon_name("loader-circle")
                        .duration(None),
                );
            }),
    ))
    .style(style().gap(8).align_items(Align::Center))
}
// end example

// example: action
fn with_action() -> impl IntoView {
    let undone = signal(0);
    column((
        button("Create Event")
            .variant(ButtonVariant::Outline)
            .on_press(move || {
                use_toaster().push(
                    Toast::new("Event created")
                        .action("Undo", move || undone.update(|count| *count += 1)),
                );
            }),
        basic::muted(move || format!("Undone {} time(s).", undone.get())),
    ))
    .style(style().gap(12).align_items(Align::Center))
}
// end example

// example: duration
fn duration() -> impl IntoView {
    let last = signal(None);
    row((
        button("For 2 seconds")
            .variant(ButtonVariant::Outline)
            .on_press(|| {
                use_toaster().push(Toast::new("Gone soon").duration(Some(Duration::from_secs(2))));
            }),
        button("Until dismissed")
            .variant(ButtonVariant::Outline)
            .on_press(move || {
                last.set(Some(use_toaster().push(Toast::new("Stays").duration(None))));
            }),
        button("Dismiss it")
            .variant(ButtonVariant::Ghost)
            .on_press(move || {
                if let Some(id) = last.get() {
                    use_toaster().dismiss(id);
                }
            }),
    ))
    .style(style().gap(8).align_items(Align::Center))
}
// end example
