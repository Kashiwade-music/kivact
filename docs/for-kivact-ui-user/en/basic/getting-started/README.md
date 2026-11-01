# Tutorial: build a settings screen with the Basic kit

[日本語](../../../jp/basic/getting-started/README.md)

`kivact-ui-basic` is Kivact's neutral, general-purpose kit. It follows the look of the shadcn/ui "Lyra" preset: Neutral colors, square corners, 12 px text and Lucide icons. In this tutorial you build one app, a "Settings" screen, and it grows with every step. You never start over: each page continues the program of the previous one.

By the end you have a window with a theme you can switch between light and dark, two cards of inputs and switches, a toast after saving, a confirmation dialog, and a test.

## Before you start

Finish, or at least read, the [core tutorial](../../../../for-kivact-user/en/getting-started/README.md). It explains signals, components as plain functions, layout and styles, which this tutorial uses without re-explaining them. You also need the Rust toolchain the repository pins (1.98.1, edition 2024). Windows and Linux (Wayland or X11) are the target platforms.

Kivact is in early development and its API changes without notice; see the [kits overview](../../README.md) for the current state of every kit.

To see what the kit contains before you begin, clone the repository and run the catalog, a window that shows every component with its source code:

```text
cargo run -p kivact-ui-basic-catalog
```

## Steps

| Step | You add | You see |
|---|---|---|
| [1. Set up the kit](01-set-up-the-kit.md) | `kivact-ui-basic`, the theme, the font hook | A window with a heading in the kit's look, then in its dark palette |
| [2. Build the settings form](02-build-the-settings-form.md) | Cards, fields, inputs, switches, a select, a toggle group | Two cards you can edit, and an "Unsaved changes" badge |
| [3. Dark mode and feedback](03-dark-mode-and-feedback.md) | A theme switch, a toast, an alert dialog, a test | The whole screen turning dark, a toast on save, a confirmation before deleting |

When you finish, the [how-to guides](../how-to-guide/README.md) show how to solve particular tasks with the kit.
