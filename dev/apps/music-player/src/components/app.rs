//! Role: Root component of the music player, which owns the player and frame loop.
//!
//! Root: owns the player, the loader action and the frame loop that
//! republishes the playback position while audio plays; the album art sits
//! beside the heading and the track title.

use super::cover::cover;
use super::spectrum::spectrum_view;
use super::transport::transport;
use super::waveform::waveform_view;
use crate::audio::{AudioClip, AudioPlayer, SharedClip};
use crate::player_state::PlayerCx;
use kivact::motion::FrameRate;
use kivact::motion::frame_loop;
use kivact::prelude::*;
use kivact::task::CancelToken;
use std::path::Path;
use std::sync::Arc;

/// The whole app.
pub fn player() -> impl IntoView {
    let player = signal(AudioPlayer::new(AudioClip::empty()));
    let loader = action(|path: Option<String>| async move {
        blocking(move |cancel: CancelToken| {
            let clip = match path {
                Some(path) => {
                    AudioClip::from_wav(Path::new(path.trim()), &|| cancel.is_cancelled())?
                }
                None => AudioClip::demo(),
            };
            Ok::<_, String>(SharedClip(Arc::new(clip)))
        })
        .await
        .map_err(UiError::msg)
    });
    loader.on_done(move |result| {
        if let Ok(clip) = result {
            player.update(|p| p.load(clip.0.clone()));
        }
    });
    loader.run(None);

    let clip = memo(move || player.with(|p| SharedClip(p.clip.clone())));
    let position = memo(move || player.with(AudioPlayer::position));
    let playing = memo(move || player.with(AudioPlayer::is_playing));
    // The rodio sink advances on its own thread: while playing, tell readers
    // of `player` that it changed, 30 times a second. Natural end flips
    // `playing` to false, which stops the loop.
    let frames = frame_loop(move |_| player.notify());
    effect(move || {
        if playing.get() {
            frames.start(FrameRate::Hz(30.0));
        } else {
            frames.stop();
        }
    });
    let state = PlayerCx {
        player,
        loader,
        wav_path: signal(String::new()),
        clip,
        position,
        playing,
    };
    let status = memo(move || match loader.state() {
        AsyncState::Pending => "Loading…".to_string(),
        _ if playing.get() => "Playing".to_string(),
        _ if player.with(|p| p.error.is_some()) => "Playback failed".to_string(),
        _ if position.get() > std::time::Duration::ZERO => "Paused".to_string(),
        _ => clip.with(|c| {
            format!(
                "Loaded {} · {} Hz · {} channel(s)",
                c.0.title, c.0.sample_rate, c.0.channels
            )
        }),
    });
    let error = memo(move || match loader.error() {
        Some(e) if !e.is_cancelled() => Some(e.to_string()),
        _ => player.with(|p| p.error.clone()),
    });

    provide(state, move || {
        scroll(
            column((
                row((
                    cover(),
                    column((
                        text("Music player · waveform and spectrum")
                            .style(style().font_size(20).weight(600))
                            .test_id("heading"),
                        text(move || clip.with(|c| c.0.title.clone())).test_id("track-title"),
                    ))
                    .style(style().gap(4)),
                ))
                .style(style().gap(12).align_items(Align::Center)),
                transport(),
                text(status).role(Role::Status).test_id("status"),
                show_some(error, |message| {
                    text(message)
                        .role(Role::Alert)
                        .style(style().color(ColorToken::Danger))
                        .test_id("player-error")
                }),
                text("Waveform · click or drag to seek (←/→ when focused)"),
                waveform_view(),
                text("Spectrum · frequency (Hz) × amplitude (0–1)"),
                spectrum_view(),
            ))
            .style(style().gap(8).padding(16)),
        )
        .style(style().grow(1))
    })
}
