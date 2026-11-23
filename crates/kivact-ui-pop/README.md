# kivact-ui-pop

A Kivact UI kit for compact desktop business applications: calm slate
surfaces, narrow blue gradients and yellow accents.

## Use it

```rust
use kivact::prelude::*;
use kivact_ui_pop as pop;

fn main() -> kivact::Result<()> {
    App::new("App").theme(pop::theme(ThemeMode::Light)).run(|| button("Save"))
}
```

The standard components of `kivact::ui` draw in this look under the theme. The kit's own components and variants are listed in the API documentation (`cargo doc -p kivact-ui-pop --open`). Guides: `docs/for-kivact-ui-user/en/pop/`.

## License

MIT OR Apache-2.0. The bundled Tabler icons are MIT
([LICENSE-tabler-icons.txt](assets/LICENSE-tabler-icons.txt)).
