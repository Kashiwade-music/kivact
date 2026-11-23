# kivact-ui-game

A Kivact UI kit for dense, hard-edged operation consoles, game control panels
and HUDs.

## Use it

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_game as game;

fn main() -> kivact::Result<()> {
    App::new("App").theme(game::theme(ThemeMode::Light)).run(|| button("Save"))
}
```

The standard components of `kivact::ui` draw in this look under the theme. The kit's own components and variants are listed in the API documentation (`cargo doc -p kivact-ui-game --open`). Guides: `docs/for-kivact-ui-user/en/game/`.

## License

MIT OR Apache-2.0. The bundled Tabler icons are MIT
