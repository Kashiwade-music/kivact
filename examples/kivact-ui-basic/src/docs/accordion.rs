//! Catalog page of Accordion: single, multiple and controlled open sections.

use catalog_shell::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact::ui::disclosure;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("accordion", "Accordion", "Components", include_str!("accordion.rs"))
        .description("A vertical stack of headings that open and close to fold their content. It stacks kivact::ui::disclosure items and binds a signal of the open item keys to each item's open binding. The basic theme draws a rule under each item.")
        .usage("use kivact::prelude::*;\nuse kivact::ui::{ControlSize, disclosure};\nuse kivact_ui_basic as basic;")
        .align(PreviewAlign::Start)
        .example(Example::new(
            "default",
            "Default",
            "An open item is held in a signal as an Option. Opening one closes the item that was open.",
            default,
        ))
        .example(Example::new(
            "multiple",
            "Multiple",
            "Hold the open items in a Vec to keep several items open at once.",
            multiple,
        ))
        .example(Example::new(
            "controlled",
            "Controlled",
            "The open state is the caller's signal. Buttons can open all items or close them all.",
            controlled,
        ))
}

// example: default
fn default() -> impl IntoView {
    let open = signal(Some("product"));
    let section = move |key: &'static str, title: &'static str, body: &'static str| {
        let is_open = binding(
            move || open.get() == Some(key),
            move |on| open.set(on.then_some(key)),
        );
        disclosure(title, is_open)
            .content(move || text(body))
            .animated(true)
    };
    column((
        section(
            "product",
            "Product Information",
            "Our flagship product combines cutting-edge technology with sleek design.",
        ),
        section(
            "shipping",
            "Shipping Details",
            "We offer worldwide shipping through trusted courier partners.",
        ),
        section(
            "returns",
            "Return Policy",
            "We stand behind our products with a 30-day return policy.",
        ),
    ))
    .style(style().width(448))
}
// end example

// example: multiple
fn multiple() -> impl IntoView {
    let open = signal(vec!["one", "two"]);
    let section = move |key: &'static str, title: &'static str, body: &'static str| {
        let is_open = binding(
            move || open.with(|keys| keys.contains(&key)),
            move |on| {
                open.update(|keys| {
                    keys.retain(|k| *k != key);
                    if on {
                        keys.push(key);
                    }
                })
            },
        );
        disclosure(title, is_open)
            .content(move || text(body))
            .animated(true)
    };
    column((
        section(
            "one",
            "Is it accessible?",
            "Yes. It adheres to the WAI-ARIA design pattern.",
        ),
        section(
            "two",
            "Is it styled?",
            "Yes. It comes with the default Lyra styles.",
        ),
        section(
            "three",
            "Is it animated?",
            "Yes. It follows the reduced motion setting.",
        ),
    ))
    .style(style().width(448))
}
// end example

// example: controlled
fn controlled() -> impl IntoView {
    let open = signal(Vec::<&'static str>::new());
    let all = ["a", "b", "c"];
    let section = move |key: &'static str| {
        let is_open = binding(
            move || open.with(|keys| keys.contains(&key)),
            move |on| {
                open.update(|keys| {
                    keys.retain(|k| *k != key);
                    if on {
                        keys.push(key);
                    }
                })
            },
        );
        disclosure(format!("Section {key}"), is_open)
            .content(move || text(format!("The content of section {key}.")))
            .animated(true)
    };
    column((
        row((
            button("Open all")
                .variant(ButtonVariant::Outline)
                .size(ControlSize::Small)
                .on_press(move || open.set(all.to_vec())),
            button("Close all")
                .variant(ButtonVariant::Outline)
                .size(ControlSize::Small)
                .on_press(move || open.set(Vec::new())),
            basic::muted(move || format!("{} open", open.get().len())),
        ))
        .style(style().gap(8).align_items(Align::Center)),
        column(all.map(section).into_iter().collect::<Vec<_>>()).style(style().width(448)),
    ))
    .style(style().gap(12))
}
// end example
