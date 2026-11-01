//! Role: File loading and the transport buttons.

use crate::audio::{AudioPlayer, format_time};
use crate::player_state::PlayerCx;
use kivact::prelude::*;

pub(super) fn transport() -> impl IntoView {
    let p = context::<PlayerCx>();
    let files = row((
        text_input("WAV file path", p.wav_path)
            .style(style().grow(1))
            .test_id("wav-path"),
        button("Load WAV")
            .on_press(move || p.loader.run(Some(p.wav_path.get())))
            .test_id("load-wav"),
        button("Use demo")
            .variant(ButtonVariant::Secondary)
            .on_press(move || p.loader.run(None))
            .test_id("load-demo"),
    ))
    .style(style().gap(8).align_items(Align::Center));
    let controls = row((
        button("Back 5 seconds")
            .variant(ButtonVariant::Secondary)
            .on_press(move || p.seek_by(-5))
            .test_id("seek-back"),
        button(move || if p.playing.get() { "Pause" } else { "Play" }.to_string())
            .on_press(move || p.player.update(AudioPlayer::toggle))
            .test_id("play-pause"),
        button("Stop")
            .variant(ButtonVariant::Secondary)
            .on_press(move || p.player.update(AudioPlayer::stop))
            .test_id("stop"),
        button("Forward 5 seconds")
            .variant(ButtonVariant::Secondary)
            .on_press(move || p.seek_by(5))
            .test_id("seek-forward"),
        text(move || {
            format!(
                "{} / {}  ·  {}",
                format_time(p.position.get()),
                p.clip.with(|c| format_time(c.0.duration())),
                if p.playing.get() { "Playing" } else { "Paused" }
            )
        })
        .test_id("position"),
    ))
    .style(style().gap(8).align_items(Align::Center));
    column((files, controls)).style(style().gap(8))
}
