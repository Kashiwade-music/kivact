# Slides and native files

[日本語](../../jp/how-to-guide/slides-and-files.md)

`ui::carousel(index, slides)` edits the caller's index with previous/next buttons and arrow keys. Horizontal arrows follow reading direction; vertical carousels use Up/Down. `.looping()` wraps at the ends and `.items_per_view(2.0)` shows several slides. `CarouselMode::Scroll` uses native scrolling; `Snap::Mandatory` settles on the nearest slide after 120 ms without scrolling. `Snap::None` preserves free scrolling while reporting the nearest index.

```rust,no_run
use kivact::prelude::*;
use kivact::ui::{carousel, CarouselMode, dropzone};
fn content() -> impl IntoView {
    let index = signal(0usize);
    column((
        carousel(index, (text("One"), text("Two")))
            .mode(CarouselMode::Scroll)
            .style(style().width(240).height(160)),
        dropzone("Choose or drop text files")
            .multiple()
            .pick_on_press()
            .validate(|file| {
                (file.path.extension().and_then(|x| x.to_str()) != Some("txt"))
                    .then(|| "Choose a text file.".into())
            })
            .on_files(|files| { println!("{} files", files.len()); }),
    ))
}
```

`FileInfo` contains a native `path`; `name()` derives its display name. Validation receives each file before delivery. To enforce a size limit, read metadata in your application's validator. The component does not access the filesystem. Single-file mode rejects a multi-file drop rather than discarding files. Picker cancellation leaves the target ready, and validation, callback and picker failures appear in the target's status.

Selection storage, clearing and selected-name display belong to the caller: store the accepted files in `.on_files`, render their names, and clear that storage with a regular button. `.hint(..)` supplies guidance and `.filters(..)` configures the native picker. Filters alone do not validate dropped files.

Carousel recipes expose root, viewport, track, slide, previous and next. Dropzone recipes expose root, label, hint and status. `UiTokens` groups optional theme overrides under `carousel` and `dropzone`; default visual values remain in the standard recipes.

The [APG carousel pattern](https://www.w3.org/WAI/ARIA/apg/patterns/carousel/) informs the native semantics. Native groups use names containing Carousel/Slide because role descriptions are unavailable. A focusable carousel group additionally supports arrow navigation. There is no automatic rotation; the regular buttons retain keyboard focus when activated. Headless custom views can use `interaction::use_carousel` and `interaction::use_dropzone`.

`.accept_drop(enabled)` gates drag/drop input independently of `.pick_on_press()`. A reactive change to false ends any current drag-over visit through the shared leave event, while native picking remains available. Use this for a compact picker-only field.
