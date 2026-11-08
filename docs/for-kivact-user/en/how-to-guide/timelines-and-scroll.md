# Run timelines, scroll-linked motion and shared layout

[日本語](../../jp/how-to-guide/timelines-and-scroll.md)

Use this guide when one animation has several steps that run in order or
together, when a list should start its items one after another, when motion
should follow how far the user has scrolled, when an element should glide from
its old place to its new one, or when the old content must finish leaving before
the new content enters. The tools are `timeline`, `Stagger`, the scroll progress
values, `PresenceMode` and `layout_id`.

| You want | Use |
|---|---|
| several values that move in order or at fixed times, under one clock | `timeline()` with tracks and segments |
| items of a list that start a step apart | `Stagger` |
| a value from 0 to 1 that follows how far a scroller has moved | `scroll_progress` |
| a value from 0 to 1 that follows an element's travel through the scroller | `view_progress` |
| a timeline that moves with a scroll position or any other progress value | `Timeline::drive` |
| the old content to finish leaving before the new content enters | `PresenceMode::Wait` |
| an element to glide from its last box to a new box | `layout_id` with a layout transition |

## Run values in order on a timeline

A timeline has its own clock. Create it in setup, add a **track** for each value
it drives, add **segments** that move a track to a target, and start it from
`on_mount` or an event handler. A track is read like a signal, so the view
updates while the timeline plays.

```rust,no_run
use kivact::motion::{At, Easing, Stagger, Timeline, Transition, timeline};
use kivact::prelude::*;

fn boot_line(tl: Timeline, delay_ms: f64, label: &'static str) -> impl IntoView {
    let shown = tl.track(0.0_f64);
    tl.to(shown, 1.0, Transition::tween(200, Easing::EaseOut))
        .at(At::Time(delay_ms));
    text(label).style(move || style().opacity(shown.get()))
}

fn boot_sequence() -> impl IntoView {
    let tl = timeline();
    let stagger = Stagger::new(180.0);
    on_mount(move || tl.play());

    column((
        boot_line(tl, stagger.delay(0, 3), "Booting kernel"),
        boot_line(tl, stagger.delay(1, 3), "Mounting drives"),
        boot_line(tl, stagger.delay(2, 3), "Ready"),
    ))
    .style(style().gap(6).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Boot").run(boot_sequence)
}
```

The rules that decide when things happen:

- A segment starts **after the previous segment** unless you move it. `.at(..)`
  takes an `At`: `At::AfterPrevious(ms)` (the default), `At::WithPrevious(ms)`
  (the same start as the previous segment), `At::Time(ms)` (from the start of
  the timeline) or `At::Label(name, ms)`. Call `.at(..)` right after `to`.
- `tl.label("name", At::Time(ms))` names a time. `tl.seek_label("name")` moves
  there. A name that was never defined is reported and ignored.
- A track's value at a moment is the value of the last segment that has started,
  so two segments on one track never fight.
- `tl.call(At::Time(ms), || …)` runs a closure once when playback crosses that
  time. A `seek` across the time does not run it.
- `tl.play()`, `pause()`, `reverse()`, `restart()`, `finish()`, `seek(ms)` and
  `set_rate(r)` control playback from a handler or an effect. `time()` and
  `progress()` (0 to 1) are tracked reads, `is_playing()` tells whether it runs
  and `duration()` gives one repetition in milliseconds.
- `tl.repeat(Repeat::Times(3))` or `Repeat::Forever` repeats the whole timeline,
  and `.alternate()` plays every other repetition backwards. A single segment
  cannot repeat forever: that is reported, and it plays once.
- `tl.on_complete(|| …)`, registered in setup, runs once when playback ends
  naturally. A timeline driven by a progress value never completes.

With reduced motion on, a timeline that is not marked essential ends at once
when it is played: its tracks jump to the end and `on_complete` runs. Mark it
with `tl.essential()` when the motion carries meaning, such as a progress
indicator that says "working".

### Shape the timing with a spring

A `Transition::spring(..)` segment takes as long as it needs to come to rest,
and the segments after it that use `At::AfterPrevious` wait for that. To plan a
timeline by time, use `Spring::with_duration(ms, bounce)`: a spring that comes to
rest in about `ms` milliseconds. `bounce` is from -1 to 1: 0 settles without
overshoot, and a positive value overshoots.

## Stagger a list

