# Use 3D transforms and depth

[日本語](../../jp/how-to-guide/3d-and-depth.md)

Use this guide to tilt, turn and move elements in three dimensions: cards that
turn over, panels that lean toward the pointer, and layers that sit at
different depths. Kivact follows CSS Transforms 2 for these: the same
functions, `perspective`, `transform-style: preserve-3d` and
`backface-visibility`. The CPU and GPU renderers draw the same picture, and
pointer hits land on the element where it appears after the transform.

## Rotate an element in space

The transform properties on a style are `translate_x`, `translate_y`,
`translate_z`, `rotate` (flat, about the z axis), `rotate_x`, `rotate_y` and
`scale`. Angles are in degrees. Without a perspective they are flat, so a
rotation about x or y only squashes the element. Give the parent a
`perspective` (the distance of the viewer, in logical pixels) to see depth.

```rust,no_run
use kivact::prelude::*;

fn tilted_card() -> impl IntoView {
    let angle = signal(0.0_f64);

    column((
        button("Turn 15 degrees").on_press(move || angle.update(|a| *a += 15.0)),
        column(text("Card"))
            .style(move || {
                style()
                    .size(200, 120)
                    .padding(16)
                    .radius(12)
                    .background(ColorToken::Surface)
                    .rotate_x(20.0)
                    .rotate_y(angle.get())
            }),
    ))
    // The perspective belongs to the parent: its children are seen from 800 px away.
    .style(style().gap(12).padding(24).perspective(800.0))
}
# fn main() -> kivact::Result<()> { App::new("Tilted card").run(tilted_card) }
```

- `perspective` is set on the element whose children should look three
  dimensional. A distance of 800 looks like a camera a little way back;
  smaller numbers exaggerate the tilt.
- `perspective_origin(x, y)` moves the point the viewer looks at, against the
  box. The default is its centre.
- `transform_origin(x, y)` moves the point the element turns about. The
  default is its centre. `transform_origin_z` sets its depth.
- To write a transform as one value, use `kivact::draw::Transform` with
  `style().transform(..)`, for example
  `Transform::new().perspective(800.0).rotate_y(angle)`. The functions apply in
  the order you write them, as in CSS. `matrix_3d` accepts a 4×4 matrix whose
  last row is `(0, 0, 0, 1)`; put the perspective in `perspective` or
  `perspective(..)` instead, since a general perspective row is not supported.

## Keep children in one 3D space

By default each element is drawn flat in its parent's plane. A card with a
front and a back needs its faces to share one 3D space, so the container
uses `transform_style(TransformStyle::Preserve3d)`. Its children then keep
their own depth and rotation relative to the container.

```rust,no_run
use kivact::prelude::*;
use kivact::style::{BackfaceVisibility, TransformStyle};

fn flip_card() -> impl IntoView {
    let flipped = signal(false);
    let turn = animated(
        move || if flipped.get() { 180.0_f64 } else { 0.0 },
        Transition::spring(Spring::GENTLE),
    );

    let front = column(text("Front")).style(
        style()
            .size(200, 120)
            .padding(16)
            .background(ColorToken::Accent)
            .backface_visibility(BackfaceVisibility::Hidden),
    );
    let back = column(text("Back")).style(
        style()
            .size(200, 120)
            .padding(16)
            .background(ColorToken::Surface)
            .backface_visibility(BackfaceVisibility::Hidden)
            .rotate_y(180.0),
    );

    column((
        button("Flip").on_press(move || flipped.update(|f| *f = !*f)),
        stack((front, back)).style(move || {
            style()
                .transform_style(TransformStyle::Preserve3d)
                .rotate_y(turn.get())
        }),
    ))
    .style(style().gap(12).padding(24).perspective(800.0))
}
# fn main() -> kivact::Result<()> { App::new("Flip card").run(flip_card) }
```

- `Preserve3d` applies to the element that holds the faces. Its children are
  placed in its 3D space, so `rotate_y(180)` on the back turns it to face the
  other way.
