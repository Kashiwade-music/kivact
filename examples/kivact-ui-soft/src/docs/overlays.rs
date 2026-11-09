//! Role: Demonstrate the standard anchored layers, dialogs and notifications.
use super::specimen::specimen;
use kivact::interaction::{Command, DialogMode, MenuEntry, Toast, toaster};
use kivact::state::IntoBinding;
use kivact::state::{Binding, signal};
use kivact::style::style;
use kivact::ui::{
    ButtonVariant, button, command_palette, dialog, hover_card, menu, toast_region, tooltip,
};
use kivact::view::{Element, Node, row, text};
use kivact_ui_soft::{
    LightboxMedia, alert_dialog, bottom_sheet, bottom_sheet_flow, dialog_header, drawer_header,
    lightbox, media_overlay,
};

pub(super) fn examples(slug: &str) -> Option<Node> {
    Some(match slug {
        "dialog" => example_dialog(),
        "alert-dialog" => example_alert_dialog(),
        "drawer" => example_drawer(),
        "dialog-header" => example_dialog_header(),
        "drawer-header" => example_drawer_header(),
        "bottom-sheet" => example_bottom_sheet(),
        "bottom-sheet-switcher" => example_bottom_sheet_switcher(),
        "popover" | "layer" => example_popover(),
        "hover-card" => example_hover_card(),
        "overlay" => example_overlay(),
        "lightbox" => example_lightbox(),
        "tooltip" => example_tooltip(),
        "dropdown-menu" => example_dropdown_menu(),
        "context-menu" => example_context_menu(),
        "command-palette" => example_command_palette(),
        "toast" => example_toast(),
        "more-menu" => example_more_menu(),
        _ => return None,
    })
}

fn dialog_example(kind: DialogMode, trigger: &'static str, name: &'static str) -> Node {
    let open = signal(false);
    let source = match kind {
        DialogMode::AlertModal => "dialog(title, open).mode(DialogMode::AlertModal)",
        DialogMode::Drawer(_) => {
            "dialog(title, open).mode(DialogMode::Drawer(kivact::geometry::Edge::Right))"
        }
        DialogMode::Popover => "dialog(title, open).mode(DialogMode::Popover)",
        _ => "dialog(title, open).mode(DialogMode::Modal)",
    };
    specimen(
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
        source,
    )
}

fn example_dialog() -> Node {
    dialog_example(DialogMode::Modal, "Open dialog", "Modal")
}

fn example_alert_dialog() -> Node {
    {
        let open = signal(false);
        specimen(
            "Confirm deletion",
            (
                button("Delete file").on_press(move || open.set(true)),
                alert_dialog("Delete file?", "This action is permanent.", open)
                    .action("Delete", move || open.set(false)),
            ),
            "alert_dialog(title, description, open).action(\"Delete\", delete_file)",
        )
    }
}

fn example_drawer() -> Node {
    dialog_example(
        DialogMode::Drawer(kivact::geometry::Edge::Right),
        "Open drawer",
        "Drawer",
    )
}

fn example_dialog_header() -> Node {
    specimen(
        "Title and description",
        dialog_header("Document settings").subtitle("Manage access to the document."),
        "dialog_header(\"Document settings\").subtitle(\"Manage access to the document.\")",
    )
}

fn example_drawer_header() -> Node {
    specimen(
        "Drawer title and description",
        drawer_header("Filters").subtitle("Narrow the results."),
        "drawer_header(\"Filters\").subtitle(\"Narrow the results.\")",
    )
}

fn example_bottom_sheet() -> Node {
    {
        let open = signal(false);
        specimen(
            "Sheet actions",
            (
                button("Open sheet").on_press(move || open.set(true)),
                bottom_sheet("Document details", open)
                    .content(move || button("Done").on_press(move || open.set(false))),
            ),
            "bottom_sheet(\"Document details\", open).content(sheet_content)",
        )
    }
}

fn example_bottom_sheet_switcher() -> Node {
    {
        let step = signal(None);
        let selected: Binding<Option<u8>> = step.into_binding();
        specimen(
            "Two sheet steps",
            (
                button("Start review").on_press(move || step.set(Some(1))),
                bottom_sheet_flow("Review notes", selected, 1)
                    .content(move || button("Continue").on_press(move || step.set(Some(2)))),
                bottom_sheet_flow("Confirm review", selected, 2)
                    .content(move || button("Done").on_press(move || step.set(None))),
            ),
            "bottom_sheet_flow(\"Review notes\", step, 1).content(review)",
        )
    }
}

fn example_popover() -> Node {
    dialog_example(DialogMode::Popover, "Open popover", "Popover")
}

fn example_hover_card() -> Node {
    specimen(
        "Preview on hover",
        hover_card(|| text("Workspace member")).attach(button("Avery Morgan")),
        "hover_card(|| text(\"Workspace member\")).attach(button(\"Avery Morgan\"))",
    )
}

fn example_overlay() -> Node {
    specimen(
        "Overlay on a preview",
        media_overlay(text("Document preview")).attach(button("Cover")),
        "media_overlay(|| text(\"Document preview\")).attach(button(\"Cover\"))",
    )
}

fn example_lightbox() -> Node {
    {
        static PREVIEW: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="320" height="180" viewBox="0 0 320 180"><rect width="320" height="180" fill="#dce9f4"/><circle cx="246" cy="56" r="26" fill="#f3b85a"/><path d="M0 180 112 58l80 93 41-46 87 75Z" fill="#517983"/></svg>"##;
        let open = signal(false);
        specimen(
            "Open an image preview",
            (
                button("Open image").on_press(move || open.set(true)),
                lightbox(
                    open,
                    vec![LightboxMedia::image(PREVIEW, "Mountain landscape")],
                )
                .zoom(true),
            ),
            "lightbox(open, media).zoom(true)",
        )
    }
}

fn example_tooltip() -> Node {
    specimen(
        "Keyboard focus and hover",
        tooltip("Save your changes").attach(button("Save")),
        "tooltip(\"Save your changes\").attach(button(\"Save\"))",
    )
}

fn example_dropdown_menu() -> Node {
    specimen(
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
        "menu(\"Actions\", entries, on_pick).attach(button(\"Actions\"))",
    )
}

fn example_context_menu() -> Node {
    specimen(
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
        "menu(\"Document actions\", entries, on_pick).attach_context(button(\"Document\"))",
    )
}

fn example_command_palette() -> Node {
    {
        let open = signal(false);
        specimen(
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
            "command_palette(commands, open).on_activate(on_run)",
        )
    }
}

fn example_toast() -> Node {
    {
        let toasts = toaster();
        specimen(
            "Notification",
            (
                button("Show notification").on_press(move || {
                    toasts.push(Toast::new("Changes saved").body("Your workspace is up to date."));
                }),
                toast_region(toasts),
            ),
            "toasts.push(Toast::new(\"Changes saved\"))",
        )
    }
}

fn example_more_menu() -> Node {
    specimen(
        "Compact action trigger",
        kivact_ui_soft::more_menu(
            vec![
                MenuEntry::item("duplicate", "Duplicate"),
                MenuEntry::item("archive", "Archive"),
            ],
            |_| {},
        ),
        "more_menu(entries, on_pick)",
    )
}
