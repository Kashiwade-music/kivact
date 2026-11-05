//! Role: Paint numeric knob and fader geometry using Creative theme tokens.
use kivact::draw::{Draw, LineCap, LineJoin, Path, Stroke, TextFormat, linear_gradient};
use kivact::geometry::{Point, Rect, Size};
use kivact::style::{Color, ColorToken};
pub(super) fn draw_knob(
    d: &mut Draw,
    size: Size,
    n: f64,
    color: kivact::style::Color,
    label: &str,
    value: &str,
    bipolar: bool,
) {
    let p = d.theme().ext::<crate::tokens::Tokens>();
    let scale = (size.width / 58.0).clamp(0.5, 1.0);
    let center = Point::new(size.width / 2.0, 24.0 * scale);
    let point = |a: f64, radius: f64| {
        Point::new(
            center.x + radius * scale * a.sin(),
            center.y - radius * scale * a.cos(),
        )
    };
    let a0 = -2.35;
    let arc = |start: f64, end: f64| {
        let mut path = Path::new().move_to(point(start, 19.0));
        for i in 1..=48 {
            path = path.line_to(point(start + (end - start) * i as f64 / 48.0, 19.0));
        }
        path
    };
    let stroke = Stroke::new(3.5 * scale)
        .cap(LineCap::Round)
        .join(LineJoin::Round);
    d.stroke_path(&arc(a0, 2.35), &stroke, p.hair_strong);
    d.stroke_path(
        &arc(if bipolar { 0.0 } else { a0 }, a0 + n * 4.7),
        &stroke,
        color,
    );
    d.rounded_rect(
        Rect::new(
            center.x - 15.0 * scale,
            9.0 * scale,
            30.0 * scale,
            30.0 * scale,
        ),
        15.0 * scale,
        linear_gradient(180).colors([p.strong, Color::token(ColorToken::Raised)]),
    );
    d.line(
        point(a0 + n * 4.7, 7.0),
        point(a0 + n * 4.7, 12.0),
        2.6,
        p.text_high,
    );
    let label_style = TextFormat {
        size: 10.5,
        color: p.dim,
        align: kivact::style::TextAlign::Center,
        ..TextFormat::default()
    };
    d.text(label, Point::new(center.x, 48.0 * scale + 7.0), label_style);
    d.text(
        value,
        Point::new(center.x, 48.0 * scale + 22.0),
        TextFormat {
            size: 10.5,
            color: p.text,
            family: Some(p.mono_font.clone()),
            align: kivact::style::TextAlign::Center,
            ..TextFormat::default()
        },
    );
}

pub(super) fn draw_fader(
    d: &mut Draw,
    size: Size,
    n: f64,
    color: kivact::style::Color,
    value: &str,
) {
    let p = d.theme().ext::<crate::tokens::Tokens>();
    let x = size.width / 2.0;
    let top = 8.0;
    let height = (size.height - 30.0).max(1.0);
    d.rounded_rect(Rect::new(x - 3.0, top, 6.0, height), 3.0, p.canvas);
    d.rounded_rect(
        Rect::new(x - 2.0, top + height * (1.0 - n), 4.0, height * n),
        2.0,
        color.with_alpha(0.6),
    );
    for i in 0..=10 {
        let y = top + i as f64 * height / 10.0;
        d.line(
            Point::new(x - 17.0, y),
            Point::new(x - 10.0, y),
            1.0,
            p.hair_strong,
        );
    }
    let y = top + height * (1.0 - n);
    d.rounded_rect(
        Rect::new(x - 11.0, y - 5.0, 22.0, 11.0),
        3.0,
        linear_gradient(180).colors([p.muted, p.strong]),
    );
    d.line(
        Point::new(x - 9.0, y),
        Point::new(x + 9.0, y),
        1.0,
        p.canvas,
    );
    d.text(
        value,
        Point::new(x, size.height - 8.0),
        TextFormat {
            size: 10.0,
            color: p.text,
            align: kivact::style::TextAlign::Center,
            family: Some(p.mono_font.clone()),
            ..TextFormat::default()
        },
    );
}
