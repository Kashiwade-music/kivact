# Step 3: Validate and submit

[日本語](../../../jp/pop/getting-started/03-validate-and-submit.md)

In this step you add two date fields, show an error under every field that
needs one, show a success message when the form is valid, and write a test
that fills the form like a user.

## The program

Replace `src/main.rs` with the complete program, tests included:

```rust,test_harness
use kivact::i18n::CalendarDate;
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact::interaction::Tone;
use kivact_ui_pop as pop;

fn sign_up(today: CalendarDate) -> impl IntoView {
    let name = signal(String::new());
    let email = signal(String::new());
    let plan = signal(Some("team"));
    let birth = signal(CalendarDate::new(2000, 1, 1));
    let start = signal(today);
    let birth_valid = signal(true);
    let start_valid = signal(true);
    let terms = signal(false);
    let attempted = signal(false);
    let created = signal(false);

    let name_missing = memo(move || name.with(|n| n.trim().is_empty()));
    let email_missing = memo(move || email.with(|e| !e.contains('@')));
    let name_error = memo(move || {
        (attempted.get() && name_missing.get()).then(|| "Enter your name".to_owned())
    });
    let email_error = memo(move || {
        (attempted.get() && email_missing.get()).then(|| "Enter an email address".to_owned())
    });
    let birth_error = memo(move || {
        (birth_valid.get() && birth.get() >= today)
            .then(|| "Birth date must be in the past".to_owned())
    });
    let start_error = memo(move || {
        (start_valid.get() && start.get() < today)
            .then(|| "Choose today or a later date".to_owned())
    });
    let ready = memo(move || {
        !name_missing.get()
            && !email_missing.get()
            && birth_valid.get()
            && birth_error.get().is_none()
            && start_valid.get()
            && start_error.get().is_none()
            && terms.get()
    });

    kivact::ui::card((
        text("New account").style(style().font_size(20).weight(600)),
        field("Name", text_input("Name", name).placeholder("Your full name"))
            .hint("Shown on your invoices.")
            .error(name_error),
        field(
            "Email",
            text_input("Email", email).placeholder("you@example.com"),
        )
        .error(email_error),
        field(
            "Plan",
            radio_group(
                "Plan",
                vec![
                    ListItem::new("solo", "Solo"),
                    ListItem::new("team", "Team"),
                    ListItem::new("business", "Business"),
                ],
                plan,
            )
            .orientation(Orientation::Horizontal)
            .variant(kivact::ui::RadioGroupVariant::Segmented),
        ),
        { let value: kivact::state::Binding<kivact::i18n::CalendarDate> = kivact::state::IntoBinding::into_binding(birth); let value = kivact::state::binding(move || Some(value.get()),move |day| {if let Some(day)=day {value.set(day);}}); kivact::ui::date_input("Birth date",value).calendar(move |calendar| calendar.today(today).month_title(|date| format!("{}-{:02}",date.year(),date.month()))) }.valid(birth_valid),
        show(move || birth_error.get().is_some(), move || {
            text(move || birth_error.get().unwrap_or_default())
                .style(style().font_size(12).color(ColorToken::Danger))
        }),
        { let value: kivact::state::Binding<kivact::i18n::CalendarDate> = kivact::state::IntoBinding::into_binding(start); let value = kivact::state::binding(move || Some(value.get()),move |day| {if let Some(day)=day {value.set(day);}}); kivact::ui::date_input("Start date",value).calendar(move |calendar| calendar.today(today).month_title(|date| format!("{}-{:02}",date.year(),date.month()))) }.valid(start_valid),
        show(move || start_error.get().is_some(), move || {
            text(move || start_error.get().unwrap_or_default())
                .style(style().font_size(12).color(ColorToken::Danger))
        }),
        checkbox("I accept the terms", terms),
        show(move || attempted.get() && !terms.get(), || {
            text("Accept the terms to continue")
                .style(style().font_size(12).color(ColorToken::Danger))
        }),
        row((
            button("Cancel").variant(pop::ButtonTone::Soft),
            button("Create account")
                .variant(pop::ButtonTone::Accent)
                .disabled(created)
                .on_press(move || {
                    attempted.set(true);
                    if ready.get() {
                        created.set(true);
                    }
                }),
        ))
        .style(style().gap(8).justify_content(Justify::End)),
        show(created, move || {
            {let tone=kivact::state::IntoValue::into_value(Tone::Success); kivact::ui::alert("Account created").tone(tone).icon(kivact::view::icon(move || kivact::style::Icon::named(match tone.get() { kivact::interaction::Tone::Danger=>"alert-circle", kivact::interaction::Tone::Warning=>"alert-triangle",kivact::interaction::Tone::Success=>"circle-check",_=>"info-circle"}))).description(move || format!("Welcome, {}.", name.get()))}
        }),
    ))
    .style(style().width(420).margin(24).gap(12))
}

fn main() -> kivact::Result<()> {
    App::new("New account")
        .theme(pop::theme(ThemeMode::Light))
        .run(|| sign_up(CalendarDate::today()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use kivact::test::{RenderOptions, Screen, render_with};
    use kivact::view::Modifiers;

    fn mount() -> Screen {
        let options = RenderOptions::default().window_size(800., 1000.0).theme(pop::theme(ThemeMode::Light));
        render_with(options, || sign_up(CalendarDate::new(2026, 11, 1)))
    }

    #[test]
    fn a_complete_form_creates_the_account() {
        let screen = mount();
        screen.get_by_role(Role::TextBox, "Name").type_text("Haruka");
        screen
            .get_by_role(Role::TextBox, "Email")
            .type_text("haruka@example.com");
        screen
            .get_by_role(Role::CheckBox, "I accept the terms")
            .click();
        screen.get_by_role(Role::Button, "Create account").click();
        screen.get_by_text("Welcome, Haruka.");
        assert!(screen.get_by_role(Role::Button, "Create account").is_disabled());
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }

    #[test]
    fn an_empty_form_shows_what_is_missing() {
        let screen = mount();
        screen.get_by_role(Role::Button, "Create account").click();
        screen.get_by_text("Enter your name");
        screen.get_by_text("Enter an email address");
        screen.get_by_text("Accept the terms to continue");
        assert!(screen.query_by_text("Account created").is_none());
    }

    #[test]
    fn a_start_date_in_the_past_is_refused() {
        let screen = mount();
        let start = screen.get_by_role(Role::TextBox, "Start date");
        start.press_key_with(
            Key::Char('a'),
            Modifiers {
                ctrl: true,
                ..Modifiers::default()
            },
        );
        start.type_text("2026-10-01");
        screen.get_by_text("Choose today or a later date");
    }
}
```

