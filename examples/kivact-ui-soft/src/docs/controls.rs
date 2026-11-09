//! Role: Demonstrate the action, feedback and choice recipes in the Soft catalog.
use super::specimen::specimen;
use kivact::interaction::ListItem;
use kivact::state::signal;
use kivact::style::style;
use kivact::ui::{
    ButtonVariant, ControlSize, button, button_group, checkbox, progress, radio_group,
    range_slider, slider, switch, toggle_button, toggle_group,
};
use kivact::ui::{RadioGroupVariant, toolbar};
use kivact::view::Orientation;
use kivact::view::{Element, Node, column, icon, row};
use kivact_ui_soft::{check_indicator, keyboard_hint, radio_indicator, with_key_hints};

pub(super) fn examples(slug: &str) -> Option<Node> {
    Some(match slug {
        "button" => example_button(),
        "button-group" => example_button_group(),
        "toggle-button-group" => example_toggle_button_group(),
        "segmented-control" => example_segmented_control(),
        "toolbar" => example_toolbar(),
        "indicator" => example_indicator(),
        "icon-button" => example_icon_button(),
        "toggle-button" => example_toggle_button(),
        "checkbox-input" => example_checkbox_input(),
        "switch" => example_switch(),
        "radio-list" => example_radio_list(),
        "slider" => example_slider(),
        "progress-bar" => example_progress_bar(),
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

fn example_button() -> Node {
    buttons()
}

fn example_button_group() -> Node {
    specimen(
        "Joined actions",
        button_group("History", (button("Undo"), button("Redo"))),
        "button_group(\"History\", (button(\"Undo\"), button(\"Redo\")))",
    )
}

fn example_toggle_button_group() -> Node {
    specimen(
        "Independent formatting choices",
        toggle_group(
            "Formatting",
            vec![
                ListItem::new("bold", "Bold"),
                ListItem::new("italic", "Italic"),
            ],
            signal(vec!["bold"]),
        ),
        "toggle_group(\"Formatting\", items, selection)",
    )
}

fn example_segmented_control() -> Node {
    specimen(
        "Choose a view",
        with_key_hints(Orientation::Horizontal, || {
            radio_group(
                "View",
                vec![ListItem::new("list", "List"), ListItem::new("grid", "Grid")],
                signal(Some("list")),
            )
            .variant(RadioGroupVariant::Segmented)
            .item_node(|target, _, _| keyboard_hint(target))
        }),
        "radio_group(\"View\", items, selection).variant(RadioGroupVariant::Segmented)",
    )
}

fn example_toolbar() -> Node {
    specimen(
        "Keyboard navigation between tools",
        with_key_hints(Orientation::Horizontal, || {
            toolbar("Editing")
                .start((keyboard_hint(button("Undo")),))
                .end((keyboard_hint(button("Redo")),))
        }),
        "toolbar(\"Editing\").start(|| keyboard_hint(button(\"Undo\")))",
    )
}

fn example_indicator() -> Node {
    specimen(
        "Selection marks",
        row((
            check_indicator(true),
            radio_indicator(true),
            radio_indicator(false),
        ))
        .style(style().gap(12)),
        "radio_indicator(true)",
    )
}

fn example_icon_button() -> Node {
    specimen(
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
    )
}

fn example_toggle_button() -> Node {
    specimen(
        "Off, on and disabled",
        row((
            toggle_button("Pin", signal(false)),
            toggle_button("Follow", signal(true)),
            toggle_button("Lock", signal(false)).disabled(true),
        ))
        .style(style().gap(8)),
        "toggle_button(\"Follow\", signal(true))",
    )
}

fn example_checkbox_input() -> Node {
    specimen(
        "Unchecked, checked, mixed and disabled",
        column((
            checkbox("Unchecked", signal(false)),
            checkbox("Checked", signal(true)),
            checkbox("Mixed", signal(false)).indeterminate(true),
            checkbox("Unavailable", signal(true)).disabled(true),
        )),
        "checkbox(\"Mixed\", signal(false)).indeterminate(true)",
    )
}

fn example_switch() -> Node {
    specimen(
        "Off, on and disabled",
        column((
            switch("Notifications", signal(false)),
            switch("Sync", signal(true)),
            switch("Unavailable", signal(true)).disabled(true),
        )),
        "switch(\"Sync\", signal(true))",
    )
}

fn example_radio_list() -> Node {
    specimen(
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
    )
}

fn example_slider() -> Node {
    specimen(
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
    )
}

fn example_progress_bar() -> Node {
    specimen(
        "Determinate and indeterminate",
        column((
            progress(Some(0.6)).style(style().width(224)),
            progress(None::<f64>).style(style().width(224)),
        ))
        .style(style().gap(16)),
        "progress(Some(0.6))\nprogress(None::<f64>)",
    )
}
