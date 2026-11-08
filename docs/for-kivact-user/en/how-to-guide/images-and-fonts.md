# Show images and use custom fonts

[日本語](../../jp/how-to-guide/images-and-fonts.md)

Use `image(source)` to show a PNG, JPEG or SVG, name it so assistive technology
can describe it, size it with a style, and register your own fonts with
`App::font`. This guide also covers images loaded at run time, animated images,
SVG files used as icons or as pictures with their own text, and the axes of
variable fonts.

## The shortest working image

```rust,test_harness
use kivact::prelude::*;

// Inline SVG bytes keep the example self-contained. In an app, embed a file
// instead: `static LOGO: &[u8] = include_bytes!("../assets/logo.svg");`
static LOGO: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="48" height="24" viewBox="0 0 48 24"><rect width="48" height="24" rx="6" fill="#2563eb"/><circle cx="12" cy="12" r="6" fill="white"/></svg>"##;

fn app() -> impl IntoView {
    column((
        image(LOGO).name("Kivact logo").test_id("logo"),
        image(LOGO).style(style().width(96)).test_id("wide"),
    ))
    .style(style().gap(12).padding(16).align_items(Align::Start))
}

fn main() -> kivact::Result<()> {
    App::new("Images").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_image_takes_its_natural_size_and_keeps_its_aspect_ratio() {
        let screen = kivact::test::render(app);
        let size = |id: &str| {
            let r = screen.get_by_test_id(id).rect().expect("laid out");
            (r.width, r.height)
        };
        assert_eq!(size("logo"), (48.0, 24.0));
        assert_eq!(size("wide"), (96.0, 48.0));
        screen.get_by_role(Role::Image, "Kivact logo");
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}
```

