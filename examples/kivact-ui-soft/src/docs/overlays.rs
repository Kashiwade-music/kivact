//! Role: Demonstrate the standard anchored layers, dialogs and notifications.
use super::specimen::specimen;
use kivact::control::{Node, NodeBuilder, row, text};
use kivact::signal;
use kivact::style::style;
use kivact::ui::{
    ButtonVariant, Command, DialogKind, MenuEntry, Side, Toast, button, command_palette, dialog,
    menu, toast_region, toaster, tooltip,
};
use kivact_ui_soft::{
    alert_dialog, bottom_sheet, dialog_header, drawer_header, hover_card, media_overlay,
};

pub(super) fn examples(slug: &str) -> Option<Node> {
    Some(match slug {
        "dialog" => dialog_example(DialogKind::Modal, "Open dialog", "Modal"),
        "alert-dialog" => {
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
        "drawer" => dialog_example(DialogKind::Drawer(Side::Right), "Open drawer", "Drawer"),
        "dialog-header" => specimen(
            "Title and description",
            dialog_header("Document settings").subtitle("Manage access to the document."),
            "dialog_header(\"Document settings\").subtitle(\"Manage access to the document.\")",
        ),
        "drawer-header" => specimen(
            "Drawer title and description",
            drawer_header("Filters").subtitle("Narrow the results."),
            "drawer_header(\"Filters\").subtitle(\"Narrow the results.\")",
        ),
        "bottom-sheet" => {
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
        "popover" => dialog_example(DialogKind::Popover, "Open popover", "Popover"),
        "hover-card" => specimen(
            "Preview on hover",
            hover_card(|| text("Workspace member")).attach(button("Avery Morgan")),
            "hover_card(|| text(\"Workspace member\")).attach(button(\"Avery Morgan\"))",
        ),
        "overlay" => specimen(
            "Overlay on a preview",
            media_overlay(|| text("Document preview")).attach(button("Cover")),
            "media_overlay(|| text(\"Document preview\")).attach(button(\"Cover\"))",
        ),
        "tooltip" => specimen(
            "Keyboard focus and hover",
            tooltip("Save your changes").attach(button("Save")),
            "tooltip(\"Save your changes\").attach(button(\"Save\"))",
        ),
        "dropdown-menu" => specimen(
            "Actions and disabled items",
            menu(
                "Actions",
                vec![
                    MenuEntry::item("duplicate", "Duplicate"),
                    MenuEntry::item("archive", "Archive").disabled(true),
                ],
                |_| {},
            )
            .attach(button("Actions")),
            "menu(\"Actions\", entries, on_pick).attach(button(\"Actions\"))",
        ),
        "context-menu" => specimen(
            "Right-click or use the context-menu key",
            menu(
                "Document actions",
                vec![
                    MenuEntry::item("inspect", "Inspect"),
                    MenuEntry::item("copy", "Copy"),
                ],
                |_| {},
            )
            .attach_context(button("Document")),
            "menu(\"Document actions\", entries, on_pick).attach_context(button(\"Document\"))",
        ),
        "command-palette" => {
            let open = signal(false);
            specimen(
                "Search commands",
                (
                    button("Open commands").on_press(move || open.set(true)),
                    command_palette(
                        open,
                        vec![
                            Command::new("open", "Open file").group("File"),
                            Command::new("save", "Save file").group("File"),
                        ],
                        |_| {},
                    ),
                ),
                "command_palette(open, commands, on_run)",
            )
        }
        "toast" => {
            let toasts = toaster();
            specimen(
                "Notification",
                (
                    button("Show notification").on_press(move || {
                        toasts.push(
                            Toast::new("Changes saved").body("Your workspace is up to date."),
                        );
                    }),
                    toast_region(toasts),
                ),
                "toasts.push(Toast::new(\"Changes saved\"))",
            )
        }
        "more-menu" => specimen(
            "Compact action trigger",
            kivact_ui_soft::more_menu(
                vec![
                    MenuEntry::item("duplicate", "Duplicate"),
                    MenuEntry::item("archive", "Archive"),
                ],
                |_| {},
            ),
            "more_menu(entries, on_pick)",
        ),
        _ => return None,
    })
}

fn dialog_example(kind: DialogKind, trigger: &'static str, name: &'static str) -> Node {
    let open = signal(false);
    let source = match kind {
        DialogKind::AlertModal => "dialog(title, open).kind(DialogKind::AlertModal)",
        DialogKind::Drawer(_) => "dialog(title, open).kind(DialogKind::Drawer(Side::Right))",
        DialogKind::Popover => "dialog(title, open).kind(DialogKind::Popover)",
        _ => "dialog(title, open).kind(DialogKind::Modal)",
    };
    specimen(
        name,
        dialog(trigger, open)
            .kind(kind)
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
