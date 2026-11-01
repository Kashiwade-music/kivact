# Build forms: fields, validation, pickers and one-time codes

[日本語](../../../jp/basic/how-to-guide/forms.md)

This guide shows how to build forms with the kit: label and validate a field, group fields, add icons and buttons to an input, and use the pickers the standard set does not have (combobox, date picker, calendar and one-time code). It assumes you have done the [tutorial](../getting-started/README.md).

The plain controls (`text_input`, `text_area`, `checkbox`, `switch`, `radio_group`, `select`, `slider`) are standard components; the theme gives them the Lyra look. Every control edits a signal that you own. The form is "submitted" when your own code reads those signals, usually in a button's `on_press`.

## Label a field and validate it

`field(label, control)` pairs a visible label with a control. `.hint(..)` adds help text under it and `.error(..)` shows a message while its value is `Some`. Compute the error from your signals with a `memo`:

```rust,no_run
use kivact::prelude::*;

fn email_field() -> impl IntoView {
    let email = signal(String::from("ada@"));
    let error = memo(move || {
        let email = email.get();
        (!email.contains("@example.com")).then(|| "Use an @example.com address.".to_string())
    });
    field("Email", text_input("Email", email))
        .hint("We never share it.")
        .error(error)
        .style(style().width(320))
}
```

While the error is `Some`, the label turns red, the control is marked invalid and the message is announced as an alert. The label is the control's accessible name, so write the same word in both places (`field("Email", text_input("Email", ..))`), and pressing the label focuses or toggles the control.

To block a submit, derive a `can_submit` memo from the same signals and use it for the button:

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn sign_up() -> impl IntoView {
    let email = signal(String::new());
    let accepted = signal(false);
    let created = signal(false);
    let error = memo(move || {
        let email = email.get();
        (!email.is_empty() && !email.contains('@')).then(|| "Enter an email address.".to_string())
    });
    let ready = memo(move || !email.get().is_empty() && error.get().is_none() && accepted.get());
    basic::field_group((
        field("Email", text_input("Email", email)).error(error),
        checkbox("I accept the terms", accepted),
        button("Create account")
            .disabled(move || !ready.get())
            .on_press(move || created.set(true)),
        basic::muted(move || if created.get() { "Account created." } else { "" }),
    ))
    .style(style().width(320))
}
```

## Group fields into sets

`basic::field_group((..))` stacks fields with the kit's spacing. `basic::field_set(..)` is a group with a legend and a description, and `basic::field_separator()` draws a rule between fields; give it `.text("Or")` to label the rule:

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn address() -> impl IntoView {
    let (name, city) = (signal(String::new()), signal(String::new()));
    basic::field_set(basic::field_group((
        field("Name", text_input("Name", name)),
        basic::field_separator().text("Or"),
        field("City", text_input("City", city)).hint("Where you live."),
    )))
    .legend("Address")
    .description("Used to ship your order.")
    .style(style().width(320))
}
```

A control with a label of its own, like a switch, reads better beside its text. Make the field a row, hide the control's own label (the field shows it) and align the children:

```rust,no_run
use kivact::prelude::*;
use kivact::style::Direction;

fn marketing() -> impl IntoView {
    let on = signal(true);
    field("Marketing emails", switch("Marketing emails", on).hide_label(true))
        .hint("Receive news and offers.")
        .style(
            style()
                .direction(Direction::Row)
                .align_items(Align::Center)
                .gap(12)
                .width(420),
        )
}
```

## Put icons, text and buttons inside an input

