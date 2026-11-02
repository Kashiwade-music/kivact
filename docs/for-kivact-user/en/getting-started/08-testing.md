# Step 8: Testing

[日本語](../../jp/getting-started/08-testing.md)

In this last step you write automated tests that use the app the way a person does: they type, click and read what is on screen. You test the loading and error states, adding, checking and removing tasks, and the empty state.

## Set up

Tests use `kivact::test`, which is behind the `test` feature. Enable it for tests only, in `Cargo.toml`:

```toml
[dev-dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact", features = ["test"] }
```

## The program with tests

This is the program from step 7 with a `tests` module added at the end. Everything above `#[cfg(test)]` is unchanged.

```rust,test_harness
use std::time::Duration;

use kivact::prelude::*;
use kivact::view::provide_theme;

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

fn task_row(tasks: Signal<Vec<Task>>, id: u32, task: Value<Task>) -> impl IntoView {
    let done = tasks.item(|task| task.id, id).field(|task| &task.done, |task| &mut task.done);
    row((
        checkbox(move || task.get().title, done),
        button("Remove")
            .name(move || format!("Remove {}", task.get().title))
            .variant(ButtonVariant::Ghost)
            .on_press(move || tasks.update(|list| list.retain(|task| task.id != id))),
    ))
    .role(Role::ListItem)
    .style(style().gap(8).align_items(Align::Center).justify_content(Justify::SpaceBetween))
}

fn task_list(tasks: Signal<Vec<Task>>) -> impl IntoView {
    column(for_each(
        tasks,
        |task| task.id,
        move |id, task| task_row(tasks, id, task),
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

#[cfg(test)]
mod tests {
    use super::*;
    use kivact::test::Screen;

    const LOAD: Duration = Duration::from_millis(600);

    /// Opens the app and gets past the simulated failure of the first load.
    fn open() -> Screen {
        let screen = kivact::test::render(app);
        screen.advance(LOAD);
        screen.get_by_role(Role::Button, "Try again").click();
        screen.advance(LOAD);
        screen
    }

    fn add(screen: &Screen, title: &str) {
        screen.get_by_role(Role::TextBox, "New task").type_text(title);
        screen.get_by_role(Role::Button, "Add").click();
    }

    #[test]
    fn the_first_load_fails_and_trying_again_shows_the_tasks() {
        let screen = kivact::test::render(app);
        screen.get_by_text("Loading tasks...");

        screen.advance(LOAD);
        screen.get_by_text("Error: Could not read the saved tasks");

        screen.get_by_role(Role::Button, "Try again").click();
        screen.advance(LOAD);
        screen.get_by_role(Role::CheckBox, "Buy milk");
        screen.get_by_text("3 left");
    }

    #[test]
    fn adding_a_task_clears_the_field() {
        let screen = open();
        let field = || screen.get_by_role(Role::TextBox, "New task");
        assert!(screen.get_by_role(Role::Button, "Add").is_disabled());

        add(&screen, "Water the plants");
        screen.get_by_role(Role::CheckBox, "Water the plants");
        screen.get_by_text("4 left");
        assert_eq!(field().value().as_deref(), Some(""));
        assert!(screen.get_by_role(Role::Button, "Add").is_disabled());
    }

    #[test]
    fn enter_adds_the_task_too() {
        let screen = open();
        screen.get_by_role(Role::TextBox, "New task").type_text("Call Sam");
        screen.get_by_role(Role::TextBox, "New task").press_key(Key::Enter);
        screen.get_by_role(Role::CheckBox, "Call Sam");
    }

    #[test]
    fn checking_a_task_lowers_the_count() {
        let screen = open();
        screen.get_by_role(Role::CheckBox, "Buy milk").click();
        assert_eq!(screen.get_by_role(Role::CheckBox, "Buy milk").checked(), Some(CheckedState::Checked));
        screen.get_by_text("2 left");
    }

    #[test]
    fn removing_every_task_shows_the_hint() {
        let screen = open();
        screen.get_by_role(Role::Button, "Remove Buy milk").click();
        assert!(screen.query_by_role(Role::CheckBox, "Buy milk").is_none());
        screen.get_by_role(Role::Button, "Remove Write the tutorial").click();
        screen.get_by_role(Role::Button, "Remove Go for a walk").click();
        screen.get_by_text("Nothing to do. Add a task above.");
    }
}
```

## What is new

**`#[cfg(test)] mod tests`** is compiled only by `cargo test`. Each `#[test]` function is one test.

**`kivact::test::render(app)`** builds the app in a real window and returns a `Screen`. The window opens without taking focus from what you are doing, and it closes when the test ends. The screen runs on a **virtual clock**: a test never waits in real time, which is why `screen.advance(LOAD)` moves the clock forward by 600 milliseconds, long enough for the simulated load to finish.

**Queries find elements the way a user or a screen reader would.**

- `get_by_role(Role::Button, "Add")` finds a control by its role and accessible name. This is why step 6 named every control.
- `get_by_text("3 left")` finds text on screen.
- `get_by_*` fails the test with a readable message if there is no match, or more than one. `query_by_*` returns an `Option` instead, which is what you use to assert that something is *not* there, as in `query_by_role(..).is_none()`. `all_by_*` returns a `Vec`.

**Actions** are real input. `.click()` presses the element, `.type_text("..")` types into a text field and `.press_key(Key::Enter)` presses a key. They check that a user could really do it: clicking a disabled or covered element fails the test.

**Assertions** read the state of an element: `.is_disabled()`, `.checked()` (an `Option<CheckedState>`) and `.value()` (the text in a field). Use plain `assert!` and `assert_eq!` for these.

**Helpers keep tests short.** `open()` and `add()` are ordinary functions that hold the repeated steps. A fresh `render` per test also means that tests do not share state.

## Run it

```text
cargo test
```

Windows flash up briefly while the tests run, then you see:

```text
running 5 tests
test tests::adding_a_task_clears_the_field ... ok
test tests::checking_a_task_lowers_the_count ... ok
test tests::enter_adds_the_task_too ... ok
test tests::removing_every_task_shows_the_hint ... ok
test tests::the_first_load_fails_and_trying_again_shows_the_tasks ... ok

test result: ok. 5 passed; 0 failed
```

Change `"3 left"` in the first test to `"4 left"` and run again. The test fails, and the message lists what was on screen. Kivact also saves a screenshot of the failing window and the accessibility tree of it in `target/kivact-test-failures/`; the failure message gives the exact path. Change it back afterwards.

Two points to know:

- A test binary that calls `kivact::test::render` must not call `App::run` in a test, because all test windows share one event loop. Calling `main` from a test is therefore not possible; test the component (`app`) instead.
- Only one process should open test windows at a time on your machine, so do not run several `cargo test` commands at once.

## Where to go next

You have built a complete app: layout, state, input, lists, styling, async loading and tests.

- [How-to guides](../how-to-guide/README.md) solve specific problems, such as form validation, dialogs and menus, in more detail than this tutorial.
- [UI kits](../../../for-kivact-ui-user/en/README.md) restyle the standard components and add more of them. Try swapping the default look of Tasks for one of the kits.
- For the full API, run `cargo doc -p kivact --open`.
