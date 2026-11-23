//! Role: The level Meter: segmented, oriented, multi-channel level bars.

use crate::color::{CANVAS, SUCCESS, WARNING, X};
use kivact::component;
use kivact::geometry::Rect;
use kivact::state::{Machine, Step, Value, machine, memo};
use kivact::style::{Color, color, style};
use kivact::view::{Orientation, Role, node};

#[derive(Clone, Copy, PartialEq, Eq)]
enum MeterClip {
    Clear,
    Held,
}
enum ClipEvent {
    LevelChanged(f64),
    Reset,
}
struct ClipMachine;
impl Machine for ClipMachine {
    type State = MeterClip;
    type Event = ClipEvent;
    type Effect = ();
    fn transition(&self, state: &MeterClip, event: ClipEvent) -> Step<MeterClip, ClipEvent, ()> {
        match event {
            ClipEvent::Reset => Step::to(MeterClip::Clear),
            ClipEvent::LevelChanged(level) if level > 1.0 => Step::to(MeterClip::Held),
            ClipEvent::LevelChanged(_) => Step::to(*state),
        }
    }
}

/// An audio meter receiving normalized linear amplitude (0..1, >1 clips),
/// with clip hold and an accessible numeric level.
///
/// `orientation` lays the channels out as a horizontal strip
/// ([`Orientation::Horizontal`]) rather than the default vertical channel
/// meter. `channels` shows one to eight channels. `segmented` chooses
/// separated LED cells rather than the source's continuous bar.
#[component]
pub fn meter(
    label: Value<String>,
    level: Value<f64>,
    #[prop(default = Orientation::Vertical)] orientation: Value<Orientation>,
    #[prop(default = 2)] channels: Value<usize>,
    #[prop(default = false)] segmented: Value<bool>,
) -> impl Element {
    let horizontal = memo(move || orientation.get() == Orientation::Horizontal);
    let channels = memo(move || channels.get().clamp(1, 8));
    let clip = machine(MeterClip::Clear, || ClipMachine, |_, ()| {});
    let clipped = clip.state();
    clip.send(ClipEvent::LevelChanged(level.get_untracked()));
    clip.feed(level, ClipEvent::LevelChanged);
    node()
        .role(Role::Meter)
        .name(label)
        .value_min(0.0)
        .value_max(1.0)
        .style(style().size(30, 180).background(CANVAS).border_radius(5))
        .style(move || {
            if horizontal.get() {
                style().size(240, 18)
            } else {
                style().size(30, 180)
            }
        })
        .value_now(move || level.get().clamp(0.0, 1.0))
        .value_text(move || format!("{:.1} dB", 20.0 * level.get().max(0.001).log10()))
        .orientation(move || {
            if horizontal.get() {
                Orientation::Horizontal
            } else {
                Orientation::Vertical
            }
        })
        .on_press(move || clip.send(ClipEvent::Reset))
        .draw(move |d, size| paint_meter(d, size, level, horizontal, channels, segmented, clipped))
}

/// Paint measured channels and the canonical clip-hold observation.
fn paint_meter(
    d: &mut kivact::draw::Draw,
    size: kivact::geometry::Size,
    level: Value<f64>,
    horizontal: Value<bool>,
    channels: Value<usize>,
    segmented: Value<bool>,
    clipped: Value<MeterClip>,
) {
    let (horizontal, segmented) = (horizontal.get(), segmented.get());
    let v = level.get().clamp(0.0, 1.0);
    let extent = if horizontal {
        size.width - 10.0
    } else {
        size.height - 12.0
    };
    let width = if horizontal {
        (size.height - 4.0) / channels.get() as f64
    } else {
        (size.width - 4.0) / channels.get() as f64
    };
    for ch in 0..channels.get() {
        let amp = v;
        for i in 0..40 {
            let n = i as f64 / 40.0;
            let color = if n >= amp {
                color::RAISED
            } else if n > 0.9 {
                X
            } else if n > 0.75 {
                WARNING
            } else {
                SUCCESS
            };
            let r = if horizontal {
                Rect::new(
                    2.0 + extent * n,
                    2.0 + ch as f64 * width,
                    (extent / 40.0 - if segmented { 1.0 } else { 0.0 }).max(1.0),
                    width - 1.0,
                )
            } else {
                Rect::new(
                    2.0 + ch as f64 * width,
                    size.height - 2.0 - extent * (n + 0.025),
                    width - 1.0,
                    (extent / 40.0 - if segmented { 1.0 } else { 0.0 }).max(1.0),
                )
            };
            d.fill_rect(r, color);
        }
    }
    let clip = if horizontal {
        Rect::new(size.width - 6.0, 2.0, 4.0, size.height - 4.0)
    } else {
        Rect::new(2.0, 2.0, size.width - 4.0, 4.0)
    };
    let held: Color = if clipped.get() == MeterClip::Held {
        X
    } else {
        X.with_alpha(0.17)
    };
    d.rounded_rect(clip, 2.0, held);
}
