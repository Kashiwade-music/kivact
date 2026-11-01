# Tutorial: build a sign-up form with the Pop kit

[日本語](../../../jp/pop/getting-started/README.md)

Pop (`kivact-ui-pop`) is the Kivact kit for calm, compact business-app
screens. In this tutorial you build **New account**, a sign-up form with text
fields, a plan switch, two dates and a Create button. The app grows a little
on every page, and after every step you run it and see the result.

You need:

- The core tutorial, [Build a to-do app](../../../../for-kivact-user/en/getting-started/README.md),
  or equivalent experience: you know `signal`, `column`, `text` and
  `button`.
- A Rust toolchain and a desktop that can open a window (Windows, or Linux
  with Wayland or X11).

Each page shows the complete program as it stands after that step, so you can
copy it into `src/main.rs` and run it.

| Step | Page | You will build |
| --- | --- | --- |
| 1 | [Set up the kit](01-set-up-the-kit.md) | A window in the Pop look with a name field and a button. |
| 2 | [Build the form](02-build-the-form.md) | Name and email fields with hints, a plan switch and styled buttons. |
| 3 | [Validate and submit](03-validate-and-submit.md) | Date fields, error messages, a success message and a test. |

To see everything the kit offers before you start, run its catalog from a
checkout of the repository:

```text
cargo run -p kivact-ui-pop-catalog
```

Start with [Set up the kit](01-set-up-the-kit.md).
