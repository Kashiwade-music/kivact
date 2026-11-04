# Step 5: Lists and conditions

[日本語](../../jp/getting-started/05-lists-and-conditions.md)

In this step the placeholder list becomes real. Tasks live in a signal, each row has a checkbox and a Remove button, and an empty list shows a hint.

## The program so far

```rust,no_run
use kivact::prelude::*;

#[derive(Clone, PartialEq)]
struct Task {
    id: u32,
    title: String,
    done: bool,
}

fn header() -> impl IntoView {
    column((
        text("Tasks"),
        text("Things to do today"),
    ))
    .style(style().gap(4))
}

fn task_row(tasks: Signal<Vec<Task>>, task: Binding<Task>) -> impl IntoView {
    let id = task.get_untracked().id;
    let done = bind!(task.done);
    row((
        checkbox(move || task.get().title, done),
        button("Remove").on_press(move || tasks.update(|list| list.retain(|task| task.id != id))),
    ))
    .style(style().gap(8))
}

fn task_list(tasks: Signal<Vec<Task>>) -> impl IntoView {
    column(for_each(
        tasks,
        |task| task.id,
        move |task| task_row(tasks, task),
    ))
    .style(style().gap(8))
}

fn app() -> impl IntoView {
    let tasks = signal(vec![
        Task { id: 1, title: "Buy milk".to_owned(), done: false },
        Task { id: 2, title: "Write the tutorial".to_owned(), done: false },
        Task { id: 3, title: "Go for a walk".to_owned(), done: false },
    ]);
    let next_id = var(4);
    let draft = signal(String::new());
    let left = memo(move || tasks.with(|list| list.iter().filter(|task| !task.done).count()));

    let add = move || {
        let title = draft.get().trim().to_owned();
        if !title.is_empty() {
            let id = next_id.get();
            next_id.set(id + 1);
            tasks.update(|list| list.push(Task { id, title, done: false }));
            draft.set(String::new());
        }
    };

    column((
        header(),
        row((
            text_input("New task", draft)
                .placeholder("What needs doing?")
                .on_submit(add)
                .style(style().width(240)),
            button("Add")
                .disabled(move || draft.with(|text| text.trim().is_empty()))
                .on_press(add),
        ))
        .style(style().gap(8)),
        show(move || tasks.with(|list| !list.is_empty()), move || task_list(tasks))
            .fallback(|| text("Nothing to do. Add a task above.")),
        text(move || format!("{} left", left.get())),
    ))
    .style(style().padding(24).gap(16))
}

fn main() -> kivact::Result<()> {
    App::new("Tasks").run(app)
}
```

The stand-in counter and button from step 3 are gone. The remaining count now comes from the tasks.

## What is new

**A task is a struct.** `Task` holds an `id`, a `title` and a `done` flag. Kivact needs two derives on anything it keeps in a list: `Clone`, because it copies items into rows, and `PartialEq`, because it compares old and new items to find out what changed. `tasks` is a `Signal<Vec<Task>>`. You change it with `update`: `push` adds a task, and `retain` keeps only the tasks that match, which removes one.

**`var`** is a plain mutable variable that the screen does not watch. `next_id` hands out unique ids. Changing it does not update anything, which is exactly what you want for a counter that only the `add` closure uses. Use `signal` for anything the screen shows.

**`for_each(items, key, row)`** builds one row per item.

- `items` is the signal that holds the list.
- `key` returns a value that identifies an item, here `task.id`. Rows are matched by key, so when you remove the second task, Kivact removes only that row and keeps the others, together with any state they hold.
- `row` is called once for each new key. It receives one `Binding<Task>`, a reactive handle that reads and writes the current item. Read it with `task.get()`.

`column(for_each(..))` places the rows in a column.

**Binding a checkbox to one field.** `checkbox(name, binding)` shows a checkbox that edits a `bool`. The row's `done` flag lives inside the list, so
`bind!(task.done)` (or `task.project(|task| &task.done, |task| &mut task.done)`)
builds a two-way binding to just that field of the task with this id. Clicking the checkbox writes into the list, and the `left` memo recounts. The first closure after `project` reads the field and the second gives write access to it.

`checkbox` takes a closure as its name, so the label follows the title. Its name is also what a screen reader announces.

**Why `task_row` takes `tasks`.** The Remove button must change the whole list, not just the row's item, so the row needs the list signal. A `Signal` is `Copy`, so passing it down costs nothing.

**`show(condition, view).fallback(other)`** shows `view` while the condition is true and `other` while it is false. The condition is a closure over a signal, like other reactive properties. Here the list appears when there is at least one task, and the hint appears when there is none. When the branch switches, Kivact builds the new branch and discards the old one.

## Run it

```text
cargo run
```

You see the three tasks, each with a checkbox and a Remove button, and "3 left" at the bottom. Check "Buy milk": the count drops to "2 left". Add a task with the field: it appears at the end of the list. Remove every task: the list is replaced by "Nothing to do. Add a task above."

Next: [Step 6: Style and accessibility](06-style-and-accessibility.md)
