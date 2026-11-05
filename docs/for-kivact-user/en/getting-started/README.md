# Tutorial: build a to-do app

[日本語](../../jp/getting-started/README.md)

In this tutorial you build **Tasks**, a small to-do app, and it grows a little
on every page. You start with an empty window, then add layout, state, a text
field, a list you can check off and remove, styling, asynchronous loading, and
finally automated tests. After every step you run the app and see the result.

You need:

- A Rust toolchain (install it with [rustup](https://rustup.rs)). Knowing a
  little Rust helps, but the tutorial explains the Rust that Kivact needs
  when it first appears.
- A desktop that can open a window: Windows, or Linux with Wayland or X11.

Follow the pages in order. Each page shows the complete program as it stands
after that step, so you can copy it into `src/main.rs` and run it.

| Step | Page | You will build |
| --- | --- | --- |
| 1 | [Create a project](01-create-a-project.md) | An empty project that opens a window with some text. |
| 2 | [Components and layout](02-components-and-layout.md) | A header and a placeholder list, split into functions. |
| 3 | [State](03-state.md) | A reactive task counter and a map of the state APIs. |
| 4 | [Input](04-input.md) | A text field and an Add button that reads it. |
| 5 | [Lists and conditions](05-lists-and-conditions.md) | A real task list with checkboxes, removal and an empty state. |
| 6 | [Style and accessibility](06-style-and-accessibility.md) | Colors, button variants, accessible names and a dark mode switch. |
| 7 | [Async](07-async.md) | Loading the saved tasks with a delay, with loading and error states. |
| 8 | [Testing](08-testing.md) | App E2E tests on a virtual clock, with an optional mirrored window. |

Start with [Create a project](01-create-a-project.md). If you already use React
or Solid, keep the [concept guide](../how-to-guide/coming-from-react-or-solid.md)
nearby as you follow the tutorial.
