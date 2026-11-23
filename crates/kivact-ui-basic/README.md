# kivact-ui-basic

The look of the shadcn/ui "Lyra" preset (neutral colors, square corners,
12 px controls, Lucide icons) for the standard Kivact components.

## Use it

```rust
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn main() -> kivact::Result<()> {
    App::new("App").theme(basic::theme(ThemeMode::Light)).run(|| button("Save"))
}
```

The standard components of `kivact::ui` draw in this look under the theme. The kit's own components and variants are listed in the API documentation (`cargo doc -p kivact-ui-basic --open`). Guides: `docs/for-kivact-ui-user/en/basic/`.
