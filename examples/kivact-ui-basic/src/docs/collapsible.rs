//! Catalog page of Collapsible: a trigger that shows and hides a region.

use catalog_shell::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("collapsible", "Collapsible", "Components", include_str!("collapsible.rs"))
        .description("A component with no look of its own that opens and closes a region from a trigger. Write collapsible(open).content(..).attach(trigger); the button passed to attach becomes the open and close button (it holds the expanded state), and the content exists only while it is open.")
        .usage("use kivact::prelude::*;\nuse kivact::ui::ControlSize;\nuse kivact_ui_basic as basic;")
        .align(PreviewAlign::Start)
        .example(Example::new(
            "default",
            "Default",
            "Pass the open state, a function that makes the content, and the trigger to attach. The trigger here is a button with an icon.",
            default,
        ))
        .example(Example::new(
            "open",
            "Initially open",
            "A signal whose initial value is true starts open. The caller can read the state.",
            initially_open,
        ))
}

// example: default
fn default() -> impl IntoView {
    let open = signal(false);
    let repository = |name: &'static str| {
        text(name).style(
            style()
                .padding_x(16)
                .padding_y(8)
                .font_size(12)
                .border(1, basic::BORDER),
        )
    };
    kivact::ui::disclosure("Details", open)
        .variant(basic::DisclosureVariant::Plain)
        .animated(false)
        .content(move || {
            column((
                repository("@radix-ui/primitives"),
                repository("@radix-ui/colors"),
                repository("@stitches/react"),
            ))
            .style(style().gap(8).padding_top(8))
        })
        .attach(
            button("@peduarte starred 3 repositories")
                .variant(ButtonVariant::Outline)
                .icon(icon("chevrons-up-down"))
                .style(style().width(pct(100))),
        )
        .style(style().width(350))
}
// end example

// example: open
fn initially_open() -> impl IntoView {
    let open = signal(true);
    column((
        basic::muted(move || {
            if open.get() {
                "Details shown"
            } else {
                "Details hidden"
            }
            .to_string()
        }),
        kivact::ui::disclosure("Details", open)
            .variant(basic::DisclosureVariant::Plain)
            .animated(false)
            .content(|| basic::paragraph("Order #4189 shipped on Friday and arrives on Tuesday."))
            .attach(
                button("Toggle details")
                    .variant(ButtonVariant::Outline)
                    .hide_label(true)
                    .icon(icon("chevrons-up-down"))
                    .variant(ButtonVariant::Ghost)
                    .size(ControlSize::Small),
            ),
    ))
    .style(style().gap(8).align_items(Align::Start))
}
// end example
