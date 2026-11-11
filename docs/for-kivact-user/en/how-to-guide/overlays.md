# Show a dialog, menu, tooltip or toast

[日本語](../../jp/how-to-guide/overlays.md)

Overlays are the layers that float above the screen. Kivact's standard
components give each of them focus handling, Escape, outside-press dismissal and
accessibility for free: `dialog` for modal windows and popovers, `menu` for
action menus and context menus, `tooltip` for hints and `toast_region` for
notifications. This guide shows each one.

## Show a dialog

The open state of a dialog is a `bool` you own. Write `true` to open it, and the
dialog writes `false` when the user closes it.

```rust,test_harness
use kivact::prelude::*;

fn app() -> impl IntoView {
    let open = signal(false);
    let deleted = signal(false);

    column((
        button("Delete note").on_press(move || open.set(true)),
        text(move || if deleted.get() { "Deleted" } else { "Kept" }),
        dialog("Delete this note?", open)
            .mode(DialogMode::AlertModal)
            .dismiss(DismissOn { escape: true, outside: false })
            .description("This cannot be undone.")
            .content(|| text("The note and its attachments will be removed."))
            .footer(move || {
                row((
                    button("Cancel")
                        .variant(ButtonVariant::Outline)
                        .on_press(move || open.set(false)),
                    button("Delete")
                        .variant(ButtonVariant::Danger)
                        .on_press(move || {
                            deleted.set(true);
                            open.set(false);
                        }),
                ))
                .style(style().gap(8))
            }),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Notes").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_closes_the_dialog_and_returns_the_focus() {
        let screen = kivact::test::render(app);
        let opener = screen.get_by_role(Role::Button, "Delete note");
        opener.focus();
        opener.click();
        screen.get_by_role(Role::AlertDialog, "Delete this note?");

        screen.press_key(Key::Escape);
        assert!(screen.query_by_role(Role::AlertDialog, "Delete this note?").is_none());
        assert!(opener.is_focused());

        opener.click();
        screen.get_by_role(Role::Button, "Delete").click();
        screen.get_by_text("Deleted");
    }
}
```

What the dialog does for you:

- **Focus.** Focus moves into the dialog when it opens (the first control in the
  content, or the target of `.initial_focus(node_ref)`), Tab stays inside a
  modal, and when it closes focus returns to the control that opened it.
- **Closing.** Escape and a press outside close it unless you change
  `.dismiss(Dismiss { escape, outside })`. The header close button is on by
  default; `.closable(false)` removes it. A modal blocks pointer and keyboard
  input to the rest of the screen.
- **Accessibility.** The title is the dialog's accessible name and its visible
  heading; `.hide_title(true)` keeps the name but draws no heading, and
  `.description(..)` adds a description.
- **Width.** `.width(480)` overrides the theme's width.

`DialogMode` chooses the style:

| Kind | Looks like | Outside press |
| --- | --- | --- |
| `Modal` (default) | a centered dialog | closes |
| `AlertModal` | a centered dialog that asks for an answer (`Role::AlertDialog`) | does not close unless `Dismiss` says so |
| `Drawer(Edge::Right)` | a sheet fixed to an edge (`Left`, `Right`, `Top`, `Bottom`) | closes |
| `Popover` | a non-modal panel next to its trigger | closes |

### The content is rebuilt on every open

`.content(..)` and `.footer(..)` are factories: they run each time the dialog
opens, in a fresh owner. A `signal` created inside the factory is dropped when
the dialog closes. Create anything that must survive a close, such as the
text of a form, **outside** the factory and capture the handle:

```rust,no_run
use kivact::prelude::*;

fn rename(name: Signal<String>) -> impl IntoView {
    let open = signal(false);
    let draft = signal(String::new()); // survives the dialog closing

    column((
        button("Rename").on_press(move || {
            draft.set(name.get());
            open.set(true);
        }),
        dialog("Rename", open)
            .content(move || text_input("Name", draft))
            .footer(move || {
                button("Save").on_press(move || {
                    name.set(draft.get());
                    open.set(false);
                })
            }),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Rename").run(|| rename(signal("Untitled".to_owned())))
}
```

### Open from a trigger, or as a popover

`.attach(trigger)` makes the trigger open the dialog and returns the trigger and
dialog together. A `Popover` needs it: the panel is placed at the trigger and a
second press on the trigger closes it again.

```rust,no_run
use kivact::prelude::*;

fn help() -> impl IntoView {
    let open = signal(false);
    dialog("Help", open)
        .mode(DialogMode::Popover)
        .content(|| text("Press Tab to move between fields."))
        .attach(button("Help"))
}

fn main() -> kivact::Result<()> {
    App::new("Help").run(help)
}
```

Use `.placement(Placement::Above)` and `.align(OverlayAlign::End)` (both in
`kivact::view`) to choose the side and the alignment. If the panel does not fit,
Kivact flips it to the other side.

## Add a tooltip

`tooltip(text).attach(trigger)` shows a hint after a short delay when the
pointer rests on the trigger or the trigger has keyboard focus, and hides it on
Escape or when the pointer leaves. Only one tooltip is shown at a time. The hint
also becomes the trigger's description for assistive technology, unless the
trigger already has one.

```rust,no_run
use kivact::prelude::*;

fn app() -> impl IntoView {
    row((
        tooltip("Save the document (Ctrl+S)").attach(button("Save")),
        tooltip("Open another file").attach(button("Open")),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Tooltips").run(app)
}
```

A tooltip is a short hint. It is not for text people must read or for anything
they must click; use a popover for that. `.placement(..)`, `.align(..)` and
`.offset(px)` position it, and `.content(|| view)` replaces the plain text with
your own view.

## Show a menu

