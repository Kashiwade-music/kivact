# Step 1: Create a project

[日本語](../../jp/getting-started/01-create-a-project.md)

In this step you create a Cargo project and open a window that says "Tasks".

## Create the project

```text
cargo new tasks
cd tasks
```

Kivact is not on crates.io yet, so add it as a Git dependency in
`Cargo.toml`:

```toml
[dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact" }
```

Kivact needs Rust 1.98.1 or newer. `cargo new` uses edition 2024 with a
current toolchain, which is what you want.

## The smallest window

Replace the contents of `src/main.rs` with this program:

```rust,no_run
use kivact::prelude::*;

fn app() -> impl IntoView {
    text("Tasks")
}

fn main() -> kivact::Result<()> {
    App::new("Tasks").run(app)
}
```

`use kivact::prelude::*;` brings in the everyday names: `App`, `text`,
`column`, `signal`, `binding`, `callback` and so on. You will use it at the top
of every page.

`App::new("Tasks")` describes the application; the string is the window title.
`.run(app)` opens the window and keeps running until you close it. It calls
`app` once, and the view it returns becomes the content of the window.

`app` is the root **view builder**: an ordinary function returning `impl IntoView`.
`impl IntoView` means "something that can be shown on screen"; you do not need
to name the exact type. `text("Tasks")` is the simplest view, a line of text.

`main` returns `kivact::Result<()>`, so an error while starting, such as no
window being available, is reported when the program exits instead of being
ignored.

## Run it

```text
cargo run
```

The first build takes a while because Kivact compiles from source. Then a
window titled "Tasks" opens with the word "Tasks" in its top-left corner.
Close the window to stop the program.

Next: [Step 2: Components and layout](02-components-and-layout.md)
