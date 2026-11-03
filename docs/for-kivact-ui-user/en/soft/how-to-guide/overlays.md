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
