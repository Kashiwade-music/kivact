# Animate values and elements

[日本語](../../jp/how-to-guide/animation.md)

Use this guide to move a value smoothly toward a new target, run code on every
frame, and make elements slide or fade in and out when they appear and
disappear. All animation in Kivact shares one clock that is driven by the
display, so animations stay in step, pause when the window is hidden, and are
easy to test.

Pick the tool by what you are animating:

| You want | Use |
|---|---|
| a number, point or rectangle that eases toward a target | `animated` |
| an element's appearance to change smoothly (hover, pressed, color, opacity, position) | `transition` in the element's style |
| an element to fade or slide in when it appears and out when it goes | `enter` and `exit` in the element's style |
| your own computation every frame (physics, a clock face, a simulation) | `frame_loop` |

The animation names are in `kivact::motion`. Styles take them as
`Transition` values.

## Animate a value toward a target

`animated(target, transition)` returns a value that follows `target`. When the
target changes, the value moves from where it is now to the new target. Read
it with `.get()` anywhere you would read a signal, so anything that reads it
updates on every step.

```rust,no_run
use kivact::motion::{Easing, Transition, animated};
use kivact::prelude::*;

fn progress() -> impl IntoView {
    let target = signal(0.0_f64);
    let amount = animated(target, Transition::tween(250, Easing::EaseOut));

    column((
        button("Fill").on_press(move || target.set(100.0)),
        button("Empty").on_press(move || target.set(0.0)),
        text(move || format!("{:.0}%", amount.get())),
    ))
    .style(style().gap(8).padding(12))
}

fn main() -> kivact::Result<()> {
    App::new("Progress").run(progress)
}
```

The target can be a signal, a closure (`move || if open.get() { 240.0 } else { 0.0 }`)
or a constant. Press "Empty" halfway through the fill and the value turns
around from where it is: an interrupted animation never jumps.

`Transition::tween(milliseconds, easing)` moves over a fixed time. The easing is
`Linear`, `EaseIn`, `EaseOut`, `EaseInOut`, `CubicBezier(x1, y1, x2, y2)` (the
same numbers as CSS) or `Easing::steps(n, StepPosition::JumpEnd)` / `Easing::steps(n, StepPosition::JumpStart)`.
`Easing::Ease` is the CSS `ease` curve. `BackIn`, `BackOut` and `BackInOut` go past
the target and come back, `ElasticIn`, `ElasticOut` and `ElasticInOut` wobble around
it, and `BounceIn`, `BounceOut` and `BounceInOut` bounce on the way. For a curve
drawn from points (CSS `linear()`), use `Easing::linear_points([...])` with
`(Some(input), output)` or `(None, output)` points, where a point without an input
is spread between its neighbours.
`Transition::spring(Spring::DEFAULT)` moves with a physical spring instead.
There is no fixed duration: it settles when it settles, and an interrupted spring
keeps its velocity. `Spring::GENTLE`, `WOBBLY` and `STIFF` are presets, and you
can set `stiffness`, `damping` and `mass` yourself. `Spring::with_duration(ms, bounce)`
builds a spring that comes to rest in about `ms` milliseconds, which a timeline
needs to plan its length (see [Run timelines, scroll-linked motion and shared layout](timelines-and-scroll.md)).
Every transition accepts
`.delay(ms)`, `.repeat(Repeat::Times(3))` (`Repeat::Forever` repeats until the
target changes) and `.alternate()` (every other iteration plays backwards); a
spring's iteration ends when it settles.
`Transition::keyframes([(0.0, 0.0), (0.6, 1.2), (1.0, 1.0)], 400)` passes
through values at offsets between 0 and 1 and ends at the target. Each pair is a
keyframe with a linear segment after it.

With `.retarget(kivact::motion::Retarget::Proportional)`, an interrupted timed transition scales its new duration by the remaining component distance to preserve its intended speed. `Restart` is the default. Springs keep their physical timing.

Use `style().layout_transition(Transition::tween(250, Easing::EaseOut))` to animate position and size changes caused by layout. Layout runs once; paint, pointer hit testing and accessibility share the interpolated visual transform. Decorative layout transitions also respect reduced motion.

