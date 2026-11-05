# Overlays

[日本語](../../../jp/soft/how-to-guide/overlays.md)

Use a tooltip for a short hint, a hover card for a preview that opens on hover or focus, and a dialog when the user needs to make a decision. For destructive confirmation, use an alert dialog with a descriptive title and action label.

```rust,no_run
use kivact::prelude::*;
use kivact_ui_soft as soft;

fn delete_confirmation(delete_file: impl Fn() + 'static) -> impl IntoView {
    let open = signal(false);

    (
        button("Delete file").on_press(move || open.set(true)),
        soft::alert_dialog("Delete file?", "This action is permanent.", open)
            .action("Delete", move || {
                delete_file();
                open.set(false);
            }),
    )
}
```

The dialog reads the open signal supplied by the caller. The confirmation callback closes it after the operation. For an asynchronous operation, update the signal after success and surface any error where the user can act on it.

Soft lightboxes share the dialog and zoom/pan behaviors. `use_lightbox(media)` owns the selected
image and its fit or zoomed view; `.view().zoom(true)` enables zoom controls. Image failure status
comes from the image node and displays the localized fallback. Bottom sheets keep their height,
scrim, purpose, focus, padding, and snap-point options; release velocity and snap selection use
the shared drag behavior.

Use `kivact::ui::hover_card(factory).attach(trigger)` for previews. Soft supplies the surface recipe, 300 ms pointer opening and 200 ms closing delays, and viewport geometry. Configure placement, alignment, focus opening, controlled visibility, and change callbacks on the standard builder. `soft::hover_card_text(label)` remains a styled, keyboard-reachable trigger.
