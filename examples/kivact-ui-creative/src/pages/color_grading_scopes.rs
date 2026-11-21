//! Role: The page of the Creative catalog.

use super::common::{col, line};
use super::controls::num;
use super::editor_images as images;
use super::editors::*;
use catalog_shell::{Example, Page};
use kivact::prelude::{Element, IntoView, memo, rgb, row, signal, text};
use kivact::state::Signal;
use kivact::style::{Color, pct, style};
use kivact_ui_creative as ui;

pub(crate) fn page() -> Page {
    Page::new(
        "color-grading-scopes",
        "Color Grading & Scopes",
        "Compositing",
        include_str!("color_grading_scopes.rs"),
    )
    .description("Lift, gamma, gain, parade, waveform and vectorscope.")
    .example(Example::new(
        "color-grading-scopes",
        "Color Grading & Scopes",
        "",
        demo,
    ))
}

// example: color-grading-scopes
fn demo() -> impl IntoView {
    let upper = signal(Some("parade".to_owned()));
    let lower = signal(Some("vectorscope".to_owned()));
    let worker_image = images::worker_gradient();
    let grade = signal(Grade::Graded);
    let bypass = grade_toggle(grade, Grade::Bypassed);
    let wipe = grade_toggle(grade, Grade::Wipe);
    let wheels = [
        signal(rgb(0x7a7a7a)),
        signal(rgb(0x909090)),
        signal(rgb(0xb4b4b4)),
        signal(rgb(0x8b7cff)),
    ];
    let samples = memo(move || {
        let tint = wheels[0]
            .get()
            .components(&kivact::style::use_theme().get());
        (0..4096)
            .map(|i| {
                let x = (i % 128) as f64 / 127.0;
                let y = (i / 128) as f64 / 31.0;
                let noise = ((i as u64 * 1664525 + 1013904223) % 65536) as f64 / 65535.0 - 0.5;
                let c = if y < 0.58 {
                    rgb(0x183961).mix(rgb(0xffb272), y / 0.58, &kivact::style::use_theme().get())
                } else {
                    rgb(0x4e94ae).mix(
                        rgb(0x053957),
                        (y - 0.58) / 0.42,
                        &kivact::style::use_theme().get(),
                    )
                };
                let c = if x < 0.22 && y > 0.45 {
                    rgb(0x279837)
                } else {
                    c
                };
                let mut c = c.components(&kivact::style::use_theme().get());
                if !bypass.get() {
                    for k in 0..3 {
                        c[k] = (c[k] + (tint[k] - 0.478) * 0.3 + noise * 0.10).clamp(0.0, 1.0);
                    }
                }
                Color::from_rgba_f64(c[0], c[1], c[2], 1.0)
            })
            .collect::<Vec<_>>()
    });
    col((
        row((
            grading_viewer(worker_image, bypass, wipe),
            grading_scopes(samples, upper, lower),
        ))
        .style(style().gap(1).height(772)),
        grade_wheels(wheels),
        line((
            num("Contrast", 1.0, 0.0, 4.0, 3, ""),
            num("Pivot", 0.435, 0.0, 1.0, 3, ""),
            num("Saturation", 1.0, 0.0, 2.0, 3, ""),
            num("Temperature", 0.0, -100.0, 100.0, 0, ""),
            num("Tint", 0.0, -100.0, 100.0, 0, ""),
        ))
        .style(style().padding(12)),
    ))
}
// end example

/// The four grading wheels (lift, gamma, gain, offset), each with its reset and channels.
fn grade_wheels(wheels: [Signal<Color>; 4]) -> impl IntoView {
    row(["Lift", "Gamma", "Gain", "Offset"]
        .into_iter()
        .enumerate()
        .map(|(i, label)| {
            col((
                line((
                    text(label).style(style().font_weight(600).flex_grow(1)),
                    kivact::ui::button("Reset")
                        .variant(kivact::ui::ButtonVariant::Secondary)
                        .on_press(move || wheels[i].set(rgb(0x808080))),
                )),
                ui::color_wheel(wheels[i], signal(ui::ColorModel::Rgb)).wheel_size(150.0),
                line((
                    channel(wheels[i], 0, "Grade R"),
                    channel(wheels[i], 1, "Grade G"),
                    channel(wheels[i], 2, "Grade B"),
                )),
            ))
            .style(style().width(pct(25)).padding(12))
            .style(
                style()
                    .border_top(1, ui::color::HAIR)
                    .border_right(1, ui::color::HAIR),
            )
        })
        .collect::<Vec<_>>())
}
