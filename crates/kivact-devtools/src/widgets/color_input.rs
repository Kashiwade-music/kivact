//! Role: A color input with a text value, HSL sliders and a preview.

use kivact::prelude::*;

/// A hex text field, three HSL sliders and a preview swatch for one color.
pub(crate) fn color_input(
    label: &str,
    value: Signal<String>,
    on_commit: impl Fn(String) + 'static,
) -> impl IntoView {
    let label = label.to_owned();
    let (hue, saturation, lightness) = hsl_of(&value.get_untracked()).unwrap_or((0.0, 0.0, 0.0));
    let hue = signal(hue);
    let saturation = signal(saturation);
    let lightness = signal(lightness);
    let commit_text = callback(on_commit);
    let commit_hsl = callback(move |()| {
        let text = hsl_hex(
            hue.get_untracked(),
            saturation.get_untracked(),
            lightness.get_untracked(),
        );
        value.set(text.clone());
        commit_text.call(text);
    });
    let (hue_name, saturation_name, lightness_name) = (
        format!("{label} hue"),
        format!("{label} saturation"),
        format!("{label} lightness"),
    );
    let hex = text_input(label.clone(), value)
        .style(style().grow(1).min_width(0).font_size(12.0))
        .on_submit(move || {
            let text = value.get_untracked();
            if let Some((h, s, l)) = hsl_of(&text) {
                hue.set(h);
                saturation.set(s);
                lightness.set(l);
            }
            commit_text.call(text);
        });
    column((
        row((preview(&label, value), hex)).style(style().gap(6).align_items(Align::Center)),
        slider_row("H", hue_name, hue, 0.0..=360.0, commit_hsl),
        slider_row("S", saturation_name, saturation, 0.0..=100.0, commit_hsl),
        slider_row("L", lightness_name, lightness, 0.0..=100.0, commit_hsl),
    ))
    .style(style().gap(2).min_width(0))
}

/// One named slider whose commit reports the whole HSL color.
fn slider_row(
    letter: &'static str,
    name: String,
    value: Signal<f64>,
    range: std::ops::RangeInclusive<f64>,
    commit: Callback<()>,
) -> impl IntoView {
    row((
        text(letter).style(style().width(12).shrink(0).font_size(11.0)),
        slider(name, value)
            .range(range)
            .step(1.0)
            .style(style().grow(1).min_width(0))
            .on_edit(move |phase| {
                if phase == EditPhase::Commit {
                    commit.call(());
                }
            }),
    ))
    .style(style().gap(6).align_items(Align::Center))
}

/// A small box filled with the color the text names.
fn preview(label: &str, value: Signal<String>) -> impl IntoView {
    node().name(format!("{label} preview")).style(move || {
        let color = Color::css(value.get().trim()).unwrap_or(Color::TRANSPARENT);
        style()
            .size(18, 18)
            .shrink(0)
            .radius(4)
            .background(color)
            .border(1, ColorToken::Border)
    })
}

/// The hue in degrees and the saturation and lightness in percent.
fn hsl_of(text: &str) -> Option<(f64, f64, f64)> {
    let text = text.trim();
    let parsed = Color::css(text).or_else(|_| Color::css(&format!("#{text}")));
    let [r, g, b, _] = parsed.ok()?.components();
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let lightness = (max + min) / 2.0;
    let saturation = if delta == 0.0 {
        0.0
    } else {
        delta / (1.0 - (2.0 * lightness - 1.0).abs())
    };
    let hue = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * ((g - b) / delta).rem_euclid(6.0)
    } else if max == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };
    Some((hue.rem_euclid(360.0), saturation * 100.0, lightness * 100.0))
}

/// `#rrggbb` for the hue in degrees and the saturation and lightness in percent.
fn hsl_hex(hue: f64, saturation: f64, lightness: f64) -> String {
    let [r, g, b, _] = kivact::style::hsl(hue, saturation / 100.0, lightness / 100.0).components();
    let channel = |value: f64| (value * 255.0).round().clamp(0.0, 255.0) as u8;
    format!("#{:02x}{:02x}{:02x}", channel(r), channel(g), channel(b))
}
