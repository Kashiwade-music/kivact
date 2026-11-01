# Tutorial: build a loadout panel with the Game kit

[日本語](../../../jp/game/getting-started/README.md)

Game (`kivact-ui-game`) is the Kivact kit for game-style control panels and
HUDs: hard edges, slanted buttons, segmented meters and a dense, compact
layout. In this tutorial you build **Loadout**, the screen where a player
picks a weapon and armor before a mission. The app grows a little on every
page, and after every step you run it and see the result.

You need:

- The core tutorial, [Build a to-do app](../../../../for-kivact-user/en/getting-started/README.md),
  or equivalent experience: you know `signal`, `memo`, `column`, `text` and
  `button`.
- A Rust toolchain and a desktop that can open a window (Windows, or Linux
  with Wayland or X11).

Each page shows the complete program as it stands after that step, so you can
copy it into `src/main.rs` and run it.

| Step | Page | You will build |
| --- | --- | --- |
| 1 | [Set up the kit](01-set-up-the-kit.md) | A window in the Game look with a framed panel and a slanted button. |
| 2 | [Build the panel](02-build-the-panel.md) | A heading, a status badge, meters and buttons that change them. |
| 3 | [Tabs and items](03-tabs-and-items.md) | Weapon and armor tabs, an item list, an Equip button and a test. |

To see everything the kit offers before you start, run its catalog from a
checkout of the repository:

```text
cargo run -p kivact-ui-game-catalog
```

Start with [Set up the kit](01-set-up-the-kit.md).