`menu(label, entries).on_activate(choose)` draws a button named `label` that opens the
entries. `choose` receives the key of the entry the user picked. The menu keeps
no state, so check marks come from your own signals.

```rust,test_harness
use kivact::prelude::*;

fn app() -> impl IntoView {
    let show_grid = signal(true);
    let last = signal(String::from("nothing"));

    let entries = move || {
        vec![
            MenuEntry::item("new", "New").with_shortcut("Ctrl N"),
            MenuEntry::item("open", "Open"),
            MenuEntry::separator(),
            MenuEntry::checkbox("grid", "Show grid", show_grid.get()).keep_open(true),
            MenuEntry::submenu(
                "export",
                "Export",
                vec![MenuEntry::item("png", "PNG"), MenuEntry::item("svg", "SVG")],
            ),
            MenuEntry::item("delete", "Delete").destructive(true),
        ]
    };

    column((
        menu("File", entries).on_activate(move |key: &'static str| {
            if key == "grid" {
                show_grid.update(|on| *on = !*on);
            }
            last.set(key.to_owned());
        }),
        text(move || format!("last: {}", last.get())),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Menus").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn choosing_an_item_reports_its_key() {
        let screen = kivact::test::render(app);
        screen.get_by_role(Role::Button, "File").click();
        screen.get_by_role(Role::MenuItem, "Open").click();
        screen.get_by_text("last: open");
    }
}
```

- Entries are built with `MenuEntry::item`, `checkbox(key, label, checked)`,
  `radio`, `submenu`, `separator`, `heading` and `group`. The constructor
  chooses the kind of the row. Modifiers add `.shortcut("Ctrl N")` (text only;
  the menu binds no key), `.icon(..)`, `.description(..)`, `.disabled(bool)`,
  `.destructive(true)` and `.keep_open(true)`. A modifier that does not apply to
  the entry's kind (a `shortcut` on a separator, `keep_open` on a submenu) is a
  usage error.
- Pass a closure for `entries` (as above) when check marks or labels depend on
  signals; the menu re-reads it. A plain `Vec` is fine for a fixed menu.
- The keys are yours. Use `&'static str` or an enum with `Clone + Eq + Hash`.
- Arrow keys, Enter, Space, Right and Left (for submenus), Escape, and typing the
  first letters of an item all work. Choosing an item closes the menu unless the
  entry has `.keep_open(true)`.

To open the menu from your own control, use `.attach(trigger)`. To open it on a
right click, or the context-menu key, use `.attach_context(target)`; the menu
then appears at the pointer.

```rust,no_run
use kivact::prelude::*;

fn app() -> impl IntoView {
    let entries = vec![
        MenuEntry::item("copy", "Copy"),
        MenuEntry::item("paste", "Paste"),
    ];
    menu("Edit", entries).on_activate(|_key: &'static str| {})
        .attach_context(node().child(text("Right-click me")).style(style().padding(48)))
}

fn main() -> kivact::Result<()> {
    App::new("Context menu").run(app)
}
```

`.placement(..)`, `.align(..)`, `.offset(px)` and `.at_least_anchor_width()`
control where a menu opens.

## Show toasts

A toast is a short notification that disappears by itself. Mount one
`toast_region` near the root of the app and push `Toast` values from anywhere.

```rust,test_harness
use std::time::Duration;

use kivact::prelude::*;

fn app() -> impl IntoView {
    let toaster = toaster();
    let count = signal(0);

    column((
        button("Save").on_press(move || {
            toaster.push(
                Toast::new("Saved")
                    .body("Your changes were written to disk.")
                    .tone(Tone::Success),
            );
        }),
        button("Delete").on_press(move || {
            count.update(|n| *n += 1);
            toaster.push(
                Toast::new("Deleted")
                    .tone(Tone::Danger)
                    .duration(Some(Duration::from_secs(10)))
                    .action("Undo", move || count.update(|n| *n -= 1)),
            );
        }),
        text(move || format!("deleted: {}", count.get())),
        toast_region(toaster),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Toasts").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_toast_appears_and_its_action_runs() {
        let screen = kivact::test::render(app);
        screen.get_by_role(Role::Button, "Save").click();
        screen.find_by_text("Saved");

        screen.get_by_role(Role::Button, "Delete").click();
        screen.get_by_text("deleted: 1");
        screen.find_by_role(Role::Button, "Undo").click();
        screen.get_by_text("deleted: 0");
    }
}
```

- `toaster()` returns the app's single shared queue, so you can call it in any
  component and every call gets the same queue. Pass it to a handler by capturing
  it in a closure; it is a `Copy` handle.
- A toast stays 5 seconds by default. `.duration(Some(d))` changes it and
  `.duration(None)` keeps it until the user closes it. The timer stops while the
  pointer is over the toast or focus is inside it.
- `Tone` (`Neutral`, `Info`, `Success`, `Warning`, `Danger`, `Accent`) sets the
  icon and how it is announced: `Warning` and `Danger` are announced at once,
  the others politely.
- `.action(label, f)` adds one button that runs `f` and dismisses the toast.
- At most three toasts show at a time (`toaster.max_visible(n)`); the rest wait.
  `toaster.dismiss(id)` removes one early, where `id` is what `push` returned.
- The theme places the region at the bottom right of the window. Mount one
  region near the root of the app.

## See also

- [Forms](forms.md): confirm unsaved edits with a dialog.
- [Theming](theming.md): change how dialogs, menus and toasts look.
- [Internationalization](i18n.md): translate the close button and other built-in
  texts.
- [Write reusable components](custom-components.md): build your own overlay with the
  `kivact::interaction` parts and `kivact::view::overlay`.
- Reference: `cargo doc -p kivact-ui --open` for `Dialog`, `Menu`, `Tooltip` and
  `ToastRegion`.
