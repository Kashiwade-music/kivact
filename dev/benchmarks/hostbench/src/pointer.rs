//! Role: Benchmarks of the writes a pointer drag or hover makes, one per frame.
//!
//! The public API cannot send pointer input, so each scenario makes, on
//! every display frame, the write a pointer move makes in the component
//! it imitates, on a page of text beneath (`PARAGRAPHS`), and the frame
//! log times what the host does with it. The pointer path itself (hit
//! testing, the hover chain, the drag session) is not in the measurement.
//!
//! - `drag_backdrop`: a drawer pulled by its handle. A panel fixed to the
//!   window's bottom moves with a translate (a paint change) over a
//!   full-window scrim that blurs the page behind it (`backdrop_blur`).
//! - `drag_slider`: a slider's thumb dragged back and forth: the slider's
//!   value changes, which places the thumb and the filled range again
//!   (a layout of their absolute boxes).
//! - `hover_tooltip`: a chart's hover: a canvas paints the band under the
//!   pointer and a card follows the pointer by a translate; every 8th
//!   frame the pointer enters another band, which paints the canvas again
//!   and rebuilds the card's content.

use kivact::view::{View, dynamic};
use std::cell::{Cell, RefCell};

use kivact::draw::Rect;
use kivact::prelude::*;
use kivact::style::{Color, Position};

use crate::report::{Json, mark, stats};
use crate::scenes::{SETTLE, every_frame, paragraph_text};

/// Measured ticks (about 3 s at 60 Hz).
const MEASURED: u64 = 180;
/// Paragraphs of the page under the moving part.
const PARAGRAPHS: usize = 60;
/// Bands of the hovered canvas.
const BANDS: usize = 12;

/// The page of text the scenarios draw over.
fn page() -> Node {
    column(
        (0..PARAGRAPHS)
            .map(|i| text(paragraph_text(i)).into_view())
            .collect::<Vec<_>>(),
    )
    .style(style().gap(8).padding(16))
}

/// A triangle wave from 0 to `span` and back, `period` ticks long.
fn wave(tick: u64, period: u64, span: f32) -> f32 {
    let phase = (tick % period) as f32 / period as f32;
    span * (1.0 - (2.0 * phase - 1.0).abs())
}

/// Measure `step` on every tick for `MEASURED` ticks and report `name`.
fn measure(name: &'static str, mut step: impl FnMut(u64) + 'static) {
    let begin = Cell::new(0u128);
    let ticks = RefCell::new(Vec::new());
    every_frame(move |tick, info| {
        if tick < SETTLE {
            return true;
        }
        if tick == SETTLE {
            begin.set(mark("phase_begin"));
        }
        if tick < SETTLE + MEASURED {
            step(tick - SETTLE);
            ticks.borrow_mut().push(info.delta.as_secs_f64() * 1e3);
            return true;
        }
        let end = mark("phase_end");
        Json::new(name)
            .int("paragraphs", PARAGRAPHS as u128)
            .frames(begin.get(), end)
            .stats("tick_ms", stats(ticks.take()))
            .finish();
        false
    });
}

pub(crate) fn drag_backdrop() -> View {
    let pulled = signal(0.0f32);
    measure("drag_backdrop", move |tick| {
        pulled.set(wave(tick, 60, 240.0))
    });
    let scrim = node().style(
        style()
            .position(Position::Fixed)
            .inset(0)
            .background(Color::BLACK.with_alpha(0.1))
            .backdrop_blur(4),
    );
    let panel =
        column((text("Move Goal"), text("Set your daily activity goal."))).style(move || {
            style()
                .position(Position::Fixed)
                .left(0)
                .right(0)
                .bottom(0)
                .height(320)
                .padding(16)
                .gap(8)
                .background(rgb(0xffffff))
                .translate(0, pulled.get())
        });
    column((page(), scrim, panel)).into_view()
}

pub(crate) fn drag_slider() -> View {
    let value = signal(50.0f64);
    measure("drag_slider", move |tick| {
        value.set(f64::from(wave(tick, 60, 100.0)).round())
    });
    column((
        slider("Volume", value).style(style().width(320)),
        text(move || format!("Volume: {}", value.get())),
        page(),
    ))
    .style(style().gap(12).padding(16))
    .into_view()
}

pub(crate) fn hover_tooltip() -> View {
    let at = signal((0.0f32, 0.0f32));
    let band = signal(0usize);
    measure("hover_tooltip", move |tick| {
        let x = wave(tick, BANDS as u64 * 8 * 2, 640.0);
        at.set((x, 120.0 + wave(tick, 40, 60.0)));
        band.set(((x / 640.0) * (BANDS as f32 - 1.0)).round() as usize);
    });
    let canvas = node()
        .style(style().width(640).height(320).shrink(0))
        .draw(move |d, size| {
            let width = size.width / BANDS as f32;
            let hovered = band.get();
            for i in 0..BANDS {
                let shade = if i == hovered { 0xe5e7eb } else { 0xffffff };
                d.fill_rect(
                    Rect::new(i as f32 * width, 0.0, width, size.height),
                    rgb(shade),
                );
                let height = size.height * (0.3 + 0.05 * (i % 7) as f32);
                d.fill_rect(
                    Rect::new(
                        i as f32 * width + 8.0,
                        size.height - height,
                        width - 16.0,
                        height,
                    ),
                    rgb(0x4f86f7),
                );
            }
        });
    let card = dynamic(
        move || band.get(),
        move |band| {
            column((
                text(format!("Row {band}")),
                text(format!("Desktop {}", band * 37 % 300)),
                text(format!("Mobile {}", band * 53 % 300)),
            ))
            .hit_testable(false)
            .style(move || {
                let (x, y) = at.get();
                style()
                    .position(Position::Absolute)
                    .left(0)
                    .top(0)
                    .min_width(128)
                    .padding(8)
                    .gap(4)
                    .background(rgb(0xffffff))
                    .border(1, rgb(0xe5e7eb))
                    .translate(x + 14.0, y)
            })
        },
    );
    column((
        node()
            .style(
                style()
                    .width(640)
                    .direction(kivact::style::Direction::Column),
            )
            .child(canvas)
            .child(card),
        page(),
    ))
    .style(style().gap(12).padding(16))
    .into_view()
}
