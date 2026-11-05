# Offer choices and quick actions

[日本語](../../../jp/game/how-to-guide/choices-and-toolbars.md)

Use this guide to let the player choose one thing in a game-style menu (a
difficulty, a plan, a screen) and to offer quick actions in a toolbar. The
kit does not add new controls for choosing: it gives the standard
`radio_group` and `tabs` four extra looks, selected with `.variant(..)`. The
behavior, the keyboard use and the accessibility stay the standard ones.

## Choose one option with slanted segments

A segmented control picks exactly one option, so it is a radio group.

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_game as game;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Difficulty {
    Easy,
    Normal,
    Hard,
}

fn difficulty() -> impl IntoView {
    let level = signal(Some(Difficulty::Normal));
    column((
        radio_group(
            "Difficulty",
            vec![
                ListItem::new(Difficulty::Easy, "EASY"),
                ListItem::new(Difficulty::Normal, "NORMAL"),
                ListItem::new(Difficulty::Hard, "HARD"),
            ],
            level,
        )
        .orientation(Orientation::Horizontal)
        .variant(kivact::ui::RadioGroupVariant::Segmented),
        game::mono(move || format!("{:?}", level.get())),
    ))
    .style(style().gap(12).margin(24))
}

fn main() -> kivact::Result<()> {
    App::new("Difficulty")
        .theme(game::theme(ThemeMode::Dark))
        .run(difficulty)
}
```

The value is an `Option` of the option's key; `None` means nothing is chosen
yet. The keys can be an `enum` of your own, as here. Arrow keys move and
choose, wrapping at the ends, and a disabled option (`ListItem::disabled(true)`)
is skipped. Do not use `tabs` for this: tabs switch a view, a segmented
control sets a value.

## Choose a plan with cards

`ChoiceCard` draws the options as large cards. `game::card_label` draws the
icon, the title and the description of an item; the slot adds anything else,
such as a price.

```rust,no_run
use kivact::prelude::*;
use kivact_ui_game as game;

fn plans() -> impl IntoView {
    let plan = signal(Some("team"));
    radio_group(
        "Plan",
        vec![
            ListItem::new("solo", "SOLO").with_description("1 seat"),
            ListItem::new("team", "TEAM").with_description("50 seats"),
        ],
        plan,
    )
    .variant(game::ChoiceCard)
    .item(|item, state| {
        let price = match item.with(|item| *item.key()) {
            "solo" => "$9/month",
            _ => "$39/month",
        };
        (game::card_label(item, state), text(price))
    })
}
# fn main() {}
```

`.item(|item, state| view)` is the standard way to change what an option shows.
Return a tuple to place several views in the card.

## Navigate with vertical tabs

Tabs with the `Navigation` look make a side menu. The tabs write the key of the
open section; `.content(..)` shows the section.

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::Tone;
use kivact_ui_game as game;

fn menu() -> impl IntoView {
    let section = signal("inbox");
    tabs(
        "Sections",
        vec![
            ListItem::new("inbox", "INBOX").with_icon("bell"),
            ListItem::new("squad", "SQUAD").with_icon("users"),
        ],
        section,
    )
    .orientation(Orientation::Vertical)
    .variant(game::Navigation)
    .item(|item, state| {
        let count = match item.with(|item| *item.key()) {
            "inbox" => Some(game::badge("3", Tone::Info)),
            _ => None,
        };
        (game::navigation_label(item, state), count)
    })
    .content(|key| text(format!("Section: {key}")))
}
# fn main() {}
```

`game::navigation_label` draws the icon, the label and the description; the
slot puts a badge or a count after it. For horizontal tabs, the default look is
underlined and `.variant(game::Boxed)` puts the tabs in a box, as in the
[tutorial](../getting-started/03-tabs-and-items.md). To keep what is typed in
a panel when the player switches tabs, add `.keep_mounted()`: every panel is
built once and hidden while its tab is not selected.

## Add quick actions to a toolbar

`kivact::ui::toolbar(label).start(children)` is a dense bordered row, `kivact::ui::button_group(label, children)`
packs related buttons together, and `game::tool_button(label)` is a square
ghost button for an icon. Give a tool button its icon and hide its label; the
label stays as the accessible name.

```rust,no_run
use kivact::prelude::*;
use kivact_ui_game as game;

fn tools(zoom: Signal<f64>) -> impl IntoView {
    kivact::ui::toolbar("View tools").start((
        kivact::ui::button_group("Zoom", (
            game::tool_button("Zoom out")
                .icon(icon("chevron-down"))
                .hide_label(true)
                .on_press(move || zoom.update(|z| *z = (*z - 0.25).max(0.25))),
            game::tool_button("Zoom in")
                .icon(icon("chevron-up"))
                .hide_label(true)
                .on_press(move || zoom.update(|z| *z += 0.25)),
        )),
        game::mono(move || format!("{:.0}%", zoom.get() * 100.0)),
        game::icon_menu(
            "More",
            vec![MenuEntry::item("reset", "Reset view"), MenuEntry::item("quit", "Quit")],
        )
        .on_select(move |key| {
            if key == "reset" {
                zoom.set(1.0);
            }
        }),
    ))
}
# fn main() {}
```

`game::chevron(label, caption)` is a large call-to-action button with a
monospaced caption; `game::stepper(labels, current)`, `game::breadcrumb(items)`
and `game::pagination(page, count)` are navigation helpers that take the
current position from your signals.

## See also

- [Show items in lists and trees](item-lists-and-selection.md)
- [Show meters and status](meters-and-status.md)
- All looks and components: `cargo doc -p kivact-ui-game --open`

## File drop targets

Use `kivact::ui::dropzone(label)` under the Game theme. The recipe retains the console's dashed border, diagonal stripes and drag highlight. Add `.icon(kivact::view::icon("cloud-upload"))` for the upload glyph. Use `.multiple().pick_on_press()` to accept several files and enable the native picker. Callbacks receive `FileInfo`; use `file.path` for the native path. `.hint(..)`, `.filters(..)` and `.validate(..)` provide guidance, picker filters and application validation. Callback errors appear as alerts until the next successful drop or pick.
