//! Role: The page of the Creative catalog.

use super::audio::*;
use super::audio_values::TransportState;
use super::audio_values::level;
use super::common::*;
use super::common::{col, note};
use catalog_shell::{Example, Page};
use kivact::prelude::{Element, IntoView, rgb, row, scroll, signal};
use kivact::state::IntoValue;
use kivact::state::Signal;
use kivact::state::binding;
use kivact::style::style;
use kivact::view::ScrollAxes;
use kivact_ui_creative as ui;

pub(crate) fn page() -> Page {
    Page::new(
        "mixer-console",
        "Mixer Console",
        "Audio",
        include_str!("mixer_console.rs"),
    )
    .description("Routing, inserts, pan, mute and solo, and volume.")
    .example(Example::new("mixer-console", "Mixer Console", "", demo))
}

// example: mixer-console
fn demo() -> impl IntoView {
    let state = transport();
    let channels = [
        ("Kick", 0xff5c6c, -3.5),
        ("Snare", 0xff9b4a, -6.0),
        ("Bass", 0xffd25a, -4.8),
        ("Keys", 0xa397ff, -9.0),
        ("Pad", 0x5aa9ff, -14.0),
        ("Vox", 0x34d399, -2.2),
        ("FX Ret", 0x3ad3e0, -12.0),
        ("Stereo Out", 0xd6d6d6, 0.0),
    ];
    let solo = signal(vec![false; channels.len()]);
    let strips = channels
        .into_iter()
        .enumerate()
        .map(|(i, (name, color, gain))| {
            let gain = signal(gain);
            let mute = signal(false);
            ui::mixer_strip(
                name,
                rgb(color),
                ui::ChannelState {
                    gain,
                    mute,
                    pan: (signal([0.0, 8.0, 0.0, -35.0, 30.0, 0.0, 0.0, 0.0][i])),
                    solo: binding((move || solo.with(|s| s[i])).into_value(), move |v| {
                        solo.update(|s| s[i] = v)
                    }),
                },
            )
            .gain_range(-60.0..=6.0)
            .routes(
                ["Drum Bus", "In 1-2", "Main Out 1-2", "No Input"]
                    .into_iter()
                    .map(|item| item.to_string())
                    .collect::<Vec<_>>(),
            )
            .route(signal(if i < 2 {
                0
            } else if i == 7 {
                2
            } else {
                1
            }))
            .inserts(
                [
                    ["Gate", "EQ-4", "—", "—"],
                    ["EQ-4", "Comp", "Saturate", "—"],
                    ["Amp Sim", "Comp", "—", "—"],
                    ["Chorus", "—", "—", "—"],
                    ["Filter", "Reverb", "—", "—"],
                    ["De-Esser", "Comp", "EQ-8", "Limiter"],
                    ["Reverb", "—", "—", "—"],
                    ["Bus Comp", "Limiter", "—", "—"],
                ][i]
                    .into_iter()
                    .map(|item| item.to_string())
                    .collect::<Vec<_>>(),
            )
            .level(move || {
                if mute.get() || solo.with(|s| s.iter().any(|x| *x) && !s[i]) {
                    0.0
                } else {
                    level(state, i)() * 10_f64.powf(gain.get() / 20.0)
                }
            })
            .style(style().width(94))
        })
        .collect::<Vec<_>>();
    col((
        scroll(row(strips).style(style().gap(4).padding(8))).axes(ScrollAxes::Horizontal),
        note("Shows insert names. M / S mute and solo. Meters run during playback."),
    ))
    .style(style().padding_bottom(16))
}
// end example
