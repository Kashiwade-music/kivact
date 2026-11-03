//! Role: Demonstrate the standard anchored layers, dialogs and notifications.
use super::examples::specimen;
use kivact::control::{Node, NodeBuilder, row, text};
use kivact::signal;
use kivact::style::style;
use kivact::ui::{
    ButtonVariant, Command, DialogKind, MenuEntry, Side, Toast, button, command_palette, dialog,
    menu, toast_region, toaster, tooltip,
};

pub(super) fn examples(slug: &str) -> Option<Node> {
    Some(match slug {
        "dialog" => dialog_example(DialogKind::Modal, "Open dialog", "Modal"),
        "alert-dialog" => dialog_example(DialogKind::AlertModal, "Delete file", "AlertModal"),
        "drawer" => dialog_example(DialogKind::Drawer(Side::Right), "Open drawer", "Drawer"),
        "popover" => dialog_example(DialogKind::Popover, "Open popover", "Popover"),
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
