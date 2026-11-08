# Add effects and depth

[日本語](../../../jp/game/how-to-guide/effects-and-depth.md)

Use this guide to add motion, depth and light to a Game screen: a HUD that
reacts to the pointer, cards that tilt in 3D, a panel that turns, a glitch on a
title, a shader background, a boot sequence, and a header that follows the
scroll. The seven samples below are the ones on the catalog's Effects page. Each
is a plain function you can paste into an app, and each uses the kit's own colors
and corners, so it follows the theme and the density.

The rule for every effect here is the kit's rule for its panels: the effect
supplies motion and light, and the kit's tokens supply the colors and the hard
edges.

## Keep the kit's colors and corners

- Take colors from `game::ColorName` (`Cyan`, `Yellow`, `Navy`, `Canvas`, and so on)
  and never from a fixed value. A `ColorName` is read from the theme the node is
  painted in, so the effect follows a theme switch.
- Game panels have hard edges: keep `.radius(0)`. For the signature corner, clip
  the panel with `.clip_path(game::cut_corner(12.0))`, which cuts two opposite
  corners.
- To make a glow or a tint, compute one new color from a token with `with_alpha`:
  `use_theme().get().ext::<game::Tokens>().cyan.with_alpha(0.4)`. Use this only for a
  color that does not exist in the kit.

## The pointer HUD: parallax

`pointer_position()` is the pointer's position inside the window, or `None` when it
is outside. Turn it into an offset from the centre of the window, and let each
layer move by its own depth. A `spring` makes the motion settle smoothly, and the
layers that are farther back move less.

```rust,no_run
use kivact::motion::{Spring, Transition, animated};
use kivact::platform;
use kivact::prelude::*;
use kivact::view::pointer_position;
use kivact_ui_game as game;

fn parallax_hud() -> impl IntoView {
    let pointer = pointer_position();
    let window = platform::window_size();
    // The pointer as an offset from the centre, -0.5 to 0.5 on each axis.
    let offset = animated(
        move || match pointer.get() {
            Some(p) => {
                let size = window.get();
                (p.x / size.width - 0.5, p.y / size.height - 0.5)
            }
            None => (0.0, 0.0),
        },
        Transition::spring(Spring::GENTLE),
    );

    // A layer moves by its depth times the offset: the grid least, the readout most.
    let layer = |depth: f64, content: Node| {
        content.style(move || {
            let (x, y) = offset.get();
            style().translate(x * depth, y * depth)
        })
    };

    stack((
        layer(
            4.0,
            node().style(style().size(260, 160).border(1, game::ColorName::Cyan)),
        ),
        layer(
            12.0,
            text("TARGET LOCK").style(style().color(game::ColorName::Yellow).padding(12)),
        ),
    ))
}
# fn main() {}
```

With reduced motion on, a spring is not animated: the layers jump to the new
offset at once, which keeps the HUD readable.

## A tilt card in 3D

A card turns toward the pointer. Its transform carries the perspective, so the
card is seen from a fixed distance, and `backface_visibility` hides it when it
turns past a right angle. The card keeps the kit's navy panel, cyan border and cut
corner.

```rust,no_run
use kivact::draw::Transform;
use kivact::motion::{Spring, Transition, animated};
use kivact::platform;
use kivact::prelude::*;
use kivact::style::BackfaceVisibility;
use kivact::view::pointer_position;
use kivact_ui_game as game;

fn tilt_card() -> impl IntoView {
    let pointer = pointer_position();
    let window = platform::window_size();
    // Rotations about the x and y axes, in degrees, from the pointer's place.
    let tilt = animated(
        move || match pointer.get() {
            Some(p) => {
                let size = window.get();
                ((p.y / size.height - 0.5) * -20.0, (p.x / size.width - 0.5) * 20.0)
            }
            None => (0.0, 0.0),
        },
        Transition::spring(Spring::STIFF),
    );

    column((
        text("OPERATOR").style(style().color(game::ColorName::Text2)),
        text("Pilot 07").style(style().color(ColorToken::Text)),
    ))
    .style(move || {
        let (rx, ry) = tilt.get();
        style()
            .padding(20)
            .size(220, 120)
            .background(game::ColorName::Navy)
            .border(1, game::ColorName::Cyan)
            .radius(0)
            .clip_path(game::cut_corner(12.0))
            .transform(Transform::new().perspective(600.0).rotate_x(rx).rotate_y(ry))
            .backface_visibility(BackfaceVisibility::Hidden)
    })
}
# fn main() {}
```

