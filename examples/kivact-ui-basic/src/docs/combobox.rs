//! Catalog page of Combobox: basic, clear, multiple and disabled.

use catalog_shell::{Example, Page};
use kivact::prelude::*;

pub(crate) fn page() -> Page {
    Page::new("combobox", "Combobox", "Components", include_str!("combobox.rs"))
        .description("Choose from a list of options while typing to narrow it down.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "basic",
            "Basic",
            "Typing narrows the list; the arrow keys and Enter choose. Focus stays in the input.",
            basic,
        ))
        .example(Example::new(
            "clear",
            "Clear button",
            "show_clear shows a button that clears the selection. empty_text is the text shown when nothing matches.",
            clear,
        ))
        .example(Example::new(
            "multiple",
            "Multiple",
            "Pass a Vec binding to choose several items; the chosen items sit as chips that can be removed with the x.",
            multiple,
        ))
        .example(Example::new(
            "disabled",
            "Disabled",
            "The input cannot be typed in while it is disabled.",
            disabled,
        ))
}

// example: basic
fn basic() -> impl IntoView {
    let items = vec![
        ListItem::new("next", "Next.js"),
        ListItem::new("nuxt", "Nuxt.js"),
        ListItem::new("remix", "Remix"),
        ListItem::new("astro", "Astro"),
    ];
    kivact::ui::combobox(items, signal(None))
        .label("Framework")
        .placeholder("Select a framework")
        .name("Framework")
        .style(style().width(260))
}
// end example

// example: clear
fn clear() -> impl IntoView {
    let items = vec![
        ListItem::new("next", "Next.js"),
        ListItem::new("nuxt", "Nuxt.js"),
        ListItem::new("remix", "Remix"),
        ListItem::new("astro", "Astro"),
    ];
    kivact::ui::combobox(items, signal(Some("remix")))
        .label("Framework")
        .show_clear(true)
        .empty_text("No frameworks found.")
        .name("Framework")
        .style(style().width(260))
}
// end example

// example: multiple
fn multiple() -> impl IntoView {
    let items = vec![
        ListItem::new("next", "Next.js"),
        ListItem::new("nuxt", "Nuxt.js"),
        ListItem::new("remix", "Remix"),
        ListItem::new("astro", "Astro"),
    ];
    kivact::ui::combobox(items, signal(vec!["next", "astro"]))
        .label("Frameworks")
        .chips()
        .placeholder("Add framework")
        .name("Frameworks")
        .style(style().width(360))
}
// end example

// example: disabled
fn disabled() -> impl IntoView {
    let items = vec![
        ListItem::new("next", "Next.js"),
        ListItem::new("nuxt", "Nuxt.js"),
        ListItem::new("remix", "Remix"),
        ListItem::new("astro", "Astro"),
    ];
    kivact::ui::combobox(items, signal(Some("nuxt")))
        .label("Framework")
        .disabled(true)
        .name("Framework")
        .style(style().width(260))
}
// end example
