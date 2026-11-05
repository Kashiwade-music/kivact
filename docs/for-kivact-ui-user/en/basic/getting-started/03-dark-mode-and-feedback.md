# 3. Dark mode and feedback

[日本語](../../../jp/basic/getting-started/03-dark-mode-and-feedback.md)

In this step the screen learns three things. A switch in the header changes between the light and the dark theme while the app runs. Saving shows a toast. A "Delete account" button asks for confirmation in an alert dialog. At the end you write a test that drives the screen in a real window.

## Replace `src/main.rs`

`profile_card` and `notifications_card` are the ones from step 2. `settings` now takes the `dark` signal, and a new `root` function installs the theme.

```rust,test_harness
use kivact::prelude::*;
use kivact::style::theme::Typography;
use kivact::style::{Direction, Theme, ThemeMode};
use kivact::interaction::{DialogMode, Toast, Tone, toaster};
use kivact::ui::{toast_region};
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

fn settings(dark: Signal<bool>) -> impl IntoView {
    let name = signal(String::from("Ada"));
    let saved_name = signal(String::from("Ada"));
    let language = signal(Some("en"));
    let digest = signal(true);
    let mentions = signal(false);
    let reminder = signal(Some("weekly"));
    let confirm_delete = signal(false);
    column((
        row((
            column((
                basic::h3("Settings"),
                basic::muted("Manage your profile and notifications."),
            ))
            .style(style().grow(1).gap(8)),
            switch("Dark mode", dark),
        ))
        .style(style().align_items(Align::Center)),
        row((
            profile_card(name, saved_name, language),
            notifications_card(digest, mentions, reminder),
        ))
        .style(style().gap(16).align_items(Align::Start)),
        row((
            button("Save").icon(icon("check")).on_press(move || {
                saved_name.set(name.get());
                toaster().push(
                    Toast::new("Settings saved")
                        .tone(Tone::Success)
                        .body(format!("Display name: {}", name.get())),
                );
            }),
            basic::muted(move || format!("Saved name: {}", saved_name.get())),
        ))
        .style(style().gap(12).align_items(Align::Center)),
        basic::separator(),
        basic::alert("Danger zone")
            .variant(basic::AlertVariant::Destructive)
            .icon(icon("triangle-alert"))
            .description("Deleting your account cannot be undone.")
            .action({
                button("Delete account")
                    .variant(kivact::ui::ButtonVariant::Danger)
                    .on_press(move || confirm_delete.set(true))
            })
            .style(style().width(856)),
        dialog("Delete your account?", confirm_delete)
            .mode(DialogMode::AlertModal)
            .closable(false)
            .description("All of your settings will be removed.")
            .footer(move || {
                (
                    button("Cancel")
                        .variant(kivact::ui::ButtonVariant::Outline)
                        .on_press(move || confirm_delete.set(false)),
                    button("Delete")
                        .variant(kivact::ui::ButtonVariant::Danger)
                        .on_press(move || {
                            confirm_delete.set(false);
                            toaster().push(Toast::new("Account deleted"));
                        }),
                )
            }),
        toast_region(toaster()).hold_all_on_hover(true),
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

fn root() -> impl IntoView {
    let dark = signal(false);
    kivact::style::provide_theme(
        move || theme(if dark.get() { ThemeMode::Dark } else { ThemeMode::Light }),
        move || settings(dark),
    )
}

fn main() -> kivact::Result<()> {
    let mut app = App::new("Settings").theme(theme(ThemeMode::Light));
    for font in fonts() {
        app = app.font(font);
    }
    app.run(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kivact::test::{RenderOptions, render_with};

    fn options() -> RenderOptions {
        RenderOptions::default().window_size(1000.0, 800.0).theme(theme(ThemeMode::Light))
    }

    #[test]
    fn saving_shows_a_toast_and_clears_the_badge() {
        let screen = render_with(options(), root);
        assert!(screen.query_by_text("Unsaved changes").is_none());
        screen.get_by_role(Role::TextBox, "Display name").type_text(" Lovelace");
        screen.get_by_text("Unsaved changes");
        screen.get_by_role(Role::Button, "Save").click();
        screen.get_by_role(Role::Status, "Settings saved");
        screen.get_by_text("Saved name: Ada Lovelace");
        assert!(screen.query_by_text("Unsaved changes").is_none());
    }

    #[test]
    fn the_switch_changes_the_theme_and_keeps_the_input() {
        let screen = render_with(options(), root);
        let top_left = |screen: &kivact::test::Screen| {
            screen.screenshot().pixel(4, 4).expect("inside the frame")
        };
        assert_eq!(top_left(&screen) & 0xff, 0xff, "light page");
        screen.get_by_role(Role::Switch, "Dark mode").click();
        assert!(top_left(&screen) & 0xff < 0x20, "dark page");
        assert_eq!(
            screen.get_by_role(Role::TextBox, "Display name").value().as_deref(),
            Some("Ada")
        );
    }

    #[test]
    fn deleting_asks_first() {
        let screen = render_with(options(), root);
        screen.get_by_role(Role::Button, "Delete account").click();
        screen.get_by_role(Role::AlertDialog, "Delete your account?");
        screen.get_by_role(Role::Button, "Cancel").click();
        assert!(screen.query_by_role(Role::AlertDialog, "Delete your account?").is_none());
        assert!(screen.query_by_role(Role::Status, "Account deleted").is_none());

        screen.get_by_role(Role::Button, "Delete account").click();
        screen.get_by_role(Role::Button, "Delete").click();
        screen.get_by_role(Role::Status, "Account deleted");
    }
}
```