Run `cargo run`, then press Create account on the empty form: red messages
appear under the name and email fields and next to the checkbox. Fill the
form and press the button again: an "Account created" message appears and the
button is disabled. Type `2000-02-30` into a date field: the field turns red
and says the date does not exist.

Run `cargo test`. The three tests open real windows, drive them with
simulated input and check what is on screen.

## How it works

**Validation lives in the app.** Each `memo` derives a value from the signals;
a memo re-runs only when the signals it read change. `.error(name_error)`
takes an `Option<String>` that is `Some` while there is a problem. The field
then shows the message under the control, marks the control invalid and
connects the message to it for screen readers. The name and email errors also
read `attempted`, so they stay quiet until the user presses the button once.

**`kivact::ui::date_input(label, optional_value)`** is an editable date with a
calendar popup. The text follows `YYYY-MM-DD`, and the signal (`birth`) only
changes when the text is a whole, real date, so half-typed input never reaches
your state. Configure `.calendar(|calendar| calendar.today(today))` yourself, which keeps the component predictable
and testable: the program passes `CalendarDate::today()` and the test passes a
fixed date.

**`.valid(signal)`** writes `true` while the text is a real date and `false`
otherwise. The component draws its own format error ("Use YYYY-MM-DD" or "This
date does not exist"). Business rules, such as "in the past", are yours:
`birth_error` and `start_error` check the value only while the text is valid,
so the two messages never overlap.

**`show(condition, || view)`** mounts the view only while the condition is
true. `Tone::Success` turns `kivact::ui::alert` green; alerts and similar display
components take their meaning from the standard `kivact::interaction::Tone`.

**The test** passes `pop::theme(..)` through `RenderOptions`, so it draws the
same theme as the app, and queries by role and accessible name, the same words
a screen reader announces. To replace the text of a date field it selects all
with Ctrl+A and types.

## What you built

You now have a form in the Pop look with typed state, field errors, real date
inputs and a passing test. To go further:

- [Set up the Pop theme and tokens](../how-to-guide/theme-and-tokens.md) for
  dark mode and custom colors.
- [Pick dates and ranges](../how-to-guide/dates-and-calendars.md) for ranges,
  a calendar on its own and event dots.

Next: [Pop how-to guides](../how-to-guide/README.md)