## A drawable panel that turns

A settings panel can be drawn into a canvas as a picture and turned as a whole.
Create the panel with `drawable()`, mount its content on a node with
`.drawable(panel, content)`, and draw it inside the node's `draw` with
`draw.drawable(panel, at)`, under a layer that carries the transform. The turned
picture shows the same view that the node mounts. The
[drawables guide](../../../../for-kivact-user/en/how-to-guide/drawables.md) explains the mechanism.

```rust,no_run
use kivact::draw::{Layer, Transform};
use kivact::geometry::Point;
use kivact::motion::{Spring, Transition, animated};
use kivact::platform;
use kivact::prelude::*;
use kivact::style::BackfaceVisibility;
use kivact::view::pointer_position;
use kivact_ui_game as game;

fn turning_panel() -> impl IntoView {
    let panel = drawable();
    let pointer = pointer_position();
    let window = platform::window_size();
    let turn = animated(
        move || match pointer.get() {
            Some(p) => (p.x / window.get().width - 0.5) * 60.0,
            None => 0.0,
        },
        Transition::spring(Spring::GENTLE),
    );

    node()
        .style(style().size(260, 180))
        .drawable(panel, || {
            column((
                text("GAIN").style(style().color(game::ColorName::Yellow)),
                text("CONTRAST").style(style().color(game::ColorName::Yellow)),
            ))
            .style(style().padding(16).background(game::ColorName::Navy))
        })
        .draw(move |draw, size| {
            let angle = turn.get();
            draw.layer(
                Layer::new()
                    .transform(
                        Transform::new()
                            .translate(size.width / 2.0, size.height / 2.0)
                            .perspective(700.0)
                            .rotate_y(angle),
                    )
                    .backface_visibility(BackfaceVisibility::Hidden),
                |draw| draw.drawable(panel, Point::new(-130.0, -90.0)),
            );
        })
}
# fn main() {}
```

## A glitch: turbulence and displacement

A filter graph can distort what is under it. Here a turbulence noise moves the
pixels of the title: `displacement_map` shifts each pixel by the noise's red and
green channels, scaled by `strength`. Change the strength to make the glitch come
and go, and the kit's yellow stays the title's color.

```rust,no_run
use kivact::draw::filter::Channel;
use kivact::draw::{Filter, FilterGraph, FilterInput};
use kivact::prelude::*;
use kivact_ui_game as game;

fn glitch(strength: f64) -> Filter {
    let mut graph = FilterGraph::new();
    let noise = graph.turbulence(0.04, 2, 7.0);
    graph.displacement_map(FilterInput::SOURCE, noise, strength, Channel::R, Channel::G);
    Filter::new().graph(graph)
}

fn title(strength: Signal<f64>) -> impl IntoView {
    text("SIGNAL LOST").style(move || {
        style()
            .padding(12)
            .color(game::ColorName::Yellow)
            .filter(glitch(strength.get()))
    })
}
# fn main() {}
```

The strength is in pixels of displacement, so a small value gives a faint wobble
and a large one tears the letters apart.

## A shader background with a fallback

A paint shader draws a scanline pattern behind the content. Its colors come from
the kit's `navy` token, read in the `draw` closure from the theme, so the shader
follows the theme too. The GPU runs the shader. A window that draws on the CPU draws the `fallback`
instead, a solid navy, so the screen always shows the same background. The
shader's clock ticks at 30 Hz, and it stops under reduced motion.

