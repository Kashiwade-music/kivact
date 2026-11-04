# Forms

[日本語](../../../jp/soft/how-to-guide/forms.md)

Use the standard Kivact text input for strings and Soft's typed date input for an optional calendar date. Bind each field to its own signal. The Save button can read both signals together and validate them as one record.

```rust,no_run
use kivact::i18n::CalendarDate;
use kivact::prelude::*;
use kivact_ui_soft as soft;

fn event_form(save: impl Fn(String, CalendarDate) + 'static) -> impl IntoView {
    let name = signal(String::new());
    let date = signal(None::<CalendarDate>);

    column((
        text_input("Event name", name),
        soft::date_input("Event date", date),
        button("Save event")
            .variant(kivact::ui::ButtonVariant::Primary)
            .disabled(move || name.get().trim().is_empty() || date.get().is_none())
            .on_press(move || {
                if let Some(date) = date.get() {
                    save(name.get().trim().to_owned(), date);
                }
            }),
    ))
    .style(|| style().gap(soft::tokens().spacing.s3))
}
```

For fields with validation errors, keep the error visible next to the field and explain how to correct it. Validate the submitted values again at the persistence boundary.

Use `soft::input_group_with` to create related controls at the group's default size. A control's own `.size(..)` takes precedence. Use `soft::DateRangePreset` with `soft::date_range_input(..).presets(..)` to offer named ranges alongside calendar selection.

Bind `soft::rich_text_editor` to a Markdown string. In Write mode, the toolbar applies bold, italic, strikethrough, or inline code to the selection and returns focus to the editor. Switch to Preview to read the formatted result while keeping the same source binding.
