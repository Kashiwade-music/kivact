# kivact-ui-soft

The Astryx Neutral look in light and dark appearances: the standard components of `kivact::ui` drawn in the Soft style, with an app frame and content parts.

## Use it

```rust
use kivact::prelude::*;
use kivact_ui_soft as soft;

fn main() -> kivact::Result<()> {
    App::new("App").theme(soft::theme(ThemeMode::Light)).run(|| button("Save"))
}
```

The standard components of `kivact::ui` draw in this look under the theme. The kit's own components and variants are listed in the API documentation (`cargo doc -p kivact-ui-soft --open`). Guides: `docs/for-kivact-ui-user/en/soft/`.
