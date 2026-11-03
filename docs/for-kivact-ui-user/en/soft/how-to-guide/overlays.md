# Overlays

[日本語](../../../jp/soft/how-to-guide/overlays.md)

Bind a dialog to an open signal and provide a descriptive title. An alert dialog can focus the Cancel action on open. Supply the destructive action and update the open signal from its callback.

```rust,no_run
use kivact::prelude::*;
fn delete_confirmation() -> impl IntoView {
    let open = signal(false);
    (
        button("Delete file").on_press(move || open.set(true)),
        kivact_ui_soft::alert_dialog("Delete file?", "This action is permanent.", open)
            .action("Delete", move || open.set(false)),
    )
}
```
