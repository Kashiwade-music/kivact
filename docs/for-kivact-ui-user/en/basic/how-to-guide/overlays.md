# Show dialogs, sheets, popovers, menus and toasts

[日本語](../../../jp/basic/how-to-guide/overlays.md)

This guide shows how to put something above the page: a dialog, a confirmation, a side sheet, a bottom drawer, a popover, a hover card, a menu and a toast. It assumes you have done the [tutorial](../getting-started/README.md) and that the app uses `basic::theme`.

Most of these are not kit components. Dialogs, alert dialogs and sheets are the standard `dialog(title, open)` with a different `.mode(..)`, and a popover is a `use_popup` surface, menus are the standard `menu(..)`, and toasts are the standard `toast_region` and `use_toaster`. The theme draws them in the Lyra look. Swipe drawers also use this standard dialog; hover previews use standard `kivact::ui::hover_card` with the Basic surface recipe.

Two rules hold for all of them:

- **You own the open state.** A dialog, sheet or drawer shows while a `signal(bool)` is `true`. A button sets it to `true`, and the buttons inside set it to `false`. Escape also closes them (and writes `false`), and the focus returns to the control that opened it.
- **The content is built each time it opens.** `.content(..)` and `.footer(..)` take functions that run when the overlay opens, so a closed overlay costs nothing and a reopened one starts fresh. Keep what must survive in a signal outside the closure.

## Show a dialog

```rust,no_run
use kivact::prelude::*;

#[component]
fn edit_profile() -> impl Element {
    let open = signal(false);
    let name = signal(String::from("Ada"));
    column((
        button("Edit profile")
            .variant(kivact::ui::ButtonVariant::Outline)
            .on_press(move || open.set(true)),
        dialog("Edit profile", open)
            .description("Make changes to your profile here.")
            .content(move || text_input(name).label("Name"))
            .footer(move || button("Save changes").on_press(move || open.set(false))),
    ))
}
```

The dialog is modal: focus stays inside until it closes, and it starts on the first focusable control of the content (here the input). It draws a close button in the corner; `.closable(false)` removes it. The title is also the dialog's accessible name. `.hide_title(true)` keeps the name but does not show the heading.

## Ask for confirmation

An alert dialog is a dialog of kind `AlertModal`. A click outside it does not close it, so the user has to answer with one of the footer buttons (Escape still cancels). It has no corner close button, so pass `.closable(false)`:

```rust,no_run
use kivact::prelude::*;
use kivact::ui::DialogKind;
use kivact_ui_basic as basic;

#[component]
fn delete_chat() -> impl Element {
    let open = signal(false);
    let deleted = signal(false);
    column((
        button("Delete chat")
            .variant(kivact::ui::ButtonVariant::Danger)
            .on_press(move || open.set(true)),
        basic::muted(move || if deleted.get() { "Chat deleted." } else { "Chat kept." }),
        dialog("Delete chat?", open)
            .mode(DialogKind::AlertModal)
            .closable(false)
            .description("This will permanently delete this conversation.")
            .footer(move || {
                (
                    button("Cancel")
                        .variant(kivact::ui::ButtonVariant::Outline)
                        .on_press(move || open.set(false)),
                    button("Delete")
                        .variant(kivact::ui::ButtonVariant::Danger)
                        .on_press(move || {
                            deleted.set(true);
                            open.set(false);
                        }),
                )
            }),
    ))
    .style(style().gap(12))
}
```

The footer returns a tuple of buttons. The focus starts on the first one, so put the safe choice (Cancel) first. Variants of the alert dialog:

- `.variant(basic::AlertDialogSize::Sm)` makes it a narrow, centered 320 px dialog. Give both footer buttons `style().flex_grow(1).flex_basis(0)` to share the width.
- `.media(|| icon("trash-2").style(style().size(24, 24)))` shows an icon beside the title.

## Open a sheet from the edge

A sheet is a dialog of kind `Drawer(side)`:

```rust,no_run
use kivact::prelude::*;
use kivact::ui::DialogKind;
use kivact::geometry::{Edge};
use kivact_ui_basic as basic;

#[component]
fn notifications() -> impl Element {
    let open = signal(false);
    column((
        button("Notifications")
            .variant(kivact::ui::ButtonVariant::Outline)
            .on_press(move || open.set(true)),
        dialog("Notifications", open)
            .mode(DialogKind::Drawer(Edge::Right))
            .description("Choose what you want to hear about.")
            .content(|| basic::muted("No new notifications."))
            .footer(move || button("Close").on_press(move || open.set(false))),
    ))
}
```