`basic::input_group(input)` wraps an input in a shared border and lets you add addons before and after it. `.start(..)` and `.end(..)` take a view; pressing an addon focuses the input. Use `basic::input_group_text(..)` for plain text and `basic::input_group_button(..)` for a button:

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn search_and_copy() -> impl IntoView {
    let query = signal(String::new());
    let url = signal(String::from("https://example.com"));
    column((
        basic::input_group(basic::input_group_input("Search", query).placeholder("Search..."))
            .start(icon("search"))
            .end(basic::input_group_text("12 results"))
            .style(style().width(320)),
        basic::input_group(basic::input_group_input("Link", url))
            .end(
                basic::input_group_button("Copy")
                    .icon(icon("copy"))
                    .on_press(|| {}),
            )
            .style(style().width(320)),
    ))
    .style(style().gap(12))
}
```

- `basic::input_group_textarea(label, signal)` takes the place of the input for multi-line text, and `.block_start(..)` / `.block_end(..)` put an addon above or below it (a character counter, say).
- `.invalid(true)` marks the whole group invalid.

## Pick one or several options from a long list

`basic::combobox(items, selected)` is a text box with a filtered list. The items are `ListItem`s. The selection type decides the mode: `signal(None)` (an `Option<K>`) picks one, `signal(Vec::new())` picks several and shows them as removable chips:

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn frameworks() -> impl IntoView {
    let items = vec![
        ListItem::new("next", "Next.js"),
        ListItem::new("remix", "Remix"),
        ListItem::new("astro", "Astro"),
    ];
    let one = signal(None);
    let many = signal(vec!["next"]);
    column((
        basic::combobox(items.clone(), one)
            .placeholder("Select a framework")
            .name("Framework")
            .show_clear(true)
            .style(style().width(260)),
        basic::combobox(items, many)
            .placeholder("Add framework")
            .name("Frameworks")
            .style(style().width(360)),
    ))
    .style(style().gap(12))
}
```

- Give it `.name(..)`: it is the accessible name, since a combobox has no label of its own. Wrap it in a `field` to show one.
- `.empty_text("No frameworks found.")` replaces the text shown when the filter matches nothing; `.disabled(true)` turns it off.
- For short lists, the standard `select("Language", items, signal)` is simpler. `basic::native_select(items, signal)` is a select with the plain bordered look of a browser's.

## Pick a date or a date range

`basic::date_picker(label, date)` is a button that opens a calendar. `date` is a signal of `Option<CalendarDate>`. `basic::date_range_picker` edits a `DateRange` the same way:

```rust,no_run
use kivact::i18n::{CalendarDate, DateRange};
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn dates() -> impl IntoView {
    let due = signal(Some(CalendarDate::new(2026, 10, 3)));
    let stay = signal(None::<DateRange>);
    column((
        basic::date_picker("Due date", due)
            .placeholder("Pick a due date")
            .format(|d| format!("{:04}/{:02}/{:02}", d.year(), d.month(), d.day()))
            .style(style().width(280)),
        basic::date_range_picker("Stay", stay)
            .calendar(|calendar| calendar.number_of_months(2))
            .style(style().width(300)),
    ))
    .style(style().gap(12))
}
```

`.calendar(|c| ..)` configures the calendar in the popup. For a calendar that is always visible, use `basic::calendar(signal)` or `basic::calendar_range(signal)`. Both accept the same options:

```rust,no_run
use kivact::i18n::{CalendarMonth, Weekday};
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn weekdays_only() -> impl IntoView {
    basic::calendar(signal(None))
        .default_month(CalendarMonth::new(2026, 10))
        .week_start(Weekday::Monday)
        .disabled_dates(|d| matches!(d.weekday(), Weekday::Saturday | Weekday::Sunday))
        .caption(basic::CalendarCaption::Dropdown)
}
```

Month and weekday names, and the date format, come from the messages under `basic.calendar.*` and `basic.date_picker.*`. Replace them with `App::messages` for another language; see [Theme and tokens](theme-and-tokens.md#choose-the-fonts).

## Ask for a one-time code

`basic::input_otp(label, code, length)` shows one cell per character and edits a `String`. The user can type or paste the whole code:

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn verify() -> impl IntoView {
    let code = signal(String::new());
    let done = signal(String::new());
    column((
        basic::input_otp("Verification code", code, 6)
            .groups(&[3, 3])
            .pattern(basic::OtpPattern::Digits)
            .on_complete(move |code: String| done.set(code)),
        basic::muted(move || format!("Entered: {}", done.get())),
    ))
    .style(style().gap(8))
}
```

`.groups(&[3, 3])` splits the cells with a separator, `.pattern(..)` limits the characters (`Digits` is the default, `Alphanumeric` lets letters in) and `.on_complete(..)` runs when the last cell is filled. Verification itself is yours.

## See also

- [Overlays](overlays.md): put a form into a dialog or a sheet.
- [Navigation](navigation.md): tabs and wizards between pages of fields.
- Rustdoc: `cargo doc -p kivact-ui-basic --open` for the input group, combobox, pickers and `input_otp`; `cargo doc -p kivact-ui --open` for `field`, `select` and `text_input`.
