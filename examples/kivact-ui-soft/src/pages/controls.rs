//! Demonstrate the action, feedback and choice recipes in the Soft catalog.
use super::labelled;
use kivact::interaction::ListItem;
use kivact::state::signal;
use kivact::style::style;
use kivact::ui::{
    ButtonVariant, ControlSize, button, button_group, checkbox, progress, radio_group,
    range_slider, slider, switch, toggle_button,
};
use kivact::ui::{RadioGroupVariant, toolbar};
use kivact::view::Orientation;
use kivact::view::{Element, Node, column, icon, row};
use kivact_ui_soft::{check_indicator, keyboard_hint, radio_indicator, with_key_hints};

fn buttons() -> Node {
    column((
        labelled(
            "Variants",
            row((
                button("Primary"),
                button("Secondary").variant(ButtonVariant::Secondary),
                button("Ghost").variant(ButtonVariant::Ghost),
                button("Danger").variant(ButtonVariant::Danger),
            ))
            .style(style().gap(8).flex_wrap(true)),
        ),
        labelled(
            "Sizes and states",
            row((
                button("Extra small").size(ControlSize::ExtraSmall),
                button("Small").size(ControlSize::Small),
                button("Medium").size(ControlSize::Medium),
                button("Large").size(ControlSize::Large),
                button("Unavailable").disabled(true),
                button("Saving").busy(true),
            ))
            .style(style().gap(8).flex_wrap(true)),
        ),
    ))
    .style(style().gap(12))
}

// example: button
pub(super) fn example_button() -> Node {
    buttons()
}
// end example

// example: button-group
pub(super) fn example_button_group() -> Node {
    labelled(
        "Joined actions",
        button_group("History", (button("Undo"), button("Redo"))),
    )
}
// end example

// example: segmented-control
pub(super) fn example_segmented_control() -> Node {
    labelled(
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
    )
}
// end example

// example: toolbar
pub(super) fn example_toolbar() -> Node {
    labelled(
        "Keyboard navigation between tools",
        with_key_hints(Orientation::Horizontal, || {
            toolbar("Editing")
                .start((keyboard_hint(button("Undo")),))
                .end((keyboard_hint(button("Redo")),))
        }),
    )
}
// end example

// example: indicator
pub(super) fn example_indicator() -> Node {
    labelled(
        "Selection marks",
        row((
            check_indicator(true),
            radio_indicator(true),
            radio_indicator(false),
        ))
        .style(style().gap(12)),
    )
}
// end example

// example: icon-button
pub(super) fn example_icon_button() -> Node {
    labelled(
        "Icon-only actions",
        row((
            button("Search").icon(icon("search")).hide_label(true),
            button("Dismiss")
                .variant(ButtonVariant::Ghost)
                .icon(icon("x"))
                .hide_label(true),
        ))
        .style(style().gap(8)),
    )
}
// end example

// example: toggle-button
pub(super) fn example_toggle_button() -> Node {
    labelled(
        "Off, on and disabled",
        row((
            toggle_button("Pin", signal(false)),
            toggle_button("Follow", signal(true)),
            toggle_button("Lock", signal(false)).disabled(true),
        ))
        .style(style().gap(8)),
    )
}
// end example

// example: checkbox-input
pub(super) fn example_checkbox_input() -> Node {
    labelled(
        "Unchecked, checked, mixed and disabled",
        column((
            checkbox("Unchecked", signal(false)),
            checkbox("Checked", signal(true)),
            checkbox("Mixed", signal(false)).indeterminate(true),
            checkbox("Unavailable", signal(true)).disabled(true),
        )),
    )
}
// end example

// example: switch
pub(super) fn example_switch() -> Node {
    labelled(
        "Off, on and disabled",
        column((
            switch("Notifications", signal(false)),
            switch("Sync", signal(true)),
            switch("Unavailable", signal(true)).disabled(true),
        )),
    )
}
// end example

// example: radio-list
pub(super) fn example_radio_list() -> Node {
    labelled(
        "Single selection",
        radio_group(
            "Delivery",
            vec![
                ListItem::new("pickup", "Pickup"),
                ListItem::new("delivery", "Delivery"),
            ],
            signal(Some("pickup")),
        ),
    )
}
// end example

// example: slider
pub(super) fn example_slider() -> Node {
    labelled(
        "Single value, range and vertical",
        column((
            slider(signal(40.0))
                .label("Volume")
                .range(0.0..=100.0)
                .style(style().width(224)),
            range_slider(signal(20.0), signal(70.0))
                .label("Limits")
                .range(0.0..=100.0)
                .style(style().width(224)),
            slider(signal(60.0))
                .label("Height")
                .range(0.0..=100.0)
                .orientation(Orientation::Vertical),
        ))
        .style(style().gap(16)),
    )
}
// end example

// example: progress-bar
pub(super) fn example_progress_bar() -> Node {
    labelled(
        "Determinate and indeterminate",
        column((
            progress(Some(0.6)).style(style().width(224)),
            progress(None::<f64>).style(style().width(224)),
        ))
        .style(style().gap(16)),
    )
}
// end example
