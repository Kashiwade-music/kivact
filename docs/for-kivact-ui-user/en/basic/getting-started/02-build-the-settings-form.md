# 2. Build the settings form

[日本語](../../../jp/basic/getting-started/02-build-the-settings-form.md)

In this step the "Settings" screen gets two cards. A Profile card holds a name input and a language select, and a Notifications card holds two switches and a reminder choice. A Save button remembers the name, and a badge tells you when the name has changed since the last save.

The kit has no copies of the standard controls. `text_input`, `select`, `switch` and `button` come from Kivact itself, and the theme from step 1 gives them the kit's look. The kit adds what the standard set lacks: `card`, `badge`, `toggle_group`, `field_group` and more.

## Replace `src/main.rs`

`theme`, `fonts` and `main` are unchanged. The new parts are `profile_card`, `notifications_card` and `settings`:

```rust,no_run
use kivact::prelude::*;
use kivact::style::theme::Typography;
use kivact::style::{Direction, Theme, ThemeMode};
use kivact_ui_basic as basic;

fn theme(mode: ThemeMode) -> Theme {
    let theme = basic::theme(mode);
    let text = Typography {
        family: Some("Inter, sans-serif".into()),
        ..theme.text().clone()
    };
    theme.with_text(text)
}

fn fonts() -> Vec<&'static [u8]> {
    Vec::new()
}

fn profile_card(
    name: Signal<String>,
    saved_name: Signal<String>,
    language: Signal<Option<&'static str>>,
) -> impl IntoView {
    let languages = vec![
        ListItem::new("en", "English"),
        ListItem::new("ja", "Japanese"),
        ListItem::new("de", "German"),
    ];
    let changed = move || name.get() != saved_name.get();
    basic::card()
        .header(
            basic::card_header("Profile")
                .description("Choose how others see you.")
                .action({
                    show(changed, || {
                        basic::badge("Unsaved changes").variant(basic::BadgeVariant::Secondary)
                    })
                }),
        )
        .content({
            basic::field_group((
                field("Display name", text_input("Display name", name))
                    .hint("Shown on your public page."),
                field("Language", select("Language", languages, language)),
            ))
        })
        .style(style().width(420))
}

fn notifications_card(
    digest: Signal<bool>,
    mentions: Signal<bool>,
    reminder: Signal<Option<&'static str>>,
) -> impl IntoView {
    let reminders = vec![
        ListItem::new("daily", "Daily"),
        ListItem::new("weekly", "Weekly"),
        ListItem::new("never", "Never"),
    ];
    basic::card()
        .header(
            basic::card_header("Notifications").description("Choose what you want to hear about."),
        )
        .content({
            basic::field_group((
                field("Weekly digest", switch("Weekly digest", digest).hide_label(true))
                    .hint("A summary of your week, every Monday.")
                    .style(style().direction(Direction::Row).align_items(Align::Center).gap(12)),
                field("Mentions", switch("Mentions", mentions).hide_label(true))
                    .hint("Tell me when someone mentions me.")
                    .style(style().direction(Direction::Row).align_items(Align::Center).gap(12)),
                basic::field_separator(),
                field(
                    "Reminders",
                    kivact::ui::toggle_group("Options", reminders, reminder).variant(basic::ToggleVariant::Outline),
                ),
            ))
        })
        .style(style().width(420))
}

fn settings() -> impl IntoView {
    let name = signal(String::from("Ada"));
    let saved_name = signal(String::from("Ada"));
    let language = signal(Some("en"));
    let digest = signal(true);
    let mentions = signal(false);
    let reminder = signal(Some("weekly"));
    column((
        basic::h3("Settings"),
        basic::muted("Manage your profile and notifications."),
        row((
            profile_card(name, saved_name, language),
            notifications_card(digest, mentions, reminder),
        ))
        .style(style().gap(16).align_items(Align::Start)),
        row((
            button("Save").icon(icon("check")).on_press(move || saved_name.set(name.get())),
            basic::muted(move || format!("Saved name: {}", saved_name.get())),
        ))
        .style(style().gap(12).align_items(Align::Center)),
    ))
    .style(
        style()
            .size(pct(100), pct(100))
            .padding(24)
            .gap(16)
            .background(basic::ColorName::Background)
            .color(basic::ColorName::Foreground),
    )
}

fn main() -> kivact::Result<()> {
    let mut app = App::new("Settings").theme(theme(ThemeMode::Light));
    for font in fonts() {
        app = app.font(font);
    }
    app.run(settings)
}
```

Run it. You should see two bordered cards side by side under the heading, with the Save button below them. Then try these:

1. Type in "Display name". A secondary badge "Unsaved changes" appears in the Profile card's corner.
2. Press Save. The badge disappears, and the line next to the button shows the new name.
3. Press Tab to move the focus from control to control. The focused control gets a thin ring. Press Space on a switch to flip it.
4. Open "Language" and pick another entry from the list.
5. Press one of the Daily, Weekly and Never toggles. Only one stays pressed.

## How it works

- **State lives in signals.** `signal(value)` creates a `Signal`, a small handle that is `Copy`. You hand the same handle to a control to edit it and to a closure to read it, and `move` closures simply copy it. A `Signal<String>` holds the name; `saved_name` is the last saved value, kept separately so the app can tell the two apart.
- **Edits stay in the signal.** The controls (`text_input`, `select`, `switch`, `toggle_group`) take a signal and write to it as the user types or picks. Nothing is saved until your `on_press` copies `name` into `saved_name`.
- **`changed` is a closure.** `show(changed, || ..)` calls it whenever the signals it reads change, and mounts the badge only while it returns `true`. A plain closure is enough for a reactive value.
- **Choices are `ListItem`s.** A `ListItem::new(key, label)` pairs a key you choose with the text shown. `select` and `toggle_group` hold the chosen key as `Option<K>`, here `Option<&'static str>`; `None` means nothing is chosen.
- **Slots take closures.** `card().header(..)` takes a ready header, while `.content(|| ..)` and `.footer(|| ..)` take a closure that the card calls once when it mounts. They are written `move || ..` so that they own the signals they use; since signals are `Copy`, no clone is needed.
- **`field(label, control)`** from Kivact pairs a visible label with a control and an optional `.with_hint(..)`. Pressing the label activates the control, and screen readers announce the label as the control's name. The switch inside has `.hide_label(true)` because the field already shows the label.
- **`basic::field_group`** stacks fields with the kit's spacing, and `basic::field_separator()` draws a rule between them.

## Next

[3. Dark mode and feedback](03-dark-mode-and-feedback.md) switches the theme while the app runs, shows a toast when you save, and asks for confirmation before a destructive action.
