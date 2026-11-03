//! Role: Demonstrate the action, feedback and choice recipes in the Soft catalog.
use super::examples::specimen;
use kivact::control::{Node, NodeBuilder, column, icon, row};
use kivact::signal;
use kivact::style::style;
use kivact::ui::{
    ButtonVariant, ControlSize, ListItem, button, checkbox, progress, radio_group, range_slider,
    slider, switch, toggle,
};
use kivact::view::Orientation;

pub(super) fn examples(slug: &str) -> Option<Node> {
    Some(match slug {
        "button" => buttons(),
        "icon-button" => specimen(
            "Icon-only actions",
            row((
                button("Search").icon(icon("search")).hide_label(true),
                button("Dismiss")
                    .variant(ButtonVariant::Ghost)
                    .icon(icon("x"))
                    .hide_label(true),
            ))
            .style(style().gap(8)),
            "button(\"Search\").icon(icon(\"search\")).hide_label(true)",
        ),
        "toggle-button" => specimen(
            "Off, on and disabled",
            row((
                toggle("Pin", signal(false)),
                toggle("Follow", signal(true)),
                toggle("Lock", signal(false)).disabled(true),
            ))
            .style(style().gap(8)),
            "toggle(\"Follow\", signal(true))",
        ),
        "checkbox-input" => specimen(
            "Unchecked, checked, mixed and disabled",
            column((
                checkbox("Unchecked", signal(false)),
                checkbox("Checked", signal(true)),
                checkbox("Mixed", signal(false)).indeterminate(true),
                checkbox("Unavailable", signal(true)).disabled(true),
            )),
            "checkbox(\"Mixed\", signal(false)).indeterminate(true)",
        ),
        "switch" => specimen(
            "Off, on and disabled",
            column((
                switch("Notifications", signal(false)),
                switch("Sync", signal(true)),
                switch("Unavailable", signal(true)).disabled(true),
            )),
            "switch(\"Sync\", signal(true))",
        ),
        "radio-list" => specimen(
            "Single selection",
            radio_group(
                "Delivery",
                vec![
                    ListItem::new("pickup", "Pickup"),
                    ListItem::new("delivery", "Delivery"),
                ],
                signal(Some("pickup")),
            ),
            "radio_group(\"Delivery\", options, signal(Some(\"pickup\")))",
        ),
        "slider" => specimen(
            "Single value, range and vertical",
            column((
                slider("Volume", signal(40.0))
                    .range(0.0..=100.0)
                    .style(style().width(224)),
                range_slider("Limits", signal(20.0), signal(70.0))
                    .range(0.0..=100.0)
                    .style(style().width(224)),
                slider("Height", signal(60.0))
                    .range(0.0..=100.0)
                    .orientation(Orientation::Vertical),
            ))
            .style(style().gap(16)),
            "slider(\"Volume\", signal(40.0)).range(0.0..=100.0)",
        ),
        "progress-bar" => specimen(
            "Determinate and indeterminate",
            column((
                progress(Some(0.6)).style(style().width(224)),
                progress(None::<f64>).style(style().width(224)),
            ))
            .style(style().gap(16)),
            "progress(Some(0.6))\nprogress(None::<f64>)",
        ),
        _ => return None,
    })
}

fn buttons() -> Node {
    column((
        specimen(
            "Variants",
            row((
                button("Primary"),
                button("Secondary").variant(ButtonVariant::Secondary),
                button("Ghost").variant(ButtonVariant::Ghost),
                button("Danger").variant(ButtonVariant::Danger),
            ))
            .style(style().gap(8).wrap(true)),
            "button(\"Danger\").variant(ButtonVariant::Danger)",
        ),
        specimen(
            "Sizes and states",
            row((
                button("Extra small").size(ControlSize::ExtraSmall),
                button("Small").size(ControlSize::Small),
                button("Medium").size(ControlSize::Medium),
                button("Large").size(ControlSize::Large),
                button("Unavailable").disabled(true),
                button("Saving").loading(true),
            ))
            .style(style().gap(8).wrap(true)),
            "button(\"Small\").size(ControlSize::Small)\nbutton(\"Saving\").loading(true)",
        ),
    ))
    .style(style().gap(12))
}
