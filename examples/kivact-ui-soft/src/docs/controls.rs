//! Role: Demonstrate the action, feedback and choice recipes in the Soft catalog.
use super::specimen::specimen;
use kivact::interaction::ListItem;
use kivact::state::signal;
use kivact::style::style;
use kivact::ui::{
    ButtonVariant, ControlSize, button, checkbox, progress, radio_group, range_slider, slider,
    switch, toggle_button,
};
use kivact::view::Orientation;
use kivact::view::{Node, NodeBuilder, column, icon, row};
use kivact_ui_soft::{
    button_group, check_indicator, radio_indicator, segmented_control, toggle_button_group,
    toolbar, toolbar_item,
};

pub(super) fn examples(slug: &str) -> Option<Node> {
    Some(match slug {
        "button" => buttons(),
        "button-group" => specimen(
            "Joined actions",
            button_group("History", (button("Undo"), button("Redo"))),
            "button_group(\"History\", (button(\"Undo\"), button(\"Redo\")))",
        ),
        "toggle-button-group" => specimen(
            "Independent formatting choices",
            toggle_button_group(
                "Formatting",
                vec![
                    ListItem::new("bold", "Bold"),
                    ListItem::new("italic", "Italic"),
                ],
                signal(vec!["bold"]),
            ),
            "toggle_button_group(\"Formatting\", items, selection)",
        ),
        "segmented-control" => specimen(
            "Choose a view",
            segmented_control(
                "View",
                vec![ListItem::new("list", "List"), ListItem::new("grid", "Grid")],
                signal(Some("list")),
            ),
            "segmented_control(\"View\", items, selection)",
        ),
        "toolbar" => specimen(
            "Keyboard navigation between tools",
            toolbar("Editing")
                .start((toolbar_item(button("Undo")),))
                .end((toolbar_item(button("Redo")),)),
            "toolbar(\"Editing\").start(|| toolbar_item(button(\"Undo\")))",
        ),
        "indicator" => specimen(
            "Selection marks",
            row((
                check_indicator(true),
                radio_indicator(true),
                radio_indicator(false),
            ))
            .style(style().gap(12)),
            "radio_indicator(true)",
        ),
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
                toggle_button("Pin", signal(false)),
                toggle_button("Follow", signal(true)),
                toggle_button("Lock", signal(false)).disabled(true),
            ))
            .style(style().gap(8)),
            "toggle_button(\"Follow\", signal(true))",
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
                button("Saving").busy(true),
            ))
            .style(style().gap(8).wrap(true)),
            "button(\"Small\").size(ControlSize::Small)\nbutton(\"Saving\").busy(true)",
        ),
    ))
    .style(style().gap(12))
}
