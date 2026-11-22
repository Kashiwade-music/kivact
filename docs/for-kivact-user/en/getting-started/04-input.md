# Step 4: Input

[日本語](../../jp/getting-started/04-input.md)

In this step you add a text field and an Add button. The button stays disabled
until you type something, and it clears the field after adding.

## The program so far

```rust,no_run
use kivact::prelude::*;

#[component]
fn header() -> impl Element {
    column((
        text("Tasks"),
        text("Things to do today"),
    ))
    .style(style().gap(4))
}

#[component]
fn task_row(title: Value<String>) -> impl Element {
    row((text("-"), text(title))).style(style().gap(8))
}

#[component]
fn task_list() -> impl Element {
    column((
        task_row("Buy milk"),
        task_row("Write the tutorial"),
        task_row("Go for a walk"),
    ))
    .style(style().gap(8))
}

fn app() -> impl IntoView {
    let completed = signal(0);
    let left = memo(move || 3 - completed.get());
    let draft = signal(String::new());
    let last_added = signal(String::new());

    let add = callback(move |_| {
        let title = draft.get().trim().to_owned();
        if !title.is_empty() {
            last_added.set(title);
            draft.set(String::new());
        }
    });

    column((
        header(),
        task_list(),
        text(move || format!("{} tasks left", left.get())),
        button("Complete one")
            .disabled(move || left.get() == 0)
            .on_press(move || completed.update(|n| *n += 1)),
        row((
            text_input(draft)
                .label("New task")
                .placeholder("What needs doing?")
                .on_submit(add)
                .style(style().width(240)),
            button("Add")
                .disabled(move || draft.with(|text| text.trim().is_empty()))
                .on_press(add),
        ))
        .style(style().gap(8)),
        text(move || {
            let title = last_added.get();
            if title.is_empty() {
                "Nothing added yet".to_owned()
            } else {
                format!("Added: {title}")
            }
        }),
    ))
    .style(style().padding(24).gap(16))
}

fn main() -> kivact::Result<()> {
    App::new("Tasks").run(app)
}
```

The new task does not appear in the list yet. For now the program only
remembers the last title you added and shows it at the bottom. Step 5 turns the
list into real data.

## What is new

**`text_input(draft)`** creates a single-line text field. Its argument is a
`Signal<String>`; the field and the signal are connected in both directions.
Typing updates `draft`, and calling `draft.set(..)` changes what the field
shows. That is how the program clears the field after adding. For a field
inside a larger struct, pass `bind!(profile.name)` to edit that one field of
the struct; step 5 uses the same pattern for task checkboxes.

**`.label("New task")`** gives the field its **name**. Screen readers announce
it, and tests use it to find the field. It is not drawn on screen, which is why
the field also has `.placeholder(..)`, the hint shown while the field is empty.
For a label that is drawn next to the field, write
`field("New task", text_input(draft))` instead: `field` shows the label and
gives it to the field as its name. Every control that can take focus needs a
name. If one has none, Kivact reports it when the control appears, and in a
debug build or a test that report is an error.

**`.on_submit(add)`** runs when you press Enter inside the field.
**`.on_press(add)`** runs when you press the button. Both use the same
action: `callback(move |_| ..)` turns the closure into a `Callback`, a small
`Copy` handle that any number of handlers can share. The `_` is the event the
handler receives; Enter and a press carry none.

**`.disabled(move || ..)`** takes a closure that reads state, like `text` does.
The Add button is disabled while the draft is empty or only spaces. Here
`draft.with(|text| ..)` looks at the string without copying it, whereas
`draft.get()` clones the current string.

The `add` closure also checks for an empty title, because Enter in the field
can run it even when the button is disabled.

## Run it

```text
cargo run
```

Type "Water the plants" in the field. The Add button becomes enabled. Press
Enter or click Add: the field empties and the bottom line reads "Added: Water
the plants". The Add button is disabled again.

Next: [Step 5: Lists and conditions](05-lists-and-conditions.md)
