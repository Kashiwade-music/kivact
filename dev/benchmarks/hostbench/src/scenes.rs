//! Role: Benchmark scenario definitions.
//!
//! The startup, animation, text, resize and idle scenarios, and the
//! per-frame driver they share with the others.

use kivact::view::View;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use kivact::Binding;
use kivact::motion::FrameRate;
use kivact::motion::{FrameInfo, FrameLoop, frame_loop};
use kivact::prelude::*;
use kivact::style::Color;
use kivact::task::sleep;
use kivact::task::spawn;

use crate::report::{Json, frame_log, main_ns, mark, ms, stats, unix_ns};

/// Ticks to wait after mount before measuring (the first frames, font and
/// glyph caches).
pub(crate) const SETTLE: u64 = 30;
/// Measured ticks of the per-frame scenarios (about 3 s at 60 Hz).
const MEASURED: u64 = 180;

/// Run `step(tick, info)` on every display frame from mount until it
/// returns `false`.
pub(crate) fn every_frame(step: impl FnMut(u64, FrameInfo) -> bool + 'static) {
    let slot: Rc<Cell<Option<FrameLoop>>> = Rc::new(Cell::new(None));
    let step = RefCell::new(step);
    let tick = Cell::new(0u64);
    let this = slot.clone();
    let frames = frame_loop(move |info| {
        let t = tick.get();
        tick.set(t + 1);
        if !(step.borrow_mut())(t, info)
            && let Some(l) = this.get()
        {
            l.stop();
        }
    });
    slot.set(Some(frames));
    on_mount(move || frames.start(FrameRate::Display));
}

/// A paragraph of mixed English and Japanese text.
pub(crate) fn paragraph_text(i: usize) -> String {
    const EN: [&str; 4] = [
        "The quick brown fox jumps over the lazy dog while the reactive core sends only the changed properties to the host.",
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
        "Layout, shaping and painting run on the host; the benchmark measures how fast each host turns a change into pixels.",
        "Pack my box with five dozen liquor jugs; sphinx of black quartz, judge my vow; how vexingly quick daft zebras jump.",
    ];
    const JA: [&str; 4] = [
        "吾輩は猫である。名前はまだ無い。どこで生れたかとんと見当がつかぬ。",
        "日本語の文章は単語の間に空白がないので、改行の位置は辞書で決める。",
        "春はあけぼの。やうやう白くなりゆく山ぎは、すこしあかりて、紫だちたる雲のほそくたなびきたる。",
        "描画の速さと文字の美しさを、同じ場面で二つの host に比べてもらう。",
    ];
    format!("{i}. {} {} {}", EN[i % 4], JA[i % 4], EN[(i + 1) % 4])
}

fn tile_color(i: usize) -> Color {
    const COLORS: [u32; 6] = [0x4f86f7, 0xf7a14f, 0x5cc489, 0xd96bd1, 0xe0c341, 0x5bc0de];
    rgb(COLORS[i % COLORS.len()])
}

/// Process start to the first presented frame of a small form and a
/// 100-row list. `main_to_setup` is the host's start (window, fonts, GPU)
/// before the root component runs.
pub(crate) fn startup() -> View {
    let setup = unix_ns();
    let name = signal(String::new());
    let agree = signal(false);
    let rows: Vec<View> = (0..100)
        .map(|i| {
            row((
                text(format!("{i:03}")),
                text(paragraph_text(i).chars().take(48).collect::<String>()),
            ))
            .style(style().gap(12).height(22).shrink(0))
            .into_view()
        })
        .collect();
    every_frame(move |tick, _| {
        if tick < SETTLE {
            return true;
        }
        let log = frame_log();
        // The first frame after the root's setup (a host may present the
        // window once before it starts the client).
        let Some(first) = log.first_after(setup) else {
            // Keep waiting for a frame while the log is on (10 s at most).
            return log.enabled && tick < 600;
        };
        let main = main_ns();
        Json::new("startup")
            .num("main_to_setup_ms", ms(setup - main))
            .num("main_to_first_present_ms", ms(first.end - main))
            .num("setup_to_first_present_ms", ms(first.end - setup))
            .num("first_work_ms", first.work as f64 / 1e6)
            .int("main_unix_ns", main)
            .int("first_present_unix_ns", first.end)
            .finish();
        false
    });
    column((
        text("hostbench — 起動の計測").style(style().font_size(20)),
        row((
            kivact::control::node()
                .name("Name / 名前")
                .part(kivact::interaction::use_text_edit(name).control()),
            {
                let checked: Binding<bool> = (agree).into();
                node()
                    .role(Role::CheckBox)
                    .focusable(true)
                    .name("")
                    .checked(checked)
                    .on_press(move || checked.set(!checked.get()))
                    .child(text(move || {
                        (if checked.get() { "[x]" } else { "[ ]" }).to_owned()
                    }))
                    .child(text(""))
                    .style(
                        style()
                            .padding(8)
                            .min_height(32)
                            .focus(style().ring(2, ColorToken::Focus)),
                    )
            },
            {
                let label = ("Save").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
                    .style(
                        style()
                            .padding(8)
                            .min_height(32)
                            .background(ColorToken::SurfaceAlt)
                            .focus(style().ring(2, ColorToken::Focus)),
                    )
            },
            {
                let label = ("Cancel").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
                    .style(
                        style()
                            .padding(8)
                            .min_height(32)
                            .background(ColorToken::SurfaceAlt)
                            .focus(style().ring(2, ColorToken::Focus)),
                    )
            },
        ))
        .style(style().gap(8)),
        scroll(rows).style(style().grow(1).min_height(0)),
    ))
    .style(style().padding(16).gap(8).grow(1).min_height(0))
    .into_view()
}