```rust,no_run
use kivact::draw::PaintShader;
use kivact::geometry::Rect;
use kivact::motion::FrameRate;
use kivact::prelude::*;
use kivact_ui_game as game;

const SCANLINES: &str = r#"
struct Params { speed: f32, tint: vec4<f32> }

fn paint(position: vec2<f32>, context: Context, params: Params) -> vec4<f32> {
    let scan = 0.85 + 0.15 * step(0.5, fract(position.y * 0.25 + context.time * params.speed));
    return vec4<f32>(params.tint.rgb * scan, params.tint.a);
}
"#;

fn scanline_background(content: impl IntoView) -> impl IntoView {
    let shader = PaintShader::wgsl(SCANLINES).expect("the scanline shader is valid WGSL");
    stack((
        node()
            .style(style().size(pct(100), pct(100)))
            .draw(move |draw, size| {
                let navy = draw.theme().ext::<game::Tokens>().navy;
                let paint = shader
                    .paint()
                    .param("speed", 1.5)
                    .param("tint", navy)
                    .fallback(navy)
                    .animate(FrameRate::Hz(30.0));
                draw.fill_rect(Rect::new(0.0, 0.0, size.width, size.height), paint);
            }),
        content,
    ))
}
# fn main() {}
```

Use `.essential()` on the shader paint only when the motion carries meaning. A
scanline is decoration, so under reduced motion its clock stops and it stays on
one frame.

## A boot sequence with a timeline and a stagger

Lines of a boot log come up one after another. Each line has a track on one
timeline, and `Stagger` gives each line its start time, so the list reads in order
without a hand-written delay for every line.

```rust,no_run
use kivact::motion::{At, Easing, Stagger, Timeline, Transition, timeline};
use kivact::prelude::*;
use kivact_ui_game as game;

fn log_line(tl: Timeline, at_ms: f64, label: &'static str) -> impl IntoView {
    let shown = tl.track(0.0_f64);
    tl.to(shown, 1.0, Transition::tween(180, Easing::EaseOut))
        .at(At::Time(at_ms));
    text(label).style(move || {
        style()
            .opacity(shown.get())
            .color(game::ColorName::Cyan)
    })
}

fn boot_log() -> impl IntoView {
    let tl = timeline();
    let stagger = Stagger::new(150.0);
    on_mount(move || tl.play());
    column((
        log_line(tl, stagger.delay(0, 3), "> LINK ESTABLISHED"),
        log_line(tl, stagger.delay(1, 3), "> LOADING MODULES"),
        log_line(tl, stagger.delay(2, 3), "> READY"),
    ))
    .style(style().gap(4).padding(16).background(game::ColorName::Canvas))
}
# fn main() {}
```

## A header that follows the scroll

The header of a long list compacts as the list scrolls, so the list has more room
while the reader is in the middle of it. `scroll_progress` gives the scroll
position from 0 at the top to 1 at the end, and the header's height follows it.

```rust,no_run
use kivact::prelude::*;
use kivact::view::scroll_progress;
use kivact_ui_game as game;

fn log_list(entries: usize) -> impl IntoView {
    let scroller = node_ref();
    let progress = scroll_progress(scroller, Orientation::Vertical);

    let rows: Vec<Node> = (0..entries)
        .map(|i| text(format!("EVENT {i:03}")).style(style().padding(8)))
        .collect();

    column((
        row((game::mono("LOG"), text("ENTRIES"))).style(move || {
            // 56 px when the list is at the top, 32 px at the end.
            style()
                .height(56.0 - progress.get() * 24.0)
                .padding(12)
                .background(game::ColorName::Navy)
                .radius(0)
                .clip_path(game::cut_corner(10.0))
        }),
        scroll(column(rows))
            .node_ref(scroller)
            .style(style().height(200).background(game::ColorName::Canvas)),
    ))
}
# fn main() {}
```

## Choose the effect for the job

- **Motion that follows the user** (the HUD, the tilt): a spring on a pointer
  value. It settles on its own and needs no frame loop.
- **Motion in order** (the boot log): a timeline with staggered starts.
- **Motion that follows a scroll**: a scroll progress value, not a timer.
- **Light and distortion**: a filter or a shader, with the kit's tokens as the
  colors.

## See also

- [Show items in lists and trees](item-lists-and-selection.md)
- [Set the theme and density](theme-and-density.md): the tokens these effects read.
- [Run timelines, scroll-linked motion and shared layout](../../../../for-kivact-user/en/how-to-guide/timelines-and-scroll.md)
- [Draw with a canvas](../../../../for-kivact-user/en/how-to-guide/canvas.md)
- Every token and component: `cargo doc -p kivact-ui-game --open`