- Put `perspective` on the parent of that container, as the usual CSS pattern
  does. The container's own rotation is then seen in perspective, and the
  faces inside share its 3D space.
- A group breaks a preserve-3d space and flattens its children into one plane.
  The following make an element a group: an `opacity` below 1, a `filter`, a
  `mask`, a `clip_path`, a `backdrop_filter`, a `mix_blend_mode` other than
  `Normal`, and `isolation(Isolation::Isolate)`.
  Keep the faces free of these when they must share the space.

## Hide the back of a face

`backface_visibility(BackfaceVisibility::Hidden)` hides an element while its
back faces the viewer. In the flip card, each face hides its back, so only the
side facing the viewer is drawn. With the default `Visible`, a card turned
past 90 degrees shows its back as a mirror image.

## Follow the pointer

`pointer_position()` gives the last pointer position in window coordinates,
independently of what is under the pointer. It is absent (`None`) when the
pointer is outside the window. Combine it with `window_size()` to turn it into
a value from -1 to 1, and move the value through `animated` with a spring so
the tilt follows the pointer smoothly and keeps its speed when the pointer
changes direction.

```rust,no_run
use kivact::platform::window_size;
use kivact::prelude::*;
use kivact::view::pointer_position;

fn tilt_card() -> impl IntoView {
    let pointer = pointer_position();
    let window = window_size();

    // Pointer position as -1..1 across the window; centred when it is outside.
    let aim = animated(
        move || {
            let size = window.get();
            let point = pointer.get().map_or(Point::new(size.width / 2.0, size.height / 2.0), |p| p);
            Point::new(
                (point.x / size.width.max(1.0)) * 2.0 - 1.0,
                (point.y / size.height.max(1.0)) * 2.0 - 1.0,
            )
        },
        Transition::spring(Spring::GENTLE),
    );

    column(
        column((text("Move the pointer"), text("This card leans toward it.")))
            .style(move || {
                let a = aim.get();
                style()
                    .padding(24)
                    .radius(16)
                    .background(ColorToken::Surface)
                    .rotate_x(-a.y * 12.0)
                    .rotate_y(a.x * 12.0)
                    .translate_z(20.0)
            }),
    )
    .style(style().padding(40).perspective(600.0))
}
# fn main() -> kivact::Result<()> { App::new("Tilt card").run(tilt_card) }
```

- The pointer position is in window coordinates, so the example compares it
  with the window size, not with the card. Pick the limit of the tilt
  (here 12 degrees) to suit your layout.
- `translate_z` lifts an element toward the viewer; a value of 20 px lifts it a
  little above the plane of its parent. Give background layers a negative
  `translate_z` and a larger move for a parallax effect.
- The spring keeps its velocity when the pointer leaves the window and the
  tilt returns to the centre.

## Things to know

- **Hit tests follow the transform.** A rotated or lifted element is hit where
  it is drawn.
- **Crowded 3D spaces are drawn flat.** When a preserve-3d space holds more
  than 256 participating elements, the whole space is drawn and hit-tested as a
  flat stack in document order. Use fewer, larger 3D surfaces for large
  scenes, or split them into several spaces.
- **Only affine and perspective matrices.** `matrix_3d` takes a matrix with a
  bottom row of `(0, 0, 0, 1)`. Other bottom rows are reported and replaced.
- **Transforms do not run the layout engine.** Turning and lifting an element
  costs about the same as moving it in 2D.
- **Moving layers can look soft.** While a 3D element moves, its edges may look
  slightly blurred by sub-pixel placement, and they are sharp again when it
  stops.

## See also

- [Animate values and elements](animation.md): springs, transitions and `animated`.
- [Draw with a canvas](canvas.md): 3D layers inside a canvas.
- [Use effects](effects.md): filters, masks and blending on the same elements.
- API reference: `cargo doc -p kivact --open`, then `kivact::style` and `kivact::draw::Transform`.
