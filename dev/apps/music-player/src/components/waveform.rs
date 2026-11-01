//! Role: Full-track waveform with a playback cursor.
//! Click or drag to seek; the
//! arrow keys and the slider actions seek by 5 seconds.

use crate::analysis::waveform;
use crate::audio::{AudioClip, format_time};
use crate::player_state::PlayerCx;
use kivact::control::canvas;
use kivact::draw::TextStyle;
use kivact::draw::{Draw, Point, Size};
use kivact::prelude::*;
use kivact::primitive::NodeExt;
use kivact::style::Color;
use kivact::view::PointerPhase;
use kivact::view::{A11yAction, Handled, PointerId};
use std::time::Duration;

const HEIGHT: f32 = 170.0;
const LEFT: f32 = 45.0;
const RIGHT: f32 = 12.0;
const TOP: f32 = 15.0;
const BOTTOM: f32 = 26.0;

fn background() -> Color {
    rgb(0x111b2d)
}
fn grid() -> Color {
    rgb(0x35445b)
}
fn muted() -> Color {
    rgb(0xabbcd0)
}
fn label(size: f32) -> TextStyle {
    TextStyle {
        size,
        weight: 400,
        color: muted(),
        ..TextStyle::default()
    }
}

fn draw_wave(d: &mut Draw, size: Size, clip: &AudioClip) {
    let width = size.width.max(1.0);
    let height = size.height.max(1.0);
    let right = (width - RIGHT).max(LEFT + 1.0);
    let bottom = (height - BOTTOM).max(TOP + 1.0);
    let middle = (TOP + bottom) / 2.0;
    let half = (bottom - TOP) / 2.0;
    d.fill_rect(size.to_rect(), background());
    for fraction in [0.0_f32, 0.25, 0.5, 0.75, 1.0] {
        let x = LEFT + (right - LEFT) * fraction;
        d.line(Point::new(x, TOP), Point::new(x, bottom), 1.0, grid());
        let label_x = if fraction == 1.0 { x - 27.0 } else { x };
        d.text(
            &format_time(clip.duration().mul_f32(fraction)),
            Point::new(label_x.max(2.0), height - 21.0),
            label(11.0),
        );
    }
    d.line(
        Point::new(LEFT, middle),
        Point::new(right, middle),
        1.0,
        grid(),
    );
    d.text("+1", Point::new(8.0, TOP + 1.0), label(10.0));
    d.text("0", Point::new(16.0, middle - 6.0), label(10.0));
    d.text("−1", Point::new(8.0, bottom - 11.0), label(10.0));
    let columns = (right - LEFT).ceil().clamp(1.0, 1024.0) as usize;
    for (i, (low, high)) in waveform(clip, columns).into_iter().enumerate() {
        let x = LEFT + i as f32 * (right - LEFT) / columns as f32;
        d.line(
            Point::new(x, middle - high * half),
            Point::new(x, middle - low * half),
            1.0,
            rgb(0x47d6bc),
        );
    }
}

fn draw_cursor(d: &mut Draw, size: Size, position: Duration, duration: Duration) {
    let right = (size.width - RIGHT).max(LEFT + 1.0);
    let fraction =
        (position.as_secs_f64() / duration.as_secs_f64().max(0.001)).clamp(0.0, 1.0) as f32;
    let x = LEFT + (right - LEFT) * fraction;
    d.line(
        Point::new(x, TOP - 3.0),
        Point::new(x, (size.height - BOTTOM).max(TOP + 1.0)),
        2.0,
        rgb(0xffd35b),
    );
}

pub(super) fn waveform_view() -> impl IntoView {
    let p = context::<PlayerCx>();
    let node = node_ref();
    let rect = node.rect();
    // The pointer that owns the drag (useRef: never rendered).
    let drag = var(None::<PointerId>);
    let seek_at = move |x: f32| {
        let Some(width) = rect.get_untracked().map(|r| r.width) else {
            return;
        };
        let fraction = ((x - LEFT) / (width - LEFT - RIGHT).max(1.0)).clamp(0.0, 1.0);
        let duration = p.clip.with_untracked(|c| c.0.duration());
        p.seek_to(duration.mul_f32(fraction));
    };
    canvas(move |d, size| p.clip.with(|c| draw_wave(d, size, &c.0)))
        .draw_over(move |d, size| {
            let duration = p.clip.with(|c| c.0.duration());
            draw_cursor(d, size, p.position.get(), duration);
        })
        .on_pointer(move |e| {
            let owner = drag.get();
            match e.phase {
                PointerPhase::Down if owner.is_none_or(|id| id == e.id) => {
                    drag.set(Some(e.id));
                    seek_at(e.pos.x);
                }
                PointerPhase::Move if owner == Some(e.id) => seek_at(e.pos.x),
                PointerPhase::Up if owner == Some(e.id) => {
                    seek_at(e.pos.x);
                    drag.set(None);
                }
                PointerPhase::Cancel if owner == Some(e.id) => drag.set(None),
                _ => return Handled::No,
            }
            Handled::Yes
        })
        .focusable(true)
        .on_key(move |k| match k.key {
            Key::ArrowLeft => {
                p.seek_by(-5);
                Handled::Yes
            }
            Key::ArrowRight => {
                p.seek_by(5);
                Handled::Yes
            }
            _ => Handled::No,
        })
        .on_a11y(A11yAction::Increment, move || p.seek_by(5))
        .on_a11y(A11yAction::Decrement, move || p.seek_by(-5))
        .node_ref(node)
        .role(Role::Slider)
        .name("Waveform; click or drag to seek")
        .value_text(move || format_time(p.position.get()))
        .style(
            style()
                .height(HEIGHT)
                .radius(4)
                .focus(style().ring(2, ColorToken::Focus)),
        )
        .test_id("waveform")
}
