# Step 2: Build the form

[日本語](../../../jp/pop/getting-started/02-build-the-form.md)

In this step you turn the single field into a real form: name and email
fields with hints, a plan switch, a terms checkbox and a pair of kit-styled
buttons.

## The program

Replace `src/main.rs` with:

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_pop as pop;

fn sign_up() -> impl IntoView {
    let name = signal(String::new());
    let email = signal(String::new());
    let plan = signal(Some("team"));
    let terms = signal(false);

    kivact::ui::card((
        text("New account").style(style().font_size(20).weight(600)),
        field(
            "Name",
            text_input("Name", name).placeholder("Your full name"),
        )
        .hint("Shown on your invoices."),
        field(
            "Email",
            text_input("Email", email).placeholder("you@example.com"),
        ),
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
        )
        .hint("You can change the plan later."),
        checkbox("I accept the terms", terms),
        row((
            button("Cancel").variant(pop::ButtonTone::Soft),
            button("Create account").variant(pop::ButtonTone::Accent),
        ))
        .style(style().gap(8).justify_content(Justify::End)),
    ))
    .style(style().width(420).margin(24).gap(16))
}

fn main() -> kivact::Result<()> {
    App::new("New account")
        .theme(pop::theme(ThemeMode::Light))
        .run(sign_up)
}
```

Run it. You should see two labelled fields, the second hint in muted text,
a pill-shaped switch with three segments (Team is selected), a checkbox, and
two buttons on the right: a grey Cancel and a yellow Create account. Click a
segment, or focus the switch and press the arrow keys, to change the plan.
Tick the checkbox with a click or with Space.

## What each part does

**`field(label, control)`** is a standard component. It draws the label above
the control, and `.hint(..)` adds a muted description below it. The label also
becomes the control's accessible name, so screen readers announce "Name" for
the field. In the next step you add `.error(..)` to the same builder.

**Rust note: `signal(Some("team"))`.** The plan is optional because a radio
group can have nothing chosen; the signal holds an `Option` of the option's
key. The key can be any type that is `Clone + Eq + Hash`, such as a string
slice here or your own `enum` later.

**`radio_group(..).variant(kivact::ui::RadioGroupVariant::Segmented)`** draws a radio group as a
segmented control. A segmented control picks exactly one option, so it is a
radio group, not tabs. `.orientation(Orientation::Horizontal)` lays the
options out in a row. `.variant(..)` is how a kit adds a look to a standard
component: the component stays the same, only its drawing changes.

**`button(..).variant(pop::ButtonTone::Accent)`** picks one of the kit's
button tones. Pop has `Accent` (yellow), `Pink`, `Soft`, `Glass` and `Dark`.
Without a tone, `kivact::ui::ButtonVariant::Primary`, `Secondary`, `Outline`, `Ghost` and
`Link` from the core tutorial still work and are drawn in Pop colors. A button
can take a tone and a shape: add `.variant(pop::Slant)` to draw it as a
parallelogram. Try `button("Create account").variant(pop::ButtonTone::Pink)
.variant(pop::Slant)`.

The signals `name`, `email`, `plan` and `terms` are the form's state. Nothing
reads them yet; the next step does.

Next: [Step 3: Validate and submit](03-validate-and-submit.md)
