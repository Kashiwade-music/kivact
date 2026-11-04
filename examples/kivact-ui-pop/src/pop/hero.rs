//! Role: The catalog's landing hero: a gradient banner with actions and statistics.
use kivact::draw::{LineCap, LineJoin, Stroke, linear_gradient};
use kivact::state::IntoText;
use kivact::style::use_theme;
use kivact::style::{Align, Color, pct, style};
use kivact::view::IntoView;
use kivact::view::{Node, column, node, row, text};
use kivact_ui_pop::Tokens;

/// Gradient hero with blueprint stripes and the halo ornament. `heading` is a
/// custom heading, such as inline marks built with public controls.
#[track_caller]
pub(super) fn hero(
    heading: impl IntoView,
    subtitle: impl IntoText,
    actions: impl IntoView,
    statistics: impl IntoView,
) -> Node {
    row((
        column((
            node().child(heading).style(style().margin_bottom(8)),
            text(subtitle).style(
                style()
                    .max_width(520)
                    .color(kivact::style::rgba(0xffffff, 0.88)),
            ),
            row(actions).style(style().gap(10).wrap(true).margin_top(20)),
        ))
        .style(style().grow(1).basis(540).min_width(0)),
        row(statistics).style(style().gap(10).grow(0).basis(440).align_items(Align::End)),
    ))
    .style(move || {
        let t = &use_theme().get();
        let p = t.ext::<Tokens>();
        style()
            .padding(32)
            .min_height(208)
            .gap(28)
            .align_items(Align::End)
            .radius(14)
            .color(Color::WHITE)
            .background(
                linear_gradient(120)
                    .stop(p.blue_deep, pct(0))
                    .stop(p.blue, pct(45))
                    .stop(p.cyan, pct(100)),
            )
            .clip(true)
    })
    .draw(|d, s| {
        let stripe = kivact::style::rgba(0xffffff, 0.06);
        for i in 0..((s.width / 22.0) as usize + 20) {
            let x = i as f64 * 22.0 - 250.0;
            d.line(
                kivact::geometry::Point::new(x, s.height),
                kivact::geometry::Point::new(x + s.height * 0.47, 0.0),
                2.0,
                stripe,
            );
        }
        let p = d.theme().ext::<Tokens>();
        let x = s.width * 0.7;
        d.fill_path(
            &kivact::draw::Path::new()
                .move_to(kivact::geometry::Point::new(x, -40.0))
                .line_to(kivact::geometry::Point::new(x + 20.0, -40.0))
                .line_to(kivact::geometry::Point::new(x - 80.0, s.height))
                .line_to(kivact::geometry::Point::new(x - 100.0, s.height))
                .close(),
            p.yellow.mix_in(Color::TRANSPARENT, 0.6, d.theme()),
        );
        let center = kivact::geometry::Point::new(s.width - 130.0, 110.0);
        let mut circle = kivact::draw::Path::new();
        for i in 0..=80 {
            let a = std::f64::consts::TAU * i as f64 / 80.0;
            let pt = kivact::geometry::Point::new(
                center.x + 180.0 * a.cos(),
                center.y + 180.0 * a.sin(),
            );
            circle = if i == 0 {
                circle.move_to(pt)
            } else {
                circle.line_to(pt)
            };
        }
        d.stroke_path(
            &circle,
            &Stroke::new(14.0).cap(LineCap::Round).join(LineJoin::Round),
            kivact::style::rgba(0xffffff, 0.16),
        );
        let inner = kivact::draw::Path::new()
            .arc(center, 122, 0, 360, false)
            .close();
        d.stroke_path(
            &inner,
            &kivact::draw::Stroke::new(2).dash([4.0, 4.0]),
            kivact::style::rgba(0xffffff, 0.3),
        );
    })
}
