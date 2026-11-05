# Build the settings form

[日本語](../../../jp/soft/getting-started/02-build-the-settings-form.md)

Keep the display name in a signal. The standard input reads and updates that signal. The Save button uses the current value and calls the supplied persistence callback.

```rust,no_run
use kivact::prelude::*;
use kivact_ui_soft as soft;

fn settings(save: impl Fn(String) + 'static) -> impl IntoView {
    let display_name = signal(String::new());

    column((
        soft::heading(1, "Profile"),
        text_input("Display name", display_name),
        button("Save")
            .variant(kivact::ui::ButtonVariant::Primary)
            .disabled(move || display_name.get().trim().is_empty())
            .on_press(move || save(display_name.get().trim().to_owned())),
    ))
    .style(|| style().gap(kivact::style::use_tokens::<soft::Tokens>().spacing.s3))
}
```

Use the same signal to derive the button's enabled state. The next step shows how to present the save result and change the appearance.
