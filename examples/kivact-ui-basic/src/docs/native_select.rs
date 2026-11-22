//! Catalog page of Native Select: basic, sizes, disabled and invalid.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("native-select", "Native Select", "Components", include_str!("native_select.rs"))
        .description("A simple selection field close to the standard look.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "basic",
            "Basic",
            "Pass the options and a binding that holds the option. The arrow keys choose a value without opening; a press opens the list.",
            basic,
        ))
        .example(Example::new("sizes", "Sizes", "Size Sm is 28 px high.", sizes))
        .example(Example::new(
            "disabled",
            "Disabled",
            "A disabled selection field is drawn faint and cannot be opened.",
            disabled,
        ))
        .example(Example::new(
            "invalid",
            "Invalid",
            "invalid set to true makes the frame and the ring the destructive color.",
            invalid,
        ))
}

// example: basic
fn fruits() -> Vec<ListItem<&'static str>> {
    vec![
        ListItem::new("apple", "Apple"),
        ListItem::new("banana", "Banana"),
        ListItem::new("blueberry", "Blueberry").disabled(true),
        ListItem::new("grapes", "Grapes"),
    ]
}

fn basic() -> impl IntoView {
    let fruit = signal(None::<&'static str>);
    basic::native_select(fruits(), fruit)
        .placeholder("Select fruit")
        .name("Fruit")
}
// end example

// example: sizes
fn sizes() -> impl IntoView {
    let (a, b) = (signal(Some("apple")), signal(Some("banana")));
    row((
        basic::native_select(vec![ListItem::new("apple", "Apple")], a).name("Default"),
        basic::native_select(vec![ListItem::new("banana", "Banana")], b)
            .size(basic::NativeSelectSize::Sm)
            .name("Small"),
    ))
    .style(style().gap(12))
}
// end example

// example: disabled
fn disabled() -> impl IntoView {
    basic::native_select(vec![ListItem::new("a", "Apple")], signal(Some("a")))
        .disabled(true)
        .name("Fruit")
}
// end example

// example: invalid
fn invalid() -> impl IntoView {
    basic::native_select(vec![ListItem::new("a", "Apple")], signal(None))
        .placeholder("Select fruit")
        .invalid(true)
        .name("Fruit")
}
// end example