`image(source)` decides the format from the content: PNG (animated PNG too), JPEG,
GIF, WebP, SVG and SVGZ. Animated GIF, WebP and PNG play; see
[Play an animated image](#play-an-animated-image). Without a size in its style it has its natural
size; with one side set, the other follows the aspect ratio. The column here
uses `align_items(Align::Start)` because the default `Stretch` would make every
image as wide as the column.

## Where an image comes from

`image` accepts everything that converts to an `Asset`:

| Source | Use it for |
| --- | --- |
| `include_bytes!("logo.png")` or a `static` byte slice | files shipped inside the executable (never read from disk) |
| `Vec<u8>`, `AssetBytes` | bytes you got at run time, such as a download |
| `"photos/cover.jpg"`, `String`, `PathBuf`, `&Path` | a file read when it is first shown, relative to the current directory |
| `Asset::from(..)` | any of the above, as a value you keep |
| a closure, `Signal<Asset>` or `Value<Asset>` | an image that changes |

```rust,no_run
use kivact::view::Asset;
use kivact::prelude::*;

fn gallery(paths: Vec<&'static str>) -> impl IntoView {
    let current = signal(0_usize);
    column((
        // The closure re-runs when `current` changes and returns another Asset.
        image(move || Asset::from(paths[current.get()])).style(style().height(160)),
        button("Next").on_press(move || current.update(|i| *i = (*i + 1) % 3)),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Gallery").run(|| gallery(vec!["a.jpg", "b.jpg", "c.jpg"]))
}
```

### Bytes obtained at run time

Pass owned bytes directly: `image(bytes)` for a `Vec<u8>`, or
`Pixels::new(width, height, rgba)` for your renderer's RGBA8 buffer. When several
values need to retain the same bytes, construct `AssetBytes::new(bytes)` once
and clone that value. `AssetBytes::copy_from_slice(slice)` copies borrowed bytes
once. Both `Asset` and `AssetBytes` are available from `kivact::view`.


An `Asset` is cached by **identity**, not by content: the same file path, the
same embedded slice, or the same retained byte allocation. `Asset::from(bytes.clone())`
creates a new allocation, hence a new asset that is decoded again, every time
the expression runs. So for bytes you produced yourself, create the `Asset` once
in setup and clone the `Asset`:

```rust,no_run
use kivact::view::Asset;
use kivact::prelude::*;

fn avatar(bytes: Vec<u8>) -> impl IntoView {
    let asset = Asset::from(bytes); // once
    column((
        image(asset.clone()).style(style().size(64, 64)).name("Profile picture"),
        image(asset).style(style().size(24, 24)), // the same decoded image
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Avatar").run(|| avatar(Vec::new()))
}
```

For embedded bytes write a `static`, not a `const`: a `const` is copied into
every use, so the same file may be decoded more than once.

To load an image from the network or disk without blocking the window, fetch the
bytes with a `resource` and show the image when they arrive (see
[Run async work from a component](async-and-tasks.md)).

## Size and fit

```rust,no_run
use kivact::view::ImageFit;
use kivact::prelude::*;

static PHOTO: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="20"><rect width="40" height="20" fill="#888"/></svg>"##;

fn thumbnails() -> impl IntoView {
    row((
        // Fits inside the box, keeping the proportions (the default).
        image(PHOTO).style(style().size(80, 80)),
        // Fills the box and crops what overflows.
        image(PHOTO).fit(ImageFit::Cover).style(style().size(80, 80)),
        // Stretches to the box.
        image(PHOTO).fit(ImageFit::Fill).style(style().size(80, 80)),
    ))
    .style(style().gap(12).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Fit").run(thumbnails)
}
```

- With both width and height set, `fit` decides how the picture fills the box:
  `Contain` (default), `Cover`, `Fill`, `None` (natural size, clipped by the box)
  and `ScaleDown` (natural size, unless the box is smaller).
- Set one side only to keep the aspect ratio: `style().height(160)`.
- `style().radius(8)` rounds the corners, `style().clip(true)` clips children,
  and `style().opacity(..)` fades the image.

## Play an animated image

An animated GIF, WebP or PNG plays by itself. `.playing(..)` controls it: give it
a `bool` or a signal, and set it to `false` to stop the animation on its current
frame. With reduced motion on, an animated picture shows its first frame and does
not play, whatever `playing` says.

```rust,no_run
use kivact::prelude::*;

fn loading(busy: Signal<bool>) -> impl IntoView {
    column((
        image("assets/spinner.gif")
            .name("Loading")
            .playing(busy)
            .style(style().size(32, 32)),
        text(move || String::from(if busy.get() { "Loading" } else { "Done" })),
    ))
    .style(style().gap(8).padding(16).align_items(Align::Start))
}

fn main() -> kivact::Result<()> {
    App::new("Spinner").run(|| loading(signal(true)))
}
```

A picture that moves on its own is a decoration unless it carries meaning: name
it only when the animation tells the user something (a progress spinner that
says "loading" is named "Loading" above), otherwise leave it unnamed.

## Describe an image

An image is exposed to assistive technology only when you name it:

- **Informative** (a logo, a chart, a photo that carries meaning):
  `.name("Kivact logo")`. Write what a person who cannot see it needs to know,
  as `alt` text.
- **Decorative** (a flourish, a background, an icon beside a text label): leave
  out `.name(..)`. The image is skipped by screen readers, like `alt=""`.
- A test id alone does not expose an image.

## Use an SVG as an icon

An image can be painted with the text color only, so one SVG works in light and
dark themes and changes with hover, press and disabled states:

```rust,no_run
use kivact::prelude::*;

static STAR: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M12 2l3 7h7l-6 4 2 7-6-4-6 4 2-7-6-4h7z"/></svg>"#;

fn favorite() -> impl IntoView {
    row((
        image(STAR)
            .monochrome(true)
            .style(style().size(20, 20).color(ColorToken::Warning)),
        text("Favorite"),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Icon").run(favorite)
}
```

`.monochrome(true)` keeps the image's alpha and fills it with the node's text
color. To use the same icon everywhere by name (and let a theme replace it),
register it in the theme and draw it with `icon("star")`; see
[Theming](theming.md#icons).

## Put text and pictures inside an SVG

An SVG keeps its own `<text>` and embedded pictures, and both are drawn:

- A `<text>` element is drawn with the fonts of the app. Name the family the
  way you registered it (see [Register your own fonts](#register-your-own-fonts)),
  and the text takes that font's shapes and metrics. Register the font before the
  first screen, as with any other font.
- An `<image>` whose `href` is a `data:` URL of a PNG, JPEG, GIF or WebP is drawn.
  A link to a file (`href="logo.png"`) is not read: the SVG is drawn without that
  picture, so embed the bytes or show the picture with `image(..)` beside it.
- An SVG that would need too much memory for its text and its pictures is refused
  and reported as an asset error, like any other source that cannot be read.

```rust,no_run
use kivact::prelude::*;

static BADGE: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="120" height="32" viewBox="0 0 120 32"><rect width="120" height="32" rx="6" fill="#1e3a8a"/><text x="12" y="21" font-family="Inter" font-size="14" fill="#ffffff">Online</text></svg>"##;

fn status() -> impl IntoView {
    image(BADGE).name("Status: online")
}

fn main() -> kivact::Result<()> {
    let regular = std::fs::read("assets/Inter-Regular.ttf").expect("font file");
    App::new("Status").font(regular).run(status)
}
```

## Adjust a variable font

A variable font has axes, such as weight and width, that you set to any value
within the font's range. Write them with `font_variation_settings`, a list of
four-letter axis tags and values. Other settings adjust the design along the
axes the font provides:

- `font_variation_settings([("wght", 650.0)])` sets an axis by its tag: weight
  `wght`, width `wdth`, slant `slnt`, optical size `opsz`, and so on.
- `font_width(75.0)` (CSS `font-stretch`) sets the width as a percentage of the
  normal width, from 50 to 200.
- `font_optical_sizing(false)` turns off the optical size axis, which follows the
  font size by default, so a headline and a caption keep one design.

```rust,no_run
use kivact::prelude::*;

fn headlines() -> impl IntoView {
    column((
        text("Variable weight").style(
            style()
                .font_family("Inter")
                .font_variation_settings([("wght", 650.0)]),
        ),
        text("Condensed width").style(style().font_family("Inter").font_width(75.0)),
        text("One design at every size").style(
            style()
                .font_family("Inter")
                .font_size(48)
                .font_optical_sizing(false),
        ),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    let inter = std::fs::read("assets/Inter-Variable.ttf").expect("font file");
    App::new("Headlines").font(inter).run(headlines)
}
```

A setting for an axis that the font does not have does nothing. `font_feature_settings([("tnum", 1)])`
switches OpenType features on or off in the same way, and `tabular_nums()` is a
shorthand for the `tnum` feature.

## Register your own fonts

`App::font` loads a TTF, OTF or TTC file before the first screen. After that you
can use the font's **family name** (the name written in the font, which is often
not the file name) in `style().font_family(..)`:

```rust,no_run
use kivact::prelude::*;

fn app() -> impl IntoView {
    column((
        text("The quick brown fox").style(style().font_family("Inter")),
        text("Falls back per glyph")
            .style(style().font_family("Inter, 'Noto Sans JP', sans-serif")),
        text("let x = 1;").style(style().font_family("monospace")),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    // In an app, embed the files: `.font(include_bytes!("../assets/Inter-Regular.ttf"))`.
    let regular = std::fs::read("assets/Inter-Regular.ttf").expect("font file");
    let bold = std::fs::read("assets/Inter-Bold.ttf").expect("font file");

    App::new("Fonts").font(regular).font(bold).run(app)
}
```

- Call `.font(..)` once per file. Regular and bold are usually separate files;
  register both and pick the face with `style().weight(700)`. `italic(true)`
  picks an italic face when the family has one.
- `font_family` is inherited by the node's descendants, so set it on a container
  to change a whole panel. The generic names `system-ui`, `sans-serif`, `serif`
  and `monospace` always work.
- A comma-separated list picks a font **per glyph**: the first family that has
  the character wins. Use it to put a Latin font before a CJK font.
- A family that is not installed falls back to the system UI font and is
  reported once as an `Asset` error. A file that is not a usable font (WOFF, a
  truncated file) is reported too, and the app still starts.
- For one default font in the whole app, set the theme's text family:
  `theme.with_text(Typography { family: Some("Inter".into()), ..theme.text().clone() })`
  with `Typography` from `kivact::style::theme`.
- `kivact::platform::font_families()` lists the families available, including
  the ones you registered. Call it from setup or a handler.

## When an image does not appear

A source that cannot be read or decoded draws nothing and is reported once as an
`Asset` error, with the path of the call and, for a file, the absolute path that
was tried. The rest of the app keeps running. In a test, `screen.errors()` lists
such problems (see [Test your app](testing.md)), and
[Find out why something failed or updated](diagnostics.md) explains how to read them at run time.

## See also

- [Theming](theming.md): theme icons, colors and the default font.
- [Layout](layout.md): size and align images.
- [Run async work from a component](async-and-tasks.md): load pictures without freezing
  the window.
- [Internationalization](i18n.md): fonts for other scripts and text direction.
- Reference: `cargo doc -p kivact --open`, search for `Asset` and `ImageFit`.

Node references expose reactive `scroll_state()` (`NotScrollable`, `AtStart`, `Middle`, `AtEnd`) and `image_status()` (`Loading`, `Ready`, `Failed`) values. Create these readers during setup. Image status follows the current source, including retries.
