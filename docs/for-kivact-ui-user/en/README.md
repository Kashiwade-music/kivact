# Kivact UI kits

[日本語](../jp/README.md)

A **kit** is a ready-made look for Kivact apps. It is a theme that restyles the
standard components of `kivact::ui` (buttons, inputs, tabs, tables, dialogs and
so on) plus a set of components for its domain, such as date pickers or meters.
Every kit uses only the public `kivact` API, and your app keeps its state in
signals as always: a kit draws what you give it and reports what the user
does. Switching kits changes how your app looks, not how you write it.

Kivact is in early development, and the API changes without notice. The kits
are not on crates.io yet; add them as Git dependencies (each tutorial shows how).

## Choose a kit

| Kit | Crate | Look | Use it for |
| --- | --- | --- | --- |
| Basic | `kivact-ui-basic` | Neutral and square, like the shadcn/ui Lyra preset | A general-purpose app when you have no special style in mind |
| Pop | `kivact-ui-pop` | Calm slate surfaces, soft blue gradients, yellow accents | Business apps: forms, dashboards, tables, schedules |
| Game | `kivact-ui-game` | Hard edges, slanted buttons, segmented meters | Game control panels, HUDs, operation consoles |
| Creative | `kivact-ui-creative` | Dense and compact, in the style of a dark editing tool (a light mode exists) | Creative editing tools: property panels, number fields, color pickers, docks |
| Soft | `kivact-ui-soft` | Astryx Neutral in light and dark appearances | Applications that use rounded controls and neutral surfaces |

Start with the tutorial of the kit you chose, then use its how-to guides when
you have a specific task.

| Kit | Tutorial (you build an app) | How-to guides (one task each) | See it in action |
| --- | --- | --- | --- |
| Basic | [Tutorial](basic/getting-started/README.md) | [Guides](basic/how-to-guide/README.md) | `cargo run -p kivact-ui-basic-catalog` |
| Pop | [Tutorial](pop/getting-started/README.md) | [Guides](pop/how-to-guide/README.md) | `cargo run -p kivact-ui-pop-catalog` |
| Game | [Tutorial](game/getting-started/README.md) | [Guides](game/how-to-guide/README.md) | `cargo run -p kivact-ui-game-catalog` |
| Creative | [Tutorial](creative/getting-started/README.md) | [Guides](creative/how-to-guide/README.md) | `cargo run -p kivact-ui-creative-catalog` |
| Soft | [Tutorial](soft/getting-started/README.md) | [Guides](soft/how-to-guide/README.md) | `cargo run -p kivact-ui-soft-catalog` |

The last column is a catalog app that shows every component of the kit. Run
it from a checkout of the repository: it is the quickest way to compare the
kits.

## Before you start

The tutorials assume you know the basics of Kivact: signals, components and
layout. Finish the [core tutorial](../../for-kivact-user/en/getting-started/README.md)
first if you have not.

## How a kit is used

Every kit works the same way, so what you learn in one carries over:

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;

fn app() -> impl IntoView {
    column((
        text_input("Name", signal(String::new())),
        button("Save").variant(kivact::ui::ButtonVariant::Primary),
    ))
    .style(style().gap(12).padding(24))
}

fn main() -> kivact::Result<()> {
    App::new("Hello")
        .theme(kivact_ui_pop::theme(ThemeMode::Light))
        .run(app)
}
```

Replace `kivact_ui_pop` with `kivact_ui_basic`, `kivact_ui_game`,
`kivact_ui_creative`, or `kivact_ui_soft` to apply that kit's look. The kit's
own components are used as `kivact_ui_pop::name`. They are builders like the
standard ones: they accept `.style(..)`, `.test_id(..)` and the other
modifiers.

## Reference

The full list of components, options and tokens of a kit is its API
documentation, which you generate locally:

```text
cargo doc -p kivact-ui-pop --open
```

Use `kivact-ui-basic`, `kivact-ui-game`, `kivact-ui-creative`, or `kivact-ui-soft` for the other
kits. The standard components themselves are documented under `kivact::ui`
(`cargo doc -p kivact --open`).
