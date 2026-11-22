# Step 7: Async

[日本語](../../jp/getting-started/07-async.md)

In this step the app loads its saved tasks when it starts. Loading takes a moment, and it can fail, so the app shows a loading message, an error with a retry button, or the task list.

## The program so far

```rust,no_run
use std::time::Duration;

use kivact::prelude::*;

const LOGO: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><rect width="24" height="24" rx="6" fill="#336699"/><path d="M6 12.5l4 4 8-9" stroke="white" stroke-width="2.5" fill="none"/></svg>"##;

#[derive(Clone, PartialEq)]
struct Task {
    id: u32,
    title: String,
    done: bool,
}

#[component]
fn header(dark: Signal<bool>) -> impl Element {
    row((
        image(LOGO).name("Tasks logo").style(style().size(32, 32)),
        column((
            text("Tasks").role(Role::Heading).level(1).style(style().font_size(28).font_weight(700)),
            text("Things to do today").style(style().color(color::TEXT_MUTED)),
        ))
        .style(style().gap(2).flex_grow(1)),
        switch("Dark mode", dark),
    ))
    .style(style().gap(12).align_items(Align::Center))
}

#[component]
fn task_row(tasks: Signal<Vec<Task>>, task: Signal<Task>) -> impl Element {
    let id = task.get().id;
    row((
        checkbox(move || task.get().title, bind!(task.done)),
        button("Remove")
            .name(move || format!("Remove {}", task.get().title))
            .variant(ButtonVariant::Ghost)
            .on_press(move || tasks.update(|list| list.retain(|task| task.id != id))),
    ))
    .role(Role::ListItem)
    .style(style().gap(8).align_items(Align::Center).justify_content(Justify::SpaceBetween))
}

#[component]
fn task_list(tasks: Signal<Vec<Task>>) -> impl Element {
    column(for_each(
        tasks,
        |task| task.id,
        move |task| task_row(tasks, task),
    ))
    .role(Role::List)
    .name("Task list")
    .style(
        style()
            .gap(8)
            .padding(space::MD)
            .border_radius(radius::MD)
            .background(color::SURFACE_ALT)
            .border(1, color::BORDER),
    )
}

#[component]
fn tasks_screen(dark: Signal<bool>, saved: Vec<Task>) -> impl Element {
    let next_id = var(saved.iter().map(|task| task.id).max().unwrap_or(0) + 1);
    let tasks = signal(saved);
    let draft = signal(String::new());
    let left = memo(move || tasks.with(|list| list.iter().filter(|task| !task.done).count()));

    let add = callback(move |_| {
        let title = draft.get().trim().to_owned();
        if !title.is_empty() {
            let id = next_id.get();
            next_id.set(id + 1);
            tasks.update(|list| list.push(Task { id, title, done: false }));
            draft.set(String::new());
        }
    });

    column((
        header(dark),
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
        show(move || tasks.with(|list| !list.is_empty()), move || task_list(tasks))
            .fallback(|| text("Nothing to do. Add a task above.")),
        text(move || format!("{} left", left.get())).style(style().color(color::TEXT_MUTED)),
    ))
    .style(
        style()
            .padding(24)
            .gap(16)
            .width(pct(100))
            .height(pct(100))
            .background(color::SURFACE)
            .color(color::TEXT),
    )
}

/// Pretends to read the saved tasks from disk. The first attempt fails so that
/// you can see the error state; every later attempt succeeds.
async fn load_tasks(attempt: u32) -> kivact::Result<Vec<Task>> {
    kivact::task::sleep(Duration::from_millis(600)).await;
    if attempt == 1 {
        return Err(UiError::msg("Could not read the saved tasks"));
    }
    Ok(vec![
        Task { id: 1, title: "Buy milk".to_owned(), done: false },
        Task { id: 2, title: "Write the tutorial".to_owned(), done: false },
        Task { id: 3, title: "Go for a walk".to_owned(), done: false },
    ])
}

#[component]
fn loading_screen(dark: Signal<bool>) -> impl Element {
    // Counts the loads, so that the fake loader can fail only the first one.
    let attempts = var(0);
    let saved = resource((), move |()| {
        attempts.update(|n| *n += 1);
        load_tasks(attempts.get())
    });
    async_view(saved, move |tasks| tasks_screen(dark, tasks))
        .loading(|| text("Loading tasks...").role(Role::Status).style(style().padding(24)))
        .fallback(|error, retry| {
            column((
                text(format!("Error: {error}")).role(Role::Alert),
                button("Try again").on_press(retry),
            ))
            .style(style().padding(24).gap(12))
        })
}

fn app() -> impl IntoView {
    let dark = signal(false);
    provide_theme(
        move || if dark.get() { Theme::dark() } else { Theme::light() },
        move || loading_screen(dark),
    )
}

fn main() -> kivact::Result<()> {
    App::new("Tasks").run(app)
}
```

`tasks_screen` now receives the loaded tasks instead of creating them. The rest of the screen is unchanged from step 6.

## What is new

**`resource(source, load)`** starts an asynchronous job and keeps its progress as state.

- `source` is the input of the job. Kivact starts the job once at the beginning, and again whenever `source` changes; with a signal as the source, each change makes a new request and a request still running is discarded. Here the source is `()`, which never changes, so the job runs again only when you ask for it: `retry` in the error view below, or `saved.refetch()`.
- `load` receives the current source value and returns a future of `kivact::Result<T>`: `Ok(value)` on success or `Err(UiError::msg("..."))` on failure. Use `?` on an error implementing `std::error::Error + Send + Sync + 'static`, and it converts into a `UiError`.

`load_tasks` stands in for reading a file or calling a server. `kivact::task::sleep(..).await` waits without blocking the window, so the window stays responsive. Slow synchronous work, such as reading a big file, belongs in `kivact::task::blocking(..)`.

To let you see every state, the fake loader fails on the first attempt. `attempts` is a `var` (step 5) that the loader increments on every call. Remove the `if attempt == 1` check when you are done experimenting.

**`async_view(resource, ready)`** shows the state of the job, like `<Suspense>` together with an error boundary:

- `.loading(..)` builds the view shown while nothing has arrived yet.
- `ready` builds the view from the value once the job succeeds. Here it builds the whole task screen from the loaded tasks.
- `.fallback(|error, retry| ..)` builds the view shown when the job failed. `retry` is a `Callback<()>` that starts the job again; give it straight to `on_press`.

Without `.loading(..)` and `.fallback(..)`, `async_view` shows a standard loading indicator and an error message with a retry button.

The `ready` view lives as long as that result is shown. `tasks_screen` creates the `tasks` signal when the data arrives, and "Try again" starts a new load, which builds a fresh screen from the new result. Keep state that must survive a reload outside `async_view`, and pass its handles in.

When you need the states yourself, `saved.state()` returns an `AsyncState` (`Idle`, `Pending`, `Failed` or `Ready`); [Run async work from a component](../how-to-guide/async-and-tasks.md) shows how.

**Accessible roles for status messages.** `Role::Status` marks the loading message and `Role::Alert` marks the error, so a screen reader announces them when they appear.

## Run it

```text
cargo run
```

The window shows "Loading tasks..." for about half a second, then "Error: Could not read the saved tasks" and a "Try again" button. Press the button: the loading message appears again, and then the task list you built in step 6, with "3 left" at the bottom.

Next: [Step 8: Testing](08-testing.md)
