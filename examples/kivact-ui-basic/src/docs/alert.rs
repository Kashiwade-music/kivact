//! Catalog page of Alert: default, destructive and with an action.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("alert", "Alert", "Components", include_str!("alert.rs"))
        .description("A short message that draws the user's attention. It is published as Role::Alert and can carry an icon, a description and an action.")
        .usage("use kivact::prelude::*;\nuse kivact::ui::ControlSize;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "default",
            "Default",
            "Pass a title to alert, with an icon and a description.",
            default,
        ))
        .example(Example::new(
            "destructive",
            "Destructive",
            "Pass Destructive as the variant to draw the text and the icon in the warning color.",
            destructive,
        ))
        .example(Example::new(
            "action",
            "Action",
            "action puts an action at the right end. The action is centered on the height of the alert (shadcn puts it at the top right).",
            with_action,
        ))
}

// example: default
fn default() -> impl IntoView {
    kivact::ui::alert("Heads up!")
        .icon(icon("info"))
        .description("You can add components to your app using the CLI.")
        .style(style().width(420))
}
// end example

// example: destructive
fn destructive() -> impl IntoView {
    kivact::ui::alert("Unable to process your payment.")
        .variant(basic::AlertTone::Destructive)
        .icon(icon("circle-alert"))
        .description("Please verify your billing information and try again.")
        .style(style().width(420))
}
// end example

// example: action
fn with_action() -> impl IntoView {
    kivact::ui::alert("Update available")
        .icon(icon("download"))
        .description("A new version is ready to install.")
        .actions({
            button("Undo")
                .variant(ButtonVariant::Outline)
                .size(ControlSize::Small)
        })
        .style(style().width(420))
}
// end example
