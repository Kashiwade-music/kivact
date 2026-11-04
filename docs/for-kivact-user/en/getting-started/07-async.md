# Step 7: Async

[日本語](../../jp/getting-started/07-async.md)

In this step the app loads its saved tasks when it starts. Loading takes a moment, and it can fail, so the app shows a loading message, an error with a retry button, or the task list.

## The program so far

```rust,no_run
use std::time::Duration;

use kivact::prelude::*;
use kivact::style::provide_theme;

const LOGO: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><rect width="24" height="24" rx="6" fill="#336699"/><path d="M6 12.5l4 4 8-9" stroke="white" stroke-width="2.5" fill="none"/></svg>"##;

#[derive(Clone, PartialEq)]
struct Task {
    id: u32,
    title: String,
    done: bool,
}

fn header(dark: Signal<bool>) -> impl IntoView {
    row((
        image(LOGO).name("Tasks logo").style(style().size(32, 32)),
        column((
            text("Tasks").role(Role::Heading).style(style().font_size(28).weight(700)),
            text("Things to do today").style(style().color(ColorToken::TextMuted)),
        ))
        .style(style().gap(2).grow(1)),
        switch("Dark mode", dark),
    ))
    .style(style().gap(12).align_items(Align::Center))
}

fn task_row(tasks: Signal<Vec<Task>>, task: Binding<Task>) -> impl IntoView {
    let id = task.get_untracked().id;
    let done = bind!(task.done);
    row((
        checkbox(move || task.get().title, done),
        button("Remove")
            .name(move || format!("Remove {}", task.get().title))
            .variant(kivact::ui::ButtonVariant::Ghost)
            .on_press(move || tasks.update(|list| list.retain(|task| task.id != id))),
    ))
    .role(Role::ListItem)
    .style(style().gap(8).align_items(Align::Center).justify_content(Justify::SpaceBetween))
}

fn task_list(tasks: Signal<Vec<Task>>) -> impl IntoView {
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
            .padding(12)
            .radius(8)
            .background(ColorToken::SurfaceAlt)
            .border(1, ColorToken::Border),
    )
}

fn tasks_screen(dark: Signal<bool>, saved: Vec<Task>) -> impl IntoView {
    let next_id = var(saved.iter().map(|task| task.id).max().unwrap_or(0) + 1);
    let tasks = signal(saved);
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
        header(dark),
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
        text(move || format!("{} left", left.get())).style(style().color(ColorToken::TextMuted)),
    ))
    .style(
        style()
            .padding(24)
            .gap(16)
            .width(pct(100))
            .height(pct(100))
            .background(ColorToken::Surface)
            .color(ColorToken::Text),
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

fn loading_screen(dark: Signal<bool>, attempt: Signal<u32>) -> impl IntoView {
    let saved = resource(attempt, load_tasks);
    dynamic(
        move || saved.state(),
        move |state| match state {
            AsyncState::Idle | AsyncState::Pending => text("Loading tasks...")
                .role(Role::Status)
                .style(style().padding(24))
                .into_view(),
            AsyncState::Failed(error) => column((
                text(format!("Error: {error}")).role(Role::Alert),
                button("Try again").on_press(move || attempt.update(|n| *n += 1)),
            ))
            .style(style().padding(24).gap(12))
            .into_view(),
            AsyncState::Ready(tasks) => tasks_screen(dark, tasks).into_view(),
        },
    )
}

fn app() -> impl IntoView {
    let dark = signal(false);
    let attempt = signal(1);
    let theme = memo(move || if dark.get() { Theme::dark() } else { Theme::light() });
    provide_theme(theme, move || loading_screen(dark, attempt))
}

fn main() -> kivact::Result<()> {
    App::new("Tasks").run(app)
}
```

`tasks_screen` now receives the loaded tasks instead of creating them. The rest of the screen is unchanged from step 6.

## What is new

**`resource(source, load)`** starts an asynchronous job and keeps its progress as state.

- `source` is reactive input for the job, here the `attempt` signal. Kivact starts the job once at the beginning, then starts it again whenever `source` changes. If a job is still running when the source changes, its result is discarded.
- `load` is an `async` function that receives the current source value and returns `kivact::Result<T>`: `Ok(value)` on success or `Err(UiError::msg("..."))` on failure. Use `?` on any error type that implements `std::error::Error`, and it converts into a `UiError`.

`load_tasks` stands in for reading a file or calling a server. `kivact::task::sleep(..).await` waits without blocking the window, so the window stays responsive. A `fn` or a closure both work as `load`. Slow synchronous work, such as reading a big file, belongs in `kivact::task::blocking(..)`.

To let you see every state, the fake loader fails on the first attempt. Remove the `if attempt == 1` check when you are done experimenting.

**`saved.state()`** returns an `AsyncState`, which is one of:

- `Idle` or `Pending`: nothing has finished yet.
- `Failed(error)`: the job returned an error.
- `Ready(value)`: the job succeeded with a value.

**`dynamic(source, build)`** picks what to show from a value. Whenever `source` changes, Kivact discards the old view and calls `build` with the new value. Here each state gets its own view, and the `match` makes Rust check that you handled all of them. Because the arms return different kinds of views, each ends in `.into_view()` to give them one common type.

State created inside a branch lives only as long as that branch. `tasks_screen` creates the `tasks` signal when the data arrives, and "Try again" starts a new load, which builds a fresh screen from the new result.

**Accessible roles for status messages.** `Role::Status` marks the loading message and `Role::Alert` marks the error, so a screen reader announces them when they appear.

## Run it

```text
cargo run
```

The window shows "Loading tasks..." for about half a second, then "Error: Could not read the saved tasks" and a "Try again" button. Press the button: the loading message appears again, and then the task list you built in step 6, with "3 left" at the bottom.

Next: [Step 8: Testing](08-testing.md)