`Stagger::new(each_ms)` gives every item of a list a delay, one step after the
item before it. `stagger.delay(index, count)` returns the delay in milliseconds
for one item. Choose where the count starts with `.from(StaggerFrom::…)` and
shape the steps with `.easing(..)`:

- `StaggerFrom::Start` (the default) counts from the first item,
- `StaggerFrom::End` counts from the last,
- `StaggerFrom::Center` starts in the middle and moves outwards,
- `StaggerFrom::Edges` starts at both ends and moves inwards,
- `StaggerFrom::Index(i)` counts from item `i`.

```rust,no_run
use kivact::motion::{Easing, Stagger, StaggerFrom, Transition};
use kivact::prelude::*;
use kivact::style::TransitionProp;

fn chips(labels: &'static [&'static str]) -> impl IntoView {
    let stagger = Stagger::new(60.0)
        .from(StaggerFrom::Center)
        .easing(Easing::EaseOut);
    let count = labels.len();

    let items: Vec<Node> = labels
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let delay = stagger.delay(i, count);
            text(*label).style(
                style()
                    .padding(8)
                    .background(ColorToken::SurfaceAlt)
                    .enter(style().opacity(0.0).translate_y(8))
                    .transition(
                        TransitionProp::All,
                        Transition::tween(240, Easing::EaseOut).delay(delay),
                    ),
            )
        })
        .collect();

    row(items).style(style().gap(8).padding(12))
}
# fn main() -> kivact::Result<()> { App::new("Chips").run(|| chips(&["Ore", "Fuel", "Food"])) }
```

A stagger only supplies delays. The transition of each item decides how it
moves, so the same delays work with `enter`, a style transition or a timeline
track.

## Follow the scroll position

`scroll_progress(scroller, axis)` returns a value from 0 at the start of the
scroll range to 1 at the end. Give the scroll container a `NodeRef` with
`.node_ref(..)`, and read the value in a style closure. The value is 0 when the
container cannot scroll along that axis.

`view_progress(target, scroller, axis, range)` measures how far an element
has travelled through the visible part of the scroller, as a value from 0 to 1.
The `ViewRange` says where the measurement starts and ends:

| `ViewRange` | 0 at | 1 at |
|---|---|---|
| `Entry` | the element starts to enter from the end | the element is fully inside |
| `Contain` | the element is fully inside | the element is fully outside |
| `Cover` | the element starts to enter from the end | the element has left at the start |
| `Exit` | the element starts to leave | the element has left at the start |

```rust,no_run
use kivact::prelude::*;
use kivact::view::{ViewRange, scroll_progress, view_progress};

fn release_notes(lines: usize) -> impl IntoView {
    let scroller = node_ref();
    let card = node_ref();
    let header = scroll_progress(scroller, Orientation::Vertical);
    let reveal = view_progress(card, scroller, Orientation::Vertical, ViewRange::Entry);

    let entries: Vec<Node> = (0..lines)
        .map(|i| {
            let entry = text(format!("Entry {i}")).style(style().padding(12));
            if i == 4 {
                // This entry fades and rises while it enters the viewport.
                entry.node_ref(card).style(move || {
                    let p = reveal.get();
                    style().padding(12).opacity(p).translate_y((1.0 - p) * 24.0)
                })
            } else {
                entry
            }
        })
        .collect();

    column((
        // The header fades as the list scrolls past the top.
        text("Release notes").style(move || {
            let p = header.get();
            style().opacity(1.0 - p * 0.6).translate_y(-p * 8.0)
        }),
        scroll(column(entries)).node_ref(scroller).style(style().height(240)),
    ))
    .style(style().gap(8).padding(16))
}
# fn main() -> kivact::Result<()> { App::new("Notes").run(|| release_notes(40)) }
```

The scroll values update when the host reports a new scroll position or layout.
A node that is not mounted, or a scroller that cannot scroll, reads as 0.

## Drive a timeline from scroll

A timeline can take its time from a progress value instead of its clock:
`tl.drive(progress)` maps `0..=1` onto the whole timeline. Call it after the
segments are added. A driven timeline cannot be played, paused or restarted,
and it never completes, so scrolling is the only thing that moves it. This is
how a scroll-linked animation is written without a frame loop:

