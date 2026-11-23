# kivact-ui-creative

Dense creative-tool controls: panels, timelines, node and graph editors,
color pickers, audio controls and numeric fields.

## Use it

```rust
use kivact::prelude::*;
use kivact_ui_creative as creative;

fn main() -> kivact::Result<()> {
    App::new("App")
        .theme(creative::theme(ThemeMode::Dark))
        .run(|| kivact::ui::button("Render").variant(kivact::ui::ButtonVariant::Primary))
}
```

The standard components of `kivact::ui` draw in this look under the theme. The
kit's own components and variants are listed in the API documentation
(`cargo doc -p kivact-ui-creative --open`). Guides: `docs/for-kivact-ui-user/en/creative/`.

## License

Tabler SVG
glyphs retain their [MIT notice](LICENSE-tabler-icons.txt).