`Edge` is `Left`, `Right`, `Top` or `Bottom`. For a bottom panel that the user can swipe away, use the kit's `kivact::ui::dialog(title, open).mode(DialogKind::Drawer(Edge::Bottom))` instead. It takes the same `.description(..)`, `.content(..)` and `.footer(..)` and `.mode(DialogKind::Drawer(Edge::..))` (bottom by default):

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

#[component]
fn move_goal() -> impl Element {
    let open = signal(false);
    let goal = signal(350);
    column((
        button("Open drawer")
            .variant(kivact::ui::ButtonVariant::Outline)
            .on_press(move || open.set(true)),
        kivact::ui::dialog("Move goal", open)
            .mode(kivact::ui::DialogKind::Drawer(kivact::geometry::Edge::Bottom))
            .variant(basic::DrawerVariant::Drawer)
            .closable(false)
            .description("Set your daily activity goal.")
            .content(move || text(move || format!("{} calories/day", goal.get())))
            .footer(move || {
                (
                    button("Submit").on_press(move || open.set(false)),
                    button("Cancel")
                        .variant(kivact::ui::ButtonVariant::Outline)
                        .on_press(move || open.set(false)),
                )
            }),
    ))
}
```

`kivact::ui::dialog` and `kivact::ui::hover_card` are not `Element`s: they draw nothing where you place them, so `.style(..)` and `.test_id(..)` do not apply to them.

## Anchor a popover to a button

A popover is a surface anchored to a node of the page. `use_popup(open)` opens and closes it with the `open` signal, `.anchor(node)` names the node it hangs from, and `surface(placement, content)` builds it. Bind the node to the trigger with `.node_ref(..)`:

```rust,no_run
use kivact::interaction::use_popup;
use kivact::prelude::*;
use kivact::view::Placement;

#[component]
fn dimensions() -> impl Element {
    let open = signal(false);
    let width = signal(String::from("100%"));
    let anchor = node_ref();
    let popup = use_popup(open).anchor(anchor);
    column((
        button("Open popover")
            .variant(kivact::ui::ButtonVariant::Outline)
            .node_ref(anchor)
            .on_press(move || open.set(true)),
        popup.surface(Placement::Below, move || {
            column((
                kivact::ui::heading("Dimensions", 4),
                text_input(width).label("Width"),
            ))
            .style(style().gap(8).width(240))
        }),
    ))
}
```

- The `Placement` you pass to `surface` (`Above`, `Below`, `Start`, `End`) picks the side of the anchor. If the popover does not fit in the window it flips to the other side.
- A popover is not modal. It closes on Escape, on a click outside, and when the focus leaves both it and the trigger.

## Preview something on hover

`kivact::ui::hover_card` opens when the pointer rests on its trigger. It opens after 600 ms and closes 300 ms after the pointer leaves; change those with `.open_delay(..)` and `.close_delay(..)`:

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

#[component]
fn mention() -> impl IntoView {
    kivact::ui::hover_card(|| {
        column((
            text("@kivact").style(style().font_weight(600)),
            basic::muted("A native GUI framework for Rust."),
        ))
        .style(style().gap(8))
    })
    .attach(button("@kivact").variant(kivact::ui::ButtonVariant::Link))
}
```

It takes `.placement(..)` and `.align(..)` like a popover. It is a preview, so use a popover for anything the user has to interact with.

## Open a menu from a button or a right click

A menu is the standard `menu(label, entries).on_activate(choose)`. `entries` is a list of `MenuEntry` and `choose` receives the key of the entry the user picked. `.attach(trigger)` opens it from a button; `.attach_context(target)` opens it on a right click on `target`:

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