```rust,no_run
use kivact::motion::{Easing, Transition, timeline};
use kivact::prelude::*;
use kivact::view::scroll_progress;

fn reading_progress(scroller: NodeRef) -> impl IntoView {
    let tl = timeline();
    let width = tl.track(0.0_f64);
    tl.to(width, 100.0, Transition::tween(1000, Easing::Linear));
    tl.drive(scroll_progress(scroller, Orientation::Vertical));

    column((
        node().style(move || style().height(4).width(pct(width.get())).background(ColorToken::Accent)),
        scroll(text("Long article")).node_ref(scroller).style(style().height(240)),
    ))
}
# fn main() -> kivact::Result<()> { App::new("Reading").run(|| reading_progress(node_ref())) }
```

## Let the old content leave before the new one enters

`show`, `show_some` and `dynamic` replace one branch with another when their
condition changes. By default the old content leaves while the new content
enters (`PresenceMode::Sync`). Choose the other two modes with `.presence(..)`:

- `PresenceMode::Sync`: the old content leaves while the new one enters.
- `PresenceMode::Wait`: the new content mounts only when the old content's exit
  has finished, so the two never overlap. Use it for a dialog body, a page or a
  panel whose two states would collide if drawn together.
- `PresenceMode::PopLayout`: the old content leaves out of the layout at once,
  while the new one enters, so the things around it do not wait for the exit.

Give the content a `style().exit(..)` and a `TransitionProp::All` transition so
there is an exit to wait for. Without one, it is removed at once in every mode.

```rust,no_run
use kivact::motion::{Easing, Transition};
use kivact::prelude::*;
use kivact::style::TransitionProp;
use kivact::view::PresenceMode;

fn fading(label: &'static str) -> Node {
    text(label).style(
        style()
            .padding(12)
            .background(ColorToken::SurfaceAlt)
            .exit(style().opacity(0.0))
            .transition(TransitionProp::All, Transition::tween(200, Easing::EaseOut)),
    )
}

fn play_pause(playing: Signal<bool>) -> impl IntoView {
    column((
        button("Toggle").on_press(move || playing.update(|p| *p = !*p)),
        show(playing, || fading("Playing"))
            .fallback(|| fading("Paused"))
            .presence(PresenceMode::Wait),
    ))
    .style(style().gap(8).padding(12))
}
# fn main() -> kivact::Result<()> { App::new("Player").run(|| play_pause(signal(true))) }
```

## Glide an element to its new place

Give two nodes that stand for the same thing the same `layout_id`. When one
appears where the other was, Kivact animates it from the last box of the node
that had that id. The animation uses the **layout transition** of the new node,
so without one nothing animates. An id is an attribute of the node, not a style
property.

```rust,no_run
use kivact::motion::{Spring, Transition};
use kivact::prelude::*;

fn tabs() -> impl IntoView {
    let selected = signal(0_usize);
    row((
        tab("Home", selected, 0),
        tab("Inbox", selected, 1),
        tab("Settings", selected, 2),
    ))
    .style(style().gap(16).padding(12))
}

fn tab(label: &'static str, selected: Signal<usize>, index: usize) -> impl IntoView {
    column((
        button(label).on_press(move || selected.set(index)),
        show(move || selected.get() == index, move || {
            // One marker moves between the tabs: each tab mounts it
            // under its own button, and it glides from the last box.
            node().layout_id("tab-marker").style(
                style()
                    .height(3)
                    .background(ColorToken::Accent)
                    .layout_transition(Transition::spring(Spring::STIFF)),
            )
        }),
    ))
}
# fn main() -> kivact::Result<()> { App::new("Tabs").run(tabs) }
```

Rules for `layout_id`:

- An id must be unique among the nodes that are mounted at the same time. A
  second node with the same id is reported once as `duplicate layout_id`.
- A layout motion moves a box to its new place once, so the layout transition
  ignores `repeat` and `alternate`. A style that sets them is reported once.
- With reduced motion on, the layout transition ends at once, unless the
  transition is marked `.essential()`.

## See also

- [Animate values and elements](animation.md): one value, style transitions, enter and exit, and the frame loop.
- [Lay out a screen](layout.md): the scroll container and sizes the examples use.
- [Map what you know from React or SolidJS](coming-from-react-or-solid.md): AnimatePresence, layoutId, useScroll, GSAP timelines and stagger.
- [Test your app](testing.md): the virtual clock that drives timelines in tests.
- API reference: `cargo doc -p kivact --open`, then `kivact::motion` and `kivact::view`.