### Use the value in a style

Moving an element with `transform` and `opacity` is cheap, because it does not
change layout. Read the animated value in a style closure:

```rust,no_run
use kivact::motion::{Spring, Transition, animated};
use kivact::prelude::*;

fn drawer() -> impl IntoView {
    let open = signal(false);
    let offset = animated(
        move || if open.get() { 0.0_f64 } else { -240.0 },
        Transition::spring(Spring::DEFAULT),
    );

    column((
        button("Toggle drawer").on_press(move || open.update(|o| *o = !*o)),
        text("Drawer content").style(move || {
            style()
                .padding(16)
                .background(ColorToken::SurfaceAlt)
                .translate_x(offset.get())
        }),
    ))
    .style(style().gap(8).padding(12))
}
# fn main() -> kivact::Result<()> { App::new("Drawer").run(drawer) }
```

Animating `width`, `height` or margins also works, but asks the layout engine
to run on every frame. Prefer a transform when you have the choice.

### More control

- `Animated::from(start, target, transition)` also animates the first
  appearance, from `start`. Plain `animated` starts at the target.
- `jump_to(value)` sets the value at once and cancels the run.
- `set_velocity(v)` gives the value a velocity in units per second, per component,
  so a value released from a drag keeps its momentum. A running or paused spring
  keeps its target and goes on with the velocity. An idle or finished value starts
  a run from where it is toward its target, once and without delay. It applies to
  springs only: on a tween it is reported and ignored.
- `pause()` and `resume()` stop and continue without counting the paused time.
- `is_running()` is a tracked read that tells whether it is moving.
- `.on_complete(|| …)`, registered in setup, runs once when a run finishes
  naturally. It does not run for a superseded target, a `jump_to` or an
  infinite repeat. A target equal to the current value starts no run, so it
  does not run then either.
- The value type can be `f32`, `f64`, a tuple of up to four of them, `Point`,
  `Size`, `Rect`, `Color` or `Transform`. A color moves through the Oklab space,
  so an in-between color stays close to what the eye expects. Implement
  `Animatable` to animate your own type (it converts to and from a few `f64`
  components).
- `animated(color, transition)` is the same as a style transition, but the
  value is available in your code: use it to draw a color that follows a target.

## Animate an element's appearance

For a change of look between states (hover, pressed, focused, or a style closure
that returns different values), give the style a transition. It interpolates
opacity, solid backgrounds, border colors, the transform, shadows and filters:

```rust,no_run
use kivact::motion::{Easing, Transition};
use kivact::prelude::*;
use kivact::style::TransitionProp;

fn hover_card() -> impl IntoView {
    column(text("Hover me"))
        .style(
            style()
                .padding(24)
                .background(ColorToken::Surface)
                .hover(style().background(ColorToken::SurfaceAlt).translate_y(-2))
                .transition(TransitionProp::All, Transition::tween(150, Easing::EaseOut)),
        )
}
# fn main() -> kivact::Result<()> { App::new("Hover").run(hover_card) }
```

The host runs these transitions itself; your code writes only the target style
and does no work per frame.

### Give each property its own transition

`TransitionProp::All` covers every property, as CSS `transition: all` does. Add a
transition for one property and it takes over from `All` for that property, so
the rest can use another timing:

```rust,no_run
use kivact::motion::{Easing, Spring, Transition};
use kivact::prelude::*;
use kivact::style::TransitionProp;

fn chip() -> impl IntoView {
    text("Pinned")
        .style(
            style()
                .padding(8)
                .background(ColorToken::Surface)
                .hover(style().background(ColorToken::SurfaceAlt).translate_y(-2).opacity(0.9))
                // Colors and opacity in 150 ms; the lift on a spring.
                .transition(TransitionProp::All, Transition::tween(150, Easing::EaseOut))
                .transition(TransitionProp::Transform, Transition::spring(Spring::STIFF)),
        )
}
# fn main() -> kivact::Result<()> { App::new("Chip").run(chip) }
```

