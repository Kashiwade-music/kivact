//! Role: Animated halo demonstration for the Pop catalog.
use kivact::control::{Node, NodeBuilder, node};
use kivact::draw::{LineCap, LineJoin, Path, Point, Rect, Stroke};
use kivact::i18n::message;
use kivact::motion::{Animated, Easing, Transition};
use kivact::style::{Color, style};
use kivact::view::Role;
use kivact_ui_pop::Tokens;
/// A floating halo with a soft shadow and an animated cyan-to-yellow pulse.
#[track_caller]
pub(super) fn halo_loader() -> Node {
    let pulse = Animated::from(
        0.0f32,
        1.0f32,
        Transition::tween(800, Easing::EaseInOut)
            .repeat(None)
            .alternate()
            .decorative(false),
    );
    node()
        .role(Role::Status)
        .name(message("pop.loader.label", "Loading"))
        .style(style().size(48, 48))
        .draw(move |draw, _| {
            let p = draw.theme().get::<Tokens>();
            let phase = pulse.get();
            let mut ring = Path::new();
            for i in 0..=64 {
                let a = std::f32::consts::TAU * i as f32 / 64.0;
                let point = Point::new(24.0 + 20.0 * a.cos(), 20.0 - phase * 6.0 + 8.0 * a.sin());
                ring = if i == 0 {
                    ring.move_to(point)
                } else {
                    ring.line_to(point)
                };
            }
            draw.stroke_path(
                &ring,
                &Stroke::new(3.0).cap(LineCap::Round).join(LineJoin::Round),
                p.cyan.mix_in(p.yellow, phase, draw.theme()),
            );
            draw.rounded_rect(
                Rect::new(14.0 + phase * 3.0, 30.0, 20.0 - phase * 6.0, 8.0),
                4.0,
                p.blue
                    .mix_in(Color::TRANSPARENT, 0.78 + phase * 0.11, draw.theme()),
            );
        })
}