/// 200 boxes moved and faded by one signal, set every frame for 3 s.
pub(crate) fn animation() -> View {
    const BOXES: usize = 200;
    let t = signal(0.0f32);
    let boxes: Vec<View> = (0..BOXES)
        .map(|i| {
            node()
                .style(move || {
                    let phase = t.get() * 3.0 + i as f32 * 0.31;
                    style()
                        .size(28, 28)
                        .radius(6)
                        .background(tile_color(i))
                        .opacity(0.55 + 0.45 * (phase * 0.5).sin().abs())
                        .translate(phase.sin() * 12.0, phase.cos() * 12.0)
                })
                .into_view()
        })
        .collect();
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
            t.set(info.now.as_secs_f32());
            ticks.borrow_mut().push(info.delta.as_secs_f64() * 1e3);
            return true;
        }
        let end = mark("phase_end");
        Json::new("animation")
            .int("boxes", BOXES as u128)
            .frames(begin.get(), end)
            .stats("tick_ms", stats(ticks.take()))
            .finish();
        false
    });
    row(boxes)
        .style(style().wrap(true).gap(16).padding(24))
        .into_view()
}

/// 300 wrapped paragraphs (English and Japanese) in a scroll box; the 12
/// at the top change text every frame for 3 s.
pub(crate) fn text_heavy() -> View {
    const PARAGRAPHS: usize = 300;
    const LIVE: usize = 12;
    let version = signal(0u64);
    let paragraphs: Vec<View> = (0..PARAGRAPHS)
        .map(|i| {
            let base = paragraph_text(i);
            let t = if i < LIVE {
                text(move || format!("{base} [{}]", version.get()))
            } else {
                text(base)
            };
            t.style(style().text_wrap(true).padding_y(3)).into_view()
        })
        .collect();
    let begin = Cell::new(0u128);
    every_frame(move |tick, _| {
        if tick < SETTLE {
            return true;
        }
        if tick == SETTLE {
            begin.set(mark("phase_begin"));
        }
        if tick < SETTLE + MEASURED {
            version.set(tick);
            return true;
        }
        let end = mark("phase_end");
        Json::new("text")
            .int("paragraphs", PARAGRAPHS as u128)
            .int("live", LIVE as u128)
            .frames(begin.get(), end)
            .finish();
        false
    });
    column((
        text(move || format!("frame {}", version.get())),
        scroll(paragraphs).style(style().grow(1).min_height(0)),
    ))
    .style(style().padding(12).gap(6).grow(1).min_height(0))
    .into_view()
}

/// Wrapped text and a grid of tiles, shown for 4 s while the runner
/// resizes the window (`MARK ready`); the frames in that window are
/// reported.
pub(crate) fn resize() -> View {
    let root = node_ref();
    let rect = root.rect();
    let layouts = Rc::new(Cell::new(0u32));
    let seen = layouts.clone();
    effect(move || {
        rect.get();
        seen.set(seen.get() + 1);
    });
    spawn(async move {
        sleep(Duration::from_millis(800)).await;
        let begin = mark("ready");
        sleep(Duration::from_secs(4)).await;
        let end = mark("phase_end");
        Json::new("resize")
            .frames(begin, end)
            .int("layout_reports", u128::from(layouts.get()))
            .finish();
    });
    let paragraphs: Vec<View> = (0..60)
        .map(|i| {
            text(paragraph_text(i))
                .style(style().text_wrap(true))
                .into_view()
        })
        .collect();
    let tiles: Vec<View> = (0..100)
        .map(|i| {
            node()
                .style(style().size(36, 36).radius(4).background(tile_color(i)))
                .into_view()
        })
        .collect();
    column((
        row(tiles).style(style().wrap(true).gap(6)),
        column(paragraphs).style(style().gap(4)),
    ))
    .node_ref(root)
    .style(style().padding(12).gap(8).grow(1).min_height(0).clip(true))
    .into_view()
}

/// Nothing changes for 3 s, then a focused text field for 3 s; the runner
/// samples CPU time at the marks.
pub(crate) fn idle() -> View {
    let field = node_ref();
    let value = signal(String::from("hostbench"));
    spawn(async move {
        sleep(Duration::from_secs(1)).await;
        let a = mark("idle_begin");
        sleep(Duration::from_secs(3)).await;
        let b = mark("idle_end");
        field.focus();
        sleep(Duration::from_millis(500)).await;
        let c = mark("caret_begin");
        sleep(Duration::from_secs(3)).await;
        let d = mark("caret_end");
        let log = frame_log();
        Json::new("idle")
            .int("frames_idle", log.between(a, b).len() as u128)
            .int("frames_caret", log.between(c, d).len() as u128)
            .finish();
    });
    let rows: Vec<View> = (0..200)
        .map(|i| {
            text(paragraph_text(i))
                .style(style().height(20).shrink(0))
                .into_view()
        })
        .collect();
    column((
        kivact::control::node()
            .name("Idle field")
            .part(kivact::interaction::use_text_edit(value).control())
            .node_ref(field),
        scroll(rows).style(style().grow(1).min_height(0)),
    ))
    .style(style().padding(12).gap(8).grow(1).min_height(0))
    .into_view()
}
