# Dark mode and feedback

[日本語](../../../jp/soft/getting-started/03-dark-mode-and-feedback.md)

Keep the appearance setting and the save result in separate signals. Wrap the settings view in `provide_theme` so existing controls pick up the new colors when the appearance changes. Mount a success banner after the save succeeds.

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact::view::{provide_theme, show};
use kivact_ui_soft as soft;

fn settings_feedback() -> impl IntoView {
    let dark = signal(false);
    let saved = signal(false);
    provide_theme(
        move || soft::theme(if dark.get() { ThemeMode::Dark } else { ThemeMode::Light }),
        move || {
            column((
                button("Toggle dark mode").on_press(move || dark.update(|on| *on = !*on)),
                button("Save").on_press(move || saved.set(true)),
                show(saved, move || {
                    soft::banner(soft::BannerStatus::Success, "Settings saved")
                        .description("Your changes are ready to use.")
                        .dismissable(true)
                        .on_dismiss(move || saved.set(false))
                }),
            ))
            .style(|| style().gap(soft::tokens().spacing.s3))
        },
    )
}
```

The example models a successful save. In your application, set `saved` from the persistence callback after the write completes. Use the failure result to show an error message next to the affected field or in an error banner.