The properties are `All`, `Opacity`, `Transform`, `Background`, `BorderColor`,
`BorderWidth`, `Radius`, `TextColor`, `Shadow`, `Filter`, `BackdropFilter` and
`Size` (the sizes of an element entering or leaving, see `enter_size` and
`exit_size`). A property with no transition changes at once.

## Animate elements entering and leaving

`enter` is the look an element starts from when it is added, and `exit` is the
look it moves to when it is removed. Both use the style's `TransitionProp::All` transition.
Pair them with `show`, `show_some`, a list or any other way elements come and
go.

```rust,no_run
use kivact::motion::{Easing, Transition};
use kivact::prelude::*;
use kivact::style::TransitionProp;

fn notice(shown: Signal<bool>) -> impl IntoView {
    show(shown, || {
        text("Saved").style(
            style()
                .padding(12)
                .background(ColorToken::SurfaceAlt)
                // Slide in from the right and fade in; leave the same way.
                .enter(style().translate_x(40).opacity(0.0))
                .exit(style().translate_x(40).opacity(0.0))
                .transition(TransitionProp::All, Transition::tween(260, Easing::EaseOut)),
        )
    })
}

fn app() -> impl IntoView {
    let shown = signal(false);
    column((
        button("Toggle").on_press(move || shown.update(|s| *s = !*s)),
        notice(shown),
    ))
    .style(style().gap(8).padding(12))
}
# fn main() -> kivact::Result<()> { App::new("Presence").run(app) }
```

What to know about a leaving element:

- It is drawn until the transition ends, then removed. From the moment of
  removal it takes no clicks, no focus, and does not appear in the accessibility
  tree, so it cannot be activated twice or announced after it is "gone".
- It keeps its place in the layout until it finishes, so what follows moves up
  at the end. To avoid that, give the element `style().position(Position::Absolute)` (`kivact::style::Position`).
- Without a `TransitionProp::All` transition, an element appears and disappears at once.

The standard toast, dialog, popup and tooltip components use this too: a
theme's recipe for them can return a style with `enter` and `exit`.

## Run code on every frame

When you need your own computation (a simulation, a clock, a game loop), use
`frame_loop`. The callback gets a `FrameInfo` with the time of the tick, the
time since the previous tick (`delta`), a frame counter and the estimated
presentation time. Start it with a rate:

```rust,no_run
use kivact::view::canvas;
use kivact::draw::{Path, Stroke};
use kivact::geometry::{Point};
use kivact::motion::{FrameInfo, FrameRate, frame_loop};
use kivact::prelude::*;

fn clock_hand() -> impl IntoView {
    let angle = signal(0.0_f64);

    let frames = frame_loop(move |frame: FrameInfo| {
        // One turn every 10 seconds, from the app clock, not by counting ticks.
        angle.set((frame.now.as_secs_f64() / 10.0 * 360.0) % 360.0);
    });
    on_mount(move || frames.start(FrameRate::Hz(30.0)));

    canvas(move |draw, size| {
        let center = Point::new(size.width / 2.0, size.height / 2.0);
        let radians = angle.get().to_radians();
        let tip = Point::new(
            center.x + radians.sin() * 40.0,
            center.y - radians.cos() * 40.0,
        );
        draw.stroke_path(&Path::new().move_to(center).line_to(tip), &Stroke::new(3), ColorToken::Accent);
    })
    .style(style().size(120, 120))
    .name("Clock hand")
}
# fn main() -> kivact::Result<()> { App::new("Clock").run(clock_hand) }
```

- **Any positive, finite rate works**: `FrameRate::Hz(24.0)`, `Hz(90.0)`, `Hz(144.0)`.
  `FrameRate::Display` ticks on every display refresh. A zero, negative, infinite
  or NaN rate is reported as an error and ignored.
- A rate above the display's refresh rate ticks at the refresh rate (a
  120 Hz loop on a 60 Hz display runs at 60), since frames are only produced
  when the display wants one.
- Scale movement by `frame.delta` instead of assuming a fixed step, so speed does
  not depend on the actual rate.
- `start(rate)` again changes the rate, `stop()` stops, and `rate()` is a reactive
  read of the current rate (`None` when stopped).
