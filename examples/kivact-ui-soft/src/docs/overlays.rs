//! Role: Demonstrate the standard anchored layers, dialogs and notifications.
use super::specimen::specimen;
use kivact::control::{Node, NodeBuilder, row, text};
use kivact::style::style;
use kivact::ui::{
    ButtonVariant, Command, DialogKind, MenuEntry, Side, Toast, button, command_palette, dialog,
    menu, toast_region, toaster, tooltip,
};
use kivact::{Binding, signal};
use kivact_ui_soft::{
    LightboxMedia, alert_dialog, bottom_sheet, bottom_sheet_flow, dialog_header, drawer_header,
    hover_card, lightbox, media_overlay,
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
        "bottom-sheet-switcher" => {
            let step = signal(None);
            let selected: Binding<Option<u8>> = step.into();
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
        "popover" | "layer" => dialog_example(DialogKind::Popover, "Open popover", "Popover"),
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
        "lightbox" => {
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
