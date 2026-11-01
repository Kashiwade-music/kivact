//! Role: 64-band spectrum of the PCM around the playback position.

use crate::analysis::{BANDS, MAX_HZ, spectrum};
use crate::player_state::PlayerCx;
use kivact::control::canvas;
use kivact::draw::TextStyle;
use kivact::draw::{Draw, Point, Rect, Size};
use kivact::prelude::*;

const HEIGHT: f32 = 190.0;

fn label(size: f32) -> TextStyle {
    TextStyle {
        size,
        weight: 400,
        color: rgb(0xabbcd0),
        ..TextStyle::default()
    }
}

fn draw_spectrum(d: &mut Draw, size: Size, bands: &[f32; BANDS], max_hz: f32) {
    let width = size.width.max(1.0);
    let height = size.height.max(1.0);
    let left = 50.0;
    let right = (width - 18.0).max(left + 1.0);
    let top = 12.0;
    let bottom = (height - 32.0).max(top + 1.0);
    let grid = rgb(0x35445b);
    d.fill_rect(size.to_rect(), rgb(0x111b2d));
    for fraction in [0.0_f32, 0.25, 0.5, 0.75, 1.0] {
        let x = left + (right - left) * fraction;
        let label_x = if fraction == 1.0 { x - 42.0 } else { x };
        d.line(Point::new(x, top), Point::new(x, bottom), 1.0, grid);
        d.text(
            &format!("{:.0} Hz", max_hz * fraction),
            Point::new(label_x, height - 25.0),
            label(11.0),
        );
    }
    for (fraction, text) in [(0.0, "1.0"), (0.5, "0.5"), (1.0, "0")] {
        let y = top + (bottom - top) * fraction;
        d.line(Point::new(left, y), Point::new(right, y), 1.0, grid);
        d.text(text, Point::new(10.0, y - 6.0), label(10.0));
    }
    let step = (right - left) / BANDS as f32;
    for (index, amplitude) in bands.iter().enumerate() {
        let bar = (amplitude.clamp(0.0, 1.0) * (bottom - top)).max(1.0);
        d.fill_rect(
            Rect::new(
                left + index as f32 * step + 1.0,
                bottom - bar,
                (step - 2.0).max(1.0),
                bar,
            ),
            rgb(0x6fa6ff),
        );
    }
}

pub(super) fn spectrum_view() -> impl IntoView {
    let p = context::<PlayerCx>();
    let bands = memo(move || {
        let position = p.position.get();
        p.clip.with(|c| {
            let center = (position.as_secs_f64() * f64::from(c.0.sample_rate)) as usize;
            spectrum(&c.0, center)
        })
    });
    let max_hz = memo(move || p.clip.with(|c| MAX_HZ.min(c.0.sample_rate as f32 / 2.0)));
    canvas(move |d, size| bands.with(|b| draw_spectrum(d, size, b, max_hz.get())))
        .name("Frequency spectrum")
        .description("64 frequency bands from 0 to 8000 hertz, amplitude 0 to 1")
        .style(style().height(HEIGHT).radius(4))
        .test_id("spectrum")
}
