//! Role: The level Meter: segmented, oriented, multi-channel level bars.

use kivact::geometry::Rect;
use kivact::view::{Node, NodeBuilder, node};

use kivact::state::{IntoText, IntoValue};
use kivact::state::{Machine, Step, Value, effect, machine, memo};
use kivact::style::{Color, ColorToken, style};
use kivact::view::{IntoView, Orientation, Role, View};

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

/// An audio meter with clip hold and accessible numeric level.
pub struct Meter {
    root: Node,
    level: Value<f64>,
    horizontal: Value<bool>,
    channels: Value<usize>,
    segmented: Value<bool>,
}

/// An audio meter receiving normalized linear amplitude (0..1, >1 clips).
#[track_caller]
pub fn meter(label: impl IntoText, level: impl IntoValue<f64>) -> Meter {
    Meter {
        root: node()
            .role(Role::Meter)
            .name(label)
            .value_min(0.0)
            .value_max(1.0)
            .style(
                style()
                    .size(30, 180)
                    .background(crate::ColorName::Canvas)
                    .radius(5),
            ),
        level: level.into_value(),
        horizontal: false.into_value(),
        channels: 2.into_value(),
        segmented: false.into_value(),
    }
}

impl Meter {
    /// Choose separated LED cells rather than the source's continuous bar
    /// while true (constant or reactive).
    #[track_caller]
    pub fn segmented(mut self, enabled: impl IntoValue<bool>) -> Self {
        self.segmented = enabled.into_value();
        self
    }
    /// Lay the channels out as a horizontal strip
    /// ([`Orientation::Horizontal`]) rather than the default vertical channel
    /// meter (constant or reactive).
    #[track_caller]
    pub fn orientation(mut self, orientation: impl IntoValue<Orientation>) -> Self {
        let orientation = orientation.into_value();
        let enabled = memo(move || orientation.get() == Orientation::Horizontal);
        self.horizontal = enabled;
        self.root = self.root.style(move || {
            if enabled.get() {
                style().size(240, 18)
            } else {
                style().size(30, 180)
            }
        });
        self
    }
    /// Display one to eight channels.
    pub fn channels(mut self, channels: impl IntoValue<usize>) -> Self {
        let channels = channels.into_value();
        self.channels = memo(move || channels.get().clamp(1, 8));
        self
    }
}

impl NodeBuilder for Meter {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Meter {
    fn into_view(self) -> View {
        let Self {
            root,
            level,
            horizontal,
            channels,
            segmented,
        } = self;
        let clip = machine(MeterClip::Clear, || ClipMachine, |_, ()| {});
        let clipped = clip.state();
        effect(move || {
            clip.send(ClipEvent::LevelChanged(level.get()));
        });
        root.value_now(move || level.get().clamp(0.0, 1.0))
            .value_text(move || format!("{:.1} dB", 20.0 * level.get().max(0.001).log10()))
            .orientation(move || {
                if horizontal.get() {
                    Orientation::Horizontal
                } else {
                    Orientation::Vertical
                }
            })
            .on_press(move || clip.send(ClipEvent::Reset))
            .draw(move |d, size| {
                paint_meter(d, size, level, horizontal, channels, segmented, clipped)
            })
            .into_view()
    }
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
    let p = d.theme().ext::<crate::tokens::Tokens>();
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
                Color::token(ColorToken::Raised)
            } else if n > 0.9 {
                p.x
            } else if n > 0.75 {
                p.warning
            } else {
                p.success
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
    d.rounded_rect(
        clip,
        2.0,
        if clipped.get() == MeterClip::Held {
            p.x
        } else {
            p.x.with_alpha(0.17)
        },
    );
}
