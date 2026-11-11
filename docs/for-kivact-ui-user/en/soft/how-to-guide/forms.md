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
        kivact::ui::date_input("Event date", date),
        button("Save event")
            .variant(kivact::ui::ButtonVariant::Primary)
            .disabled(move || name.get().trim().is_empty() || date.get().is_none())
            .on_press(move || {
                if let Some(date) = date.get() {
                    save(name.get().trim().to_owned(), date);
                }
            }),
    ))
    .style(|| style().gap(kivact::style::use_tokens::<soft::Tokens>().spacing.s3))
}
```

For fields with validation errors, keep the error visible next to the field and explain how to correct it. Validate the submitted values again at the persistence boundary.

Use `kivact::ui::input_group_with` to create related controls at the group's default size. A control's own `.size(..)` takes precedence. Use `kivact::ui::DateRangePreset` with `kivact::ui::date_range_input(..).presets(..)` to offer named ranges alongside calendar selection.

Bind `soft::rich_text_editor` to a Markdown string. In Write mode, the toolbar applies bold, italic, strikethrough, or inline code to the selection and returns focus to the editor. Switch to Preview to read the formatted result while keeping the same source binding.

Keyed choices use the standard APIs: `select(label, items, selected).open_on_arrow(true)` keeps the previous arrow-opening behavior; `combobox(label, items, selected)` adds search. For multiple choices use a `Vec<K>` binding and `.chips()`, adding `.exclude_selected()` for a tokenizer. Async `.search(..)`, `.debounce(..)`, `.filter(..)`, `.placeholder(..)`, and `.empty_text(..)` remain standard builder options. Use `.presentation(SelectPresentation::BottomSheet)` for touch drawers, or `Adaptive` to follow host touch support. A caller-owned chooser is `selection_field(label, value_label).content(factory)`; keep its search binding outside the factory. Soft supplies select and combobox recipes and tokens.

Use `kivact::ui::button_group(label, children)` for joined actions and `toggle_group(label, items, selected)` for keyed toggles. Both support orientation, inherited control size, and standard `.disabled(..)`. Use `ButtonGroupMember::from(button).solid(true).keep_size(true)` for member divider and size overrides. `.variant(soft::ButtonGroupVariant(soft::Elevation::Low))` adds group elevation. Set `ListItem::with_pressed_icon(..)` on each option for its selected icon; `.icon_only(..)` hides visible labels. Multiple selection uses `Vec<K>` and single selection uses `Option<K>`.

Use standard `calendar(selected)` with either `Option<CalendarDate>` or `Option<DateRange>`; the same calendar supports default month, disabled dates, bounds, inclusive range spans, variable rows, month count, outside days, week start, autofocus, and commit callbacks. `date_input`, `date_range_input`, `date_time_input`, and `time_input` come from `kivact::ui`; date/time values come from `kivact::i18n`. Keep `.calendar(|calendar| ...)` configuration and range presets. Canonical field bounds use `Some(date)` and precision uses `TimeStyle { seconds: true }` with `.format(...)` or `.time_format(...)`. Soft installs calendar/date/time recipes and wheel measurements. Locale owns date parsing and calendar names; component messages use the standard calendar/date/time catalog keys.

Use standard `dropzone(label).icon(icon("upload")).pick_on_press()` for native file choosing and optional drops. Soft `DropzoneVariant::Field` selects compact chrome; the default is a larger target. Keep chosen files in caller state through `.on_files(...)`, show their names with `.hint(...)`, and clear that state with a regular button that returns focus to the target. `.multiple()` chooses native multi-file mode. `.filters(...)` supplies picker filters; `.validate(...)` also validates drops. Extension and metadata/size checks belong in that application callback. Use `.accept_drop(false)` for a compact picker-only field while keeping the native picker enabled.

Adjacent controls use standard `input_group(label, children)` or `input_group_with(label, factory)` to inherit a default size while keeping explicit control sizes. Replace `input_group_text` with standard `input_addon`. The Soft recipes preserve joined logical corners, overlapping borders, raised focused members, the visible label, and accent-muted addons. Standard start/end and block_start/block_end slots support additional adornments.