- Create the loop in setup. Start and stop it from `on_mount`, an event handler
  or an `effect`. It stops by itself when its component is disposed.
- While the window cannot be drawn (minimized, hidden, the app suspended) no
  frames are produced. When it comes back, the loop continues at its rate
  without a burst of catch-up ticks, and animations continue from where they
  stood.
- Write results into signals and let the view read them, as above. A frame
  callback that does a lot of work delays the frame, so keep it light and move
  heavy computation to `blocking` (see [Run async work from a component](async-and-tasks.md)).

## Respect reduced motion

Many people ask their system to reduce motion. Kivact reports that setting and
acts on it for you in most cases:

- Transitions are *decorative* by default. With reduced motion on, they finish
  at once: `animated` jumps to its target, and `enter`/`exit` apply immediately.
- Mark motion that carries meaning, such as a spinner that says "working", with
  `Transition::tween(…).essential()`. It keeps moving.
- A frame loop is **decorative** by default. Under reduced motion it requests no
  frame and runs no callback, and when reduced motion is switched off it counts
  again from that moment, with no burst of catch-up ticks.
- A loop that carries meaning, such as a game or a simulation, keeps running:
  mark it with `.essential()` in setup. Its callback runs at its rate as before,
  so keep the work light and follow the reduced-motion setting in what it draws
  if that matters to the user:

```rust,no_run
use kivact::motion::{FrameInfo, FrameRate, frame_loop};
use kivact::prelude::*;

fn starfield() -> impl IntoView {
    let shift = signal(0.0_f64);
    // A simulation keeps moving under reduced motion.
    let frames = frame_loop(move |frame: FrameInfo| shift.set(frame.now.as_secs_f64() * 12.0))
        .essential();
    on_mount(move || frames.start(FrameRate::Hz(30.0)));
    text(move || format!("{:.0}", shift.get()))
}
# fn main() -> kivact::Result<()> { App::new("Starfield").run(starfield) }
```

A decorative loop that stops leaves the view showing the last frame it drew. If
you want a fixed picture instead, read `kivact::platform::is_motion_reduced().get()`
(a tracked read) in the view and draw that picture.

## Test animations

The test clock is virtual and starts and stops with the frames, so tests do not
wait. `screen.advance(Duration)` moves time forward through every frame. You can
start a test with reduced motion on or switch it while the screen runs:

```rust,test_harness
use kivact::motion::{Easing, Transition, animated};
use kivact::prelude::*;

fn progress() -> impl IntoView {
    let target = signal(0.0_f64);
    let amount = animated(target, Transition::tween(250, Easing::Linear));
    column((
        button("Fill").on_press(move || target.set(100.0)),
        text(move || format!("{:.0}%", amount.get())),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Progress").run(progress)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use kivact::test::{RenderOptions, render, render_with};

    #[test]
    fn the_value_arrives_after_the_transition() {
        let screen = render(progress);
        screen.get_by_role(Role::Button, "Fill").click();
        assert!(screen.query_by_text("100%").is_none());
        screen.advance(Duration::from_millis(300));
        screen.get_by_text("100%");
    }

    #[test]
    fn reduced_motion_arrives_at_once() {
        let options = RenderOptions::default().reduced_motion();
        let screen = render_with(options, progress);
        screen.get_by_role(Role::Button, "Fill").click();
        screen.get_by_text("100%");
    }
}
```

## See also

- [Draw with a canvas](canvas.md): read animated values and frame-loop state in a drawing.
- [Run timelines, scroll-linked motion and shared layout](timelines-and-scroll.md): steps in order, stagger, scroll progress and `layout_id`.
- [Run async work from a component](async-and-tasks.md): timers and background work.
- [Test your app](testing.md): the virtual clock and `RenderOptions`.
- API reference: `cargo doc -p kivact --open`, then `kivact::motion` and `kivact::style::Style`.

Use `frame_loop(callback).while_present(node_ref)` to pause frame scheduling while the referenced node is entering, leaving or unbound. When it becomes present, the loop resumes at its requested rate. `start`, `stop` and `rate` retain their meaning while paused; a stopped loop stays stopped after remounting.
