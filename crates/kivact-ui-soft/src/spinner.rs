//! Role: Neutral loading spinner and its shared arc paint.
use kivact::state::IntoValue;
use kivact::state::Value;

use crate::tokens;
use kivact::draw::LineCap;
use kivact::draw::Path;
use kivact::draw::Stroke;
use kivact::geometry::Point;
use kivact::motion::FrameRate;
use kivact::motion::frame_loop;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::node;
use kivact::view::text;

use kivact::state::IntoText;
use kivact::state::signal;
use kivact::style::Align;
use kivact::style::Color;
use kivact::style::Direction;
use kivact::style::style;
use kivact::view::IntoView;
use kivact::view::Role;
use kivact::view::View;

/// Spinner box dimensions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SpinnerSize {
    Sm,
    #[default]
    Md,
    Lg,
    Xl,
}

/// Spinner ink and track pairing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SpinnerShade {
    #[default]
    Default,
    Subtle,
    OnMedia,
    Inherit,
}

/// Named status indicator for indeterminate work.
pub struct Spinner {
    root: Node,
    size: Value<SpinnerSize>,
    shade: Value<SpinnerShade>,
    label: Option<View>,
    named: bool,
    ink: Option<Color>,
}

#[track_caller]
pub fn spinner() -> Spinner {
    Spinner {
        root: node().role(Role::Status),
        size: (SpinnerSize::Md).into_value(),
        shade: (SpinnerShade::Default).into_value(),
        label: None,
        named: false,
        ink: None,
    }
}

impl Spinner {
    pub fn size(mut self, size: impl IntoValue<SpinnerSize>) -> Self {
        self.size = size.into_value();
        self
    }

    pub fn shade(mut self, shade: impl IntoValue<SpinnerShade>) -> Self {
        self.shade = shade.into_value();
        self
    }

    /// Supply the surrounding ink explicitly when using the inherit shade.
    pub fn ink(mut self, ink: Color) -> Self {
        self.ink = Some(ink);
        self
    }

    pub fn name(mut self, name: impl IntoText) -> Self {
        self.root = self.root.name(name);
        self.named = true;
        self
    }

    pub fn label(mut self, label: impl IntoText) -> Self {
        let label = label.into_text_value();
        if !self.named {
            self.root = self.root.name(label);
            self.named = true;
        }
        self.label_view(text(label))
    }

    pub fn label_view(mut self, label: impl IntoView) -> Self {
        self.label = Some(label.into_view());
        self
    }
}

impl NodeBuilder for Spinner {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Spinner {
    fn into_view(mut self) -> View {
        if !self.named {
            self.root = self
                .root
                .name(kivact::i18n::message("soft.spinner.loading", "Loading"));
        }
        let (diameter, width) = match self.size.get() {
            SpinnerSize::Sm => (10.0, 2.0),
            SpinnerSize::Md => (14.0, 3.0),
            SpinnerSize::Lg => (18.0, 3.0),
            SpinnerSize::Xl => (28.0, 4.0),
        };
        let side = diameter + width * 2.0;
        let shade = self.shade.get();
        let inherit = self.ink;
        let elapsed = signal(0.0f64);
        frame_loop(move |frame| elapsed.set(frame.now.as_secs_f64() * 1000.0))
            .start(FrameRate::Display);
        let glyph = node().style(move || {
            let t = tokens();
            let (ink, track) = match shade {
                SpinnerShade::Default => (t.colors.accent, t.colors.track),
                SpinnerShade::Subtle => (t.colors.text_secondary, t.colors.track),
                SpinnerShade::OnMedia => {
                    (t.colors.on_dark, t.colors.on_dark.with_alpha(77.0 / 255.0))
                }
                SpinnerShade::Inherit => {
                    let ink = inherit.unwrap_or(t.colors.text_primary);
                    (ink, ink.with_alpha(0.3))
                }
            };
            let cycle = if kivact::platform::reduced_motion().get() {
                3000.0
            } else {
                t.motion.slow_min as f64
            };
            let start = -90.0 + elapsed.get().rem_euclid(cycle) * 360.0 / cycle;
            style().size(side, side).shrink(0).paint(move |d, size| {
                let center = Point::new(size.width / 2.0, size.height / 2.0);
                let stroke = Stroke::new(width).cap(LineCap::Round);
                d.stroke_path(
                    &Path::new().arc(center, diameter / 2.0, 0, 360, false),
                    &stroke,
                    track,
                );
                d.stroke_path(
                    &Path::new().arc(center, diameter / 2.0, start, start + 135.0, false),
                    &stroke,
                    ink,
                );
            })
        });
        self.root
            .base_style(
                style()
                    .direction(Direction::Column)
                    .align_items(Align::Center)
                    .gap(8)
                    .shrink(0),
            )
            .child(glyph)
            .child(self.label.map(|label| {
                node()
                    .style(|| {
                        style()
                            .color(tokens().colors.text_primary)
                            .font_size(14)
                            .line_height(20.0 / 14.0)
                            .weight(700)
                    })
                    .child(label)
            }))
            .into_view()
    }
}

kivact::state::constant_value!(SpinnerShade, SpinnerSize);
