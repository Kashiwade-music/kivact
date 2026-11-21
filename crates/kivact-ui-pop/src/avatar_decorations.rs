//! Role: Draw the Pop halo ring and online dot of an avatar's status slot.

use crate::color;
use kivact::component;
use kivact::motion::{Animated, Easing, Repeat, Transition};
use kivact::state::Value;
use kivact::style::Shadow;
use kivact::style::{Position, pct, style};
use kivact::view::{node, show};

/// The halo ring above an avatar and the online dot at its corner, each
/// shown when its flag is set. Mount it in an avatar's `status_content`:
/// `avatar(..).status_content(avatar_decorations().halo(true))`.
#[component]
pub fn avatar_decorations(
    #[prop(default = 40.0)] size: Value<f64>,
    #[prop(default = 205.0)] hue: Value<f64>,
    #[prop(default = false)] halo: Value<bool>,
    #[prop(default = false)] online: Value<bool>,
) -> impl Element {
    let halo = show(halo, move || {
        let offset = Animated::from(
            0.0f64,
            -3.0f64,
            Transition::tween(1600, Easing::EaseInOut)
                .repeat(Repeat::Forever)
                .alternate(),
        );
        node().style(move || {
            let size = size.get().max(16.0);
            style()
                .position(Position::Absolute)
                .left(pct(14))
                .right(pct(14))
                .top(size * -0.22)
                .height(size * 0.2)
                .border_radius(pct(50))
                .border((size * 0.045).max(2.0), color::avatar_halo(hue.get()))
                .shadow(Shadow::new(0, 0, 8, color::avatar_glow(hue.get())))
                .translate(0, offset.get())
        })
    });
    let online = show(online, move || {
        node().style(move || {
            let size = size.get().max(16.0);
            style()
                .position(Position::Absolute)
                .right(0)
                .bottom(0)
                .size(size * 0.28, size * 0.28)
                .border_radius(999)
                .background(color::MINT)
                .outline(2, color::SURFACE)
        })
    });
    node().child((halo, online))
}
