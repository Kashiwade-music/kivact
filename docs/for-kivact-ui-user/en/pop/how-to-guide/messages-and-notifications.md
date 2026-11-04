# Show messages and notifications

[日本語](../../../jp/pop/how-to-guide/messages-and-notifications.md)

Use this guide to tell people what happened: an inline alert next to the
thing it is about, a toast that disappears by itself, a notification history
behind a bell button, and a loading or empty placeholder while there is
nothing to show.

All of them are driven by your state. The kit draws messages; it never decides
when to show one, stores a history or marks it read.

## Show an alert on the page

`pop::alert(title)` is an inline box with a title, an optional description and
optional actions. Its color comes from the standard `kivact::interaction::Tone`.

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::Tone;
use kivact_ui_pop as pop;

fn payment_status(failed: Signal<bool>) -> impl IntoView {
    show(failed, || {
        pop::alert("Payment failed")
            .tone(Tone::Danger)
            .description("Your card was declined. Update it and try again.")
            .actions(button("Update card").variant(kivact::ui::ButtonVariant::Outline))
    })
}
# fn main() {}
```

Use `show(condition, ..)` to put the alert on the page and take it away. The
alert has a `Role::Alert` so screen readers announce it when it appears. It
has no close button: if people should be able to dismiss it, add a button in
`.actions(..)` that clears your signal. The tones are `Neutral` (the default),
`Info`, `Success`, `Warning`, `Danger` and `Accent`.

## Show a toast

A toast is a short message that appears in a corner and goes away after a few
seconds. Two parts work together: a `toaster`, the queue, and a `toast_region`,
which draws it. Create the toaster once in setup, mount one region near the
root, and push messages from anywhere.

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact::interaction::{Toast, Tone, toaster};
use kivact::ui::{toast_region};
use kivact_ui_pop as pop;
use std::time::Duration;

fn app() -> impl IntoView {
    let toaster = toaster().max_visible(3);
    column((
        button("Save").on_press(move || {
            toaster.push(
                Toast::new("Saved")
                    .tone(Tone::Success)
                    .body("Your draft is stored.")
                    .duration(Some(Duration::from_secs(4))),
            );
        }),
        toast_region(toaster),
    ))
    .style(style().gap(12).padding(24))
}

fn main() -> kivact::Result<()> {
    App::new("Toasts")
        .theme(pop::theme(ThemeMode::Light))
        .run(app)
}
```

- `Toast::new(title)` is the first line; `.body(text)` adds a second line and
  `.tone(Tone::..)` picks the icon and color. The Pop theme draws a
  rounded card with the tone's icon at the bottom right.
- `.duration(Some(d))` sets how long it stays (5 seconds by default);
  `.duration(None)` keeps it until closed. It pauses while the pointer or keyboard focus is on it.
- `.action(label, handler)` adds a button to the toast (an "Undo", for example).
  Pressing it runs the handler and closes the toast.
- `.icon(|| view)` replaces the tone's icon, for example
  `.icon(|| icon("gift"))`.
- `max_visible(n)` limits how many show at once; call it where you create
  the toaster.
- The region is part of the app, not of one screen. Put it once at the root, and
  call `toaster()` once there and pass the `Toaster` (it is `Copy`) to the
  code that pushes.

## Keep a history of notifications

`pop::notification_menu(trigger, items).on_mark_read(mark_read)` shows a popup list of
notifications from a trigger, usually a bell button. A dot on the trigger
shows while any item is unread.

```rust,no_run
use kivact::prelude::*;
use kivact_ui_pop as pop;

fn bell() -> impl IntoView {
    let items = signal(vec![
        pop::NotificationItem {
            unread: true,
            hue: pop::Hue::Mint,
            ..pop::NotificationItem::new(1, "Order paid", "Order 1042 was paid.", "2 min ago")
        },
        pop::NotificationItem::new(2, "Report ready", "The weekly report is ready.", "Yesterday"),
    ]);
    pop::notification_menu(
        button("Notifications")
            .icon(icon("bell"))
            .hide_label(true)
            .variant(kivact::ui::ButtonVariant::Ghost),
        items,
    ).on_mark_read(move || items.update(|list| list.iter_mut().for_each(|item| item.unread = false)))
}
# fn main() {}
```

The menu only reports "mark all as read": the callback is yours, and it must
change `unread` in your list. The popup stays open afterwards. Adding a new
notification is pushing an item into your signal; give each item an `id` that
never changes, so its row survives read-state updates. The texts ("Notifications",
"Mark all as read", "Unread notifications") are English; replace the keys
`pop.notifications.title`, `pop.notifications.mark_read` and
`pop.notifications.unread` with `App::messages`.

## Show that something is loading or empty

While data loads, show a placeholder instead of nothing. While there is no
data, say so and offer the next action:

```rust,no_run
use kivact::prelude::*;
use kivact_ui_pop as pop;

fn orders(loading: Signal<bool>, count: Signal<usize>) -> impl IntoView {
    column((
        show(loading, || pop::skeleton().rows(3)),
        show(move || !loading.get() && count.get() == 0, || {
            pop::empty_state("No orders yet")
                .description("Orders appear here as soon as customers check out.")
                .content(button("Create an order").variant(kivact::ui::ButtonVariant::Primary))
        }),
        show(loading, || row((pop::spinner(), text("Loading orders")))
            .style(style().gap(8))),
    ))
    .style(style().gap(16).padding(24))
}
# fn main() {}
```

- `pop::skeleton().rows(n)` draws placeholder bars where text will appear.
- `pop::spinner()` and `pop::dots_loader()` are small animated loaders; they
  are named "Loading" for screen readers (message `pop.loader.label`).
- `pop::empty_state(title)` draws an illustration, a description and an
  optional action.

How to start and finish loading (a task that sets `loading` when data arrives)
is a core topic, covered in the core guide on asynchronous work.

## Which one to pick

| Situation | Use |
| --- | --- |
| The page cannot continue until the user reads it | `alert` next to the cause |
| Confirm that something just happened | a toast |
| Things happened while the user was away | `notification_menu` |
| Nothing to show yet | `skeleton`, `spinner` or `empty_state` |
| The user must decide now | the standard `kivact::ui::dialog` |

## See also

- [Show figures, status and records on a business screen](status-and-figures.md)
- [Pick dates and ranges](dates-and-calendars.md) for the text keys of the
  calendar
- Toast and toaster API: `kivact::ui::Toast`, `Toaster` and `toast_region` in
  `cargo doc -p kivact --open`; all Pop components: `cargo doc -p kivact-ui-pop
  --open`
