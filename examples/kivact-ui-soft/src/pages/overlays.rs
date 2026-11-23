//! Demonstrate the standard anchored layers, dialogs and notifications.
use super::labelled;
use kivact::interaction::{Command, MenuEntry, Toast, use_toaster};
use kivact::state::Signal;
use kivact::state::signal;
use kivact::style::{rgb, style};
use kivact::ui::{
    ButtonVariant, DialogKind, button, command_palette, dialog, hover_card, menu, toast_region,
    tooltip,
};
use kivact::view::IntoView;
use kivact::view::{Element, Node, column, node, row, text};
use kivact_ui_soft::{
    LightboxMedia, alert_dialog, bottom_sheet, bottom_sheet_flow, lightbox, media_overlay,
};

fn dialog_example(kind: DialogKind, trigger: &'static str, name: &'static str) -> Node {
    let open = signal(false);
    labelled(
        name,
        dialog(trigger, open)
            .mode(kind)
            .content(|| text("Review the details before continuing."))
            .footer(move || {
                row((
                    button("Cancel")
                        .variant(ButtonVariant::Ghost)
                        .on_press(move || open.set(false)),
                    button("Done").on_press(move || open.set(false)),
                ))
                .style(style().gap(8))
            })
            .attach(button(trigger)),
    )
}

// example: dialog
pub(super) fn example_dialog() -> Node {
    dialog_example(DialogKind::Modal, "Open dialog", "Modal")
}
// end example

// example: alert-dialog
pub(super) fn example_alert_dialog() -> Node {
    {
        let open = signal(false);
        labelled(
            "Confirm deletion",
            (
                button("Delete file").on_press(move || open.set(true)),
                alert_dialog("Delete file?", open)
                    .description("This action is permanent.")
                    .action_label("Delete")
                    .on_press(move || open.set(false)),
            ),
        )
    }
}
// end example

// example: drawer
pub(super) fn example_drawer() -> Node {
    dialog_example(
        DialogKind::Drawer(kivact::geometry::Edge::Right),
        "Open drawer",
        "Drawer",
    )
}
// end example

// example: bottom-sheet
pub(super) fn example_bottom_sheet() -> Node {
    {
        let open = signal(false);
        labelled(
            "Sheet actions",
            (
                button("Open sheet").on_press(move || open.set(true)),
                bottom_sheet("Document details", open)
                    .content(move || button("Done").on_press(move || open.set(false))),
            ),
        )
    }
}
// end example

// example: bottom-sheet-switcher
pub(super) fn example_bottom_sheet_switcher() -> Node {
    {
        let step = signal(None);
        let selected: Signal<Option<u8>> = step;
        labelled(
            "Two sheet steps",
            (
                button("Start review").on_press(move || step.set(Some(1))),
                bottom_sheet_flow("Review notes", selected, 1)
                    .content(move || button("Continue").on_press(move || step.set(Some(2)))),
                bottom_sheet_flow("Confirm review", selected, 2)
                    .content(move || button("Done").on_press(move || step.set(None))),
            ),
        )
    }
}
// end example

// example: popover
// example: layer
pub(super) fn example_popover() -> Node {
    dialog_example(DialogKind::Popover, "Open popover", "Popover")
}
// end example

// example: hover-card
pub(super) fn example_hover_card() -> Node {
    labelled(
        "Preview on hover",
        hover_card(|| text("Workspace member")).attach(button("Avery Morgan")),
    )
}
// end example

// example: overlay
pub(super) fn example_overlay() -> Node {
    labelled(
        "Overlay on a preview",
        media_overlay(
            button("Cover").into_view(),
            column((
                text("Document preview").style(style().font_weight(600)),
                node().style(style().size(208, 12).background(rgb(0x517983))),
                node().style(style().size(160, 12).background(rgb(0x517983))),
                node().style(style().size(184, 12).background(rgb(0x517983))),
            ))
            .style(
                style()
                    .size(240, 160)
                    .padding(16)
                    .gap(10)
                    .background(rgb(0xdce9f4)),
            )
            .into_view(),
        ),
    )
}
// end example

// example: lightbox
pub(super) fn example_lightbox() -> Node {
    {
        static PREVIEW: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="320" height="180" viewBox="0 0 320 180"><rect width="320" height="180" fill="#dce9f4"/><circle cx="246" cy="56" r="26" fill="#f3b85a"/><path d="M0 180 112 58l80 93 41-46 87 75Z" fill="#517983"/></svg>"##;
        let open = signal(false);
        labelled(
            "Open an image preview",
            (
                button("Open image").on_press(move || open.set(true)),
                lightbox(
                    open,
                    vec![LightboxMedia::image(PREVIEW, "Mountain landscape")],
                )
                .zoom(true),
            ),
        )
    }
}
// end example

// example: tooltip
pub(super) fn example_tooltip() -> Node {
    labelled(
        "Keyboard focus and hover",
        tooltip("Save your changes", button("Save")),
    )
}
// end example

// example: dropdown-menu
pub(super) fn example_dropdown_menu() -> Node {
    labelled(
        "Actions and disabled items",
        menu(
            "Actions",
            vec![
                MenuEntry::item("duplicate", "Duplicate"),
                MenuEntry::item("archive", "Archive").disabled(true),
            ],
        )
        .on_activate(|_| {})
        .attach(button("Actions")),
    )
}
// end example

// example: context-menu
pub(super) fn example_context_menu() -> Node {
    labelled(
        "Right-click or use the context-menu key",
        menu(
            "Document actions",
            vec![
                MenuEntry::item("inspect", "Inspect"),
                MenuEntry::item("copy", "Copy"),
            ],
        )
        .on_activate(|_| {})
        .attach_context(button("Document")),
    )
}
// end example

// example: command-palette
pub(super) fn example_command_palette() -> Node {
    {
        let open = signal(false);
        labelled(
            "Search commands",
            (
                button("Open commands").on_press(move || open.set(true)),
                command_palette(
                    vec![
                        Command::new("open", "Open file").with_group("File"),
                        Command::new("save", "Save file").with_group("File"),
                    ],
                    open,
                )
                .on_activate(|_| {}),
            ),
        )
    }
}
// end example

// example: toast
pub(super) fn example_toast() -> Node {
    {
        let toasts = use_toaster();
        labelled(
            "Notification",
            (
                button("Show notification").on_press(move || {
                    toasts.push(Toast::new("Changes saved").body("Your workspace is up to date."));
                }),
                toast_region(toasts),
            ),
        )
    }
}
// end example

// example: more-menu
pub(super) fn example_more_menu() -> Node {
    labelled(
        "Compact action trigger",
        kivact_ui_soft::more_menu(
            vec![
                MenuEntry::item("duplicate", "Duplicate"),
                MenuEntry::item("archive", "Archive"),
            ],
            |_| {},
        ),
    )
}
// end example
