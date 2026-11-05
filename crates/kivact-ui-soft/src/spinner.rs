//! Role: Define Neutral spinner ink variants and paint the canonical spinner glyph.
use crate::Tokens;
use kivact::draw::{LineCap, Path, Stroke};
use kivact::geometry::Point;
use kivact::style::{Color, Style, Theme, style};
use kivact::ui::{ControlSize, Look, SpinnerState};
/// Spinner ink and track pairing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SpinnerShade {
    #[default]
    Default,
    Subtle,
    OnMedia,
    Inherit,
}
/// Constant ink treatment of a standard spinner.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SpinnerVariant {
    pub shade: SpinnerShade,
    pub ink: Option<Color>,
}
impl kivact::ui::Variant for SpinnerVariant {}
pub(crate) fn spinner(theme: &Theme, s: &Look<SpinnerState>) -> Style {
    let t = theme.ext::<Tokens>();
    let (diameter, width) = match s.size {
        ControlSize::ExtraSmall => (10.0, 2.0),
        ControlSize::Small => (14.0, 3.0),
        ControlSize::Medium => (18.0, 3.0),
        ControlSize::Large => (28.0, 4.0),
    };
    let variant = s.variant::<SpinnerVariant>().copied().unwrap_or_default();
    let (ink, track) = match variant.shade {
        SpinnerShade::Default => (t.colors.accent, t.colors.track),
        SpinnerShade::Subtle => (t.colors.text_secondary, t.colors.track),
        SpinnerShade::OnMedia => (t.colors.on_dark, t.colors.on_dark.with_alpha(77.0 / 255.0)),
        SpinnerShade::Inherit => {
            let ink = variant.ink.unwrap_or(t.colors.text_primary);
            (ink, ink.with_alpha(0.3))
        }
    };
    let start = -90.0 + s.turn;
    style()
        .size(diameter + width * 2.0, diameter + width * 2.0)
        .shrink(0)
        .paint(move |draw, size| {
            let center = Point::new(size.width / 2.0, size.height / 2.0);
            let stroke = Stroke::new(width).cap(LineCap::Round);
            draw.stroke_path(
                &Path::new().arc(center, diameter / 2.0, 0, 360, false),
                &stroke,
                track,
            );
            draw.stroke_path(
                &Path::new().arc(center, diameter / 2.0, start, start + 135.0, false),
                &stroke,
                ink,
            );
        })
}