#[component]
fn account_menu() -> impl Element {
    let picked = signal(String::from("Nothing chosen yet."));
    let entries = vec![
        MenuEntry::heading("My account"),
        MenuEntry::item("profile", "Profile").with_icon("user").with_shortcut("Ctrl+P"),
        MenuEntry::item("settings", "Settings").with_icon("settings").disabled(true),
        MenuEntry::separator(),
        MenuEntry::item("logout", "Log out").with_icon("log-out").destructive(true),
    ];
    column((
        menu("Account", entries).on_activate(move |key: &'static str| {
            picked.set(format!("Chosen: {key}"))
        })
        .at_least_anchor_width()
        .attach(button("Account").variant(kivact::ui::ButtonVariant::Outline)),
        basic::muted(picked),
    ))
    .style(style().gap(12))
}

#[component]
fn context_menu() -> impl IntoView {
    let area = node()
        .role(Role::Group)
        .name("Notes")
        .focusable(true)
        .style(style().width(240).height(96).border(1, basic::BORDER))
        .child(text("Right click here"));
    menu("Notes menu", vec![MenuEntry::item("copy", "Copy"), MenuEntry::item("paste", "Paste")]).on_activate(|_key: &'static str| {})
    .attach_context(area)
}
```

- Entries: `item`, `checkbox(key, label, checked)`, `radio`, `heading` (a label), `separator` and `submenu(key, label, entries)`. An item can have `.icon(name)`, `.shortcut(text)`, `.disabled(true)` and `.destructive(true)`. A modifier that does not apply to the entry's kind is a usage error.
- A menu does not keep state. For check and radio items, make `entries` a closure that reads your signals, and add `.keep_open(true)` so a pick does not close the menu; your `choose` closure updates the signal and the next list shows the new marks:

```rust,no_run
use kivact::prelude::*;

#[component]
fn view_menu() -> impl IntoView {
    let status = signal(true);
    let position = signal("bottom");
    let entries = move || {
        vec![
            MenuEntry::heading("Appearance"),
            MenuEntry::checkbox("status", "Status bar", status.get()).keep_open(true),
            MenuEntry::separator(),
            MenuEntry::radio("top", "Top", position.get() == "top").keep_open(true),
            MenuEntry::radio("bottom", "Bottom", position.get() == "bottom").keep_open(true),
        ]
    };
    menu("View", entries).on_activate(move |key: &'static str| match key {
        "status" => status.update(|on| *on = !*on),
        other => position.set(other),
    })
    .attach(button("View").variant(kivact::ui::ButtonVariant::Outline))
}
```

- A menu bar across the top of a window is `basic::menubar(menus).on_activate(choose)`; see [Navigation](navigation.md).

## Show a toast

Mount the toast region once, near the root of the app. Then any handler can push a toast:

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::{Toast, Tone, use_toaster};
use kivact::ui::{toast_region};

#[component]
fn root() -> impl Element {
    column((
        button("Create event")
            .variant(kivact::ui::ButtonVariant::Outline)
            .on_press(|| {
                use_toaster().push(
                    Toast::new("Event has been created")
                        .tone(Tone::Success)
                        .body("Sunday, December 03, 2023 at 9:00 AM"),
                );
            }),
        toast_region(use_toaster()).hold_all_on_hover(true),
    ))
}
```

- `.tone(..)` is `Neutral` (no icon), `Success`, `Info`, `Warning` or `Danger`. The theme draws the icon of the tone, and `Warning` and `Danger` are announced more urgently to screen readers.
- A toast closes after five seconds. `.duration(Some(Duration::from_secs(2)))` changes that and `.duration(None)` keeps it until it is dismissed. `push` returns an id: `use_toaster().dismiss(id)` removes the toast.
- `.action("Undo", move || ..)` adds a button. Pressing it runs the closure and dismisses the toast. The closure belongs to the toast, so a press handler can build and push one.
- For a toast that shows work in progress, use `.icon(|| kivact::ui::spinner())` with `.duration(None)`, then dismiss it when the work finishes.
- Several toasts stack behind the newest, up to three. They open into a list while the pointer is over them or focus is inside, and with `hold_all_on_hover(true)` the time of every toast pauses while they are open.

## See also

- [Forms](forms.md): put a dialog's content into fields, and validate before closing it.
- [Navigation](navigation.md): menu bars, the navigation menu and sidebars.
- [Theme and tokens](theme-and-tokens.md): how overlays follow a theme switch.
- Rustdoc: `cargo doc -p kivact-ui-basic --open` for `drawer` and `hover_card`; `cargo doc -p kivact-ui --open` for `dialog`, `menu` and `Toast`.