Run `cargo run`. You should see the same two cards with a "Dark mode" switch at the top right and a red-tinted "Danger zone" callout at the bottom. Then try these:

1. Flip "Dark mode". The page, the cards and the controls turn dark at once. The text you typed stays, because nothing is rebuilt; only the colors change.
2. Change the name and press Save. A toast "Settings saved" appears at the bottom right and disappears after a few seconds.
3. Press "Delete account". A dialog with Cancel and Delete appears and the page behind it is blocked. Clicking outside the dialog does not close it, because an alert dialog asks for an explicit answer; Cancel and Escape do. Press Cancel, then "Delete account" and Delete: the dialog closes and a toast says "Account deleted".

## How it works

- **Switching the theme.** `provide_theme(theme, child)` gives the subtree below it a theme. Its first argument is a closure that reads `dark`, so every time `dark` changes the theme is rebuilt and every style re-resolves its colors. Components keep their state because they are not remounted. Pass the same kind of closure instead of a fixed `Theme` whenever the theme can change.
- **Toasts.** `toast_region(toaster())` is mounted once; it is the place the toasts appear. Any handler can then call `toaster().push(Toast::new(title))`. `.tone(..)` picks the icon and urgency, `.body(..)` adds a second line. A toast closes itself after five seconds, pauses while the pointer is over it, and has a close button.
- **Alert dialog.** `dialog(title, open)` is the standard dialog; `.mode(DialogMode::AlertModal)` makes it the kit's alert dialog. `open` is a plain `signal(bool)`: a button sets it to `true`, and the buttons in `.footer(..)` set it to `false`. `.closable(false)` removes the close button an alert dialog does not have.
- **Alert vs. alert dialog.** `basic::alert(..)` is a static callout on the page (the "Danger zone" box). The dialog is the pop-up that interrupts.

## The test

The block above ends with `#[cfg(test)] mod tests`. Those tests run with `cargo test` (the `test` feature you added in step 1). `kivact::test::render_with(options, root)` mounts the screen in a real window on a virtual clock, with the same layout and accessibility tree an app has. You find elements the way a user or a screen reader would, by role and name (`get_by_role(Role::Button, "Save")`, `get_by_text(..)`), and act with real input (`click`, `type_text`). The theme must be passed in the options (`theme: theme(ThemeMode::Light)`), because the test does not go through `main`.

Run `cargo test`. You should see three tests pass. The first two take a moment, since each opens a window.

## Next

You have built the whole path from a blank window to a themed screen with state, feedback and a test. Continue with the [how-to guides](../how-to-guide/README.md) to solve particular tasks, such as [theme and tokens](../how-to-guide/theme-and-tokens.md) or [overlays](../how-to-guide/overlays.md). To see every component of the kit at once, run `cargo run -p kivact-ui-basic-catalog` in a clone of the repository. The [kit overview](../../README.md) lists the other kits.
