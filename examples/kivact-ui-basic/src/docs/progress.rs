//! Catalog page of Progress: static, label with value and interactive.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact::ui::progress;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("progress", "Progress", "Components", include_str!("progress.rs"))
        .description("A bar that shows the progress of work. Pass a value from 0 to 1 as Some to kivact::ui::progress; values out of range are clamped. None is an indeterminate progress that draws a moving segment. It is published as Role::ProgressBar.")
        .usage("use kivact::prelude::*;\nuse kivact::ui::{ControlSize, progress};\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "default",
            "Default",
            "A bar with a value given as Some. Its name is set with name.",
            default,
        ))
        .example(Example::new(
            "label",
            "Label",
            "Add a heading and the percentage as text from the caller.",
            label,
        ))
        .example(Example::new(
            "interactive",
            "Interactive",
            "The value follows a signal. Buttons raise and lower it.",
            interactive,
        ))
}

// example: default
fn default() -> impl IntoView {
    progress(Some(0.33))
        .name("Progress")
        .style(style().width(420))
}
// end example

// example: label
fn label() -> impl IntoView {
    column((
        row((text("Upload progress"), spacer(), basic::muted("66%"))),
        progress(Some(0.66)).name("Upload progress"),
    ))
    .style(style().gap(12).width(420))
}
// end example

// example: interactive
fn interactive() -> impl IntoView {
    let value = signal(0.4);
    column((
        row((
            text("Export"),
            spacer(),
            basic::muted(move || format!("{:.0}%", value.get() * 100.0)),
        )),
        progress(move || Some(value.get())).name("Export"),
        row((
            button("-10")
                .variant(ButtonVariant::Outline)
                .size(ControlSize::Small)
                .on_press(move || value.update(|v| *v = (*v - 0.1).max(0.0))),
            button("+10")
                .variant(ButtonVariant::Outline)
                .size(ControlSize::Small)
                .on_press(move || value.update(|v| *v = (*v + 0.1).min(1.0))),
        ))
        .style(style().gap(8)),
    ))
    .style(style().gap(16).width(420))
}
// end example
