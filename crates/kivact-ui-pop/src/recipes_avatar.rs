//! Role: Supply Pop avatar appearance and decorative halo and presence parts.
use crate::Tokens;
use kivact::draw::{Shadow, linear_gradient};
use kivact::motion::{Animated, Easing, Repeat, Transition};
use kivact::state::{IntoValue, Value};
use kivact::style::{Align, Color, Direction, Justify, Style, Theme, pct, style, use_theme};
use kivact::ui::{AvatarState, ControlSize, Look, Variant};
use kivact::view::{IntoView, node, show};

// xtask-allow(state-fields): Independent reactive decoration and appearance options, never runtime lifecycle flags.
/// Continuous hue, pixel diameter and optional Pop avatar decorations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AvatarLook {
    size: Value<f64>,
    hue: Value<f64>,
    halo: Value<bool>,
    online: Value<bool>,
}
impl Default for AvatarLook {
    fn default() -> Self {
        Self {
            size: 40.0.into_value(),
            hue: 205.0.into_value(),
            halo: false.into_value(),
            online: false.into_value(),
        }
    }
}
impl Variant for AvatarLook {}
impl AvatarLook {
    pub fn size(mut self, size: impl IntoValue<f64>) -> Self {
        let size = size.into_value();
        self.size = (move || size.get().max(16.0)).into_value();
        self
    }
    pub fn hue(mut self, hue: impl IntoValue<f64>) -> Self {
        self.hue = hue.into_value();
        self
    }
    pub fn halo(mut self, halo: impl IntoValue<bool>) -> Self {
        self.halo = halo.into_value();
        self
    }
    pub fn online(mut self, online: impl IntoValue<bool>) -> Self {
        self.online = online.into_value();
        self
    }
    /// Decorative parts mounted in the standard avatar's status-content slot.
    pub fn decorations(self) -> impl IntoView {
        let size = self.size;
        let hue = self.hue;
        let halo = show(self.halo, move || {
            let offset = Animated::from(
                0.0f64,
                -3.0f64,
                Transition::tween(1600, Easing::EaseInOut)
                    .repeat(Repeat::Forever)
                    .alternate(),
            );
            node().style(move || {
                style()
                    .position(kivact::style::Position::Absolute)
                    .left(pct(14))
                    .right(pct(14))
                    .top(size.get() * -0.22)
                    .height(size.get() * 0.2)
                    .radius(pct(50))
                    .border(
                        (size.get() * 0.045).max(2.0),
                        kivact::style::hsl(hue.get(), 0.95, 0.62),
                    )
                    .shadow(Shadow::new(
                        0,
                        0,
                        8,
                        kivact::style::hsl(hue.get(), 1.0, 0.65),
                    ))
                    .translate(0, offset.get())
            })
        });
        let online = show(self.online, move || {
            node().style(move || {
                let t = &use_theme().get();
                style()
                    .position(kivact::style::Position::Absolute)
                    .right(0)
                    .bottom(0)
                    .size(size.get() * 0.28, size.get() * 0.28)
                    .radius(999)
                    .background(t.ext::<Tokens>().mint)
                    .ring(2, t.colors().surface)
            })
        });

        (halo, online)
    }
}
pub(crate) fn avatar(_: &Theme, look: &Look<AvatarState>) -> Style {
    let size = look.variant::<AvatarLook>().map_or_else(
        || match look.size {
            ControlSize::ExtraSmall => 24.0,
            ControlSize::Small => 32.0,
            ControlSize::Medium => 40.0,
            ControlSize::Large => 56.0,
        },
        |v| v.size.get(),
    );
    let hue = look.variant::<AvatarLook>().map_or(205.0, |v| v.hue.get());
    style()
        .size(size, size)
        .shrink(0)
        .radius(999)
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .color(Color::WHITE)
        .background(linear_gradient(140).colors([
            kivact::style::hsl(hue, 0.85, 0.7),
            kivact::style::hsl(hue + 18.0, 0.75, 0.52),
        ]))
        .font_family("Barlow, sans-serif")
        .italic(true)
        .weight(800)
        .font_size(size * 0.36)
}
